use crate::env::properties::BootstrapProperties;
use application_beans::factory::bean_factory::{
    BeanFactory, ConfigurableBeanFactory, DefaultListableBeanFactory,
};

pub trait BootstrapContext: BeanFactory {
    fn get_bootstrap_properties(&self) -> &BootstrapProperties;
    fn set<T: Send + Sync + 'static>(&self, state: T) -> bool;

    fn is_registered<T: Send + Sync + 'static>(&self) -> bool;
}

pub struct DefaultBootstrapContext {
    properties: BootstrapProperties,
    bean_factory: DefaultListableBeanFactory,
}

impl DefaultBootstrapContext {
    pub fn new(bootstrap_properties: BootstrapProperties) -> Self {
        Self {
            properties: bootstrap_properties,
            bean_factory: Default::default(),
        }
    }
}

impl BeanFactory for DefaultBootstrapContext {
    fn get<T: 'static>(&self) -> &T {
        self.bean_factory.get::<T>()
    }

    fn try_get<T: 'static>(&self) -> Option<&T> {
        self.bean_factory.try_get::<T>()
    }
}

impl BootstrapContext for DefaultBootstrapContext {
    fn get_bootstrap_properties(&self) -> &BootstrapProperties {
        &self.properties
    }

    fn set<T: Send + Sync + 'static>(&self, state: T) -> bool {
        self.bean_factory.set(state)
    }

    fn is_registered<T: Send + Sync + 'static>(&self) -> bool {
        let option = self.bean_factory.try_get::<T>();
        option.is_some()
    }
}
