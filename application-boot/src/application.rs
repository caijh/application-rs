use crate::application_listener::{
    ApplicationListener, ApplicationStartingEventListener, BootstrapConfigFileApplicationListener,
    DiscoveryDeRegistryApplicationListener, DiscoveryRegistryApplicationListener,
};
use crate::application_run_listener::{ApplicationRunListeners, EventPublishingRunListener};
use crate::cloud::bootstrap::initializer::ConsulBootstrapRegistryInitializer;
use crate::context::application_event_multi_caster::ApplicationEventMultiCaster;
use crate::context::bootstrap_context::{BootstrapContext, DefaultBootstrapContext};
use crate::env::properties::BootstrapProperties;
use crate::initializer::{
    ApplicationContextInitializer, BootstrapRegistryInitializer,
    ContextIdApplicationContextInitializer, ServletContextInitializer,
};
use crate::logging::listener::{LoggingApplicationListener, LoggingCleanApplicationListener};
use crate::web::context::{ServletWebServerApplicationContext, WebServerApplicationContext};
use application_beans::factory::bean_factory::ConfigurableBeanFactory;
use application_context::context::application_context::{
    ConfigurableApplicationContext, GenericApplicationContext,
};
use application_core::env::environment::{ApplicationEnvironment, ConfigurableEnvironment};
use application_core::env::property::PropertySource;
use async_std::task::block_on;
use async_trait::async_trait;
use axum::Router;
use clap::crate_name;
use config::Config;
use std::env::consts::OS;
use std::error::Error;
use std::process::Command;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, RwLockReadGuard};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use tracing::{debug, info};

lazy_static::lazy_static! {
    pub static ref APPLICATION_CONTEXT: Arc<RwLock<Box<dyn ConfigurableApplicationContext>>> = {
        Arc::new(RwLock::new(Box::new(GenericApplicationContext::default())))
    };
}

#[derive(Clone, Copy)]
pub enum ApplicationType {
    /// 表示为独立应用程序
    App,
    /// 表示为Web应用程序
    Web,
}

pub struct RustApplication {
    pub crate_name: String,
    pub application_type: ApplicationType,
    pub bootstrap_registry_initializers: Arc<RwLock<Vec<Box<dyn BootstrapRegistryInitializer>>>>,
    pub initializers: Arc<RwLock<Vec<Box<dyn ApplicationContextInitializer>>>>,
    pub listeners: Arc<RwLock<Vec<Box<dyn ApplicationListener>>>>,
    pub servlet_context_initializers: Arc<RwLock<Vec<Box<dyn ServletContextInitializer>>>>,
}

static APPLICATION_RUN_LISTENERS: OnceLock<ApplicationRunListeners> = OnceLock::new();

impl Default for RustApplication {
    fn default() -> Self {
        RustApplication::new(crate_name!(), ApplicationType::Web)
    }
}

impl RustApplication {
    pub fn new(crate_name: &str, application_type: ApplicationType) -> Self {
        RustApplication {
            crate_name: crate_name.to_string(),
            application_type,
            bootstrap_registry_initializers: Arc::new(RwLock::new(vec![Box::new(
                ConsulBootstrapRegistryInitializer {},
            )])),
            initializers: Arc::new(RwLock::new(vec![Box::new(
                ContextIdApplicationContextInitializer {},
            )])),
            listeners: Arc::new(RwLock::new(vec![
                Box::new(LoggingApplicationListener {}),
                Box::new(ApplicationStartingEventListener {}),
                Box::new(BootstrapConfigFileApplicationListener {}),
                Box::new(DiscoveryRegistryApplicationListener {}),
                Box::new(DiscoveryDeRegistryApplicationListener {}),
                Box::new(LoggingCleanApplicationListener {}),
            ])),
            servlet_context_initializers: Arc::new(RwLock::new(vec![])),
        }
    }

    pub async fn add_initializer(&self, initializer: Box<dyn ApplicationContextInitializer>) {
        let mut initializers = self.initializers.write().await;
        initializers.push(initializer);
    }

    pub async fn add_listener(&self, listener: Box<dyn ApplicationListener>) {
        let mut listeners = self.listeners.write().await;
        listeners.push(listener);
    }

    pub async fn add_servlet_context_initializer(
        &self,
        initializer: Box<dyn ServletContextInitializer>,
    ) {
        let mut servlet_context_initializers = self.servlet_context_initializers.write().await;
        servlet_context_initializers.push(initializer);
    }

    fn get_application_run_listeners(&self) -> &ApplicationRunListeners {
        APPLICATION_RUN_LISTENERS.get_or_init(|| ApplicationRunListeners {
            listeners: Arc::new(RwLock::new(vec![Box::new(
                EventPublishingRunListener {
                    initial_multicast: Arc::new(ApplicationEventMultiCaster {}),
                },
            )])),
        })
    }

