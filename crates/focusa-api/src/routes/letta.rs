use crate::server::AppState;
use axum::{Json, Router, routing::get};
use letta_adapter::canonical_letta_capability_contract;
use serde_json::{Value, json};
use std::sync::Arc;

async fn status() -> Json<Value> {
    Json(json!({
        "schema": "focusa.letta_surface_status.v1",
        "availability": "unconfigured",
        "identity": null,
        "active_operation": null,
        "evidence_refs": [],
        "recovery": {
            "required": true,
            "next_action": "configure_approved_endpoint_and_credential_reference"
        },
        "controls": [
            {"id": "inspect", "mutation": false, "supported": true},
            {"id": "create", "mutation": true, "supported": false},
            {"id": "send", "mutation": true, "supported": false},
            {"id": "resume", "mutation": true, "supported": false},
            {"id": "checkpoint", "mutation": true, "supported": false}
        ],
        "capability_contract": canonical_letta_capability_contract()
    }))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/v1/letta/status", get(status))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn unconfigured_status_never_implies_mutation_authority() {
        let body = status().await.0;
        assert_eq!(body["availability"], "unconfigured");
        assert_eq!(body["identity"], Value::Null);
        assert!(
            body["controls"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|control| control["mutation"] == true)
                .all(|control| control["supported"] == false)
        );
    }
}
