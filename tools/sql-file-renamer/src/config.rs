use std::path::PathBuf;

use common::TemplateSection;
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
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
    pub output_path: PathBuf,
    pub sort_field: SortField,
    pub gab: usize,
    pub width: usize,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            path: PathBuf::from("./"),
            overwrite_original: false,
            output_path: PathBuf::from("./"),
            sort_field: SortField::AlphabeticalAsc,
            gab: 50,
            width: 7,
        }
    }
}

impl TemplateSection for Settings {
    fn section_name() -> &'static str {
        "settings"
    }

    fn template_body() -> &'static str {
        r#"# Sort field for renaming files.
sort_field = "alphabetical_asc"

# Gap between file numbers.
gab = 50

# Width of file number padding.
width = 7
"#
    }
}

common::tool_config! {
    settings: Settings,
}

#[cfg(test)]
#[path = "config_test.rs"]
mod tests;
