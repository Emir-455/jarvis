use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use tracing::{info, warn};

use jarvis_common::config::FaceVerificationConfig;

/// Face verification for Pose Protocol and access control.
///
/// Manages face encodings per user:
/// - register_face: Store reference encoding for a user
/// - verify_face: Compare incoming face against stored reference
/// - has_reference: Check if a user has registered faces
///
/// In production, this wraps a face recognition library (dlib/OpenCV).
pub struct FaceVerifier {
    config: FaceVerificationConfig,
    reference_dir: PathBuf,
    references: HashMap<String, Vec<FaceEncoding>>,
}

/// Placeholder for face encoding vector (128-dim in dlib).
#[derive(Clone)]
pub struct FaceEncoding(#[allow(dead_code)] Vec<f32>);

impl FaceVerifier {
    pub fn new(config: &FaceVerificationConfig) -> Self {
        let reference_dir = PathBuf::from(&config.reference_dir);

        let mut verifier = Self {
            config: config.clone(),
            reference_dir,
            references: HashMap::new(),
        };

        if let Err(e) = verifier.load_all_references() {
            warn!("Failed to load face references: {e}");
        }

        verifier
    }

    pub fn enabled(&self) -> bool {
        self.config.enabled
    }

    pub fn has_reference(&self, username: &str) -> bool {
        self.references
            .get(username)
            .is_some_and(|refs| !refs.is_empty())
    }

    /// Register a face encoding for a user.
    pub fn register_face(&mut self, username: &str, _image_bytes: &[u8]) -> Result<bool> {
        if !self.config.enabled {
            return Ok(false);
        }

        info!(username, "Registering face");

        // Production:
        // 1. Decode image bytes to RGB array
        // 2. Detect face locations
        // 3. Verify minimum face size
        // 4. Extract 128-dim face encoding
        // 5. Store encoding

        // Stub: create a dummy encoding
        let encoding = FaceEncoding(vec![0.0; 128]);
        self.references
            .entry(username.to_string())
            .or_default()
            .push(encoding);

        self.save_encodings(username)?;
        info!(username, "Face registered successfully");
        Ok(true)
    }

    /// Verify a face against stored references.
    pub fn verify_face(&self, username: &str, _image_bytes: &[u8]) -> Result<VerifyResult> {
        if !self.config.enabled {
            return Ok(VerifyResult::Disabled);
        }

        let refs = match self.references.get(username) {
            Some(refs) if !refs.is_empty() => refs,
            _ => return Ok(VerifyResult::NoReference),
        };

        // Production:
        // 1. Extract face encoding from image
        // 2. Compare against all stored encodings
        // 3. Use cosine similarity / Euclidean distance
        // 4. Check against threshold

        info!(
            username,
            ref_count = refs.len(),
            threshold = self.config.match_confidence,
            "Face verification (stub: auto-pass)"
        );

        Ok(VerifyResult::Match {
            confidence: 1.0,
            threshold: self.config.match_confidence,
        })
    }

    fn load_all_references(&mut self) -> Result<()> {
        if !self.reference_dir.exists() {
            std::fs::create_dir_all(&self.reference_dir)?;
            return Ok(());
        }

        for entry in std::fs::read_dir(&self.reference_dir)? {
            let entry = entry?;
            if entry.path().is_dir() {
                let username = entry.file_name().to_string_lossy().to_string();
                let encoding_path = entry.path().join("face_encodings.json");
                if encoding_path.exists() {
                    info!(username = %username, "Loading face reference");
                    // Production: deserialize actual encodings
                    self.references
                        .entry(username)
                        .or_default();
                }
            }
        }

        Ok(())
    }

    fn save_encodings(&self, username: &str) -> Result<()> {
        let user_dir = self.reference_dir.join(username);
        std::fs::create_dir_all(&user_dir)?;
        let path = user_dir.join("face_encodings.json");
        // Production: serialize actual encodings
        std::fs::write(&path, "[]")?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum VerifyResult {
    Match { confidence: f32, threshold: f32 },
    NoMatch { confidence: f32, threshold: f32 },
    NoReference,
    Disabled,
}
