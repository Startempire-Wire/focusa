use crate::server::AppState;
use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
};
use chrono::{DateTime, Utc};
use focusa_core::{
    tool_result::{FailureClass, ToolResultV1, ToolStatus},
    types::{
        Action, FocusaEvent, WorkspaceArtifactContent, WorkspaceArtifactEvidenceStatus,
        WorkspaceArtifactOrigin, WorkspaceArtifactRecord, WorkspaceArtifactRender,
        WorkspaceArtifactRetention, WorkspaceArtifactScope, WorkspaceArtifactSemantic,
        WorkspaceArtifactSource, WorkspaceArtifactTrust,
    },
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{sync::Arc, time::Duration};

type ApiError = (StatusCode, Json<Box<ToolResultV1>>);

#[derive(Debug, Deserialize)]
pub struct WorkspaceArtifactIntakeRequest {
    pub project_root: String,
    pub continuity_id: String,
    pub attachment_id: String,
    pub idempotency_key: String,
    pub expected_state_version: u64,
    pub artifact_kind: String,
    pub mime_type: String,
    pub title: String,
    pub summary: String,
    pub handle_ref: String,
    #[serde(default)]
    pub artifact_url: Option<String>,
    #[serde(default)]
    pub artifact_path: Option<String>,
    #[serde(default)]
    pub inline_preview: Option<String>,
    pub sha256: String,
    pub size_bytes: u64,
    pub source_system: String,
    pub source_ref: String,
    #[serde(default)]
    pub source_url: Option<String>,
    #[serde(default)]
    pub captured_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub project_identity_ref: Option<String>,
    #[serde(default)]
    pub workpoint_id: Option<String>,
    #[serde(default)]
    pub work_item_ref: Option<String>,
    pub instance_id: String,
    #[serde(default)]
    pub focusa_session_id: Option<String>,
    #[serde(default)]
    pub work_surface_id: Option<String>,
    #[serde(default)]
    pub harness_session_ref: Option<String>,
    #[serde(default)]
    pub silent_session_id: Option<String>,
    #[serde(default)]
    pub silent_run_id: Option<String>,
    #[serde(default)]
    pub uiai_session_id: Option<String>,
    #[serde(default)]
    pub browser_context_id: Option<String>,
    #[serde(default)]
    pub browser_target_id: Option<String>,
    #[serde(default)]
    pub diagnostics_refs: Vec<String>,
    pub evidence_refs: Vec<String>,
    #[serde(default)]
    pub domain_pack_refs: Vec<String>,
    #[serde(default)]
    pub candidate_object_refs: Vec<String>,
    #[serde(default)]
    pub candidate_link_refs: Vec<String>,
    #[serde(default)]
    pub candidate_claim_refs: Vec<String>,
    #[serde(default)]
    pub verification_policy_refs: Vec<String>,
    #[serde(default)]
    pub semantic_delta_refs: Vec<String>,
    #[serde(default)]
    pub citation_refs: Vec<String>,
    pub evidence_status: String,
    pub redaction_status: String,
    pub freshness_status: String,
    pub provenance_status: String,
    pub retention_policy: String,
    #[serde(default)]
    pub expires_at: Option<DateTime<Utc>>,
    pub cleanup_action: String,
    pub preferred_renderer: String,
    pub fallback_renderer: String,
    #[serde(default)]
    pub render_width: Option<u32>,
    #[serde(default)]
    pub render_height: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct WorkspaceArtifactQuery {
    pub project_root: String,
    pub continuity_id: String,
    pub attachment_id: String,
}

#[derive(Debug, Serialize)]
pub struct WorkspaceArtifactIntakeResponse {
    pub schema: &'static str,
    pub canonical_link: bool,
    pub external_artifact_authority: bool,
    pub replayed: bool,
    pub state_version: u64,
    pub artifact: WorkspaceArtifactRecord,
    pub evidence_ref: String,
    pub receipt_ref: String,
    pub tool_result: ToolResultV1,
}
#[derive(Debug, Serialize)]
pub struct WorkspaceArtifactListResponse {
    pub schema: &'static str,
    pub canonical_links: bool,
    pub external_artifact_authority: bool,
    pub state_version: u64,
    pub artifacts: Vec<WorkspaceArtifactRecord>,
}

fn fail(
    status: StatusCode,
    tool_status: ToolStatus,
    class: FailureClass,
    summary: impl Into<String>,
) -> ApiError {
    let mut result = ToolResultV1::failure(tool_status, class, summary);
    result.tool = Some("focusa_workspace_artifact_intake".into());
    result.family = Some("workspace_artifact".into());
    result.endpoint = Some("/v1/workspace/artifacts/intake".into());
    result.next_tools = vec![
        "focusa_workspace_artifacts_list".into(),
        "focusa_evidence_capture".into(),
    ];
    (status, Json(Box::new(result)))
}
fn text(value: &str, field: &str, max: usize) -> Result<String, ApiError> {
    let value = value.trim();
    if value.is_empty() || value.len() > max {
        return Err(fail(
            StatusCode::UNPROCESSABLE_ENTITY,
            ToolStatus::ValidationRejected,
            FailureClass::ValidationRejected,
            format!("{field} must contain 1-{max} characters"),
        ));
    }
    Ok(value.into())
}
fn optional(value: Option<String>, field: &str, max: usize) -> Result<Option<String>, ApiError> {
    value.map(|value| text(&value, field, max)).transpose()
}
fn stable(prefix: &str, parts: &[&str]) -> String {
    let mut h = Sha256::new();
    for p in parts {
        h.update((p.len() as u64).to_be_bytes());
        h.update(p.as_bytes())
    }
    format!("{prefix}:{}", hex::encode(&h.finalize()[..12]))
}
fn scoped(artifact: &WorkspaceArtifactRecord, q: &WorkspaceArtifactQuery) -> bool {
    artifact.scope.project_root == q.project_root
        && artifact.scope.continuity_id == q.continuity_id
        && artifact.origin.attachment_id == q.attachment_id
}
fn refs(values: Vec<String>, field: &str, min: usize, max: usize) -> Result<Vec<String>, ApiError> {
    if values.len() < min || values.len() > max {
        return Err(fail(
            StatusCode::UNPROCESSABLE_ENTITY,
            ToolStatus::ValidationRejected,
            FailureClass::ValidationRejected,
            format!("{field} must contain {min}-{max} refs"),
        ));
    }
    let mut out = Vec::new();
    for value in values {
        out.push(text(&value, field, 512)?)
    }
    out.sort();
    out.dedup();
    Ok(out)
}

fn evidence_status(value: &str) -> Result<WorkspaceArtifactEvidenceStatus, ApiError> {
    match text(value, "evidence_status", 64)?.as_str() {
        "proposal_only" => Ok(WorkspaceArtifactEvidenceStatus::ProposalOnly),
        "capture_pending" => Ok(WorkspaceArtifactEvidenceStatus::CapturePending),
        "captured" => Ok(WorkspaceArtifactEvidenceStatus::Captured),
        "linked" => Ok(WorkspaceArtifactEvidenceStatus::Linked),
        "verified" => Ok(WorkspaceArtifactEvidenceStatus::Verified),
        "stale" => Ok(WorkspaceArtifactEvidenceStatus::Stale),
        "blocked" => Ok(WorkspaceArtifactEvidenceStatus::Blocked),
        "scope_mismatch" => Ok(WorkspaceArtifactEvidenceStatus::ScopeMismatch),
        _ => Err(fail(
            StatusCode::UNPROCESSABLE_ENTITY,
            ToolStatus::ValidationRejected,
            FailureClass::ValidationRejected,
            "unsupported evidence_status",
        )),
    }
}

fn response(
    artifact: WorkspaceArtifactRecord,
    version: u64,
    replayed: bool,
) -> WorkspaceArtifactIntakeResponse {
    let evidence_ref = artifact
        .evidence_refs
        .first()
        .cloned()
        .unwrap_or_else(|| stable("evidence:workspace-artifact", &[&artifact.artifact_id]));
    let receipt_ref = stable(
        "receipt:workspace-artifact",
        &[&artifact.artifact_id, &artifact.idempotency_key],
    );
    let mut tool = ToolResultV1::success(
        if replayed {
            ToolStatus::NoOp
        } else {
            ToolStatus::Completed
        },
        if replayed {
            "Workspace Artifact link replayed idempotently"
        } else {
            "Bounded Workspace Artifact descriptor linked; UIAI runtime and blob authority remain external"
        },
    );
    tool.tool = Some("focusa_workspace_artifact_intake".into());
    tool.family = Some("workspace_artifact".into());
    tool.endpoint = Some("/v1/workspace/artifacts/intake".into());
    tool.evidence_refs = vec![evidence_ref.clone()];
    tool.side_effects = if replayed {
        vec![]
    } else {
        vec!["workspace_artifact_descriptor_linked".into()]
    };
    tool.next_tools = vec![
        "focusa_workspace_artifacts_list".into(),
        "focusa_evidence_capture".into(),
    ];
    WorkspaceArtifactIntakeResponse {
        schema: "focusa.workspace_artifact_intake_result.v1",
        canonical_link: true,
        external_artifact_authority: true,
        replayed,
        state_version: version,
        artifact,
        evidence_ref,
        receipt_ref,
        tool_result: tool,
    }
}

async fn list(
    State(state): State<Arc<AppState>>,
    Query(query): Query<WorkspaceArtifactQuery>,
) -> Result<Json<WorkspaceArtifactListResponse>, ApiError> {
    let query = WorkspaceArtifactQuery {
        project_root: text(&query.project_root, "project_root", 4096)?,
        continuity_id: text(&query.continuity_id, "continuity_id", 256)?,
        attachment_id: text(&query.attachment_id, "attachment_id", 256)?,
    };
    let state = state.focusa.read().await;
    let artifacts = state
        .workspace_artifacts
        .iter()
        .filter(|artifact| scoped(artifact, &query))
        .cloned()
        .collect();
    Ok(Json(WorkspaceArtifactListResponse {
        schema: "focusa.workspace_artifact_list.v1",
        canonical_links: true,
        external_artifact_authority: true,
        state_version: state.version,
        artifacts,
    }))
}

async fn intake(
    State(state): State<Arc<AppState>>,
    Json(request): Json<WorkspaceArtifactIntakeRequest>,
) -> Result<Json<WorkspaceArtifactIntakeResponse>, ApiError> {
    let project_root = text(&request.project_root, "project_root", 4096)?;
    let continuity_id = text(&request.continuity_id, "continuity_id", 256)?;
    let attachment_id = text(&request.attachment_id, "attachment_id", 256)?;
    let idempotency_key = text(&request.idempotency_key, "idempotency_key", 256)?;
    let artifact_kind = text(&request.artifact_kind, "artifact_kind", 64)?;
    if !matches!(
        artifact_kind.as_str(),
        "image"
            | "markdown"
            | "dataset"
            | "diff"
            | "browser_snapshot"
            | "diagnostics"
            | "chart"
            | "document"
            | "media"
            | "fpv_session"
    ) {
        return Err(fail(
            StatusCode::UNPROCESSABLE_ENTITY,
            ToolStatus::ValidationRejected,
            FailureClass::ValidationRejected,
            "unsupported artifact_kind",
        ));
    }
    let source_system = text(&request.source_system, "source_system", 64)?;
    if !matches!(
        source_system.as_str(),
        "uiai" | "focusa" | "local_file" | "connector" | "provider" | "operator"
    ) {
        return Err(fail(
            StatusCode::UNPROCESSABLE_ENTITY,
            ToolStatus::ValidationRejected,
            FailureClass::ValidationRejected,
            "unsupported source_system",
        ));
    }
    let uiai_session_id = optional(request.uiai_session_id, "uiai_session_id", 256)?;
    if source_system == "uiai" && uiai_session_id.is_none() {
        return Err(fail(
            StatusCode::UNPROCESSABLE_ENTITY,
            ToolStatus::ValidationRejected,
            FailureClass::ValidationRejected,
            "uiai source requires uiai_session_id origin",
        ));
    }
    let sha256 = text(&request.sha256, "sha256", 128)?;
    if sha256.len() != 64 || !sha256.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err(fail(
            StatusCode::UNPROCESSABLE_ENTITY,
            ToolStatus::ValidationRejected,
            FailureClass::ValidationRejected,
            "sha256 must be 64 hexadecimal characters",
        ));
    }
    let inline_preview = optional(request.inline_preview, "inline_preview", 2000)?;
    let diagnostics_refs = refs(request.diagnostics_refs, "diagnostics_refs", 0, 32)?;
    if artifact_kind == "diagnostics" && diagnostics_refs.is_empty() {
        return Err(fail(
            StatusCode::UNPROCESSABLE_ENTITY,
            ToolStatus::ValidationRejected,
            FailureClass::ValidationRejected,
            "diagnostics artifacts require at least one diagnostics_ref",
        ));
    }
    let evidence_refs = refs(request.evidence_refs, "evidence_refs", 1, 32)?;
    let evidence_status = evidence_status(&request.evidence_status)?;
    if !matches!(
        evidence_status,
        WorkspaceArtifactEvidenceStatus::Linked | WorkspaceArtifactEvidenceStatus::Verified
    ) {
        return Err(fail(
            StatusCode::UNPROCESSABLE_ENTITY,
            ToolStatus::ValidationRejected,
            FailureClass::ValidationRejected,
            "canonical artifact intake requires linked or verified Evidence",
        ));
    }
    let semantic = WorkspaceArtifactSemantic {
        domain_pack_refs: refs(request.domain_pack_refs, "domain_pack_refs", 0, 32)?,
        candidate_object_refs: refs(
            request.candidate_object_refs,
            "candidate_object_refs",
            0,
            32,
        )?,
        candidate_link_refs: refs(request.candidate_link_refs, "candidate_link_refs", 0, 32)?,
        candidate_claim_refs: refs(request.candidate_claim_refs, "candidate_claim_refs", 0, 32)?,
        verification_policy_refs: refs(
            request.verification_policy_refs,
            "verification_policy_refs",
            0,
            32,
        )?,
        semantic_delta_refs: refs(request.semantic_delta_refs, "semantic_delta_refs", 0, 32)?,
        citation_refs: refs(request.citation_refs, "citation_refs", 0, 64)?,
    };
    if matches!(artifact_kind.as_str(), "markdown" | "document")
        && semantic.citation_refs.is_empty()
    {
        return Err(fail(
            StatusCode::UNPROCESSABLE_ENTITY,
            ToolStatus::ValidationRejected,
            FailureClass::ValidationRejected,
            "research and document artifacts require a source-preserving citation_ref",
        ));
    }
    if request.render_width == Some(0)
        || request.render_height == Some(0)
        || request.render_width.is_some_and(|value| value > 16_384)
        || request.render_height.is_some_and(|value| value > 16_384)
    {
        return Err(fail(
            StatusCode::UNPROCESSABLE_ENTITY,
            ToolStatus::ValidationRejected,
            FailureClass::ValidationRejected,
            "render dimensions must be within 1-16384 pixels",
        ));
    }
    let artifact_id = stable(
        "workspace-artifact",
        &[
            &project_root,
            &continuity_id,
            &attachment_id,
            &source_system,
            &request.source_ref,
            &sha256,
        ],
    );
    let now = Utc::now();
    let mut artifact = WorkspaceArtifactRecord {
        artifact_id: artifact_id.clone(),
        artifact_kind,
        mime_type: text(&request.mime_type, "mime_type", 256)?,
        title: text(&request.title, "title", 512)?,
        summary: text(&request.summary, "summary", 2000)?,
        content: WorkspaceArtifactContent {
            handle_ref: text(&request.handle_ref, "handle_ref", 2048)?,
            artifact_url: optional(request.artifact_url, "artifact_url", 4096)?,
            artifact_path: optional(request.artifact_path, "artifact_path", 4096)?,
            inline_preview,
            sha256,
            size_bytes: request.size_bytes,
        },
        source: WorkspaceArtifactSource {
            system: source_system,
            source_ref: text(&request.source_ref, "source_ref", 2048)?,
            source_url: optional(request.source_url, "source_url", 4096)?,
            captured_at: request.captured_at.unwrap_or(now),
        },
        scope: WorkspaceArtifactScope {
            project_root: project_root.clone(),
            continuity_id: continuity_id.clone(),
            project_identity_ref: optional(
                request.project_identity_ref,
                "project_identity_ref",
                512,
            )?,
            workpoint_id: optional(request.workpoint_id, "workpoint_id", 256)?,
            work_item_ref: optional(request.work_item_ref, "work_item_ref", 256)?,
        },
        origin: WorkspaceArtifactOrigin {
            instance_id: text(&request.instance_id, "instance_id", 256)?,
            attachment_id: attachment_id.clone(),
            focusa_session_id: optional(request.focusa_session_id, "focusa_session_id", 512)?,
            work_surface_id: optional(request.work_surface_id, "work_surface_id", 256)?,
            harness_session_ref: optional(request.harness_session_ref, "harness_session_ref", 512)?,
            silent_session_id: optional(request.silent_session_id, "silent_session_id", 256)?,
            silent_run_id: optional(request.silent_run_id, "silent_run_id", 256)?,
            uiai_session_id,
            browser_context_id: optional(request.browser_context_id, "browser_context_id", 256)?,
            browser_target_id: optional(request.browser_target_id, "browser_target_id", 256)?,
        },
        trust: WorkspaceArtifactTrust {
            evidence_status,
            redaction_status: text(&request.redaction_status, "redaction_status", 64)?,
            freshness_status: text(&request.freshness_status, "freshness_status", 64)?,
            provenance_status: text(&request.provenance_status, "provenance_status", 64)?,
        },
        semantic,
        diagnostics_refs,
        evidence_refs,
        retention: WorkspaceArtifactRetention {
            policy: text(&request.retention_policy, "retention_policy", 128)?,
            expires_at: request.expires_at,
            cleanup_action: text(&request.cleanup_action, "cleanup_action", 512)?,
        },
        render: WorkspaceArtifactRender {
            preferred_renderer: text(&request.preferred_renderer, "preferred_renderer", 128)?,
            fallback_renderer: text(&request.fallback_renderer, "fallback_renderer", 128)?,
            width: request.render_width,
            height: request.render_height,
        },
        idempotency_key: idempotency_key.clone(),
        revision: 1,
        linked_at: now,
        updated_at: now,
    };
    let _writer = state.write_serial_lock.lock().await;
    let snapshot = state.focusa.read().await.clone();
    if let Some(existing) = snapshot.workspace_artifacts.iter().find(|item| {
        scoped(
            item,
            &WorkspaceArtifactQuery {
                project_root: project_root.clone(),
                continuity_id: continuity_id.clone(),
                attachment_id: attachment_id.clone(),
            },
        ) && item.idempotency_key == idempotency_key
    }) {
        return Ok(Json(response(existing.clone(), snapshot.version, true)));
    }
    if snapshot.version != request.expected_state_version {
        return Err(fail(
            StatusCode::CONFLICT,
            ToolStatus::Blocked,
            FailureClass::WriterConflict,
            format!(
                "expected_state_version={} does not match canonical version={}",
                request.expected_state_version, snapshot.version
            ),
        ));
    }
    if let Some(existing) = snapshot
        .workspace_artifacts
        .iter()
        .find(|item| item.artifact_id == artifact_id)
    {
        artifact.revision = existing.revision + 1;
        artifact.linked_at = existing.linked_at;
    }
    drop(_writer);
    state
        .command_tx
        .send(Action::EmitEvent {
            event: FocusaEvent::WorkspaceArtifactLinked {
                artifact: artifact.clone(),
            },
        })
        .await
        .map_err(|_| {
            fail(
                StatusCode::SERVICE_UNAVAILABLE,
                ToolStatus::Offline,
                FailureClass::DaemonUnavailable,
                "canonical artifact command channel unavailable",
            )
        })?;
    for _ in 0..100 {
        let current = state.focusa.read().await;
        if let Some(linked) = current
            .workspace_artifacts
            .iter()
            .find(|item| item.artifact_id == artifact_id && item.idempotency_key == idempotency_key)
        {
            return Ok(Json(response(linked.clone(), current.version, false)));
        }
        drop(current);
        tokio::time::sleep(Duration::from_millis(20)).await
    }
    Err(fail(
        StatusCode::SERVICE_UNAVAILABLE,
        ToolStatus::Degraded,
        FailureClass::ReadModelLag,
        "Workspace Artifact link dispatched but not visible",
    ))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/workspace/artifacts", get(list))
        .route("/v1/workspace/artifacts/intake", post(intake))
}
