//! UIAI research-packet intake — gap A of the tool-ecosystem audit
//! (docs/170). Converts a UIAI focusa_research_diagnostics_packet into
//! typed evidence links on the named Workpoint — the hand-in-glove seam
//! between UIAI Engine and Focusa evidence. Packets never mutate state
//! directly; they land as evidence refs + the packet digest.

use axum::Json;
use axum::Router;
use axum::extract::State;
use axum::routing::post;
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

use crate::server::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/v1/evidence/research-packet", post(ingest))
}

#[derive(Deserialize)]
pub struct ResearchPacketBody {
    pub packet: Value,
}

/// The UIAI packet's surfaces become evidence refs with the packet's
/// digest as the anchor; every surface reference is preserved so the
/// workset/completion consumers can cite exactly what the browser saw.
fn surface_evidence_refs(packet: &Value) -> Vec<String> {
    let mut refs = Vec::new();
    if let Some(surfaces) = packet.get("surfaces").and_then(Value::as_array) {
        for surface in surfaces {
            if let Some(name) = surface.as_str() {
                refs.push(format!("uiai:{}", name));
            }
        }
    }
    if let Some(source) = packet.get("source_url").and_then(Value::as_str) {
        refs.push(format!("uiai:source:{}", source));
    }
    if let Some(digest) = packet
        .get("packet_digest")
        .or_else(|| packet.get("digest"))
        .and_then(Value::as_str)
    {
        refs.push(format!("uiai:digest:{}", digest));
    }
    refs
}

async fn ingest(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ResearchPacketBody>,
) -> Json<Value> {
    let packet = body.packet;
    let evidence_refs = surface_evidence_refs(&packet);
    if evidence_refs.is_empty() {
        return Json(json!({
            "status": "rejected",
            "failure_class": "research_packet_empty",
            "retry_posture": "do_not_retry_unchanged",
            "safe_recovery": "supply a packet with surfaces or a source_url",
            "error": "packet carries no evidence surfaces",
        }));
    }
    // Record the packet durably as an evidence object.
    let path = crate::routes::events_sqlite::focusa_db_path(&state.config.data_dir);
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<Value> {
        let conn = rusqlite::Connection::open(path)?;
        crate::routes::events_sqlite::ensure_evidence_schema(&conn)?;
        let evidence_id = uuid::Uuid::now_v7().to_string();
        conn.execute(
            "INSERT INTO evidence_records (evidence_id, kind, refs_json, recorded_at)
             VALUES (?1, 'uiai_research_packet', ?2, ?3)",
            rusqlite::params![
                evidence_id,
                serde_json::to_string(&evidence_refs)?,
                chrono::Utc::now().to_rfc3339()
            ],
        )?;
        Ok(json!({
            "status": "ingested",
            "evidence_id": evidence_id,
            "evidence_refs": evidence_refs,
            "next": "link these refs to a workpoint via POST /v1/workpoint/evidence/link",
        }))
    })
    .await;
    match result {
        Ok(Ok(payload)) => Json(payload),
        Ok(Err(error)) => Json(focusa_core::error_envelope::internal_error(
            "route",
            &error.to_string(),
        )),
        Err(error) => Json(focusa_core::error_envelope::internal_error(
            "join",
            &format!("{error}"),
        )),
    }
}
