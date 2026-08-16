//! In-place help overlay for Mission Deck (Spec 117 §9).

use crate::theme;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub const HELP_TOPICS: &[&str] = &[
    "Workpoint — the saved mission state: objective, current action, proof, next action.",
    "Evidence — a test, file, screenshot, command output, or URL proving the claim.",
    "Recall — Focusa remembering the mission after compaction, restart, or handoff.",
    "Mission Ladder — high-level goal → current milestone → next safe action.",
    "Authority badges — canonical means safe to act; advisory means review first; blocked means stop and rebind.",
];

pub fn render(frame: &mut ratatui::Frame, area: Rect) {
    let popup = centered_rect(74, 62, area);
    frame.render_widget(Clear, popup);
    let text: Vec<Line> = std::iter::once(Line::from(vec![
        Span::styled("Mission Deck Help", theme::title()),
        Span::raw(" — press h or ? to close"),
    ]))
    .chain(std::iter::once(Line::from("")))
    .chain(
        HELP_TOPICS
            .iter()
            .map(|topic| Line::from(format!("• {topic}"))),
    )
    .chain(std::iter::once(Line::from("")))
    .chain(std::iter::once(Line::from(
        "Rule: Beginner Mode shows one primary next safe action, with why before commands.",
    )))
    .collect();
    let block = Block::default()
        .title(" Help Overlay ")
        .title_style(theme::title())
        .borders(Borders::ALL)
        .border_style(theme::border());
    frame.render_widget(
        Paragraph::new(text).block(block).wrap(Wrap { trim: true }),
        popup,
    );
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
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
    fn help_topics_cover_required_concepts() {
        let joined = HELP_TOPICS.join("\n");
        for required in [
            "Workpoint",
            "Evidence",
            "Recall",
            "Mission Ladder",
            "Authority badges",
        ] {
            assert!(joined.contains(required), "missing {required}");
        }
    }
}
