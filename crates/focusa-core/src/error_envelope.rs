//! Canonical error envelope (#261 slice 1) — one typed shape for every
//! failure response across API routes, tools, and comprehension surfaces.
//!
//! `audit-error-envelope-parity.mjs` classifies route files against this
//! shape; legacy bare-error responses migrate to `standard_error`.

use serde::{Deserialize, Serialize};

pub const ERROR_ENVELOPE_SCHEMA: &str = "focusa.error_envelope.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FocusaErrorEnvelope {
    pub schema: String,
    pub status: String,
    pub failure_class: String,
    pub retry_posture: String,
    pub safe_recovery: String,
    pub error: String,
}

/// Canonical failure JSON for routes. Retry posture must be one of the
/// standard set (safe_retry / do_not_retry_unchanged / operator_required);
/// safe_recovery tells the consumer exactly what to do next.
pub fn standard_error(
    status: &str,
    failure_class: &str,
    retry_posture: &str,
    safe_recovery: &str,
    error: &str,
) -> serde_json::Value {
    serde_json::json!({
        "schema": ERROR_ENVELOPE_SCHEMA,
        "status": status,
        "failure_class": failure_class,
        "retry_posture": retry_posture,
        "safe_recovery": safe_recovery,
        "error": error,
    })
}

/// Common envelope for infrastructure/persistence failures.
pub fn persistence_error(context: &str, error: &str) -> serde_json::Value {
    standard_error(
        "failed",
        "persistence_unavailable",
        "safe_retry",
        "retry after the persistence layer recovers; no state was lost",
        &format!("{context}: {error}"),
    )
}

/// Common envelope for join/task failures.
pub fn internal_error(context: &str, error: &str) -> serde_json::Value {
    standard_error(
        "failed",
        "internal_processing_error",
        "safe_retry",
        "retry the request; the failure was bounded to this operation",
        &format!("{context}: {error}"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_error_has_every_required_field() {
        let value = standard_error(
            "rejected",
            "scope_mismatch",
            "do_not_retry_unchanged",
            "provide a valid project_root + continuity_id",
            "missing scope",
        );
        assert_eq!(value["schema"], ERROR_ENVELOPE_SCHEMA);
        assert_eq!(value["failure_class"], "scope_mismatch");
        assert_eq!(value["retry_posture"], "do_not_retry_unchanged");
        assert_eq!(value["status"], "rejected");
        assert!(!value["safe_recovery"].as_str().unwrap().is_empty());
    }

    #[test]
    fn persistence_error_uses_safe_retry() {
        let value = persistence_error("checkpoint", "disk full");
        assert_eq!(value["failure_class"], "persistence_unavailable");
        assert_eq!(value["retry_posture"], "safe_retry");
        assert!(value["error"].as_str().unwrap().contains("checkpoint"));
    }

    #[test]
    fn envelope_roundtrips() {
        let envelope = FocusaErrorEnvelope {
            schema: ERROR_ENVELOPE_SCHEMA.to_string(),
            status: "failed".to_string(),
            failure_class: "internal_processing_error".to_string(),
            retry_posture: "safe_retry".to_string(),
            safe_recovery: "retry the request".to_string(),
            error: "boom".to_string(),
        };
        let json = serde_json::to_string(&envelope).unwrap();
        let parsed: FocusaErrorEnvelope = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, envelope);
    }
}
