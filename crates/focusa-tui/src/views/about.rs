//! Mission Deck About tab (Spec 117 §6 polish).
//!
//! Version, build info, telemetry opt-in state, credits, canonical FOCUSA
//! LOGO & TAGLINE, and other important data.

use crate::views::intro::{FOCUSA_LOGO, FOCUSA_TAGLINE, FOCUSA_TAGS_LINE};
use ratatui::prelude::*;
use ratatui::widgets::*;

pub const ABOUT_BUILD_INFO: &str = concat!("release · ", env!("CARGO_PKG_VERSION"));
pub const ABOUT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const ABOUT_RUSTC: &str = "rustc nightly (2024 edition)";
pub const ABOUT_WORKSPACE: &str = "rust-monorepo";
pub const ABOUT_TELEMETRY: &str = "opt-in: off by default · see FOCUSA_TELEMETRY env";
pub const ABOUT_CREDITS: &[&str] = &[
    "FOCUSA team — Verious Smith (owner) and contributors",
    "Mission Deck, Bloatgaurd, Context Cognition — Focusa core",
    "Spec 101 / 117 / 119 — focusa-spec working group",
    "Pi tool, menubar, menubar web — Focusa integrations",
    "UIAI Engine, Paste, Flow — operator preview integrations",
];
pub const ABOUT_PUBLIC_DOCS: &str =
    "https://github.com/Startempire-Wire/focusa + docs/PUBLIC_DOCS_SYNC.md";
pub const ABOUT_POSTCARD: &str = "docs/RELEASE_INSTALL_POSTCARD.md";
pub const ABOUT_GTM: &str = "docs/GTM_FIVE_MINUTE_PROOF.md";
pub const ABOUT_NEWBIE_QA: &str = "docs/NEWBIE_ONBOARDING_WALKTHROUGH_QA.md";
pub const ABOUT_LICENSE: &str = "BSL-1.1 (source-available)";

pub fn about_lines() -> Vec<(String, String)> {
    vec![
        ("Logo".to_string(), FOCUSA_LOGO.to_string()),
        ("Tagline".to_string(), FOCUSA_TAGLINE.to_string()),
        ("Tags".to_string(), FOCUSA_TAGS_LINE.to_string()),
        ("Version".to_string(), ABOUT_VERSION.to_string()),
        ("Build".to_string(), ABOUT_BUILD_INFO.to_string()),
        ("Toolchain".to_string(), ABOUT_RUSTC.to_string()),
        ("Workspace".to_string(), ABOUT_WORKSPACE.to_string()),
        ("Telemetry".to_string(), ABOUT_TELEMETRY.to_string()),
        ("License".to_string(), ABOUT_LICENSE.to_string()),
        ("Public docs".to_string(), ABOUT_PUBLIC_DOCS.to_string()),
        ("Install postcard".to_string(), ABOUT_POSTCARD.to_string()),
        ("Five-minute proof".to_string(), ABOUT_GTM.to_string()),
        ("Newbie QA".to_string(), ABOUT_NEWBIE_QA.to_string()),
    ]
}

pub fn credits_lines() -> Vec<String> {
    ABOUT_CREDITS.iter().map(|s| s.to_string()).collect()
}

pub fn render(frame: &mut ratatui::Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),
            Constraint::Length(about_lines().len() as u16 + 2),
            Constraint::Min(0),
        ])
        .split(area);

    // Header card with FOCUSA LOGO & TAGLINE
    let head = vec![
        Line::from(Span::styled(
            FOCUSA_LOGO,
            Style::new().bold().fg(Color::Cyan),
        )),
        Line::from(Span::styled(FOCUSA_TAGLINE, Style::new().italic())),
        Line::from(FOCUSA_TAGS_LINE),
        Line::from(""),
        Line::from(format!("Version {}", ABOUT_VERSION)),
        Line::from(format!("Build   {}", ABOUT_BUILD_INFO)),
        Line::from(format!("{} · {}", ABOUT_RUSTC, ABOUT_WORKSPACE)),
    ];
    let head_block = Block::default()
        .title(" About Focusa ")
        .borders(Borders::ALL);
    frame.render_widget(
        Paragraph::new(head)
            .block(head_block)
            .wrap(Wrap { trim: true }),
        chunks[0],
    );

    // Detail card
    let detail: Vec<Line> = about_lines()
        .into_iter()
        .map(|(k, v)| Line::from(format!("  {k:<14} {v}")))
        .collect();
    let detail_block = Block::default()
        .title(" Build · Telemetry · Docs ")
        .borders(Borders::ALL);
    frame.render_widget(
        Paragraph::new(detail)
            .block(detail_block)
            .wrap(Wrap { trim: true }),
        chunks[1],
    );

    // Credits card
    let credits: Vec<Line> = credits_lines()
        .into_iter()
        .map(|s| Line::from(format!("  • {s}")))
        .collect();
    let credits_block = Block::default().title(" Credits ").borders(Borders::ALL);
    frame.render_widget(
        Paragraph::new(credits)
            .block(credits_block)
            .wrap(Wrap { trim: true }),
        chunks[2],
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn about_lines_include_logo_and_tagline() {
        let lines = about_lines();
        let keys: Vec<&str> = lines.iter().map(|(k, _)| k.as_str()).collect();
        assert!(keys.contains(&"Logo"));
        assert!(keys.contains(&"Tagline"));
        assert!(keys.contains(&"Version"));
        assert!(keys.contains(&"Telemetry"));
        assert!(keys.contains(&"Credits"));
    }

    #[test]
    fn credits_include_owner_and_specs() {
        let lines = credits_lines();
        assert!(lines.iter().any(|l| l.contains("Verious Smith")));
        assert!(
            lines
                .iter()
                .any(|l| l.contains("Spec 101") || l.contains("Spec 117"))
        );
    }

    #[test]
    #[allow(clippy::const_is_empty)]
    fn about_version_is_non_empty() {
        assert!(!ABOUT_VERSION.is_empty());
        assert!(!ABOUT_BUILD_INFO.is_empty());
    }
}
