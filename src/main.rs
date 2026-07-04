mod config;
mod excel;
mod logger;
use config::Config;
use excel::ExcelData;
use logger::Logger;

fn main() {
    let config = Config::new();
    let _logger = Logger::new(&config.logging);
    println!("{:#?}", config);
    tracing::info!("This works!");
    let data = ExcelData::new(&config.excel);
    println!("{}", data.report())
}
