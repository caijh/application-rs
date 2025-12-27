use crate::bootstrap::bootstrap_context::BootstrapContext;
use std::collections::HashMap;
use std::error::Error;

use crate::application::{Application, RustApplication};
use crate::bootstrap::bootstrap_registry::BootstrapRegistry;
use crate::bootstrap::default_bootstrap_context::DefaultBootstrapContext;
use crate::cloud::client::registry::{
    ConsulServiceRegistry, Registration, ServiceCheck, ServiceInstance, ServiceRegistry,
};
use crate::env::configuration::{Configuration, ConfigurationResolver};
use crate::logging::listener::ApplicationStartingEvent;
use application_beans::factory::bean_factory::BeanFactory;
use application_context::context::application_event::{ApplicationEvenType, ApplicationEvent};
use application_core::env::environment::ConfigurableEnvironment;
use application_core::env::property::PropertySource;
use async_trait::async_trait;
use tracing::info;
use util::ip::LocalIp;

#[async_trait]
pub trait ApplicationListener: Send + Sync {
    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    fn is_support(&self, event: &dyn ApplicationEvent) -> bool;

    async fn on_application_event(
        &self,
        application: &RustApplication,
        event: &dyn ApplicationEvent,
    ) -> Result<(), Box<dyn Error>>;
}

/// Print starting log when starting
pub struct ApplicationStartingEventListener {}

#[async_trait]
impl ApplicationListener for ApplicationStartingEventListener {
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
        let application_name = &bootstrap_properties.get_application_name();
        info!("Application {} is starting", application_name);
        Ok(())
    }
}

#[derive(Clone)]
pub struct BootstrapConfigFileApplicationListener {}

#[async_trait]
impl ApplicationListener for BootstrapConfigFileApplicationListener {
    fn is_support(&self, event: &dyn ApplicationEvent) -> bool {
        event.get_event_type() == ApplicationEvenType::EnvironmentPrepared
    }

    async fn on_application_event(
        &self,
        application: &RustApplication,
        _event: &dyn ApplicationEvent,
    ) -> Result<(), Box<dyn Error>> {
        let application_context = application.get_application_context().await;
        let mut environment = application_context.get_environment_mut().await;

        let native_config = Configuration::read_native_config_from_environment(&environment)?;
        environment.add_property_source(PropertySource {
            name: "configProperties".to_string(),
            source: native_config,
        });

        let cloud_config = Configuration::read_remote_config_from_environment(&environment)?;
        environment.add_property_source(PropertySource {
            name: "cloudProperties".to_string(),
            source: cloud_config,
        });

        Ok(())
    }
}

/// 应用启动后，注册服务实例
pub struct DiscoveryRegistryApplicationListener {}

#[async_trait]
impl ApplicationListener for DiscoveryRegistryApplicationListener {
    fn is_support(&self, event: &dyn ApplicationEvent) -> bool {
        event.get_event_type() == ApplicationEvenType::Started
    }

    async fn on_application_event(
        &self,
        application: &RustApplication,
        _event: &dyn ApplicationEvent,
    ) -> Result<(), Box<dyn Error>> {
        let application_context = application.get_application_context().await;
        let bootstrap_context = application_context
            .get_bean_factory()
            .get::<DefaultBootstrapContext>();
        let properties = &bootstrap_context.get_bootstrap_properties();
        if let Some(cloud) = &properties.application.cloud {
            if let Some(discovery) = &cloud.discovery {
                let registry = bootstrap_context.get::<ConsulServiceRegistry>().unwrap();
                let service_id = &properties.application.name;
                let local_ip = LocalIp::get_local_addr_ip().unwrap();
                let mut host = match hostname::get() {
                    Ok(hostname) => hostname.to_string_lossy().to_string(),
                    Err(_) => local_ip.to_string(),
                };
                let mut port = properties.application.port.unwrap();
                if let Some(host_properties) = &discovery.host {
                    host = host_properties.ip.clone();
                    port = host_properties.port;
                }
                let schema = if port == 443 { "https" } else { "http" };
                let mut health_check_url =
                    format!("{}://{}:{}/actuator/health", schema, host, port);
                let mut interval = "30s".to_string();
                if let Some(health) = &discovery.health {
                    let check = &health.check;
                    health_check_url = format!("{}://{}:{}/{}", schema, host, port, check.path);
                    interval = check.interval.clone();
                }
                let service_check = ServiceCheck {
                    address: Some(health_check_url),
                    interval: Some(interval),
                };
                let instance_id = format!("{}@{}:{}", service_id, host, port);
                let service_instance = ServiceInstance {
                    instance_id,
                    service_id: service_id.clone(),
                    host,
                    port: port as u32,
                    is_secure: schema == "https",
                    metadata: HashMap::new(),
                    schema: schema.to_string(),
                };
                let registration = Registration {
                    service_instance,
                    service_check,
                };
                registry.register(&registration).await?;
                info!("Register {:?}", registration);
                bootstrap_context.register(registration);
            }
        }
        Ok(())
    }
}

/// 应用停止时，注销服务实例
pub struct DiscoveryDeRegistryApplicationListener {}

#[async_trait]
impl ApplicationListener for DiscoveryDeRegistryApplicationListener {
    fn is_support(&self, event: &dyn ApplicationEvent) -> bool {
        event.get_event_type() == ApplicationEvenType::Stopped
    }

    async fn on_application_event(
        &self,
        application: &RustApplication,
        _event: &dyn ApplicationEvent,
    ) -> Result<(), Box<dyn Error>> {
        let application_context = application.get_application_context().await;
        let bootstrap_context = application_context
            .get_bean_factory()
            .get::<DefaultBootstrapContext>();
        let registration = bootstrap_context.get::<Registration>();
        if let Some(service_instance) = registration {
            let registry = bootstrap_context.get::<ConsulServiceRegistry>().unwrap();
            let _ = registry.deregister(service_instance).await?;
            info!("DeRegister {:?}", service_instance);
        }
        Ok(())
    }
}
