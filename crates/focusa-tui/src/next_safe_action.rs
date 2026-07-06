//! Next Safe Action recommender for Mission Deck (Spec 117 §6.5).

use crate::app::App;
use crate::beginner_mode::{self, BeginnerModeState};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryTool {
    pub id: &'static str,
    pub label: &'static str,
    pub command: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NextSafeAction {
    pub id: &'static str,
    pub label: &'static str,
    pub command: &'static str,
    pub why: &'static str,
    pub authority_posture: &'static str,
    pub walkthrough_context: &'static str,
    /// Up to 3 bounded recovery tools (Spec 119 §7.6 + §19).
    pub recovery_tools: &'static [RecoveryTool],
}

pub fn recommend(app: &App) -> NextSafeAction {
    let beginner_state = beginner_mode::assess(app);
    let authority = authority_posture(app);
    if authority == "blocked" {
        return NextSafeAction {
            id: "review_scope_before_acting",
            label: "Review or rebind scope before changing files",
            command: "focusa workpoint resume",
            why: "The saved mission may not match the current project/request scope.",
            authority_posture: authority,
            walkthrough_context: beginner_state.id(),
            recovery_tools: &[
                RecoveryTool {
                    id: "0",
                    label: "doctor",
                    command: "focusa doctor --scope host",
                },
                RecoveryTool {
                    id: "1",
                    label: "resume",
                    command: "focusa workpoint resume",
                },
                RecoveryTool {
                    id: "2",
                    label: "walkthrough",
                    command: "focusa walkthrough show --walkthrough first-mission",
                },
            ],
        };
    }

    match beginner_state {
        BeginnerModeState::Disconnected => NextSafeAction {
            id: "start_daemon",
            label: "Start the Focusa background service",
            command: "focusa start",
            why: "Mission Deck cannot verify project, Workpoint, or evidence until the daemon responds.",
            authority_posture: authority,
            walkthrough_context: beginner_state.id(),
            recovery_tools: &[
                RecoveryTool {
                    id: "0",
                    label: "doctor",
                    command: "focusa doctor --scope host",
                },
                RecoveryTool {
                    id: "1",
                    label: "start",
                    command: "focusa start",
                },
                RecoveryTool {
                    id: "2",
                    label: "walkthrough",
                    command: "focusa walkthrough show --walkthrough first-mission",
                },
            ],
        },
        BeginnerModeState::Unbound => NextSafeAction {
            id: "bind_project",
            label: "Bind this folder as the project",
            command: "focusa init --quickstart",
            why: "Project identity is required before Focusa can trust carryover state.",
            authority_posture: authority,
            walkthrough_context: beginner_state.id(),
            recovery_tools: &[
                RecoveryTool {
                    id: "0",
                    label: "doctor",
                    command: "focusa doctor --scope project",
                },
                RecoveryTool {
                    id: "1",
                    label: "init",
                    command: "focusa init --quickstart",
                },
                RecoveryTool {
                    id: "2",
                    label: "walkthrough",
                    command: "focusa walkthrough show --walkthrough first-mission",
                },
            ],
        },
        BeginnerModeState::NoWorkpoint => NextSafeAction {
            id: "create_workpoint",
            label: "Create a Workpoint checkpoint",
            command: "focusa workpoint checkpoint",
            why: "A Workpoint is the canonical save state for mission, action, evidence, and next step.",
            authority_posture: authority,
            walkthrough_context: beginner_state.id(),
            recovery_tools: &[
                RecoveryTool {
                    id: "0",
                    label: "checkpoint",
                    command: "focusa workpoint checkpoint",
                },
                RecoveryTool {
                    id: "1",
                    label: "walkthrough",
                    command: "focusa walkthrough show --walkthrough first-mission",
                },
                RecoveryTool {
                    id: "2",
                    label: "doctor",
                    command: "focusa doctor --scope host",
                },
            ],
        },
        BeginnerModeState::NoEvidence => NextSafeAction {
            id: "attach_evidence",
            label: "Attach one proof item",
            command: "focusa workpoint checkpoint --evidence-ref <proof>",
            why: "The next agent needs proof, not just a claim that work is complete.",
            authority_posture: authority,
            walkthrough_context: beginner_state.id(),
            recovery_tools: &[
                RecoveryTool {
                    id: "0",
                    label: "checkpoint",
                    command: "focusa workpoint checkpoint --evidence-ref <proof>",
                },
                RecoveryTool {
                    id: "1",
                    label: "capture",
                    command: "focusa evidence capture",
                },
                RecoveryTool {
                    id: "2",
                    label: "walkthrough",
                    command: "focusa walkthrough show --walkthrough no-proof-no-done",
                },
            ],
        },
        BeginnerModeState::Resumable => NextSafeAction {
            id: "resume_mission",
            label: "Resume the mission",
            command: "focusa workpoint resume",
            why: "Project, Workpoint, and proof context are available enough to continue safely.",
            authority_posture: authority,
            walkthrough_context: beginner_state.id(),
            recovery_tools: &[
                RecoveryTool {
                    id: "0",
                    label: "resume",
                    command: "focusa workpoint resume",
                },
                RecoveryTool {
                    id: "1",
                    label: "context",
                    command: "focusa context cognition render",
                },
                RecoveryTool {
                    id: "2",
                    label: "walkthrough",
                    command: "focusa walkthrough show --walkthrough agent-handoff",
                },
            ],
        },
    }
}

fn authority_posture(app: &App) -> &'static str {
    let Some(workpoint) = app
        .extra_data
        .get("workpoint_resume")
        .and_then(|value| value.as_ref())
    else {
        return "advisory";
    };
    let status = workpoint
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let canonical = workpoint
        .get("canonical")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let action_authority = workpoint
        .get("action_authority_for_current_ask")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    if matches!(status, "conflict" | "scope_conflict") || !action_authority {
        "blocked"
    } else if canonical {
        "canonical"
    } else {
        "advisory"
    }
}

pub const HEADLESS_PROOF_STATES: &[&str] = &[
    "disconnected:start_daemon",
    "unbound:bind_project",
    "no_workpoint:create_workpoint",
    "no_evidence:attach_evidence",
    "resumable:resume_mission",
    "blocked:review_scope_before_acting",
];

/// Spec 119 §7.6 + §19: recovery tool list capped at 3 per next safe action.
pub const HEADLESS_PROOF_RECOVERY_TOOL_CAP: usize = 3;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;

    #[test]
    fn disconnected_recommends_start_daemon() {
        let app = App::new("http://127.0.0.1:8787".into());
        assert_eq!(recommend(&app).id, "start_daemon");
    }

    #[test]
    fn recovery_tools_are_bounded_to_three() {
        let rec = recommend(&App::new("http://127.0.0.1:8787".into()));
        assert!(rec.recovery_tools.len() <= HEADLESS_PROOF_RECOVERY_TOOL_CAP);
        assert!(!rec.recovery_tools.is_empty());
    }

    #[test]
    fn blocked_authority_wins_over_resumable() {
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
                "action_authority_for_current_ask": false,
                "verified_evidence":["proof"]
            })),
        );
        assert_eq!(recommend(&app).id, "review_scope_before_acting");
    }
}
