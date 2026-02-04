//! Training Dataset Export view.

use crate::app::App;
use crate::theme;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render(app: &App, frame: &mut ratatui::Frame, area: Rect) {
    let block = Block::default()
        .title(" Training Export & Contribution ")
        .title_style(theme::title())
        .borders(Borders::ALL)
        .border_style(theme::border());

    let data = &app.extra_data.get("training");
    let mut lines = Vec::new();

    if let Some(Some(d)) = data {
        let enabled = d["contribution_enabled"].as_bool().unwrap_or(false);
        let queue_size = d["queue_size"].as_u64().unwrap_or(0);
        let total = d["total_contributed"].as_u64().unwrap_or(0);

        lines.push(Line::from(vec![
            Span::styled("Contribution: ", theme::label()),
            Span::styled(if enabled { "enabled" } else { "disabled" }, theme::value()),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Queue Size: ", theme::label()),
            Span::styled(queue_size.to_string(), theme::value()),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Total Contributed: ", theme::label()),
            Span::styled(total.to_string(), theme::value()),
        ]));

        if let Some(policy) = d.get("policy") {
            lines.push(Line::default());
            lines.push(Line::from("  Policy:").style(theme::label()));
            lines.push(Line::from(vec![
                Span::styled("    auto_contribute: ", theme::label()),
                Span::styled(policy["auto_contribute"].to_string(), theme::value()),
            ]));
            lines.push(Line::from(vec![
                Span::styled("    require_review: ", theme::label()),
                Span::styled(policy["require_review"].to_string(), theme::value()),
            ]));
            lines.push(Line::from(vec![
                Span::styled("    anonymize: ", theme::label()),
                Span::styled(policy["anonymize"].to_string(), theme::value()),
            ]));
        }
    } else {
        lines.push(Line::from("  No training data").style(theme::label()));
    }

    let para = Paragraph::new(lines)
        .block(block)
        .scroll((app.scroll_offset, 0));
    frame.render_widget(para, area);
}
