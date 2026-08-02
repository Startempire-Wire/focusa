//! Embedded Spec 137A conformance truth for temporal runtime surfaces.

use serde_json::{Value, json};

const SPEC137A_SURFACE_PARITY: &str =
    include_str!("../../../../docs/contracts/spec137a-surface-parity.v1.yaml");

pub(super) fn spec137a_conformance_surface() -> Value {
    let mut artifact: Value = serde_json::from_str(SPEC137A_SURFACE_PARITY).unwrap_or_else(|error| {
        json!({
            "schema":"focusa.spec137a_surface_parity.v1",
            "status":"degraded",
            "full_conformance_status":"blocked",
            "warnings":[format!("embedded Spec137A surface parity artifact is invalid: {error}")],
            "recovery_tools":["focusa_temporal_authority","focusa_tool_doctor"]
        })
    });
    let incomplete = artifact
        .get("records")
        .and_then(Value::as_array)
        .map(|records| {
            records.iter().any(|record| {
                !matches!(
                    record.get("state").and_then(Value::as_str),
                    Some("implemented" | "verified_complete" | "verified_not_applicable")
                )
            })
        })
        .unwrap_or(true);
    artifact["artifact_source"] = json!("embedded_release");
    if incomplete {
        artifact["status"] = json!("degraded");
        artifact["full_conformance_status"] = json!("blocked");
        let warnings = artifact
            .get_mut("warnings")
            .and_then(Value::as_array_mut)
            .expect("Spec137A warnings must be an array");
        warnings.push(json!(
            "Spec137A contains incomplete surface records; full conformance remains blocked."
        ));
    }
    artifact
}

#[cfg(test)]
mod tests {
    use super::spec137a_conformance_surface;

    #[test]
    fn conformance_is_release_embedded_and_live_proof_still_blocks_closure() {
        let artifact = spec137a_conformance_surface();
        assert_eq!(artifact["artifact_source"], "embedded_release");
        assert_eq!(artifact["status"], "implemented");
        assert_eq!(
            artifact["full_conformance_status"],
            "blocked_live_proof_required"
        );
        assert!(
            artifact["warnings"]
                .as_array()
                .is_some_and(|warnings| warnings.iter().any(|warning| warning
                    .as_str()
                    .is_some_and(|text| text.contains("two clean exact-scope"))))
        );
    }
}
