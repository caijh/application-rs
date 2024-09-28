use crate::application::RustApplication;

use crate::context::application_event_multi_caster::ApplicationEventMultiCaster;
use crate::context::bootstrap_context::{BootstrapContext, DefaultBootstrapContext};
use crate::logging::listener::ApplicationStartingEvent;
use application_context::context::application_event::{
    ApplicationContextInitializedEvent, ApplicationEnvironmentPreparedEvent,
    ApplicationFailedEvent, ApplicationPreparedEvent, ApplicationStartedEvent,
    ApplicationStoppedEvent,
};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

#[async_trait]
pub trait ApplicationRunListener: Send + Sync {
    async fn starting(
        &self,
        application: &RustApplication,
        bootstrap_context: &DefaultBootstrapContext,
    );
    async fn environment_prepared(
        &self,
        application: &RustApplication,
        bootstrap_context: &DefaultBootstrapContext,
    );
    async fn context_prepared(&self, application: &RustApplication);
    async fn context_loaded(&self, application: &RustApplication);
    async fn started(&self, application: &RustApplication);
    async fn failed(&self, application: &RustApplication);
    async fn stopped(&self, application: &RustApplication);
}

pub struct EventPublishingRunListener {
    pub initial_multicast: Arc<ApplicationEventMultiCaster>,
}

#[async_trait]
impl ApplicationRunListener for EventPublishingRunListener {
    async fn starting(
        &self,
        application: &RustApplication,
        bootstrap_context: &DefaultBootstrapContext,
    ) {
        let properties = bootstrap_context.get_bootstrap_properties().clone();
        self.initial_multicast
            .multicast_event(
                application,
                ApplicationStartingEvent {
                    bootstrap_properties: properties,
                },
            )
            .await;
    }

    async fn environment_prepared(
        &self,
        application: &RustApplication,
        _bootstrap_context: &DefaultBootstrapContext,
    ) {
        self.initial_multicast
            .multicast_event(application, ApplicationEnvironmentPreparedEvent {})
            .await;
    }

    async fn context_prepared(&self, application: &RustApplication) {
        self.initial_multicast
            .multicast_event(application, ApplicationContextInitializedEvent {})
            .await;
    }

    async fn context_loaded(&self, application: &RustApplication) {
        self.initial_multicast
            .multicast_event(application, ApplicationPreparedEvent {})
            .await;
    }

    async fn started(&self, application: &RustApplication) {
        self.initial_multicast
            .multicast_event(application, ApplicationStartedEvent {})
            .await;
    }

    async fn failed(&self, application: &RustApplication) {
        self.initial_multicast
            .multicast_event(application, ApplicationFailedEvent {})
            .await;
    }

    async fn stopped(&self, application: &RustApplication) {
        self.initial_multicast
            .multicast_event(application, ApplicationStoppedEvent {})
            .await;
    }
}

pub struct ApplicationRunListeners {
    pub listeners: Arc<RwLock<Vec<Box<dyn ApplicationRunListener>>>>,
}

impl ApplicationRunListeners {
    pub async fn starting(
        &self,
        application: &RustApplication,
        bootstrap_context: &DefaultBootstrapContext,
    ) {
        let guard = self.listeners.read().await;
        let listeners = guard.iter();
        for listener in listeners {
            listener.starting(application, bootstrap_context).await;
        }
    }

    pub async fn environment_prepared(
        &self,
        application: &RustApplication,
        bootstrap_context: &DefaultBootstrapContext,
    ) {
        let guard = self.listeners.read().await;
        let listeners = guard.iter();
        for listener in listeners {
            listener
                .environment_prepared(application, bootstrap_context)
                .await;
        }
    }

    pub async fn context_prepared(&self, application: &RustApplication) {
        let guard = self.listeners.read().await;
        let listeners = guard.iter();
        for listener in listeners {
            listener.context_prepared(application).await;
        }
    }

    pub async fn context_loaded(&self, application: &RustApplication) {
        let guard = self.listeners.read().await;
        let listeners = guard.iter();
        for listener in listeners {
            listener.context_loaded(application).await;
        }
    }

    pub async fn started(&self, application: &RustApplication) {
        let guard = self.listeners.read().await;
        let listeners = guard.iter();
        for listener in listeners {
            listener.started(application).await;
        }
    }

    pub async fn failed(&self, application: &RustApplication) {
        let guard = self.listeners.read().await;
        let listeners = guard.iter();
        for listener in listeners {
            listener.failed(application).await;
        }
    }

    pub async fn stopped(&self, application: &RustApplication) {
        info!("Application Going stopped!");
        let guard = self.listeners.read().await;
        let listeners = guard.iter();
        for listener in listeners {
            listener.stopped(application).await;
        }
    }
}
