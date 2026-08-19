use std::{
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use uuid::Uuid;

pub const LICENSE_MIGRATION_SCHEMA: &str = "focusa.license_migration_journal.v1";
const GENESIS_HASH: &str =
    "sha256:0000000000000000000000000000000000000000000000000000000000000000";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LegacyLicenseSourceClass {
    PaidKeyRecord,
    EvaluationRecord,
    LegacyAuthorityRecord,
    LegacyReceipt,
    UnknownRecord,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LegacyLicenseInventoryItem {
    pub source_class: LegacyLicenseSourceClass,
    pub source_path_label: String,
    pub source_digest: String,
    pub byte_length: u64,
    pub modified_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LicenseMigrationStatus {
    Discovered,
    AwaitingAuthority,
    AuthorityIssued,
    Committed,
    SoftwareRolledBack,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LicenseMigrationJournalEntry {
    pub schema: String,
    pub migration_id: Uuid,
    pub sequence: u64,
    pub source_class: LegacyLicenseSourceClass,
    pub source_digest: String,
    pub status: LicenseMigrationStatus,
    pub authority_lease_id: Option<String>,
    pub authority_lease_sequence: Option<u64>,
    pub authority_lease_digest: Option<String>,
    pub preserved_data_refs: Vec<String>,
    pub evidence_refs: Vec<String>,
    pub observed_at: DateTime<Utc>,
    pub previous_entry_hash: String,
    pub entry_hash: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationAppendOutcome {
    Appended,
    IdempotentReplay,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum LicenseMigrationError {
    #[error("legacy license inventory cannot be read")]
    InventoryRead,
    #[error("migration journal is invalid or tampered")]
    JournalIntegrity,
    #[error("migration transition is invalid")]
    InvalidTransition,
    #[error("software rollback attempted to remove authority truth")]
    AuthorityRollbackForbidden,
    #[error("migration journal cannot be persisted")]
    JournalWrite,
}

pub fn migration_id_for_source_digest(source_digest: &str) -> Uuid {
    let digest = Sha256::digest(source_digest.as_bytes());
    let mut bytes = [0_u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    // RFC 4122 variant with deterministic version-8 application namespace.
    bytes[6] = (bytes[6] & 0x0f) | 0x80;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Uuid::from_bytes(bytes)
}

pub fn inventory_legacy_license_files(
    candidates: &[(LegacyLicenseSourceClass, PathBuf)],
) -> Result<Vec<LegacyLicenseInventoryItem>, LicenseMigrationError> {
    let mut inventory = Vec::new();
    for (source_class, path) in candidates {
        if !path.exists() {
            continue;
        }
        let bytes = std::fs::read(path).map_err(|_| LicenseMigrationError::InventoryRead)?;
        let metadata = std::fs::metadata(path).map_err(|_| LicenseMigrationError::InventoryRead)?;
        inventory.push(LegacyLicenseInventoryItem {
            source_class: *source_class,
            source_path_label: path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("legacy-license-record")
                .to_string(),
            source_digest: format!("sha256:{:x}", Sha256::digest(&bytes)),
            byte_length: metadata.len(),
            modified_at: metadata.modified().ok().map(DateTime::<Utc>::from),
        });
    }
    inventory.sort_by(|left, right| left.source_path_label.cmp(&right.source_path_label));
    Ok(inventory)
}

pub fn read_license_migration_journal(
    path: &Path,
) -> Result<Vec<LicenseMigrationJournalEntry>, LicenseMigrationError> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content =
        std::fs::read_to_string(path).map_err(|_| LicenseMigrationError::JournalIntegrity)?;
    let mut entries = Vec::new();
    let mut previous = GENESIS_HASH.to_string();
    for line in content.lines().filter(|line| !line.trim().is_empty()) {
        let entry: LicenseMigrationJournalEntry =
            serde_json::from_str(line).map_err(|_| LicenseMigrationError::JournalIntegrity)?;
        if entry.schema != LICENSE_MIGRATION_SCHEMA
            || entry.sequence != entries.len() as u64 + 1
            || entry.previous_entry_hash != previous
            || entry.entry_hash != compute_entry_hash(&entry)
        {
            return Err(LicenseMigrationError::JournalIntegrity);
        }
        previous = entry.entry_hash.clone();
        entries.push(entry);
    }
    Ok(entries)
}

pub fn append_license_migration_entry(
    path: &Path,
    mut candidate: LicenseMigrationJournalEntry,
) -> Result<MigrationAppendOutcome, LicenseMigrationError> {
    let entries = read_license_migration_journal(path)?;
    if let Some(existing) = entries.iter().find(|entry| {
        entry.migration_id == candidate.migration_id && entry.status == candidate.status
    }) {
        return if same_transition(existing, &candidate) {
            Ok(MigrationAppendOutcome::IdempotentReplay)
        } else {
            Err(LicenseMigrationError::InvalidTransition)
        };
    }
    let previous_for_migration = entries
        .iter()
        .rev()
        .find(|entry| entry.migration_id == candidate.migration_id);
    validate_transition(previous_for_migration, &candidate)?;
    candidate.schema = LICENSE_MIGRATION_SCHEMA.into();
    candidate.sequence = entries.len() as u64 + 1;
    candidate.previous_entry_hash = entries
        .last()
        .map(|entry| entry.entry_hash.clone())
        .unwrap_or_else(|| GENESIS_HASH.into());
    candidate.entry_hash = compute_entry_hash(&candidate);

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|_| LicenseMigrationError::JournalWrite)?;
    }
    let mut options = OpenOptions::new();
    options.create(true).append(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(path)
        .map_err(|_| LicenseMigrationError::JournalWrite)?;
    let mut encoded =
        serde_json::to_vec(&candidate).map_err(|_| LicenseMigrationError::JournalWrite)?;
    encoded.push(b'\n');
    file.write_all(&encoded)
        .and_then(|_| file.sync_all())
        .map_err(|_| LicenseMigrationError::JournalWrite)?;
    Ok(MigrationAppendOutcome::Appended)
}

fn validate_transition(
    previous: Option<&LicenseMigrationJournalEntry>,
    candidate: &LicenseMigrationJournalEntry,
) -> Result<(), LicenseMigrationError> {
    let allowed = matches!(
        (previous.map(|entry| entry.status), candidate.status),
        (None, LicenseMigrationStatus::Discovered)
            | (
                Some(LicenseMigrationStatus::Discovered),
                LicenseMigrationStatus::AwaitingAuthority
            )
            | (
                Some(LicenseMigrationStatus::AwaitingAuthority),
                LicenseMigrationStatus::AuthorityIssued
            )
            | (
                Some(LicenseMigrationStatus::AuthorityIssued),
                LicenseMigrationStatus::Committed
            )
            | (
                Some(LicenseMigrationStatus::Committed),
                LicenseMigrationStatus::SoftwareRolledBack
            )
    );
    if !allowed {
        return Err(LicenseMigrationError::InvalidTransition);
    }
    if matches!(
        candidate.status,
        LicenseMigrationStatus::AuthorityIssued
            | LicenseMigrationStatus::Committed
            | LicenseMigrationStatus::SoftwareRolledBack
    ) && (candidate
        .authority_lease_id
        .as_deref()
        .is_none_or(str::is_empty)
        || candidate
            .authority_lease_sequence
            .is_none_or(|sequence| sequence == 0)
        || candidate
            .authority_lease_digest
            .as_deref()
            .is_none_or(|digest| !digest.starts_with("sha256:")))
    {
        return Err(LicenseMigrationError::InvalidTransition);
    }
    if candidate.status == LicenseMigrationStatus::SoftwareRolledBack {
        let previous = previous.ok_or(LicenseMigrationError::AuthorityRollbackForbidden)?;
        if candidate.authority_lease_id != previous.authority_lease_id
            || candidate.authority_lease_sequence != previous.authority_lease_sequence
            || candidate.authority_lease_digest != previous.authority_lease_digest
        {
            return Err(LicenseMigrationError::AuthorityRollbackForbidden);
        }
    }
    Ok(())
}

fn same_transition(
    left: &LicenseMigrationJournalEntry,
    right: &LicenseMigrationJournalEntry,
) -> bool {
    left.source_class == right.source_class
        && left.source_digest == right.source_digest
        && left.authority_lease_id == right.authority_lease_id
        && left.authority_lease_sequence == right.authority_lease_sequence
        && left.authority_lease_digest == right.authority_lease_digest
}

fn compute_entry_hash(entry: &LicenseMigrationJournalEntry) -> String {
    let mut unsigned = entry.clone();
    unsigned.entry_hash.clear();
    format!(
        "sha256:{:x}",
        Sha256::digest(serde_json::to_vec(&unsigned).expect("migration entry serializes"))
    )
}

/// Verified-mailbox promotion for legacy evaluators (Spec 152 §7.03).
/// Legacy local evaluation (Discovered) is archived as non-authoritative;
/// only a verified email + accepted terms + authority-issued lease may
/// promote to AuthorityIssued/Committed. No silent marketing consent,
/// no indefinite grace, and no project deletion.
pub fn verified_mailbox_email_is_valid(email: &str, verified: bool, terms_accepted: bool) -> bool {
    if !verified || !terms_accepted {
        return false;
    }
    let trimmed = email.trim();
    if trimmed.is_empty() || trimmed.len() > 254 {
        return false;
    }
    let parts: Vec<&str> = trimmed.split('@').collect();
    if parts.len() != 2 {
        return false;
    }
    let (local, domain) = (parts[0], parts[1]);
    if local.is_empty() || domain.is_empty() || !domain.contains('.') {
        return false;
    }
    // Masked local identity is allowed but raw unverified claim is not;
    // this validator never logs the raw email and only checks shape.
    !trimmed.contains(' ') && !trimmed.contains('\n')
}

pub fn build_verified_mailbox_awaiting_entry(
    source_digest: &str,
    source_class: LegacyLicenseSourceClass,
    email: &str,
    verified: bool,
    terms_accepted: bool,
    observed_at: DateTime<Utc>,
) -> Result<LicenseMigrationJournalEntry, LicenseMigrationError> {
    if !verified_mailbox_email_is_valid(email, verified, terms_accepted) {
        return Err(LicenseMigrationError::InvalidTransition);
    }
    Ok(LicenseMigrationJournalEntry {
        schema: LICENSE_MIGRATION_SCHEMA.into(),
        migration_id: migration_id_for_source_digest(source_digest),
        sequence: 0,
        source_class,
        source_digest: source_digest.into(),
        status: LicenseMigrationStatus::AwaitingAuthority,
        authority_lease_id: None,
        authority_lease_sequence: None,
        authority_lease_digest: None,
        preserved_data_refs: vec![
            "workpoints".into(),
            "evidence".into(),
            "migration_journal".into(),
        ],
        evidence_refs: vec!["verified_mailbox_device_code".into()],
        observed_at,
        previous_entry_hash: String::new(),
        entry_hash: String::new(),
    })
}

#[cfg(test)]
mod migration_eval_tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn migration_eval_verified_mailbox_promotion_succeeds() {
        assert!(verified_mailbox_email_is_valid(
            "truthful-evaluator@example.com",
            true,
            true
        ));
        let entry = build_verified_mailbox_awaiting_entry(
            "sha256:abc",
            LegacyLicenseSourceClass::EvaluationRecord,
            "truthful-evaluator@example.com",
            true,
            true,
            Utc::now(),
        )
        .expect("verified mailbox promotion should build awaiting entry");
        assert_eq!(entry.status, LicenseMigrationStatus::AwaitingAuthority);
        assert!(
            entry
                .preserved_data_refs
                .contains(&"workpoints".to_string())
        );
    }

    #[test]
    fn migration_eval_rejects_unverified_email() {
        assert!(!verified_mailbox_email_is_valid(
            "unverified@example.com",
            false,
            true
        ));
        assert!(
            build_verified_mailbox_awaiting_entry(
                "sha256:abc",
                LegacyLicenseSourceClass::EvaluationRecord,
                "unverified@example.com",
                false,
                true,
                Utc::now(),
            )
            .is_err()
        );
    }

    #[test]
    fn migration_eval_rejects_missing_terms() {
        assert!(!verified_mailbox_email_is_valid(
            "evaluator@example.com",
            true,
            false
        ));
        assert!(
            build_verified_mailbox_awaiting_entry(
                "sha256:abc",
                LegacyLicenseSourceClass::EvaluationRecord,
                "evaluator@example.com",
                true,
                false,
                Utc::now(),
            )
            .is_err()
        );
    }

    #[test]
    fn migration_eval_rejects_silent_marketing_consent_without_account() {
        // Marketing consent alone never promotes; only verified account + terms does.
        assert!(!verified_mailbox_email_is_valid("", true, true));
        assert!(!verified_mailbox_email_is_valid("not-an-email", true, true));
    }

    #[test]
    fn migration_eval_archive_local_eval_non_authoritative() {
        // Legacy eval source is EvaluationRecord, never PaidKeyRecord for promotion path;
        // archive keeps data but never grants entitlement without authority lease.
        let eval_id = migration_id_for_source_digest("sha256:legacy-eval");
        let paid_id = migration_id_for_source_digest("sha256:legacy-paid");
        assert_ne!(eval_id, paid_id);
        // AuthorityIssued without lease fields must fail
        let bad = LicenseMigrationJournalEntry {
            schema: LICENSE_MIGRATION_SCHEMA.into(),
            migration_id: eval_id,
            sequence: 3,
            source_class: LegacyLicenseSourceClass::EvaluationRecord,
            source_digest: "sha256:legacy-eval".into(),
            status: LicenseMigrationStatus::AuthorityIssued,
            authority_lease_id: None,
            authority_lease_sequence: None,
            authority_lease_digest: None,
            preserved_data_refs: vec![],
            evidence_refs: vec![],
            observed_at: Utc::now(),
            previous_entry_hash: String::new(),
            entry_hash: String::new(),
        };
        let prev = LicenseMigrationJournalEntry {
            schema: LICENSE_MIGRATION_SCHEMA.into(),
            migration_id: eval_id,
            sequence: 2,
            source_class: LegacyLicenseSourceClass::EvaluationRecord,
            source_digest: "sha256:legacy-eval".into(),
            status: LicenseMigrationStatus::AwaitingAuthority,
            authority_lease_id: None,
            authority_lease_sequence: None,
            authority_lease_digest: None,
            preserved_data_refs: vec![],
            evidence_refs: vec![],
            observed_at: Utc::now(),
            previous_entry_hash: GENESIS_HASH.into(),
            entry_hash: "sha256:prev".into(),
        };
        assert!(validate_transition(Some(&prev), &bad).is_err());
    }
}
