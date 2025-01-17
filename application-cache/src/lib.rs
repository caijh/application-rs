use moka2::future::Cache;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock, RwLock};
use std::time::Duration;

static CACHE: OnceLock<CacheManager> = OnceLock::new();

pub struct NamedCache {
    pub name: String,
    cache: Cache<String, String>,
}

pub struct CacheManager {
    caches: Arc<RwLock<HashMap<String, NamedCache>>>,
}

const DEFAULT_MAX_CAPACITY: u64 = 10000;

impl CacheManager {
    pub fn get_or_init() -> &'static CacheManager {
        CACHE.get_or_init(|| {
            let cache: Cache<String, String> = Cache::builder()
                .max_capacity(DEFAULT_MAX_CAPACITY)
                .time_to_idle(Duration::from_secs(1800))
                .build();
            let default_cache = NamedCache {
                name: "".to_string(),
                cache,
            };
            let mut map = HashMap::new();
            map.insert(String::from(""), default_cache);
            CacheManager {
                caches: Arc::new(RwLock::new(map)),
            }
        })
    }

    pub async fn get(key: &str) -> Option<String> {
        Self::get_from("", key).await
    }

    pub async fn get_from(name: &str, key: &str) -> Option<String> {
        let cache_manager = Self::get_or_init();
        let caches = cache_manager.caches.read().unwrap();
        let name_cache = caches.get(name);
        if name_cache.is_none() {
            return None;
        }
        let name_cache = name_cache.unwrap();
        name_cache.cache.get(key).await
    }

    pub async fn set(key: &str, value: &str) {
        Self::set_to("", key, value, Duration::from_secs(1800)).await
    }

    pub async fn set_to(name: &str, key: &str, value: &str, duration: Duration) {
        let cache_manager = Self::get_or_init();
        let mut caches = cache_manager.caches.write().unwrap();
        let name_cache = caches.get_mut(name);
        if name_cache.is_none() {
            let cache: Cache<String, String> = Cache::builder()
                .max_capacity(DEFAULT_MAX_CAPACITY)
                .time_to_idle(duration)
                .build();
            cache.insert(key.to_string(), value.to_string()).await;
            let name_cache = NamedCache {
                name: name.to_string(),
                cache,
            };
            caches.insert(name.to_string(), name_cache);
        } else {
            let name_cache = name_cache.unwrap();
            name_cache
                .cache
                .insert(key.to_string(), value.to_string())
                .await;
        }
    }
}
