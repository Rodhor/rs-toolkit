use serde::Deserialize;
use serde::de::DeserializeOwned;
use std::fs;
use std::path::Path;

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
# path = "./logs/"

# Log format. Options:
# - StdOut: plain text to stdout only
# - Json: structured JSON, one object per line
# - Txt: plain text
format = "StdOut"
"#
    }
}
pub trait TemplateSection {
    fn section_name() -> &'static str;
    fn template_body() -> &'static str;
}

#[derive(Debug, Clone, Copy)]
pub struct Section {
    pub name: &'static str,
    pub body: &'static str,
}

pub fn section<T: TemplateSection>() -> Section {
    Section {
        name: T::section_name(),
        body: T::template_body(),
    }
}

pub trait ToolConfig: DeserializeOwned + Default {
    fn tool_name() -> &'static str;
    fn sections() -> Vec<Section>;
    fn render_default_template() -> String {
        let mut out = format!("# {} configuration\n", Self::tool_name());
        out.push_str(
            "#To reset back to defaults, simply delete this file. \
            The default configuration will be recreated on next run.\n\n",
        );

        for s in Self::sections() {
            out.push_str(&format!("[{}]\n", s.name));
            out.push_str(s.body);
            out.push_str("\n");
        }
        out
    }
}

pub const DEFAULT_CONFIG_PATH: &str = "./config.toml";
pub fn load<C: ToolConfig>() -> C {
    load_from::<C>(DEFAULT_CONFIG_PATH)
}

pub fn load_from<C: ToolConfig>(path: impl AsRef<Path>) -> C {
    let path = path.as_ref();

    if !path.exists() {
        println!(
            "No config file found at {}, using default config, creating one with defaults at {}",
            path.display(),
            path.display()
        );
        if let Err(e) = fs::write(path, C::render_default_template()) {
            println!("Failed to write default config file: {e}");
        }
        return C::default();
    }

    match fs::read_to_string(path) {
        Ok(content) => toml::from_str(&content).unwrap_or_else(|err| {
            println!("Failed to parse config file: {err}");
            println!("Using default config instead!");
            C::default()
        }),
        Err(err) => {
            println!("Error reading config file: {err}");
            println!("Using default config instead!");
            C::default()
        }
    }
}
#[cfg(test)]
#[path = "config_test.rs"]
mod tests;
