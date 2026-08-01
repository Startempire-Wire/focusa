use super::semantic_integrity::{
    Availability, ExactScope, OperationRequest, OperationResult, CONTRACT,
};
use crate::server::AppState;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use focusa_core::{
    runtime::persistence_sqlite::SqlitePersistence,
    semantic_migration::{compatibility_read, inspect_version, plan_v1_migration},
    semantic_reflex::SHARED_SEMANTIC_REFLEXES,
    semantic_replay::{replay, SemanticEventEnvelope, SemanticPairEvent},
    semantic_settlement::{evaluate_settlement, SettlementInput},
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::sync::Arc;

const EVENT_APPEND_OPERATIONS: &[&str] = &[
    "semantic_pair.create",
    "semantic_pair.obligations.compile",
    "semantic_pair.verification.plan.commit",
    "semantic_pair.verify.start",
    "semantic_pair.verify.findings",
    "semantic_pair.builder.respond",
    "semantic_pair.finding.respond",
    "semantic_pair.finding.resolve",
    "semantic_pair.builder.repair",
    "semantic_pair.settlement.commit",
];

pub fn operation_is_executable(operation_id: &str) -> bool {
    EVENT_APPEND_OPERATIONS.contains(&operation_id)
        || matches!(
            operation_id,
            "semantic_pair.get"
                | "semantic_pair.replay"
                | "semantic_pair.snapshot.get"
                | "semantic_pair.receipt.get"
                | "semantic_pair.migration.status"
                | "semantic_pair.migration.run"
                | "semantic_pair.settlement.preview"
                | "semantic.reflex.visibility"
        )
}

pub async fn execute(state: Arc<AppState>, request: &OperationRequest) -> Option<Response> {
    let operation_id = request.operation_id.as_str();
    let result = if EVENT_APPEND_OPERATIONS.contains(&operation_id) {
        append_event(&state.persistence, request)
    } else {
        match operation_id {
            "semantic_pair.get"
            | "semantic_pair.replay"
            | "semantic_pair.snapshot.get"
            | "semantic_pair.receipt.get" => load_and_replay(&state.persistence, request),
            "semantic_pair.migration.status" => migration_status(request),
            "semantic_pair.migration.run" => run_migration(&state.persistence, request),
            "semantic_pair.settlement.preview" => settlement_preview(request),
            "semantic.reflex.visibility" => reflex_visibility(),
            _ => return None,
        }
    };
    Some(match result {
        Ok((message, data, evidence_refs, receipt_refs)) => {
            success(request, message, data, evidence_refs, receipt_refs)
        }
        Err((status, code, message)) => failure(request, status, code, message),
    })
}

type ExecutorResult =
    Result<(String, Value, Vec<String>, Vec<String>), (StatusCode, &'static str, String)>;

fn append_event(persistence: &SqlitePersistence, request: &OperationRequest) -> ExecutorResult {
    let event_value = request
        .payload
        .get("event")
        .cloned()
        .ok_or_else(|| bad("event_required", "payload.event is required"))?;
    let event: SemanticEventEnvelope = serde_json::from_value(event_value).map_err(|error| {
        bad(
            "event_invalid",
            format!("event envelope is invalid: {error}"),
        )
    })?;
    validate_event_for_operation(&request.operation_id, &event.event)?;
    let pair_id = payload_pair_id(request, Some(&event.pair_id))?;
    if pair_id != event.pair_id {
        return Err(bad(
            "pair_id_mismatch",
            "payload pair_id must match event pair_id",
        ));
    }
    let storage_key = scoped_pair_key(&request.scope, &pair_id);
    let mut events = persistence
        .load_semantic_pair_events(&storage_key)
        .map_err(internal)?;
    if events
        .iter()
        .any(|stored| stored.event_id == event.event_id)
    {
        let replayed =
            replay(&events).map_err(|error| conflict("replay_invalid", error.to_string()))?;
        return Ok((
            "idempotent event already persisted".into(),
            json!({"pair_id": pair_id, "aggregate": replayed.aggregate, "head_hash": replayed.head_hash}),
            vec![format!("semantic-event:{}", event.event_id)],
            vec![],
        ));
    }
    events.push(event.clone());
    let replayed =
        replay(&events).map_err(|error| conflict("event_rejected", error.to_string()))?;
    persistence
        .append_scoped_semantic_pair_events(&storage_key, &[event.clone()])
        .map_err(internal)?;
    Ok((
        "semantic event durably persisted and replayed".into(),
        json!({"pair_id": pair_id, "aggregate": replayed.aggregate, "head_hash": replayed.head_hash}),
        vec![format!("semantic-event:{}", event.event_id)],
        replayed
            .aggregate
            .receipts
            .iter()
            .map(|receipt| receipt.receipt_id.clone())
            .collect(),
    ))
}

fn load_and_replay(persistence: &SqlitePersistence, request: &OperationRequest) -> ExecutorResult {
    let pair_id = payload_pair_id(request, None)?;
    let storage_key = scoped_pair_key(&request.scope, &pair_id);
    let events = persistence
        .load_semantic_pair_events(&storage_key)
        .map_err(internal)?;
    if events.is_empty() {
        return Err((
            StatusCode::NOT_FOUND,
            "pair_not_found",
            "semantic pair was not found in this exact scope".into(),
        ));
    }
    let replayed =
        replay(&events).map_err(|error| conflict("replay_invalid", error.to_string()))?;
    Ok((
        "semantic pair loaded by deterministic replay".into(),
        json!({"pair_id": pair_id, "aggregate": replayed.aggregate, "head_hash": replayed.head_hash, "event_count": events.len()}),
        events
            .iter()
            .map(|event| format!("semantic-event:{}", event.event_id))
            .collect(),
        replayed
            .aggregate
            .receipts
            .iter()
            .map(|receipt| receipt.receipt_id.clone())
            .collect(),
    ))
}

fn migration_status(request: &OperationRequest) -> ExecutorResult {
    let document = request
        .payload
        .get("document")
        .ok_or_else(|| bad("document_required", "payload.document is required"))?;
    let bytes =
        serde_json::to_vec(document).map_err(|error| bad("document_invalid", error.to_string()))?;
    let state =
        inspect_version(&bytes).map_err(|error| bad("version_invalid", error.to_string()))?;
    let compatibility = compatibility_read(&bytes)
        .map_err(|error| bad("compatibility_invalid", error.to_string()))?;
    Ok((
        "semantic compatibility status evaluated".into(),
        json!({"state": state, "compatibility": format!("{compatibility:?}")}),
        vec![],
        vec![],
    ))
}

fn run_migration(persistence: &SqlitePersistence, request: &OperationRequest) -> ExecutorResult {
    let document = request
        .payload
        .get("document")
        .ok_or_else(|| bad("document_required", "payload.document is required"))?;
    let bytes =
        serde_json::to_vec(document).map_err(|error| bad("document_invalid", error.to_string()))?;
    let dry_run = request
        .payload
        .get("dry_run")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let migration_id = request.idempotency_key.as_deref().unwrap_or("missing");
    let mut plan = plan_v1_migration(&bytes, migration_id, dry_run)
        .map_err(|error| bad("migration_invalid", error.to_string()))?;
    let pair_id = plan.aggregate.pair_id.clone();
    let storage_key = scoped_pair_key(&request.scope, &pair_id);
    plan.aggregate.pair_id.clone_from(&storage_key);
    plan.receipt.pair_id.clone_from(&storage_key);
    let receipt = if dry_run {
        plan.receipt.clone()
    } else {
        persistence
            .apply_semantic_pair_migration(&plan)
            .map_err(internal)?
    };
    let mut projected_receipt = receipt;
    projected_receipt.pair_id.clone_from(&pair_id);
    Ok((
        if dry_run {
            "semantic migration dry-run completed"
        } else {
            "semantic migration applied"
        }
        .into(),
        json!({"pair_id": pair_id, "receipt": projected_receipt}),
        vec![format!(
            "semantic-source:sha256:{}",
            hex::encode(Sha256::digest(&bytes))
        )],
        vec![format!("migration-receipt:{}", migration_id)],
    ))
}

fn reflex_visibility() -> ExecutorResult {
    Ok((
        "executable shared semantic reflex catalog projected".into(),
        json!({"runtime_status": "executable", "reflexes": SHARED_SEMANTIC_REFLEXES}),
        vec!["runtime:semantic-reflex:v1".into()],
        vec![],
    ))
}

fn settlement_preview(request: &OperationRequest) -> ExecutorResult {
    let input: SettlementInput = serde_json::from_value(request.payload.clone())
        .map_err(|error| bad("settlement_invalid", error.to_string()))?;
    let evaluation = evaluate_settlement(&input);
    Ok((
        "semantic settlement evaluated without mutation".into(),
        json!(evaluation),
        vec![],
        vec![],
    ))
}

fn validate_event_for_operation(
    operation_id: &str,
    event: &SemanticPairEvent,
) -> Result<(), (StatusCode, &'static str, String)> {
    let valid = match operation_id {
        "semantic_pair.create" => matches!(event, SemanticPairEvent::PairCreated { .. }),
        "semantic_pair.obligations.compile" => {
            matches!(event, SemanticPairEvent::ObligationAdded(_))
        }
        "semantic_pair.verification.plan.commit" => {
            matches!(event, SemanticPairEvent::PlanAdded(_))
        }
        "semantic_pair.verify.start" => matches!(event, SemanticPairEvent::AssignmentAdded(_)),
        "semantic_pair.verify.findings" => matches!(event, SemanticPairEvent::FindingAdded(_)),
        "semantic_pair.builder.respond" | "semantic_pair.finding.respond" => {
            matches!(event, SemanticPairEvent::ResponseAdded(_))
        }
        "semantic_pair.finding.resolve" => matches!(event, SemanticPairEvent::DispositionAdded(_)),
        "semantic_pair.builder.repair" => matches!(event, SemanticPairEvent::RerouteAdded(_)),
        "semantic_pair.settlement.commit" => matches!(event, SemanticPairEvent::SettlementAdded(_)),
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        Err(bad(
            "event_operation_mismatch",
            "event variant does not match operation_id",
        ))
    }
}

fn payload_pair_id(
    request: &OperationRequest,
    fallback: Option<&str>,
) -> Result<String, (StatusCode, &'static str, String)> {
    request
        .payload
        .get("pair_id")
        .and_then(Value::as_str)
        .or(fallback)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| bad("pair_id_required", "payload.pair_id is required"))
}

fn scoped_pair_key(scope: &ExactScope, pair_id: &str) -> String {
    let material = format!(
        "{}\0{}\0{}",
        scope.project_root, scope.continuity_id, pair_id
    );
    format!("scoped-{:x}", Sha256::digest(material.as_bytes()))
}

fn success(
    request: &OperationRequest,
    message: String,
    data: Value,
    evidence_refs: Vec<String>,
    receipt_refs: Vec<String>,
) -> Response {
    (
        StatusCode::OK,
        Json(OperationResult {
            contract: CONTRACT,
            operation_id: request.operation_id.clone(),
            scope: request.scope.clone(),
            state: Availability::Supported,
            degraded: false,
            message,
            data,
            evidence_refs,
            receipt_refs,
            observed_at: Utc::now().to_rfc3339(),
        }),
    )
        .into_response()
}

fn failure(
    request: &OperationRequest,
    status: StatusCode,
    code: &'static str,
    message: String,
) -> Response {
    (status, Json(json!({"contract": CONTRACT, "operation_id": request.operation_id, "scope": request.scope, "state": "degraded", "degraded": true, "error_code": code, "message": message, "evidence_refs": [], "receipt_refs": [], "observed_at": Utc::now().to_rfc3339()}))).into_response()
}

fn bad(code: &'static str, message: impl Into<String>) -> (StatusCode, &'static str, String) {
    (StatusCode::BAD_REQUEST, code, message.into())
}
fn conflict(code: &'static str, message: impl Into<String>) -> (StatusCode, &'static str, String) {
    (StatusCode::CONFLICT, code, message.into())
}
fn internal(error: impl std::fmt::Display) -> (StatusCode, &'static str, String) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "persistence_failure",
        error.to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use focusa_core::{
        semantic_pair::{BuilderAttempt, BuilderContext, ImmutableSnapshot},
        semantic_replay::GENESIS_HASH,
        types::FocusaConfig,
    };
    use uuid::Uuid;

    fn persistence() -> SqlitePersistence {
        let mut config = FocusaConfig::default();
        config.data_dir = std::env::temp_dir()
            .join(format!("focusa-semantic-api-{}", Uuid::now_v7()))
            .to_string_lossy()
            .to_string();
        SqlitePersistence::new(&config).expect("semantic API persistence")
    }

    fn scope(continuity_id: &str) -> ExactScope {
        ExactScope {
            project_root: "/project".into(),
            continuity_id: continuity_id.into(),
        }
    }

    fn create_event() -> SemanticEventEnvelope {
        SemanticEventEnvelope::new(
            "event-create",
            "pair-1",
            0,
            "2026-08-01T00:00:00Z",
            GENESIS_HASH,
            SemanticPairEvent::PairCreated {
                builder_attempt: BuilderAttempt {
                    attempt_id: "attempt-1".into(),
                    builder: "builder-1".into(),
                    started_at: "2026-08-01T00:00:00Z".into(),
                },
                builder_context: BuilderContext::default(),
                snapshot: ImmutableSnapshot {
                    snapshot_id: "snapshot-1".into(),
                    captured_at: "2026-08-01T00:00:00Z".into(),
                    content_hash: "sha256:snapshot".into(),
                    artifact_refs: vec![],
                },
            },
        )
        .expect("event")
    }

    fn request(operation_id: &str, scope: ExactScope, payload: Value) -> OperationRequest {
        OperationRequest {
            contract: Some(CONTRACT.into()),
            operation_id: operation_id.into(),
            scope,
            payload,
            idempotency_key: Some("idempotency-1".into()),
            confirmation: Some("confirm".into()),
        }
    }

    #[test]
    fn executable_registry_is_explicit_and_schema_only_operations_remain_false() {
        assert!(operation_is_executable("semantic_pair.create"));
        assert!(operation_is_executable("semantic_pair.replay"));
        assert!(operation_is_executable("semantic_pair.migration.run"));
        assert!(!operation_is_executable("semantic_pair.pause"));
        assert!(!operation_is_executable("vertical.bundle.activate"));
    }

    #[test]
    fn durable_create_replay_is_idempotent_and_scope_isolated() {
        let persistence = persistence();
        let event = create_event();
        let create = request(
            "semantic_pair.create",
            scope("continuity-1"),
            json!({"pair_id": "pair-1", "event": event}),
        );
        let first = append_event(&persistence, &create).expect("create");
        assert_eq!(first.1["aggregate"]["pair_id"], "pair-1");
        let second = append_event(&persistence, &create).expect("idempotent create");
        assert!(second.0.contains("idempotent"));
        let replay_request = request(
            "semantic_pair.replay",
            scope("continuity-1"),
            json!({"pair_id": "pair-1"}),
        );
        let replayed = load_and_replay(&persistence, &replay_request).expect("replay");
        assert_eq!(replayed.1["event_count"], 1);
        let foreign = request(
            "semantic_pair.replay",
            scope("foreign"),
            json!({"pair_id": "pair-1"}),
        );
        assert_eq!(
            load_and_replay(&persistence, &foreign).unwrap_err().0,
            StatusCode::NOT_FOUND
        );
    }

    #[test]
    fn event_variant_must_match_operation_authority() {
        let persistence = persistence();
        let mismatch = request(
            "semantic_pair.settlement.commit",
            scope("continuity-1"),
            json!({"pair_id": "pair-1", "event": create_event()}),
        );
        let error = append_event(&persistence, &mismatch).unwrap_err();
        assert_eq!(error.1, "event_operation_mismatch");
    }

    #[test]
    fn scoped_storage_keys_do_not_expose_or_merge_scope_text() {
        let first = scoped_pair_key(&scope("one"), "pair-1");
        let second = scoped_pair_key(&scope("two"), "pair-1");
        assert_ne!(first, second);
        assert!(first.starts_with("scoped-"));
        assert!(!first.contains("/project"));
    }
}
