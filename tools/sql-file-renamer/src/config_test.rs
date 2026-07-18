use super::*;
use common::ToolConfig;
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn default_settings_have_expected_values() {
    let settings = Settings::default();

    assert_eq!(settings.path, PathBuf::from("./"));
    assert!(!settings.overwrite_original);
    assert_eq!(settings.output_path, PathBuf::from("./out"));
    assert_eq!(settings.sort_field, SortField::AlphabeticalAsc);
    assert_eq!(settings.gab, 20);
    assert_eq!(settings.width, 5);
}

fn parse_settings(body: &str) -> Settings {
    toml::from_str(body).expect("settings snippet should be valid TOML")
}

#[test]
fn sort_field_parses_every_variant_by_name() {
    let cases = [
        ("AlphabeticalAsc", SortField::AlphabeticalAsc),
        ("AlphabeticalDesc", SortField::AlphabeticalDesc),
        ("ModifiedTimeNewest", SortField::ModifiedTimeNewest),
        ("ModifiedTimeOldest", SortField::ModifiedTimeOldest),
        ("CreatedTimeNewest", SortField::CreatedTimeNewest),
        ("CreatedTimeOldest", SortField::CreatedTimeOldest),
        ("AccessedTimeNewest", SortField::AccessedTimeNewest),
        ("AccessedTimeOldest", SortField::AccessedTimeOldest),
    ];

    for (name, expected) in cases {
        let settings = parse_settings(&format!("sort_field = \"{name}\""));
        assert_eq!(
            settings.sort_field, expected,
            "variant {name} failed to parse"
        );
    }
}

#[test]
fn sort_field_parses_fuzzy_spellings() {
    let cases = [
        ("alphabetical_asc", SortField::AlphabeticalAsc),
        ("ALPHABETICAL-ASC", SortField::AlphabeticalAsc),
        ("alphabetical asc", SortField::AlphabeticalAsc),
        ("modifiedtimenewest", SortField::ModifiedTimeNewest),
        ("Accessed_Time_Oldest", SortField::AccessedTimeOldest),
    ];

    for (spelling, expected) in cases {
        let settings = parse_settings(&format!("sort_field = \"{spelling}\""));
        assert_eq!(
            settings.sort_field, expected,
            "spelling {spelling} failed to parse"
        );
    }
}

#[test]
fn unknown_sort_field_is_rejected() {
    let result = toml::from_str::<Settings>("sort_field = \"NotARealVariant\"");
    assert!(
        result.is_err(),
        "an unknown sort_field should fail to parse"
    );
}

#[test]
fn missing_fields_fall_back_to_defaults() {
    let settings = parse_settings("width = 3");

    assert_eq!(settings.width, 3);
    assert_eq!(settings.gab, 20);
    assert_eq!(settings.sort_field, SortField::AlphabeticalAsc);
    assert_eq!(settings.path, PathBuf::from("./"));
}

#[test]
fn empty_snippet_yields_full_defaults() {
    let settings = parse_settings("");

    let default = Settings::default();
    assert_eq!(settings.gab, default.gab);
    assert_eq!(settings.width, default.width);
    assert_eq!(settings.sort_field, default.sort_field);
}

#[test]
fn explicit_values_override_defaults() {
    let settings = parse_settings(
        r#"path = "/data/in"
output_path = "/data/out"
overwrite_original = true
sort_field = "ModifiedTimeNewest"
gab = 10
width = 4
"#,
    );

    assert_eq!(settings.path, PathBuf::from("/data/in"));
    assert_eq!(settings.output_path, PathBuf::from("/data/out"));
    assert!(settings.overwrite_original);
    assert_eq!(settings.sort_field, SortField::ModifiedTimeNewest);
    assert_eq!(settings.gab, 10);
    assert_eq!(settings.width, 4);
}

#[test]
fn default_template_declares_both_sections() {
    let template = Config::render_default_template();

    assert!(
        template.contains("[settings]"),
        "missing [settings] section"
    );
    assert!(template.contains("[logging]"), "missing [logging] section");
    assert!(
        template.contains("sql-file-renamer"),
        "missing tool name header"
    );
}

#[test]
fn config_default_matches_settings_default() {
    let config = Config::default();

    assert_eq!(config.settings.gab, Settings::default().gab);
    assert_eq!(config.settings.width, Settings::default().width);
    assert_eq!(config.settings.sort_field, Settings::default().sort_field);
}

#[test]
fn rendered_template_can_be_written_and_read_back() {
    let file = NamedTempFile::new().expect("failed to create temp file");
    fs::write(file.path(), Config::render_default_template()).expect("failed to write");

    let content = fs::read_to_string(file.path()).expect("failed to read back");
    assert!(content.contains("[settings]"));
    assert!(content.contains("[logging]"));
}

#[test]
fn default_template_round_trips() {
    let template = Config::render_default_template();
    let parsed: Config =
        toml::from_str(&template).expect("default template should parse back into Config");

    assert_eq!(parsed.settings.sort_field, Settings::default().sort_field);
    assert_eq!(parsed.settings.gab, Settings::default().gab);
    assert_eq!(parsed.settings.width, Settings::default().width);
}
