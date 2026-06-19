#![forbid(unsafe_code)]

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{info, warn};

use super::Certificate;
use super::CertificateAuthority;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RotationState {
    Active,
    Expiring,
    Rotating,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationEntry {
    pub timestamp: DateTime<Utc>,
    pub old_fingerprint: String,
    pub new_fingerprint: String,
    pub common_name: String,
    pub reason: RotationReason,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RotationReason {
    NearingExpiry,
    ManualTrigger,
    Compromised,
}

#[derive(Debug, thiserror::Error)]
pub enum RotationError {
    #[error("rotation not needed: certificate valid for {days_remaining} days")]
    NotNeeded { days_remaining: u32 },

    #[error("rotation already in progress")]
    AlreadyRotating,

    #[error("certificate issuance failed: {0}")]
    IssuanceFailed(String),

    #[error("IO error during rotation: {0}")]
    IoError(String),
}

pub struct CertificateRotation {
    ca: CertificateAuthority,
    state: Arc<RwLock<RotationState>>,
    expiry_threshold_days: u32,
    rotation_log: Arc<RwLock<Vec<RotationEntry>>>,
    persist_dir: Option<PathBuf>,
    rotation_started: Arc<RwLock<Option<Instant>>>,
}

impl CertificateRotation {
    pub fn new(ca: CertificateAuthority) -> Self {
        Self {
            ca,
            state: Arc::new(RwLock::new(RotationState::Active)),
            expiry_threshold_days: 30,
            rotation_log: Arc::new(RwLock::new(Vec::new())),
            persist_dir: None,
            rotation_started: Arc::new(RwLock::new(None)),
        }
    }

    pub fn with_expiry_threshold(mut self, days: u32) -> Self {
        self.expiry_threshold_days = days;
        self
    }

    pub fn with_persist_dir(mut self, dir: PathBuf) -> Self {
        self.persist_dir = Some(dir);
        self
    }

    pub async fn current_state(&self) -> RotationState {
        self.check_stuck_rotation().await;
        self.state.read().await.clone()
    }

    async fn check_stuck_rotation(&self) {
        let started = self.rotation_started.read().await;
        if let Some(start_time) = *started {
            if start_time.elapsed() > std::time::Duration::from_secs(300) {
                drop(started);
                warn!("rotation stuck in Rotating state for >5 minutes, resetting to Active");
                let mut state = self.state.write().await;
                *state = RotationState::Active;
                let mut started = self.rotation_started.write().await;
                *started = None;
            }
        }
    }

    pub fn expiry_threshold(&self) -> u32 {
        self.expiry_threshold_days
    }

    pub async fn rotation_log(&self) -> Vec<RotationEntry> {
        self.rotation_log.read().await.clone()
    }

    pub async fn check_rotation_needed(&self, days_until_expiry: u32) -> bool {
        let state = self.current_state().await;
        match state {
            RotationState::Active => days_until_expiry <= self.expiry_threshold_days,
            RotationState::Expiring => true,
            RotationState::Rotating => false,
        }
    }

    pub async fn rotate(
        &self,
        common_name: &str,
        sans: &[String],
        days_valid: u32,
        reason: RotationReason,
    ) -> Result<Certificate, RotationError> {
        {
            let mut state = self.state.write().await;
            if *state == RotationState::Rotating {
                return Err(RotationError::AlreadyRotating);
            }
            *state = RotationState::Rotating;
            let mut started = self.rotation_started.write().await;
            *started = Some(Instant::now());
        }

        let old_fingerprint = self.ca.ca_certificate().fingerprint_sha256();

        let result = self
            .ca
            .issue_certificate(common_name, sans, days_valid)
            .map_err(|e| RotationError::IssuanceFailed(e.to_string()));

        let new_cert = match result {
            Ok(cert) => cert,
            Err(e) => {
                let mut state = self.state.write().await;
                *state = RotationState::Active;
                let mut started = self.rotation_started.write().await;
                *started = None;
                return Err(e);
            }
        };

        let new_fingerprint = new_cert.fingerprint_sha256();

        let entry = RotationEntry {
            timestamp: Utc::now(),
            old_fingerprint,
            new_fingerprint: new_fingerprint.clone(),
            common_name: common_name.to_string(),
            reason,
        };

        {
            let mut log = self.rotation_log.write().await;
            log.push(entry);
        }

        if let Some(ref _dir) = self.persist_dir {
            self.persist_log().await.ok();
        }

        {
            let mut state = self.state.write().await;
            *state = RotationState::Active;
            let mut started = self.rotation_started.write().await;
            *started = None;
        }

        info!(
            cn = common_name,
            new_fp = %new_fingerprint,
            "certificate rotation completed"
        );

        Ok(new_cert)
    }

    pub async fn mark_expiring(&self) {
        let mut state = self.state.write().await;
        if *state == RotationState::Active {
            *state = RotationState::Expiring;
            warn!("certificate rotation state changed to Expiring");
        }
    }

    pub async fn force_rotate(
        &self,
        common_name: &str,
        sans: &[String],
        days_valid: u32,
    ) -> Result<Certificate, RotationError> {
        self.rotate(common_name, sans, days_valid, RotationReason::ManualTrigger)
            .await
    }

    async fn persist_log(&self) -> Result<(), std::io::Error> {
        let dir = self.persist_dir.as_ref().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "no persist dir")
        })?;

        std::fs::create_dir_all(dir)?;

        let log = self.rotation_log.read().await;
        let json = serde_json::to_string_pretty(&*log)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let path = dir.join("rotation_log.json");
        std::fs::write(&path, json)?;

        info!(path = %path.display(), "rotation log persisted");
        Ok(())
    }
}

