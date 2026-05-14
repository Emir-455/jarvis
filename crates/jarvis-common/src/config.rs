use serde::{Deserialize, Serialize};

use crate::types::HardwareProfile;

/// Top-level merged configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JarvisConfig {
    #[serde(default)]
    pub system: SystemConfig,
    #[serde(default)]
    pub hardware: HardwareProfile,
    #[serde(default)]
    pub llm: LlmConfig,
    #[serde(default)]
    pub memory: MemoryConfig,
    #[serde(default)]
    pub voice: VoiceConfig,
    #[serde(default)]
    pub security: SecurityConfig,
    #[serde(default)]
    pub cocoon: CocoonConfig,
    #[serde(default)]
    pub screen: ScreenConfig,
    #[serde(default)]
    pub daemon: DaemonConfig,
    #[serde(default)]
    pub proactive: ProactiveConfig,
    #[serde(default)]
    pub biometrics: BiometricsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub name: String,
    pub version_mark: u32,
    pub version_semver: String,
    pub creator: String,
    pub language: String,
    pub offline_mode_capable: bool,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            name: "J.A.R.V.I.S.".into(),
            version_mark: 1,
            version_semver: "2.0.1".into(),
            creator: "Emir".into(),
            language: "tr-TR".into(),
            offline_mode_capable: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub backend: String,
    pub model_path: String,
    #[serde(default)]
    pub ollama_url: String,
    #[serde(default)]
    pub ollama_model: String,
    pub n_ctx: u32,
    pub n_gpu_layers: i32,
    pub n_threads: u32,
    pub n_batch: u32,
    pub temperature: f32,
    pub top_p: f32,
    pub repeat_penalty: f32,
    pub max_tokens: u32,
    pub use_mmap: bool,
    pub use_mlock: bool,
    pub seed: i64,
    #[serde(default)]
    pub stop_tokens: Vec<String>,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            backend: "llama_cpp".into(),
            model_path: String::new(),
            ollama_url: String::new(),
            ollama_model: String::new(),
            n_ctx: 4096,
            n_gpu_layers: 35,
            n_threads: 12,
            n_batch: 1024,
            temperature: 0.6,
            top_p: 0.9,
            repeat_penalty: 1.1,
            max_tokens: 256,
            use_mmap: true,
            use_mlock: false,
            seed: -1,
            stop_tokens: vec!["</s>".into(), "[/INST]".into()],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    pub sync_interval_seconds: u64,
    #[serde(default)]
    pub drive: DriveConfig,
    #[serde(default)]
    pub vector_cache: VectorCacheConfig,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            sync_interval_seconds: 300,
            drive: DriveConfig::default(),
            vector_cache: VectorCacheConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveConfig {
    pub enabled: bool,
    pub root_folder: String,
    pub credentials_path: String,
    pub token_path: String,
    pub offline_buffer_path: String,
    pub capacity_tb: u32,
    pub request_timeout_seconds: u64,
}

impl Default for DriveConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            root_folder: "JARVIS_BRAIN".into(),
            credentials_path: String::new(),
            token_path: String::new(),
            offline_buffer_path: "data/offline_buffer.jsonl".into(),
            capacity_tb: 5,
            request_timeout_seconds: 15,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorCacheConfig {
    pub path: String,
    pub embed_model: String,
    pub top_k: u32,
    pub collection_name: String,
    pub batch_size: u32,
    pub max_cache_size_mb: u32,
    pub persist_on_shutdown: bool,
}

