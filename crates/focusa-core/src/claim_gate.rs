//! Completion Claim Gate — Spec107
//!
//! Enforces evidence-quality discipline before beads can be closed.
//!
//! Evidence classes:
//! - `actual`    — from the exact runtime/platform/surface required by acceptance criteria
//! - `partial`   — covers some but not all acceptance criteria
//! - `surrogate` — from a different surface than required
//! - `blocked`   — proof attempt failed due to environment/dependency boundary
//! - `missing`   — no evidence submitted
//!
//! Gate decision:
//! - `allow`  — required evidence_class satisfied
//! - `block`  — insufficient evidence or overclaim detected
//!
//! Source: docs/107-spec-first-feature-lifecycle-and-claim-discipline-spec.md
//! Design: docs/current/FOCUSA_SPEC107_EVIDENCE_TAXONOMY_DESIGN_2026-06-15.md

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Evidence classification per Spec107 §5.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EvidenceClass {
    /// Evidence from the exact runtime/platform/surface required by acceptance criteria.
    Actual,
    /// Useful but incomplete proof.
    Partial,
    /// Proof from a different surface than required.
    Surrogate,
    /// Proof attempt failed due to environment or dependency boundary.
    Blocked,
    /// No evidence submitted.
    Missing,
}

impl std::fmt::Display for EvidenceClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvidenceClass::Actual => write!(f, "actual"),
            EvidenceClass::Partial => write!(f, "partial"),
            EvidenceClass::Surrogate => write!(f, "surrogate"),
            EvidenceClass::Blocked => write!(f, "blocked"),
            EvidenceClass::Missing => write!(f, "missing"),
        }
    }
}

impl EvidenceClass {
    /// Whether this class satisfies the preclose gate.
    pub fn is_sufficient(self) -> bool {
        matches!(self, EvidenceClass::Actual | EvidenceClass::Blocked)
    }

    /// Whether this class constitutes an overclaim.
    pub fn is_overclaim(self) -> bool {
        matches!(
            self,
            EvidenceClass::Surrogate | EvidenceClass::Partial | EvidenceClass::Missing
        )
    }

    /// Parse from string (case-insensitive).
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "actual" => Some(EvidenceClass::Actual),
            "partial" => Some(EvidenceClass::Partial),
            "surrogate" => Some(EvidenceClass::Surrogate),
            "blocked" => Some(EvidenceClass::Blocked),
            "missing" => Some(EvidenceClass::Missing),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Evidence Citation
// ---------------------------------------------------------------------------

/// A single evidence citation parsed from a close_reason.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceCitation {
    /// The stable reference string (e.g., "tests/spec107_gate_test.sh").
    pub reference: String,
    /// Detected evidence class for this citation.
    #[serde(default)]
    pub class: Option<EvidenceClass>,
    /// Optional inline annotation from the citation (e.g., "(actual: test 1)").
    #[serde(default)]
    pub annotation: Option<String>,
    /// Whether this citation matches a known stable format.
    pub format_valid: bool,
}

impl EvidenceCitation {
    /// Valid citation formats per Spec107 §2.
    const VALID_PREFIXES: &'static [&'static str] = &[
        "tests/",
        "docs/",
        "crates/",
        "apps/",
        "git:",
        "/v1/",
        "cargo test",
        "uiai:",
        "api:",
    ];

    /// Classify a citation reference into a runtime/platform/surface hint.
    pub fn surface_hint(&self) -> Option<&'static str> {
        let r = &self.reference;
        if r.starts_with("tests/") || r.starts_with("cargo test") {
            Some("test")
        } else if r.starts_with("crates/") {
            Some("crates")
        } else if r.starts_with("apps/menubar") || r.starts_with("apps/pi-extension") {
            Some("app")
        } else if r.starts_with("docs/") {
            Some("docs")
        } else if r.starts_with("git:") {
            Some("git")
        } else if r.starts_with("/v1/") || r.starts_with("api:") {
            Some("api")
        } else if r.starts_with("uiai:") {
            Some("uiai")
        } else {
            None
        }
    }

    /// Check if the citation format is valid.
    pub fn is_format_valid(&self) -> bool {
        Self::VALID_PREFIXES
            .iter()
            .any(|p| self.reference.starts_with(p))
    }
}