    async fn create_bootstrap_context(&self) -> DefaultBootstrapContext {
        dotenvy::dotenv().ok();

        debug!("create_bootstrap_context");

        let mut properties = BootstrapProperties::read_from_path("./bootstrap.toml").unwrap();
        if properties.application.name.is_empty() {
            properties.application.name = self.crate_name.clone();
        }
        let context = DefaultBootstrapContext::new(properties);
        let initializers = self.bootstrap_registry_initializers.read().await;
        let initializers = initializers.iter();
        for initializer in initializers {
            initializer.initial(&context);
        }

        context
    }

    fn create_environment(
        &self,
        bootstrap_properties: &BootstrapProperties,
    ) -> Result<ApplicationEnvironment, Box<dyn Error>> {
        let active_profiles = bootstrap_properties
            .application
            .config
            .activate
            .profiles
            .clone();
        let locations = bootstrap_properties.application.config.locations.clone();
        let search_file_names = bootstrap_properties.application.config.file_names.clone();
        Ok(ApplicationEnvironment::new(
            active_profiles,
            locations,
            search_file_names,
        ))
    }

    async fn prepare_environment(
        &self,
        bootstrap_context: &DefaultBootstrapContext,
    ) -> Result<(), Box<dyn Error>> {
        debug!("prepare_environment");
        {
            let bootstrap_properties = bootstrap_context.get_bootstrap_properties();
            let mut environment = self.create_environment(bootstrap_properties)?;
            environment = self.configure_environment(environment, bootstrap_properties);
            let application_context = self.get_application_context().await;
            application_context.set_environment(environment).await;
        }
        let listeners = self.get_application_run_listeners();
        listeners
            .environment_prepared(self, bootstrap_context)
            .await;
        Ok(())
    }

    fn configure_environment(
        &self,
        environment: ApplicationEnvironment,
        bootstrap_properties: &BootstrapProperties,
    ) -> ApplicationEnvironment {
        let mut builder = Config::builder();
        builder = builder
            .set_default(
                "application.name",
                bootstrap_properties.get_application_name(),
            )
            .unwrap();
        builder = builder
            .set_default(
                "application.port",
                bootstrap_properties.get_application_port(),
            )
            .unwrap();

        if let Some(cloud) = &bootstrap_properties.application.cloud {
            if let Some(discovery) = &cloud.discovery {
                builder = builder
                    .set_default(
                        "application.cloud.discovery.address",
                        discovery.address.clone(),
                    )
                    .unwrap();
                builder = builder
                    .set_default("application.cloud.discovery.token", discovery.token.clone())
                    .unwrap();
                if let Some(service) = &discovery.service {
                    builder = builder
                        .set_default(
                            "application.cloud.discovery.service.check.address",
                            service.check.address.clone(),
                        )
                        .unwrap();
                    builder = builder
                        .set_default(
                            "application.cloud.discovery.service.check.interval",
                            service.check.interval.clone(),
                        )
                        .unwrap();
                }
            }
            if let Some(config) = &cloud.config {
                builder = builder
                    .set_default("application.cloud.config.enabled", config.enabled)
                    .unwrap();
                builder = builder
                    .set_default("application.cloud.config.address", config.address.clone())
                    .unwrap();
                builder = builder
                    .set_default("application.cloud.config.token", config.token.clone())
                    .unwrap();
            }
        }

        let mut env = environment;
        env.add_property_source(PropertySource {
            name: "defaultProperties".to_string(),
            source: builder.build().unwrap(),
        });

        env
    }

    pub fn create_application_context(&self) {
        debug!("create_application_context");
        let context: Box<dyn ConfigurableApplicationContext> = match self.application_type {
            ApplicationType::App => Box::new(GenericApplicationContext::default()),
            ApplicationType::Web => Box::new(ServletWebServerApplicationContext::default()),
        };

        let mut application_context_write = block_on(APPLICATION_CONTEXT.write());
        *application_context_write = context;
    }

    async fn prepare_context(
        &self,
        bootstrap_context: DefaultBootstrapContext,
    ) -> Result<(), Box<dyn Error>> {
        debug!("prepare_context");
        {
            let application_context = self.get_application_context().await;
            application_context
                .get_bean_factory()
                .set(bootstrap_context);

            self.apply_initializers(&application_context).await;
        }

        let listeners = self.get_application_run_listeners();
        listeners.context_prepared(self).await;

        self.load();

        listeners.context_loaded(self).await;

        Ok(())
    }

    async fn refresh_context(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        debug!("refresh_context");

        let application_context = self.get_application_context().await;

        application_context.refresh().await;

        Ok(())
    }

