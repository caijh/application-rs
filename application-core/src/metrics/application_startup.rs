use crate::metrics::startup_step::StartupStep;

pub trait ApplicationStartup: Send + Sync {
    fn start(&self, name: &str) -> Box<dyn StartupStep>;
}