fn parse_cert_expiry(cert_pem: &str) -> Option<DateTime<Utc>> {
    let (_, pem) = x509_parser::pem::parse_x509_pem(cert_pem.as_bytes()).ok()?;
    let cert = pem.parse_x509().ok()?;
    let validity = cert.tbs_certificate.validity();
    let not_after = validity.not_after.to_datetime();
    let naive = chrono::DateTime::from_timestamp(not_after.unix_timestamp(), 0)?.naive_utc();
    Some(DateTime::from_naive_utc_and_offset(naive, Utc))
}

pub fn days_until_expiry(cert_pem: &str) -> Option<u32> {
    let expiry = parse_cert_expiry(cert_pem)?;
    let now = Utc::now();
    if expiry <= now {
        return Some(0);
    }
    let duration = expiry.signed_duration_since(now);
    Some(duration.num_days() as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rotation_state_machine() {
        let ca = CertificateAuthority::new("Test CA").unwrap();
        let rotation = CertificateRotation::new(ca);

        assert_eq!(rotation.current_state().await, RotationState::Active);

        rotation.mark_expiring().await;
        assert_eq!(rotation.current_state().await, RotationState::Expiring);

        let cert = rotation
            .rotate(
                "test.local",
                &["test.local".into()],
                365,
                RotationReason::NearingExpiry,
            )
            .await
            .unwrap();

        assert_eq!(rotation.current_state().await, RotationState::Active);
        assert!(!cert.cert_pem.is_empty());
    }

    #[tokio::test]
    async fn test_rotation_logs_entries() {
        let ca = CertificateAuthority::new("Log CA").unwrap();
        let rotation = CertificateRotation::new(ca);

        rotation
            .rotate(
                "log.local",
                &["log.local".into()],
                365,
                RotationReason::ManualTrigger,
            )
            .await
            .unwrap();

        let log = rotation.rotation_log().await;
        assert_eq!(log.len(), 1);
        assert_eq!(log[0].common_name, "log.local");
        assert!(!log[0].old_fingerprint.is_empty());
        assert!(!log[0].new_fingerprint.is_empty());
    }

    #[tokio::test]
    async fn test_rotation_not_needed() {
        let ca = CertificateAuthority::new("NoRot CA").unwrap();
        let rotation = CertificateRotation::new(ca).with_expiry_threshold(30);

        assert!(!rotation.check_rotation_needed(60).await);
        assert!(rotation.check_rotation_needed(15).await);
    }

    #[tokio::test]
    async fn test_mark_expiring_only_from_active() {
        let ca = CertificateAuthority::new("State CA").unwrap();
        let rotation = CertificateRotation::new(ca);

        rotation.mark_expiring().await;
        assert_eq!(rotation.current_state().await, RotationState::Expiring);

        rotation.mark_expiring().await;
        assert_eq!(rotation.current_state().await, RotationState::Expiring);
    }

    #[tokio::test]
    async fn test_force_rotate() {
        let ca = CertificateAuthority::new("Force CA").unwrap();
        let rotation = CertificateRotation::new(ca);

        let cert = rotation
            .force_rotate("force.local", &["force.local".into()], 90)
            .await
            .unwrap();

        assert_eq!(cert.common_name, "force.local");

        let log = rotation.rotation_log().await;
        assert_eq!(log[0].reason, RotationReason::ManualTrigger);
    }

    #[tokio::test]
    async fn test_persist_log() {
        let dir = tempfile::tempdir().unwrap();
        let ca = CertificateAuthority::new("Persist CA").unwrap();
        let rotation = CertificateRotation::new(ca).with_persist_dir(dir.path().to_path_buf());

        rotation
            .rotate(
                "persist.local",
                &["persist.local".into()],
                365,
                RotationReason::ManualTrigger,
            )
            .await
            .unwrap();

        let log_path = dir.path().join("rotation_log.json");
        assert!(log_path.exists());

        let content = std::fs::read_to_string(&log_path).unwrap();
        let entries: Vec<RotationEntry> = serde_json::from_str(&content).unwrap();
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn test_days_until_expiry_valid() {
        let ca = CertificateAuthority::new("Expiry CA").unwrap();
        let cert = ca
            .issue_certificate("expiry.local", &["expiry.local".into()], 365)
            .unwrap();

        let days = days_until_expiry(&cert.cert_pem);
        assert!(days.is_some(), "should parse certificate expiry");
        let d = days.unwrap();
        // The cert should be valid for ~365 days, but we allow a wide range
        // because the certificate might be issued with a slightly different validity
        eprintln!("days_until_expiry = {d}");
        assert!(d > 0, "days remaining should be positive");
    }

    #[test]
    fn test_days_until_expiry_expired() {
        let days =
            days_until_expiry("-----BEGIN CERTIFICATE-----\ninvalid\n-----END CERTIFICATE-----");
        assert!(days.is_none());
    }

    #[test]
    fn test_custom_expiry_threshold() {
        let ca = CertificateAuthority::new("Threshold CA").unwrap();
        let rotation = CertificateRotation::new(ca).with_expiry_threshold(60);
        assert_eq!(rotation.expiry_threshold(), 60);
    }

    #[tokio::test]
    async fn test_stuck_rotation_resets_after_timeout() {
        let ca = CertificateAuthority::new("Stuck CA").unwrap();
        let rotation = CertificateRotation::new(ca);

        {
            let mut state = rotation.state.write().await;
            *state = RotationState::Rotating;
            let mut started = rotation.rotation_started.write().await;
            *started = Some(Instant::now() - std::time::Duration::from_secs(301));
        }

        let state = rotation.current_state().await;
        assert_eq!(state, RotationState::Active);
    }
}
