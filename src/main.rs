//! J.A.R.V.I.S. 2.0 (Mark I) — Autonomous Personal AI Assistant
//!
//! Boot sequence (mirrors Python daemon_run):
//! 1. Config load
//! 2. Logging init
//! 3. Daemon PID write
//! 4. Signal handlers
//! 5. Kernel create
//! 6. Shield (antivirus) start
//! 7. Memory connect (RAG + Drive)
//! 8. LLM load
//! 9. Voice warmup + STT listener
//! 10. Cocoon watcher
//! 11. HTTP/WS server
//! 12. Health monitor loop

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use tokio::sync::Notify;
use tracing::{error, info};

fn main() -> Result<()> {
    // Build tokio runtime with hardware-optimized thread count
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(6) // Match 6 physical cores of Ryzen 5 7600X
        .enable_all()
        .build()?;

    runtime.block_on(async_main())
}

async fn async_main() -> Result<()> {
    // ── 1. Config ──────────────────────────────────────────────
    let config = match jarvis_config::load_config() {
        Ok(cfg) => {
            println!(
                "[BOOT] Config loaded: {} {} ({})",
                cfg.system.name, cfg.system.version_semver, cfg.system.creator
            );
            cfg
        }
        Err(e) => {
            eprintln!("[BOOT] Config load failed ({e}), using defaults");
            jarvis_common::config::JarvisConfig::default()
        }
    };

    // ── 2. Logging ─────────────────────────────────────────────
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,jarvis=debug".into()),
        )
        .with_target(true)
        .with_thread_names(true)
        .init();

    info!(
        name = %config.system.name,
        mark = config.system.version_mark,
        version = %config.system.version_semver,
        creator = %config.system.creator,
        "=== J.A.R.V.I.S. 2.0 BOOT SEQUENCE ==="
    );

    // ── 3. Daemon PID ──────────────────────────────────────────
    let daemon = jarvis_daemon::Daemon::new(&config.daemon.pid_file);
    daemon.start()?;
    info!("Daemon PID written");

    // ── 4. Signal handlers ─────────────────────────────────────
    let shutdown = Arc::new(Notify::new());
    jarvis_daemon::signals::setup_signal_handlers(Arc::clone(&shutdown));

    // ── 5–8. Kernel (Shield, Memory, LLM) ──────────────────────
    let mut kernel = jarvis_core::kernel::JarvisKernel::new(config.clone());
    if let Err(e) = kernel.boot().await {
        error!("Kernel boot failed: {e}");
    }

    // ── 9. Voice ───────────────────────────────────────────────
    let voice = Arc::new(jarvis_voice::manager::VoiceManager::new(&config.voice));
    let router_for_voice = Arc::clone(&kernel.router);
    let voice_callback = Arc::new(move |text: String| {
        let router = Arc::clone(&router_for_voice);
        tokio::spawn(async move {
            match router.handle(&text).await {
                Ok(response) => info!(response = %response, "Voice command handled"),
                Err(e) => error!("Voice command error: {e}"),
            }
        });
    });

    if let Err(e) = voice.start(voice_callback).await {
        error!("Voice start failed: {e}");
    }
    kernel.attach_voice(Arc::clone(&voice));

    // ── 10. Cocoon watcher ─────────────────────────────────────
    let cocoon = Arc::clone(&kernel.cocoon);
    let cocoon_shutdown = Arc::clone(&shutdown);
    let cocoon_poll_secs = config.cocoon.poll_interval_seconds;
    let proactive_interval_secs = config.proactive.research_interval_minutes as u64 * 60;
    tokio::spawn(async move {
        if let Err(e) = cocoon.init().await {
            error!("Cocoon init failed: {e}");
            return;
        }
        loop {
            tokio::select! {
                _ = cocoon_shutdown.notified() => break,
                _ = tokio::time::sleep(std::time::Duration::from_secs(cocoon_poll_secs)) => {
                    if let Err(e) = cocoon.poll().await {
                        error!("Cocoon poll error: {e}");
                    }
                }
            }
        }
    });

    // ── 10b. Proactive monitoring ─────────────────────────────
    let proactive_llm = Arc::clone(&kernel.llm);
    let proactive_shutdown = Arc::clone(&shutdown);
    tokio::spawn(async move {
        let engine = jarvis_mind::proactive::ProactiveEngine::new(
            proactive_llm,
            proactive_interval_secs,
        );
        engine.run(proactive_shutdown).await;
    });

    // ── 11. HTTP/WS server ─────────────────────────────────────
    let addr: SocketAddr = "127.0.0.1:8080".parse()?;
    let server_shutdown = Arc::clone(&shutdown);
    let system = Arc::clone(&kernel.state);
    let router = Arc::clone(&kernel.router);

    tokio::spawn(async move {
        if let Err(e) =
            jarvis_core::server::start_server(system, router, addr, server_shutdown).await
        {
            error!("Server error: {e}");
        }
    });

    info!(
        "=== J.A.R.V.I.S. 2.0 (Mark {}) ONLINE ===",
        kernel.state.mark_version.roman()
    );
    info!("Health: http://127.0.0.1:8080/health");
    info!("WebSocket: ws://127.0.0.1:8080/ws");

    // ── 12. Wait for shutdown ──────────────────────────────────
    shutdown.notified().await;
    info!("=== SHUTDOWN SEQUENCE ===");

    kernel.shutdown().await?;
    daemon.stop();

    info!("=== J.A.R.V.I.S. OFFLINE ===");
    Ok(())
}
