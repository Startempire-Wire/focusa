//! Visual workflow evidence routes.
//!
//! POST /v1/visual-workflow/evidence/store — persist a visual evidence artifact via reducer action channel
//! GET  /v1/visual-workflow/evidence       — list visual evidence handles (optionally filtered)

use crate::server::AppState;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::{
    Json, Router,
    routing::{get, post},
};
use focusa_core::types::{Action, HandleKind};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

#[derive(Debug, Clone, Deserialize)]
struct StoreVisualEvidenceBody {
    run_id: String,
    phase: String,
    evidence_kind: String,
    label: String,
    kind: HandleKind,
    /// Base64-encoded content.
    content_b64: Option<String>,
    /// Plain text content (alternative to base64).
    #[serde(default)]
    content: Option<String>,
}

impl StoreVisualEvidenceBody {
    fn resolve_content(&self) -> Result<Vec<u8>, StatusCode> {
        if let Some(ref b64) = self.content_b64 {
            use base64::Engine;
            base64::engine::general_purpose::STANDARD
                .decode(b64)
                .map_err(|_| StatusCode::BAD_REQUEST)
        } else if let Some(ref txt) = self.content {
            Ok(txt.as_bytes().to_vec())
        } else {
            Err(StatusCode::BAD_REQUEST)
        }
    }

    fn to_artifact_label(&self) -> String {
        format!(
            "visual:{}:{}:{}:{}",
            self.run_id, self.phase, self.evidence_kind, self.label
        )
    }
}

#[derive(Debug, Clone, Deserialize)]
struct EvidenceQuery {
    #[serde(default)]
    run_id: Option<String>,
    #[serde(default)]
    phase: Option<String>,
    #[serde(default)]
    evidence_kind: Option<String>,
}

async fn store_visual_evidence(
    State(state): State<Arc<AppState>>,
    Json(body): Json<StoreVisualEvidenceBody>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let content = body.resolve_content()?;
    let label = body.to_artifact_label();

    state
        .command_tx
        .send(Action::StoreArtifact {
            kind: body.kind,
            label: label.clone(),
            content,
        })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let handle_id = loop {
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        let focusa = state.focusa.read().await;
        if let Some(h) = focusa
            .reference_index
            .handles
            .iter()
            .find(|h| h.label == label)
        {
            break h.id;
        }
    };

    Ok(Json(json!({
        "id": handle_id,
        "status": "accepted",
        "run_id": body.run_id,
        "phase": body.phase,
        "evidence_kind": body.evidence_kind,
        "label": body.label,
    })))
}

async fn list_visual_evidence(
    State(state): State<Arc<AppState>>,
    Query(query): Query<EvidenceQuery>,
) -> Json<serde_json::Value> {
    let focusa = state.focusa.read().await;

    let mut items = Vec::new();
    for handle in focusa
        .reference_index
        .handles
        .iter()
        .filter(|h| h.label.starts_with("visual:"))
    {
        let mut parts = handle.label.splitn(5, ':');
        let prefix = parts.next().unwrap_or_default();
        let run_id = parts.next().unwrap_or_default();
        let phase = parts.next().unwrap_or_default();
        let evidence_kind = parts.next().unwrap_or_default();
        let label = parts.next().unwrap_or_default();

        if prefix != "visual" {
            continue;
        }
        if query.run_id.as_deref().is_some_and(|q| q != run_id) {
            continue;
        }
        if query.phase.as_deref().is_some_and(|q| q != phase) {
            continue;
        }
        if query
            .evidence_kind
            .as_deref()
            .is_some_and(|q| q != evidence_kind)
        {
            continue;
        }

        items.push(json!({
            "run_id": run_id,
            "phase": phase,
            "evidence_kind": evidence_kind,
            "label": label,
            "handle": handle,
        }));
    }

    Json(json!({
        "evidence": items,
        "count": items.len(),
    }))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/v1/visual-workflow/evidence/store",
            post(store_visual_evidence),
        )
        .route("/v1/visual-workflow/evidence", get(list_visual_evidence))
}
