use moka2::future::Cache;
use std::sync::OnceLock;
use std::time::Duration;

static CACHE: OnceLock<CacheManager> = OnceLock::new();

pub struct CacheManager {
    cache: Cache<String, String>,
}

const DEFAULT_MAX_CAPACITY: u64 = 10000;

impl CacheManager {
    pub fn get_or_init(max_capacity: u64, ttl: Duration) -> &'static CacheManager {
        CACHE.get_or_init(|| {
            let cache: Cache<String, String> = Cache::builder()
                .max_capacity(max_capacity)
                .time_to_idle(ttl)
                .build();
            CacheManager { cache }
        })
    }

    pub async fn get(key: &str) -> Option<String> {
        let cache_manager = Self::get_or_init(DEFAULT_MAX_CAPACITY, Duration::from_secs(3600));
        cache_manager.cache.get(key).await
    }

    pub async fn set(key: &str, value: &str) {
        let cache_manager = Self::get_or_init(DEFAULT_MAX_CAPACITY, Duration::from_secs(3600));
        cache_manager
            .cache
            .insert(key.to_string(), value.to_string())
            .await;
    }
}
