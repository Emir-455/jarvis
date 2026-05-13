use std::collections::HashMap;

use anyhow::{bail, Result};
use chrono::{DateTime, Utc};
use tracing::info;

use jarvis_biometrics::face_verifier::{FaceVerifier, VerifyResult};
use jarvis_common::types::AccountRole;

/// Pose Protocol: Remote friend access with Face ID verification.
///
/// Access flow:
/// 1. Friend connects (WebSocket or API)
/// 2. Provide name + token
/// 3. Face verification via camera
/// 4. If verified → limited access to data in Drive/pose/ folder
/// 5. All actions logged and isolated
pub struct PoseProtocol {
    face_verifier: FaceVerifier,
    sessions: HashMap<String, PoseSession>,
}

#[derive(Debug, Clone)]
pub struct PoseSession {
    pub username: String,
    pub role: AccountRole,
    pub authenticated_at: DateTime<Utc>,
    pub face_verified: bool,
}

impl PoseProtocol {
    pub fn new(face_verifier: FaceVerifier) -> Self {
        Self {
            face_verifier,
            sessions: HashMap::new(),
        }
    }

    /// Authenticate a Pose user with token + face verification.
    pub fn authenticate(
        &mut self,
        username: &str,
        token: &str,
        face_image: Option<&[u8]>,
    ) -> Result<PoseSession> {
        // 1. Token verification
        let expected_token = std::env::var("JARVIS_POSE_TOKEN").unwrap_or_default();
        if token != expected_token && !expected_token.is_empty() {
            bail!("Invalid Pose token for user: {username}");
        }

        // 2. Face verification (if available)
        let face_verified = if let Some(image) = face_image {
            match self.face_verifier.verify_face(username, image)? {
                VerifyResult::Match { .. } => true,
                VerifyResult::NoReference => {
                    info!(username, "No face reference — skipping verification");
                    false
                }
                VerifyResult::Disabled => false,
                VerifyResult::NoMatch { .. } => {
                    bail!("Face verification failed for user: {username}");
                }
            }
        } else {
            false
        };

        let session = PoseSession {
            username: username.to_string(),
            role: AccountRole::Pose,
            authenticated_at: Utc::now(),
            face_verified,
        };

        info!(
            username,
            face_verified, "Pose Protocol: user authenticated"
        );

        self.sessions.insert(username.to_string(), session.clone());
        Ok(session)
    }

    pub fn get_session(&self, username: &str) -> Option<&PoseSession> {
        self.sessions.get(username)
    }

    pub fn revoke(&mut self, username: &str) {
        self.sessions.remove(username);
        info!(username, "Pose session revoked");
    }
}
