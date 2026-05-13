use tokio::sync::Notify;
use std::sync::Arc;

use tracing::info;

/// Set up signal handlers for graceful shutdown.
pub fn setup_signal_handlers(shutdown: Arc<Notify>) {
    tokio::spawn(async move {
        #[cfg(unix)]
        {
            use tokio::signal::unix::{signal, SignalKind};
            let mut sigint = signal(SignalKind::interrupt()).expect("SIGINT handler");
            let mut sigterm = signal(SignalKind::terminate()).expect("SIGTERM handler");

            tokio::select! {
                _ = sigint.recv() => info!("Received SIGINT"),
                _ = sigterm.recv() => info!("Received SIGTERM"),
            }
        }

        #[cfg(not(unix))]
        {
            tokio::signal::ctrl_c()
                .await
                .expect("Ctrl+C handler");
            info!("Received Ctrl+C");
        }

        shutdown.notify_waiters();
    });
}
