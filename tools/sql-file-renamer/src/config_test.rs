use super::*;
use common::ToolConfig;
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn default_template_matches_default_config() {
    let file = NamedTempFile::new().expect("failed to create temp file");
    fs::write(file.path(), Config::render_default_template()).expect("failed to write");

    let content = fs::read_to_string(file.path()).expect("failed to read back");
    let parsed: Config = toml::from_str(&content).expect("template is not valid TOML");

    assert_eq!(parsed.settings.example, Settings::default().example);
}
