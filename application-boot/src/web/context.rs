use application_beans::factory::bean_factory::{
    BeanFactory, DefaultListableBeanFactory, ListableBeanFactory,
};
use application_context::context::application_context::{
    ApplicationContext, ConfigurableApplicationContext,
};
use application_context::context::application_event::{
    ApplicationEvent, ApplicationEventPublisher,
};
use application_core::env::environment::{ApplicationEnvironment, EnvironmentCapable};
use application_core::env::property_resolver::PropertyResolver;
use async_std::task::block_on;
use async_trait::async_trait;
use std::any::{Any, TypeId};
use std::sync::Arc;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use web::server::{AxumServer, WebServer};

#[async_trait]
pub trait WebServerApplicationContext: ConfigurableApplicationContext {
    async fn get_web_server(&self) -> RwLockReadGuard<'_, Box<dyn WebServer>>;
}

pub struct ServletWebServerApplicationContext {
    environment: Arc<RwLock<ApplicationEnvironment>>,
    bean_factory: DefaultListableBeanFactory,
    web_server: Arc<RwLock<Box<dyn WebServer>>>,
}

impl Default for ServletWebServerApplicationContext {
    fn default() -> Self {
        Self {
            environment: Default::default(),
            bean_factory: Default::default(),
            web_server: Arc::new(RwLock::new(Box::new(AxumServer { port: 0 }))),
        }
    }
}

impl BeanFactory for ServletWebServerApplicationContext {
    fn get<T: 'static>(&self) -> &T {
        self.bean_factory.get::<T>()
    }

    fn try_get<T: 'static>(&self) -> Option<&T> {
        self.bean_factory.try_get::<T>()
    }
}

impl ListableBeanFactory for ServletWebServerApplicationContext {
    fn get_bean_definition_count(&self) -> usize {
        self.bean_factory.get_bean_definition_count()
    }
}

#[async_trait]
impl EnvironmentCapable for ServletWebServerApplicationContext {
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
impl ApplicationEventPublisher for ServletWebServerApplicationContext {
    fn publish_event(&self, _event: Arc<Box<dyn ApplicationEvent>>) {}
}

#[async_trait]
impl ApplicationContext for ServletWebServerApplicationContext {
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

#[async_trait]
impl ConfigurableApplicationContext for ServletWebServerApplicationContext {
    async fn refresh(&self) {
        let port = self
            .get_environment()
            .await
            .get_property::<u16>("application.port")
            .unwrap();
        let web_server = AxumServer { port };
        let mut application_web_server = self.web_server.write().await;
        *application_web_server = Box::new(web_server);
    }
}

#[async_trait]
impl WebServerApplicationContext for ServletWebServerApplicationContext {
    async fn get_web_server(&self) -> RwLockReadGuard<'_, Box<dyn WebServer>> {
        self.web_server.read().await
    }
}
