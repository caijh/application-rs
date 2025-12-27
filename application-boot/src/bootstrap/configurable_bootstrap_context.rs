use crate::bootstrap::bootstrap_context::BootstrapContext;
use crate::bootstrap::bootstrap_registry::BootstrapRegistry;

pub trait ConfigurableBootstrapContext: BootstrapRegistry + BootstrapContext {}
