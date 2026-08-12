use crate::config::{CleanPreset, CleaningConfig, ExcelConfig, NamingConvention};
use calamine::{Reader, Xlsx, XlsxError, open_workbook};
use csv::Writer;
use regex::Regex;
use std::fmt::Write;
use std::path::Path;
use std::{
    fs::{self, File},
    io::BufReader,
    path::PathBuf,
};

pub struct LoadResult {
    pub data: Vec<ExcelData>,
    pub errors: Vec<PathBuf>,
}

impl LoadResult {
    pub fn report(&self) -> String {
        let mut report = String::new();

        writeln!(report, "{}", "=".repeat(50)).unwrap();
        writeln!(report, "Processing report").unwrap();
        writeln!(report, "{}", "=".repeat(50)).unwrap();
        writeln!(report, "Files processed: {}", self.data.len()).unwrap();
        writeln!(report, "Files failed:    {}", self.errors.len()).unwrap();
        report.push('\n');

        for excel_data in &self.data {
            report.push_str(&excel_data.report());
        }

        if !self.errors.is_empty() {
            writeln!(report, "{}", "-".repeat(50)).unwrap();
            writeln!(report, "Failed paths:").unwrap();
            for path in &self.errors {
                writeln!(report, "  - {:?}", path).unwrap();
            }
        }

        report
    }
}

// Helper struct to define cleaning and load/check regex patterns
pub struct CellCleaner<'a> {
    config: &'a CleaningConfig,
    regexes: Vec<Regex>,
}

impl<'a> CellCleaner<'a> {
    fn new(config: &'a CleaningConfig) -> Self {
        let regexes = config
            .remove_patterns
            .iter()
            .filter(|pattern| !pattern.is_empty())
            .filter_map(|pattern| match Regex::new(pattern) {
                Ok(regex) => Some(regex),
                Err(err) => {
                    tracing::warn!("invalid regex pattern '{}' skipped: {:?}", pattern, err);
                    None
                }
            })
            .collect();
        Self { config, regexes }
    }

    fn clean_cell(&self, cell: &mut String) -> bool {
        let raw_value = cell.clone();

        for preset in &self.config.presets {
            match preset {
                CleanPreset::Tabs => *cell = cell.replace('\t', ""),
                CleanPreset::Newlines => *cell = cell.replace(['\n', '\r'], ""),
                CleanPreset::TrimWhitespace => *cell = cell.trim().to_string(),
                CleanPreset::ControlChars => cell.retain(|c| !c.is_control()),
            }
        }

        for target in &self.config.remove_strings {
            if !target.is_empty() {
                *cell = cell.replace(target, "")
            }
        }

        for re in &self.regexes {
            *cell = re.replace_all(cell, "").to_string();
        }

        *cell != raw_value
    }
}

pub struct ExcelData {
    pub source: PathBuf,
    data: Vec<SheetData>,
    pub sheet_errors: Vec<(String, XlsxError)>,
}
struct SheetData {
    name: String,
    rows: Vec<RowData>,
    cleaned_cells: usize,
}
struct RowData {
    col_data: Vec<String>,
}

impl ExcelData {
    pub fn load(cfg: &ExcelConfig) -> LoadResult {
        let mut result = LoadResult {
            data: Vec::new(),
            errors: Vec::new(),
        };
        let cleaner = if cfg.clean_data {
            Some(CellCleaner::new(&cfg.clean_config))
        } else {
            None
        };

        if cfg.input.is_file() {
            tracing::info!(
                "file detected - initalising file processing: {:?}",
                cfg.input
            );
            match Self::from_file(&cfg.input, &cfg.exclude_sheets, cleaner.as_ref()) {
                Some(d) => result.data.push(d),
                None => {
                    tracing::error!("failed to process file: {:?}", cfg.input);
                    result.errors.push(cfg.input.clone());
                }
            }
        } else if cfg.input.is_dir() {
            tracing::info!(
                "directory detected - initalising directory processing: {:?}",
                cfg.input
            );
            match Self::from_dir(&cfg.input, &cfg.exclude_sheets, cleaner.as_ref()) {
                Some(dir_result) => {
                    result.data = dir_result.data;
                    result.errors = dir_result.errors;
                }
                None => {
                    tracing::error!("failed to read directory: {:?}", cfg.input);
                    result.errors.push(cfg.input.clone());
                }
            }
        } else {
            panic!(
                "input path is neither a file nor a directory: {:?}",
                cfg.input
            );
        }

        result
    }

    pub fn from_file(
        input: &PathBuf,
        skip: &[String],
        cleaner: Option<&CellCleaner>,
    ) -> Option<ExcelData> {
        if input.extension().is_none_or(|ext| ext != "xlsx") {
            tracing::error!("a none excelfile was passed int: {:?}", input);
            return None;
        }
        // Open workbook returning the workbook object
        tracing::info!("opening workbook: {:?}", input);
        let mut workbook: Xlsx<_> = match open_workbook(input) {
            Ok(wb) => wb,
            Err(e) => {
                tracing::error!("failed to open excel file {:?}: {:?}", input, e);
                return None;
            }
        };

        // Get a list of sheetnames in the workbook
        let sheet_names: Vec<String> = workbook.sheet_names();
        tracing::info!("sheets found: {:?}", sheet_names);

        Some(Self::build_data(
            input.clone(),
            sheet_names,
            skip,
            &mut workbook,
            cleaner,
        ))
    }

