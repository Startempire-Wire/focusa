//! Intuition view — signal timeline and pattern clusters.
//!
//! Per 27-tui-spec §8: signal timeline, pattern clusters, confidence bands.

use crate::app::App;
use crate::theme;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render(app: &App, frame: &mut ratatui::Frame, area: Rect) {
    let block = Block::default()
        .title(" Intuition Engine — Signal Timeline ")
        .title_style(theme::title())
        .borders(Borders::ALL)
        .border_style(theme::border());

    // Get signals from state (via API)
    let signals = app.extra_data.get("signals").and_then(|v| v.as_ref());

    if signals.is_none() {
        let para = Paragraph::new("Loading intuition signals...")
            .style(theme::label())
            .block(block);
        frame.render_widget(para, area);
        return;
    }

    let data = signals.unwrap();
    let signal_list = data.get("signals").and_then(|v| v.as_array()).cloned().unwrap_or_default();

    if signal_list.is_empty() {
        let para = Paragraph::new("\nNo intuition signals observed yet.\n\nSignals are generated from user input patterns, tool outputs, and behavioral observations.")
            .style(theme::label())
            .block(block);
        frame.render_widget(para, area);
        return;
    }

    // Build rows for recent signals
    let rows: Vec<Row> = signal_list
        .iter()
        .take(15)
        .map(|s| {
            let kind = s.get("kind").and_then(|v| v.as_str()).unwrap_or("?");
            let origin = s.get("origin").and_then(|v| v.as_str()).unwrap_or("?");
            let summary = s.get("summary").and_then(|v| v.as_str()).unwrap_or("?");
            let summary_short: String = summary.chars().take(40).collect();
            
            Row::new(vec![
                Cell::from(kind.to_string()),
                Cell::from(origin.to_string()),
                Cell::from(summary_short),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(20),
            Constraint::Length(12),
            Constraint::Min(30),
        ],
    )
    .header(
        Row::new(vec!["Signal Kind", "Origin", "Summary"])
            .style(theme::value().add_modifier(Modifier::BOLD)),
    )
    .block(block);

    frame.render_widget(table, area);
}
