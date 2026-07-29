//! Spec 140 canonical Project Agent Runtime Constitution contracts.
//!
//! This module contains facts and validation only. Discovery, compilation,
//! activation, delivery, and enforcement remain explicit service operations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

macro_rules! reference_type {
    ($($name:ident),+ $(,)?) => {$ (
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub String);
    )+ };
}

reference_type!(
    AgentIdentityReference,
    CapabilityProfileReference,
    ConstitutionalKernelReference,
    PermissionProfileReference,
    RoleProfileReference,
    RuntimeAwarenessContractReference,
);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeConstitutionLifecycleState {
    Draft,
    Reconciled,
    PendingOperator,
    Approved,
    Active,
    Superseded,
    Revoked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstructionSourceAuthority {
    HarnessSystem,
    FocusaConstitution,
    ProjectRoot,
    PathLocal,
    UserManaged,
    Imported,
    Untrusted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstructionTrustClass {
    TrustedManaged,
    TrustedProject,
    UserConfirmed,
    Advisory,
    Untrusted,
    Quarantined,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstructionApplicability {
    Applicable,
    Conditional,
    NotApplicable,
    Disputed,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstructionFreshness {
    Current,
    Stale,
    Superseded,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromptLayer {
    HarnessSystem,
    FocusaKernel,
    ProjectConstitution,
    Role,
    PathOverlay,
    SessionAwareness,
    OperatorSteering,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionSource {
    pub source_id: String,
    pub source_ref: String,
    pub content_sha256: String,
    pub authority: InstructionSourceAuthority,
    pub trust: InstructionTrustClass,
    pub freshness: InstructionFreshness,
    pub scope_ref: String,
    pub discovered_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionClaim {
    pub claim_id: String,
    pub source_id: String,
    pub claim_class: String,
    pub normalized_text: String,
    pub source_text_sha256: String,
    pub applicability: InstructionApplicability,
    pub scope_ref: String,
    pub condition: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionConflict {
    pub conflict_id: String,
    pub claim_refs: Vec<String>,
    pub conflict_class: String,
    pub authority_graph_ref: String,
    pub requires_operator: bool,
    pub detected_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionResolution {
    pub resolution_id: String,
    pub conflict_id: String,
    pub disposition: String,
    pub winning_claim_refs: Vec<String>,
    pub suppressed_claim_refs: Vec<String>,
    pub rationale: String,
    pub operator_confirmed: bool,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionAuthorityGraph {
    pub graph_id: String,
    pub ordered_authorities: Vec<InstructionSourceAuthority>,
    pub conditional_edges: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionAnalysisFinding {
    pub finding_id: String,
    pub source_ref: String,
    pub finding_class: String,
    pub severity: String,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionSmell {
    pub smell_id: String,
    pub source_ref: String,
    pub smell_class: String,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionInjectionRecord {
    pub record_id: String,
    pub source_ref: String,
    pub trust: InstructionTrustClass,
    pub blocked: bool,
    pub reason_code: String,
    pub content_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionOverlay {
    pub overlay_id: String,
    pub path_scope: String,
    pub claim_refs: Vec<String>,
    pub precedence: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathInstructionPolicy {
    pub project_root: String,
    pub allow_parent_walk: bool,
    pub stop_markers: Vec<String>,
    pub deny_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentOperatingContract {
    pub purpose: String,
    pub responsibilities: Vec<String>,
    pub non_responsibilities: Vec<String>,
    pub authority_order: Vec<String>,
    pub execution_boundaries: Vec<String>,
    pub output_contracts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetCapabilityProfile {
    pub profile_version: String,
    pub target: String,
    pub supported_layers: BTreeSet<PromptLayer>,
    pub supported_features: BTreeSet<String>,
    pub unsupported_features: BTreeMap<String, String>,
    pub max_prompt_bytes: usize,
    pub supports_session_pinning: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PiPromptVariant {
    pub variant_id: String,
    pub target: String,
    pub prompt_sha256: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemPromptAssemblyPlan {
    pub plan_id: String,
    pub ordered_layers: Vec<PromptLayer>,
    pub source_refs: Vec<String>,
    pub excluded_claims: BTreeMap<String, String>,
    pub target_profile: TargetCapabilityProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptGroundingManifest {
    pub prompt_sha256: String,
    pub source_hashes: BTreeMap<String, String>,
    pub resolution_refs: Vec<String>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptIntegrityManifest {
    pub assembly_plan_sha256: String,
    pub prompt_sha256: String,
    pub signature_ref: Option<String>,
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptRevocation {
    pub revocation_id: String,
    pub version: String,
    pub reason_code: String,
    pub effective_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillBinding {
    pub skill_id: String,
    pub source_ref: String,
    pub activation_condition: String,
    pub authority_scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillActivationPlan {
    pub plan_id: String,
    pub bindings: Vec<SkillBinding>,
    pub excluded: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRoutingPlan {
    pub plan_id: String,
    pub allowed_tools: BTreeSet<String>,
    pub denied_tools: BTreeMap<String, String>,
    pub confirmation_required: BTreeSet<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementControl {
    pub control_id: String,
    pub boundary: String,
    pub enforcement_point: String,
    pub failure_posture: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentEnforcementPlan {
    pub plan_id: String,
    pub controls: Vec<EnforcementControl>,
    pub permission_profile_refs: Vec<PermissionProfileReference>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub rule_id: String,
    pub requirement_ref: String,
    pub check_kind: String,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationMatrix {
    pub matrix_id: String,
    pub rules: Vec<ValidationRule>,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptEvaluation {
    pub evaluation_id: String,
    pub variant_id: String,
    pub score: f64,
    pub dimensions: BTreeMap<String, f64>,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptEpistemicOutcomeRecord {
    pub record_id: String,
    pub evaluation_id: String,
    pub prompt_identity_sha256: String,
    pub constitution_version: String,
    pub source_manifest_sha256: String,
    pub environment_refs: Vec<String>,
    pub topology_refs: Vec<String>,
    pub prediction_refs: Vec<String>,
    pub outcome_refs: Vec<String>,
    pub calibration_refs: Vec<String>,
    pub transfer_refs: Vec<String>,
    pub drift_refs: Vec<String>,
    pub negative_transfer_refs: Vec<String>,
    pub proposal_ref: Option<String>,
    pub activation_authorized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentContractImpactAssessment {
    pub assessment_id: String,
    pub changed_source_refs: Vec<String>,
    pub affected_artifacts: Vec<String>,
    pub risk: String,
    pub required_checks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeArtifactProjection {
    pub target: String,
    pub artifact_ref: String,
    pub content_sha256: String,
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRuntimeDeliveryManifest {
    pub manifest_id: String,
    pub constitution_version: String,
    pub artifacts: Vec<RuntimeArtifactProjection>,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEnvironmentBinding {
    pub binding_id: String,
    pub project_root: String,
    pub continuity_id: String,
    pub target: String,
    pub environment_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionPromptPin {
    pub session_id: String,
    pub constitution_version: String,
    pub prompt_sha256: String,
    pub pinned_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConstitutionVersion {
    pub version: String,
    pub parent_version: Option<String>,
    pub content_sha256: String,
    pub lifecycle: RuntimeConstitutionLifecycleState,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectAgentRuntimeConstitution {
    pub schema: String,
    pub constitution_id: String,
    pub project_ref: String,
    pub genesis_ref: String,
    pub approved_spec_ref: String,
    pub agent_identity_ref: AgentIdentityReference,
    pub base_agent_constitution_ref: ConstitutionalKernelReference,
    pub role_profile_ref: RoleProfileReference,
    pub revision: u64,
    pub status: RuntimeConstitutionLifecycleState,
    pub operating_contract: AgentOperatingContract,
    pub instruction_sources: Vec<InstructionSource>,
    pub claim_refs: Vec<String>,
    pub resolution_refs: Vec<String>,
    pub awareness_contract_ref: RuntimeAwarenessContractReference,
    pub extensions: BTreeMap<String, Value>,
}

impl ProjectAgentRuntimeConstitution {
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if self.schema != "focusa.project_agent_runtime_constitution.v1" {
            errors.push("unsupported_schema".to_string());
        }
        for (field, value) in [
            ("constitution_id", self.constitution_id.as_str()),
            ("project_ref", self.project_ref.as_str()),
            ("approved_spec_ref", self.approved_spec_ref.as_str()),
        ] {
            if value.trim().is_empty() {
                errors.push(format!("missing_{field}"));
            }
        }
        let mut source_ids = BTreeSet::new();
        for source in &self.instruction_sources {
            if source.source_id.trim().is_empty() || !source_ids.insert(&source.source_id) {
                errors.push("invalid_or_duplicate_instruction_source".to_string());
            }
            if source.content_sha256.len() != 64 {
                errors.push(format!("invalid_source_hash:{}", source.source_id));
            }
            if source.trust == InstructionTrustClass::Untrusted
                && self.status == RuntimeConstitutionLifecycleState::Active
            {
                errors.push(format!("active_untrusted_source:{}", source.source_id));
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload", rename_all = "snake_case")]
pub enum RuntimeConstitutionEvent {
    InstructionSourceScanStarted(Value),
    InstructionSourceDiscovered(InstructionSource),
    InstructionSourceChanged(Value),
    InstructionClaimExtracted(InstructionClaim),
    InstructionConflictDetected(InstructionConflict),
    InstructionConflictResolved(InstructionResolution),
    RuntimeConstitutionDrafted(RuntimeConstitutionVersion),
    RuntimeConstitutionApproved(RuntimeConstitutionVersion),
    RuntimeConstitutionActivated(RuntimeConstitutionVersion),
    RuntimeConstitutionRevoked(PromptRevocation),
    PromptVariantCompiled(PiPromptVariant),
    PromptVariantEvaluated(PromptEvaluation),
    PromptOutcomeTransferred(PromptEpistemicOutcomeRecord),
    ArtifactDeliveryVerified(RuntimeArtifactProjection),
    ContractDriftDetected(AgentContractImpactAssessment),
    ContractRollbackActivated(RuntimeConstitutionVersion),
}

impl RuntimeConstitutionEvent {
    pub fn event_name(&self) -> &'static str {
        match self {
            Self::InstructionSourceScanStarted(_) => "instruction.source_scan_started",
            Self::InstructionSourceDiscovered(_) => "instruction.source_discovered",
            Self::InstructionSourceChanged(_) => "instruction.source_changed",
            Self::InstructionClaimExtracted(_) => "instruction.claim_extracted",
            Self::InstructionConflictDetected(_) => "instruction.conflict_detected",
            Self::InstructionConflictResolved(_) => "instruction.conflict_resolved",
            Self::RuntimeConstitutionDrafted(_) => "runtime_constitution.drafted",
            Self::RuntimeConstitutionApproved(_) => "runtime_constitution.approved",
            Self::RuntimeConstitutionActivated(_) => "runtime_constitution.activated",
            Self::RuntimeConstitutionRevoked(_) => "runtime_constitution.revoked",
            Self::PromptVariantCompiled(_) => "prompt.variant_compiled",
            Self::PromptVariantEvaluated(_) => "prompt.variant_evaluated",
            Self::PromptOutcomeTransferred(_) => "prompt.outcome_transferred",
            Self::ArtifactDeliveryVerified(_) => "artifact.delivery_verified",
            Self::ContractDriftDetected(_) => "contract.drift_detected",
            Self::ContractRollbackActivated(_) => "contract.rollback_activated",
        }
    }
}
