//! UIAI research-packet intake — gap A of the tool-ecosystem audit
//! (docs/170). Converts a UIAI focusa_research_diagnostics_packet into
//! bounded durable evidence refs. Intake does not mutate a Workpoint;
//! callers explicitly link the returned refs through the canonical
//! Workpoint evidence route.

use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{Value, json};
use std::path::PathBuf;
use std::sync::Arc;

use crate::server::AppState;

const MAX_EVIDENCE_REFS: usize = 64;
const MAX_EVIDENCE_REF_BYTES: usize = 2_048;

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/v1/evidence/research-packet", post(ingest))
}

#[derive(Deserialize)]
pub struct ResearchPacketBody {
    pub packet: Value,
}

fn push_bounded_ref(refs: &mut Vec<String>, prefix: &str, value: &str) {
    if refs.len() >= MAX_EVIDENCE_REFS {
        return;
    }
    let value = value.trim();
    if value.is_empty() {
        return;
    }
    let evidence_ref = format!("{prefix}{value}");
    if evidence_ref.len() <= MAX_EVIDENCE_REF_BYTES && !refs.contains(&evidence_ref) {
        refs.push(evidence_ref);
    }
}

/// The UIAI packet's surfaces become bounded evidence refs with the packet's
/// digest as an anchor. Raw browser payloads never enter the evidence ledger.
fn surface_evidence_refs(packet: &Value) -> Vec<String> {
    let mut refs = Vec::new();
    if let Some(surfaces) = packet.get("surfaces").and_then(Value::as_array) {
        for surface in surfaces {
            if let Some(name) = surface.as_str() {
                push_bounded_ref(&mut refs, "uiai:", name);
            }
        }
    }
    if let Some(source) = packet.get("source_url").and_then(Value::as_str) {
        push_bounded_ref(&mut refs, "uiai:source:", source);
    }
    if let Some(digest) = packet
        .get("packet_digest")
        .or_else(|| packet.get("digest"))
        .and_then(Value::as_str)
    {
        push_bounded_ref(&mut refs, "uiai:digest:", digest);
    }
    refs
}

fn persist_research_evidence(path: PathBuf, evidence_refs: Vec<String>) -> anyhow::Result<Value> {
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
}

async fn ingest(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ResearchPacketBody>,
) -> Json<Value> {
    let evidence_refs = surface_evidence_refs(&body.packet);
    if evidence_refs.is_empty() {
        return Json(json!({
            "status": "rejected",
            "failure_class": "research_packet_empty",
            "retry_posture": "do_not_retry_unchanged",
            "safe_recovery": "supply a packet with surfaces, source_url, or packet_digest",
            "error": "packet carries no bounded evidence references",
        }));
    }
    let path = crate::routes::events_sqlite::focusa_db_path(&state.config.data_dir);
    match tokio::task::spawn_blocking(move || persist_research_evidence(path, evidence_refs)).await
    {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packet_refs_are_bounded_deduplicated_and_raw_payload_free() {
        let packet = json!({
            "surfaces": ["screenshot:abc", "screenshot:abc", "", "network:def"],
            "source_url": "https://example.test/page",
            "packet_digest": "sha256:123",
            "raw_private_payload": "must-not-persist"
        });
        let refs = surface_evidence_refs(&packet);
        assert_eq!(
            refs,
            vec![
                "uiai:screenshot:abc",
                "uiai:network:def",
                "uiai:source:https://example.test/page",
                "uiai:digest:sha256:123",
            ]
        );
        assert!(
            !serde_json::to_string(&refs)
                .expect("serialize refs")
                .contains("must-not-persist")
        );
    }

    #[test]
    fn research_packet_persists_as_typed_evidence_record() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("focusa.sqlite");
        let payload =
            persist_research_evidence(path.clone(), vec!["uiai:digest:sha256:123".to_string()])
                .expect("persist evidence");
        assert_eq!(payload["status"], "ingested");

        let conn = rusqlite::Connection::open(path).expect("open evidence database");
        let (kind, refs_json): (String, String) = conn
            .query_row(
                "SELECT kind, refs_json FROM evidence_records WHERE evidence_id = ?1",
                [payload["evidence_id"].as_str().expect("evidence id")],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("read evidence record");
        assert_eq!(kind, "uiai_research_packet");
        assert_eq!(refs_json, r#"["uiai:digest:sha256:123"]"#);
    }
}
