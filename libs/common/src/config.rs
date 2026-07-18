use fuzzy_derive::FuzzyFromStr;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_with::DeserializeFromStr;

use std::fs;
use std::path::Path;

#[derive(Debug, DeserializeFromStr, PartialEq, FuzzyFromStr)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, PartialEq, FuzzyFromStr, DeserializeFromStr)]
pub enum LogFormat {
    Txt,
    Json,
}

#[derive(Debug, DeserializeFromStr, PartialEq, Clone)]
pub enum LogTarget {
    Stdout,
    Stderr,
    Path(String),
}

impl std::str::FromStr for LogTarget {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(
            match s
                .trim()
                .to_lowercase()
                .replace(['-', ' ', '_'], "")
                .as_str()
            {
                "stdout" => LogTarget::Stdout,
                "stderr" => LogTarget::Stderr,
                _ => LogTarget::Path(s.trim().to_string()),
            },
        )
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct LoggingConfig {
    pub level: LogLevel,
    pub target: LogTarget,
    pub format: LogFormat,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            target: LogTarget::Stdout,
            format: LogFormat::Txt,
        }
    }
}

impl TemplateSection for LoggingConfig {
    fn section_name() -> &'static str {
        "logging"
    }

    fn template_body() -> &'static str {
        r#"# Logging level. Options: "Debug", "Info", "Warn", "Error"
level = "Info"

# Log target. Options:
# - "Stdout"
# - "Stderr"
# - Any file path string (e.g., "./logs/", "app.log")
target = "Stdout"

# Log format. Options: "Txt", "Json"
format = "Txt"
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
            out.push('\n');
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

// Macro for defining a tool's configuration struct and implementing ToolConfig
#[macro_export]
macro_rules! tool_config {
    ( $( $field:ident : $section:ty ),* $(,)? ) => {
        #[derive(Debug, ::serde::Deserialize, Default)]
        #[serde(default)]
        pub struct Config {
            $( pub $field: $section, )*
            pub logging: $crate::LoggingConfig,
        }

        impl $crate::ToolConfig for Config {
            fn tool_name() -> &'static str {
                env!("CARGO_PKG_NAME")
            }

            fn sections() -> ::std::vec::Vec<$crate::Section> {
                ::std::vec![
                    $( $crate::section::<$section>(), )*
                    $crate::section::<$crate::LoggingConfig>(),
                ]
            }
        }
    };
}

#[cfg(test)]
#[path = "config_test.rs"]
mod tests;
