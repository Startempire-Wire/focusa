use crate::server::AppState;
use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use chrono::Utc;
use focusa_core::{
    runtime::context_retrieval::{
        ContextRetrievalIndex, ContextRetrievalMode, ContextRetrievalQuery, ContextRetrievalResult,
    },
    tool_result::{FailureClass, ToolResultV1, ToolStatus},
    types::{
        Action, ContextSourceEvidence, ContextSourceHealth, ContextSourceReceipt,
        ContextSourceRecord, FocusaEvent,
    },
};
use reqwest::multipart::{Form, Part};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{env, sync::Arc, time::Duration};

const COMMIT_OPERATION_ID: &str = "focusa.context.source.commit";
const INGEST_OPERATION_ID: &str = "focusa.context.source.ingest";
const RETRIEVE_OPERATION_ID: &str = "focusa.context.retrieve";
const MAX_CONTEXT_BYTES: usize = 64 * 1024;
const MAX_TEXT_INGEST_BYTES: usize = 2 * 1024 * 1024;
const MAX_PDF_INGEST_BYTES: usize = 20 * 1024 * 1024;

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

#[derive(Debug, Deserialize)]
pub struct ContextRetrieveRequest {
    pub project_root: String,
    pub continuity_id: String,
    #[serde(default)]
    pub attachment_id: Option<String>,
    pub query: String,
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default)]
    pub mode: ContextRetrievalMode,
    #[serde(default)]
    pub include_contradictions: bool,
}

