//! Executable, provider-neutral adapter manifests for the Master Release Cycle.
//!
//! Manifests declare bounded operations. Provider plugins implement
//! `ReleaseOperationExecutor`; only the orchestrator owns release state.

use std::{
    collections::{BTreeMap, BTreeSet},
    process::Stdio,
};

use anyhow::{Context, ensure};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;

use crate::release_cycle::{ReleaseEvidence, ReleaseStage, ReleaseTopology};
use crate::release_orchestrator::{
    AdapterOutcome, RELEASE_ADAPTER_SCHEMA, ReleaseAdapter, ReleaseAdapterDescriptor,
    ReleaseStageReceipt, ReleaseStageRequest,
};

pub const RELEASE_ADAPTER_MANIFEST_SCHEMA: &str = "focusa.release_adapter_manifest.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseOperationKind {
    LocalCommand,
    GithubWorkflow,
    Container,
    HttpHealth,
    ToolCall,
    Evidence,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReleaseOperation {
    pub operation_id: String,
    pub stage: ReleaseStage,
    pub executor_id: String,
    pub kind: ReleaseOperationKind,
    pub action: String,
    #[serde(default)]
    pub surface_ids: Vec<String>,
    pub mutates: bool,
    pub timeout_seconds: u64,
    #[serde(default)]
    pub parallel_group: Option<String>,
    #[serde(default)]
    pub inputs: BTreeMap<String, Value>,
}

impl ReleaseOperation {
    /// Returns the canonical capability family for this release operation.
    ///
    /// All release orchestration operations (mutating stages) require the
    /// `release_proof` premium family. Read-only operations like status
    /// checks are classified as `ReadProjection` and do not require premium.
    pub fn capability_family(&self) -> focusa_license::CapabilityFamily {
        if self.mutates {
            focusa_license::CapabilityFamily::ReleaseProof
        } else {
            focusa_license::CapabilityFamily::ReadProjection
        }
    }

