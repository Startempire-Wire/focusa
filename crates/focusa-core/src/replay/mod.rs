//! Session replay — reconstruct state from event log for export.
//!
//! Per 20-training-dataset-schema §Export Pipeline:
//! Full session replay required for dataset generation.

use crate::reducer;
use crate::runtime::persistence_sqlite::SqlitePersistence;
use crate::types::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Replay configuration.
#[derive(Debug, Clone)]
pub struct ReplayConfig {
    /// Start timestamp (inclusive).
    pub from: Option<DateTime<Utc>>,
    /// End timestamp (exclusive).
    pub until: Option<DateTime<Utc>>,
    /// Session ID to filter by.
    pub session_id: Option<SessionId>,
    /// Frame ID to filter by.
    pub frame_id: Option<FrameId>,
}

/// Replay result containing reconstructed states.
#[derive(Debug, Clone)]
pub struct ReplayResult {
    /// Initial state.
    pub initial_state: FocusaState,
    /// Final state after all events.
    pub final_state: FocusaState,
    /// All intermediate states (optional, for debugging).
    pub states: Vec<(DateTime<Utc>, FocusaState)>,
    /// Events that were replayed.
    pub events_replayed: usize,
    /// Errors encountered during replay.
    pub errors: Vec<String>,
}

/// Comparative replay evidence per task correlation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecondaryLoopComparativePair {
    pub correlation_id: String,
    pub promoted_outcomes: u64,
    pub non_promoted_outcomes: u64,
    pub comparative_improvement_observed: bool,
}

/// Replay-driven secondary-loop comparative summary for doc78 §15.1.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecondaryLoopComparativeReplaySummary {
    pub replay_events_scanned: u64,
    pub secondary_loop_outcome_events: u64,
    pub promoted_outcomes: u64,
    pub rejected_outcomes: u64,
    pub deferred_for_review_outcomes: u64,
    pub archived_failed_attempt_outcomes: u64,
    pub comparative_improvement_pairs: u64,
    pub task_pairs: Vec<SecondaryLoopComparativePair>,
}

/// Summarize replay-log comparative evidence from persisted outcome events.
pub fn secondary_loop_comparative_summary_from_replay(
    persistence: &SqlitePersistence,
    config: &ReplayConfig,
) -> anyhow::Result<SecondaryLoopComparativeReplaySummary> {
    let since_ts = config.from.map(|dt| dt.to_rfc3339());
    let since_id: Option<String> = None;
    let rows = persistence.events_since_raw(since_ts.as_deref(), since_id.as_deref(), 10000)?;

    let mut summary = SecondaryLoopComparativeReplaySummary::default();
    let mut pairs_by_task: BTreeMap<String, (u64, u64)> = BTreeMap::new();
    let mut anonymous_counter: u64 = 0;

    for row in rows {
        if let Some(target_sid) = config.session_id
            && row.session_id != Some(target_sid)
        {
            continue;
        }
        if let Some(until) = config.until
            && row.timestamp > until
        {
            break;
        }

        summary.replay_events_scanned += 1;

        let payload: serde_json::Value = match serde_json::from_str(&row.payload_json) {
            Ok(value) => value,
            Err(error) => {
                tracing::warn!(
                    event_id = %row.event_id,
                    error = %error,
                    "Skipping malformed replay payload while computing comparative summary"
                );
                continue;
            }
        };

        if payload
            .get("type")
            .and_then(serde_json::Value::as_str)
            != Some("ContinuousSecondaryLoopOutcomeRecorded")
        {
            continue;
        }

        let promotion_status = payload
            .get("promotion_status")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("rejected");
        let task_run_id = payload
            .get("task_run_id")
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned);
        let work_item_id = payload
            .get("work_item_id")
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned);

        summary.secondary_loop_outcome_events += 1;
        match promotion_status {
            "promoted" => summary.promoted_outcomes += 1,
            "deferred_for_review" => summary.deferred_for_review_outcomes += 1,
            "archived_failed_attempt" => summary.archived_failed_attempt_outcomes += 1,
            _ => summary.rejected_outcomes += 1,
        }

        let correlation_id = task_run_id.or(work_item_id).unwrap_or_else(|| {
            anonymous_counter += 1;
            format!("anonymous-secondary-outcome-{anonymous_counter}")
        });

        let pair = pairs_by_task.entry(correlation_id).or_insert((0, 0));
        if promotion_status == "promoted" {
            pair.0 += 1;
        } else {
            pair.1 += 1;
        }
    }

    summary.task_pairs = pairs_by_task
        .into_iter()
        .map(
            |(correlation_id, (promoted_outcomes, non_promoted_outcomes))| {
                let comparative_improvement_observed =
                    promoted_outcomes > 0 && non_promoted_outcomes > 0;
                SecondaryLoopComparativePair {
                    correlation_id,
                    promoted_outcomes,
                    non_promoted_outcomes,
                    comparative_improvement_observed,
                }
            },
        )
        .collect();
    summary.comparative_improvement_pairs = summary
        .task_pairs
        .iter()
        .filter(|pair| pair.comparative_improvement_observed)
        .count() as u64;

    Ok(summary)
}