// ---------------------------------------------------------------------------
// Gate Input / Output
// ---------------------------------------------------------------------------

/// Input to the claim gate.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClaimGateInput {
    /// The bead/work item being claimed as complete.
    pub work_item_id: String,
    /// The claim text (typically the close reason).
    pub claim_text: String,
    /// Acceptance criteria from the bead.
    #[serde(default)]
    pub acceptance_criteria: Vec<String>,
    /// The evidence_policy from the bead (optional).
    #[serde(default)]
    pub evidence_policy: Option<EvidencePolicy>,
    /// Runtime/platform surfaces required by acceptance.
    #[serde(default)]
    pub surfaces_required: Vec<String>,
    /// Whether an operator explicitly approved blocked evidence.
    #[serde(default)]
    pub operator_deferred: bool,
}

/// Evidence policy declared by a bead.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidencePolicy {
    /// Minimum evidence class required.
    #[serde(default = "default_required_class")]
    pub required_class: String,
    /// Whether blocked evidence is acceptable with operator deferral.
    #[serde(default)]
    pub blocked_acceptable: bool,
    /// Forbidden evidence classes.
    #[serde(default)]
    pub forbidden_classes: Vec<String>,
    /// Minimum citation count.
    #[serde(default = "default_citation_minimum")]
    pub citation_minimum: usize,
    /// Required runtime/platform surfaces.
    #[serde(default)]
    pub surfaces_required: Vec<EvidenceSurface>,
}

fn default_required_class() -> String {
    "actual".to_string()
}
fn default_citation_minimum() -> usize {
    1
}

/// A required runtime/surface pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceSurface {
    pub runtime: String,
    pub surface: String,
}

/// Output from the claim gate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimGateOutput {
    #[serde(default)]
    pub schema: String,
    /// Whether the gate allows closure.
    pub decision: GateDecision,
    /// Primary evidence class determined.
    pub evidence_class: EvidenceClass,
    /// Individual citation classifications.
    pub citations: Vec<EvidenceCitation>,
    /// Aspects of acceptance criteria with no evidence.
    #[serde(default)]
    pub missing_evidence: Vec<String>,
    /// Detected overclaim risks.
    #[serde(default)]
    pub overclaim_risks: Vec<String>,
    /// Suggested recovery actions.
    #[serde(default)]
    pub recovery_commands: Vec<String>,
    /// Timestamp of gate evaluation.
    pub evaluated_at: DateTime<Utc>,
}

impl ClaimGateOutput {
    /// Build the gate output with all findings.
    pub fn build(input: &ClaimGateInput) -> Self {
        let citations = parse_evidence_citations(&input.claim_text);
        let cited_formats = citations.iter().filter(|c| c.is_format_valid()).count();
        let cited_invalid = citations.iter().filter(|c| !c.is_format_valid()).count();

        // Determine primary evidence class
        let evidence_class = classify_overall(&citations, &input.surfaces_required);
        let is_overclaim = evidence_class.is_overclaim();
        let has_citations = cited_formats > 0;

        // Build overclaim risks
        let mut overclaim_risks = Vec::new();
        if cited_invalid > 0 {
            overclaim_risks.push(format!(
                "{} citation(s) have invalid format — must use: tests/ docs/ crates/ apps/ git: /v1/ cargo test uiai:",
                cited_invalid
            ));
        }
        if cited_formats == 0 && !input.claim_text.trim().is_empty() {
            overclaim_risks.push("No valid evidence citations found in close reason".to_string());
        }
        for citation in &citations {
            if let Some(cls) = citation.class
                && cls.is_overclaim()
            {
                overclaim_risks.push(format!(
                    "Citation '{}' is {} evidence",
                    citation.reference, cls
                ));
            }
        }

        // Build recovery commands
        let mut recovery_commands = Vec::new();
        if !has_citations {
            recovery_commands.push(
                "Add evidence citations to close_reason using: Evidence citations: tests/... ; docs/... ; git:...".to_string()
            );
        }
        if is_overclaim {
            if evidence_class == EvidenceClass::Surrogate {
                recovery_commands.push(
                    "Replace surrogate evidence with actual evidence from required surface"
                        .to_string(),
                );
            }
            if evidence_class == EvidenceClass::Partial {
                recovery_commands.push(
                    "Collect remaining evidence to satisfy all acceptance criteria".to_string(),
                );
            }
        }
        if evidence_class == EvidenceClass::Missing {
            recovery_commands.push("Attach evidence before closing: run tests, capture logs, screenshot, or document blocker".to_string());
        }

        // Missing evidence list
        let missing_evidence = if input.acceptance_criteria.is_empty() {
            vec![]
        } else {
            input
                .acceptance_criteria
                .iter()
                .filter(|ac| !has_evidence_for_criterion(ac, &citations))
                .cloned()
                .collect()
        };

        // Decision
        let decision = decide(
            &evidence_class,
            cited_formats,
            &input.surfaces_required,
            input.operator_deferred,
            input.evidence_policy.as_ref(),
            &citations,
        );

        Self {
            schema: "focusa.claim_gate_output.v1".to_string(),
            decision,
            evidence_class,
            citations,
            missing_evidence,
            overclaim_risks,
            recovery_commands,
            evaluated_at: Utc::now(),
        }
    }

