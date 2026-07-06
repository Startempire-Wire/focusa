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

pub const RECALL_AUTHORITY_RULE: &str = "Recall is advisory: inspect/verify first; canonical Workpoint promotion requires operator approval.";

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
    let text = vec![
        Line::from("RecallDeckCard preview:"),
        Line::from(format!("provider: focusa-local · project_root: {project}")),
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