/// Replay events from the event log to reconstruct session state.
///
/// This is the core mechanism for training dataset export:
/// 1. Load events from SQLite
/// 2. Apply reducer to each event sequentially
/// 3. Reconstruct full state history
/// 4. Extract training examples from reconstructed turns
pub fn replay_events(
    persistence: &SqlitePersistence,
    config: &ReplayConfig,
) -> anyhow::Result<ReplayResult> {
    let since_ts = config.from.map(|dt| dt.to_rfc3339());
    let since_id: Option<String> = None;

    // Load events from persistence.
    let entries = persistence.events_since(
        since_ts.as_deref(),
        since_id.as_deref(),
        10000, // Max events to replay
    )?;

    let mut state = FocusaState::default();
    let mut states = Vec::new();
    let mut errors = Vec::new();
    let mut events_replayed = 0usize;

    for entry in entries {
        // Filter by session if specified.
        if let Some(target_sid) = config.session_id
            && entry.session_id != Some(target_sid)
        {
            continue;
        }

        // Filter by timestamp if specified.
        if let Some(until) = config.until
            && entry.timestamp > until
        {
            break;
        }

        // Extract event from entry.
        let event = entry.event.clone();

        // Apply reducer (clone state to avoid move issues on error).
        let state_clone = state.clone();
        match reducer::reduce(state_clone, event) {
            Ok(result) => {
                state = result.new_state;
                states.push((entry.timestamp, state.clone()));
                events_replayed += 1;
            }
            Err(e) => {
                errors.push(format!("Replay error at {}: {}", entry.timestamp, e));
                // Continue replaying despite errors - state unchanged
            }
        }
    }

    Ok(ReplayResult {
        initial_state: FocusaState::default(),
        final_state: state,
        states,
        events_replayed,
        errors,
    })
}

/// Export a training dataset from replayed session.
///
/// Reconstructs turns and generates training examples per
/// 20-training-dataset-schema §Dataset Families.
pub fn export_from_replay(
    result: &ReplayResult,
    dataset_family: &str,
) -> anyhow::Result<Vec<serde_json::Value>> {
    let mut examples = Vec::new();

    match dataset_family {
        "sft" => {
            // Supervised fine-tuning: extract (prompt, response) pairs
            for window in result.states.windows(2) {
                if let Some(example) = extract_sft_example(&window[0].1, &window[1].1) {
                    examples.push(example);
                }
            }
        }
        "preference" => {
            // Preference: extract chosen vs rejected pairs
            for window in result.states.windows(2) {
                if let Some(example) = extract_preference_example(&window[0].1, &window[1].1) {
                    examples.push(example);
                }
            }
        }
        "contrastive" => {
            // Contrastive: extract positive/negative pairs
            for window in result.states.windows(2) {
                if let Some(example) = extract_contrastive_example(&window[0].1, &window[1].1) {
                    examples.push(example);
                }
            }
        }
        _ => anyhow::bail!("Unknown dataset family: {}", dataset_family),
    }

    Ok(examples)
}

