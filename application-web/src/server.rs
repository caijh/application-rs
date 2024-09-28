use crate::handler::auto_router;
use async_std::task::block_on;
use axum::{async_trait, Router};
use axum_server::Handle;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::{signal, spawn};
use tracing::info;

#[async_trait]
pub trait WebServer: Send + Sync {
    fn get_port(&self) -> u16;

    fn start(&self, router: Router) -> Result<Arc<(Mutex<bool>, Condvar)>, Box<dyn Error>>;

    async fn stop(&self) -> Result<(), Box<dyn Error>>;
}

pub struct AxumServer {
    pub port: u16,
}

#[async_trait]
impl WebServer for AxumServer {
    fn get_port(&self) -> u16 {
        self.port
    }

    fn start(&self, router: Router) -> Result<Arc<(Mutex<bool>, Condvar)>, Box<dyn Error>> {
        let mut app = auto_router();
        app = app.merge(router);

        // run it with hyper on localhost
        let addr = SocketAddr::from(([0, 0, 0, 0], self.port));
        let tcp_listener = block_on(TcpListener::bind(addr)).unwrap();
        info!("Start axum server, listening on {}", addr);

        let pair = Arc::new((Mutex::new(false), Condvar::new()));
        spawn(Self::run(app, tcp_listener, Arc::clone(&pair)));
        Ok(pair)
    }

    async fn stop(&self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}

impl AxumServer {
    async fn run(app: Router, listener: TcpListener, condvar_pair: Arc<(Mutex<bool>, Condvar)>) {
        //Create a handle for our TLS server so the shutdown signal can all shutdown
        let handle = Handle::new();
        //save the future for easy shutting down of redirect server
        let shutdown_future = shutdown_signal(handle.clone(), condvar_pair);
        // Run the server with graceful shutdown
        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_future)
            .await
            .unwrap();
    }
}

async fn shutdown_signal(handle: Handle, condvar_pair: Arc<(Mutex<bool>, Condvar)>) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    info!("Received termination signal shutting down");
    handle.graceful_shutdown(Some(Duration::from_secs(10)));
    let (lock, cvar) = &*condvar_pair;
    let mut stopped = lock.lock().unwrap();
    *stopped = true;
    cvar.notify_one();
}
