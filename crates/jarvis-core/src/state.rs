use std::sync::atomic::{AtomicBool, Ordering};

use chrono::{DateTime, Utc};
use parking_lot::RwLock;

use jarvis_common::types::{HealthStatus, MarkVersion};

/// Global mutable system state shared across subsystems.
pub struct SystemState {
    pub mark_version: MarkVersion,
    pub booted: AtomicBool,
    pub shutdown_requested: AtomicBool,
    pub boot_time: RwLock<Option<DateTime<Utc>>>,
    pub subsystem_health: dashmap::DashMap<String, HealthStatus>,
}

impl SystemState {
    pub fn new(mark: u32) -> Self {
        Self {
            mark_version: MarkVersion(mark),
            booted: AtomicBool::new(false),
            shutdown_requested: AtomicBool::new(false),
            boot_time: RwLock::new(None),
            subsystem_health: dashmap::DashMap::new(),
        }
    }

    pub fn set_booted(&self) {
        self.booted.store(true, Ordering::SeqCst);
        *self.boot_time.write() = Some(Utc::now());
    }

    pub fn is_booted(&self) -> bool {
        self.booted.load(Ordering::SeqCst)
    }

    pub fn request_shutdown(&self) {
        self.shutdown_requested.store(true, Ordering::SeqCst);
    }

    pub fn is_shutdown_requested(&self) -> bool {
        self.shutdown_requested.load(Ordering::SeqCst)
    }

    pub fn update_health(&self, name: &str, status: HealthStatus) {
        self.subsystem_health
            .insert(name.to_string(), status);
    }

    pub fn overall_health(&self) -> HealthStatus {
        let mut worst = HealthStatus::Healthy;
        for entry in self.subsystem_health.iter() {
            match *entry.value() {
                HealthStatus::Down => return HealthStatus::Down,
                HealthStatus::Degraded => worst = HealthStatus::Degraded,
                HealthStatus::Healthy => {}
            }
        }
        worst
    }
}

impl Default for SystemState {
    fn default() -> Self {
        Self::new(1)
    }
}
