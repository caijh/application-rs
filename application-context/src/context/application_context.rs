use crate::context::application_event::{ApplicationEvent, ApplicationEventPublisher};
use application_beans::factory::bean_factory::{
    BeanFactory, DefaultListableBeanFactory, ListableBeanFactory,
};
use application_core::env::environment::{ApplicationEnvironment, EnvironmentCapable};
use application_core::env::property_resolver::PropertyResolver;
use async_std::task::block_on;
use async_trait::async_trait;
use std::any::{Any, TypeId};
use std::sync::Arc;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

#[async_trait]
pub trait ApplicationContext: EnvironmentCapable + ApplicationEventPublisher + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn get_id(&self) -> TypeId;
    fn get_application_name(&self) -> String {
        self.get_environment_blocking()
            .get_property::<String>("application.name")
            .unwrap_or_default()
    }
    fn get_bean_factory(&self) -> &DefaultListableBeanFactory;
    async fn set_environment(&self, environment: ApplicationEnvironment);
}
#[async_trait]
pub trait ConfigurableApplicationContext: ApplicationContext {
    async fn refresh(&self) {}
    async fn after_refresh(&self) {}
}

#[derive(Default)]
pub struct GenericApplicationContext {
    environment: Arc<RwLock<ApplicationEnvironment>>,
    bean_factory: DefaultListableBeanFactory,
}

impl BeanFactory for GenericApplicationContext {
    fn get<T: 'static>(&self) -> &T {
        self.bean_factory.get::<T>()
    }

    fn try_get<T: 'static>(&self) -> Option<&T> {
        self.bean_factory.try_get::<T>()
    }
}

impl ListableBeanFactory for GenericApplicationContext {
    fn get_bean_definition_count(&self) -> usize {
        self.bean_factory.get_bean_definition_count()
    }
}

#[async_trait]
impl EnvironmentCapable for GenericApplicationContext {
    async fn get_environment(&self) -> RwLockReadGuard<'_, ApplicationEnvironment> {
        self.environment.read().await
    }

    async fn get_environment_mut(&self) -> RwLockWriteGuard<'_, ApplicationEnvironment> {
        self.environment.write().await
    }

    fn get_environment_blocking(&self) -> RwLockReadGuard<'_, ApplicationEnvironment> {
        block_on(self.environment.read())
    }
}

#[async_trait]
impl ApplicationEventPublisher for GenericApplicationContext {
    fn publish_event(&self, _event: Arc<Box<dyn ApplicationEvent>>) {}
}

#[async_trait]
impl ApplicationContext for GenericApplicationContext {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_id(&self) -> TypeId {
        self.type_id()
    }

    fn get_bean_factory(&self) -> &DefaultListableBeanFactory {
        &self.bean_factory
    }

    async fn set_environment(&self, environment: ApplicationEnvironment) {
        let mut application_environment = self.environment.write().await;
        *application_environment = environment;
    }
}

impl ConfigurableApplicationContext for GenericApplicationContext {}

lazy_static::lazy_static! {
    pub static ref APPLICATION_CONTEXT: Arc<RwLock<Box<dyn ConfigurableApplicationContext>>> = {
        Arc::new(RwLock::new(Box::new(GenericApplicationContext::default())))
    };
}
