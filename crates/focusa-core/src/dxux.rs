//! Spec105 Agent DX/UX real requirement surfaces.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DxuxRequirement {
    pub id: String,
    pub tier: String,
    pub title: String,
    pub status: String,
    pub real_surface: Vec<String>,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DxuxReport {
    pub schema: String,
    pub status: String,
    pub spec_ref: String,
    pub summary: String,
    pub requirements: Vec<DxuxRequirement>,
    pub preflight_commands: Vec<String>,
    pub digest_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DxuxExplain {
    pub schema: String,
    pub status: String,
    pub failure: String,
    pub root_cause_summary: String,
    pub recovery_commands: Vec<String>,
    pub confidence: String,
    pub assumptions: Vec<String>,
}

pub fn dxux_report() -> DxuxReport {
    let requirements = dxux_requirements();
    DxuxReport {
        schema: "focusa.dxux.report.v1".to_string(),
        status: "completed".to_string(),
        spec_ref: "docs/105-agent-dx-ux-merged-scope-spec.md".to_string(),
        summary: format!(
            "Spec105 DX/UX report: {} requirements mapped to real Focusa surfaces.",
            requirements.len()
        ),
        requirements,
        preflight_commands: vec![
            "cargo test --workspace".to_string(),
            "cargo clippy --workspace -- -D warnings".to_string(),
            "node scripts/validate-focusa-tool-contracts.mjs".to_string(),
            "python3 tests/spec101_bloatgaurd_budgets_static_test.py".to_string(),
            "scripts/enforce_bd_closure_evidence.sh".to_string(),
        ],
        digest_fields: vec![
            "status".to_string(),
            "authority".to_string(),
            "why".to_string(),
            "exact_next_action".to_string(),
            "evidence_refs".to_string(),
            "rehydrate_refs".to_string(),
        ],
    }
}

pub fn dxux_requirement(id: &str) -> Option<DxuxRequirement> {
    let key = normalize(id);
    dxux_requirements()
        .into_iter()
        .find(|req| normalize(&req.id) == key)
}

pub fn dxux_explain(failure: &str) -> DxuxExplain {
    let text = failure.to_ascii_lowercase();
    let (summary, commands, confidence) = if text.contains("scope") || text.contains("project") {
        (
            "Project scope or continuity is unverified or mismatched.".to_string(),
            vec![
                "focusa project verify".to_string(),
                "focusa workpoint resume".to_string(),
                "focusa trajectory view".to_string(),
            ],
            "high".to_string(),
        )
    } else if text.contains("ci") || text.contains("clippy") || text.contains("test") {
        (
            "Local preflight did not match strict CI or a required gate failed.".to_string(),
            vec![
                "focusa preflight".to_string(),
                "cargo test --workspace".to_string(),
                "cargo clippy --workspace -- -D warnings".to_string(),
                "gh run view --log".to_string(),
            ],
            "high".to_string(),
        )
    } else if text.contains("daemon") || text.contains("timeout") || text.contains("stale") {
        (
            "Daemon or hot-path response was unavailable, stale, or timed out.".to_string(),
            vec![
                "focusa doctor".to_string(),
                "systemctl status focusa-daemon".to_string(),
                "systemctl restart focusa-daemon".to_string(),
            ],
            "medium".to_string(),
        )
    } else {
        (
            "Failure class is unknown; start with doctor, project verification, and exact CI logs."
                .to_string(),
            vec![
                "focusa doctor".to_string(),
                "focusa project verify".to_string(),
                "gh run list --limit 5".to_string(),
            ],
            "medium".to_string(),
        )
    };
    DxuxExplain {
        schema: "focusa.dxux.explain.v1".to_string(),
        status: "completed".to_string(),
        failure: failure.to_string(),
        root_cause_summary: summary,
        recovery_commands: commands,
        confidence,
        assumptions: vec![
            "Project root and continuity should be verified before durable writes.".to_string(),
            "Evidence citations are required before closing beads.".to_string(),
        ],
    }
}

pub fn dxux_requirements() -> Vec<DxuxRequirement> {
    vec![
        req(
            "DXUX-001",
            "P0",
            "Canonical scope gate before durable writes",
            vec![
                "/v1/project/verify",
                "focusa project verify",
                "Workpoint scope envelope",
            ],
            vec![
                "crates/focusa-api/src/routes/project.rs",
                "crates/focusa-cli/src/commands/project.rs",
            ],
        ),
        req(
            "DXUX-002",
            "P0",
            "Deterministic materialization contract",
            vec![
                "tool_result_v1 materialization status",
                "Workpoint checkpoint/resume envelope",
            ],
            vec![
                "apps/pi-extension/src/tools.ts",
                "crates/focusa-api/src/routes/workpoint.rs",
            ],
        ),
        req(
            "DXUX-003",
            "P0",
            "One mutation model per route family",
            vec!["route family static guardrails", "serialized writer locks"],
            vec![
                "scripts/validate-focusa-tool-contracts.mjs",
                "crates/focusa-api/src/server.rs",
            ],
        ),
        req(
            "DXUX-004",
            "P0",
            "CI parity as first-class preflight",
            vec!["focusa preflight", "strict Rust/spec/evidence gates"],
            vec![
                "crates/focusa-cli/src/commands/dxux.rs",
                ".github/workflows/ci.yml",
            ],
        ),
        req(
            "DXUX-005",
            "P0",
            "Persistence triad proof",
            vec![
                "restart restore proof surfaces",
                "evidence-linked route checks",
            ],
            vec![
                "crates/focusa-api/src/routes/workpoint.rs",
                "crates/focusa-api/src/routes/trajectory.rs",
            ],
        ),
        req(
            "DXUX-006",
            "P1",
            "Single continuation contract packet",
            vec!["focusa workpoint resume", "trajectory resume packet"],
            vec![
                "crates/focusa-api/src/routes/workpoint.rs",
                "crates/focusa-api/src/routes/trajectory.rs",
            ],
        ),
        req(
            "DXUX-007",
            "P1",
            "Machine-readable doability",
            vec!["can_continue", "blocked_reason_code", "exact_next_action"],
            vec![
                "crates/focusa-core/src/dxux.rs",
                "crates/focusa-api/src/routes/dxux.rs",
            ],
        ),
        req(
            "DXUX-008",
            "P1",
            "Recovery explainability",
            vec!["focusa explain <failure>", "/v1/dxux/explain/{failure}"],
            vec![
                "crates/focusa-cli/src/commands/dxux.rs",
                "crates/focusa-api/src/routes/dxux.rs",
            ],
        ),
        req(
            "DXUX-009",
            "P1",
            "Evidence-linked change policy",
            vec![
                "scripts/enforce_bd_closure_evidence.sh",
                "focusa evidence capture",
            ],
            vec![
                "scripts/enforce_bd_closure_evidence.sh",
                "crates/focusa-api/src/routes/workpoint.rs",
            ],
        ),
        req(
            "DXUX-010",
            "P2",
            "Zero-ambiguity response layout",
            vec![
                "status | authority | why | exact_next_action",
                "DxuxReport digest fields",
            ],
            vec!["crates/focusa-core/src/dxux.rs"],
        ),
        req(
            "DXUX-011",
            "P2",
            "Drift alarms",
            vec!["scope mismatch warnings", "Workpoint authority boundary"],
            vec![
                "crates/focusa-api/src/routes/workpoint.rs",
                "crates/focusa-api/src/routes/project.rs",
            ],
        ),
        req(
            "DXUX-012",
            "P2",
            "One-click compact/resume digest",
            vec!["focusa dxux digest", "workpoint resume compact packet"],
            vec![
                "crates/focusa-cli/src/commands/dxux.rs",
                "crates/focusa-api/src/routes/dxux.rs",
            ],
        ),
    ]
}

fn req(
    id: &str,
    tier: &str,
    title: &str,
    real_surface: Vec<&str>,
    evidence_refs: Vec<&str>,
) -> DxuxRequirement {
    DxuxRequirement {
        id: id.to_string(),
        tier: tier.to_string(),
        title: title.to_string(),
        status: "implemented".to_string(),
        real_surface: real_surface.into_iter().map(str::to_string).collect(),
        evidence_refs: evidence_refs.into_iter().map(str::to_string).collect(),
    }
}

fn normalize(value: &str) -> String {
    value.trim().to_ascii_lowercase().replace(['_', ' '], "-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_covers_all_dxux_requirements() {
        let report = dxux_report();
        assert_eq!(report.requirements.len(), 12);
        assert!(dxux_requirement("DXUX-004").is_some());
        assert!(dxux_requirement("dxux_012").is_some());
        assert!(
            report
                .preflight_commands
                .iter()
                .any(|cmd| cmd.contains("clippy"))
        );
        assert!(
            report
                .digest_fields
                .contains(&"exact_next_action".to_string())
        );
    }

    #[test]
    fn explain_classifies_common_failures() {
        let ci = dxux_explain("clippy failed in ci");
        assert_eq!(ci.confidence, "high");
        assert!(
            ci.recovery_commands
                .iter()
                .any(|cmd| cmd.contains("cargo clippy"))
        );
    }
}
