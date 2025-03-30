use application_logger::Logger;
use std::collections::HashMap;
use std::sync::Arc;

pub mod listener;

lazy_static::lazy_static! {
    pub static ref LOGGING_SYSTEM: Arc<tokio::sync::RwLock<HashMap<String, Logger>>> = {
        Arc::new(tokio::sync::RwLock::new(HashMap::new()))
    };
}
