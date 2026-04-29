//! Spec93 non-Pi agent awareness card API.

use crate::server::AppState;
use axum::extract::{Query, State};
use axum::{Json, Router, routing::get};
use focusa_core::types::{FocusaState, WorkpointRecord};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

#[derive(Debug, Deserialize, Default)]
pub struct AwarenessCardQuery {
    pub adapter_id: Option<String>,
    pub workspace_id: Option<String>,
    pub agent_id: Option<String>,
    pub operator_id: Option<String>,
    pub session_id: Option<String>,
    pub project_root: Option<String>,
}

fn clean(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
}

fn active_workpoint(state: &FocusaState) -> Option<&WorkpointRecord> {
    state.workpoint.active_workpoint_id.and_then(|id| {
        state
            .workpoint
            .records
            .iter()
            .find(|record| record.workpoint_id == id)
    })
}

fn scoped_workpoint<'a>(
    focusa: &'a FocusaState,
    query: &AwarenessCardQuery,
) -> Option<&'a WorkpointRecord> {
    let active = active_workpoint(focusa)?;
    if let Some(expected) = clean(query.session_id.as_deref())
        && active.session_id.as_deref().map(str::trim) != Some(expected.as_str())
    {
        return None;
    }
    if let Some(expected) = clean(query.project_root.as_deref())
        && active.project_root.as_deref().map(str::trim) != Some(expected.as_str())
    {
        return None;
    }
    Some(active)
}

fn render_card(query: &AwarenessCardQuery, record: Option<&WorkpointRecord>) -> String {
    let adapter = clean(query.adapter_id.as_deref()).unwrap_or_else(|| "non-pi-agent".to_string());
    let workspace =
        clean(query.workspace_id.as_deref()).unwrap_or_else(|| "unknown-workspace".to_string());
    let agent = clean(query.agent_id.as_deref()).unwrap_or_else(|| adapter.clone());
    let operator =
        clean(query.operator_id.as_deref()).unwrap_or_else(|| "unknown-operator".to_string());
    let mission = record
        .and_then(|r| r.mission.as_deref())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("Use latest operator instruction and harness-local project/workspace context.");
    let next = record
        .and_then(|r| r.next_slice.as_deref())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("Fetch or create a scoped Workpoint before risky continuation.");
    let project_root = clean(query.project_root.as_deref())
        .or_else(|| record.and_then(|r| r.project_root.clone()))
        .unwrap_or_else(|| {
            "unbound; derive from harness workspace before checkpointing".to_string()
        });
    let session = clean(query.session_id.as_deref())
        .or_else(|| record.and_then(|r| r.session_id.clone()))
        .unwrap_or_else(|| "unbound; use harness session id if available".to_string());
    let canonical = record.map(|r| r.canonical).unwrap_or(false);
    [
        "# Focusa Utility Card".to_string(),
        format!("Status: available{}", if canonical { " / scoped Workpoint found" } else { " / no scoped Workpoint found" }),
        format!("Agent: adapter={adapter} workspace={workspace} agent={agent} operator={operator}"),
        format!("Mission: {mission}"),
        format!("Next anchor: {next}"),
        format!("Scope: project_root={project_root}; session_id={session}"),
        String::new(),
        "Use Focusa as agent working memory and governance:".to_string(),
        "- First when uncertain/degraded: call /v1/doctor or run `focusa doctor --json`.".to_string(),
        "- Before compaction/model switch/fork/risky continuation: checkpoint a scoped Workpoint.".to_string(),
        "- After compaction/reload/resume: fetch Workpoint resume; do not trust transcript tail over Workpoint.".to_string(),
        "- After proof/tests/API/file evidence: capture or link evidence to the active Workpoint.".to_string(),
        "- Before risky or uncertain next action: record a prediction; after outcome: evaluate it.".to_string(),
        "- If Focusa is unavailable, mark cognition_degraded=true and continue only with explicit fallback context.".to_string(),
        "Operator steering always wins; Focusa guides, preserves, and audits.".to_string(),
    ].join("\n")
}

async fn card(
    State(state): State<Arc<AppState>>,
    Query(query): Query<AwarenessCardQuery>,
) -> Json<Value> {
    let focusa = state.focusa.read().await;
    let record = scoped_workpoint(&focusa, &query);
    let rendered_card = render_card(&query, record);
    Json(json!({
        "status": "completed",
        "canonical": true,
        "surface": "focusa_awareness_card",
        "adapter_id": query.adapter_id,
        "workspace_id": query.workspace_id,
        "agent_id": query.agent_id,
        "operator_id": query.operator_id,
        "session_id": query.session_id,
        "project_root": query.project_root,
        "workpoint_id": record.map(|r| r.workpoint_id),
        "workpoint_canonical": record.map(|r| r.canonical).unwrap_or(false),
        "rendered_card": rendered_card,
        "next_step_hint": "inject rendered_card into the non-Pi agent system/developer prompt before reasoning"
    }))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/v1/awareness/card", get(card))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn awareness_card_mentions_required_agent_rules() {
        let card = render_card(
            &AwarenessCardQuery {
                adapter_id: Some("openclaw".into()),
                workspace_id: Some("wirebot".into()),
                agent_id: Some("wirebot".into()),
                operator_id: Some("verious.smith".into()),
                session_id: Some("session-1".into()),
                project_root: Some("/data/wirebot/users/verious".into()),
            },
            None,
        );
        for needle in [
            "Focusa Utility Card",
            "adapter=openclaw",
            "workspace=wirebot",
            "/v1/doctor",
            "checkpoint a scoped Workpoint",
            "fetch Workpoint resume",
            "capture or link evidence",
            "record a prediction",
            "cognition_degraded=true",
            "Operator steering always wins",
        ] {
            assert!(card.contains(needle), "missing {needle}: {card}");
        }
    }
}
