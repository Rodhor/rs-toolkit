use super::*;
use tempfile::NamedTempFile;

#[derive(Debug, Deserialize, PartialEq)]
#[serde(default)]
struct DummySection {
    greeting: String,
    retries: u32,
}
impl Default for DummySection {
    fn default() -> Self {
        Self {
            greeting: "hello".to_string(),
            retries: 3,
        }
    }
}

impl TemplateSection for DummySection {
    fn section_name() -> &'static str {
        "dummy"
    }

    fn template_body() -> &'static str {
        r#"# What the tool says on startup.
greeting = "hello"

# How many times to retry.
retries = 3
"#
    }
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct DummyConfig {
    dummy: DummySection,
    logging: LoggingConfig,
}

impl ToolConfig for DummyConfig {
    fn tool_name() -> &'static str {
        "dummy-tool"
    }

    fn sections() -> Vec<Section> {
        vec![section::<DummySection>(), section::<LoggingConfig>()]
    }
}

#[test]
fn rendered_template_is_valid_toml_and_round_trips_to_defaults() {
    let file = NamedTempFile::new().expect("failed to create temp file");
    fs::write(file.path(), DummyConfig::render_default_template())
        .expect("failed to write template");

    let cfg: DummyConfig = load_from(file.path());

    assert_eq!(cfg.dummy.greeting, DummySection::default().greeting);
    assert_eq!(cfg.dummy.retries, DummySection::default().retries);
    assert_eq!(cfg.logging.level, LoggingConfig::default().level);
    assert_eq!(cfg.logging.format, LoggingConfig::default().format);
}

#[test]
fn missing_file_is_created_with_a_template() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let path = dir.path().join("config.toml");
    assert!(!path.exists());

    let cfg: DummyConfig = load_from(&path);

    assert!(
        path.exists(),
        "load() should have written a default template"
    );
    assert_eq!(cfg.dummy.retries, DummySection::default().retries);

    let written = fs::read_to_string(&path).expect("failed to read back");
    assert!(written.starts_with("# dummy-tool configuration"));
    assert!(written.contains("[dummy]"));
    assert!(written.contains("[logging]"));
}

#[test]
fn sections_appear_in_the_order_declared() {
    let rendered = DummyConfig::render_default_template();
    let dummy_at = rendered.find("[dummy]").expect("no [dummy] section");
    let logging_at = rendered.find("[logging]").expect("no [logging] section");

    // ToolConfig::sections() listed dummy first, so it must render first.
    assert!(dummy_at < logging_at);
}

#[test]
fn garbage_config_falls_back_to_defaults() {
    let file = NamedTempFile::new().expect("failed to create temp file");
    fs::write(file.path(), "this is not valid toml {{{").expect("failed to write");

    let cfg: DummyConfig = load_from(file.path());

    assert_eq!(cfg.dummy.greeting, DummySection::default().greeting);
}
