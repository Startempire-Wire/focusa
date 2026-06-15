//! Programmatic guard against false completion/evidence claims.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CompletionClaimRequest {
    pub work_item_id: Option<String>,
    pub claim: String,
    pub acceptance_criteria: Vec<String>,
    pub evidence_refs: Vec<String>,
    pub evidence_summaries: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompletionClaimGateReport {
    pub schema: String,
    pub status: String,
    pub decision: String,
    pub evidence_class: String,
    pub authority: String,
    pub missing_required_evidence: Vec<String>,
    pub overclaim_risks: Vec<String>,
    pub recovery_commands: Vec<String>,
    pub reason: String,
}

pub fn completion_claim_gate(input: CompletionClaimRequest) -> CompletionClaimGateReport {
    let combined = format!(
        "{}\n{}\n{}\n{}",
        input.claim,
        input.acceptance_criteria.join("\n"),
        input.evidence_refs.join("\n"),
        input.evidence_summaries.join("\n")
    )
    .to_ascii_lowercase();
    let evidence_text = format!(
        "{}\n{}",
        input.evidence_refs.join("\n"),
        input.evidence_summaries.join("\n")
    )
    .to_ascii_lowercase();
    let criteria_text = input.acceptance_criteria.join("\n").to_ascii_lowercase();

    let mut risks = Vec::new();
    let mut missing = Vec::new();

    let evidence_class = classify_evidence(&evidence_text);
    if evidence_class != "actual" {
        risks.push(format!(
            "evidence_class={evidence_class}; completion requires actual evidence"
        ));
    }

    let platform_requirements = [
        ("macos", ["macos", "mac ", ".app"].as_slice()),
        ("keychain", ["keychain"].as_slice()),
        (
            "restart persistence",
            ["restart", "relaunch", "survives restart"].as_slice(),
        ),
        (
            "screenshots/logs",
            ["screenshot", "screenshots", "logs"].as_slice(),
        ),
        (
            "native runtime",
            ["native", "tauri", "invoke", "window", "menu"].as_slice(),
        ),
    ];

    for (label, needles) in platform_requirements {
        if needles.iter().any(|needle| criteria_text.contains(needle))
            && !needles.iter().any(|needle| evidence_text.contains(needle))
        {
            missing.push(label.to_string());
        }
    }

    let surrogate_markers = [
        "api/web-only",
        "api/web",
        "local proof",
        "local-only",
        "surrogate",
        "partial",
        "not validated",
        "not complete",
        "blocked",
        "missing",
    ];
    if surrogate_markers
        .iter()
        .any(|marker| combined.contains(marker))
    {
        risks.push("claim or evidence contains partial/surrogate/blocker language".to_string());
    }

    let mac_runtime_claim = criteria_text.contains("mac")
        || criteria_text.contains("keychain")
        || criteria_text.contains(".app")
        || criteria_text.contains("native");
    let api_web_only = ["curl", "/v1/", "vite", "web build", "api"]
        .iter()
        .any(|marker| evidence_text.contains(marker))
        && !["keychain", ".app launch", "macos app", "screenshot"]
            .iter()
            .any(|marker| evidence_text.contains(marker));
    if mac_runtime_claim && api_web_only {
        risks.push(
            "runtime/platform acceptance cannot be satisfied by API/web-only evidence".to_string(),
        );
    }

    let decision = if missing.is_empty() && risks.is_empty() {
        "allow"
    } else {
        "block"
    };
    let reason = if decision == "allow" {
        "claim has actual evidence for stated acceptance criteria".to_string()
    } else {
        "completion claim is not supported by actual evidence for the stated acceptance criteria"
            .to_string()
    };

    CompletionClaimGateReport {
        schema: "focusa.completion_claim_gate.v1".to_string(),
        status: "completed".to_string(),
        decision: decision.to_string(),
        evidence_class,
        authority: "advisory gate for bd close/final report; blocks overclaim unless actual evidence exists".to_string(),
        missing_required_evidence: missing,
        overclaim_risks: risks,
        recovery_commands: vec![
            "record partial evidence without closing the bead".to_string(),
            "collect missing actual runtime/platform proof".to_string(),
            "rerun focusa claim preclose with actual evidence refs".to_string(),
        ],
        reason,
    }
}

fn classify_evidence(text: &str) -> String {
    if text.trim().is_empty() {
        return "missing".to_string();
    }
    if [
        "blocked",
        "failed",
        "not validated",
        "not complete",
        "missing",
    ]
    .iter()
    .any(|marker| text.contains(marker))
    {
        return "blocked".to_string();
    }
    if [
        "partial",
        "surrogate",
        "local-only",
        "local proof",
        "api/web-only",
        "api/web",
    ]
    .iter()
    .any(|marker| text.contains(marker))
    {
        return "partial".to_string();
    }
    "actual".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mac_pairing_api_web_only_evidence_blocks_completion() {
        let report = completion_claim_gate(CompletionClaimRequest {
            work_item_id: Some("focusa-ui0y.15".to_string()),
            claim: "Mac menubar pairing E2E complete".to_string(),
            acceptance_criteria: vec![
                "actual macOS .app launch".to_string(),
                "Keychain persistence".to_string(),
                "restart persistence".to_string(),
                "screenshots/logs".to_string(),
                "native Tauri runtime".to_string(),
            ],
            evidence_refs: vec![
                "/v1/device/pair/*".to_string(),
                "npm menubar web build".to_string(),
            ],
            evidence_summaries: vec![
                "partial local proof only; API/web flow passed; native Mac not validated"
                    .to_string(),
            ],
        });
        assert_eq!(report.decision, "block");
        assert_ne!(report.evidence_class, "actual");
        assert!(
            report
                .overclaim_risks
                .iter()
                .any(|risk| risk.contains("API/web-only") || risk.contains("partial"))
        );
    }

    #[test]
    fn actual_matching_evidence_allows_completion() {
        let report = completion_claim_gate(CompletionClaimRequest {
            work_item_id: Some("demo".to_string()),
            claim: "Mac pairing E2E complete".to_string(),
            acceptance_criteria: vec![
                "macOS .app launch".to_string(),
                "Keychain persistence".to_string(),
                "restart persistence".to_string(),
                "screenshots/logs".to_string(),
                "native Tauri runtime".to_string(),
            ],
            evidence_refs: vec!["macos-app-screenshot:pair-complete".to_string()],
            evidence_summaries: vec![
                "Actual macOS app launch verified; Keychain token persisted; relaunch restart preserved paired state; screenshots and logs captured; native Tauri invoke window menu lifecycle verified.".to_string(),
            ],
        });
        assert_eq!(report.decision, "allow");
        assert_eq!(report.evidence_class, "actual");
    }
}
