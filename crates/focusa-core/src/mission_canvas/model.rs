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

impl CandidateContribution {
    /// Validate the exact authority packet used to evaluate this generated
    /// candidate definition.
    ///
    /// `CandidateContribution` intentionally remains the generated registry
    /// DTO: it is not given a client-invented project/continuity owner.  The
    /// Workstream binding is supplied explicitly by the core resolver and is
    /// validated here before the candidate can participate in a projection.
    pub fn validate_scope(&self, scope: &MissionCanvasScope) -> Result<(), &'static str> {
        scope.validate()
    }
}

/// Core-only association between a generated candidate definition and the
/// exact Workstream that supplied it.  The generated CandidateContribution
/// shape remains transport-owned; this binding is never inferred from a
/// project path, continuity id, selected tab, or registry position.
#[derive(Clone, Debug, PartialEq)]
pub struct ScopedCandidateContribution {
    pub candidate: CandidateContribution,
    pub scope: MissionCanvasScope,
}

impl ScopedCandidateContribution {
    pub fn new(
        candidate: CandidateContribution,
        scope: MissionCanvasScope,
    ) -> Result<Self, &'static str> {
        candidate.validate_scope(&scope)?;
        Ok(Self { candidate, scope })
    }
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

impl ResolvedWorkspaceProjection {
    /// Validate that this projection belongs to the exact Workstream authority
    /// supplied by the caller. A project path, continuity id, focused tab, or
    /// registry row is never sufficient to select a projection.
    ///
    /// A Workstream-only expected scope may validate an aggregate projection.
    /// When the expected scope carries subordinate identity, every supplied
    /// subordinate value must match as well. This preserves the authority
    /// chain without making an aggregate read guess an Attachment or surface.
    pub fn validate_scope(&self, expected_scope: &MissionCanvasScope) -> Result<(), &'static str> {
        validate_projection_scope(&self.scope)?;
        validate_projection_scope(expected_scope)?;

        if let Some(focused_work_surface_id) = self.focused_work_surface_id.as_deref() {
            if focused_work_surface_id.trim().is_empty() {
                return Err("invalid_work_surface");
            }
            if self.scope.attachment.is_none() || self.scope.work_surface_id.is_none() {
                return Err("work_surface_authority_missing");
            }
            if self
                .scope
                .work_surface_id
                .as_ref()
                .map(WorkSurfaceId::as_str)
                != Some(focused_work_surface_id)
            {
                return Err("work_surface_mismatch");
            }
        }

        if self.scope.workstream != expected_scope.workstream {
            return Err("workstream_mismatch");
        }
        if expected_scope.continuity_id.is_some()
            && self.scope.continuity_id != expected_scope.continuity_id
        {
            return Err("continuity_mismatch");
        }
        if expected_scope.attachment.is_some() && self.scope.attachment != expected_scope.attachment
        {
            return Err("attachment_mismatch");
        }
        if expected_scope.workspace_binding_id.is_some()
            && self.scope.workspace_binding_id != expected_scope.workspace_binding_id
        {
            return Err("workspace_binding_mismatch");
        }
        if expected_scope.runtime_object.is_some()
            && self.scope.runtime_object != expected_scope.runtime_object
        {
            return Err("runtime_object_mismatch");
        }
        if expected_scope.work_surface_id.is_some()
            && self.scope.work_surface_id != expected_scope.work_surface_id
        {
            return Err("work_surface_mismatch");
        }

        for contribution in &self.eligible_contributions {
            validate_resolved_contribution_scope(contribution, &self.scope)?;
        }
        Ok(())
    }
}

