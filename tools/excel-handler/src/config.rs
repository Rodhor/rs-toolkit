use common::TemplateSection;
use fuzzy_derive::FuzzyFromStr;
use serde::Deserialize;
use serde_with::DeserializeFromStr;
use std::path::PathBuf;

#[derive(Debug, PartialEq, FuzzyFromStr, DeserializeFromStr)]
pub enum NamingConvention {
    SheetName,
    Index,
}

#[derive(Debug, PartialEq, FuzzyFromStr, DeserializeFromStr)]
pub enum CleanPreset {
    Tabs,
    Newlines,
    TrimWhitespace,
    ControlChars,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct CleaningConfig {
    pub presets: Vec<CleanPreset>,
    pub remove_strings: Vec<String>,
    pub remove_patterns: Vec<String>,
}

impl Default for CleaningConfig {
    fn default() -> Self {
        Self {
            presets: vec![CleanPreset::Tabs, CleanPreset::ControlChars],
            remove_strings: Vec::new(),
            remove_patterns: Vec::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct ExcelConfig {
    pub input: PathBuf,
    pub output_dir: PathBuf,
    pub exclude_sheets: Vec<String>,
    pub naming: NamingConvention,
    pub min_rows: usize,
    pub clean_data: bool,
    pub clean_config: CleaningConfig,
}

impl Default for ExcelConfig {
    fn default() -> Self {
        Self {
            input: PathBuf::from("./"),
            output_dir: PathBuf::from("./out"),
            exclude_sheets: vec!["Legende".to_string()],
            naming: NamingConvention::SheetName,
            min_rows: 3,
            clean_data: true,
            clean_config: CleaningConfig::default(),
        }
    }
}

impl TemplateSection for ExcelConfig {
    fn section_name() -> &'static str {
        "excel"
    }
    fn template_body() -> &'static str {
        r#"# Path to a single .xlsx file, or a directory containing one or more .xlsx files.
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

# Toggle as to weather the data should be cleaned or not
# By default this is set to true and will apply the cleaning steps defined next
# If set to false, no cleaning will happen, even if the cleaning is defined.
clean_data = true

# Configuration for datacleaning
# the cleaning config is setup by individual parts
[clean_config]

# Preset options:
# - Tabs
# - Newlines
# - TrimWhitespace
# - ControlChars
presets = ["Tabs", "ControlChars"]

# Strings to remove - Takes a list of strings with comma between
remove_strings = [""]

# Regex of patterns to remove - Takes a list of regex strings with comma between
remove_patterns = [""]
"#
    }
}

common::tool_config! {
    excel: ExcelConfig
}

#[cfg(test)]
#[path = "config_test.rs"]
mod tests;
