use crate::application::RustApplication;
use crate::application_run_listeners::EventPublishingRunListener;
use crate::bootstrap::default_bootstrap_context::DefaultBootstrapContext;
use crate::logging::listener::ApplicationStartingEvent;
use application_context::context::application_event::{
    ApplicationContextInitializedEvent, ApplicationEnvironmentPreparedEvent,
    ApplicationFailedEvent, ApplicationPreparedEvent, ApplicationStartedEvent,
    ApplicationStoppedEvent,
};
use async_trait::async_trait;

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
    async fn context_prepared(
        &self,
        application: &RustApplication,
        bootstrap_context: &DefaultBootstrapContext,
    );
    async fn context_loaded(
        &self,
        application: &RustApplication,
        bootstrap_context: &DefaultBootstrapContext,
    );
    async fn started(
        &self,
        application: &RustApplication,
        bootstrap_context: &DefaultBootstrapContext,
    );
    async fn failed(
        &self,
        application: &RustApplication,
        bootstrap_context: &DefaultBootstrapContext,
    );
    async fn stopped(
        &self,
        application: &RustApplication,
        bootstrap_context: &DefaultBootstrapContext,
    );
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

    async fn context_prepared(
        &self,
        application: &RustApplication,
        _bootstrap_context: &DefaultBootstrapContext,
    ) {
        self.initial_multicast
            .multicast_event(application, ApplicationContextInitializedEvent {})
            .await;
    }

    async fn context_loaded(
        &self,
        application: &RustApplication,
        _bootstrap_context: &DefaultBootstrapContext,
    ) {
        self.initial_multicast
            .multicast_event(application, ApplicationPreparedEvent {})
            .await;
    }

    async fn started(
        &self,
        application: &RustApplication,
        _bootstrap_context: &DefaultBootstrapContext,
    ) {
        self.initial_multicast
            .multicast_event(application, ApplicationStartedEvent {})
            .await;
    }

    async fn failed(
        &self,
        application: &RustApplication,
        _bootstrap_context: &DefaultBootstrapContext,
    ) {
        self.initial_multicast
            .multicast_event(application, ApplicationFailedEvent {})
            .await;
    }

    async fn stopped(
        &self,
        application: &RustApplication,
        _bootstrap_context: &DefaultBootstrapContext,
    ) {
        self.initial_multicast
            .multicast_event(application, ApplicationStoppedEvent {})
            .await;
    }
}
