//! Mission Deck Welcome Intro splash (Spec 117 §6 polish).
//!
//! Logo + tagline + version. Mobile-friendly, dismissable, auto-timeout.

use crate::app::App;
use crate::theme;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub const FOCUSA_LOGO: &str = "FOCUSA";
pub const FOCUSA_TAGLINE: &str = "Local-first mission cohesion for AI coding agents.";

pub const INTRO_HEADLINE_LINES: &[&str] = &[FOCUSA_LOGO, FOCUSA_TAGLINE];

pub const INTRO_FOOTER_LINES: &[&str] = &[
    "",
    "Press any key to enter Mission Deck · auto-dismiss in 2.5s.",
    "Mission Deck · keep the mission, prove the handoff.",
];

pub const INTRO_VERSION_LINE: &str = "Focusa TUI · deck home default · ready.";

pub fn intro_lines() -> Vec<String> {
    let mut out: Vec<String> = INTRO_HEADLINE_LINES.iter().map(|s| s.to_string()).collect();
    out.extend(INTRO_FOOTER_LINES.iter().map(|s| s.to_string()));
    out.push(INTRO_VERSION_LINE.to_string());
    out
}

pub fn render(app: &App, frame: &mut ratatui::Frame, area: Rect) {
    if !app.show_intro {
        return;
    }
    let popup = centered(60, 50, area);
    frame.render_widget(Clear, popup);
    let lines: Vec<Line> = intro_lines()
        .into_iter()
        .enumerate()
        .map(|(idx, s)| {
            if idx == 0 {
                Line::from(Span::styled(s, theme::title()))
            } else if idx == 1 {
                Line::from(Span::styled(s, theme::label()))
            } else if s.is_empty() {
                Line::from("")
            } else {
                Line::from(s)
            }
        })
        .collect();
    let block = Block::default()
        .title(" Welcome to Focusa ")
        .title_style(theme::title())
        .borders(Borders::ALL)
        .border_style(theme::border());
    frame.render_widget(
        Paragraph::new(lines)
            .block(block)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true }),
        popup,
    );
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
    fn focusa_logo_and_tagline_match_canonical() {
        assert_eq!(FOCUSA_LOGO, "FOCUSA");
        assert_eq!(
            FOCUSA_TAGLINE,
            "Local-first mission cohesion for AI coding agents."
        );
        assert!(!FOCUSA_TAGLINE.starts_with(FOCUSA_LOGO));
    }

    #[test]
    fn intro_lines_include_logo_tagline_and_keypress_hint() {
        let lines = intro_lines();
        assert!(lines.iter().any(|l| l == FOCUSA_LOGO));
        assert!(lines.iter().any(|l| l == FOCUSA_TAGLINE));
        assert!(
            lines
                .iter()
                .any(|l| l.contains("Press any key") || l.contains("auto-dismiss"))
        );
    }

    #[test]
    fn intro_lines_are_mobile_friendly() {
        for line in intro_lines() {
            assert!(line.chars().count() <= 72, "line too long: {line}");
        }
    }

    #[test]
    fn splash_version_line_is_non_empty() {
        assert!(!INTRO_VERSION_LINE.is_empty());
    }
}
