use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, PartialEq)]
pub enum NamingConvention {
    SheetName,
    Index,
}

#[derive(Debug, Deserialize, PartialEq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Deserialize, PartialEq)]
pub enum LogFormat {
    Txt,
    Json,
    StdOut,
}

pub trait TemplateSection {
    fn section_name() -> &'static str;
    fn template_body() -> &'static str;
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

impl TemplateSection for LoggingConfig {
    fn section_name() -> &'static str {
        "logging"
    }

    fn template_body() -> &'static str {
        r#"# Logging level. Options:
# - "Debug"
# - "Info"
# - "Warn"
# - "Error"
level = "Info"

# Log file path (optional). Leave unset or remove the line entirely to skip writing logs to a file.
# If you do want to write to a file provide either the folder or filepath.
# path = "./excelsplitter.log"

# Log format. Options:
# - StdOut: plain text to stdout only
# - Json: structured JSON, one object per line
# - Txt: plain text
format = "StdOut"
"#
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
            println!("No configfile found, creating one with defaults at {config_path}");
            if let Err(e) = Self::write_default_template(config_path) {
                println!("Failed to write default config file: {e}");
            }
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

    pub fn write_default_template(path: &str) -> std::io::Result<()> {
        let mut out = String::new();
        out.push_str("# ExcelSplitter configuration\n");
        out.push_str("# To reset back to defaults, simply delete this file. The default configuration will be recreated in the next run.\n\n");

        // Excel settings
        out.push_str(&format!("[{}]\n", ExcelConfig::section_name()));
        out.push_str(ExcelConfig::template_body());
        out.push('\n');

        // Logging settings
        out.push_str(&format!("[{}]\n", LoggingConfig::section_name()));
        out.push_str(LoggingConfig::template_body());

        fs::write(path, out)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::NamedTempFile;

    use super::*;
    #[test]
    fn default_template_is_valid_toml() {
        let file = NamedTempFile::new().expect("failed to create a temp file");
        let path_str = file.path().to_str().unwrap();

        Config::write_default_template(path_str).expect("failed to write template");

        let content = std::fs::read_to_string(path_str).expect("failed to read back the template");
        let _parsed: Config = toml::from_str(&content)
            .expect("generated default template is not valid TOML / doesn't match Config");
    }

    #[test]
    fn default_excel_template_matches_default_config() {
        let file = NamedTempFile::new().expect("failed to create a temp file");
        let path_str = file.path().to_str().unwrap();

        Config::write_default_template(path_str).expect("failed to write template");

        let content = std::fs::read_to_string(path_str).expect("failed to read back the template");
        let parsed: Config = toml::from_str(&content)
            .expect("generated default template is not valid TOML / doesn't match Config");

        assert_eq!(parsed.excel.min_rows, ExcelConfig::default().min_rows);
        assert_eq!(
            parsed.excel.exclude_sheets,
            ExcelConfig::default().exclude_sheets
        );
        assert_eq!(parsed.excel.naming, ExcelConfig::default().naming);
        assert_eq!(parsed.excel.output_dir, ExcelConfig::default().output_dir);
        assert_eq!(parsed.excel.input, ExcelConfig::default().input);
        assert_ne!(parsed.excel.output_dir, ExcelConfig::default().input);
        assert_ne!(parsed.excel.input, ExcelConfig::default().output_dir);
    }

    #[test]
    fn default_logging_template_matches_default_config() {
        let file = NamedTempFile::new().expect("failed to create a temp file");
        let path_str = file.path().to_str().unwrap();

        Config::write_default_template(path_str).expect("failed to write template");

        let content = std::fs::read_to_string(path_str).expect("failed to read back the template");
        let parsed: Config = toml::from_str(&content)
            .expect("generated default template is not valid TOML / doesn't match Config");

        assert_eq!(parsed.logging.format, LoggingConfig::default().format);
        assert_eq!(parsed.logging.level, LoggingConfig::default().level);
        assert_eq!(parsed.logging.path, LoggingConfig::default().path);
    }
}