    /// Human-readable summary for CLI/API output.
    pub fn summary(&self) -> String {
        let mut lines = vec![
            format!("decision: {}", self.decision),
            format!("evidence_class: {}", self.evidence_class),
        ];
        if !self.citations.is_empty() {
            lines.push(format!(
                "citations: {} parsed ({} valid)",
                self.citations.len(),
                self.citations
                    .iter()
                    .filter(|c| c.is_format_valid())
                    .count()
            ));
        }
        if !self.missing_evidence.is_empty() {
            lines.push(format!(
                "missing_evidence: {} item(s)",
                self.missing_evidence.len()
            ));
        }
        if !self.overclaim_risks.is_empty() {
            lines.push("⚠ overclaim_risks:".to_string());
            for risk in &self.overclaim_risks {
                lines.push(format!("  - {}", risk));
            }
        }
        if !self.recovery_commands.is_empty() {
            lines.push("recovery_commands:".to_string());
            for cmd in &self.recovery_commands {
                lines.push(format!("  - {}", cmd));
            }
        }
        lines.join("\n")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GateDecision {
    Allow,
    Block,
}

impl std::fmt::Display for GateDecision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GateDecision::Allow => write!(f, "allow"),
            GateDecision::Block => write!(f, "block"),
        }
    }
}

// ---------------------------------------------------------------------------
// Parsing
// ---------------------------------------------------------------------------

/// Parse evidence citations from a close_reason string.
///
/// Format: "Evidence citations: <cite1> [; <cite2>...] [(annotation)]"
fn parse_evidence_citations(text: &str) -> Vec<EvidenceCitation> {
    let mut citations = Vec::new();

    // Find the "Evidence citations:" prefix
    let Some(start_idx) = text.find("Evidence citations:") else {
        return citations;
    };

    let after_prefix = &text[start_idx + "Evidence citations:".len()..];
    // Split on ';'
    for part in after_prefix.split(';') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        // Extract inline annotation in parentheses
        let (reference, annotation) = if let Some(paren_start) = part.find("(class:") {
            let reference = part[..paren_start].trim().to_string();
            let rest = &part[paren_start..];
            let annotation = rest.trim_end_matches(')').to_string();
            (reference, Some(annotation))
        } else {
            (part.to_string(), None)
        };

        // Parse inline class annotation
        let class = annotation
            .as_ref()
            .and_then(|a| a.strip_prefix("(class:").and_then(EvidenceClass::parse));

        citations.push(EvidenceCitation {
            reference,
            class,
            annotation,
            format_valid: false, // set below
        });
    }

    // Mark format validity
    for citation in &mut citations {
        citation.format_valid = citation.is_format_valid();
    }

    citations
}