    async fn after_refresh(&self) {
        let application_context = self.get_application_context().await;
        application_context.after_refresh().await;
        match self.application_type {
            ApplicationType::App => self.started().await,
            ApplicationType::Web => {
                let application_context = application_context
                    .as_any()
                    .downcast_ref::<ServletWebServerApplicationContext>()
                    .unwrap();
                let web_server = application_context.get_web_server().await;
                let servlet_context_initializers = self.servlet_context_initializers.read().await;
                let servlet_context_initializers = servlet_context_initializers.iter();
                // route
                let mut router = Router::new().layer((
                    TraceLayer::new_for_http(),
                    // Graceful shutdown will wait for outstanding requests to complete. Add a timeout so
                    // requests don't hang forever.
                    TimeoutLayer::new(Duration::from_secs(30)),
                ));

                for initializer in servlet_context_initializers {
                    router = initializer.initialize(router);
                }
                let condvar_pair = web_server.start(router).unwrap();
                self.started().await;
                // 等待线程启动。
                let (lock, cvar) = &*condvar_pair;
                let mut stopped = lock.lock().unwrap();
                // 只要 `Mutex<bool>` 内部的值为 `false`，我们就等待。
                while !*stopped {
                    stopped = cvar.wait(stopped).unwrap();
                }
            }
        }
    }

    async fn apply_initializers(
        &self,
        application_context: &Box<dyn ConfigurableApplicationContext>,
    ) {
        let initializers = &self.initializers.read().await;
        let initializers = initializers.iter();
        for initializer in initializers {
            initializer.initialize(&application_context);
        }
    }

    fn load(&self) {}

    async fn starting(&self, bootstrap_context: &DefaultBootstrapContext) {
        let listeners = self.get_application_run_listeners();
        listeners.starting(self, bootstrap_context).await;
    }

    pub async fn started(&self) {
        let listeners = self.get_application_run_listeners();
        listeners.started(self).await;
    }

    async fn stopped(&self) {
        let listeners = self.get_application_run_listeners();
        listeners.stopped(self).await;
    }

    async fn failed(&self) {
        let listeners = self.get_application_run_listeners();
        listeners.failed(self).await;
    }
}

#[async_trait]
pub trait Application {
    async fn run(&self) -> Result<(), Box<dyn Error>>;

    fn stop(&self, application_name: &str) -> Result<(), Box<dyn Error>> {
        match OS {
            "windows" => info!("{} not support", OS),
            _ => {
                let process_name = application_name;

                // Execute the `pgrep` command to find the PID of the running application
                let output = Command::new("pgrep")
                    .arg("-f")
                    .arg(process_name)
                    .arg("-o")
                    .output()
                    .expect("Failed to execute pgrep command");

                // Check if any PID is found
                if output.stdout.is_empty() {
                    info!("No running application found");
                    return Ok(());
                }

                // Extract the PID from the output
                let pid = String::from_utf8_lossy(&output.stdout).trim().to_string();

                // Execute the `kill` command to terminate the application
                let mut kill_output = Command::new("kill")
                    .arg("-2")
                    .arg(&pid)
                    .output()
                    .expect("Failed to execute kill command");

                if !kill_output.status.success() {
                    kill_output = Command::new("kill")
                        .arg("-9")
                        .arg(&pid)
                        .output()
                        .expect("Failed to execute kill command");
                    info!("Application with PID {} killed successfully", pid);
                }
                // Check the output of the kill command
                if kill_output.status.success() {
                    info!("Application with PID {} killed successfully", pid);
                } else {
                    info!("Failed to kill application with PID {}", pid);
                }
            }
        }

        Ok(())
    }

    async fn get_application_context(
        &self,
    ) -> RwLockReadGuard<'_, Box<dyn ConfigurableApplicationContext>> {
        APPLICATION_CONTEXT.read().await
    }

    async fn get_application_context_blocking(
        &self,
    ) -> RwLockReadGuard<'_, Box<dyn ConfigurableApplicationContext>> {
        block_on(APPLICATION_CONTEXT.read())
    }


}

#[async_trait]
impl Application for RustApplication {
    async fn run(&self) -> Result<(), Box<dyn Error>> {
        let start_time = Instant::now();

        let bootstrap_context = self.create_bootstrap_context().await;

        self.starting(&bootstrap_context).await;

        self.create_application_context();

        self.prepare_environment(&bootstrap_context).await?;

        self.prepare_context(bootstrap_context).await?;

        let result = self.refresh_context().await;

        let duration = start_time.elapsed();
        info!(
            "Started {} in {} millis",
            self.crate_name,
            duration.as_millis()
        );

        match result {
            Ok(_) => {
                self.after_refresh().await;
                self.stopped().await;
                Ok(())
            }
            Err(e) => {
                info!("Application start failed {:?}", e);
                self.failed().await;
                Ok(())
            }
        }
    }
}
