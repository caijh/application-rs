use serde::Deserialize;

pub trait PropertyResolver {
    fn get_property<'de, T: Deserialize<'de>>(&self, key: &str) -> Option<T>;

    fn get_property_default<'de, T: Deserialize<'de>>(&self, key: &str, data: T) -> T {
        let property = self.get_property::<T>(key);
        property.unwrap_or(data)
    }
}
