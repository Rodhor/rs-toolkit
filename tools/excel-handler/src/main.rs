mod config;
mod excel;

use common::Logger;
use config::Config;
use excel::ExcelData;

fn main() {
    // Load in config
    let config: Config = common::config::load();
    // Initialise logging
    let _logger = Logger::new(&config.logging);
    // Load excelfiles into data. Data might contain several excelfiles
    let data = ExcelData::load(&config.excel);
    // Print a report from the loaded excelfiles - giving the user some information about the success/failures of loading
    println!("{}", data.report());

    // Loop through all excelfiles and export to csv
    let mut export_failed = false;
    for excel_file in &data.data {
        tracing::info!("exporting file: {:?}", excel_file.source);

        if let Some(file_name) = excel_file.source.file_stem() {
            let file_output_dir = config.excel.output_dir.join(file_name);

            if let Err(e) = std::fs::create_dir_all(&file_output_dir) {
                tracing::error!("failed to create directory {:?}: {:?}", file_output_dir, e);
                export_failed = true;
                continue;
            }
            match excel_file.to_csv(
                &file_output_dir,
                &config.excel.naming,
                config.excel.min_rows,
            ) {
                Ok(_) => {
                    tracing::info!(
                        "export to csv successful for {:?}. Exported to: {:?}",
                        excel_file.source,
                        file_output_dir,
                    );
                }
                Err(e) => {
                    tracing::error!("failed to export {:?} to csv: {:?}", excel_file.source, e);
                    export_failed = true;
                }
            }
        }

        if export_failed {
            tracing::warn!("some files failed to export due to disk/IO errors");
        }
    }
}
