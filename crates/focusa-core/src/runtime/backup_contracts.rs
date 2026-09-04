//! Versioned Focusa backup policy, manifest, receipt, and health contracts.

use anyhow::{Result, bail};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

pub const BACKUP_POLICY_SCHEMA: &str = "focusa.backup_policy.v1";
pub const BACKUP_MANIFEST_SCHEMA: &str = "focusa.backup_generation_manifest.v1";
pub const BACKUP_RECEIPT_SCHEMA: &str = "focusa.backup_receipt.v1";
pub const BACKUP_HEALTH_SCHEMA: &str = "focusa.backup_health.v1";
pub const PRUNE_DECISION_SCHEMA: &str = "focusa.backup_prune_decision.v1";
pub const RESTORE_RECEIPT_SCHEMA: &str = "focusa.backup_restore_receipt.v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackupPolicy {
    pub schema: String,
    pub enabled: bool,
    pub backup_root: PathBuf,
    pub rpo_seconds: u64,
    pub rto_seconds: u64,
    pub full_interval_seconds: u64,
    pub incremental_interval_seconds: u64,
    pub keep_hourly: usize,
    pub keep_daily: usize,
    pub keep_weekly: usize,
    pub keep_monthly: usize,
    pub restore_interval_seconds: u64,
    pub local_required: bool,
    pub off_host_required: bool,
    pub off_host_remote: Option<String>,
    pub min_free_bytes: u64,
    pub min_free_percent: u8,
    pub max_concurrent_operations: u8,
    pub compression: String,
    pub compression_level: i32,
    pub incremental_strategy: String,
    pub policy_digest: String,
}

impl BackupPolicy {
    pub fn from_env(data_dir: &Path) -> Result<Self> {
        let enabled = env_bool("FOCUSA_BACKUP_ENABLED", false)?;
        let configured_root = std::env::var_os("FOCUSA_BACKUP_ROOT").map(PathBuf::from);
        if enabled && configured_root.is_none() {
            bail!("FOCUSA_BACKUP_ROOT is required when backups are enabled")
        }
        let root = configured_root.unwrap_or_else(|| {
            data_dir
                .parent()
                .unwrap_or(data_dir)
                .join("focusa-backups-disabled")
        });
        let mut policy = Self {
            schema: BACKUP_POLICY_SCHEMA.to_string(),
            enabled,
            backup_root: root,
            rpo_seconds: 900,
            rto_seconds: 7_200,
            full_interval_seconds: env_u64("FOCUSA_BACKUP_FULL_INTERVAL_SECS", 3_600)?,
            incremental_interval_seconds: env_u64("FOCUSA_BACKUP_INCREMENTAL_INTERVAL_SECS", 900)?,
            keep_hourly: 24,
            keep_daily: 14,
            keep_weekly: 8,
            keep_monthly: 12,
            restore_interval_seconds: env_u64("FOCUSA_BACKUP_RESTORE_INTERVAL_SECS", 604_800)?,
            local_required: true,
            off_host_required: env_bool("FOCUSA_BACKUP_OFF_HOST_REQUIRED", true)?,
            off_host_remote: std::env::var("FOCUSA_BACKUP_OFF_HOST_REMOTE").ok(),
            min_free_bytes: env_u64("FOCUSA_BACKUP_MIN_FREE_BYTES", 10 * 1024 * 1024 * 1024)?,
            min_free_percent: env_u64("FOCUSA_BACKUP_MIN_FREE_PERCENT", 10)?
                .try_into()
                .map_err(|_| anyhow::anyhow!("FOCUSA_BACKUP_MIN_FREE_PERCENT exceeds 255"))?,
            max_concurrent_operations: 1,
            compression: "zstd".to_string(),
            compression_level: env_i32("FOCUSA_BACKUP_ZSTD_LEVEL", 3)?,
            incremental_strategy: std::env::var("FOCUSA_BACKUP_INCREMENTAL_STRATEGY")
                .unwrap_or_else(|_| "required_not_implemented".to_string()),
            policy_digest: String::new(),
        };
        policy.validate(data_dir)?;
        policy.policy_digest = digest_serializable(&policy)?;
        Ok(policy)
    }

