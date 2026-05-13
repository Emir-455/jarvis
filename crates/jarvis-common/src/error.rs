use thiserror::Error;

#[derive(Error, Debug)]
pub enum JarvisError {
    #[error("config error: {0}")]
    Config(String),

    #[error("LLM error: {0}")]
    Llm(String),

    #[error("memory error: {0}")]
    Memory(String),

    #[error("voice error: {0}")]
    Voice(String),

    #[error("shield error: {0}")]
    Shield(String),

    #[error("cocoon error: {0}")]
    Cocoon(String),

    #[error("protocol error: {0}")]
    Protocol(String),

    #[error("biometrics error: {0}")]
    Biometrics(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("auth error: {0}")]
    Auth(String),

    #[error("subsystem '{name}' is down")]
    SubsystemDown { name: String },

    #[error("upgrade failed: {0}")]
    Upgrade(String),

    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

pub type JarvisResult<T> = Result<T, JarvisError>;