impl Default for VectorCacheConfig {
    fn default() -> Self {
        Self {
            path: "memory/vector/chroma".into(),
            embed_model: "sentence-transformers/all-MiniLM-L6-v2".into(),
            top_k: 6,
            collection_name: "jarvis_memory".into(),
            batch_size: 64,
            max_cache_size_mb: 512,
            persist_on_shutdown: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceConfig {
    pub enabled: bool,
    #[serde(default)]
    pub stt: SttConfig,
    #[serde(default)]
    pub tts: TtsConfig,
    #[serde(default)]
    pub wake_word: WakeWordConfig,
}

impl Default for VoiceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            stt: SttConfig::default(),
            tts: TtsConfig::default(),
            wake_word: WakeWordConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SttConfig {
    pub engine: String,
    pub model_size: String,
    pub language: String,
    pub sample_rate: u32,
    pub chunk_duration: f32,
    pub silence_timeout: f32,
    pub min_command_chunks: u32,
    pub rms_threshold: f32,
}

impl Default for SttConfig {
    fn default() -> Self {
        Self {
            engine: "whisper".into(),
            model_size: "small".into(),
            language: "tr".into(),
            sample_rate: 16000,
            chunk_duration: 1.0,
            silence_timeout: 1.5,
            min_command_chunks: 2,
            rms_threshold: 0.002,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsConfig {
    pub engine: String,
    pub voice_profile: String,
    pub output_path: String,
    pub sample_rate: u32,
    pub language: String,
}

impl Default for TtsConfig {
    fn default() -> Self {
        Self {
            engine: "xtts_v2".into(),
            voice_profile: String::new(),
            output_path: "data/last_reply.wav".into(),
            sample_rate: 24000,
            language: "tr".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WakeWordConfig {
    pub primary: String,
    #[serde(default)]
    pub variants: Vec<String>,
    #[serde(default)]
    pub fuzzy_match: FuzzyMatchConfig,
}

impl Default for WakeWordConfig {
    fn default() -> Self {
        Self {
            primary: "jarvis".into(),
            variants: vec![
                "jarvis".into(),
                "carvis".into(),
                "garvis".into(),
                "jervis".into(),
            ],
            fuzzy_match: FuzzyMatchConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuzzyMatchConfig {
    pub enabled: bool,
    pub max_distance: u32,
    pub min_word_length: u32,
}

impl Default for FuzzyMatchConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_distance: 3,
            min_word_length: 4,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct SecurityConfig {
    #[serde(default)]
    pub ws_auth: WsAuthConfig,
    #[serde(default)]
    pub antivirus: AntivirusConfig,
    #[serde(default)]
    pub sandbox: SandboxConfig,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsAuthConfig {
    pub enabled: bool,
    pub auth_timeout_seconds: u64,
    pub max_failed_attempts: u32,
    pub lockout_duration_seconds: u64,
}

impl Default for WsAuthConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auth_timeout_seconds: 5,
            max_failed_attempts: 3,
            lockout_duration_seconds: 300,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntivirusConfig {
    pub enabled: bool,
    pub always_on_during_cocoon: bool,
    #[serde(default)]
    pub watch_paths: Vec<String>,
    pub quarantine_path: String,
    pub yara_rules_path: String,
    pub auto_synthesize_antidote: bool,
}

impl Default for AntivirusConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            always_on_during_cocoon: true,
            watch_paths: Vec::new(),
            quarantine_path: "shield/quarantine".into(),
            yara_rules_path: "shield/antivirus/rules".into(),
            auto_synthesize_antidote: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub enabled: bool,
    pub max_execution_time_seconds: u64,
    pub allowed_network: bool,
    pub memory_limit_mb: u64,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_execution_time_seconds: 30,
            allowed_network: false,
            memory_limit_mb: 256,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CocoonConfig {
    pub auto_upgrade_enabled: bool,
    pub mark_threshold_loc_changed: u64,
    pub current_mark_dir: String,
    pub staging_dir: String,
    pub rollback_dir: String,
    pub marks_root: String,
    pub project_root: String,
    pub security_must_stay_alive: bool,
    pub health_check_timeout_seconds: u64,
    pub poll_interval_seconds: u64,
    pub version_semver: String,
}

impl Default for CocoonConfig {
    fn default() -> Self {
        Self {
            auto_upgrade_enabled: true,
            mark_threshold_loc_changed: 5000,
            current_mark_dir: "marks/mark_I".into(),
            staging_dir: "marks/_staging".into(),
            rollback_dir: "marks/_rollback".into(),
            marks_root: "marks".into(),
            project_root: ".".into(),
            security_must_stay_alive: true,
            health_check_timeout_seconds: 30,
            poll_interval_seconds: 60,
            version_semver: "2.0.1".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScreenConfig {
    #[serde(default)]
    pub capture: ScreenCaptureConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenCaptureConfig {
    pub interval_seconds: f32,
}

impl Default for ScreenCaptureConfig {
    fn default() -> Self {
        Self {
            interval_seconds: 5.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfig {
    pub pid_file: String,
    pub log_dir: String,
    pub crash_dump_dir: String,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            pid_file: "data/jarvis.pid".into(),
            log_dir: "logs".into(),
            crash_dump_dir: "logs/crashes".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProactiveConfig {
    pub enabled: bool,
    pub research_interval_minutes: u64,
    pub max_concurrent_tasks: u32,
}

impl Default for ProactiveConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            research_interval_minutes: 30,
            max_concurrent_tasks: 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct BiometricsConfig {
    #[serde(default)]
    pub face_verification: FaceVerificationConfig,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceVerificationConfig {
    pub enabled: bool,
    pub reference_dir: String,
    pub match_confidence: f32,
    pub min_face_size: u32,
    pub liveness_check_enabled: bool,
}

impl Default for FaceVerificationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            reference_dir: "data/biometrics/faces".into(),
            match_confidence: 0.6,
            min_face_size: 80,
            liveness_check_enabled: false,
        }
    }
}
