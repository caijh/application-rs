use crate::bootstrap::default_bootstrap_context::DefaultBootstrapContext;

pub trait BootstrapRegistryInitializer: Send + Sync {
    fn initial(&self, context: &DefaultBootstrapContext);
}
