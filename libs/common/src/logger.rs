use crate::config::{LogFormat, LogLevel, LogTarget, LoggingConfig};
use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    Layer, Registry,
    fmt::{self, writer::BoxMakeWriter},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

use std::path::Path;

pub struct Logger {
    _guard: Option<WorkerGuard>,
}

impl Logger {
    pub fn new(cfg: &LoggingConfig) -> Self {
        let level = to_tracing_level(&cfg.level);
        let filter = tracing_subscriber::filter::LevelFilter::from_level(level);

        let (writer, guard, is_file) = match &cfg.target {
            LogTarget::Stdout => (BoxMakeWriter::new(std::io::stdout), None, false),
            LogTarget::Stderr => (BoxMakeWriter::new(std::io::stderr), None, false),
            LogTarget::Path(path) => {
                let path = {
                    let p = Path::new(path);
                    let looks_like_dir =
                        path.ends_with('/') || path.ends_with('\\') || p.extension().is_none();
                    if looks_like_dir {
                        std::fs::create_dir_all(p).expect("Failed to create log directory");
                        p.join("app.log")
                    } else {
                        p.with_extension("log")
                    }
                };
                let file = std::fs::File::create(path).expect("Failed to create log file");
                let (nb, guard) = tracing_appender::non_blocking(file);
                (BoxMakeWriter::new(nb), Some(guard), true)
            }
        };

        match cfg.format {
            LogFormat::Json => {
                let layer = fmt::layer().json().with_writer(writer).with_filter(filter);
                Registry::default().with(layer).init();
            }
            LogFormat::Txt => {
                let layer = fmt::layer()
                    .with_writer(writer)
                    .with_ansi(!is_file)
                    .with_filter(filter);
                Registry::default().with(layer).init();
            }
        }
        Self { _guard: guard }
    }
}

fn to_tracing_level(level: &LogLevel) -> Level {
    match level {
        LogLevel::Debug => Level::DEBUG,
        LogLevel::Info => Level::INFO,
        LogLevel::Warn => Level::WARN,
        LogLevel::Error => Level::ERROR,
    }
}
