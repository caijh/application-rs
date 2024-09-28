use crate::application::{Application, RustApplication};
use crate::application_listener::ApplicationListener;
use crate::context::bootstrap_context::{BootstrapContext, DefaultBootstrapContext};
use crate::env::properties::BootstrapProperties;
use crate::logging::LOGGING_SYSTEM;
use application_beans::factory::bean_factory::BeanFactory;
use application_context::context::application_event::{ApplicationEvenType, ApplicationEvent};
use async_trait::async_trait;
use config::Config;
use logger::Logger;
use std::any::Any;
use std::error::Error;
use std::fs;
use std::time::{Duration, SystemTime};

/// 设置logger，在启动时
pub struct LoggingApplicationListener {}

pub struct ApplicationStartingEvent {
    pub bootstrap_properties: BootstrapProperties,
}
impl ApplicationEvent for ApplicationStartingEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn get_event_type(&self) -> ApplicationEvenType {
        ApplicationEvenType::Starting
    }
}

#[async_trait]
impl ApplicationListener for LoggingApplicationListener {
    fn is_support(&self, event: &dyn ApplicationEvent) -> bool {
        event.get_event_type() == ApplicationEvenType::Starting
    }

    async fn on_application_event(
        &self,
        _application: &RustApplication,
        event: &dyn ApplicationEvent,
    ) -> Result<(), Box<dyn Error>> {
        let event: &ApplicationStartingEvent = event
            .as_any()
            .downcast_ref::<ApplicationStartingEvent>()
            .unwrap();
        let bootstrap_properties = &event.bootstrap_properties;
        let logger_properties = &bootstrap_properties.logger;
        let mut builder = Config::builder();
        builder = builder
            .set_default("logger.enabled", logger_properties.enabled)
            .unwrap();
        builder = builder
            .set_default("logger.level", logger_properties.level.clone())
            .unwrap();
        builder = builder
            .set_default("logger.file", logger_properties.file.clone())
            .unwrap();
        builder = builder
            .set_default("logger.log_dir", logger_properties.log_dir.clone())
            .unwrap();
        let config = builder.build().unwrap();
        let logger = Logger::init_logger(&config);
        let mut loggers = LOGGING_SYSTEM.write().await;
        loggers.insert(bootstrap_properties.application.name.clone(), logger);
        Ok(())
    }
}

pub struct LoggingCleanApplicationListener {}

#[async_trait]
impl ApplicationListener for LoggingCleanApplicationListener {
    fn is_support(&self, event: &dyn ApplicationEvent) -> bool {
        event.get_event_type() == ApplicationEvenType::Stopped
    }

    async fn on_application_event(
        &self,
        application: &RustApplication,
        _event: &dyn ApplicationEvent,
    ) -> Result<(), Box<dyn Error>> {
        let application_context = application.get_application_context();
        let bootstrap_context = application_context
            .get_bean_factory()
            .get::<DefaultBootstrapContext>();
        let bootstrap_properties = bootstrap_context.get_bootstrap_properties();
        let logger_properties = &bootstrap_properties.logger;
        if logger_properties.enabled {
            // clean old logs older than 5 days
            let log_dir = &logger_properties.log_dir;
            let _ = clean_log_files(log_dir, 7).await;
        }
        Ok(())
    }
}

async fn clean_log_files(log_dir: &str, days: u64) -> Result<(), Box<dyn Error>> {
    let threshold = SystemTime::now() - Duration::from_secs(days * 24 * 60 * 60);
    if let Ok(entries) = fs::read_dir(log_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let metadata = fs::metadata(&path)?;
                let modified = metadata.modified()?;
                if modified < threshold {
                    fs::remove_file(&path)?;
                }
            }
        }
    }
    Ok(())
}
