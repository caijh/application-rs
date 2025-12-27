use crate::application::RustApplication;

use crate::application_run_listener::ApplicationRunListener;
use crate::bootstrap::default_bootstrap_context::DefaultBootstrapContext;
use crate::context::application_event_multi_caster::ApplicationEventMultiCaster;
use application_core::metrics::application_startup::ApplicationStartup;
use async_std::task::block_on;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct EventPublishingRunListener {
    pub initial_multicast: Arc<ApplicationEventMultiCaster>,
}

pub struct ApplicationRunListeners {
    pub application_startup: Arc<RwLock<Box<dyn ApplicationStartup>>>,
    pub listeners: Arc<RwLock<Vec<Box<dyn ApplicationRunListener>>>>,
}

impl ApplicationRunListeners {
    async fn do_with_listeners(
        &self,
        step_name: &str,
        application: &RustApplication,
        bootstrap_context: &DefaultBootstrapContext,
        f: impl Fn(&Box<dyn ApplicationRunListener>, &RustApplication, &DefaultBootstrapContext),
    ) {
        let application_startup = self.application_startup.read().await;
        let startup_step = application_startup.start(step_name);
        let guard = self.listeners.read().await;
        let listeners = guard.iter();
        for listener in listeners {
            f(listener, application, bootstrap_context);
        }
        startup_step.end();
    }

    pub async fn starting(
        &self,
        application: &RustApplication,
        bootstrap_context: &DefaultBootstrapContext,
    ) {
        self.do_with_listeners(
            "application.starting",
            application,
            bootstrap_context,
            |listener: &Box<dyn ApplicationRunListener>,
             application: &RustApplication,
             bootstrap_context: &DefaultBootstrapContext| {
                block_on(listener.starting(application, bootstrap_context));
            },
        )
        .await;
    }

    pub async fn environment_prepared(
        &self,
        application: &RustApplication,
        bootstrap_context: &DefaultBootstrapContext,
    ) {
        self.do_with_listeners(
            "application.environment-prepared",
            application,
            bootstrap_context,
            |listener: &Box<dyn ApplicationRunListener>, application, bootstrap_context| {
                block_on(listener.environment_prepared(application, bootstrap_context));
            },
        )
        .await;
    }

    pub async fn context_prepared(
        &self,
        application: &RustApplication,
        bootstrap_context: &DefaultBootstrapContext,
    ) {
        self.do_with_listeners(
            "application.context-prepared",
            application,
            bootstrap_context,
            |listener: &Box<dyn ApplicationRunListener>, application, bootstrap_context| {
                block_on(listener.context_prepared(application, bootstrap_context));
            },
        )
        .await;
    }

    pub async fn context_loaded(
        &self,
        application: &RustApplication,
        bootstrap_context: &DefaultBootstrapContext,
    ) {
        self.do_with_listeners(
            "application.context-loaded",
            application,
            bootstrap_context,
            |listener: &Box<dyn ApplicationRunListener>, application, bootstrap_context| {
                block_on(listener.context_loaded(application, bootstrap_context));
            },
        )
        .await;
    }

    pub async fn started(
        &self,
        application: &RustApplication,
        bootstrap_context: &DefaultBootstrapContext,
    ) {
        self.do_with_listeners(
            "application.started",
            application,
            bootstrap_context,
            |listener: &Box<dyn ApplicationRunListener>, application, bootstrap_context| {
                block_on(listener.started(application, bootstrap_context));
            },
        )
        .await;
    }

    pub async fn failed(
        &self,
        application: &RustApplication,
        bootstrap_context: &DefaultBootstrapContext,
    ) {
        self.do_with_listeners(
            "application.failed",
            application,
            bootstrap_context,
            |listener: &Box<dyn ApplicationRunListener>, application, bootstrap_context| {
                block_on(listener.failed(application, bootstrap_context));
            },
        )
        .await;
    }

    pub async fn stopped(
        &self,
        application: &RustApplication,
        bootstrap_context: &DefaultBootstrapContext,
    ) {
        self.do_with_listeners(
            "application.stop",
            application,
            bootstrap_context,
            |listener: &Box<dyn ApplicationRunListener>, application, bootstrap_context| {
                block_on(listener.stopped(application, bootstrap_context));
            },
        )
        .await;
    }
}
