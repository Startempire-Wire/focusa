//! Modal overlays (Spec 117 §6 launch polish).
//!
//! ModalKind::Recall | Learn | Help | About | CommandPalette.
//! Modals replace the canvas (not stacked). Esc closes.

use crate::app::{App, ModalKind};
use crate::theme;
use crate::views::intro::FOCUSA_LOGO;
use crate::views::walkthroughs;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render_modal(modal: ModalKind, app: &App, frame: &mut ratatui::Frame, area: Rect) {
    let (title, body) = build_body(modal, app, area.width);
    let popup = centered(75, 70, area);
    frame.render_widget(Clear, popup);
    let p = Paragraph::new(body)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(theme::border()),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(p, popup);
}

fn build_body(modal: ModalKind, app: &App, width: u16) -> (String, Vec<Line<'static>>) {
    match modal {
        ModalKind::Recall => (modal.title().to_string(), recall_body(app)),
        ModalKind::Learn => (modal.title().to_string(), learn_body(app)),
        ModalKind::Help => (modal.title().to_string(), help_body()),
        ModalKind::About => (modal.title().to_string(), about_body()),
        ModalKind::CommandPalette => (modal.title().to_string(), palette_body(app)),
    }
}

fn recall_body(app: &App) -> Vec<Line<'static>> {
    vec![
        Line::from(Span::styled(
            "Sources (advisory): events, workpoints, evidence refs, audit timeline,",
            theme::label(),
        )),
        Line::from("  agent bootstrap, UIAI diagnostics, manual session notes."),
        Line::from(""),
        Line::from(Span::styled("Recall is advisory only.", theme::title())),
        Line::from("Promote to Workpoint candidate only after operator approval."),
        Line::from(""),
        Line::from(format!("Continuity: {}", short(project_root(app), 48))),
        Line::from("Press Esc to close."),
    ]
}

fn learn_body(app: &App) -> Vec<Line<'static>> {
    let mut out: Vec<Line<'static>> = Vec::new();
    let titles = walkthroughs::WALKTHROUGH_TITLES;
    let audiences = walkthroughs::WALKTHROUGH_AUDIENCES;
    out.push(Line::from(Span::styled(
        "Walkthroughs available:",
        theme::title(),
    )));
    for (i, t) in titles.iter().enumerate() {
        let aud = audiences.get(i).copied().unwrap_or("?");
        out.push(Line::from(format!(
            "  [{}] {} · audience: {}",
            i + 1,
            t,
            aud
        )));
    }
    out.push(Line::from(""));
    out.push(Line::from(Span::styled(
        "Tip: open CLI for full control — focusa walkthrough show --walkthrough <id>",
        theme::label(),
    )));
    out.push(Line::from(format!(
        "Press Esc to close · current selection {}",
        app.modal_selection
    )));
    out
}

fn help_body() -> Vec<Line<'static>> {
    vec![
        Line::from(Span::styled(
            "Concepts overlay (Workpoint, Evidence, Recall, Mission Ladder).",
            theme::title(),
        )),
        Line::from(""),
        Line::from(
            "Workpoint     the saved mission: intent, current action, next action, evidence, do-not-drift.",
        ),
        Line::from(
            "Evidence      a test, file, screenshot, command output, or URL proving the claim.",
        ),
        Line::from(
            "Recall        Focusa remembering the mission after compaction, restart, or handoff.",
        ),
        Line::from(
            "Mission Ladder HLT (long-term goal) > MLG (mid-level) > STG (short-term) > Workpoint > Evidence.",
        ),
        Line::from(
            "Authority     canonical (safe to act) | advisory (review) | blocked (rebind) | unbound.",
        ),
        Line::from(""),
        Line::from("Press Esc to close."),
    ]
}

fn about_body() -> Vec<Line<'static>> {
    let credits = crate::views::about::ABOUT_CREDITS;
    let mut out: Vec<Line<'static>> = Vec::new();
    out.push(Line::from(Span::styled(FOCUSA_LOGO, theme::title())));
    out.push(Line::from(Span::styled(
        crate::views::intro::FOCUSA_TAGLINE,
        theme::label(),
    )));
    out.push(Line::from(""));
    out.push(Line::from(format!(
        "Version       {}",
        crate::views::about::ABOUT_VERSION
    )));
    out.push(Line::from(format!(
        "Build         {}",
        crate::views::about::ABOUT_BUILD_INFO
    )));
    out.push(Line::from("License       BSL-1.1 (source-available)"));
    out.push(Line::from(""));
    out.push(Line::from(Span::styled("Credits", theme::title())));
    for c in credits {
        out.push(Line::from(format!("  • {c}")));
    }
    out.push(Line::from(""));
    out.push(Line::from("Press Esc to close."));
    out
}

fn palette_body(app: &App) -> Vec<Line<'static>> {
    let q = if app.palette_buffer.is_empty() {
        "type a command and press Enter"
    } else {
        app.palette_buffer.as_str()
    };
    let mut out: Vec<Line<'static>> = Vec::new();
    out.push(Line::from(Span::styled(
        "Pick an action (Enter to run, Esc to close):",
        theme::title(),
    )));
    out.push(Line::from(""));
    out.push(Line::from(format!("  > {q}")));
    out.push(Line::from(""));
    out.push(Line::from(Span::styled(
        "Available commands:",
        theme::label(),
    )));
    for (cmd, desc) in palette_commands() {
        out.push(Line::from(format!("  {cmd:<14} {desc}")));
    }
    out.push(Line::from(""));
    out.push(Line::from("Press Esc to close."));
    out
}

fn palette_commands() -> &'static [(&'static str, &'static str)] {
    &[
        ("recall", "open Recall modal"),
        ("learn", "open Learn modal"),
        ("help", "open Help modal"),
        ("about", "open About modal"),
        ("refresh", "refresh Focusa daemon state"),
        ("dismiss-intro", "force-dismiss the welcome intro"),
        ("quit", "quit Mission Deck"),
    ]
}

fn project_root(app: &App) -> &str {
    if let Some(wp) = app
        .extra_data
        .get("workpoint_resume")
        .and_then(|v| v.as_ref())
    {
        if let Some(id) = wp.get("project_root").and_then(|v| v.as_str()) {
            return id;
        }
    }
    "(not bound)"
}

fn short(text: &str, max_chars: usize) -> String {
    let chars: String = text.chars().collect();
    if chars.chars().count() <= max_chars {
        return chars;
    }
    let mut cut = String::new();
    for (i, c) in chars.chars().enumerate() {
        if i + 1 >= max_chars {
            cut.push('…');
            break;
        }
        cut.push(c);
    }
    cut
}

fn centered(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centered_does_not_exceed_area() {
        let r = centered(80, 80, Rect::new(0, 0, 100, 40));
        assert!(r.width <= 100);
        assert!(r.height <= 40);
    }

    #[test]
    fn short_truncates_with_ellipsis() {
        assert!(short("abcdefghijklmnop", 4).ends_with('…'));
        assert_eq!(short("abcd", 8), "abcd");
    }
}
