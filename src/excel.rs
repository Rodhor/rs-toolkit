use std::{collections::HashMap, fs::File, io::BufReader};

use crate::config::{ExcelConfig, NamingConvention};

use calamine::{Reader, Xlsx, open_workbook};
pub struct ExcelData {
    sheets: Vec<String>,
    data: Vec<SheetData>,
}
struct SheetData {
    rows: Vec<RowData>,
}
struct RowData {
    col_data: Vec<String>,
}

impl ExcelData {
    pub fn from_sheet(cfg: &ExcelConfig) {
        // Open workbook returning the workbook object
        let mut workbook: Xlsx<_> =
            open_workbook(cfg.input_dir.clone()).expect("failed to open excel file");

        // Get a list of sheetnames in the workbook
        let sheet_names: Vec<String> = workbook.sheet_names();

        // Depending on the defined namingconvention, call the needed function returning a worksheet data object.
        match cfg.naming {
            NamingConvention::Index => create_index_map(sheet_names, &mut workbook),
            NamingConvention::SheetName => create_index_map(sheet_names, &mut workbook),
        };
    }

    pub fn from_multiple_sheets(cfg: &ExcelConfig) {}
}
fn create_index_map(
    names: Vec<String>,
    workbook: &mut Xlsx<BufReader<File>>,
) -> HashMap<usize, Vec<Vec<String>>> {
    let mut total_data = HashMap::new();
    for (i, name) in names.iter().enumerate() {
        let mut sheet_data = Vec::new();
        if let Ok(range) = workbook.worksheet_range(name) {
            for row in range.rows() {
                let mut row_data = Vec::new();
                for col in row.iter() {
                    row_data.push(col.to_string());
                    println!("This is the extracted data: {}", col)
                }
                sheet_data.push(row_data)
            }
        }
        total_data.insert(i, sheet_data);
    }
    total_data
}
