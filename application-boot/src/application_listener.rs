use std::error::Error;

use crate::application::{Application, RustApplication};
use crate::context::bootstrap_context::{BootstrapContext, DefaultBootstrapContext};
use crate::env::configuration::{Configuration, ConfigurationResolver};
use crate::logging::listener::ApplicationStartingEvent;
use application_beans::factory::bean_factory::BeanFactory;
use application_context::context::application_event::{ApplicationEvenType, ApplicationEvent};
use application_core::env::environment::ConfigurableEnvironment;
use application_core::env::property::PropertySource;
use async_trait::async_trait;
use consulrs::client::ConsulClient;
use registration::{deregister, register, ServiceCheck, ServiceInstance, ServiceRegistration};
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
                let consul_client = bootstrap_context.get::<ConsulClient>();
                let service_check_properties = &discovery.service.clone().unwrap().check;
                let service_check = ServiceCheck {
                    address: Some(service_check_properties.address.clone()),
                    interval: Some(service_check_properties.interval.clone()),
                };

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

                let registration = ServiceRegistration::new(
                    service_id.as_str(),
                    host.as_str(),
                    port,
                    &service_check,
                );
                let service_instance = register(consul_client, &registration).await?;
                info!("Register {:?}", service_instance);
                bootstrap_context.set(service_instance);
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
        let service_instance = bootstrap_context.try_get::<ServiceInstance>();
        if let Some(service_instance) = service_instance {
            let consul_client = bootstrap_context.get::<ConsulClient>();
            let _ = deregister(consul_client, service_instance).await?;
            info!("DeRegister {:?}", service_instance);
        }
        Ok(())
    }
}
