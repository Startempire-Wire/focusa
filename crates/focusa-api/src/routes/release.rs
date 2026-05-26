use crate::server::AppState;
use axum::extract::State;
use axum::{Json, Router, routing::get};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn latest_proof_path(state: &AppState) -> PathBuf {
    PathBuf::from(&state.config.data_dir)
        .join("release-proof")
        .join("latest.json")
}

fn manual_gate(path: &Path) -> Value {
    json!({
        "status": "manual_proof_required",
        "canonical": true,
        "degraded": false,
        "version": env!("CARGO_PKG_VERSION"),
        "summary": "Run release proof before publishing or relying on attached artifacts.",
        "required_command": "focusa release prove --tag <tag> --github",
        "proof_artifact": {"latest_path": path.display().to_string(), "exists": false},
        "evidence_refs": [
            "docs/current/VALIDATION_AND_RELEASE_PROOF.md",
            "docs/current/PRODUCTION_RELEASE_COMMANDS.md"
        ],
        "next_tools": ["focusa doctor", "focusa release prove --tag <tag> --github"],
        "details": {
            "tool_result_v1": {
                "ok": true,
                "status": "manual_proof_required",
                "canonical": true,
                "degraded": false,
                "failure_class": null,
                "retry": {"safe": true, "posture": "manual_gate"},
                "side_effects": [],
                "evidence_refs": ["docs/current/VALIDATION_AND_RELEASE_PROOF.md"]
            }
        }
    })
}

async fn proof_status(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let path = latest_proof_path(&state);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Json(manual_gate(&path));
    };
    let Ok(proof) = serde_json::from_str::<Value>(&text) else {
        let mut payload = manual_gate(&path);
        payload["status"] = json!("proof_artifact_invalid");
        payload["degraded"] = json!(true);
        payload["summary"] =
            json!("Release proof artifact exists but is not valid JSON; rerun release proof.");
        payload["details"]["tool_result_v1"]["status"] = json!("proof_artifact_invalid");
        payload["details"]["tool_result_v1"]["degraded"] = json!(true);
        payload["details"]["tool_result_v1"]["failure_class"] = json!("invalid_artifact");
        return Json(payload);
    };

    let proof_status = proof
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let proven = proof_status == "completed";
    Json(json!({
        "status": if proven { "proven" } else { proof_status },
        "canonical": true,
        "degraded": !proven,
        "version": env!("CARGO_PKG_VERSION"),
        "summary": proof.get("summary").cloned().unwrap_or_else(|| json!("Release proof artifact loaded.")),
        "required_command": "focusa release prove --tag <tag> --github",
        "proof_artifact": {"latest_path": path.display().to_string(), "exists": true},
        "proof": proof,
        "next_tools": ["focusa doctor", "focusa release prove --tag <tag> --github"],
        "details": {
            "tool_result_v1": {
                "ok": proven,
                "status": if proven { "proven" } else { proof_status },
                "canonical": true,
                "degraded": !proven,
                "failure_class": if proven { Value::Null } else { json!("proof_blocked") },
                "retry": {"safe": true, "posture": if proven { "not_needed" } else { "rerun_after_fix" }},
                "side_effects": [],
                "evidence_refs": [path.display().to_string()]
            }
        }
    }))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/v1/release/proof/status", get(proof_status))
}