/// Mission Canvas projections carry the same exact scope in each eligible
/// contribution's generated authority descriptor. Keep this check in core so
/// a deserialized or tampered projection cannot smuggle a foreign contribution
/// into an otherwise valid Workstream projection.
fn validate_resolved_contribution_scope(
    contribution: &ResolvedContribution,
    projection_scope: &MissionCanvasScope,
) -> Result<(), &'static str> {
    let authority = contribution
        .authority
        .as_object()
        .ok_or("missing_contribution_authority")?;
    if !authority.contains_key("workstream") {
        return Err("missing_contribution_workstream");
    }
    let authority_scope: MissionCanvasScope =
        serde_json::from_value(Value::Object(authority.clone()))
            .map_err(|_| "invalid_contribution_authority")?;
    validate_projection_scope(&authority_scope)?;
    if authority_scope.workstream != projection_scope.workstream {
        return Err("contribution_workstream_mismatch");
    }
    if projection_scope.continuity_id.is_some()
        && authority_scope.continuity_id != projection_scope.continuity_id
    {
        return Err("contribution_continuity_mismatch");
    }
    if projection_scope.attachment.is_some()
        && authority_scope.attachment != projection_scope.attachment
    {
        return Err("contribution_attachment_mismatch");
    }
    if projection_scope.workspace_binding_id.is_some()
        && authority_scope.workspace_binding_id != projection_scope.workspace_binding_id
    {
        return Err("contribution_workspace_binding_mismatch");
    }
    if projection_scope.runtime_object.is_some()
        && authority_scope.runtime_object != projection_scope.runtime_object
    {
        return Err("contribution_runtime_object_mismatch");
    }
    if projection_scope.work_surface_id.is_some()
        && authority_scope.work_surface_id != projection_scope.work_surface_id
    {
        return Err("contribution_work_surface_mismatch");
    }
    Ok(())
}

