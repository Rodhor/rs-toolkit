mod config;
mod io;
mod models;

use common::Logger;
use config::Config;
use io::list_files;

fn main() {
    let config: Config = common::config::load();
    let _logger = Logger::new(&config.logging);
    tracing::info!("{} starting", env!("CARGO_PKG_NAME"));

    let files = list_files(&config.settings);
    io::process_files(files, &config.settings);
}
