use std::collections::BTreeSet;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use thiserror::Error;

use super::{
    layout::{InspectorSide, LayoutConstraints, LayoutError, resolve_layout},
    model::{
        CandidateContribution, CompositionEvent, ResolvedContribution, ResolvedWorkspaceProjection,
    },
    resolver::{EligibilityContext, EligibilityDecision, resolve_eligibility},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResolveProjectionInput {
    pub candidates: Vec<CandidateContribution>,
    pub eligibility: EligibilityContext,
    pub workspace_profile_revision: u64,
    pub activity_mode_revision: u64,
    pub focused_work_surface_id: Option<String>,
    pub canonical_read_model_revision: u64,
    pub viewport_width: u32,
    pub viewport_height: u32,
    pub viewport_class: String,
    pub focused_semantic_target: String,
    pub previous_projection_revision: u64,
    pub previous_layout_revision: u64,
    pub event_cursor: String,
    pub causation_id: Option<String>,
    pub idempotency_key: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecompositionEvidence {
    pub evidence_id: String,
    #[serde(flatten)]
    pub scope: super::model::MissionCanvasScope,
    pub trigger: String,
    pub input_projection_digest: Option<String>,
    pub output_projection_digest: String,
    pub rule_revision: String,
    pub candidate_contribution_ids: Vec<String>,
    pub eligibility_decisions: Vec<EligibilityDecision>,
    pub observed_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecompositionReceipt {
    pub receipt_id: String,
    #[serde(flatten)]
    pub scope: super::model::MissionCanvasScope,
    pub accepted: bool,
    pub projection_revision: u64,
    pub layout_revision: u64,
    pub projection_digest: String,
    pub event_cursor: String,
    pub evidence_id: String,
    pub idempotency_key: String,
    pub issued_at: String,
}

#[derive(Clone, Debug)]
pub struct RecompositionResult {
    pub projection: ResolvedWorkspaceProjection,
    pub evidence: RecompositionEvidence,
    pub receipt: RecompositionReceipt,
    pub event: CompositionEvent,
}

#[derive(Debug, Error)]
pub enum RecompositionError {
    #[error(transparent)]
    Layout(#[from] LayoutError),
    #[error("projection serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("invalid Workstream authority: {0}")]
    Identity(&'static str),
}

pub fn resolve_projection(
    input: ResolveProjectionInput,
    previous_projection_digest: Option<String>,
) -> Result<RecompositionResult, RecompositionError> {
    input
        .eligibility
        .scope
        .validate()
        .map_err(RecompositionError::Identity)?;
    let now = Utc::now().to_rfc3339();
    let candidate_ids = input
        .candidates
        .iter()
        .map(|candidate| candidate.contribution_id.clone())
        .collect::<Vec<_>>();
    let eligibility = resolve_eligibility(input.candidates, &input.eligibility);
    let layout = resolve_layout(
        &eligibility.eligible,
        &LayoutConstraints {
            viewport_width: input.viewport_width,
            viewport_height: input.viewport_height,
            minimum_primary_span: if input.viewport_width <= 1024 { 8 } else { 6 },
            inspector_side: InspectorSide::End,
            focused_contribution_id: focused_contribution_id(
                &eligibility.eligible,
                &input.focused_semantic_target,
            ),
        },
    )?;
    let projection_revision = input.previous_projection_revision + 1;
    let layout_revision = input.previous_layout_revision + 1;
    let resolved_contributions = eligibility
        .eligible
        .iter()
        .map(|candidate| resolve_contribution(candidate, &input.eligibility.scope, &now))
        .collect::<Vec<_>>();
    let operation_bindings = resolved_contributions
        .iter()
        .flat_map(|contribution| {
            contribution.operation_ids.iter().map(|operation_id| {
                json!({
                    "operation_id": operation_id,
                    "target_contribution_id": contribution.contribution_id,
                    "enabled": true,
                    "authority_ref": format!("authority:{}", contribution.contribution_id),
                    "confirmation": "none",
                    "disabled_reason_ref": Value::Null,
                })
            })
        })
        .collect::<Vec<_>>();
    let mut projection = ResolvedWorkspaceProjection {
        schema: "focusa.resolved_workspace_projection.v1".into(),
        scope: input.eligibility.scope.clone(),
        workspace_profile_id: input.eligibility.profile_id.clone(),
        workspace_profile_revision: input.workspace_profile_revision,
        activity_mode_id: input.eligibility.activity_mode_id.clone(),
        activity_mode_revision: input.activity_mode_revision,
        focused_work_surface_id: input.focused_work_surface_id,
        canonical_read_model_revision: input.canonical_read_model_revision,
        candidate_contribution_ids: candidate_ids.clone(),
        eligible_contributions: resolved_contributions,
        omission_diagnostics: eligibility.omissions,
        layout_tree: serde_json::to_value(layout)?,
        operation_bindings,
        focused_semantic_target: input.focused_semantic_target,
        projection_revision,
        layout_revision,
        durable_event_cursor: input.event_cursor.clone(),
        projection_digest: String::new(),
        resolved_at: Some(now.clone()),
        evidence_refs: vec![],
        receipt_refs: vec![],
    };
    projection.projection_digest = projection_digest(&projection)?;
    let evidence_id = format!("recomposition-evidence:{projection_revision}");
    let receipt_id = format!("recomposition-receipt:{projection_revision}");
    projection.evidence_refs.push(evidence_id.clone());
    projection.receipt_refs.push(receipt_id.clone());
    let evidence = RecompositionEvidence {
        evidence_id: evidence_id.clone(),
        scope: input.eligibility.scope.clone(),
        trigger: "explicit_resolve".into(),
        input_projection_digest: previous_projection_digest,
        output_projection_digest: projection.projection_digest.clone(),
        rule_revision: super::resolver::RESOLVER_RULE_REVISION.into(),
        candidate_contribution_ids: candidate_ids,
        eligibility_decisions: eligibility.decisions,
        observed_at: now.clone(),
    };
    let receipt = RecompositionReceipt {
        receipt_id: receipt_id.clone(),
        scope: input.eligibility.scope.clone(),
        accepted: true,
        projection_revision,
        layout_revision,
        projection_digest: projection.projection_digest.clone(),
        event_cursor: input.event_cursor.clone(),
        evidence_id: evidence_id.clone(),
        idempotency_key: input.idempotency_key,
        issued_at: now.clone(),
    };
    let event = CompositionEvent {
        event_id: format!("projection-event:{projection_revision}"),
        event_kind: "projection_resolved".into(),
        scope: projection.scope.clone(),
        projection_revision,
        layout_revision,
        causation_id: input.causation_id,
        correlation_id: Some(format!("resolve:{projection_revision}")),
        occurred_at: now,
        payload: json!({
            "projection_digest": projection.projection_digest,
            "viewport_class": input.viewport_class,
            "evidence": evidence,
            "receipt": receipt,
            "omission_diagnostics": projection.omission_diagnostics,
        }),
        evidence_refs: vec![evidence_id],
        receipt_refs: vec![receipt_id],
    };
    Ok(RecompositionResult {
        projection,
        evidence,
        receipt,
        event,
    })
}

/// Hash the canonical projection material, including the flattened
/// WorkstreamKey and the semantic, projection, layout, and event-cursor
/// revisions carried by `ResolvedWorkspaceProjection`.
///
/// Evidence/Receipt references and resolution time are produced around the
/// digest itself, so they are deliberately excluded. Object keys are sorted
/// recursively for a stable transport-independent digest; array order remains
/// meaningful because it is part of the resolved composition.
pub fn projection_digest(
    projection: &ResolvedWorkspaceProjection,
) -> Result<String, serde_json::Error> {
    let mut normalized = serde_json::to_value(projection)?;
    if let Some(object) = normalized.as_object_mut() {
        object.remove("projection_digest");
        object.remove("resolved_at");
        object.remove("evidence_refs");
        object.remove("receipt_refs");
    }
    let normalized = canonical_json(&normalized);
    let bytes = serde_json::to_vec(&normalized)?;
    Ok(format!("sha256:{}", hex::encode(Sha256::digest(bytes))))
}

/// Normalize JSON object key order without changing array order or values.
/// Arrays encode ordered projection decisions/layout children and therefore
/// must not be treated as sets.
fn canonical_json(value: &Value) -> Value {
    match value {
        Value::Object(object) => {
            let mut entries: Vec<(&String, &Value)> = object.iter().collect();
            entries.sort_by(|left, right| left.0.cmp(right.0));
            let mut normalized = Map::new();
            for (key, child) in entries {
                normalized.insert(key.clone(), canonical_json(child));
            }
            Value::Object(normalized)
        }
        Value::Array(values) => Value::Array(values.iter().map(canonical_json).collect()),
        other => other.clone(),
    }
}

fn focused_contribution_id(
    candidates: &[CandidateContribution],
    focused_semantic_target: &str,
) -> Option<String> {
    candidates
        .iter()
        .find(|candidate| candidate.semantic_binding_id == focused_semantic_target)
        .map(|candidate| candidate.contribution_id.clone())
}

fn resolve_contribution(
    candidate: &CandidateContribution,
    scope: &super::model::MissionCanvasScope,
    observed_at: &str,
) -> ResolvedContribution {
    let data_ref = candidate
        .canonical_content_refs
        .first()
        .cloned()
        .unwrap_or_else(|| json!({"kind": "none", "ref": "none", "revision": 0}));
    ResolvedContribution {
        contribution_id: candidate.contribution_id.clone(),
        kind: candidate.kind.clone(),
        semantic_binding_id: candidate.semantic_binding_id.clone(),
        renderer_binding_id: candidate.renderer_binding_id.clone(),
        data_ref,
        operation_ids: candidate.required_operations.clone(),
        authority: json!({
            "canonical_owner": "Focusa Core",
            "mutation_owner": "Focusa Core",
            "workstream": scope.workstream,
            "continuity_id": scope.continuity_id,
            "attachment": scope.attachment,
            "workspace_binding_id": scope.workspace_binding_id,
            "runtime_object": scope.runtime_object,
            "work_surface_id": scope.work_surface_id,
            "read_only": false
        }),
        freshness: json!({"status": "current", "observed_at": observed_at}),
        resolved_geometry: candidate.geometry.clone(),
        accessibility: json!({"label": candidate.contribution_id, "landmark_role": "region", "focus_semantic_id": candidate.semantic_binding_id}),
        contribution_revision: 1,
        evidence_refs: vec![],
    }
}

pub fn candidate_partition_is_complete(projection: &ResolvedWorkspaceProjection) -> bool {
    let candidates = projection
        .candidate_contribution_ids
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let eligible = projection
        .eligible_contributions
        .iter()
        .map(|contribution| contribution.contribution_id.clone())
        .collect::<BTreeSet<_>>();
    let omitted = projection
        .omission_diagnostics
        .iter()
        .map(|diagnostic| diagnostic.contribution_id.clone())
        .collect::<BTreeSet<_>>();
    eligible.is_disjoint(&omitted) && candidates == eligible.union(&omitted).cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mission_canvas::model::{MissionCanvasScope, OmissionDiagnostic};
    use crate::scoped_state::ScopeRef as LegacyScopeRef;
    use crate::workstream_identity::{ScopeRef, WorkstreamId, WorkstreamKey};
    use serde_json::json;

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

    fn projection(workstream_id: &str) -> ResolvedWorkspaceProjection {
        ResolvedWorkspaceProjection {
            schema: "focusa.resolved_workspace_projection.v1".into(),
            scope: MissionCanvasScope::new(workstream(workstream_id), None).unwrap(),
            workspace_profile_id: "software".into(),
            workspace_profile_revision: 2,
            activity_mode_id: "overview".into(),
            activity_mode_revision: 1,
            focused_work_surface_id: None,
            canonical_read_model_revision: 41,
            candidate_contribution_ids: vec!["contribution:primary".into()],
            eligible_contributions: vec![],
            omission_diagnostics: vec![],
            layout_tree: json!({
                "kind": "split",
                "node_id": "layout:root",
                "orientation": "horizontal",
                "ratio": 0.5,
                "children": [
                    {"kind": "single", "node_id": "layout:primary", "contribution_id": "contribution:primary"},
                    {"kind": "single", "node_id": "layout:inspector", "contribution_id": "contribution:inspector"}
                ]
            }),
            operation_bindings: vec![],
            focused_semantic_target: "semantic:primary".into(),
            projection_revision: 7,
            layout_revision: 5,
            durable_event_cursor: "event:41".into(),
            projection_digest: "sha256:placeholder".into(),
            resolved_at: None,
            evidence_refs: vec![],
            receipt_refs: vec![],
        }
    }

    #[test]
    fn mission_canvas_projection_digest_distinguishes_equal_layouts_under_different_workstreams() {
        let local = projection("ws:local");
        let foreign = projection("ws:foreign");

        assert_eq!(local.layout_tree, foreign.layout_tree);
        assert_ne!(local.scope.workstream, foreign.scope.workstream);
        assert_eq!(
            serde_json::to_value(&local).unwrap()["workstream"]["workstream_id"],
            json!("ws:local")
        );
        assert_ne!(
            projection_digest(&local).unwrap(),
            projection_digest(&foreign).unwrap(),
            "a digest must not collapse equal layouts across WorkstreamKey boundaries"
        );
    }

    #[test]
    fn mission_canvas_projection_digest_includes_semantic_and_layout_revisions() {
        let base = projection("ws:revisions");
        let base_digest = projection_digest(&base).unwrap();

        let mut profile_revision = base.clone();
        profile_revision.workspace_profile_revision += 1;
        assert_ne!(projection_digest(&profile_revision).unwrap(), base_digest);

        let mut activity_revision = base.clone();
        activity_revision.activity_mode_revision += 1;
        assert_ne!(projection_digest(&activity_revision).unwrap(), base_digest);

        let mut semantic_read_model_revision = base.clone();
        semantic_read_model_revision.canonical_read_model_revision += 1;
        assert_ne!(
            projection_digest(&semantic_read_model_revision).unwrap(),
            base_digest
        );

        let mut projection_revision = base.clone();
        projection_revision.projection_revision += 1;
        assert_ne!(
            projection_digest(&projection_revision).unwrap(),
            base_digest
        );

        let mut layout_revision = base.clone();
        layout_revision.layout_revision += 1;
        assert_ne!(projection_digest(&layout_revision).unwrap(), base_digest);

        let mut cursor = base.clone();
        cursor.durable_event_cursor = "event:42".into();
        assert_ne!(
            projection_digest(&cursor).unwrap(),
            base_digest,
            "a stale/replayed cursor must not share a digest with current projection state"
        );
    }

    #[test]
    fn mission_canvas_projection_digest_ignores_only_volatile_proof_metadata() {
        let base = projection("ws:metadata");
        let base_digest = projection_digest(&base).unwrap();
        let mut metadata = base.clone();
        metadata.projection_digest = "sha256:another-value".into();
        metadata.resolved_at = Some("2026-08-07T00:00:00Z".into());
        metadata.evidence_refs = vec!["evidence:recomposition".into()];
        metadata.receipt_refs = vec!["receipt:recomposition".into()];

        assert_eq!(
            projection_digest(&metadata).unwrap(),
            base_digest,
            "proof links and resolution time must not create a recursive digest"
        );
    }

    #[test]
    fn mission_canvas_projection_digest_is_stable_for_object_key_order() {
        let mut first = projection("ws:canonical-json");
        first.layout_tree =
            serde_json::from_str(r#"{"z":{"b":2,"a":1},"a":[{"d":4,"c":3}],"kind":"single"}"#)
                .unwrap();
        let mut second = first.clone();
        second.layout_tree =
            serde_json::from_str(r#"{"kind":"single","a":[{"c":3,"d":4}],"z":{"a":1,"b":2}}"#)
                .unwrap();

        assert_eq!(
            projection_digest(&first).unwrap(),
            projection_digest(&second).unwrap(),
            "JSON object ordering is not semantic projection state"
        );
    }

    #[test]
    fn mission_canvas_projection_digest_preserves_fail_closed_omissions() {
        let base = projection("ws:omissions");
        let base_digest = projection_digest(&base).unwrap();
        let mut omitted = base;
        omitted.candidate_contribution_ids = vec!["contribution:empty".into()];
        omitted.omission_diagnostics = vec![OmissionDiagnostic {
            contribution_id: "contribution:empty".into(),
            reason: "capability_not_present".into(),
            rule_revision: "adaptive-composition:v1".into(),
            projection_revision: omitted.projection_revision,
            canonical_input_refs: vec![],
            details_ref: Some("diagnostic:capability_not_present".into()),
            observed_at: "2026-08-07T00:00:00Z".into(),
        }];

        assert!(omitted.eligible_contributions.is_empty());
        assert_ne!(
            projection_digest(&omitted).unwrap(),
            base_digest,
            "unavailable contributions remain omitted and observable in canonical digest material"
        );
    }

    #[test]
    fn mission_canvas_projection_digest_never_repairs_missing_or_foreign_authority() {
        let local = projection("ws:local");
        let foreign = projection("ws:foreign");
        assert_ne!(
            projection_digest(&local).unwrap(),
            projection_digest(&foreign).unwrap()
        );

        let mut legacy = serde_json::to_value(&local).unwrap();
        legacy.as_object_mut().unwrap().remove("workstream");
        assert!(
            serde_json::from_value::<ResolvedWorkspaceProjection>(legacy).is_err(),
            "a legacy project/continuity row cannot be repaired into a canonical projection"
        );
    }
}
