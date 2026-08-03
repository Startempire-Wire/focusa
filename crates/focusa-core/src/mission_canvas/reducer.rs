use std::collections::BTreeSet;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use thiserror::Error;

use super::{
    layout::{resolve_layout, InspectorSide, LayoutConstraints, LayoutError},
    model::{
        CandidateContribution, CompositionEvent, ResolvedContribution, ResolvedWorkspaceProjection,
    },
    resolver::{resolve_eligibility, EligibilityContext, EligibilityDecision},
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
}

pub fn resolve_projection(
    input: ResolveProjectionInput,
    previous_projection_digest: Option<String>,
) -> Result<RecompositionResult, RecompositionError> {
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
    let bytes = serde_json::to_vec(&normalized)?;
    Ok(format!("sha256:{}", hex::encode(Sha256::digest(bytes))))
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
        authority: json!({"canonical_owner": "Focusa Core", "mutation_owner": "Focusa Core", "scope": scope, "read_only": false}),
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