/// Classify the overall evidence class based on citations and required surfaces.
fn classify_overall(citations: &[EvidenceCitation], surfaces_required: &[String]) -> EvidenceClass {
    if citations.is_empty() {
        return EvidenceClass::Missing;
    }

    // Check if all citations are format-valid
    let all_valid = citations.iter().all(|c| c.format_valid);
    if !all_valid {
        return EvidenceClass::Missing; // malformed citations treated as missing
    }

    // If any citation has an explicit class annotation, use the worst one
    let cited_classes: Vec<_> = citations.iter().filter_map(|c| c.class).collect();
    if !cited_classes.is_empty() {
        // Return the worst cited class
        return *cited_classes
            .iter()
            .max_by_key(|c| match c {
                EvidenceClass::Missing => 4,
                EvidenceClass::Surrogate => 3,
                EvidenceClass::Partial => 2,
                EvidenceClass::Blocked => 1,
                EvidenceClass::Actual => 0,
            })
            .unwrap();
    }

    // Check cited surfaces
    let cited_surfaces: HashSet<&str> = citations.iter().filter_map(|c| c.surface_hint()).collect();

    // If surfaces are required and no code/test surface is cited, it's surrogate
    let has_code_surface = cited_surfaces.contains("crates")
        || cited_surfaces.contains("app")
        || cited_surfaces.contains("test");
    if !surfaces_required.is_empty() && !has_code_surface {
        return EvidenceClass::Surrogate;
    }

    // Default to actual if we have valid citations
    EvidenceClass::Actual
}

/// Check whether any citation covers a given acceptance criterion.
fn has_evidence_for_criterion(_criterion: &str, citations: &[EvidenceCitation]) -> bool {
    // Simple heuristic: any valid citation counts as evidence
    !citations.is_empty() && citations.iter().all(|c| c.format_valid)
}

// ---------------------------------------------------------------------------
// Decision Logic
// ---------------------------------------------------------------------------

/// Determine the gate decision.
fn decide(
    class: &EvidenceClass,
    citation_count: usize,
    surfaces_required: &[String],
    operator_deferred: bool,
    policy: Option<&EvidencePolicy>,
    citations: &[EvidenceCitation],
) -> GateDecision {
    // Policy overrides if present
    if let Some(p) = policy {
        if p.forbidden_classes.iter().any(|f| {
            let parsed = EvidenceClass::parse(f);
            parsed.as_ref() == Some(class)
        }) {
            return GateDecision::Block;
        }
        if citation_count < p.citation_minimum {
            return GateDecision::Block;
        }
        if class == &EvidenceClass::Blocked && !p.blocked_acceptable && !operator_deferred {
            return GateDecision::Block;
        }
    }

    // Surface requirement check
    if !surfaces_required.is_empty() && *class == EvidenceClass::Surrogate {
        return GateDecision::Block;
    }

    // Check per-citation class annotations (inline override)
    let cited_classes: Vec<_> = citations.iter().filter_map(|c| c.class).collect();

    // If any citation explicitly declares a class, the worst class governs
    if !cited_classes.is_empty() {
        let worst = cited_classes
            .iter()
            .max_by_key(|c| {
                // Rank: missing > surrogate > partial > blocked > actual
                match c {
                    EvidenceClass::Missing => 4,
                    EvidenceClass::Surrogate => 3,
                    EvidenceClass::Partial => 2,
                    EvidenceClass::Blocked => 1,
                    EvidenceClass::Actual => 0,
                }
            })
            .copied();

        if let Some(worst_class) = worst {
            match worst_class {
                EvidenceClass::Actual => {}
                EvidenceClass::Blocked if operator_deferred => {}
                EvidenceClass::Blocked => return GateDecision::Block,
                EvidenceClass::Partial => return GateDecision::Block,
                EvidenceClass::Surrogate => return GateDecision::Block,
                EvidenceClass::Missing => return GateDecision::Block,
            }
        }
    }

    // Core rules from Spec107 §4.6
    match class {
        EvidenceClass::Actual => GateDecision::Allow,
        EvidenceClass::Blocked if operator_deferred => GateDecision::Allow,
        EvidenceClass::Blocked => GateDecision::Block,
        EvidenceClass::Partial => GateDecision::Block,
        EvidenceClass::Surrogate => GateDecision::Block,
        EvidenceClass::Missing => GateDecision::Block,
    }
}

// ---------------------------------------------------------------------------
// CLI Entry Point
// ---------------------------------------------------------------------------

