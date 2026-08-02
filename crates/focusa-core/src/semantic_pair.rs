//! Durable domain model for a build/verify semantic pair.
//!
//! The aggregate deliberately stores large values by handle.  Event payloads and
//! snapshots therefore remain small enough to replay and audit safely.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub const SEMANTIC_PAIR_SCHEMA_VERSION: u32 = 2;
pub const MAX_INLINE_TEXT_BYTES: usize = 16 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactHandleRef {
    pub handle: String,
    pub content_hash: String,
    pub byte_len: u64,
    pub media_type: String,
}

impl ArtifactHandleRef {
    pub fn validate(&self) -> Result<(), SemanticPairError> {
        if self.handle.trim().is_empty() || self.content_hash.trim().is_empty() {
            return Err(SemanticPairError::InvalidArtifactHandle);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuilderAttempt {
    pub attempt_id: String,
    pub builder: String,
    pub started_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct BuilderContext {
    pub project_root: String,
    pub continuity_id: String,
    #[serde(default)]
    pub attributes: BTreeMap<String, String>,
    #[serde(default)]
    pub artifact_refs: Vec<ArtifactHandleRef>,
}

/// A content-addressed snapshot.  Once installed by `PairCreated`, replay never
/// permits it to be replaced.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImmutableSnapshot {
    pub snapshot_id: String,
    pub captured_at: String,
    pub content_hash: String,
    #[serde(default)]
    pub artifact_refs: Vec<ArtifactHandleRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticItem {
    pub id: String,
    pub statement: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub artifact_refs: Vec<ArtifactHandleRef>,
    #[serde(default)]
    pub attributes: BTreeMap<String, String>,
}

impl SemanticItem {
    pub fn validate(&self) -> Result<(), SemanticPairError> {
        if self.id.trim().is_empty() {
            return Err(SemanticPairError::MissingId);
        }
        if self.statement.len() > MAX_INLINE_TEXT_BYTES {
            return Err(SemanticPairError::InlineArtifactTooLarge {
                bytes: self.statement.len(),
            });
        }
        for artifact in &self.artifact_refs {
            artifact.validate()?;
        }
        Ok(())
    }
}

pub type Obligation = SemanticItem;
pub type Plan = SemanticItem;
pub type Assignment = SemanticItem;
pub type Finding = SemanticItem;
pub type Response = SemanticItem;
pub type Disposition = SemanticItem;
pub type Validation = SemanticItem;
pub type Reroute = SemanticItem;
pub type Settlement = SemanticItem;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticReceipt {
    pub receipt_id: String,
    pub kind: String,
    pub issued_at: String,
    #[serde(default)]
    pub evidence_refs: Vec<ArtifactHandleRef>,
    #[serde(default)]
    pub attributes: BTreeMap<String, String>,
}

impl SemanticReceipt {
    pub fn validate(&self) -> Result<(), SemanticPairError> {
        if self.receipt_id.trim().is_empty() {
            return Err(SemanticPairError::MissingId);
        }
        for artifact in &self.evidence_refs {
            artifact.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SemanticPairLifecycleStatus {
    #[default]
    Active,
    Paused,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticControlRecord {
    pub authority_id: String,
    pub actor_id: String,
    pub reason: String,
    pub effective_at: String,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticContractRecord {
    pub contract_id: String,
    pub content_hash: String,
    pub committed_at: String,
    #[serde(default)]
    pub artifact_refs: Vec<ArtifactHandleRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticBuilderClaimRecord {
    pub attempt_id: String,
    pub claimant_id: String,
    pub claimed_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticRollbackRecord {
    pub target_sequence: u64,
    pub reason: String,
    pub committed_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticVerticalActivationRecord {
    pub bundle_id: String,
    pub bundle_hash: String,
    pub activated_at: String,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticPair {
    pub pair_id: String,
    pub schema_version: u32,
    pub builder_attempt: BuilderAttempt,
    pub builder_context: BuilderContext,
    pub snapshot: ImmutableSnapshot,
    #[serde(default)]
    pub obligations: Vec<Obligation>,
    #[serde(default)]
    pub plans: Vec<Plan>,
    #[serde(default)]
    pub assignments: Vec<Assignment>,
    #[serde(default)]
    pub findings: Vec<Finding>,
    #[serde(default)]
    pub responses: Vec<Response>,
    #[serde(default)]
    pub dispositions: Vec<Disposition>,
    #[serde(default)]
    pub validations: Vec<Validation>,
    #[serde(default)]
    pub reroutes: Vec<Reroute>,
    #[serde(default)]
    pub settlements: Vec<Settlement>,
    #[serde(default)]
    pub receipts: Vec<SemanticReceipt>,
    #[serde(default)]
    pub lifecycle_status: SemanticPairLifecycleStatus,
    #[serde(default)]
    pub lifecycle_history: Vec<SemanticControlRecord>,
    #[serde(default)]
    pub contract: Option<SemanticContractRecord>,
    #[serde(default)]
    pub snapshot_frozen: bool,
    #[serde(default)]
    pub builder_claim: Option<SemanticBuilderClaimRecord>,
    #[serde(default)]
    pub rollback: Option<SemanticRollbackRecord>,
    #[serde(default)]
    pub vertical_activation: Option<SemanticVerticalActivationRecord>,
}

impl SemanticPair {
    pub fn empty(
        pair_id: impl Into<String>,
        builder_attempt: BuilderAttempt,
        builder_context: BuilderContext,
        snapshot: ImmutableSnapshot,
    ) -> Self {
        Self {
            pair_id: pair_id.into(),
            schema_version: SEMANTIC_PAIR_SCHEMA_VERSION,
            builder_attempt,
            builder_context,
            snapshot,
            obligations: vec![],
            plans: vec![],
            assignments: vec![],
            findings: vec![],
            responses: vec![],
            dispositions: vec![],
            validations: vec![],
            reroutes: vec![],
            settlements: vec![],
            receipts: vec![],
            lifecycle_status: SemanticPairLifecycleStatus::Active,
            lifecycle_history: vec![],
            contract: None,
            snapshot_frozen: false,
            builder_claim: None,
            rollback: None,
            vertical_activation: None,
        }
    }

    pub fn canonical_hash(&self) -> Result<String, serde_json::Error> {
        let bytes = serde_json::to_vec(self)?;
        Ok(hex::encode(Sha256::digest(bytes)))
    }

    pub fn validate(&self) -> Result<(), SemanticPairError> {
        if self.pair_id.trim().is_empty() || self.builder_attempt.attempt_id.trim().is_empty() {
            return Err(SemanticPairError::MissingId);
        }
        if self.schema_version > SEMANTIC_PAIR_SCHEMA_VERSION {
            return Err(SemanticPairError::FutureVersion(self.schema_version));
        }
        for artifact in &self.builder_context.artifact_refs {
            artifact.validate()?;
        }
        for artifact in &self.snapshot.artifact_refs {
            artifact.validate()?;
        }
        for item in self
            .obligations
            .iter()
            .chain(&self.plans)
            .chain(&self.assignments)
            .chain(&self.findings)
            .chain(&self.responses)
            .chain(&self.dispositions)
            .chain(&self.validations)
            .chain(&self.reroutes)
            .chain(&self.settlements)
        {
            item.validate()?;
        }
        for receipt in &self.receipts {
            receipt.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum SemanticPairError {
    #[error("semantic pair id is required")]
    MissingId,
    #[error("artifact handle and content hash are required")]
    InvalidArtifactHandle,
    #[error("inline value is too large ({bytes} bytes); persist it and use an artifact handle")]
    InlineArtifactTooLarge { bytes: usize },
    #[error("semantic pair schema version {0} is newer than this runtime")]
    FutureVersion(u32),
}
