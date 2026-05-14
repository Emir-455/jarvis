use async_trait::async_trait;

use crate::error::JarvisResult;
use crate::types::HealthStatus;

/// Trait for subsystems that can be started and stopped.
#[async_trait]
pub trait Subsystem: Send + Sync {
    fn name(&self) -> &str;
    async fn start(&mut self) -> JarvisResult<()>;
    async fn stop(&mut self) -> JarvisResult<()>;
    fn health(&self) -> HealthStatus;
}

/// Trait for subsystems that must remain alive during cocoon upgrade.
pub trait CocoonGuarded {
    fn must_stay_alive(&self) -> bool;
}
