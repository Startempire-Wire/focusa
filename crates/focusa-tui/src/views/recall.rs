//! Mission Recall tab (Spec 117 §15).
//!
//! Recall is advisory only: it can help inspect prior context, but cannot create
//! canonical Workpoint authority without the promotion/preflight flow.

use crate::app::App;
use crate::theme;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub const RECALL_SEARCH_SOURCES: &[&str] = &[
    "Focusa events",
    "Workpoints",
    "Evidence refs",
    "Audit timeline",
    "Agent bootstrap packets",
    "Pi/Codex/Claude/Cursor/OpenCode imports",
    "UIAI diagnostics packets",
    "manual session notes",
];

pub const RECALL_CARD_FIELDS: &[&str] = &[
    "result_id",
    "provider",
    "source_session_id",
    "project_root",
    "continuity_id",
    "timestamp",
    "span_type",
    "memory_status",
    "scope_status",
    "proof_status",
    "allowed_use",
    "safe_excerpt",
    "evidence_refs",
    "next_action",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryStatus {
    Active,
    Stale,
    Superseded,
    Contradicted,
    Noise,
    Quarantined,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScopeStatus {
    Current,
    SameProjectOtherContinuity,
    OtherProject,
    GlobalAdvisory,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProofStatus {
    None,
    Linked,
    Verified,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AllowedUse {
    Include,
    InspectOnly,
    VerifyFirst,
    Exclude,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecallDeckCard {
    pub result_id: String,
    pub provider: String,
    pub source_session_id: String,
    pub project_root: String,
    pub continuity_id: String,
    pub timestamp: String,
    pub span_type: String,
    pub memory_status: MemoryStatus,
    pub scope_status: ScopeStatus,
    pub proof_status: ProofStatus,
    pub allowed_use: AllowedUse,
    pub safe_excerpt: String,
    pub evidence_refs: Vec<String>,
    pub next_action: String,
}

impl RecallDeckCard {
    pub fn demo(project_root: &str) -> Self {
        Self {
            result_id: "demo-recall-card".to_string(),
            provider: "focusa-local".to_string(),
            source_session_id: "unknown".to_string(),
            project_root: project_root.to_string(),
            continuity_id: "current".to_string(),
            timestamp: "now".to_string(),
            span_type: "workpoint".to_string(),
            memory_status: MemoryStatus::Active,
            scope_status: ScopeStatus::Current,
            proof_status: ProofStatus::None,
            allowed_use: AllowedUse::InspectOnly,
            safe_excerpt: "Advisory recall excerpt; verify scope and proof before promotion."
                .to_string(),
            evidence_refs: Vec::new(),
            next_action: "verify_first".to_string(),
        }
    }
}

pub const MEMORY_STATUS_VALUES: &[&str] = &[
    "active",
    "stale",
    "superseded",
    "contradicted",
    "noise",
    "quarantined",
];
pub const SCOPE_STATUS_VALUES: &[&str] = &[
    "current",
    "same_project_other_continuity",
    "other_project",
    "global_advisory",
];
pub const PROOF_STATUS_VALUES: &[&str] = &["none", "linked", "verified"];
pub const ALLOWED_USE_VALUES: &[&str] = &["include", "inspect_only", "verify_first", "exclude"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkpointCandidatePromotion {
    pub source: &'static str,
    pub candidate_kind: &'static str,
    pub authority_gate: &'static str,
    pub proof_gate: &'static str,
    pub approval_gate: &'static str,
    pub canonical_write: &'static str,
    /// Spec 119 §7.11 preview-before-commit invariant. Always true
    /// until operator approval has been recorded.
    pub preview_state: &'static str,
}

impl WorkpointCandidatePromotion {
    pub fn recall_default() -> Self {
        Self {
            source: "RecallDeckCard",
            candidate_kind: "advisory_workpoint_candidate",
            authority_gate: "verify project_root + continuity_id + Context Authority preflight",
            proof_gate: "require proof_status linked|verified or explicit proof-gap acknowledgement",
            approval_gate: "explicit operator approval required",
            canonical_write: "canonical Workpoint checkpoint only after approval",
            preview_state: "preview_only_until_operator_approval",
        }
    }

    /// True when the candidate has not yet been approved; canonical
    /// write must remain blocked.
    pub fn is_preview_only(&self) -> bool {
        self.preview_state == "preview_only_until_operator_approval"
    }
}

pub const WORKPOINT_CANDIDATE_PROMOTION_FLOW: &[&str] = &[
    "recall_search",
    "recall_deck_card",
    "verify_project_root_and_continuity_id",
    "context_authority_preflight",
    "proof_check",
    "render_workpoint_candidate",
    "operator_approval",
    "canonical_workpoint_checkpoint",
];

pub const WORKPOINT_CANDIDATE_FORBIDDEN: &[&str] = &[
    "recall_direct_canonical_write",
    "promotion_without_scope_verification",
    "promotion_without_operator_approval",
    "promotion_without_proof_or_explicit_gap",
];

pub const RECALL_AUTHORITY_RULE: &str = "Recall is advisory: inspect/verify first; canonical Workpoint promotion requires operator approval.";
pub const RECALL_FORBIDDEN_RULE: &str =
    "Recall cannot directly create canonical Workpoint authority";

pub fn render(app: &App, frame: &mut ratatui::Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),
            Constraint::Length(8),
            Constraint::Min(0),
        ])
        .split(area);

    render_search_card(frame, chunks[0]);
    render_preview_card(app, frame, chunks[1]);
    render_authority_card(frame, chunks[2]);
}

fn render_search_card(frame: &mut ratatui::Frame, area: Rect) {
    let text = vec![
        Line::from(vec![
            Span::styled("/ Mission Recall", theme::title()),
            Span::raw(" — search prior context without weakening authority"),
        ]),
        Line::from(
            "Search sources: events, Workpoints, evidence, audit timeline, bootstrap packets, agent imports, UIAI diagnostics, notes.",
        ),
        Line::from(
            "Use: inspect first → verify scope/proof → promote candidate only after approval.",
        ),
    ];
    let block = Block::default()
        .title(" Recall Search ")
        .title_style(theme::title())
        .borders(Borders::ALL)
        .border_style(theme::border());
    frame.render_widget(
        Paragraph::new(text).block(block).wrap(Wrap { trim: true }),
        area,
    );
}

fn render_preview_card(app: &App, frame: &mut ratatui::Frame, area: Rect) {
    let project = app
        .extra_data
        .get("project_identity")
        .and_then(|value| value.as_ref())
        .and_then(|value| value.get("root").or_else(|| value.get("project_root")))
        .and_then(|value| value.as_str())
        .unwrap_or("current project unknown");
    let card = RecallDeckCard::demo(project);
    let text = vec![
        Line::from("RecallDeckCard preview:"),
        Line::from(format!(
            "provider: {} · project_root: {}",
            card.provider, card.project_root
        )),
        Line::from("memory_status: active|stale|superseded|contradicted|noise|quarantined"),
        Line::from(
            "scope_status: current|same_project_other_continuity|other_project|global_advisory",
        ),
        Line::from(
            "proof_status: none|linked|verified · allowed_use: include|inspect_only|verify_first|exclude",
        ),
    ];
    let block = Block::default()
        .title(" RecallDeckCard ")
        .title_style(theme::title())
        .borders(Borders::ALL)
        .border_style(theme::border());
    frame.render_widget(
        Paragraph::new(text).block(block).wrap(Wrap { trim: true }),
        area,
    );
}

fn render_authority_card(frame: &mut ratatui::Frame, area: Rect) {
    let text = vec![
        Line::from(RECALL_AUTHORITY_RULE),
        Line::from(RECALL_FORBIDDEN_RULE),
        Line::from(
            "Promotion flow: search → card → verify project/continuity → authority preflight → proof check → candidate → operator approval → checkpoint.",
        ),
        Line::from("Hotkeys: / Recall · h/? help · d Deck Home · n next safe action · q quit"),
    ];
    let block = Block::default()
        .title(" Recall Authority Boundary ")
        .title_style(theme::title())
        .borders(Borders::ALL)
        .border_style(theme::border());
    frame.render_widget(
        Paragraph::new(text).block(block).wrap(Wrap { trim: true }),
        area,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recall_sources_cover_spec_inputs() {
        let joined = RECALL_SEARCH_SOURCES.join("\n");
        for required in [
            "Focusa events",
            "Workpoints",
            "Evidence refs",
            "UIAI diagnostics packets",
        ] {
            assert!(joined.contains(required), "missing {required}");
        }
    }

    #[test]
    fn recall_deck_card_schema_values_match_spec() {
        assert_eq!(MEMORY_STATUS_VALUES.len(), 6);
        assert_eq!(SCOPE_STATUS_VALUES.len(), 4);
        assert_eq!(PROOF_STATUS_VALUES, &["none", "linked", "verified"]);
        assert_eq!(
            ALLOWED_USE_VALUES,
            &["include", "inspect_only", "verify_first", "exclude"]
        );
        let card = RecallDeckCard::demo("/tmp/project");
        assert_eq!(card.provider, "focusa-local");
        assert_eq!(card.allowed_use, AllowedUse::InspectOnly);
    }

    #[test]
    fn workpoint_candidate_preview_state_blocks_canonical_write() {
        let p = WorkpointCandidatePromotion::recall_default();
        assert_eq!(p.preview_state, "preview_only_until_operator_approval");
        assert!(p.is_preview_only());
        // Spec 119 §7.11: canonical_write must remain forbidden until operator approval.
        let forbidden = WORKPOINT_CANDIDATE_FORBIDDEN.join("\n");
        assert!(forbidden.contains("promotion_without_operator_approval"));
    }

    #[test]
    fn workpoint_candidate_promotion_is_guarded() {
        let flow = WORKPOINT_CANDIDATE_PROMOTION_FLOW.join("→");
        for required in [
            "recall_deck_card",
            "verify_project_root_and_continuity_id",
            "context_authority_preflight",
            "proof_check",
            "operator_approval",
            "canonical_workpoint_checkpoint",
        ] {
            assert!(flow.contains(required), "missing {required}");
        }
        let forbidden = WORKPOINT_CANDIDATE_FORBIDDEN.join(
            "
",
        );
        assert!(forbidden.contains("recall_direct_canonical_write"));
        let promotion = WorkpointCandidatePromotion::recall_default();
        assert!(promotion.approval_gate.contains("operator approval"));
    }

    #[test]
    fn recall_card_fields_cover_authority_labels() {
        let joined = RECALL_CARD_FIELDS.join("\n");
        for required in [
            "memory_status",
            "scope_status",
            "proof_status",
            "allowed_use",
        ] {
            assert!(joined.contains(required), "missing {required}");
        }
        assert!(RECALL_AUTHORITY_RULE.contains("advisory"));
        assert!(RECALL_AUTHORITY_RULE.contains("operator approval"));
    }
}
