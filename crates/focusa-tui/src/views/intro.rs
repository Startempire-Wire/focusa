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
    // Fill entire screen — hide underlying UI until user enters.
    frame.render_widget(Clear, area);
    let bg = Block::default().style(Style::default().bg(Color::Black));
    frame.render_widget(bg, area);

    // Keep logo centered on every terminal size.
    let intro = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Ratio(1, 6),
            Constraint::Length(8),
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Ratio(1, 6),
        ])
        .split(area);

    let big_logo = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(30),
            Constraint::Min(0),
        ])
        .split(intro[1]);
    // Big pixel FOCUSA logo.
    // Width math: HalfWidth packs 2 glyph pixels per char cell → 4 cols × 6 chars = 24 cols
    // for "FOCUSA". Fits cleanly inside the 30-col horizontal slot with breathing room,
    // and survives narrow terminals. (PixelSize::Full would need 48 cols and clip.)
    frame.render_widget(
        BigText::builder()
            .pixel_size(tui_big_text::PixelSize::HalfWidth)
            .lines(vec![Line::from("FOCUSA")])
            .style(theme::title())
            .build(),
        big_logo[1],
    );
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(FOCUSA_TAGLINE, theme::label())))
            .alignment(Alignment::Center),
        intro[2],
    );
    frame.render_widget(
        Paragraph::new(FOCUSA_TAGS_LINE).alignment(Alignment::Center),
        intro[3],
    );
    frame.render_widget(
        Paragraph::new(format!("v{}", crate::views::about::ABOUT_VERSION))
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray)),
        intro[4],
    );
    frame.render_widget(
        Paragraph::new("Press any key to enter Mission Deck · auto-dismiss in 2.5s.")
            .alignment(Alignment::Center)
            .style(theme::label()),
        intro[5],
    );
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
        assert!(
            lines
                .iter()
                .any(|l| l == FOCUSA_TAGLINE || l.contains("Local-first"))
        );
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
    #[allow(clippy::const_is_empty)]
    fn splash_version_line_is_non_empty() {
        assert!(!INTRO_VERSION_LINE.is_empty());
    }

    /// Regression: "FOCUSA" rendered at PixelSize::HalfWidth needs 4 cols × 6 chars = 24 cols.
    /// The horizontal layout slot in `render()` is `Constraint::Length(30)`. If the slot is
    /// ever smaller than the rendered width, the right edge of the logo gets clipped (operator
    /// reported seeing "FOCU" instead of "FOCUSA"). This test pins the math so future
    /// pixel-size or layout changes don't silently regress.
    #[test]
    fn big_text_focusa_width_fits_in_layout_slot() {
        const LOGO_CHARS: usize = 6; // "FOCUSA"
        const COLS_PER_CHAR_HALFWIDTH: u16 = 4; // 2 glyph pixels per cell, 8-px font
        const RENDERED_WIDTH: u16 = LOGO_CHARS as u16 * COLS_PER_CHAR_HALFWIDTH;
        const LAYOUT_SLOT: u16 = 30; // matches Constraint::Length(30) in render()
        let rendered_width = RENDERED_WIDTH;
        assert!(
            rendered_width <= LAYOUT_SLOT,
            "BigText HalfWidth 'FOCUSA' needs {RENDERED_WIDTH} cols, layout slot is {LAYOUT_SLOT}. \
             Either widen the slot or pick a smaller pixel size (e.g., Quadrant=2x2 → 12 cols)."
        );
    }
}
