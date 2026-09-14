//! Generated Spec144 semantic-pair client DTO parity.
use serde::{Deserialize, Serialize};
pub mod temporal;
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SemanticPairTruthState {
    Supported, SchemaOnly, PackMissing, MigrationRequired, VerificationRequired,
    VerificationBlocked, OperatorRequired, UnsupportedFutureDefinition,
    WriterBlocked, Degraded, Stale, Conflicted, Quarantined,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SemanticPairScope { pub project_root: String, pub continuity_id: String }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SemanticPairObligation {
    pub obligation_id: String, pub statement: String, pub source_refs: Vec<String>, pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SemanticPairFinding {
    pub finding_id: String, pub obligation_id: Option<String>, pub severity: String,
    pub verdict: String, pub summary: String, pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SemanticPairSettlement {
    pub status: String, pub verdict: Option<String>, pub settled_at: Option<String>, pub receipt_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SemanticPairReplay {
    pub status: String, pub generation: u64, pub last_event_sequence: Option<u64>, pub receipt_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SemanticPairRecovery {
    pub required: bool, pub state: Option<SemanticPairTruthState>,
    pub next_operation: Option<SemanticPairOperationId>, pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SemanticPairPortfolioItem {
    pub pair_id: String, pub title: String, pub state: SemanticPairTruthState,
    pub obligations: Vec<SemanticPairObligation>, pub findings: Vec<SemanticPairFinding>,
    pub settlement: SemanticPairSettlement, pub replay: SemanticPairReplay,
    pub recovery: SemanticPairRecovery, pub evidence_refs: Vec<String>,
    pub receipt_refs: Vec<String>, pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SemanticPairPortfolio {
    pub schema: String, pub scope: SemanticPairScope, pub items: Vec<SemanticPairPortfolioItem>,
    pub state: SemanticPairTruthState, pub stale: bool, pub conflicted: bool, pub quarantined: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SemanticPairCapability { Read, Mutate }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SemanticPairAction {
    pub operation_id: SemanticPairOperationId, pub capability: SemanticPairCapability,
    pub available: bool, pub disabled_reason: Option<String>,
    pub idempotency_key: Option<String>, pub confirmation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SemanticPairActionRequest {
    pub operation_id: SemanticPairOperationId, pub scope: SemanticPairScope,
    pub pair_id: Option<String>, pub idempotency_key: Option<String>,
    pub confirmation: Option<String>, pub payload: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SemanticPairOperationId {
    #[serde(rename = "semantic.integrity.status")] SemanticIntegrityStatus,
    #[serde(rename = "semantic.integrity.registry")] SemanticIntegrityRegistry,
    #[serde(rename = "semantic.integrity.artifact.list")] SemanticIntegrityArtifactList,
    #[serde(rename = "semantic.integrity.artifact.get")] SemanticIntegrityArtifactGet,
    #[serde(rename = "semantic.integrity.validate")] SemanticIntegrityValidate,
    #[serde(rename = "semantic.integrity.reason.preview")] SemanticIntegrityReasonPreview,
    #[serde(rename = "semantic.integrity.reason.explain")] SemanticIntegrityReasonExplain,
    #[serde(rename = "semantic.integrity.receipt.get")] SemanticIntegrityReceiptGet,
    #[serde(rename = "semantic_pair.create")] Create,
    #[serde(rename = "semantic_pair.get")] Get,
    #[serde(rename = "semantic_pair.pause")] Pause,
    #[serde(rename = "semantic_pair.resume")] Resume,
    #[serde(rename = "semantic_pair.cancel")] Cancel,
    #[serde(rename = "semantic_pair.contract.preview")] ContractPreview,
    #[serde(rename = "semantic_pair.contract.commit")] ContractCommit,
    #[serde(rename = "semantic_pair.builder.start")] BuilderStart,
    #[serde(rename = "semantic_pair.builder.claim")] BuilderClaim,
    #[serde(rename = "semantic_pair.builder.respond")] BuilderRespond,
    #[serde(rename = "semantic_pair.builder.repair")] BuilderRepair,
    #[serde(rename = "semantic_pair.snapshot.freeze")] SnapshotFreeze,
    #[serde(rename = "semantic_pair.snapshot.get")] SnapshotGet,
    #[serde(rename = "semantic_pair.obligations.compile")] ObligationsCompile,
    #[serde(rename = "semantic_pair.verification.plan.preview")] VerificationPlanPreview,
    #[serde(rename = "semantic_pair.verification.plan.commit")] VerificationPlanCommit,
    #[serde(rename = "semantic_pair.verify.start")] VerifyStart,
    #[serde(rename = "semantic_pair.verify.findings")] VerifyFindings,
    #[serde(rename = "semantic_pair.verify.verdict")] VerifyVerdict,
    #[serde(rename = "semantic_pair.finding.respond")] FindingRespond,
    #[serde(rename = "semantic_pair.finding.resolve")] FindingResolve,
    #[serde(rename = "semantic_pair.settlement.preview")] SettlementPreview,
    #[serde(rename = "semantic_pair.settlement.commit")] SettlementCommit,
    #[serde(rename = "semantic_pair.receipt.get")] ReceiptGet,
    #[serde(rename = "semantic_pair.replay")] Replay,
    #[serde(rename = "semantic_pair.eval")] Eval,
    #[serde(rename = "semantic_pair.migration.status")] MigrationStatus,
    #[serde(rename = "semantic_pair.migration.run")] MigrationRun,
    #[serde(rename = "semantic_pair.rollback.preview")] RollbackPreview,
    #[serde(rename = "semantic_pair.rollback.commit")] RollbackCommit,
    #[serde(rename = "vertical.bundle.validate")] VerticalBundleValidate,
    #[serde(rename = "vertical.bundle.preview")] VerticalBundlePreview,
    #[serde(rename = "vertical.bundle.activate")] VerticalBundleActivate,
    #[serde(rename = "vertical.bundle.conformance")] VerticalBundleConformance,
    #[serde(rename = "semantic.reflex.visibility")] SemanticReflexVisibility,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RequestSpec {
    pub method: &'static str,
    pub url: String,
    pub body: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticPairClient { base_url: String }

impl SemanticPairClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self { base_url: base_url.into().trim_end_matches('/').to_owned() }
    }
    fn query(scope: &SemanticPairScope) -> String {
        format!("project_root={}&continuity_id={}", encode(&scope.project_root), encode(&scope.continuity_id))
    }
    pub fn status(&self, scope: &SemanticPairScope) -> RequestSpec {
        RequestSpec { method: "GET", url: format!("{}/v1/semantic-integrity/status?{}", self.base_url, Self::query(scope)), body: None }
    }
    pub fn operations(&self, scope: &SemanticPairScope) -> RequestSpec {
        RequestSpec { method: "GET", url: format!("{}/v1/semantic-integrity/operations?{}&limit=100", self.base_url, Self::query(scope)), body: None }
    }
    pub fn invoke(&self, request: &SemanticPairActionRequest) -> Result<RequestSpec, serde_json::Error> {
        let id = serde_json::to_value(&request.operation_id)?.as_str().unwrap_or_default().to_owned();
        Ok(RequestSpec { method: "POST", url: format!("{}/v1/semantic-integrity/operations/{}", self.base_url, id), body: Some(serde_json::to_value(request)?) })
    }
}

fn encode(value: &str) -> String {
    value.bytes().flat_map(|byte| match byte {
        b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => vec![byte as char],
        _ => format!("%{byte:02X}").chars().collect(),
    }).collect()
}
