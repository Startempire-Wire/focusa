//! Bounded Spec 137 temporal context projection for continuation and awareness surfaces.

use chrono::Utc;
use focusa_core::temporal::{TemporalEventKind, TemporalLedger, TemporalScope, project_temporal};
use serde_json::{Value, json};

pub fn bounded_temporal_context(
    project_root: &str,
    continuity_id: &str,
    workpoint_id: Option<String>,
    item_id: Option<String>,
) -> Value {
    let mut scope = TemporalScope::project(project_root.to_string(), continuity_id.to_string());
    scope.workpoint_id = workpoint_id;
    scope.item_id = item_id;
    let ledger = match TemporalLedger::for_project(scope.clone()) {
        Ok(ledger) => ledger,
        Err(error) => {
            return json!({
                "schema":"focusa.bounded_temporal_context.v1",
                "status":"unavailable",
                "canonical":false,
                "scope":scope,
                "failure_class":"unsafe_temporal_scope",
                "reason":format!("{error:?}"),
                "cache_safe_refs_only":true,
                "recovery_tools":["focusa_project_verify","focusa_temporal_authority"]
            });
        }
    };
    let events = match ledger.read_all() {
        Ok(events) => events,
        Err(error) => {
            return json!({
                "schema":"focusa.bounded_temporal_context.v1",
                "status":"unavailable",
                "canonical":false,
                "scope":scope,
                "failure_class":"temporal_ledger_unavailable",
                "reason":format!("{error:?}"),
                "cache_safe_refs_only":true,
                "recovery_tools":["focusa_temporal_authority","focusa_tool_doctor"]
            });
        }
    };
    let as_of = Utc::now();
    let projection = project_temporal(scope.clone(), &events, as_of);
    let attested_legacy_digests = events
        .iter()
        .filter(|event| event.event_kind == TemporalEventKind::LegacySignatureAttestation)
        .filter_map(|event| event.metadata.get("legacy_event_digests"))
        .filter_map(Value::as_array)
        .flatten()
        .filter_map(Value::as_str)
        .collect::<std::collections::BTreeSet<_>>();
    let unsigned_legacy_event_count = events
        .iter()
        .filter(|event| {
            event.signature.is_none()
                && event.event_kind != TemporalEventKind::LegacySignatureAttestation
                && !attested_legacy_digests.contains(event.digest.as_str())
        })
        .count();
    let mut evidence_refs = projection
        .temporal_priority_frame
        .as_ref()
        .map(|frame| frame.evidence_refs.clone())
        .unwrap_or_default();
    if let Some(guard) = projection.temporal_execution_guard.as_ref() {
        evidence_refs.push(guard.receipt_ref.clone());
    }
    evidence_refs.sort();
    evidence_refs.dedup();
    evidence_refs.truncate(16);

    json!({
        "schema":"focusa.bounded_temporal_context.v1",
        "status":if unsigned_legacy_event_count == 0 { "completed" } else { "degraded" },
        "canonical":unsigned_legacy_event_count == 0,
        "scope":scope,
        "as_of":as_of,
        "source_event_count":events.len(),
        "integrity_status":if unsigned_legacy_event_count == 0 { "signed_verified" } else { "legacy_attestation_required" },
        "unsigned_legacy_event_count":unsigned_legacy_event_count,
        "deadline_status":projection.deadline_status,
        "active_claim_ref":projection.active_commitment.as_ref().map(|claim| json!({
            "claim_id":claim.claim_id,
            "revision":claim.revision,
            "kind":claim.kind,
            "target_at":claim.target_at,
            "evidence_refs":claim.evidence_refs.iter().take(8).collect::<Vec<_>>()
        })),
        "forecast_range":projection.authorized_forecast_range,
        "calendar_context_ref":projection.human_calendar_context.as_ref().map(|context| &context.context_id),
        "priority_frame_ref":projection.temporal_priority_frame.as_ref().map(|frame| &frame.frame_id),
        "execution_guard_ref":projection.temporal_execution_guard.as_ref().map(|guard| &guard.guard_id),
        "urgency":projection.urgency,
        "warning_refs":projection.warnings.into_iter().take(8).collect::<Vec<_>>(),
        "evidence_refs":evidence_refs,
        "cache_safe_refs_only":true,
        "rehydrate_tool":"focusa_temporal_authority"
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_scope_is_bounded_signed_and_never_infers_urgency() {
        let root = std::env::temp_dir().join(format!(
            "focusa-bounded-temporal-context-{}",
            uuid::Uuid::now_v7()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let context = bounded_temporal_context(
            root.to_string_lossy().as_ref(),
            "continuity:bounded-temporal-test",
            Some("workpoint:test".into()),
            Some("item:test".into()),
        );
        assert_eq!(context["status"], "completed");
        assert_eq!(context["canonical"], true);
        assert_eq!(context["source_event_count"], 0);
        assert_eq!(context["cache_safe_refs_only"], true);
        assert!(context["urgency"].is_null());
        assert!(context.get("projection").is_none());
        std::fs::remove_dir_all(root).unwrap();
    }
}
