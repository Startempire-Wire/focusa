//! Mission Deck Welcome Intro splash (Spec 117 §6 polish).
//!
//! Logo + tagline + version. Mobile-friendly, dismissable, auto-timeout.

use crate::app::App;
use crate::theme;
use ratatui::prelude::*;
use ratatui::style::Color;
use ratatui::widgets::*;
use tui_big_text::BigText;

pub const FOCUSA_LOGO: &str = "FOCUSA";
pub const FOCUSA_TAGLINE: &str = "Local-first mission cohesion for AI coding agents.";
pub const FOCUSA_TAGS: &[&str] = &["local-first", "evidence-backed", "handoff-ready"];
pub const FOCUSA_TAGS_LINE: &str = "Tags: local-first · evidence-backed · handoff-ready";

pub const INTRO_HEADLINE_LINES: &[&str] = &[FOCUSA_LOGO];

pub const INTRO_FOOTER_LINES: &[&str] = &[
    FOCUSA_TAGLINE,
    FOCUSA_TAGS_LINE,
    "",
    "Press any key to enter · auto-dismiss in 2.5s.",
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
    let popup = if area.width < 80 { centered(90, 60, area) } else { centered(70, 50, area) };
    frame.render_widget(Clear, popup);
    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(popup);
    // Big pixel FOCUSA logo
    frame.render_widget(
        BigText::builder()
            .pixel_size(tui_big_text::PixelSize::Full)
            .lines(vec![Line::from("FOCUSA")])
            .style(theme::title())
            .build(),
        inner[0],
    );
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(FOCUSA_TAGLINE, theme::label())))
            .alignment(Alignment::Center),
        inner[1],
    );
    frame.render_widget(
        Paragraph::new(FOCUSA_TAGS_LINE).alignment(Alignment::Center),
        inner[2],
    );
    frame.render_widget(
        Paragraph::new(format!("v{}", crate::views::about::ABOUT_VERSION))
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray)),
        inner[3],
    );
    frame.render_widget(
        Paragraph::new("Press any key to enter Mission Deck · auto-dismiss in 2.5s.")
            .alignment(Alignment::Center)
            .style(theme::label()),
        inner[4],
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
        assert_eq!(
            FOCUSA_TAGS,
            ["local-first", "evidence-backed", "handoff-ready"]
        );
    }

    #[test]
    fn intro_lines_include_logo_tagline_and_keypress_hint() {
        let lines = intro_lines();
        assert!(lines.iter().any(|l| l == FOCUSA_LOGO));
        assert!(lines.iter().any(|l| l == FOCUSA_TAGLINE || l.contains("Local-first")));
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
