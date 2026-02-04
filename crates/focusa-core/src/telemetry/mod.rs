//! Cognitive Telemetry — docs/29-30-31-32
//!
//! Privacy-by-design, structured telemetry.
//! Every event has schema_version, session context, CLT linkage.
//! Aggregation: per-task token tracking, cost accounting, efficiency metrics.
//! Storage: append to ~/.focusa/telemetry/events.jsonl

use crate::types::*;
use chrono::Utc;
use uuid::Uuid;

/// Record a telemetry event.
pub fn record(state: &mut TelemetryState, event: TelemetryEvent) {
    // Update aggregate stats.
    if event.event_type == TelemetryEventType::ModelTokens {
        if let Some(tokens) = event.payload.get("prompt_tokens").and_then(|v| v.as_u64()) {
            state.total_prompt_tokens += tokens;
        }
        if let Some(tokens) = event.payload.get("completion_tokens").and_then(|v| v.as_u64()) {
            state.total_completion_tokens += tokens;
        }
    }
    state.total_events += 1;
}

/// Create a model-tokens telemetry event.
pub fn model_tokens_event(
    session_id: Option<SessionId>,
    agent_id: Option<&str>,
    model_id: Option<&str>,
    prompt_tokens: u64,
    completion_tokens: u64,
    clt_id: Option<&str>,
    focus_frame_id: Option<FrameId>,
) -> TelemetryEvent {
    TelemetryEvent {
        event_id: Uuid::now_v7(),
        event_type: TelemetryEventType::ModelTokens,
        timestamp: Utc::now(),
        session_id,
        agent_id: agent_id.map(String::from),
        model_id: model_id.map(String::from),
        clt_id: clt_id.map(String::from),
        focus_frame_id,
        payload: serde_json::json!({
            "prompt_tokens": prompt_tokens,
            "completion_tokens": completion_tokens,
        }),
        schema_version: "1.0.0".into(),
    }
}

/// Create a focus-transition telemetry event.
pub fn focus_transition_event(
    session_id: Option<SessionId>,
    from_frame: Option<FrameId>,
    to_frame: Option<FrameId>,
) -> TelemetryEvent {
    TelemetryEvent {
        event_id: Uuid::now_v7(),
        event_type: TelemetryEventType::FocusTransition,
        timestamp: Utc::now(),
        session_id,
        agent_id: None,
        model_id: None,
        clt_id: None,
        focus_frame_id: to_frame,
        payload: serde_json::json!({
            "from_frame": from_frame,
            "to_frame": to_frame,
        }),
        schema_version: "1.0.0".into(),
    }
}

/// Update per-task token tracking.
pub fn update_task_tokens(
    state: &mut TelemetryState,
    task_id: &str,
    prompt_tokens: u64,
    completion_tokens: u64,
) {
    if let Some(entry) = state.tokens_per_task.iter_mut().find(|e| e.task_id == task_id) {
        entry.prompt_tokens += prompt_tokens;
        entry.completion_tokens += completion_tokens;
        entry.turns += 1;
    } else {
        state.tokens_per_task.push(TokensPerTask {
            task_id: task_id.into(),
            prompt_tokens,
            completion_tokens,
            turns: 1,
        });
    }
}

/// Get total cost estimate (simple model).
pub fn estimate_cost(state: &TelemetryState, cost_per_1k_prompt: f64, cost_per_1k_completion: f64) -> f64 {
    (state.total_prompt_tokens as f64 / 1000.0) * cost_per_1k_prompt
        + (state.total_completion_tokens as f64 / 1000.0) * cost_per_1k_completion
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_tokens() {
        let mut state = TelemetryState::default();
        let event = model_tokens_event(None, None, Some("claude-3.5"), 500, 200, None, None);
        record(&mut state, event);
        assert_eq!(state.total_prompt_tokens, 500);
        assert_eq!(state.total_completion_tokens, 200);
        assert_eq!(state.total_events, 1);
    }

    #[test]
    fn test_task_tracking() {
        let mut state = TelemetryState::default();
        update_task_tokens(&mut state, "task-1", 100, 50);
        update_task_tokens(&mut state, "task-1", 200, 100);
        assert_eq!(state.tokens_per_task.len(), 1);
        assert_eq!(state.tokens_per_task[0].turns, 2);
        assert_eq!(state.tokens_per_task[0].prompt_tokens, 300);
    }
}