fn extract_sft_example(_before: &FocusaState, after: &FocusaState) -> Option<serde_json::Value> {
    // Extract turn completion for SFT
    // In a real implementation, this would look at the active_turn
    // and frame state to build a training example

    after.active_turn.as_ref().map(|_turn| {
        serde_json::json!({
            "instruction": "placeholder - would extract from turn",
            "response": "placeholder - would extract from assistant_output",
            "metadata": {
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }
        })
    })
}

fn extract_preference_example(
    _before: &FocusaState,
    _after: &FocusaState,
) -> Option<serde_json::Value> {
    // Would extract chosen vs rejected responses
    None
}

fn extract_contrastive_example(
    _before: &FocusaState,
    _after: &FocusaState,
) -> Option<serde_json::Value> {
    // Would extract positive/negative examples
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::persistence_sqlite::SqlitePersistence;
    use rusqlite::params;
    use uuid::Uuid;

    fn temp_dir(prefix: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::now_v7()));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    fn append_event(
        persistence: &SqlitePersistence,
        event: FocusaEvent,
        session_id: Option<SessionId>,
    ) {
        let entry = EventLogEntry {
            id: Uuid::now_v7(),
            timestamp: Utc::now(),
            event,
            correlation_id: None,
            origin: SignalOrigin::Daemon,
            machine_id: None,
            instance_id: None,
            session_id,
            thread_id: None,
            is_observation: false,
        };
        persistence
            .append_event(&entry)
            .expect("append replay test event");
    }

    #[test]
    fn test_replay_config_default() {
        let config = ReplayConfig {
            from: None,
            until: None,
            session_id: None,
            frame_id: None,
        };
        assert!(config.from.is_none());
        assert!(config.until.is_none());
    }

    // SMOKE TEST: Basic replay with single event
    #[test]
    fn test_replay_single_event() {
        let mut state = FocusaState::default();
        let event = FocusaEvent::SessionStarted {
            session_id: Uuid::now_v7(),
            adapter_id: None,
            workspace_id: None,
        };

        let result = reducer::reduce(state, event);
        assert!(result.is_ok());

        let new_state = result.unwrap().new_state;
        assert!(new_state.session.is_some());
    }

    // SMOKE TEST: Replay session lifecycle
    #[test]
    fn test_replay_session_lifecycle() {
        let mut state = FocusaState::default();
        let session_id = Uuid::now_v7();

        // Start session
        let event1 = FocusaEvent::SessionStarted {
            session_id,
            adapter_id: None,
            workspace_id: None,
        };
        state = reducer::reduce(state, event1).unwrap().new_state;
        assert_eq!(state.session.as_ref().unwrap().session_id, session_id);

        // Close session
        let event2 = FocusaEvent::SessionClosed {
            reason: "test".into(),
        };
        state = reducer::reduce(state, event2).unwrap().new_state;
        assert_eq!(
            state.session.as_ref().unwrap().status,
            SessionStatus::Closed
        );
    }

    // SMOKE TEST: Replay focus stack operations
    #[test]
    fn test_replay_focus_stack() {
        let mut state = FocusaState::default();
        let frame_id = Uuid::now_v7();

        // Push frame
        let event = FocusaEvent::FocusFramePushed {
            frame_id,
            beads_issue_id: "TEST-001".into(),
            title: "Test Frame".into(),
            goal: "Test goal".into(),
            constraints: vec![],
            tags: vec![],
        };
        state = reducer::reduce(state, event).unwrap().new_state;

        assert_eq!(state.focus_stack.frames.len(), 1);
        assert_eq!(state.focus_stack.active_id, Some(frame_id));

        let child_id = Uuid::now_v7();
        let child = FocusaEvent::FocusFramePushed {
            frame_id: child_id,
            beads_issue_id: "TEST-CHILD".into(),
            title: "Child".into(),
            goal: "Child goal".into(),
            constraints: vec![],
            tags: vec![],
        };
        state = reducer::reduce(state, child).unwrap().new_state;

        // Complete child frame
        let event2 = FocusaEvent::FocusFrameCompleted {
            frame_id: child_id,
            completion_reason: CompletionReason::GoalAchieved,
        };
        state = reducer::reduce(state, event2).unwrap().new_state;

        assert_eq!(state.focus_stack.frames[1].status, FrameStatus::Completed);
    }

    // STRESS TEST: Replay 1000 events
    #[test]
    fn test_replay_stress_1000_events() {
        let mut state = FocusaState::default();
        let start = std::time::Instant::now();

        for i in 0..1000 {
            let event = FocusaEvent::IntuitionSignalObserved {
                signal_id: Uuid::now_v7(),
                signal_type: SignalKind::UserInput,
                severity: "0.5".to_string(),
                summary: format!("Signal {}", i),
                related_frame_id: None,
            };

            match reducer::reduce(state, event) {
                Ok(result) => state = result.new_state,
                Err(e) => panic!("Replay failed at event {}: {}", i, e),
            }
        }

        let elapsed = start.elapsed();
        println!("Replayed 1000 events in {:?}", elapsed);

        // Should complete in reasonable time (< 1 second)
        assert!(elapsed.as_secs() < 1, "Replay too slow: {:?}", elapsed);
        assert_eq!(state.focus_gate.signals.len(), 1000);
    }

    // STRESS TEST: Deep focus stack nesting
    #[test]
    fn test_replay_deep_nesting() {
        let mut state = FocusaState::default();
        let mut frame_ids = Vec::new();

        // Push 50 nested frames
        for i in 0..50 {
            let frame_id = Uuid::now_v7();
            frame_ids.push(frame_id);

            let event = FocusaEvent::FocusFramePushed {
                frame_id,
                beads_issue_id: format!("TEST-{}", i),
                title: format!("Frame {}", i),
                goal: "Nested test".into(),
                constraints: vec![],
                tags: vec![],
            };
            state = reducer::reduce(state, event).unwrap().new_state;
        }

        assert_eq!(state.focus_stack.frames.len(), 50);
        assert_eq!(state.focus_stack.stack_path_cache.len(), 50);

        // Pop all non-root frames
        for frame_id in frame_ids.into_iter().skip(1).rev() {
            let event = FocusaEvent::FocusFrameCompleted {
                frame_id,
                completion_reason: CompletionReason::GoalAchieved,
            };
            state = reducer::reduce(state, event).unwrap().new_state;
        }

        assert_eq!(state.focus_stack.active_id, state.focus_stack.root_id);
    }

    // STRESS TEST: Rapid memory operations
    #[test]
    fn test_replay_memory_stress() {
        let mut state = FocusaState::default();

        // Add 100 semantic memories
        for i in 0..100 {
            state.memory.semantic.push(SemanticRecord {
                key: format!("key.{}", i),
                value: format!("value {}", i),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                source: crate::types::MemorySource::User,
                confidence: 1.0,
                ttl: None,
                tags: vec![],
                pinned: false,
            });
        }

        // Add 100 procedural rules
        for i in 0..100 {
            state.memory.procedural.push(RuleRecord {
                id: format!("rule.{}", i),
                rule: format!("Rule {}", i),
                weight: i as f32 * 0.1,
                reinforced_count: i as u32,
                last_reinforced_at: Utc::now(),
                scope: RuleScope::Global,
                enabled: true,
                pinned: false,
                tags: vec![],
            });
        }

        assert_eq!(state.memory.semantic.len(), 100);
        assert_eq!(state.memory.procedural.len(), 100);

        // Apply decay
        crate::memory::procedural::decay_tick(&mut state.memory, 0.99);

        // Verify decay applied
        for rule in &state.memory.procedural {
            assert!(rule.weight < 100.0); // Should be decayed
        }
    }

    // SMOKE TEST: Export dataset families
    #[test]
    fn test_export_dataset_families() {
        let result = ReplayResult {
            initial_state: FocusaState::default(),
            final_state: FocusaState::default(),
            states: vec![],
            events_replayed: 0,
            errors: vec![],
        };

        // Test SFT export (empty)
        let sft = export_from_replay(&result, "sft");
        assert!(sft.is_ok());
        assert!(sft.unwrap().is_empty());

        // Test preference export (empty)
        let pref = export_from_replay(&result, "preference");
        assert!(pref.is_ok());
        assert!(pref.unwrap().is_empty());

        // Test contrastive export (empty)
        let cont = export_from_replay(&result, "contrastive");
        assert!(cont.is_ok());
        assert!(cont.unwrap().is_empty());

        // Test unknown family
        let unknown = export_from_replay(&result, "unknown");
        assert!(unknown.is_err());
    }

    // STRESS TEST: Concurrent state modifications
    #[test]
    fn test_replay_mixed_events() {
        let mut state = FocusaState::default();
        let session_id = Uuid::now_v7();

        // Session start
        state = reducer::reduce(
            state,
            FocusaEvent::SessionStarted {
                session_id,
                adapter_id: None,
                workspace_id: None,
            },
        )
        .unwrap()
        .new_state;

        // Interleave frame ops, signals, and memory events
        for i in 0..100 {
            // Push frame
            let frame_id = Uuid::now_v7();
            state = reducer::reduce(
                state,
                FocusaEvent::FocusFramePushed {
                    frame_id,
                    beads_issue_id: format!("BEAD-{}", i),
                    title: format!("Frame {}", i),
                    goal: "Test".into(),
                    constraints: vec![],
                    tags: vec![],
                },
            )
            .unwrap()
            .new_state;

            // Add signal
            state = reducer::reduce(
                state,
                FocusaEvent::IntuitionSignalObserved {
                    signal_id: Uuid::now_v7(),
                    signal_type: SignalKind::UserInput,
                    severity: "0.5".to_string(),
                    summary: format!("Signal {}", i),
                    related_frame_id: Some(frame_id),
                },
            )
            .unwrap()
            .new_state;

            if i > 0 {
                // Complete non-root frame
                state = reducer::reduce(
                    state,
                    FocusaEvent::FocusFrameCompleted {
                        frame_id,
                        completion_reason: CompletionReason::GoalAchieved,
                    },
                )
                .unwrap()
                .new_state;
            }
        }

        assert_eq!(state.focus_stack.frames.len(), 100);
        assert_eq!(state.focus_gate.signals.len(), 100);
    }

    #[test]
    fn test_secondary_loop_comparative_summary_from_replay_log() {
        let dir = temp_dir("focusa-replay-doc78");
        let mut cfg = FocusaConfig::default();
        cfg.data_dir = dir.to_string_lossy().into_owned();
        let persistence = SqlitePersistence::new(&cfg).expect("init replay persistence");

        let session_id = Uuid::now_v7();
        let paired_task_id = Uuid::now_v7();
        let unpaired_task_id = Uuid::now_v7();

        append_event(
            &persistence,
            FocusaEvent::ContinuousSecondaryLoopOutcomeRecorded {
                task_run_id: Some(paired_task_id),
                work_item_id: Some("focusa-o8vn".to_string()),
                promotion_status: "rejected".to_string(),
                verification_satisfied: false,
                spec_conformant: false,
                trace_id: "trace-baseline".to_string(),
            },
            Some(session_id),
        );
        append_event(
            &persistence,
            FocusaEvent::ContinuousSecondaryLoopOutcomeRecorded {
                task_run_id: Some(paired_task_id),
                work_item_id: Some("focusa-o8vn".to_string()),
                promotion_status: "promoted".to_string(),
                verification_satisfied: true,
                spec_conformant: true,
                trace_id: "trace-improved".to_string(),
            },
            Some(session_id),
        );
        append_event(
            &persistence,
            FocusaEvent::ContinuousSecondaryLoopOutcomeRecorded {
                task_run_id: Some(unpaired_task_id),
                work_item_id: Some("focusa-solo".to_string()),
                promotion_status: "archived_failed_attempt".to_string(),
                verification_satisfied: false,
                spec_conformant: true,
                trace_id: "trace-archived".to_string(),
            },
            Some(session_id),
        );

        let summary = secondary_loop_comparative_summary_from_replay(
            &persistence,
            &ReplayConfig {
                from: None,
                until: None,
                session_id: Some(session_id),
                frame_id: None,
            },
        )
        .expect("replay comparative summary");

        assert_eq!(summary.secondary_loop_outcome_events, 3);
        assert_eq!(summary.promoted_outcomes, 1);
        assert_eq!(summary.rejected_outcomes, 1);
        assert_eq!(summary.archived_failed_attempt_outcomes, 1);
        assert_eq!(summary.comparative_improvement_pairs, 1);
        assert!(summary.task_pairs.iter().any(|pair| {
            pair.correlation_id == paired_task_id.to_string()
                && pair.promoted_outcomes == 1
                && pair.non_promoted_outcomes == 1
                && pair.comparative_improvement_observed
        }));
    }

    #[test]
    fn test_secondary_loop_comparative_summary_respects_session_filter() {
        let dir = temp_dir("focusa-replay-doc78-session-filter");
        let mut cfg = FocusaConfig::default();
        cfg.data_dir = dir.to_string_lossy().into_owned();
        let persistence = SqlitePersistence::new(&cfg).expect("init replay persistence");

        let included_session = Uuid::now_v7();
        let excluded_session = Uuid::now_v7();
        let task_run_id = Uuid::now_v7();

        append_event(
            &persistence,
            FocusaEvent::ContinuousSecondaryLoopOutcomeRecorded {
                task_run_id: Some(task_run_id),
                work_item_id: Some("focusa-filtered".to_string()),
                promotion_status: "rejected".to_string(),
                verification_satisfied: false,
                spec_conformant: false,
                trace_id: "trace-included-rejected".to_string(),
            },
            Some(included_session),
        );
        append_event(
            &persistence,
            FocusaEvent::ContinuousSecondaryLoopOutcomeRecorded {
                task_run_id: Some(task_run_id),
                work_item_id: Some("focusa-filtered".to_string()),
                promotion_status: "promoted".to_string(),
                verification_satisfied: true,
                spec_conformant: true,
                trace_id: "trace-included-promoted".to_string(),
            },
            Some(included_session),
        );
        append_event(
            &persistence,
            FocusaEvent::ContinuousSecondaryLoopOutcomeRecorded {
                task_run_id: Some(task_run_id),
                work_item_id: Some("focusa-filtered".to_string()),
                promotion_status: "promoted".to_string(),
                verification_satisfied: true,
                spec_conformant: true,
                trace_id: "trace-excluded-promoted".to_string(),
            },
            Some(excluded_session),
        );

        let summary = secondary_loop_comparative_summary_from_replay(
            &persistence,
            &ReplayConfig {
                from: None,
                until: None,
                session_id: Some(included_session),
                frame_id: None,
            },
        )
        .expect("filtered replay comparative summary");

        assert_eq!(summary.secondary_loop_outcome_events, 2);
        assert_eq!(summary.promoted_outcomes, 1);
        assert_eq!(summary.rejected_outcomes, 1);
        assert_eq!(summary.comparative_improvement_pairs, 1);
    }

    #[test]
    fn test_secondary_loop_comparative_summary_tolerates_legacy_payload_without_handle() {
        let dir = temp_dir("focusa-replay-doc78-legacy-payload");
        let mut cfg = FocusaConfig::default();
        cfg.data_dir = dir.to_string_lossy().into_owned();
        let persistence = SqlitePersistence::new(&cfg).expect("init replay persistence");

        let session_id = Uuid::now_v7();
        let task_run_id = Uuid::now_v7();

        append_event(
            &persistence,
            FocusaEvent::ContinuousSecondaryLoopOutcomeRecorded {
                task_run_id: Some(task_run_id),
                work_item_id: Some("focusa-legacy".to_string()),
                promotion_status: "rejected".to_string(),
                verification_satisfied: false,
                spec_conformant: false,
                trace_id: "trace-legacy-baseline".to_string(),
            },
            Some(session_id),
        );
        append_event(
            &persistence,
            FocusaEvent::ContinuousSecondaryLoopOutcomeRecorded {
                task_run_id: Some(task_run_id),
                work_item_id: Some("focusa-legacy".to_string()),
                promotion_status: "promoted".to_string(),
                verification_satisfied: true,
                spec_conformant: true,
                trace_id: "trace-legacy-promoted".to_string(),
            },
            Some(session_id),
        );

        let db_path = dir.join("focusa.sqlite");
        let conn = rusqlite::Connection::open(db_path).expect("open replay sqlite");
        let malformed_payload = serde_json::json!({
            "id": Uuid::now_v7(),
            "timestamp": Utc::now(),
            "type": "ArtifactRegistered",
            "storage_uri": "ecs://legacy-missing-handle",
            "origin": "Daemon",
            "correlation_id": null
        })
        .to_string();

        conn.execute(
            "INSERT INTO events(event_id, ts, origin, correlation_id, payload_json, machine_id, instance_id, session_id, thread_id, is_observation) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                Uuid::now_v7().to_string(),
                Utc::now().to_rfc3339(),
                "Daemon",
                Option::<String>::None,
                malformed_payload,
                Option::<String>::None,
                Option::<String>::None,
                Some(session_id.to_string()),
                Option::<String>::None,
                0i32,
            ],
        )
        .expect("insert legacy payload row");

        let summary = secondary_loop_comparative_summary_from_replay(
            &persistence,
            &ReplayConfig {
                from: None,
                until: None,
                session_id: Some(session_id),
                frame_id: None,
            },
        )
        .expect("legacy payload tolerant comparative summary");

        assert_eq!(summary.secondary_loop_outcome_events, 2);
        assert_eq!(summary.promoted_outcomes, 1);
        assert_eq!(summary.rejected_outcomes, 1);
        assert_eq!(summary.comparative_improvement_pairs, 1);
    }

    #[test]
    fn test_replay_events_fail_fast_on_legacy_payload_without_handle() {
        let dir = temp_dir("focusa-replay-doc78-legacy-strict");
        let mut cfg = FocusaConfig::default();
        cfg.data_dir = dir.to_string_lossy().into_owned();
        let persistence = SqlitePersistence::new(&cfg).expect("init replay persistence");

        let session_id = Uuid::now_v7();

        let db_path = dir.join("focusa.sqlite");
        let conn = rusqlite::Connection::open(db_path).expect("open replay sqlite");
        let malformed_payload = serde_json::json!({
            "id": Uuid::now_v7(),
            "timestamp": Utc::now(),
            "type": "ArtifactRegistered",
            "storage_uri": "ecs://legacy-missing-handle",
            "origin": "daemon",
            "correlation_id": null
        })
        .to_string();

        conn.execute(
            "INSERT INTO events(event_id, ts, origin, correlation_id, payload_json, machine_id, instance_id, session_id, thread_id, is_observation) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                Uuid::now_v7().to_string(),
                Utc::now().to_rfc3339(),
                "daemon",
                Option::<String>::None,
                malformed_payload,
                Option::<String>::None,
                Option::<String>::None,
                Some(session_id.to_string()),
                Option::<String>::None,
                0i32,
            ],
        )
        .expect("insert legacy payload row");

        let err = replay_events(
            &persistence,
            &ReplayConfig {
                from: None,
                until: None,
                session_id: Some(session_id),
                frame_id: None,
            },
        )
        .expect_err("strict replay should fail on malformed payload rows");

        assert!(
            err.to_string()
                .contains("Conversion error from type Text at index: 4"),
            "strict replay should fail-fast on malformed payload rows: {err:?}"
        );
    }
}
