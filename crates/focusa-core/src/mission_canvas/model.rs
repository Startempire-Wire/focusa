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
    /// Construct canonical Mission Canvas scope from the generated Workstream
    /// identity and, when present, its exact Attachment owner.
    pub fn new(
        workstream: WorkstreamKey,
        attachment: Option<AttachmentKey>,
    ) -> Result<Self, &'static str> {
        let continuity_id = attachment
            .as_ref()
            .and_then(|value| value.continuity_id.clone());
        let workspace_binding_id = attachment
            .as_ref()
            .map(|value| value.workspace_binding_id.clone());
        Self::from_parts(
            workstream,
            continuity_id,
            attachment,
            workspace_binding_id,
            None,
            None,
        )
    }

    /// Adapter for generated authority DTOs that carry subordinate context
    /// alongside the canonical WorkstreamKey.  No field is inferred from CWD,
    /// a current tab, or a legacy record.
    pub fn from_parts(
        workstream: WorkstreamKey,
        continuity_id: Option<ContinuityId>,
        attachment: Option<AttachmentKey>,
        workspace_binding_id: Option<WorkspaceBindingId>,
        runtime_object: Option<RuntimeObjectRef>,
        work_surface_id: Option<WorkSurfaceId>,
    ) -> Result<Self, &'static str> {
        let context = Self {
            workstream,
            continuity_id,
            attachment,
            workspace_binding_id,
            runtime_object,
            work_surface_id,
        };
        context.validate()?;
        Ok(context)
    }

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
                .workstream
                .validate()
                .map_err(|_| "invalid_attachment_workstream")?;
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

/// The governed transport receipt for `focusa.mission_canvas.domain_pack.install`.
///
/// The public DTO is deliberately small and generated-contract shaped.  The
/// idempotency key, authority, request digest, and durable event cursor remain
/// in the core receipt ledger rather than being client-invented response data.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainPackInstallReceipt {
    pub schema: String,
    pub workstream: WorkstreamKey,
    pub installed: bool,
    pub pack_id: String,
    pub receipt_ref: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GovernedDomainPackInstallReceipt {
    pub receipt: DomainPackInstallReceipt,
    pub idempotency_key: String,
    pub request_digest: String,
    pub authority_ref: String,
    pub event_cursor: String,
    pub issued_at: String,
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scoped_state::ScopeRef as LegacyScopeRef;
    use crate::workstream_identity::{
        AttachmentId, AttachmentKey, ContinuityId, InstanceId, ScopeRef, SessionId,
        WorkspaceBindingId, WorkstreamId, WorkstreamKey,
    };

    fn workstream(id: &str) -> WorkstreamKey {
        let legacy = LegacyScopeRef::project(
            "project:focusa",
            "/workspace/focusa",
            "Focusa",
            "host-a:worktree-main",
        )
        .unwrap();
        WorkstreamKey::new(
            ScopeRef::project(legacy).unwrap(),
            WorkstreamId::parse(id).unwrap(),
        )
    }

    fn attachment(owner: WorkstreamKey, id: &str) -> AttachmentKey {
        AttachmentKey::new(
            owner,
            Some(ContinuityId::parse("continuity:mission-canvas").unwrap()),
            InstanceId::parse("instance:pi").unwrap(),
            SessionId::parse("session:pi").unwrap(),
            AttachmentId::parse(id).unwrap(),
            WorkspaceBindingId::parse("workspace:mission-canvas").unwrap(),
        )
    }

    #[test]
    fn mission_canvas_scope_new_requires_exact_attachment_owner() {
        let owner = workstream("ws:mission-canvas");
        let scope = MissionCanvasScope::new(
            owner.clone(),
            Some(attachment(owner.clone(), "attachment:pi")),
        )
        .expect("canonical Workstream and Attachment should construct a scope");
        assert_eq!(scope.workstream, owner);
        assert_eq!(
            scope.attachment.as_ref().unwrap().attachment_id.as_str(),
            "attachment:pi"
        );

        let foreign = attachment(workstream("ws:other"), "attachment:foreign");
        assert_eq!(
            MissionCanvasScope::new(workstream("ws:mission-canvas"), Some(foreign)).unwrap_err(),
            "foreign_attachment_workstream"
        );
    }

    #[test]
    fn mission_canvas_scope_new_allows_workstream_only_scope() {
        let scope = MissionCanvasScope::new(workstream("ws:overview"), None)
            .expect("a Workstream-only aggregate scope is canonical");
        assert!(scope.attachment.is_none());
        assert!(scope.validate().is_ok());
    }
}
