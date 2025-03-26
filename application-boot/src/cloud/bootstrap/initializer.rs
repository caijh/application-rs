use crate::cloud::client::registry::ConsulServiceRegistry;
use crate::context::bootstrap_context::{BootstrapContext, DefaultBootstrapContext};
use crate::initializer::BootstrapRegistryInitializer;
use consulrs::client::{ConsulClient, ConsulClientSettingsBuilder};

pub struct RefreshBootstrapRegistryInitializer {}

impl BootstrapRegistryInitializer for RefreshBootstrapRegistryInitializer {
    fn initial(&self, _context: &DefaultBootstrapContext) {
        todo!()
    }
}

/// Consul ClientInitializer
pub struct ConsulBootstrapRegistryInitializer {}

impl BootstrapRegistryInitializer for ConsulBootstrapRegistryInitializer {
    fn initial(&self, context: &DefaultBootstrapContext) {
        let bootstrap_properties = context.get_bootstrap_properties();
        if let Some(cloud) = &bootstrap_properties.application.cloud {
            if let Some(discovery) = &cloud.discovery {
                let server_properties = &discovery.server;
                let token = server_properties.token.clone();
                let address = &discovery.server.address;
                let client = ConsulClient::new(
                    ConsulClientSettingsBuilder::default()
                        .address(address)
                        .token(token.unwrap_or_default())
                        .build()
                        .unwrap(),
                )
                .unwrap();
                let registry = ConsulServiceRegistry { client };
                context.set(registry);
            }
        }
    }
}
