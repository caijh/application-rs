use std::any::Any;
use std::sync::Arc;

#[derive(Eq, PartialEq, Debug)]
pub enum ApplicationEvenType {
    Starting,
    EnvironmentPrepared,
    ContextInitialized,
    Prepared,
    Started,
    Failed,
    Stopped,
}

pub trait ApplicationEventPublisher {
    fn publish_event(&self, event: Arc<Box<dyn ApplicationEvent>>);
}

pub trait ApplicationEvent: Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn get_event_type(&self) -> ApplicationEvenType;
}

pub struct ApplicationEnvironmentPreparedEvent {}
impl ApplicationEvent for ApplicationEnvironmentPreparedEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn get_event_type(&self) -> ApplicationEvenType {
        ApplicationEvenType::EnvironmentPrepared
    }
}

pub struct ApplicationContextInitializedEvent {}

impl ApplicationEvent for ApplicationContextInitializedEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn get_event_type(&self) -> ApplicationEvenType {
        ApplicationEvenType::ContextInitialized
    }
}

pub struct ApplicationPreparedEvent {}

impl ApplicationEvent for ApplicationPreparedEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn get_event_type(&self) -> ApplicationEvenType {
        ApplicationEvenType::Prepared
    }
}

pub struct ApplicationStartedEvent {}

impl ApplicationEvent for ApplicationStartedEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn get_event_type(&self) -> ApplicationEvenType {
        ApplicationEvenType::Started
    }
}

pub struct ApplicationFailedEvent {}

impl ApplicationEvent for ApplicationFailedEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn get_event_type(&self) -> ApplicationEvenType {
        ApplicationEvenType::Failed
    }
}

pub struct ApplicationStoppedEvent {}

impl ApplicationEvent for ApplicationStoppedEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn get_event_type(&self) -> ApplicationEvenType {
        ApplicationEvenType::Stopped
    }
}