    pub fn from_dir(
        input: &PathBuf,
        skip: &[String],
        cleaner: Option<&CellCleaner>,
    ) -> Option<LoadResult> {
        // Try to laod the directory, failing fast if not possible
        let entries = match fs::read_dir(input) {
            Ok(entries) => entries,
            Err(e) => {
                tracing::error!("failed to read directory {:?}: {:?}", input, e);
                return None;
            }
        };

        // Reuse LoadResult to keep track of failed files
        let mut result = LoadResult {
            data: Vec::new(),
            errors: Vec::new(),
        };

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(e) => {
                    tracing::warn!("failed to read directory entry: {:?}", e);
                    continue;
                }
            };
            tracing::info!("processing: {:?}", entry);
            // Skip non excel files
            if entry.path().extension().is_none_or(|ext| ext != "xlsx") {
                tracing::info!(
                    "skipping entry, as this is not an excelfile: {:?}",
                    entry.path()
                );
                continue;
            }
            // Open workbook returning the workbook object
            let mut workbook: Xlsx<_> = match open_workbook(entry.path()) {
                Ok(wb) => wb,
                Err(e) => {
                    tracing::error!("failed to open excel file {:?}: {:?}", entry.path(), e);
                    result.errors.push(entry.path());
                    continue;
                }
            };

            // Get a list of sheetnames in the workbook
            let sheet_names: Vec<String> = workbook.sheet_names();
            tracing::info!("Sheets found: {:?}", sheet_names);

            result.data.push(Self::build_data(
                entry.path(),
                sheet_names,
                skip,
                &mut workbook,
                cleaner,
            ));
        }

        Some(result)
    }

    fn build_data(
        source: PathBuf,
        names: Vec<String>,
        skip: &[String],
        workbook: &mut Xlsx<BufReader<File>>,
        cleaner: Option<&CellCleaner>,
    ) -> ExcelData {
        let mut data = ExcelData {
            source,
            data: Vec::new(),
            sheet_errors: Vec::new(),
        };

        for name in &names {
            if skip.contains(&name.to_string()) {
                continue;
            }
            // Prepare sheet data
            let mut sheet_data = SheetData {
                name: name.clone(),
                rows: Vec::new(),
                cleaned_cells: 0,
            };
            // Loop through the sheet - row by row. (Skip if the sheet range is not found.)
            match workbook.worksheet_range(name) {
                Ok(range) => {
                    sheet_data.rows = range
                        .rows()
                        .map(|row| {
                            let col_data = row
                                .iter()
                                .map(|col| {
                                    let mut val = col.to_string();
                                    if let Some(c) = cleaner
                                        && c.clean_cell(&mut val)
                                    {
                                        sheet_data.cleaned_cells += 1;
                                    }
                                    val
                                })
                                .collect();
                            RowData { col_data }
                        })
                        .collect();

                    if sheet_data.cleaned_cells > 0 {
                        tracing::info!(
                            "sheet {:?}: cleaned {} cells",
                            sheet_data.name,
                            sheet_data.cleaned_cells
                        );
                    }
                    data.data.push(sheet_data);
                }
                Err(e) => {
                    tracing::warn!("skipping sheet {:?}: {:?}", name, e);
                    data.sheet_errors.push((name.clone(), e));
                }
            }
        }
        data
    }

    pub fn to_csv(
        &self,
        output_dir: &Path,
        naming: &NamingConvention,
        min_rows: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for (idx, sheet) in self.data.iter().enumerate() {
            if sheet.rows.len() < min_rows {
                tracing::warn!(
                    "sheet {:?} skipped because it only contains {} rows",
                    &sheet.name,
                    sheet.rows.len()
                );
                continue;
            } else {
                tracing::info!("creating csv from {:?}", &sheet.name)
            }

            let file_path = match naming {
                NamingConvention::Index => output_dir.join(format!("{}.csv", idx)),
                NamingConvention::SheetName => output_dir.join(format!("{}.csv", sheet.name)),
            };
            tracing::info!("saving sheet to path: {:?}", &file_path);
            let file = File::create(&file_path)?;

            let mut wtr = Writer::from_writer(file);
            for (counter, row) in sheet.rows.iter().enumerate() {
                match wtr.write_record(&row.col_data) {
                    Ok(_) => {
                        tracing::debug!("row {} successfully written.", counter)
                    }
                    Err(e) => {
                        tracing::warn!("error writing row {}: {:?}", counter, e)
                    }
                }
            }
            match wtr.flush() {
                Ok(_) => {}
                Err(e) => {
                    tracing::warn!("Error flushing writer: {:?}", e)
                }
            }
        }
        Ok(())
    }
    pub fn report(&self) -> String {
        let mut report = String::new();

        let total_cleaned: usize = self.data.iter().map(|s| s.cleaned_cells).sum();

        writeln!(report, "{}", "-".repeat(50)).unwrap();
        writeln!(report, "File: {:?}", self.source).unwrap();
        writeln!(report, "  Sheets processed: {}", self.data.len()).unwrap();
        writeln!(report, "  Sheets failed:    {}", self.sheet_errors.len()).unwrap();
        if total_cleaned > 0 {
            writeln!(report, "  Total cells cleaned: {}", total_cleaned).unwrap();
        }

        if !self.data.is_empty() {
            report.push('\n');
            for sheet in &self.data {
                let clean_info = if sheet.cleaned_cells > 0 {
                    format!(" ({} cells cleaned)", sheet.cleaned_cells)
                } else {
                    String::new()
                };
                writeln!(
                    report,
                    "    {:<40} {:>6} rows{}",
                    format!("{:?}", sheet.name),
                    sheet.rows.len(),
                    clean_info
                )
                .unwrap();
            }
        }

        if !self.sheet_errors.is_empty() {
            report.push('\n');
            writeln!(report, "  Errors:").unwrap();
            for (name, err) in &self.sheet_errors {
                writeln!(report, "    {:<40} {:?}", format!("{:?}", name), err).unwrap();
            }
        }

        report
    }
}
