//! Beginner Mode decision tree for Mission Deck (Spec 117 §10).

use crate::app::App;
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeginnerModeState {
    Disconnected,
    Unbound,
    NoWorkpoint,
    NoEvidence,
    Resumable,
}

impl BeginnerModeState {
    pub fn id(self) -> &'static str {
        match self {
            BeginnerModeState::Disconnected => "disconnected",
            BeginnerModeState::Unbound => "unbound",
            BeginnerModeState::NoWorkpoint => "no_workpoint",
            BeginnerModeState::NoEvidence => "no_evidence",
            BeginnerModeState::Resumable => "resumable",
        }
    }

    pub fn explanation(self) -> &'static str {
        match self {
            BeginnerModeState::Disconnected => {
                "The Focusa background service is not running. Start it, then reopen Mission Deck."
            }
            BeginnerModeState::Unbound => {
                "Focusa does not yet know what project this mission belongs to. Bind this folder first."
            }
            BeginnerModeState::NoWorkpoint => {
                "This project is known, but no saved mission exists yet. Create a Workpoint before changing files."
            }
            BeginnerModeState::NoEvidence => {
                "The agent says work is in progress, but Focusa has no proof yet. Attach a test, file, screenshot, or command output."
            }
            BeginnerModeState::Resumable => {
                "This mission has enough project, Workpoint, and proof context to resume safely."
            }
        }
    }

    pub fn primary_action(self) -> &'static str {
        match self {
            BeginnerModeState::Disconnected => "focusa start",
            BeginnerModeState::Unbound => "focusa init --quickstart",
            BeginnerModeState::NoWorkpoint => "focusa workpoint checkpoint",
            BeginnerModeState::NoEvidence => "focusa workpoint checkpoint --evidence-ref <proof>",
            BeginnerModeState::Resumable => "focusa workpoint resume",
        }
    }
}

pub fn assess(app: &App) -> BeginnerModeState {
    if !app.connected {
        return BeginnerModeState::Disconnected;
    }
    if !has_verified_project(app) {
        return BeginnerModeState::Unbound;
    }
    let Some(workpoint) = app
        .extra_data
        .get("workpoint_resume")
        .and_then(|value| value.as_ref())
    else {
        return BeginnerModeState::NoWorkpoint;
    };
    if !has_workpoint(workpoint) {
        return BeginnerModeState::NoWorkpoint;
    }
    if !has_evidence(workpoint) {
        return BeginnerModeState::NoEvidence;
    }
    BeginnerModeState::Resumable
}

fn has_verified_project(app: &App) -> bool {
    let Some(project) = app
        .extra_data
        .get("project_identity")
        .and_then(|value| value.as_ref())
    else {
        return false;
    };
    let status = project
        .get("status")
        .or_else(|| project.pointer("/fields/status"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    let confidence = project
        .get("confidence")
        .and_then(Value::as_str)
        .unwrap_or_default();
    matches!(status, "verified" | "completed") || confidence == "high"
}

fn has_workpoint(value: &Value) -> bool {
    let status = value
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let canonical = value
        .get("canonical")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let has_id = value
        .get("id")
        .or_else(|| value.get("workpoint_id"))
        .and_then(Value::as_str)
        .map(|id| !id.is_empty() && id != "none")
        .unwrap_or(false);
    matches!(status, "completed" | "accepted") && (canonical || has_id)
}

fn has_evidence(value: &Value) -> bool {
    contains_non_empty_array(value, "evidence_refs")
        || contains_non_empty_array(value, "verified_evidence")
        || value
            .pointer("/packet/verified_evidence")
            .and_then(Value::as_array)
            .map(|items| !items.is_empty())
            .unwrap_or(false)
}

fn contains_non_empty_array(value: &Value, key: &str) -> bool {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(|items| !items.is_empty())
        .unwrap_or(false)
}

pub const DECISION_TREE: &[&str] = &[
    "disconnected",
    "unbound",
    "no_workpoint",
    "no_evidence",
    "resumable",
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;

    #[test]
    fn disconnected_wins_first() {
        let app = App::new("http://127.0.0.1:8787".into());
        assert_eq!(assess(&app), BeginnerModeState::Disconnected);
    }

    #[test]
    fn verified_project_without_workpoint_is_no_workpoint() {
        let mut app = App::new("http://127.0.0.1:8787".into());
        app.connected = true;
        app.extra_data.insert(
            "project_identity".into(),
            Some(serde_json::json!({"status":"verified","confidence":"high"})),
        );
        assert_eq!(assess(&app), BeginnerModeState::NoWorkpoint);
    }

    #[test]
    fn complete_workpoint_without_evidence_is_no_evidence() {
        let mut app = App::new("http://127.0.0.1:8787".into());
        app.connected = true;
        app.extra_data.insert(
            "project_identity".into(),
            Some(serde_json::json!({"status":"verified","confidence":"high"})),
        );
        app.extra_data.insert(
            "workpoint_resume".into(),
            Some(serde_json::json!({"status":"completed","canonical":true,"id":"wp1"})),
        );
        assert_eq!(assess(&app), BeginnerModeState::NoEvidence);
    }

    #[test]
    fn complete_workpoint_with_evidence_is_resumable() {
        let mut app = App::new("http://127.0.0.1:8787".into());
        app.connected = true;
        app.extra_data.insert(
            "project_identity".into(),
            Some(serde_json::json!({"status":"verified","confidence":"high"})),
        );
        app.extra_data.insert(
            "workpoint_resume".into(),
            Some(serde_json::json!({
                "status":"completed",
                "canonical":true,
                "id":"wp1",
                "verified_evidence":["test"]
            })),
        );
        assert_eq!(assess(&app), BeginnerModeState::Resumable);
    }
}
