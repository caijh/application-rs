use crate::bootstrap::bootstrap_context::BootstrapContext;
use crate::bootstrap::bootstrap_registry::BootstrapRegistry;
use crate::bootstrap::configurable_bootstrap_context::ConfigurableBootstrapContext;
use crate::env::properties::BootstrapProperties;
use state::TypeMap;

pub struct DefaultBootstrapContext {
    properties: BootstrapProperties,
    instances: TypeMap![Send + Sync],
}

impl DefaultBootstrapContext {
    pub fn new(bootstrap_properties: BootstrapProperties) -> Self {
        Self {
            properties: bootstrap_properties,
            instances: Default::default(),
        }
    }

    pub fn get_bootstrap_properties(&self) -> &BootstrapProperties {
        &self.properties
    }
}

impl BootstrapRegistry for DefaultBootstrapContext {
    fn register<T: Send + Sync + 'static>(&self, state: T) -> bool {
        self.instances.set::<T>(state)
    }

    fn is_registered<T: Send + Sync + 'static>(&self) -> bool {
        self.instances.try_get::<T>().is_some()
    }

    fn register_if_absent<T: Send + Sync + 'static>(&self, state: T) {
        let is_registered = self.instances.try_get::<T>().is_some();
        if is_registered {
            return;
        }
        self.register(state);
    }
}

impl BootstrapContext for DefaultBootstrapContext {
    fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.instances.try_get::<T>()
    }

    fn is_registered<T: Send + Sync + 'static>(&self) -> bool {
        self.instances.try_get::<T>().is_some()
    }
}

impl ConfigurableBootstrapContext for DefaultBootstrapContext {}
