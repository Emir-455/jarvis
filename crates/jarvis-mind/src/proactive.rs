use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Notify;
use tracing::{info, warn};

use crate::engine::LlmEngine;

/// Proactive behavior engine — J.A.R.V.I.S. doesn't wait for commands.
///
/// Like the MCU JARVIS, this module:
/// - Monitors system health and alerts on anomalies
/// - Tracks resource usage and warns before thresholds
/// - Observes patterns and offers suggestions
/// - Reports security events immediately
pub struct ProactiveEngine {
    llm: Arc<LlmEngine>,
    alert_callback: Option<Arc<dyn Fn(ProactiveAlert) + Send + Sync>>,
    poll_interval: Duration,
}

#[derive(Debug, Clone)]
pub struct ProactiveAlert {
    pub severity: AlertSeverity,
    pub category: AlertCategory,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone)]
pub enum AlertCategory {
    SystemHealth,
    Security,
    DiskSpace,
    Performance,
    Network,
    Suggestion,
}

impl ProactiveEngine {
    pub fn new(llm: Arc<LlmEngine>, poll_interval_secs: u64) -> Self {
        Self {
            llm,
            alert_callback: None,
            poll_interval: Duration::from_secs(poll_interval_secs),
        }
    }

    pub fn set_alert_callback<F>(&mut self, callback: F)
    where
        F: Fn(ProactiveAlert) + Send + Sync + 'static,
    {
        self.alert_callback = Some(Arc::new(callback));
    }

    /// Start the proactive monitoring loop.
    pub async fn run(&self, shutdown: Arc<Notify>) {
        info!("Proaktif izleme sistemi aktif — J.A.R.V.I.S. nöbet tutuyor");

        loop {
            tokio::select! {
                _ = shutdown.notified() => {
                    info!("Proaktif izleme kapatılıyor");
                    break;
                }
                _ = tokio::time::sleep(self.poll_interval) => {
                    self.check_all().await;
                }
            }
        }
    }

    async fn check_all(&self) {
        self.check_disk_space().await;
        self.check_memory_usage().await;
        self.check_cpu_load().await;
    }

    async fn check_disk_space(&self) {
        let usage = get_disk_usage_percent();
        if usage > 95.0 {
            self.emit(ProactiveAlert {
                severity: AlertSeverity::Critical,
                category: AlertCategory::DiskSpace,
                message: format!(
                    "Efendim, disk kullanımı kritik seviyede: %{:.0}. \
                     Acil temizlik öneriyorum.",
                    usage
                ),
            });
        } else if usage > 85.0 {
            self.emit(ProactiveAlert {
                severity: AlertSeverity::Warning,
                category: AlertCategory::DiskSpace,
                message: format!(
                    "Efendim, disk kullanımı %{:.0}'e ulaştı. \
                     Gereksiz dosyaları temizlememi ister misiniz?",
                    usage
                ),
            });
        }
    }

    async fn check_memory_usage(&self) {
        let usage = get_memory_usage_percent();
        if usage > 90.0 {
            self.emit(ProactiveAlert {
                severity: AlertSeverity::Critical,
                category: AlertCategory::SystemHealth,
                message: format!(
                    "Efendim, RAM kullanımı %{:.0}. \
                     Bellek sızıntısı olabilir — detaylı analiz yapıyorum.",
                    usage
                ),
            });
        } else if usage > 80.0 {
            self.emit(ProactiveAlert {
                severity: AlertSeverity::Warning,
                category: AlertCategory::Performance,
                message: format!(
                    "Efendim, bellek kullanımı %{:.0}. \
                     İzlemeye devam ediyorum.",
                    usage
                ),
            });
        }
    }

    async fn check_cpu_load(&self) {
        // CPU load check placeholder — reads /proc/loadavg on Linux
        let load = get_cpu_load_percent();
        if load > 95.0 {
            self.emit(ProactiveAlert {
                severity: AlertSeverity::Warning,
                category: AlertCategory::Performance,
                message: format!(
                    "Efendim, CPU yükü %{:.0}. \
                     Performans düşüşü yaşanabilir.",
                    load
                ),
            });
        }
    }

    fn emit(&self, alert: ProactiveAlert) {
        match alert.severity {
            AlertSeverity::Critical => warn!("[JARVIS ALERT] {}", alert.message),
            AlertSeverity::Warning => warn!("[JARVIS] {}", alert.message),
            AlertSeverity::Info => info!("[JARVIS] {}", alert.message),
        }

        if let Some(ref cb) = self.alert_callback {
            cb(alert);
        }
    }

    /// Allow unused field — LLM will be used for intelligent alert generation
    #[allow(dead_code)]
    fn _llm(&self) -> &LlmEngine {
        &self.llm
    }
}

// ── System metrics helpers ─────────────────────────────────────────

fn get_disk_usage_percent() -> f64 {
    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = std::process::Command::new("df")
            .args(["-h", "/"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Some(line) = stdout.lines().nth(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 5 {
                    if let Some(pct) = parts[4].strip_suffix('%') {
                        return pct.parse().unwrap_or(0.0);
                    }
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(output) = std::process::Command::new("powershell")
            .args(["-Command", "Get-PSDrive C | Select-Object -ExpandProperty Used"])
            .output()
        {
            let _ = output; // Parse Windows disk info
        }
    }

    0.0
}

fn get_memory_usage_percent() -> f64 {
    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
            let mut total: u64 = 0;
            let mut available: u64 = 0;
            for line in content.lines() {
                if line.starts_with("MemTotal:") {
                    total = parse_meminfo_kb(line);
                } else if line.starts_with("MemAvailable:") {
                    available = parse_meminfo_kb(line);
                }
            }
            if total > 0 {
                return ((total - available) as f64 / total as f64) * 100.0;
            }
        }
    }

    0.0
}

fn get_cpu_load_percent() -> f64 {
    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = std::fs::read_to_string("/proc/loadavg") {
            if let Some(load_1min) = content.split_whitespace().next() {
                let load: f64 = load_1min.parse().unwrap_or(0.0);
                // Normalize to percentage based on core count (6 for 7600X)
                let cores = 6.0_f64;
                return (load / cores * 100.0).min(100.0);
            }
        }
    }

    0.0
}

#[cfg(target_os = "linux")]
fn parse_meminfo_kb(line: &str) -> u64 {
    line.split_whitespace()
        .nth(1)
        .and_then(|v| v.parse().ok())
        .unwrap_or(0)
}
