use std::collections::HashMap;

use chrono::{DateTime, Utc};
use tracing::info;

use jarvis_common::types::AccountRole;

/// Guest Protocol: Limited access for physical guests.
///
/// When Emir has a guest (e.g. "Ramos"), JARVIS:
/// 1. Detects presence or is informed
/// 2. Creates a limited session
/// 3. Only basic queries and safe commands
/// 4. No access to private data or code
pub struct GuestProtocol {
    sessions: HashMap<String, GuestSession>,
}

#[derive(Debug, Clone)]
pub struct GuestSession {
    pub guest_name: String,
    pub role: AccountRole,
    pub started_at: DateTime<Utc>,
    pub allowed_commands: Vec<String>,
}

impl GuestProtocol {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    /// Register a guest.
    pub fn register_guest(&mut self, name: &str) -> GuestSession {
        let session = GuestSession {
            guest_name: name.to_string(),
            role: AccountRole::Guest,
            started_at: Utc::now(),
            allowed_commands: default_guest_commands(),
        };

        info!(guest = name, "Guest Protocol: guest registered");
        self.sessions.insert(name.to_string(), session.clone());
        session
    }

    pub fn get_session(&self, name: &str) -> Option<&GuestSession> {
        self.sessions.get(name)
    }

    pub fn is_command_allowed(&self, name: &str, command: &str) -> bool {
        if let Some(session) = self.sessions.get(name) {
            session
                .allowed_commands
                .iter()
                .any(|c| command.starts_with(c))
        } else {
            false
        }
    }

    pub fn remove_guest(&mut self, name: &str) {
        self.sessions.remove(name);
        info!(guest = name, "Guest Protocol: guest removed");
    }
}

impl Default for GuestProtocol {
    fn default() -> Self {
        Self::new()
    }
}

fn default_guest_commands() -> Vec<String> {
    vec![
        "saat".into(),
        "hava".into(),
        "müzik".into(),
        "ışık".into(),
        "merhaba".into(),
    ]
}
