use crate::env::property::{MutablePropertySources, PropertySource};
use crate::env::property_resolver::PropertyResolver;
use async_trait::async_trait;
use serde::Deserialize;
use tokio::sync::{RwLockReadGuard, RwLockWriteGuard};

pub trait Environment: PropertyResolver {
    fn get_active_profiles(&self) -> Vec<String>;
    fn get_config_locations(&self) -> Option<Vec<String>>;
    fn get_file_names(&self) -> Option<Vec<String>>;
}

pub trait ConfigurableEnvironment: Environment {
    fn add_property_source(&mut self, property_source: PropertySource);

    fn get_property_sources(&self) -> &MutablePropertySources;
}
#[derive(Default, Clone)]
pub struct ApplicationEnvironment {
    active_profiles: Vec<String>,
    config_locations: Option<Vec<String>>,
    config_file_names: Option<Vec<String>>,
    property_sources: MutablePropertySources,
}

impl ApplicationEnvironment {
    pub fn new(
        active_profiles: Vec<String>,
        locations: Option<Vec<String>>,
        file_names: Option<Vec<String>>,
    ) -> Self {
        ApplicationEnvironment {
            active_profiles,
            config_locations: locations,
            config_file_names: file_names,
            property_sources: Default::default(),
        }
    }
}

impl Environment for ApplicationEnvironment {
    fn get_active_profiles(&self) -> Vec<String> {
        self.active_profiles.clone()
    }

    fn get_config_locations(&self) -> Option<Vec<String>> {
        self.config_locations.clone()
    }

    fn get_file_names(&self) -> Option<Vec<String>> {
        self.config_file_names.clone()
    }
}

impl ConfigurableEnvironment for ApplicationEnvironment {
    fn add_property_source(&mut self, property_source: PropertySource) {
        self.property_sources.add_last(property_source);
    }

    fn get_property_sources(&self) -> &MutablePropertySources {
        &self.property_sources
    }
}

impl PropertyResolver for ApplicationEnvironment {
    fn get_property<'de, T: Deserialize<'de>>(&self, key: &str) -> Option<T> {
        for property_source in self.get_property_sources().get_sources() {
            let result = property_source.get_property::<T>(key);
            if result.is_ok() {
                return result.ok();
            }
        }
        None
    }
}

#[async_trait]
pub trait EnvironmentCapable {
    async fn get_environment(&self) -> RwLockReadGuard<'_, ApplicationEnvironment>;
    async fn get_environment_mut(&self) -> RwLockWriteGuard<'_, ApplicationEnvironment>;
    fn get_environment_blocking(&self) -> RwLockReadGuard<'_, ApplicationEnvironment>;
}
