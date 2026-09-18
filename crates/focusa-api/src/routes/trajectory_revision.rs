//! Read-only reconciliation with the existing trajectory ledger; never creates authority.
use super::*;

pub(super) async fn enrich(payload: &mut Value, state: &Arc<AppState>) {
    if payload
        .pointer("/trajectory/durable_lifecycle/canonical")
        .and_then(Value::as_bool)
        != Some(true)
    {
        return;
    }
    let root = view_project_root(payload);
    let Some(continuity) = view_continuity_id(payload) else {
        return;
    };
    let reader = Arc::clone(state);
    let result = tokio::task::spawn_blocking(move || {
        reader
            .persistence
            .read_trajectory_ladder_events(&root, Some(&continuity), 500)
    })
    .await;
    match result {
        Ok(Ok(events)) => reconcile(payload, &events),
        Ok(Err(error)) => {
            warn!("Trajectory revision lookup failed: {error}");
            unavailable(payload);
        }
        Err(error) => {
            warn!("Trajectory revision lookup task failed: {error}");
            unavailable(payload);
        }
    }
}

fn unavailable(payload: &mut Value) {
    payload["trajectory"]["hlt_version"] = Value::Null;
    payload["trajectory"]["ledger_coherence"] = json!({
        "status": "unavailable", "failure_class": "trajectory_ledger_read_failed",
        "source": "trajectory_ladder_ledger", "verified": false
    });
}

pub(super) fn reconcile(payload: &mut Value, events: &[TrajectoryLadderEvent]) {
    let root = view_project_root(payload);
    let continuity = view_continuity_id(payload);
    let id = payload
        .pointer("/trajectory/trajectory_id")
        .and_then(Value::as_str);
    let scoped: Vec<_> = events
        .iter()
        .filter(|event| {
            event.project_root == root
                && event.continuity_id == continuity
                && Some(event.trajectory_id.as_str()) == id
        })
        .collect();
    let head = scoped
        .iter()
        .copied()
        .filter(|event| event.level == TrajectoryLadderLevel::Hlt)
        .max_by_key(|event| (event.lamport_ts, event.timestamp, &event.event_id));
    let Some(head) = head else {
        payload["trajectory"]["hlt_version"] = Value::Null;
        payload["trajectory"]["ledger_coherence"] = json!({
            "status": "not_found", "source": "trajectory_ladder_ledger", "verified": false,
            "reason": "No matching HLT revision in the bounded ledger window"
        });
        return;
    };
    let version = head.hlt_version;
    let event_id = head.event_id.clone();
    let mut revision: Vec<_> = scoped
        .into_iter()
        .filter(|event| event.hlt_version == version)
        .cloned()
        .collect();
    revision.sort_by(|left, right| {
        left.lamport_ts
            .cmp(&right.lamport_ts)
            .then_with(|| left.timestamp.cmp(&right.timestamp))
            .then_with(|| left.event_id.cmp(&right.event_id))
    });
    let reconstruction = reconstruct_trajectory_events(&revision);
    let mismatches: Vec<_> = ["hlt", "mlg", "stg"]
        .into_iter()
        .filter(|level| {
            let ledger = reconstruction[*level]
                .as_str()
                .map(|value| bounded(value, 240));
            let view = payload["trajectory"]["trajectory_ladder"][*level].as_str();
            ledger.as_deref() != view
        })
        .collect();
    let matched = mismatches.is_empty();
    payload["trajectory"]["hlt_version"] = if matched { json!(version) } else { Value::Null };
    payload["trajectory"]["ledger_coherence"] = json!({
        "status": if matched { "matched" } else { "mismatch" },
        "source": "trajectory_ladder_ledger", "verified": matched,
        "ledger_hlt_version": version, "hlt_event_id": event_id,
        "mismatched_levels": mismatches
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn revision_matches_only_exact_scope_and_matching_levels() {
        let record = TrajectoryProjectionRecord {
            trajectory_id: "goal:a".into(),
            project_root: Some("/fixture/a".into()),
            continuity_id: Some("continuity:a".into()),
            long_term_goal: "HLT".into(),
            mid_level_goal: Some("MLG".into()),
            short_term_goal: Some("STG".into()),
            ..TrajectoryProjectionRecord::default()
        };
        let entry = HltLedgerEntry::new("/fixture/a".into(), "HLT".into(), "test", 7)
            .with_scope(Some("continuity:a".into()), None);
        let events = trajectory_commit_events(&record, None, &entry, &[], None);
        let original = json!({
            "project_identity": {"project_root":"/fixture/a", "continuity_id":"continuity:a"},
            "trajectory": {"trajectory_id":"goal:a", "trajectory_ladder":{"hlt":"HLT","mlg":"MLG","stg":"STG"}}
        });
        let mut view = original.clone();
        reconcile(&mut view, &events);
        assert_eq!(view["trajectory"]["hlt_version"], 7);
        assert_eq!(view["trajectory"]["ledger_coherence"]["status"], "matched");
        view["trajectory"]["trajectory_ladder"]["stg"] = json!("different");
        reconcile(&mut view, &events);
        assert!(view["trajectory"]["hlt_version"].is_null());
        assert_eq!(view["trajectory"]["ledger_coherence"]["status"], "mismatch");
        for pointer in [
            "/project_identity/project_root",
            "/project_identity/continuity_id",
            "/trajectory/trajectory_id",
        ] {
            let mut foreign = original.clone();
            *foreign.pointer_mut(pointer).unwrap() = json!("foreign");
            reconcile(&mut foreign, &events);
            assert!(foreign["trajectory"]["hlt_version"].is_null());
            assert_eq!(
                foreign["trajectory"]["ledger_coherence"]["status"],
                "not_found"
            );
        }
        unavailable(&mut view);
        assert_eq!(
            view["trajectory"]["ledger_coherence"]["status"],
            "unavailable"
        );
    }
}
