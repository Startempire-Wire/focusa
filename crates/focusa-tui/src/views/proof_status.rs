//! Proof Meter + Scope Badge visuals (Spec 117 §6.7 / §14.2).

use crate::app::App;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofMeter {
    pub status: &'static str,
    pub visual: &'static str,
    pub label: String,
    /// Spec 119 §30 affordance reality: practical possibility assessment,
    /// not desired outcome. One of possible|limited|unavailable.
    pub affordance_reality: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeBadge {
    pub posture: &'static str,
    pub visual: &'static str,
    pub label: String,
    /// Spec 119 §31 governing prior chain that yielded the current posture.
    pub precedence_frame: &'static str,
}

pub const PROOF_METER_STATES: &[&str] = &["none:[-----]", "linked:[##---]", "verified:[#####]"];
pub const SCOPE_BADGE_STATES: &[&str] = &["canonical", "advisory", "blocked", "unbound"];

pub const AFFORDANCE_REALITY_POSSIBLE: &str = "possible";
pub const AFFORDANCE_REALITY_LIMITED: &str = "limited";
pub const AFFORDANCE_REALITY_UNAVAILABLE: &str = "unavailable";

pub const PRECEDENCE_FRAME_PROJECT: &str = "project_identity -> workpoint -> operator";
pub const PRECEDENCE_FRAME_AUTHORITY: &str = "scope -> authority_posture -> operator";
pub const PRECEDENCE_FRAME_OPERATOR: &str = "operator_only";

pub fn proof_meter(app: &App) -> ProofMeter {
    let workpoint = app
        .extra_data
        .get("workpoint_resume")
        .and_then(|value| value.as_ref());
    let verified = evidence_count(workpoint, "verified_evidence")
        .or_else(|| evidence_count(workpoint, "/packet/verified_evidence"))
        .unwrap_or(0);
    let linked = evidence_count(workpoint, "evidence_refs").unwrap_or(0);

    if verified > 0 {
        ProofMeter {
            status: "verified",
            affordance_reality: crate::views::proof_status::AFFORDANCE_REALITY_POSSIBLE,
            visual: "[#####]",
            label: format!("verified ({verified} refs)"),
        }
    } else if linked > 0 {
        ProofMeter {
            status: "linked",
            affordance_reality: crate::views::proof_status::AFFORDANCE_REALITY_LIMITED,
            visual: "[##---]",
            label: format!("linked ({linked} refs)"),
        }
    } else {
        ProofMeter {
            status: "none",
            affordance_reality: crate::views::proof_status::AFFORDANCE_REALITY_UNAVAILABLE,
            visual: "[-----]",
            label: "no proof refs visible".to_string(),
        }
    }
}

pub fn scope_badge(app: &App) -> ScopeBadge {
    let project = app
        .extra_data
        .get("project_identity")
        .and_then(|value| value.as_ref());
    let workpoint = app
        .extra_data
        .get("workpoint_resume")
        .and_then(|value| value.as_ref());

    if project.is_none() {
        return ScopeBadge {
            posture: "unbound",
            precedence_frame: crate::views::proof_status::PRECEDENCE_FRAME_PROJECT,
            visual: "[unbound]",
            label: "project identity unavailable".to_string(),
        };
    }

    if is_blocked(workpoint) {
        ScopeBadge {
            posture: "blocked",
            precedence_frame: crate::views::proof_status::PRECEDENCE_FRAME_AUTHORITY,
            visual: "[blocked]",
            label: "scope or action authority conflict".to_string(),
        }
    } else if is_canonical(workpoint) {
        ScopeBadge {
            posture: "canonical",
            precedence_frame: crate::views::proof_status::PRECEDENCE_FRAME_AUTHORITY,
            visual: "[canonical]",
            label: "safe to act within this scope".to_string(),
        }
    } else {
        ScopeBadge {
            posture: "advisory",
            precedence_frame: crate::views::proof_status::PRECEDENCE_FRAME_OPERATOR,
            visual: "[advisory]",
            label: "review before acting".to_string(),
        }
    }
}

fn evidence_count(source: Option<&Value>, key_or_pointer: &str) -> Option<usize> {
    let value = source?;
    let array = if key_or_pointer.starts_with('/') {
        value.pointer(key_or_pointer)
    } else {
        value.get(key_or_pointer)
    }?;
    array.as_array().map(|items| items.len())
}

fn is_canonical(source: Option<&Value>) -> bool {
    source
        .and_then(|value| value.get("canonical"))
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn is_blocked(source: Option<&Value>) -> bool {
    let Some(value) = source else {
        return false;
    };
    let status = value
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let authority = value
        .get("action_authority_for_current_ask")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    matches!(status, "conflict" | "scope_conflict" | "blocked") || !authority
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;

    #[test]
    fn affordance_reality_matches_status() {
        let mut app = App::new("http://127.0.0.1:8787".into());
        app.extra_data.insert(
            "workpoint_resume".into(),
            Some(serde_json::json!({"verified_evidence":["t"]})),
        );
        let meter = proof_meter(&app);
        assert_eq!(meter.affordance_reality, AFFORDANCE_REALITY_POSSIBLE);

        app.extra_data.insert(
            "workpoint_resume".into(),
            Some(serde_json::json!({"evidence_refs":["t"]})),
        );
        let meter = proof_meter(&app);
        assert_eq!(meter.affordance_reality, AFFORDANCE_REALITY_LIMITED);
    }

    #[test]
    fn scope_badge_carries_precedence_frame() {
        let badge = scope_badge(&App::new("http://127.0.0.1:8787".into()));
        assert_eq!(badge.precedence_frame, PRECEDENCE_FRAME_PROJECT);
    }

    #[test]
    fn proof_meter_none_when_no_workpoint_evidence() {
        let app = App::new("http://127.0.0.1:8787".into());
        let meter = proof_meter(&app);
        assert_eq!(meter.status, "none");
        assert_eq!(meter.visual, "[-----]");
    }

    #[test]
    fn proof_meter_verified_when_verified_refs_exist() {
        let mut app = App::new("http://127.0.0.1:8787".into());
        app.extra_data.insert(
            "workpoint_resume".into(),
            Some(serde_json::json!({"verified_evidence":["test"]})),
        );
        let meter = proof_meter(&app);
        assert_eq!(meter.status, "verified");
        assert_eq!(meter.visual, "[#####]");
    }

    #[test]
    fn scope_badge_blocks_authority_conflict() {
        let mut app = App::new("http://127.0.0.1:8787".into());
        app.extra_data.insert(
            "project_identity".into(),
            Some(serde_json::json!({"status":"verified"})),
        );
        app.extra_data.insert(
            "workpoint_resume".into(),
            Some(serde_json::json!({"canonical":true,"action_authority_for_current_ask":false})),
        );
        let badge = scope_badge(&app);
        assert_eq!(badge.posture, "blocked");
    }
}
