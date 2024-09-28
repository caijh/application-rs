use crate::application::RustApplication;
use application_context::context::application_event::ApplicationEvent;
use tracing::info;

pub struct ApplicationEventMultiCaster {}

impl ApplicationEventMultiCaster {
    pub async fn multicast_event<T: ApplicationEvent>(
        &self,
        application: &RustApplication,
        event: T,
    ) {
        let listeners = application.listeners.read().await;
        let listeners = listeners.iter();
        for listener in listeners {
            if listener.is_support(&event) {
                let result = listener.on_application_event(application, &event).await;
                match result {
                    Ok(_) => {}
                    Err(e) => {
                        info!(
                            "{} listen on {:?} Event failed, {:?}",
                            listener.type_name(),
                            event.get_event_type(),
                            e
                        )
                    }
                }
            }
        }
    }
}
