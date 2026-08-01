//! Version compatibility, migration planning, and rollback boundaries.

use crate::semantic_pair::{
    BuilderAttempt, BuilderContext, ImmutableSnapshot, SEMANTIC_PAIR_SCHEMA_VERSION, SemanticPair,
    SemanticReceipt,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticStoreState {
    Ready,
    Degraded { reason: String },
    MigrationRequired { found: u32, supported: u32 },
    QuarantinedFutureVersion { found: u32, supported: u32 },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationReceipt {
    pub migration_id: String,
    pub pair_id: String,
    pub from_version: u32,
    pub to_version: u32,
    pub source_hash: String,
    pub result_hash: String,
    pub dry_run: bool,
    pub applied: bool,
    pub rollback_boundary: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationPlan {
    pub aggregate: SemanticPair,
    pub receipt: MigrationReceipt,
    source_bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct VersionProbe {
    schema_version: u32,
}

/// Compatibility shape used by version 1.  Missing collections remain empty;
/// no inferred semantic facts are invented during migration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LegacySemanticPairV1 {
    pub schema_version: u32,
    pub pair_id: String,
    pub attempt_id: String,
    pub builder: String,
    #[serde(default)]
    pub started_at: String,
    #[serde(default)]
    pub project_root: String,
    #[serde(default)]
    pub continuity_id: String,
    pub snapshot_id: String,
    pub snapshot_hash: String,
}

pub fn inspect_version(bytes: &[u8]) -> Result<SemanticStoreState, MigrationError> {
    let probe: VersionProbe = serde_json::from_slice(bytes).map_err(MigrationError::Decode)?;
    Ok(match probe.schema_version {
        SEMANTIC_PAIR_SCHEMA_VERSION => SemanticStoreState::Ready,
        1..SEMANTIC_PAIR_SCHEMA_VERSION => SemanticStoreState::MigrationRequired {
            found: probe.schema_version,
            supported: SEMANTIC_PAIR_SCHEMA_VERSION,
        },
        version if version > SEMANTIC_PAIR_SCHEMA_VERSION => {
            SemanticStoreState::QuarantinedFutureVersion {
                found: version,
                supported: SEMANTIC_PAIR_SCHEMA_VERSION,
            }
        }
        version => SemanticStoreState::Degraded {
            reason: format!("invalid semantic schema version {version}"),
        },
    })
}

/// Read current data directly, identify old data without silently mutating it,
/// and quarantine future data before attempting to deserialize its shape.
pub fn compatibility_read(bytes: &[u8]) -> Result<CompatibilityRead, MigrationError> {
    match inspect_version(bytes)? {
        SemanticStoreState::Ready => {
            let pair = serde_json::from_slice(bytes).map_err(MigrationError::Decode)?;
            Ok(CompatibilityRead::Current(pair))
        }
        state @ SemanticStoreState::MigrationRequired { .. } => {
            Ok(CompatibilityRead::MigrationRequired(state))
        }
        state @ SemanticStoreState::QuarantinedFutureVersion { .. } => {
            Ok(CompatibilityRead::Quarantined(state))
        }
        state => Ok(CompatibilityRead::Degraded(state)),
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompatibilityRead {
    Current(SemanticPair),
    MigrationRequired(SemanticStoreState),
    Quarantined(SemanticStoreState),
    Degraded(SemanticStoreState),
}

pub fn plan_v1_migration(
    bytes: &[u8],
    migration_id: impl Into<String>,
    dry_run: bool,
) -> Result<MigrationPlan, MigrationError> {
    let legacy: LegacySemanticPairV1 =
        serde_json::from_slice(bytes).map_err(MigrationError::Decode)?;
    if legacy.schema_version != 1 {
        return Err(if legacy.schema_version > SEMANTIC_PAIR_SCHEMA_VERSION {
            MigrationError::FutureVersion(legacy.schema_version)
        } else {
            MigrationError::UnsupportedVersion(legacy.schema_version)
        });
    }
    let pair = SemanticPair::empty(
        legacy.pair_id.clone(),
        BuilderAttempt {
            attempt_id: legacy.attempt_id,
            builder: legacy.builder,
            started_at: legacy.started_at,
        },
        BuilderContext {
            project_root: legacy.project_root,
            continuity_id: legacy.continuity_id,
            ..BuilderContext::default()
        },
        ImmutableSnapshot {
            snapshot_id: legacy.snapshot_id,
            captured_at: String::new(),
            content_hash: legacy.snapshot_hash,
            artifact_refs: vec![],
        },
    );
    let source_hash = digest(bytes);
    let result_hash = pair.canonical_hash().map_err(MigrationError::Encode)?;
    let migration_id = migration_id.into();
    let rollback_boundary =
        digest(format!("{}:{}:{}", migration_id, source_hash, result_hash).as_bytes());
    let receipt = MigrationReceipt {
        migration_id,
        pair_id: pair.pair_id.clone(),
        from_version: 1,
        to_version: SEMANTIC_PAIR_SCHEMA_VERSION,
        source_hash,
        result_hash,
        dry_run,
        applied: false,
        rollback_boundary,
    };
    Ok(MigrationPlan {
        aggregate: pair,
        receipt,
        source_bytes: bytes.to_vec(),
    })
}

impl MigrationPlan {
    /// Marks the plan applied. Persistence adapters must do this only in the
    /// same transaction that stores the migrated aggregate.
    pub fn applied_receipt(&self) -> Result<MigrationReceipt, MigrationError> {
        if self.receipt.dry_run {
            return Err(MigrationError::DryRunCannotApply);
        }
        let mut receipt = self.receipt.clone();
        receipt.applied = true;
        Ok(receipt)
    }

    /// Rollback is bounded to the exact migration receipt and pre-migration
    /// source hash. Later writes must reject this boundary in persistence.
    pub fn rollback(&self, receipt: &MigrationReceipt) -> Result<Vec<u8>, MigrationError> {
        if !receipt.applied
            || receipt.rollback_boundary != self.receipt.rollback_boundary
            || digest(&self.source_bytes) != receipt.source_hash
        {
            return Err(MigrationError::RollbackBoundaryMismatch);
        }
        Ok(self.source_bytes.clone())
    }
}

pub fn migration_receipt_as_semantic_receipt(receipt: &MigrationReceipt) -> SemanticReceipt {
    SemanticReceipt {
        receipt_id: receipt.migration_id.clone(),
        kind: "semantic_pair_migration".to_string(),
        issued_at: String::new(),
        evidence_refs: vec![],
        attributes: [
            ("source_hash".to_string(), receipt.source_hash.clone()),
            ("result_hash".to_string(), receipt.result_hash.clone()),
            (
                "rollback_boundary".to_string(),
                receipt.rollback_boundary.clone(),
            ),
        ]
        .into_iter()
        .collect(),
    }
}

fn digest(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

#[derive(Debug, thiserror::Error)]
pub enum MigrationError {
    #[error("semantic pair JSON could not be decoded: {0}")]
    Decode(serde_json::Error),
    #[error("semantic pair JSON could not be encoded: {0}")]
    Encode(serde_json::Error),
    #[error("semantic schema version {0} is unsupported")]
    UnsupportedVersion(u32),
    #[error("semantic schema version {0} is newer than this runtime and must be quarantined")]
    FutureVersion(u32),
    #[error("a dry-run migration cannot be applied")]
    DryRunCannotApply,
    #[error("migration rollback boundary does not match")]
    RollbackBoundaryMismatch,
}
