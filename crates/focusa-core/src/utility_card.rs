//! Utility/bootstrap/post-compaction cards for Focusa-aware agents.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UtilityCard {
    pub schema: String,
    pub status: String,
    pub purpose: String,
    pub preferred_layer: String,
    pub authority_boundary: String,
    pub usefulness_bar: Vec<String>,
    pub scope_gate: Vec<String>,
    pub bootstrap_card: Vec<String>,
    pub post_compaction_card: Vec<String>,
    pub exact_next_actions: Vec<String>,
    pub do_not_drift: Vec<String>,
    pub evidence_policy: Vec<String>,
    pub brevity_rules: Vec<String>,
    pub recovery_order: Vec<String>,
    pub proof_commands: Vec<String>,
    pub next_tools: Vec<String>,
}

pub fn utility_card() -> UtilityCard {
    UtilityCard {
        schema: "focusa.utility_card.v1".to_string(),
        status: "completed".to_string(),
        purpose: "Compact but decision-useful startup, bootstrap, post-compaction, recovery, and tool-brevity guidance for Focusa-aware agents.".to_string(),
        preferred_layer: "focusa_* tools before raw daemon calls".to_string(),
        authority_boundary: "Action authority requires matching project_root plus continuity_id; trajectory is north-star context only.".to_string(),
        usefulness_bar: vec![
            "A card is useful only if it states status, authority, why, exact next action, evidence refs, and recovery path.".to_string(),
            "Brevity removes filler, not decision-critical context.".to_string(),
            "Every card must let the next agent act without transcript-tail authority.".to_string(),
        ],
        scope_gate: vec![
            "Resolve project identity before trusting Workpoint or Trajectory authority.".to_string(),
            "Compare project_root and continuity_id before durable writes.".to_string(),
            "If scope conflicts, verify project then checkpoint before durable writes.".to_string(),
        ],
        bootstrap_card: vec![
            "Read focusa_utility_card or focusa_agent_prompt at session start.".to_string(),
            "Resume Workpoint and verify canonical=true plus matching project_root/continuity_id.".to_string(),
            "Read Trajectory as north-star context, not mutation authority.".to_string(),
            "Run git status and bd ready from the verified project root.".to_string(),
            "If changing code: inspect diff, implement smallest useful slice, run gates, capture evidence, commit, push.".to_string(),
        ],
        post_compaction_card: vec![
            "Treat transcript tail as non-authoritative; use WorkpointResumePacket first.".to_string(),
            "State any scope conflict visibly before editing.".to_string(),
            "Rehydrate only bounded refs needed for the next action.".to_string(),
            "Keep previous proof as handles, not pasted logs.".to_string(),
            "Before final report: evaluate/re-record relevant prediction and capture metacog only if reusable.".to_string(),
        ],
        exact_next_actions: vec![
            "focusa_workpoint_resume -- project_root + continuity_id".to_string(),
            "focusa_project_identity -- verify current repository".to_string(),
            "bd ready -- choose highest-priority unblocked bead".to_string(),
            "git status --short --branch -- separate intended edits from generated residue".to_string(),
            "focusa_evidence_capture -- attach proof after checks or live probes".to_string(),
        ],
        do_not_drift: vec![
            "Do not treat stale transcript summaries as authority.".to_string(),
            "Do not stage generated ECS/runtime residue unless the bead explicitly requires it.".to_string(),
            "Do not hide blockers behind static tests when the acceptance criterion requires product/runtime evidence.".to_string(),
            "Do not shorten tool descriptions so far that action timing or recovery path disappears.".to_string(),
        ],
        evidence_policy: vec![
            "Prefer stable handles: git commit, test id, API route, CLI command, artifact path, browser session id.".to_string(),
            "Capture evidence after verification, not as a substitute for verification.".to_string(),
            "End reports include task outcome, proof, prediction outcome, reusable lesson, next bounded possibility.".to_string(),
        ],
        brevity_rules: vec![
            "One-line summaries must preserve status + authority + next action.".to_string(),
            "Tool descriptions should say when to use the tool and what it returns.".to_string(),
            "Prompt snippets should be one actionable sentence.".to_string(),
            "Docs should link canonical contracts instead of duplicating long payloads.".to_string(),
        ],
        recovery_order: vec![
            "focusa_workpoint_resume".to_string(),
            "focusa_project_identity".to_string(),
            "focusa_project_verify".to_string(),
            "focusa_tool_doctor".to_string(),
            "focusa_dxux_explain".to_string(),
        ],
        proof_commands: vec![
            "focusa utility card".to_string(),
            "focusa utility bootstrap".to_string(),
            "focusa utility post-compaction".to_string(),
            "curl -fsS http://127.0.0.1:8787/v1/utility/card | jq .schema".to_string(),
            "node scripts/validate-focusa-tool-contracts.mjs".to_string(),
            "cargo test --workspace".to_string(),
            "cargo clippy --workspace -- -D warnings".to_string(),
        ],
        next_tools: vec![
            "focusa_utility_card".to_string(),
            "focusa_workpoint_resume".to_string(),
            "focusa_trajectory_view".to_string(),
            "focusa_evidence_capture".to_string(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utility_card_has_bootstrap_compaction_and_brevity_sections() {
        let card = utility_card();
        assert_eq!(card.schema, "focusa.utility_card.v1");
        assert!(!card.bootstrap_card.is_empty());
        assert!(!card.post_compaction_card.is_empty());
        assert!(!card.brevity_rules.is_empty());
        assert!(
            card.next_tools
                .contains(&"focusa_workpoint_resume".to_string())
        );
    }

    #[test]
    fn utility_card_is_compact_but_decision_useful() {
        let card = utility_card();
        assert!(card.authority_boundary.contains("project_root"));
        assert!(
            card.usefulness_bar
                .iter()
                .any(|line| line.contains("exact next action"))
        );
        assert!(
            card.exact_next_actions
                .iter()
                .any(|line| line.contains("bd ready"))
        );
        assert!(
            card.do_not_drift
                .iter()
                .any(|line| line.contains("ECS/runtime residue"))
        );
        assert!(
            card.evidence_policy
                .iter()
                .any(|line| line.contains("git commit"))
        );
    }
}
