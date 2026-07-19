use std::path::PathBuf;

use common::TemplateSection;
use fuzzy_derive::FuzzyFromStr;
use serde::Deserialize;
use serde_with::DeserializeFromStr;

#[derive(Debug, DeserializeFromStr, FuzzyFromStr, PartialEq)]
pub enum SortField {
    AlphabeticalAsc,
    AlphabeticalDesc,
    ModifiedTimeNewest,
    ModifiedTimeOldest,
    CreatedTimeNewest,
    CreatedTimeOldest,
    AccessedTimeNewest,
    AccessedTimeOldest,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub path: PathBuf,
    pub overwrite_original: bool,
    pub sort_field: SortField,
    pub include_pattern: String,
    pub exclude_pattern: String,
    pub gab: usize,
    pub width: usize,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            path: PathBuf::from("./"),
            overwrite_original: false,
            sort_field: SortField::AlphabeticalAsc,
            include_pattern: String::from(""),
            exclude_pattern: String::from(""),
            gab: 20,
            width: 5,
        }
    }
}

impl TemplateSection for Settings {
    fn section_name() -> &'static str {
        "settings"
    }

    fn template_body() -> &'static str {
        r#"#The Path to the directory containing files to be renamed.
path = "./"

# Overwrite original files:
# if true, the original files will be overwritten with the renamed ones.
# if false, original files will be moved into a timestamped directory.
overwrite_original = false

# Sort field for renaming files. Options:
# - alphabetical_asc: Sort by alphabetical order, ascending
# - alphabetical_desc: Sort by alphabetical order, descending
# - modified_time_newest: Sort by modified time, newest first
# - modified_time_oldest: Sort by modified time, oldest first
# - created_time_newest: Sort by created time, newest first
# - created_time_oldest: Sort by created time, oldest first
# - accessed_time_newest: Sort by accessed time, newest first
# - accessed_time_oldest: Sort by accessed time, oldest first
sort_field = "alphabetical_asc"

# Include pattern - which filetypes to include, with the use of Wildcard - i.e. "*.sql"
include_pattern = ""

# Exclude pattern - which filetypes to exclude, with the use for wildcarts - i.e. "00*"
exclude_pattern = ""

# Gap between file numbers.
gab = 20

# Width of file number padding - i.e. 00001_file
width = 5
"#
    }
}

common::tool_config! {
    settings: Settings,
}

#[cfg(test)]
#[path = "config_test.rs"]
mod tests;
