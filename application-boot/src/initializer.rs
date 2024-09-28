use crate::context::bootstrap_context::DefaultBootstrapContext;
use application_beans::factory::bean_factory::ConfigurableBeanFactory;
use application_context::context::application_context::ConfigurableApplicationContext;
use application_core::env::property_resolver::PropertyResolver;
use async_trait::async_trait;
use axum::Router;
use tracing::info;

pub trait BootstrapRegistryInitializer: Send + Sync {
    fn initial(&self, context: &DefaultBootstrapContext);
}

pub trait ApplicationContextInitializer: Send + Sync {
    fn initialize(&self, application_context: &Box<dyn ConfigurableApplicationContext>);
}
#[async_trait]
pub trait ServletContextInitializer: Send + Sync {
    fn initialize(&self, router: Router) -> Router;
}

pub struct ContextIdApplicationContextInitializer {}

#[derive(Debug)]
pub struct ContextId {
    pub id: String,
}

impl ApplicationContextInitializer for ContextIdApplicationContextInitializer {
    fn initialize(&self, application_context: &Box<dyn ConfigurableApplicationContext>) {
        let environment = application_context.get_environment_blocking();
        let id = environment
            .get_property::<String>("application.name")
            .unwrap_or("application".to_string());
        let context_id = ContextId { id };
        info!("Initializer set {:?}", context_id);
        application_context.get_bean_factory().set(context_id);
    }
}