fn validate_projection_scope(scope: &MissionCanvasScope) -> Result<(), &'static str> {
    scope.validate()?;
    if scope.work_surface_id.is_some() && scope.attachment.is_none() {
        return Err("attachment_missing");
    }
    Ok(())
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
        AttachmentId, AttachmentKey, ContinuityId, InstanceId, ScopeRef, SessionId, WorkSurfaceId,
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

    fn projection(scope: MissionCanvasScope) -> ResolvedWorkspaceProjection {
        ResolvedWorkspaceProjection {
            schema: "focusa.resolved_workspace_projection.v1".into(),
            scope,
            workspace_profile_id: "software".into(),
            workspace_profile_revision: 2,
            activity_mode_id: "overview".into(),
            activity_mode_revision: 1,
            focused_work_surface_id: None,
            canonical_read_model_revision: 41,
            candidate_contribution_ids: vec!["contribution:pi".into()],
            eligible_contributions: vec![],
            omission_diagnostics: vec![],
            layout_tree: serde_json::json!({
                "kind": "single",
                "node_id": "layout:pi",
                "contribution_id": "contribution:pi"
            }),
            operation_bindings: vec![],
            focused_semantic_target: "semantic:pi".into(),
            projection_revision: 7,
            layout_revision: 5,
            durable_event_cursor: "event:41".into(),
            projection_digest: "sha256:projection".into(),
            resolved_at: None,
            evidence_refs: vec![],
            receipt_refs: vec![],
        }
    }

    fn resolved_contribution(authority: Value) -> ResolvedContribution {
        ResolvedContribution {
            contribution_id: "contribution:pi".into(),
            kind: ContributionKind::FocusedWorkSurface,
            semantic_binding_id: "semantic:pi".into(),
            renderer_binding_id: "renderer:pi".into(),
            data_ref: serde_json::json!({
                "kind": "work_surface",
                "ref": "surface:pi",
                "revision": 7
            }),
            operation_ids: vec![],
            authority,
            freshness: serde_json::json!({"status": "current"}),
            resolved_geometry: serde_json::json!({"preferred_regions": ["primary"]}),
            accessibility: serde_json::json!({"label": "Pi"}),
            contribution_revision: 1,
            evidence_refs: vec![],
        }
    }

    #[test]
    fn resolved_workspace_projection_scope_serializes_exact_workstream_and_revision() {
        let owner = workstream("ws:projection");
        let mut scope = MissionCanvasScope::new(
            owner.clone(),
            Some(attachment(owner, "attachment:projection")),
        )
        .expect("projection scope should be canonical");
        scope.work_surface_id = Some(WorkSurfaceId::parse("surface:pi").unwrap());
        let mut projection = projection(scope.clone());
        projection
            .eligible_contributions
            .push(resolved_contribution(
                serde_json::to_value(scope.clone()).expect("authority serializes"),
            ));
        let encoded = serde_json::to_value(&projection).expect("projection serializes");

        assert_eq!(encoded["workstream"], serde_json::json!(scope.workstream));
        assert_eq!(encoded["projection_revision"], serde_json::json!(7));
        let round_trip: ResolvedWorkspaceProjection =
            serde_json::from_value(encoded).expect("generated projection shape round-trips");
        assert_eq!(round_trip.validate_scope(&scope), Ok(()));
    }

    #[test]
    fn resolved_workspace_projection_scope_rejects_focus_without_exact_surface_authority() {
        let scope = MissionCanvasScope::new(workstream("ws:focus"), None).unwrap();
        let mut unbound_projection = projection(scope.clone());
        unbound_projection.focused_work_surface_id = Some("surface:legacy".into());
        assert_eq!(
            unbound_projection.validate_scope(&scope),
            Err("work_surface_authority_missing")
        );

        let owner = workstream("ws:focus");
        let mut bound_scope =
            MissionCanvasScope::new(owner.clone(), Some(attachment(owner, "attachment:focus")))
                .unwrap();
        bound_scope.work_surface_id = Some(WorkSurfaceId::parse("surface:actual").unwrap());
        let mut mismatched = projection(bound_scope.clone());
        mismatched.focused_work_surface_id = Some("surface:other".into());
        assert_eq!(
            mismatched.validate_scope(&bound_scope),
            Err("work_surface_mismatch")
        );
    }

    #[test]
    fn resolved_workspace_projection_scope_allows_workstream_aggregate_contributions() {
        let owner = workstream("ws:aggregate");
        let aggregate = MissionCanvasScope::new(owner.clone(), None).unwrap();
        let attachment_scope = MissionCanvasScope::new(
            owner.clone(),
            Some(attachment(owner, "attachment:aggregate")),
        )
        .unwrap();
        let mut projection = projection(aggregate.clone());
        projection
            .eligible_contributions
            .push(resolved_contribution(
                serde_json::to_value(attachment_scope).expect("authority serializes"),
            ));

        assert_eq!(projection.validate_scope(&aggregate), Ok(()));
    }

    #[test]
    fn resolved_workspace_projection_scope_rejects_foreign_workstream_and_attachment() {
        let owner = workstream("ws:projection");
        let scope = MissionCanvasScope::new(
            owner.clone(),
            Some(attachment(owner.clone(), "attachment:projection")),
        )
        .unwrap();
        let projection = projection(scope);

        let foreign_owner = workstream("ws:foreign");
        let foreign = MissionCanvasScope::new(
            foreign_owner.clone(),
            Some(attachment(foreign_owner, "attachment:foreign")),
        )
        .unwrap();
        assert_eq!(
            projection.validate_scope(&foreign),
            Err("workstream_mismatch")
        );

        let different_attachment =
            MissionCanvasScope::new(owner.clone(), Some(attachment(owner, "attachment:other")))
                .unwrap();
        assert_eq!(
            projection.validate_scope(&different_attachment),
            Err("attachment_mismatch")
        );
    }

    #[test]
    fn resolved_workspace_projection_scope_rejects_missing_and_foreign_contribution_authority() {
        let owner = workstream("ws:projection");
        let scope = MissionCanvasScope::new(
            owner.clone(),
            Some(attachment(owner.clone(), "attachment:projection")),
        )
        .unwrap();
        let mut missing = projection(scope.clone());
        missing
            .eligible_contributions
            .push(resolved_contribution(serde_json::json!({})));
        assert_eq!(
            missing.validate_scope(&scope),
            Err("missing_contribution_workstream")
        );

        let foreign_owner = workstream("ws:foreign");
        let foreign_scope = MissionCanvasScope::new(
            foreign_owner.clone(),
            Some(attachment(foreign_owner, "attachment:foreign")),
        )
        .unwrap();
        let mut foreign = projection(scope.clone());
        foreign.eligible_contributions.push(resolved_contribution(
            serde_json::to_value(foreign_scope).expect("authority serializes"),
        ));
        assert_eq!(
            foreign.validate_scope(&scope),
            Err("contribution_workstream_mismatch")
        );
    }

    #[test]
    fn resolved_workspace_projection_scope_rejects_legacy_or_invalid_authority() {
        let scope = MissionCanvasScope::new(workstream("ws:projection"), None).unwrap();
        let projection = projection(scope.clone());
        let mut legacy = serde_json::to_value(&projection).unwrap();
        legacy.as_object_mut().unwrap().remove("workstream");
        assert!(serde_json::from_value::<ResolvedWorkspaceProjection>(legacy).is_err());

        let mut invalid = serde_json::to_value(&projection).unwrap();
        invalid["workstream"]["workstream_id"] = serde_json::json!("");
        let invalid: ResolvedWorkspaceProjection = serde_json::from_value(invalid).unwrap();
        assert_eq!(invalid.validate_scope(&scope), Err("missing_workstream"));
    }
}