    pub fn validate(&self, data_dir: &Path) -> Result<()> {
        if self.schema != BACKUP_POLICY_SCHEMA {
            bail!("unsupported backup policy schema")
        }
        if self.rpo_seconds != 900 || self.rto_seconds != 7_200 {
            bail!("backup recovery targets must remain 15-minute RPO and 2-hour RTO")
        }
        if self.min_free_percent > 100 {
            bail!("backup minimum free percent must be at most 100")
        }
        if self.full_interval_seconds == 0
            || self.incremental_interval_seconds == 0
            || self.restore_interval_seconds == 0
            || self.max_concurrent_operations != 1
        {
            bail!("backup cadence and concurrency policy are invalid")
        }
        if self.keep_hourly != 24
            || self.keep_daily != 14
            || self.keep_weekly != 8
            || self.keep_monthly != 12
        {
            bail!("backup retention differs from approved policy")
        }
        if !self.backup_root.is_absolute() || self.backup_root.starts_with(data_dir) {
            bail!("backup root must be absolute and outside live data directory")
        }
        if self.compression != "zstd" || !(-7..=19).contains(&self.compression_level) {
            bail!("unsupported backup compression policy")
        }
        if !matches!(
            self.incremental_strategy.as_str(),
            "required_not_implemented" | "experimental_full_snapshot_chunks_v0"
        ) {
            bail!("unsupported backup incremental strategy")
        }
        if let Some(remote) = &self.off_host_remote {
            let valid = remote.split_once(':').is_some_and(|(name, prefix)| {
                !name.is_empty()
                    && name
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
                    && !prefix.is_empty()
                    && !prefix.starts_with('/')
                    && !prefix.contains(':')
                    && !prefix.contains("..")
                    && prefix.bytes().all(|byte| {
                        byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'/' | b'.')
                    })
            });
            if remote.len() > 240 || !valid {
                bail!("FOCUSA_BACKUP_OFF_HOST_REMOTE is invalid")
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackupChunkRef {
    pub index: u64,
    pub offset: u64,
    pub uncompressed_bytes: u64,
    pub content_sha256: String,
    pub storage_key: String,
    pub compressed_bytes: u64,
    pub compressed_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackupArtifact {
    pub path: String,
    pub bytes: u64,
    pub sha256: String,
    pub media_type: String,
    pub compression: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupGenerationManifest {
    pub schema: String,
    pub generation_id: String,
    pub slot_id: String,
    pub generation_kind: String,
    pub state: String,
    pub created_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub source_database: String,
    pub source_file_identity: String,
    pub runtime_version: String,
    pub schema_version: Option<String>,
    pub page_size: u64,
    pub page_count: u64,
    pub event_count: u64,
    pub event_chain_index: Option<i64>,
    pub event_chain_hash: Option<String>,
    pub persisted_chain_anchor: Option<String>,
    pub ecs_inventory_digest: String,
    pub cold_export_inventory_digest: String,
    pub artifacts: Vec<BackupArtifact>,
    pub chunks: Vec<BackupChunkRef>,
    pub source_snapshot_sha256: String,
    pub parent_generation_id: Option<String>,
    pub policy_digest: String,
    pub off_host_settlement_ref: Option<String>,
    pub restore_receipt_ref: Option<String>,
    pub manifest_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupReceipt {
    pub schema: String,
    pub run_id: String,
    pub generation_id: String,
    pub slot_id: String,
    pub phase: String,
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub policy_digest: String,
    pub source_database: String,
    pub bytes: u64,
    pub artifact_sha256: Option<String>,
    pub quick_check: Option<String>,
    pub error_code: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupHealth {
    pub schema: String,
    pub enabled: bool,
    pub policy_digest: String,
    pub backup_root: String,
    pub last_verified_generation_id: Option<String>,
    pub last_verified_at: Option<DateTime<Utc>>,
    pub verified_generation_count: usize,
    pub free_bytes: Option<u64>,
    pub overall_status: String,
    pub full_status: String,
    pub rpo_status: String,
    pub off_host_status: String,
    pub restore_status: String,
    pub last_failure: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupOffHostReceipt {
    pub schema: String,
    pub settlement_id: String,
    pub generation_id: String,
    pub manifest_sha256: String,
    pub remote: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub status: String,
    pub verification: String,
    pub files: usize,
    pub bytes: u64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupPruneDecision {
    pub schema: String,
    pub policy_digest: String,
    pub decision: String,
    pub retained_generation_ids: Vec<String>,
    pub candidate_generation_ids: Vec<String>,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupPruneReceipt {
    pub schema: String,
    pub prune_id: String,
    pub policy_digest: String,
    pub plan_digest: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub status: String,
    pub deleted_generation_ids: Vec<String>,
    pub deleted_chunk_count: usize,
    pub reclaimed_bytes: u64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupRestoreReceipt {
    pub schema: String,
    pub restore_id: String,
    pub generation_id: String,
    pub generation_kind: String,
    pub manifest_sha256: String,
    pub isolated_target: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub restored_sha256: Option<String>,
    pub quick_check: Option<String>,
    pub event_count: Option<u64>,
    pub event_chain_index: Option<i64>,
    pub event_chain_hash: Option<String>,
    pub elapsed_seconds: u64,
    pub rto_status: String,
    pub status: String,
    pub error: Option<String>,
}

pub(crate) fn digest_serializable<T: Serialize + Clone>(value: &T) -> Result<String> {
    let mut json = serde_json::to_value(value)?;
    if let Some(object) = json.as_object_mut() {
        if object.contains_key("manifest_sha256") {
            object.remove("manifest_sha256");
        } else {
            object.remove("policy_digest");
        }
    }
    Ok(hex::encode(Sha256::digest(serde_json::to_vec(&json)?)))
}

fn env_u64(name: &str, default: u64) -> Result<u64> {
    match std::env::var(name) {
        Ok(value) => value
            .parse::<u64>()
            .map_err(|error| anyhow::anyhow!("{name}: {error}")),
        Err(std::env::VarError::NotPresent) => Ok(default),
        Err(error) => Err(error.into()),
    }
}
fn env_i32(name: &str, default: i32) -> Result<i32> {
    match std::env::var(name) {
        Ok(value) => value
            .parse::<i32>()
            .map_err(|error| anyhow::anyhow!("{name}: {error}")),
        Err(std::env::VarError::NotPresent) => Ok(default),
        Err(error) => Err(error.into()),
    }
}
fn env_bool(name: &str, default: bool) -> Result<bool> {
    match std::env::var(name) {
        Ok(v) if v == "1" || v.eq_ignore_ascii_case("true") => Ok(true),
        Ok(v) if v == "0" || v.eq_ignore_ascii_case("false") => Ok(false),
        Ok(_) => bail!("{name} must be true/false or 1/0"),
        Err(std::env::VarError::NotPresent) => Ok(default),
        Err(e) => Err(e.into()),
    }
}
