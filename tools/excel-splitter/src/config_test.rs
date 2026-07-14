use super::*;
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn default_excel_template_matches_default_config() {
    let file = NamedTempFile::new().expect("failed to create a temp file");

    // render_default_template() comes free from the ToolConfig trait —
    // it replaces the old Config::write_default_template().
    fs::write(file.path(), Config::render_default_template()).expect("failed to write template");

    let content = fs::read_to_string(file.path()).expect("failed to read back the template");
    let parsed: Config = toml::from_str(&content)
        .expect("generated default template is not valid TOML / doesn't match Config");

    // The point of this test: the VALUES WRITTEN IN THE TEMPLATE COMMENTS
    // must agree with the Rust Default impl. They're maintained by hand in
    // two places, so they will drift eventually — this catches it.
    assert_eq!(parsed.excel.min_rows, ExcelConfig::default().min_rows);
    assert_eq!(
        parsed.excel.exclude_sheets,
        ExcelConfig::default().exclude_sheets
    );
    assert_eq!(parsed.excel.naming, ExcelConfig::default().naming);
    assert_eq!(parsed.excel.output_dir, ExcelConfig::default().output_dir);
    assert_eq!(parsed.excel.input, ExcelConfig::default().input);

    // Guards against a copy-paste swap of input/output_dir in the template.
    assert_ne!(parsed.excel.output_dir, ExcelConfig::default().input);
    assert_ne!(parsed.excel.input, ExcelConfig::default().output_dir);
}
