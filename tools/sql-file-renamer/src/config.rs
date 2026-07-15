use common::TemplateSection;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub example: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            example: "change me".to_string(),
        }
    }
}

impl TemplateSection for Settings {
    fn section_name() -> &'static str {
        "settings"
    }

    fn template_body() -> &'static str {
        r#"# An example setting. Replace with your tool's real config.
example = "change me"
"#
    }
}

common::tool_config! {
    settings: Settings,
}

#[cfg(test)]
#[path = "config_test.rs"]
mod tests;
