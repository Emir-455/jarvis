use jarvis_common::types::AccountRole;

/// Permission checker for account roles.
pub struct AccessControl;

impl AccessControl {
    /// Get default permissions for a role.
    pub fn permissions(role: &AccountRole) -> Vec<&'static str> {
        match role {
            AccountRole::Admin => vec![
                "system.config",
                "system.upgrade",
                "system.shutdown",
                "llm.query",
                "llm.config",
                "memory.read",
                "memory.write",
                "memory.admin",
                "voice.command",
                "shield.admin",
                "dev.code_analysis",
                "dev.execute",
                "media.generate",
                "protocols.manage",
            ],
            AccountRole::Pose => vec![
                "llm.query",
                "memory.read",
                "voice.command",
            ],
            AccountRole::Guest => vec![
                "llm.query",
                "voice.command",
            ],
        }
    }

    pub fn has_permission(role: &AccountRole, permission: &str) -> bool {
        Self::permissions(role).contains(&permission)
    }
}
