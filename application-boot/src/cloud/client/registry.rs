use async_trait::async_trait;
use consulrs::api::check::common::AgentServiceCheckBuilder;
use consulrs::api::service::requests::RegisterServiceRequest;
use consulrs::client::ConsulClient;
use consulrs::service;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServiceInstance {
    pub instance_id: String,
    pub service_id: String,
    pub host: String,
    pub port: u32,
    pub is_secure: bool,
    pub metadata: HashMap<String, String>,
    pub schema: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServiceCheck {
    pub address: Option<String>,
    pub interval: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Registration {
    pub service_instance: ServiceInstance,
    pub service_check: ServiceCheck,
}

#[async_trait]
pub trait ServiceRegistry {
    async fn register(&self, registration: &Registration) -> Result<(), Box<dyn Error>>;
    async fn deregister(&self, registration: &Registration) -> Result<(), Box<dyn Error>>;
}

pub struct ConsulServiceRegistry {
    pub client: ConsulClient,
}

#[async_trait]
impl ServiceRegistry for ConsulServiceRegistry {
    async fn register(&self, registration: &Registration) -> Result<(), Box<dyn Error>> {
        let service_check_address = registration
            .service_check
            .address
            .clone()
            .unwrap_or_default();
        let service_check_interval = registration
            .service_check
            .interval
            .clone()
            .unwrap_or_default();
        let service_instance = &registration.service_instance;
        let host = service_instance.host.clone();
        let response = service::register(
            &self.client,
            &service_instance.service_id,
            Some(
                RegisterServiceRequest::builder()
                    .id(&service_instance.instance_id)
                    .address(&host)
                    .port(service_instance.port as u64)
                    .check(
                        AgentServiceCheckBuilder::default()
                            .name("health_check")
                            .interval(&service_check_interval)
                            .http(service_check_address)
                            .status("passing")
                            .build()
                            .unwrap(),
                    ),
            ),
        )
        .await;
        match response {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    async fn deregister(&self, registration: &Registration) -> Result<(), Box<dyn Error>> {
        let service_instance = &registration.service_instance;
        let result = service::deregister(&self.client, &service_instance.instance_id, None).await;
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}
