use crate::workstream_identity::{
    AttachmentKey, ContinuityId, RuntimeObjectRef, WorkSurfaceId, WorkspaceBindingId, WorkstreamKey,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Canonical Mission Canvas authority context.
///
/// The old flat project/continuity/session DTO is intentionally gone from the
/// canonical model.  A compatibility caller must resolve its legacy input to
/// this Workstream-owned chain before entering Mission Canvas state.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MissionCanvasAuthorityContext {
    pub workstream: WorkstreamKey,
    pub continuity_id: Option<ContinuityId>,
    pub attachment: Option<AttachmentKey>,
    pub workspace_binding_id: Option<WorkspaceBindingId>,
    pub runtime_object: Option<RuntimeObjectRef>,
    pub work_surface_id: Option<WorkSurfaceId>,
}

impl MissionCanvasAuthorityContext {
    pub fn storage_key(&self) -> String {
        serde_json::to_string(self).expect("MissionCanvas authority context is serializable")
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.workstream.workstream_id.as_str().trim().is_empty() {
            return Err("missing_workstream");
        }
        if self
            .continuity_id
            .as_ref()
            .is_some_and(|value| value.as_str().trim().is_empty())
        {
            return Err("invalid_continuity");
        }
        if self
            .workspace_binding_id
            .as_ref()
            .is_some_and(|value| value.as_str().trim().is_empty())
        {
            return Err("invalid_workspace_binding");
        }
        self.workstream
            .legacy_scope()
            .validate()
            .map_err(|_| "invalid_workstream_scope")?;
        if let Some(runtime) = self.runtime_object.as_ref() {
            if runtime.runtime_kind.trim().is_empty() || runtime.runtime_id.trim().is_empty() {
                return Err("invalid_runtime_object");
            }
        }
        if let Some(work_surface_id) = self.work_surface_id.as_ref() {
            if work_surface_id.as_str().trim().is_empty() {
                return Err("invalid_work_surface");
            }
        }
        if let Some(attachment) = self.attachment.as_ref() {
            if attachment.instance_id.as_str().trim().is_empty()
                || attachment.session_id.as_str().trim().is_empty()
                || attachment.attachment_id.as_str().trim().is_empty()
                || attachment.workspace_binding_id.as_str().trim().is_empty()
            {
                return Err("invalid_attachment");
            }
            attachment
                .validate_owner(&self.workstream)
                .map_err(|_| "foreign_attachment_workstream")?;
            if let (Some(requested), Some(attached)) =
                (&self.continuity_id, &attachment.continuity_id)
            {
                if requested != attached {
                    return Err("continuity_mismatch");
                }
            }
            if self.workspace_binding_id.as_ref() != Some(&attachment.workspace_binding_id) {
                return Err("workspace_binding_mismatch");
            }
        }
        Ok(())
    }
}

/// Generated-contract name for the Mission Canvas authority context.
pub type WorkstreamAuthorityContext = MissionCanvasAuthorityContext;

/// Transitional Rust name retained for bounded Mission Canvas internals.  It
/// is a canonical WorkstreamAuthorityContext, not the removed flat DTO.
pub type MissionCanvasScope = WorkstreamAuthorityContext;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContributionKind {
    WorkSurfaceStrip,
    FocusedWorkSurface,
    Inspector,
    InspectorSection,
    WorkRail,
    SteeringQueue,
    FollowUpQueue,
    PromptEditor,
    ScopeBar,
    ActivityNavigation,
    ToolbarControl,
    ContextualAction,
    TransientNotification,
    GeneratedSurface,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CandidateContribution {
    pub contribution_id: String,
    pub kind: ContributionKind,
    pub semantic_binding_id: String,
    pub renderer_binding_id: String,
    pub priority: i64,
    pub applicable_profile_ids: Vec<String>,
    pub applicable_activity_mode_ids: Vec<String>,
    #[serde(default)]
    pub canonical_content_refs: Vec<Value>,
    #[serde(default)]
    pub required_capabilities: Vec<String>,
    #[serde(default)]
    pub required_permissions: Vec<String>,
    #[serde(default)]
    pub required_operations: Vec<String>,
    pub geometry: Value,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ResolvedContribution {
    pub contribution_id: String,
    pub kind: ContributionKind,
    pub semantic_binding_id: String,
    pub renderer_binding_id: String,
    pub data_ref: Value,
    pub operation_ids: Vec<String>,
    pub authority: Value,
    pub freshness: Value,
    pub resolved_geometry: Value,
    pub accessibility: Value,
    pub contribution_revision: u64,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OmissionDiagnostic {
    pub contribution_id: String,
    pub reason: String,
    pub rule_revision: String,
    pub projection_revision: u64,
    #[serde(default)]
    pub canonical_input_refs: Vec<Value>,
    pub details_ref: Option<String>,
    pub observed_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ResolvedWorkspaceProjection {
    pub schema: String,
    #[serde(flatten)]
    pub scope: MissionCanvasScope,
    pub workspace_profile_id: String,
    pub workspace_profile_revision: u64,
    pub activity_mode_id: String,
    pub activity_mode_revision: u64,
    pub focused_work_surface_id: Option<String>,
    pub canonical_read_model_revision: u64,
    pub candidate_contribution_ids: Vec<String>,
    pub eligible_contributions: Vec<ResolvedContribution>,
    pub omission_diagnostics: Vec<OmissionDiagnostic>,
    pub layout_tree: Value,
    pub operation_bindings: Vec<Value>,
    pub focused_semantic_target: String,
    pub projection_revision: u64,
    pub layout_revision: u64,
    pub durable_event_cursor: String,
    pub projection_digest: String,
    pub resolved_at: Option<String>,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    #[serde(default)]
    pub receipt_refs: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CompositionEvent {
    pub event_id: String,
    pub event_kind: String,
    #[serde(flatten)]
    pub scope: MissionCanvasScope,
    pub projection_revision: u64,
    pub layout_revision: u64,
    pub causation_id: Option<String>,
    pub correlation_id: Option<String>,
    pub occurred_at: String,
    pub payload: Value,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    #[serde(default)]
    pub receipt_refs: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StoredDocument {
    pub document_id: String,
    #[serde(flatten)]
    pub scope: MissionCanvasScope,
    pub revision: u64,
    pub payload: Value,
    pub updated_at: String,
}