    /// Returns the required premium feature identifier for this operation,
    /// if it is a premium operation.
    pub fn required_feature(&self) -> Option<&'static str> {
        if self.mutates {
            Some("focusa.release.proof")
        } else {
            None
        }
    }

    fn validate(&self, topology: &ReleaseTopology) -> anyhow::Result<()> {
        ensure!(
            !self.operation_id.trim().is_empty(),
            "operation_id is required"
        );
        ensure!(
            !self.executor_id.trim().is_empty(),
            "executor_id is required"
        );
        ensure!(
            !self.action.trim().is_empty(),
            "operation action is required"
        );
        ensure!(
            self.timeout_seconds > 0 && self.timeout_seconds <= 21_600,
            "operation timeout must be within 1..=21600 seconds"
        );
        let known: BTreeSet<_> = topology
            .surfaces
            .iter()
            .map(|surface| surface.surface_id.as_str())
            .collect();
        ensure!(
            self.surface_ids
                .iter()
                .all(|surface| known.contains(surface.as_str())),
            "operation references unknown surface"
        );
        ensure!(
            self.stage != ReleaseStage::Plan,
            "adapter cannot execute plan authority"
        );
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReleaseAdapterManifest {
    pub schema: String,
    pub manifest_id: String,
    pub topology_ref: String,
    pub descriptor: ReleaseAdapterDescriptor,
    pub operations: Vec<ReleaseOperation>,
}

impl ReleaseAdapterManifest {
    pub fn validate(&self, topology: &ReleaseTopology) -> anyhow::Result<()> {
        ensure!(
            self.schema == RELEASE_ADAPTER_MANIFEST_SCHEMA,
            "unsupported release adapter manifest schema"
        );
        ensure!(
            !self.manifest_id.trim().is_empty(),
            "manifest_id is required"
        );
        ensure!(
            !self.topology_ref.trim().is_empty(),
            "topology_ref is required"
        );
        topology.validate()?;
        self.descriptor.validate_for(topology)?;
        ensure!(
            !self.operations.is_empty(),
            "adapter manifest requires operations"
        );
        let mut ids = BTreeSet::new();
        for operation in &self.operations {
            ensure!(
                ids.insert(operation.operation_id.as_str()),
                "duplicate adapter operation_id"
            );
            operation.validate(topology)?;
            ensure!(
                self.descriptor.supported_stages.contains(&operation.stage),
                "operation stage is not declared by adapter"
            );
        }
        for stage in self.descriptor.supported_stages.iter().filter(|stage| {
            **stage != ReleaseStage::CanaryDeployed
                || topology
                    .surfaces
                    .iter()
                    .any(|surface| surface.canary_required)
        }) {
            ensure!(
                self.operations
                    .iter()
                    .any(|operation| operation.stage == *stage),
                "adapter manifest lacks operation for required stage {stage:?}"
            );
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseOperationReceipt {
    pub operation_id: String,
    pub executor_id: String,
    pub exact_sha: String,
    pub outcome: AdapterOutcome,
    pub observed_at: String,
    pub evidence_refs: Vec<String>,
    pub artifact_set_id: Option<String>,
    pub rollback_ref: Option<String>,
    pub elapsed_ms: u64,
    pub queue_ms: u64,
    pub retry_ms: u64,
    #[serde(default)]
    pub reason_codes: Vec<String>,
}

impl ReleaseOperationReceipt {
    fn validate(
        &self,
        operation: &ReleaseOperation,
        request: &ReleaseStageRequest,
    ) -> anyhow::Result<()> {
        ensure!(
            self.operation_id == operation.operation_id,
            "operation receipt id mismatch"
        );
        ensure!(
            self.executor_id == operation.executor_id,
            "operation executor mismatch"
        );
        ensure!(
            self.exact_sha == request.exact_sha,
            "operation receipt SHA mismatch"
        );
        ensure!(
            !self.observed_at.trim().is_empty(),
            "operation receipt timestamp is required"
        );
        ensure!(
            !self.evidence_refs.is_empty(),
            "operation receipt requires evidence"
        );
        Ok(())
    }
}

#[async_trait]
pub trait ReleaseOperationExecutor: Send + Sync {
    fn executor_id(&self) -> &str;
    async fn execute(
        &self,
        operation: &ReleaseOperation,
        request: &ReleaseStageRequest,
    ) -> anyhow::Result<ReleaseOperationReceipt>;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReleasePluginEnvelope {
    pub schema: String,
    pub operation: ReleaseOperation,
    pub request: ReleaseStageRequest,
}

pub struct JsonProcessReleaseExecutor {
    executor_id: String,
    program: std::path::PathBuf,
    project_root: std::path::PathBuf,
}

impl JsonProcessReleaseExecutor {
    pub fn new(
        executor_id: impl Into<String>,
        program: impl Into<std::path::PathBuf>,
        project_root: impl Into<std::path::PathBuf>,
    ) -> anyhow::Result<Self> {
        let executor_id = executor_id.into();
        let program = program.into();
        let project_root = project_root.into();
        ensure!(!executor_id.trim().is_empty(), "executor id is required");
        ensure!(
            program.is_absolute() && program.is_file(),
            "release plugin must be an existing absolute executable path"
        );
        ensure!(
            project_root.is_absolute() && project_root.is_dir(),
            "release plugin project root must be an existing absolute directory"
        );
        Ok(Self {
            executor_id,
            program,
            project_root,
        })
    }
}

#[async_trait]
impl ReleaseOperationExecutor for JsonProcessReleaseExecutor {
    fn executor_id(&self) -> &str {
        &self.executor_id
    }

    async fn execute(
        &self,
        operation: &ReleaseOperation,
        request: &ReleaseStageRequest,
    ) -> anyhow::Result<ReleaseOperationReceipt> {
        ensure!(
            operation.executor_id == self.executor_id,
            "plugin executor authority mismatch"
        );
        ensure!(
            request.project_root == self.project_root.to_string_lossy(),
            "plugin project root differs from candidate authority"
        );
        let envelope = ReleasePluginEnvelope {
            schema: "focusa.release_plugin_envelope.v1".into(),
            operation: operation.clone(),
            request: request.clone(),
        };
        let input = serde_json::to_vec(&envelope)?;
        let mut child = tokio::process::Command::new(&self.program)
            .current_dir(&self.project_root)
            .env_clear()
            .env("FOCUSA_RELEASE_PLUGIN", "1")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .with_context(|| format!("spawn release plugin {}", self.program.display()))?;
        child
            .stdin
            .take()
            .context("release plugin stdin unavailable")?
            .write_all(&input)
            .await?;
        let output = tokio::time::timeout(
            std::time::Duration::from_secs(operation.timeout_seconds),
            child.wait_with_output(),
        )
        .await
        .context("release plugin timed out")??;
        ensure!(
            output.status.success(),
            "release plugin failed: {}",
            bounded_text(&output.stderr)
        );
        ensure!(
            output.stdout.len() <= 1_048_576,
            "release plugin receipt exceeds 1 MiB"
        );
        let receipt: ReleaseOperationReceipt = serde_json::from_slice(&output.stdout)
            .context("release plugin returned invalid receipt JSON")?;
        receipt.validate(operation, request)?;
        Ok(receipt)
    }
}

fn bounded_text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(&bytes[..bytes.len().min(4096)]).into_owned()
}

pub struct ManifestReleaseAdapter<E> {
    manifest: ReleaseAdapterManifest,
    topology: ReleaseTopology,
    executors: BTreeMap<String, E>,
}

impl<E: ReleaseOperationExecutor> ManifestReleaseAdapter<E> {
    pub fn new(
        manifest: ReleaseAdapterManifest,
        topology: ReleaseTopology,
        executors: Vec<E>,
    ) -> anyhow::Result<Self> {
        manifest.validate(&topology)?;
        let mut by_id = BTreeMap::new();
        for executor in executors {
            let id = executor.executor_id().to_string();
            ensure!(!id.trim().is_empty(), "executor id is required");
            ensure!(
                by_id.insert(id.clone(), executor).is_none(),
                "duplicate executor id {id}"
            );
        }
        for operation in &manifest.operations {
            ensure!(
                by_id.contains_key(&operation.executor_id),
                "missing executor {}",
                operation.executor_id
            );
        }
        Ok(Self {
            manifest,
            topology,
            executors: by_id,
        })
    }

    pub fn manifest(&self) -> &ReleaseAdapterManifest {
        &self.manifest
    }
}

#[async_trait]
impl<E: ReleaseOperationExecutor> ReleaseAdapter for ManifestReleaseAdapter<E> {
    fn descriptor(&self) -> ReleaseAdapterDescriptor {
        self.manifest.descriptor.clone()
    }

    async fn execute(&self, request: ReleaseStageRequest) -> anyhow::Result<ReleaseStageReceipt> {
        ensure!(
            request.topology == self.topology,
            "adapter request topology differs from manifest"
        );
        let operations: Vec<_> = self
            .manifest
            .operations
            .iter()
            .filter(|operation| operation.stage == request.stage)
            .collect();
        ensure!(
            !operations.is_empty(),
            "no adapter operations for stage {:?}",
            request.stage
        );
        let mut receipts = Vec::with_capacity(operations.len());
        for operation in operations {
            let executor = self
                .executors
                .get(&operation.executor_id)
                .context("operation executor disappeared")?;
            let receipt = executor
                .execute(operation, &request)
                .await
                .with_context(|| format!("operation {} failed", operation.operation_id))?;
            receipt.validate(operation, &request)?;
            let blocked = receipt.outcome == AdapterOutcome::Blocked;
            receipts.push(receipt);
            if blocked {
                break;
            }
        }
        aggregate_receipts(
            &self.manifest.descriptor.adapter_id,
            request.stage,
            &request.exact_sha,
            receipts,
        )
    }
}

fn aggregate_receipts(
    adapter_id: &str,
    stage: ReleaseStage,
    exact_sha: &str,
    receipts: Vec<ReleaseOperationReceipt>,
) -> anyhow::Result<ReleaseStageReceipt> {
    ensure!(
        !receipts.is_empty(),
        "stage receipt aggregation requires operations"
    );
    let outcome = if receipts
        .iter()
        .any(|receipt| receipt.outcome == AdapterOutcome::Blocked)
    {
        AdapterOutcome::Blocked
    } else if receipts
        .iter()
        .all(|receipt| receipt.outcome == AdapterOutcome::Skipped)
    {
        AdapterOutcome::Skipped
    } else {
        AdapterOutcome::Passed
    };
    let observed_at = receipts
        .iter()
        .map(|receipt| receipt.observed_at.as_str())
        .max()
        .unwrap_or_default()
        .to_string();
    let evidence_refs = receipts
        .iter()
        .flat_map(|receipt| receipt.evidence_refs.clone())
        .collect();
    let artifact_ids: Vec<_> = receipts
        .iter()
        .filter_map(|receipt| receipt.artifact_set_id.clone())
        .collect();
    let artifact_set_id =
        (!artifact_ids.is_empty()).then(|| aggregate_identity("artifact", &artifact_ids));
    let rollback_refs: Vec<_> = receipts
        .iter()
        .filter_map(|receipt| receipt.rollback_ref.clone())
        .collect();
    let rollback_ref =
        (!rollback_refs.is_empty()).then(|| aggregate_identity("rollback", &rollback_refs));
    let reason_codes = receipts
        .iter()
        .flat_map(|receipt| receipt.reason_codes.clone())
        .collect();
    Ok(ReleaseStageReceipt {
        stage,
        outcome,
        evidence: ReleaseEvidence {
            stage,
            exact_sha: exact_sha.into(),
            observed_at,
            evidence_refs,
            invalidates: Vec::new(),
        },
        adapter_id: adapter_id.into(),
        artifact_set_id,
        rollback_ref,
        elapsed_ms: receipts
            .iter()
            .map(|receipt| receipt.elapsed_ms)
            .max()
            .unwrap_or(0),
        queue_ms: receipts.iter().map(|receipt| receipt.queue_ms).sum(),
        retry_ms: receipts.iter().map(|receipt| receipt.retry_ms).sum(),
        reason_codes,
    })
}

fn aggregate_identity(kind: &str, ids: &[String]) -> String {
    let mut stable = ids.to_vec();
    stable.sort();
    stable.dedup();
    let digest = Sha256::digest(stable.join("\n").as_bytes());
    format!("{kind}:sha256:{digest:x}")
}

pub fn load_release_adapter_manifest(
    path: impl AsRef<std::path::Path>,
    topology: &ReleaseTopology,
) -> anyhow::Result<ReleaseAdapterManifest> {
    let path = path.as_ref();
    let body = std::fs::read_to_string(path)
        .with_context(|| format!("read adapter manifest {}", path.display()))?;
    let manifest: ReleaseAdapterManifest = serde_json::from_str(&body)
        .with_context(|| format!("parse adapter manifest {}", path.display()))?;
    manifest.validate(topology)?;
    Ok(manifest)
}

#[cfg(test)]
#[path = "release_adapters_test.rs"]
mod tests;
