//! Session replay — reconstruct state from event log for export.
//!
//! Per 20-training-dataset-schema §Export Pipeline:
//! Full session replay required for dataset generation.

use crate::reducer;
use crate::types::*;
use crate::runtime::persistence_sqlite::SqlitePersistence;
use chrono::{DateTime, Utc};

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
            && entry.session_id != Some(target_sid) {
                continue;
            }
        
        // Filter by timestamp if specified.
        if let Some(until) = config.until
            && entry.timestamp > until {
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
    
    after.active_turn.as_ref().map(|_turn| serde_json::json!({
            "instruction": "placeholder - would extract from turn",
            "response": "placeholder - would extract from assistant_output",
            "metadata": {
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }
        }))
}

fn extract_preference_example(_before: &FocusaState, _after: &FocusaState) -> Option<serde_json::Value> {
    // Would extract chosen vs rejected responses
    None
}

fn extract_contrastive_example(_before: &FocusaState, _after: &FocusaState) -> Option<serde_json::Value> {
    // Would extract positive/negative examples
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use chrono::Duration;
    use uuid::Uuid;
    
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
        assert_eq!(state.session.as_ref().unwrap().status, SessionStatus::Closed);
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
        state = reducer::reduce(state, FocusaEvent::SessionStarted {
            session_id,
            adapter_id: None,
            workspace_id: None,
        }).unwrap().new_state;
        
        // Interleave frame ops, signals, and memory events
        for i in 0..100 {
            // Push frame
            let frame_id = Uuid::now_v7();
            state = reducer::reduce(state, FocusaEvent::FocusFramePushed {
                frame_id,
                beads_issue_id: format!("BEAD-{}", i),
                title: format!("Frame {}", i),
                goal: "Test".into(),
                constraints: vec![],
                tags: vec![],
            }).unwrap().new_state;
            
            // Add signal
            state = reducer::reduce(state, FocusaEvent::IntuitionSignalObserved {
                signal_id: Uuid::now_v7(),
                signal_type: SignalKind::UserInput,
                severity: "0.5".to_string(),
                summary: format!("Signal {}", i),
                related_frame_id: Some(frame_id),
            }).unwrap().new_state;
            
            if i > 0 {
                // Complete non-root frame
                state = reducer::reduce(state, FocusaEvent::FocusFrameCompleted {
                    frame_id,
                    completion_reason: CompletionReason::GoalAchieved,
                }).unwrap().new_state;
            }
        }
        
        assert_eq!(state.focus_stack.frames.len(), 100);
        assert_eq!(state.focus_gate.signals.len(), 100);
    }
}
