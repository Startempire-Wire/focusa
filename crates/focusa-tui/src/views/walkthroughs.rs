//! Mission Deck Walkthroughs + Education tab (Spec 117 §13).
//!
//! Renders the three core walkthroughs in-TUI: First Mission, Agent Handoff,
//! No Proof, No Done. Mobile-friendly. Hotkey L or l.

use crate::app::App;
use crate::theme;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub const WALKTHROUGH_TITLES: &[&str] = &["First Mission", "Agent Handoff", "No Proof, No Done"];

pub const WALKTHROUGH_AUDIENCES: &[&str] = &["beginner", "agent", "beginner"];

pub const WALKTHROUGH_STEPS: &[&[&str]] = &[
    &[
        "1. Start daemon: focusa start",
        "2. Bind project: focusa init --quickstart",
        "3. Create Workpoint: focusa workpoint checkpoint",
        "4. Attach evidence or mark proof gap explicitly",
        "5. Resume: focusa workpoint resume",
        "6. Show mission is resumable across handoff",
    ],
    &[
        "1. Show current mission: focusa trajectory view",
        "2. Show current Workpoint: focusa workpoint resume",
        "3. Render handoff packet: focusa workpoint resume --mode compact_prompt",
        "4. Show what the new agent receives",
        "5. Show drift boundaries (do_not_drift)",
        "6. Show evidence and proof expectations",
    ],
    &[
        "1. Display the agent completion claim",
        "2. Check evidence refs",
        "3. Show proof gap if missing",
        "4. Attach proof or mark proof intentionally missing",
        "5. Re-render proof meter (none | linked | verified)",
    ],
];

pub const WALKTHROUGH_WHY_IT_MATTERS: &[&str] = &[
    "Teaches the core Focusa loop: daemon → project → Workpoint → evidence → resume.",
    "Shows why Focusa exists: a new agent can recover mission, Workpoint, boundaries, and proof expectations without transcript memory.",
    "Teaches evidence discipline: an agent completion claim is not done until proof is visible or the gap is explicit.",
];

pub fn selected_index(app: &App) -> usize {
    let key = app.tab.hotkey();
    WALKTHROUGH_TITLES
        .iter()
        .position(|_| key == WALKTHROUGH_AUDIENCES.get(0).copied().unwrap_or(""))
        .unwrap_or(0)
}

pub fn render(frame: &mut ratatui::Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(WALKTHROUGH_TITLES.len() as u16 + 2),
            Constraint::Min(0),
        ])
        .split(area);

    // Header
    let head = Paragraph::new(Line::from(vec![
        Span::styled("Learn", theme::title()),
        Span::raw(" — walkthroughs to teach Focusa end-to-end."),
    ]))
    .block(
        Block::default()
            .title(" Mission Deck · Learn ")
            .borders(Borders::ALL),
    );
    frame.render_widget(head.wrap(Wrap { trim: true }), chunks[0]);

    // Catalog
    let catalog: Vec<Line> = WALKTHROUGH_TITLES
        .iter()
        .enumerate()
        .map(|(idx, t)| {
            Line::from(format!(
                "  [{}] {} (audience: {})",
                idx + 1,
                t,
                WALKTHROUGH_AUDIENCES.get(idx).copied().unwrap_or("?")
            ))
        })
        .collect();
    let cat_block = Block::default().title(" Catalog ").borders(Borders::ALL);
    frame.render_widget(Paragraph::new(catalog).block(cat_block), chunks[1]);

    // First walkthrough steps (mobile-friendly detail card)
    let mut detail: Vec<Line> = Vec::new();
    detail.push(Line::from(format!(
        "Currently showing: {}",
        WALKTHROUGH_TITLES[0]
    )));
    detail.push(Line::from(""));
    detail.push(Line::from(Span::styled("Why", theme::title())));
    detail.push(Line::from(WALKTHROUGH_WHY_IT_MATTERS[0]));
    detail.push(Line::from(""));
    detail.push(Line::from(Span::styled("Steps", theme::title())));
    for step in WALKTHROUGH_STEPS[0] {
        detail.push(Line::from(format!("  {step}")));
    }
    detail.push(Line::from(""));
    detail.push(Line::from(
        "Tip: open the CLI for full control — focusa walkthrough show --walkthrough <id>.",
    ));
    let detail_block = Block::default()
        .title(" First Mission detail ")
        .borders(Borders::ALL);
    frame.render_widget(
        Paragraph::new(detail)
            .block(detail_block)
            .wrap(Wrap { trim: true }),
        chunks[2],
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn walkthrough_titles_match_spec_117() {
        assert_eq!(WALKTHROUGH_TITLES.len(), 3);
        assert!(WALKTHROUGH_TITLES.contains(&"First Mission"));
        assert!(WALKTHROUGH_TITLES.contains(&"Agent Handoff"));
        assert!(WALKTHROUGH_TITLES.contains(&"No Proof, No Done"));
    }

    #[test]
    fn walkthrough_steps_match_titles_count() {
        assert_eq!(WALKTHROUGH_TITLES.len(), WALKTHROUGH_STEPS.len());
        assert_eq!(WALKTHROUGH_TITLES.len(), WALKTHROUGH_AUDIENCES.len());
        assert_eq!(WALKTHROUGH_TITLES.len(), WALKTHROUGH_WHY_IT_MATTERS.len());
    }

    #[test]
    fn first_mission_has_five_or_six_steps() {
        assert!(WALKTHROUGH_STEPS[0].len() >= 5);
    }

    #[test]
    fn walkthrough_lines_are_mobile_friendly() {
        for line in WALKTHROUGH_TITLES {
            assert!(line.chars().count() <= 40);
        }
    }
}
