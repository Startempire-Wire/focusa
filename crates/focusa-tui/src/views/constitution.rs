//! Agent Constitution view.

use crate::app::App;
use crate::theme;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render(app: &App, frame: &mut ratatui::Frame, area: Rect) {
    let block = Block::default()
        .title(" Agent Constitution ")
        .title_style(theme::title())
        .borders(Borders::ALL)
        .border_style(theme::border());

    let data = &app.extra_data.get("constitution");
    let mut lines = Vec::new();

    if let Some(Some(d)) = data {
        if let Some(err) = d.get("error") {
            lines.push(Line::from(vec![
                Span::styled("  ", theme::label()),
                Span::styled(err.as_str().unwrap_or("Unknown error"), theme::status_err()),
            ]));
        } else {
            let version = d["version"].as_str().unwrap_or("?");
            let agent = d["agent_id"].as_str().unwrap_or("?");
            lines.push(Line::from(vec![
                Span::styled("Version: ", theme::label()),
                Span::styled(version, theme::value()),
            ]));
            lines.push(Line::from(vec![
                Span::styled("Agent: ", theme::label()),
                Span::styled(agent, theme::value()),
            ]));

            if let Some(principles) = d["principles"].as_array() {
                lines.push(Line::default());
                lines.push(Line::from("  Principles:").style(theme::label()));
                for p in principles {
                    let id = p["id"].as_str().unwrap_or("?");
                    let text = p["text"].as_str().unwrap_or("?");
                    lines.push(Line::from(vec![
                        Span::styled(format!("    [{id}] "), Style::default().fg(Color::DarkGray)),
                        Span::styled(text, theme::value()),
                    ]));
                }
            }
        }
    } else {
        lines.push(Line::from("  No constitution data").style(theme::label()));
    }

    let para = Paragraph::new(lines)
        .block(block)
        .scroll((app.scroll_offset, 0));
    frame.render_widget(para, area);
}
