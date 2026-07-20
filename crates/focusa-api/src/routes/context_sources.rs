use crate::server::AppState;
use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
};
use chrono::Utc;
use focusa_core::{
    tool_result::{FailureClass, ToolResultV1, ToolStatus},
    types::{
        Action, ContextSourceEvidence, ContextSourceReceipt, ContextSourceRecord, FocusaEvent,
    },
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{sync::Arc, time::Duration};

const COMMIT_OPERATION_ID: &str = "focusa.context.source.commit";
const MAX_CONTEXT_BYTES: usize = 64 * 1024;

type ApiError = (StatusCode, Json<ToolResultV1>);

#[derive(Debug, Deserialize)]
pub struct ContextSourceCommitRequest {
    pub project_root: String,
    pub continuity_id: String,
    pub attachment_id: String,
    pub idempotency_key: String,
    pub expected_state_version: u64,
    pub source_kind: String,
    pub title: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct ContextSourceCommitResponse {
    pub schema: &'static str,
    pub canonical: bool,
    pub replayed: bool,
    pub state_version: u64,
    pub source: ContextSourceRecord,
    pub evidence_ref: String,
    pub receipt_ref: String,
    pub tool_result: ToolResultV1,
}

#[derive(Debug, Deserialize)]
pub struct ContextSourceListQuery {
    pub project_root: String,
    pub continuity_id: String,
    pub attachment_id: String,
}

#[derive(Debug, Serialize)]
pub struct ContextSourceListResponse {
    pub schema: &'static str,
    pub canonical: bool,
    pub state_version: u64,
    pub sources: Vec<ContextSourceRecord>,
}

fn failure(
    status: StatusCode,
    tool_status: ToolStatus,
    class: FailureClass,
    summary: impl Into<String>,
) -> ApiError {
    let mut result = ToolResultV1::failure(tool_status, class, summary);
    result.tool = Some("focusa_context_source_commit".to_string());
    result.family = Some("context".to_string());
    result.endpoint = Some("/v1/context/sources/commit".to_string());
    result.next_tools = vec![
        "focusa_context_sources_list".to_string(),
        "focusa_active_object_resolve".to_string(),
    ];
    (status, Json(result))
}

fn validate_nonempty(value: &str, field: &str, max: usize) -> Result<String, ApiError> {
    let value = value.trim();
    if value.is_empty() || value.len() > max {
        return Err(failure(
            StatusCode::UNPROCESSABLE_ENTITY,
            ToolStatus::ValidationRejected,
            FailureClass::ValidationRejected,
            format!("{field} must contain 1-{max} characters"),
        ));
    }
    Ok(value.to_string())
}

fn digest(parts: &[&str]) -> String {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update((part.len() as u64).to_be_bytes());
        hasher.update(part.as_bytes());
    }
    hex::encode(hasher.finalize())
}

fn success_response(
    source: ContextSourceRecord,
    state_version: u64,
    replayed: bool,
) -> ContextSourceCommitResponse {
    let status = if replayed {
        ToolStatus::NoOp
    } else {
        ToolStatus::Completed
    };
    let mut tool_result = ToolResultV1::success(
        status,
        if replayed {
            "Context source commit replayed from its idempotency key"
        } else {
            "Context source committed to canonical reducer state"
        },
    );
    tool_result.tool = Some("focusa_context_source_commit".to_string());
    tool_result.family = Some("context".to_string());
    tool_result.endpoint = Some("/v1/context/sources/commit".to_string());
    tool_result.side_effects = if replayed {
        vec![]
    } else {
        vec!["canonical_context_source_committed".to_string()]
    };
    tool_result.evidence_refs = vec![source.evidence.evidence_ref.clone()];
    tool_result.next_tools = vec![
        "focusa_context_sources_list".to_string(),
        "focusa_evidence_capture".to_string(),
    ];
    ContextSourceCommitResponse {
        schema: "focusa.context_source_commit_result.v1",
        canonical: true,
        replayed,
        state_version,
        evidence_ref: source.evidence.evidence_ref.clone(),
        receipt_ref: source.receipt.receipt_ref.clone(),
        source,
        tool_result,
    }
}

async fn commit(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ContextSourceCommitRequest>,
) -> Result<Json<ContextSourceCommitResponse>, ApiError> {
    let project_root = validate_nonempty(&body.project_root, "project_root", 4096)?;
    let continuity_id = validate_nonempty(&body.continuity_id, "continuity_id", 256)?;
    let attachment_id = validate_nonempty(&body.attachment_id, "attachment_id", 256)?;
    let idempotency_key = validate_nonempty(&body.idempotency_key, "idempotency_key", 160)?;
    let title = validate_nonempty(&body.title, "title", 240)?;
    let source_kind = validate_nonempty(&body.source_kind, "source_kind", 32)?;
    if !matches!(source_kind.as_str(), "markdown" | "text" | "code" | "pdf") {
        return Err(failure(
            StatusCode::UNPROCESSABLE_ENTITY,
            ToolStatus::ValidationRejected,
            FailureClass::SchemaInvalid,
            "source_kind must be markdown, text, code, or pdf",
        ));
    }
    if body.content.trim().is_empty() || body.content.len() > MAX_CONTEXT_BYTES {
        return Err(failure(
            StatusCode::UNPROCESSABLE_ENTITY,
            ToolStatus::ValidationRejected,
            FailureClass::ResourceExhausted,
            format!("content must contain 1-{MAX_CONTEXT_BYTES} bytes"),
        ));
    }

    let before_state_version;
    {
        let focusa = state.focusa.read().await;
        if let Some(existing) = focusa.context_sources.iter().find(|source| {
            source.project_root == project_root
                && source.continuity_id == continuity_id
                && source.attachment_id == attachment_id
                && source.idempotency_key == idempotency_key
        }) {
            return Ok(Json(success_response(
                existing.clone(),
                focusa.version,
                true,
            )));
        }
        if focusa.version != body.expected_state_version {
            return Err(failure(
                StatusCode::CONFLICT,
                ToolStatus::Blocked,
                FailureClass::WriterConflict,
                format!(
                    "expected_state_version={} does not match canonical version={}",
                    body.expected_state_version, focusa.version
                ),
            ));
        }
        before_state_version = focusa.version;
    }

    let content_hash = digest(&[&body.content]);
    let scope_hash = digest(&[
        &project_root,
        &continuity_id,
        &attachment_id,
        &idempotency_key,
    ]);
    let source_id = format!("context-source:{}", &scope_hash[..24]);
    let evidence_ref = format!("evidence:context-source:{}", &scope_hash[..24]);
    let receipt_ref = format!("receipt:context-source:{}", &scope_hash[..24]);
    let committed_at = Utc::now();
    let source = ContextSourceRecord {
        source_id: source_id.clone(),
        project_root,
        continuity_id,
        attachment_id,
        source_kind,
        title,
        content: body.content,
        content_hash: content_hash.clone(),
        idempotency_key: idempotency_key.clone(),
        revision: 1,
        committed_at,
        evidence: ContextSourceEvidence {
            evidence_ref,
            target_ref: source_id.clone(),
            result: "Context source content committed and hash verified".to_string(),
            content_hash,
            captured_at: committed_at,
        },
        receipt: ContextSourceReceipt {
            receipt_ref,
            operation_id: COMMIT_OPERATION_ID.to_string(),
            idempotency_key,
            before_state_version,
            after_state_version: before_state_version + 1,
            reversible: true,
            committed_at,
        },
    };

    state
        .command_tx
        .send(Action::EmitEvent {
            event: FocusaEvent::ContextSourceCommitted {
                source: source.clone(),
            },
        })
        .await
        .map_err(|_| {
            failure(
                StatusCode::SERVICE_UNAVAILABLE,
                ToolStatus::Offline,
                FailureClass::DaemonUnavailable,
                "canonical Context command channel is unavailable",
            )
        })?;

    for _ in 0..100 {
        {
            let focusa = state.focusa.read().await;
            if let Some(committed) = focusa
                .context_sources
                .iter()
                .find(|record| record.source_id == source_id)
            {
                return Ok(Json(success_response(
                    committed.clone(),
                    focusa.version,
                    false,
                )));
            }
            if focusa.version > before_state_version {
                return Err(failure(
                    StatusCode::CONFLICT,
                    ToolStatus::Blocked,
                    FailureClass::WriterConflict,
                    "canonical state advanced before the Context commit reduced; retry with the same idempotency key and fresh version",
                ));
            }
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }

    Err(failure(
        StatusCode::SERVICE_UNAVAILABLE,
        ToolStatus::Degraded,
        FailureClass::ReadModelLag,
        "Context source commit was dispatched but is not yet visible in canonical state",
    ))
}

async fn list(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ContextSourceListQuery>,
) -> Result<Json<ContextSourceListResponse>, ApiError> {
    let project_root = validate_nonempty(&query.project_root, "project_root", 4096)?;
    let continuity_id = validate_nonempty(&query.continuity_id, "continuity_id", 256)?;
    let attachment_id = validate_nonempty(&query.attachment_id, "attachment_id", 256)?;
    let focusa = state.focusa.read().await;
    let sources = focusa
        .context_sources
        .iter()
        .filter(|source| {
            source.project_root == project_root
                && source.continuity_id == continuity_id
                && source.attachment_id == attachment_id
        })
        .cloned()
        .collect();
    Ok(Json(ContextSourceListResponse {
        schema: "focusa.context_source_list.v1",
        canonical: true,
        state_version: focusa.version,
        sources,
    }))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/context/sources/commit", post(commit))
        .route("/v1/context/sources", get(list))
}
