use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// System-wide event bus message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JarvisEvent {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub kind: EventKind,
    pub payload: serde_json::Value,
}

impl JarvisEvent {
    pub fn new(kind: EventKind, payload: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            kind,
            payload,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    // Lifecycle
    SystemBoot,
    SystemShutdown,
    SubsystemReady,
    SubsystemDown,

    // Voice
    WakeWordDetected,
    VoiceCommandReceived,
    TtsSpeaking,
    TtsDone,

    // LLM
    LlmLoaded,
    LlmResponse,
    LlmError,

    // Memory
    MemoryConnected,
    MemorySynced,
    RagQueryComplete,

    // Shield / Security
    ThreatDetected,
    Quarantined,
    AntidoteSynthesized,
    ShieldStatus,

    // Cocoon / Upgrade
    PatchDetected,
    CocoonEnter,
    CocoonExit,
    MarkUpgrade,
    HotPatchApplied,

    // Protocols
    PoseAuth,
    GuestAuth,
    AuthFailed,

    // Developer
    CodeAnalysisComplete,
    OptimizationApplied,

    // Media
    MediaGenerated,

    // Health
    HealthCheck,
}

/// Typed event bus using tokio broadcast.
pub struct EventBus {
    tx: tokio::sync::broadcast::Sender<JarvisEvent>,
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = tokio::sync::broadcast::channel(capacity);
        Self { tx }
    }

    pub fn publish(&self, event: JarvisEvent) {
        let _ = self.tx.send(event);
    }

    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<JarvisEvent> {
        self.tx.subscribe()
    }

    pub fn emit(&self, kind: EventKind, payload: serde_json::Value) {
        self.publish(JarvisEvent::new(kind, payload));
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(256)
    }
}