/// Run the claim gate from CLI arguments.
pub fn run_gate_cli(work_item_id: &str, claim_text: &str) -> ClaimGateOutput {
    let input = ClaimGateInput {
        work_item_id: work_item_id.to_string(),
        claim_text: claim_text.to_string(),
        ..Default::default()
    };
    ClaimGateOutput::build(&input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_citations() {
        let text = "Evidence citations: tests/spec107_gate_test.sh ; docs/107-spec-first-feature-lifecycle-and-claim-discipline-spec.md ; git:2c76acf";
        let citations = parse_evidence_citations(text);
        assert_eq!(citations.len(), 3);
        assert!(citations[0].format_valid);
        assert!(citations[1].format_valid);
        assert!(citations[2].format_valid);
    }

    #[test]
    fn test_parse_annotated_citations() {
        let text = "Evidence citations: tests/spec107_gate_test.sh (class: actual) ; docs/107-spec.md (class: partial)";
        let citations = parse_evidence_citations(text);
        assert_eq!(citations.len(), 2);
        assert_eq!(citations[0].class, Some(EvidenceClass::Actual));
        assert_eq!(citations[1].class, Some(EvidenceClass::Partial));
    }

    #[test]
    fn test_parse_no_citations() {
        let text = "Completed successfully";
        let citations = parse_evidence_citations(text);
        assert!(citations.is_empty());
    }

    #[test]
    fn test_evidence_class_sufficiency() {
        assert!(EvidenceClass::Actual.is_sufficient());
        assert!(!EvidenceClass::Partial.is_sufficient());
        assert!(!EvidenceClass::Surrogate.is_sufficient());
        assert!(EvidenceClass::Blocked.is_sufficient()); // sufficient with deferral
        assert!(!EvidenceClass::Missing.is_sufficient());
    }

    #[test]
    fn test_gate_allow_with_actual() {
        let input = ClaimGateInput {
            work_item_id: "focusa-bwky.4".into(),
            claim_text: "Evidence citations: crates/focusa-core/src/claim_gate.rs".into(),
            ..Default::default()
        };
        let output = ClaimGateOutput::build(&input);
        assert_eq!(output.decision, GateDecision::Allow);
        assert_eq!(output.evidence_class, EvidenceClass::Actual);
    }

    #[test]
    fn test_gate_block_with_partial() {
        let input = ClaimGateInput {
            work_item_id: "focusa-bwky.4".into(),
            claim_text: "Evidence citations: tests/spec107_gate_test.sh (class: partial)".into(),
            surfaces_required: vec!["crates/focusa-core".to_string()],
            ..Default::default()
        };
        let output = ClaimGateOutput::build(&input);
        assert_eq!(output.decision, GateDecision::Block);
        assert_eq!(output.evidence_class, EvidenceClass::Partial);
    }

    #[test]
    fn test_gate_block_with_surrogate() {
        let input = ClaimGateInput {
            work_item_id: "focusa-ui0y.15".into(),
            claim_text: "Evidence citations: api:/v1/device/pair-list".into(),
            surfaces_required: vec!["macos-arm64".to_string()],
            ..Default::default()
        };
        let output = ClaimGateOutput::build(&input);
        assert_eq!(output.decision, GateDecision::Block);
        assert_eq!(output.evidence_class, EvidenceClass::Surrogate);
    }

    #[test]
    fn test_gate_allow_blocked_with_deferral() {
        let input = ClaimGateInput {
            work_item_id: "focusa-qasy.25".into(),
            claim_text: "Evidence citations: apps/menubar/src-tauri/Cargo.toml (class: blocked)"
                .into(),
            operator_deferred: true,
            ..Default::default()
        };
        let output = ClaimGateOutput::build(&input);
        assert_eq!(output.decision, GateDecision::Allow);
        assert_eq!(output.evidence_class, EvidenceClass::Blocked);
    }

    #[test]
    fn test_gate_block_missing() {
        let input = ClaimGateInput {
            work_item_id: "focusa-bwky.4".into(),
            claim_text: "Completed implementation".into(),
            ..Default::default()
        };
        let output = ClaimGateOutput::build(&input);
        assert_eq!(output.decision, GateDecision::Block);
        assert_eq!(output.evidence_class, EvidenceClass::Missing);
    }

    #[test]
    fn test_surface_hint() {
        let c1 = EvidenceCitation {
            reference: "crates/focusa-core/src/claim_gate.rs".into(),
            class: None,
            annotation: None,
            format_valid: true,
        };
        assert_eq!(c1.surface_hint(), Some("crates"));

        let c2 = EvidenceCitation {
            reference: "tests/spec107_gate_test.sh".into(),
            class: None,
            annotation: None,
            format_valid: true,
        };
        assert_eq!(c2.surface_hint(), Some("test"));
    }
}
