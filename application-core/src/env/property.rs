use config::Config;
use serde::Deserialize;
use std::collections::LinkedList;

#[derive(Default, Clone)]
pub struct PropertySource {
    /// The name of the property source
    pub name: String,
    /// The configuration for the property source
    pub source: Config,
}

impl PropertySource {
    pub fn get_property<'de, T: Deserialize<'de>>(
        &self,
        key: &str,
    ) -> Result<T, config::ConfigError> {
        self.source.get::<T>(key)
    }
}

#[derive(Default, Clone)]
pub struct MutablePropertySources {
    sources: LinkedList<PropertySource>,
}

impl MutablePropertySources {
    pub fn add_last(&mut self, property_source: PropertySource) {
        self.sources.push_back(property_source);
    }

    pub fn get_sources(&self) -> &LinkedList<PropertySource> {
        &self.sources
    }
}
