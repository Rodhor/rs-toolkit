use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
pub enum NamingConvention {
    SheetName,
    Index,
}

#[derive(Debug, Deserialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Deserialize)]
pub enum LogFormat {
    Txt,
    Json,
    StdOut,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct ExcelConfig {
    pub input: PathBuf,
    pub output_dir: PathBuf,
    pub exclude_sheets: Vec<String>,
    pub naming: NamingConvention,
}

impl Default for ExcelConfig {
    fn default() -> Self {
        Self {
            input: PathBuf::from("./"),
            output_dir: PathBuf::from("./out"),
            exclude_sheets: vec!["Legende".to_string()],
            naming: NamingConvention::SheetName,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct LoggingConfig {
    pub level: LogLevel,
    pub path: Option<String>,
    pub format: LogFormat,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            path: None,
            format: LogFormat::StdOut,
        }
    }
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    pub excel: ExcelConfig,
    pub logging: LoggingConfig,
}

impl Config {
    pub fn new() -> Self {
        let config_path = "./config.toml";
        if !Path::new(config_path).exists() {
            println!("No configfile found, using all defaults!");
            return Config::default();
        }
        let content = fs::read_to_string(config_path).unwrap_or_else(|err| {
            println!("Error reading config file: {}", err);
            println!("Using default config instead!");
            String::new()
        });
        toml::from_str(&content).unwrap_or_else(|err| {
            println!("Error parsing config: {}", err);
            Config::default()
        })
    }
}