#[derive(Debug, Serialize)]
pub struct ContextRetrieveResponse {
    pub schema: &'static str,
    pub canonical_sources: bool,
    pub result: ContextRetrievalResult,
    pub evidence_ref: String,
    pub receipt_ref: String,
    pub tool_result: ToolResultV1,
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
        source_locator: String::new(),
        source_revision: String::new(),
        mime_type: String::new(),
        adapter_id: "focusa.context.commit".to_string(),
        ingestion_status: "committed".to_string(),
        extraction_diagnostics: Vec::new(),
        health: Default::default(),
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

#[derive(Debug, Deserialize)]
pub struct ContextSourceIngestRequest {
    pub project_root: String,
    pub continuity_id: String,
    pub attachment_id: String,
    pub idempotency_key: String,
    pub expected_state_version: u64,
    pub source_kind: String,
    pub source_locator: String,
    pub source_revision: String,
    pub title: String,
    pub mime_type: String,
    pub content: Option<String>,
    pub content_base64: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ContextSourceIngestResponse {
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
struct DoclingDocument {
    #[serde(default)]
    md_content: String,
}

#[derive(Debug, Deserialize)]
struct DoclingConversionResponse {
    document: DoclingDocument,
    status: String,
    #[serde(default)]
    processing_time: f64,
    #[serde(default)]
    errors: Vec<Value>,
}

#[derive(Debug, Serialize)]
pub struct DoclingHealthResponse {
    pub schema: &'static str,
    pub adapter_id: &'static str,
    pub configured: bool,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recovery_action: Option<String>,
    pub checked_at: chrono::DateTime<Utc>,
}

fn ingest_failure(
    status: StatusCode,
    tool_status: ToolStatus,
    class: FailureClass,
    summary: impl Into<String>,
) -> ApiError {
    let mut result = ToolResultV1::failure(tool_status, class, summary);
    result.tool = Some("focusa_context_source_ingest".to_string());
    result.family = Some("context".to_string());
    result.endpoint = Some("/v1/context/sources/ingest".to_string());
    result.next_tools = vec![
        "focusa_context_adapter_docling_health".to_string(),
        "focusa_context_sources_list".to_string(),
    ];
    (status, Json(result))
}

fn ingest_success_response(
    source: ContextSourceRecord,
    state_version: u64,
    replayed: bool,
) -> ContextSourceIngestResponse {
    let mut tool_result = ToolResultV1::success(
        if replayed {
            ToolStatus::NoOp
        } else {
            ToolStatus::Completed
        },
        if replayed {
            "Context ingestion replayed from canonical idempotent state"
        } else {
            "Context source ingested, normalized, and committed to canonical reducer state"
        },
    );
    tool_result.tool = Some("focusa_context_source_ingest".to_string());
    tool_result.family = Some("context".to_string());
    tool_result.endpoint = Some("/v1/context/sources/ingest".to_string());
    tool_result.side_effects = if replayed {
        vec![]
    } else {
        vec!["canonical_context_source_ingested".to_string()]
    };
    tool_result.evidence_refs = vec![source.evidence.evidence_ref.clone()];
    tool_result.next_tools = vec![
        "focusa_context_sources_list".to_string(),
        "focusa_evidence_capture".to_string(),
    ];
    ContextSourceIngestResponse {
        schema: "focusa.context_source_ingest_result.v1",
        canonical: true,
        replayed,
        state_version,
        evidence_ref: source.evidence.evidence_ref.clone(),
        receipt_ref: source.receipt.receipt_ref.clone(),
        source,
        tool_result,
    }
}

fn docling_base_url() -> Option<String> {
    env::var("FOCUSA_DOCLING_BASE_URL")
        .ok()
        .map(|value| value.trim().trim_end_matches('/').to_string())
        .filter(|value| value.starts_with("http://") || value.starts_with("https://"))
}

async fn extract_pdf(
    bytes: Vec<u8>,
    source_locator: &str,
) -> Result<(String, Vec<String>), ApiError> {
    let base_url = docling_base_url().ok_or_else(|| {
        ingest_failure(
            StatusCode::SERVICE_UNAVAILABLE,
            ToolStatus::Offline,
            FailureClass::DaemonUnavailable,
            "Docling Serve v1 is not configured; set FOCUSA_DOCLING_BASE_URL and retry",
        )
    })?;
    let part = Part::bytes(bytes)
        .file_name(source_locator.to_string())
        .mime_str("application/pdf")
        .map_err(|error| {
            ingest_failure(
                StatusCode::UNPROCESSABLE_ENTITY,
                ToolStatus::ValidationRejected,
                FailureClass::ValidationRejected,
                format!("invalid PDF multipart payload: {error}"),
            )
        })?;
    let form = Form::new()
        .part("files", part)
        .text("from_formats", "pdf")
        .text("to_formats", "md")
        .text("image_export_mode", "placeholder")
        .text("abort_on_error", "false")
        .text("do_ocr", "false");
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(180))
        .build()
        .map_err(|error| {
            ingest_failure(
                StatusCode::SERVICE_UNAVAILABLE,
                ToolStatus::Offline,
                FailureClass::DaemonUnavailable,
                format!("Docling client initialization failed: {error}"),
            )
        })?;
    let response = client
        .post(format!("{base_url}/v1/convert/file"))
        .multipart(form)
        .send()
        .await
        .map_err(|error| {
            ingest_failure(
                StatusCode::BAD_GATEWAY,
                ToolStatus::Degraded,
                FailureClass::DaemonUnavailable,
                format!("Docling conversion request failed: {error}"),
            )
        })?;
    let response_status = response.status();
    if !response_status.is_success() {
        let detail = response.text().await.unwrap_or_default();
        return Err(ingest_failure(
            StatusCode::BAD_GATEWAY,
            ToolStatus::Degraded,
            FailureClass::NullResponse,
            format!(
                "Docling conversion returned {response_status}: {}",
                detail.chars().take(300).collect::<String>()
            ),
        ));
    }
    let converted: DoclingConversionResponse = response.json().await.map_err(|error| {
        ingest_failure(
            StatusCode::BAD_GATEWAY,
            ToolStatus::Degraded,
            FailureClass::SchemaInvalid,
            format!("Docling response was not the v1 conversion schema: {error}"),
        )
    })?;
    if !matches!(converted.status.as_str(), "success" | "partial_success")
        || converted.document.md_content.trim().is_empty()
    {
        return Err(ingest_failure(
            StatusCode::UNPROCESSABLE_ENTITY,
            ToolStatus::Degraded,
            FailureClass::NullResponse,
            format!(
                "Docling extraction status={} errors={}",
                converted.status,
                serde_json::to_string(&converted.errors).unwrap_or_default()
            ),
        ));
    }
    let diagnostics = vec![
        format!("docling_status={}", converted.status),
        format!(
            "docling_processing_seconds={:.3}",
            converted.processing_time
        ),
        format!("docling_error_count={}", converted.errors.len()),
    ];
    Ok((converted.document.md_content, diagnostics))
}

async fn ingest(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ContextSourceIngestRequest>,
) -> Result<Json<ContextSourceIngestResponse>, ApiError> {
    let project_root = validate_nonempty(&body.project_root, "project_root", 4096)?;
    let continuity_id = validate_nonempty(&body.continuity_id, "continuity_id", 256)?;
    let attachment_id = validate_nonempty(&body.attachment_id, "attachment_id", 256)?;
    let idempotency_key = validate_nonempty(&body.idempotency_key, "idempotency_key", 160)?;
    let source_locator = validate_nonempty(&body.source_locator, "source_locator", 1024)?;
    let source_revision = validate_nonempty(&body.source_revision, "source_revision", 256)?;
    let title = validate_nonempty(&body.title, "title", 240)?;
    let mime_type = validate_nonempty(&body.mime_type, "mime_type", 128)?;
    let source_kind = validate_nonempty(&body.source_kind, "source_kind", 32)?.to_lowercase();
    if !matches!(source_kind.as_str(), "markdown" | "code" | "pdf") {
        return Err(ingest_failure(
            StatusCode::UNPROCESSABLE_ENTITY,
            ToolStatus::ValidationRejected,
            FailureClass::ValidationRejected,
            "source_kind must be markdown, code, or pdf",
        ));
    }

    let stable_hash = digest(&[
        &project_root,
        &continuity_id,
        &attachment_id,
        &source_locator,
    ]);
    let source_id = format!("context-source:{}", &stable_hash[..24]);
    let (before_state_version, existing_revision) = {
        let focusa = state.focusa.read().await;
        if let Some(existing) = focusa.context_sources.iter().find(|source| {
            source.project_root == project_root
                && source.continuity_id == continuity_id
                && source.attachment_id == attachment_id
                && source.idempotency_key == idempotency_key
        }) {
            return Ok(Json(ingest_success_response(
                existing.clone(),
                focusa.version,
                true,
            )));
        }
        if body.expected_state_version != focusa.version {
            return Err(ingest_failure(
                StatusCode::CONFLICT,
                ToolStatus::Blocked,
                FailureClass::WriterConflict,
                format!(
                    "expected_state_version={} does not match canonical version={}",
                    body.expected_state_version, focusa.version
                ),
            ));
        }
        (
            focusa.version,
            focusa
                .context_sources
                .iter()
                .find(|source| source.source_id == source_id)
                .map(|source| source.revision)
                .unwrap_or(0),
        )
    };

    let (content, adapter_id, diagnostics) = match source_kind.as_str() {
        "markdown" | "code" => {
            let content = body.content.ok_or_else(|| {
                ingest_failure(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ToolStatus::ValidationRejected,
                    FailureClass::ValidationRejected,
                    "content is required for Markdown and code ingestion",
                )
            })?;
            if content.is_empty() || content.len() > MAX_TEXT_INGEST_BYTES {
                return Err(ingest_failure(
                    StatusCode::PAYLOAD_TOO_LARGE,
                    ToolStatus::ValidationRejected,
                    FailureClass::ResourceExhausted,
                    format!("text ingestion requires 1-{MAX_TEXT_INGEST_BYTES} bytes"),
                ));
            }
            if source_kind == "markdown" && mime_type != "text/markdown" {
                return Err(ingest_failure(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ToolStatus::ValidationRejected,
                    FailureClass::ValidationRejected,
                    "Markdown ingestion requires mime_type=text/markdown",
                ));
            }
            (
                content,
                "focusa.local_text.v1".to_string(),
                vec!["native_utf8_extraction=success".to_string()],
            )
        }
        "pdf" => {
            if mime_type != "application/pdf" {
                return Err(ingest_failure(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ToolStatus::ValidationRejected,
                    FailureClass::ValidationRejected,
                    "PDF ingestion requires mime_type=application/pdf",
                ));
            }
            let encoded = body.content_base64.ok_or_else(|| {
                ingest_failure(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ToolStatus::ValidationRejected,
                    FailureClass::ValidationRejected,
                    "content_base64 is required for PDF ingestion",
                )
            })?;
            let bytes = BASE64.decode(encoded).map_err(|_| {
                ingest_failure(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ToolStatus::ValidationRejected,
                    FailureClass::ValidationRejected,
                    "content_base64 is not valid base64",
                )
            })?;
            if bytes.len() > MAX_PDF_INGEST_BYTES || !bytes.starts_with(b"%PDF-") {
                return Err(ingest_failure(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ToolStatus::ValidationRejected,
                    FailureClass::ValidationRejected,
                    format!("PDF must be valid and no larger than {MAX_PDF_INGEST_BYTES} bytes"),
                ));
            }
            let (content, diagnostics) = extract_pdf(bytes, &source_locator).await?;
            (content, "docling-serve.v1".to_string(), diagnostics)
        }
        _ => unreachable!(),
    };
    let content_hash = digest(&[&content]);

    {
        let focusa = state.focusa.read().await;
        if focusa.version != before_state_version {
            return Err(ingest_failure(
                StatusCode::CONFLICT,
                ToolStatus::Blocked,
                FailureClass::WriterConflict,
                "canonical state advanced during extraction; retry with the same idempotency key and fresh version",
            ));
        }
        if let Some(existing) = focusa
            .context_sources
            .iter()
            .find(|source| source.source_id == source_id)
            && existing.source_revision == source_revision
            && existing.content_hash == content_hash
        {
            return Ok(Json(ingest_success_response(
                existing.clone(),
                focusa.version,
                true,
            )));
        }
    }

    let committed_at = Utc::now();
    let revision = existing_revision + 1;
    let transition_hash = digest(&[
        &source_id,
        &source_revision,
        &content_hash,
        &idempotency_key,
    ]);
    let source = ContextSourceRecord {
        source_id: source_id.clone(),
        project_root,
        continuity_id,
        attachment_id,
        source_kind,
        title,
        content,
        content_hash: content_hash.clone(),
        idempotency_key: idempotency_key.clone(),
        revision,
        committed_at,
        evidence: ContextSourceEvidence {
            evidence_ref: format!("evidence:context-ingest:{}", &transition_hash[..24]),
            target_ref: source_id.clone(),
            result: "Context source ingested with source-preserving extraction and verified hash"
                .to_string(),
            content_hash,
            captured_at: committed_at,
        },
        receipt: ContextSourceReceipt {
            receipt_ref: format!("receipt:context-ingest:{}", &transition_hash[..24]),
            operation_id: INGEST_OPERATION_ID.to_string(),
            idempotency_key,
            before_state_version,
            after_state_version: before_state_version + 1,
            reversible: true,
            committed_at,
        },
        source_locator,
        source_revision,
        mime_type,
        adapter_id: adapter_id.clone(),
        ingestion_status: "completed".to_string(),
        extraction_diagnostics: diagnostics,
        health: ContextSourceHealth {
            status: "healthy".to_string(),
            adapter_id,
            message: "Source ingestion completed successfully".to_string(),
            recovery_action: None,
            last_successful_sync: Some(committed_at),
        },
    };
    state
        .command_tx
        .send(Action::EmitEvent {
            event: FocusaEvent::ContextSourceIngested {
                source: source.clone(),
            },
        })
        .await
        .map_err(|_| {
            ingest_failure(
                StatusCode::SERVICE_UNAVAILABLE,
                ToolStatus::Offline,
                FailureClass::DaemonUnavailable,
                "canonical Context command channel is unavailable",
            )
        })?;

    for _ in 0..100 {
        {
            let focusa = state.focusa.read().await;
            if let Some(ingested) = focusa
                .context_sources
                .iter()
                .find(|record| record.source_id == source_id && record.revision == revision)
            {
                return Ok(Json(ingest_success_response(
                    ingested.clone(),
                    focusa.version,
                    false,
                )));
            }
            if focusa.version > before_state_version {
                return Err(ingest_failure(
                    StatusCode::CONFLICT,
                    ToolStatus::Blocked,
                    FailureClass::WriterConflict,
                    "canonical state advanced before Context ingestion reduced; retry idempotently",
                ));
            }
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    Err(ingest_failure(
        StatusCode::SERVICE_UNAVAILABLE,
        ToolStatus::Degraded,
        FailureClass::ReadModelLag,
        "Context ingestion dispatched but is not visible in canonical state",
    ))
}

fn retrieval_failure(
    status: StatusCode,
    tool_status: ToolStatus,
    class: FailureClass,
    summary: impl Into<String>,
) -> ApiError {
    let mut result = ToolResultV1::failure(tool_status, class, summary);
    result.tool = Some("focusa_context_retrieve".to_string());
    result.family = Some("context".to_string());
    result.endpoint = Some("/v1/context/retrieve".to_string());
    result.next_tools = vec![
        "focusa_context_sources_list".to_string(),
        "focusa_context_adapter_docling_health".to_string(),
    ];
    (status, Json(result))
}

async fn retrieve(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ContextRetrieveRequest>,
) -> Result<Json<ContextRetrieveResponse>, ApiError> {
    let project_root = validate_nonempty(&request.project_root, "project_root", 4096)?;
    let continuity_id = validate_nonempty(&request.continuity_id, "continuity_id", 256)?;
    let attachment_id = request
        .attachment_id
        .as_deref()
        .map(|value| validate_nonempty(value, "attachment_id", 256))
        .transpose()?;
    let query = validate_nonempty(&request.query, "query", 2048)?;
    let limit = request.limit.unwrap_or(8);
    if !(1..=50).contains(&limit) {
        return Err(retrieval_failure(
            StatusCode::UNPROCESSABLE_ENTITY,
            ToolStatus::ValidationRejected,
            FailureClass::ValidationRejected,
            "limit must be between 1 and 50",
        ));
    }

    let sources = {
        let focusa = state.focusa.read().await;
        focusa
            .context_sources
            .iter()
            .filter(|source| {
                source.project_root == project_root
                    && source.continuity_id == continuity_id
                    && attachment_id
                        .as_ref()
                        .map(|attachment| source.attachment_id == *attachment)
                        .unwrap_or(true)
            })
            .cloned()
            .collect::<Vec<_>>()
    };
    let index = ContextRetrievalIndex::from_persistence(&state.persistence);
    let retrieval_query = ContextRetrievalQuery {
        project_root: project_root.clone(),
        continuity_id: continuity_id.clone(),
        attachment_id: attachment_id.clone(),
        query: query.clone(),
        limit,
        mode: request.mode,
        include_contradictions: request.include_contradictions,
    };
    let result = tokio::task::spawn_blocking(move || index.retrieve(&sources, retrieval_query))
        .await
        .map_err(|error| {
            retrieval_failure(
                StatusCode::INTERNAL_SERVER_ERROR,
                ToolStatus::Blocked,
                FailureClass::ResourceExhausted,
                format!("Context retrieval worker failed: {error}"),
            )
        })?
        .map_err(|error| {
            retrieval_failure(
                StatusCode::SERVICE_UNAVAILABLE,
                ToolStatus::Degraded,
                FailureClass::ReadModelLag,
                format!("Context retrieval is unavailable: {error}"),
            )
        })?;

    let result_digest = digest(&[
        RETRIEVE_OPERATION_ID,
        &project_root,
        &continuity_id,
        attachment_id.as_deref().unwrap_or(""),
        &query,
        &serde_json::to_string(&result.hits).unwrap_or_default(),
    ]);
    let evidence_ref = format!("evidence:context-retrieval:{}", &result_digest[..24]);
    let receipt_ref = format!("receipt:context-retrieval:{}", &result_digest[..24]);
    let mut tool_result = ToolResultV1::success(
        ToolStatus::Completed,
        format!(
            "Retrieved {} exact-scope Context chunks with source-preserving citations",
            result.result_count
        ),
    );
    tool_result.tool = Some("focusa_context_retrieve".to_string());
    tool_result.family = Some("context".to_string());
    tool_result.endpoint = Some("/v1/context/retrieve".to_string());
    tool_result.evidence_refs = vec![evidence_ref.clone()];
    tool_result.next_tools = vec![
        "focusa_context_sources_list".to_string(),
        "focusa_evidence_capture".to_string(),
    ];

    Ok(Json(ContextRetrieveResponse {
        schema: "focusa.context_retrieve_response.v1",
        canonical_sources: true,
        result,
        evidence_ref,
        receipt_ref,
        tool_result,
    }))
}

async fn docling_health() -> Json<DoclingHealthResponse> {
    let checked_at = Utc::now();
    let Some(base_url) = docling_base_url() else {
        return Json(DoclingHealthResponse {
            schema: "focusa.context_adapter_health.v1",
            adapter_id: "docling-serve.v1",
            configured: false,
            status: "offline".to_string(),
            endpoint: None,
            message: "Docling Serve v1 is not configured".to_string(),
            recovery_action: Some(
                "Set FOCUSA_DOCLING_BASE_URL to a healthy Docling Serve v1 endpoint".to_string(),
            ),
            checked_at,
        });
    };
    let result = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .expect("bounded reqwest client")
        .get(format!("{base_url}/health"))
        .send()
        .await;
    match result {
        Ok(response) if response.status().is_success() => Json(DoclingHealthResponse {
            schema: "focusa.context_adapter_health.v1",
            adapter_id: "docling-serve.v1",
            configured: true,
            status: "healthy".to_string(),
            endpoint: Some(base_url),
            message: "Docling Serve v1 health check passed".to_string(),
            recovery_action: None,
            checked_at,
        }),
        Ok(response) => Json(DoclingHealthResponse {
            schema: "focusa.context_adapter_health.v1",
            adapter_id: "docling-serve.v1",
            configured: true,
            status: "degraded".to_string(),
            endpoint: Some(base_url),
            message: format!("Docling health returned {}", response.status()),
            recovery_action: Some(
                "Restore Docling readiness, then retry the ingestion".to_string(),
            ),
            checked_at,
        }),
        Err(error) => Json(DoclingHealthResponse {
            schema: "focusa.context_adapter_health.v1",
            adapter_id: "docling-serve.v1",
            configured: true,
            status: "offline".to_string(),
            endpoint: Some(base_url),
            message: format!("Docling health request failed: {error}"),
            recovery_action: Some("Start or reconnect Docling Serve v1, then retry".to_string()),
            checked_at,
        }),
    }
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/context/sources/commit", post(commit))
        .route("/v1/context/sources/ingest", post(ingest))
        .route("/v1/context/sources", get(list))
        .route("/v1/context/retrieve", post(retrieve))
        .route("/v1/context/adapters/docling/health", get(docling_health))
}
