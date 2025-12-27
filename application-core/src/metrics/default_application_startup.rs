use crate::metrics::application_startup::ApplicationStartup;
use crate::metrics::startup_step::{StartupStep, Tag};

pub struct DefaultStartupStep {
    name: String,
}

impl StartupStep for DefaultStartupStep {
    fn get_name(&self) -> String {
        self.name.to_string()
    }

    fn get_id(&self) -> u64 {
        0
    }

    fn get_parent_id(&self) -> Option<u64> {
        None
    }

    fn get_tags(&self) -> Vec<Tag> {
        vec![]
    }

    fn end(&self) {}
}

pub struct DefaultApplicationStartup;

impl ApplicationStartup for DefaultApplicationStartup {
    fn start(&self, name: &str) -> Box<dyn StartupStep> {
        Box::new(DefaultStartupStep {
            name: name.to_string(),
        })
    }
}
