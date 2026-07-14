use common::{LoggingConfig, Section, TemplateSection, ToolConfig, section};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize, PartialEq)]
pub enum NamingConvention {
    SheetName,
    Index,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct ExcelConfig {
    pub input: PathBuf,
    pub output_dir: PathBuf,
    pub exclude_sheets: Vec<String>,
    pub naming: NamingConvention,
    pub min_rows: usize,
}

impl Default for ExcelConfig {
    fn default() -> Self {
        Self {
            input: PathBuf::from("./"),
            output_dir: PathBuf::from("./out"),
            exclude_sheets: vec!["Legende".to_string()],
            naming: NamingConvention::SheetName,
            min_rows: 3,
        }
    }
}

impl TemplateSection for ExcelConfig {
    fn section_name() -> &'static str {
        "excel"
    }
    fn template_body() -> &'static str {
        r#"# Path to a single .xlsx file, or a diretory containing one or more .xlsx files.
input = "./"

# Where processed output (CSV) will be written.
output_dir = "./out"

# Sheet names to exclude from processing - written as a list of strings.
exclude_sheets = ["Legende"]

# How sheets are name in the output. Options:
# - "SheetName": use the sheet's name as-is
# - "Index": use the sheet's index (1-based)
naming = "SheetName"

# Minimum number of rows a sheet must have to be processed.
# Sheets with fewer rows will be skipped.
min_rows = 3
"#
    }
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    pub excel: ExcelConfig,
    pub logging: LoggingConfig,
}

impl ToolConfig for Config {
    fn tool_name() -> &'static str {
        "excel-splitter"
    }

    fn sections() -> Vec<Section> {
        vec![section::<ExcelConfig>(), section::<LoggingConfig>()]
    }
}

#[cfg(test)]
#[path = "config_test.rs"]
mod tests;
