use std::io;
use std::str::FromStr;

use config::Config;
use serde::{Deserialize, Serialize};
use tracing::{info, Level};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{fmt, EnvFilter};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LoggerConfig {
    pub enabled: bool,
    pub level: String,
    pub file: String,
    pub log_dir: String,
}

impl LoggerConfig {
    pub fn get_config(config: &Config) -> LoggerConfig {
        config
            .get::<LoggerConfig>("logger")
            .unwrap_or(LoggerConfig {
                enabled: false,
                level: "info".to_string(),
                file: "info".to_string(),
                log_dir: "./logs".to_string(),
            })
    }
}

pub struct Logger {
    pub config: LoggerConfig,
    worker_guard: Option<WorkerGuard>,
}

impl Logger {
    fn init(&mut self) {
        info!("Setting up logger with config {:?}", &self.config);

        let level = Level::from_str(self.config.level.as_str()).unwrap();
        if self.config.enabled {
            let file_appender = rolling::daily(&self.config.log_dir, &self.config.file);
            let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
            let filter = EnvFilter::from_default_env().add_directive(level.into());

            let subscriber = tracing_subscriber::registry()
                .with(filter)
                .with(fmt::Layer::new().with_ansi(false).with_writer(io::stdout))
                .with(fmt::Layer::new().with_ansi(false).with_writer(non_blocking));
            tracing::subscriber::set_global_default(subscriber)
                .expect("Unable to set a global subscriber");
            self.worker_guard = Some(_guard)
        } else {
            tracing_subscriber::fmt()
                .with_ansi(false)
                .with_thread_names(true)
                // enable everything
                .with_max_level(level)
                // sets this to be the default, global subscriber for this application.
                .init();
        }
    }

    pub fn init_logger(config: &Config) -> Logger {
        let config = LoggerConfig::get_config(config);
        let mut logger = Logger {
            config,
            worker_guard: None,
        };
        logger.init();
        logger
    }
}

#[cfg(test)]
mod tests {}
