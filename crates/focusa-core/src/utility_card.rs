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
        authority_boundary: "Resolve ScopeRef/ProjectRootKey + WorkstreamId + ContinuityId and applicable AttachmentKey; the verified working-subpath is not a separate cognitive owner. Current instruction, stage, operation/frontier and required grants govern effects; orientation is not admission.".to_string(),
        usefulness_bar: vec![
            "A card is useful only if it states status, authority, why, exact next action, evidence refs, and recovery path.".to_string(),
            "Brevity removes filler, not decision-critical context.".to_string(),
            "Every card must let the next agent act without transcript-tail authority.".to_string(),
        ],
        scope_gate: vec![
            "For an authorized, verified safe unbound repository, inspect Bootstrap/Genesis; focusa init --quickstart is a scoped entry, not automatic permission or mandatory recovery. Verify .focusa-project.json before HLT or Workpoint admission; reuse valid existing projects.".to_string(),
            "Resolve project identity before trusting Workpoint or Trajectory authority.".to_string(),
            "Compare requested, resolved and saved Workstream/lineage/attachment plus working-subpath before durable writes.".to_string(),
            "If scope conflicts, diagnose and verify supported binding repair before checkpointing; do not infer a global migration or repeat an unchanged denial.".to_string(),
        ],
        bootstrap_card: vec![
            "Read focusa_utility_card or focusa_agent_prompt at session start.".to_string(),
            "Verify scope and resume the current Workpoint; canonical=true is necessary where required but does not itself establish current instruction match or effect admission.".to_string(),
            "Read Trajectory as north-star context, not mutation authority.".to_string(),
            "Run git status and bd ready from the verified project root.".to_string(),
            "Select only capabilities needed by the current verified action: conditional Bootstrap/Genesis, linked Ladder/spec/tasks, then Prepare/Act/Reconcile/Advance through existing owners.".to_string(),
            "For active development, apply the approved reload/deployment and verify the exact consumer promptly; commit/push only where the project delivery contract requires them, not as universal completion gates.".to_string(),
        ],
        post_compaction_card: vec![
            "Treat transcript tail as non-authoritative; use WorkpointResumePacket first.".to_string(),
            "State any scope conflict visibly before editing.".to_string(),
            "Rehydrate only bounded refs needed for the next action.".to_string(),
            "Keep previous proof as handles, not pasted logs.".to_string(),
            "Before final report: evaluate/re-record relevant prediction and capture metacog only if reusable.".to_string(),
        ],
        exact_next_actions: vec![
            "focusa_workpoint_resume -- canonical parent + working_subpath_id + continuity_id".to_string(),
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
            "Next-tool, recovery_order and proof-command lists are conditional hints, not required chains or permission to run locally.".to_string(),
            "A failed operation pauses only affected dependents: diagnose input, observation, reporting, binding, authority or missing proof; recover with installed support, verify and resume without inventing admission.".to_string(),
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
    fn utility_guidance_preserves_current_scope_and_project_specific_delivery() {
        let card = utility_card();
        assert!(card.authority_boundary.contains("WorkstreamId"));
        assert!(card.authority_boundary.contains("AttachmentKey"));
        assert!(card.bootstrap_card.iter().any(|line| line.contains("not as universal completion gates")));
        assert!(card.brevity_rules.iter().any(|line| line.contains("conditional hints")));
    }

    #[test]
    fn utility_card_is_compact_but_decision_useful() {
        let card = utility_card();
        assert!(card.authority_boundary.contains("working-subpath"));
        assert!(
            card.usefulness_bar
                .iter()
                .any(|line| line.contains("exact next action"))
        );
        assert!(
            card.scope_gate.iter().any(|line| {
                line.contains("focusa init --quickstart")
                    && line.contains(".focusa-project.json")
                    && line.contains("before HLT or Workpoint")
            }),
            "unbound repositories must receive marker-first guidance"
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
