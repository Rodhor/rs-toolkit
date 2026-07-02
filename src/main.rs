mod config;
mod logger;
use config::Config;
use logger::Logger;

fn main() {
    let config = Config::new();
    let _logger = Logger::new(&config.logging);
    println!("{:#?}", config);
    tracing::info!("This works!");
}
