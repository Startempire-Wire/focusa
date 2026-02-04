//! Proposals (PRE) view.

use crate::app::App;
use crate::theme;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render(app: &App, frame: &mut ratatui::Frame, area: Rect) {
    let block = Block::default()
        .title(" Proposal Resolution Engine ")
        .title_style(theme::title())
        .borders(Borders::ALL)
        .border_style(theme::border());

    let data = &app.extra_data.get("proposals");
    let mut lines = Vec::new();

    if let Some(Some(d)) = data {
        let pending = d["pending"].as_u64().unwrap_or(0);
        lines.push(Line::from(vec![
            Span::styled("Pending: ", theme::label()),
            Span::styled(pending.to_string(), theme::value()),
        ]));
        lines.push(Line::default());

        if let Some(proposals) = d["proposals"].as_array() {
            for p in proposals {
                let id = p["id"].as_str().unwrap_or("?");
                let kind = p["kind"].as_str().unwrap_or("?");
                let status = p["status"].as_str().unwrap_or("?");
                let score = p["score"].as_f64().unwrap_or(0.0);
                let short_id = if id.len() >= 8 { &id[..8] } else { id };

                let status_color = match status {
                    "pending" => Color::Yellow,
                    "accepted" => Color::Green,
                    "rejected" => Color::Red,
                    _ => Color::DarkGray,
                };

                lines.push(Line::from(vec![
                    Span::styled(format!("{short_id} "), Style::default().fg(Color::DarkGray)),
                    Span::styled(format!("[{kind}] "), theme::label()),
                    Span::styled(format!("score={score:.2} "), theme::value()),
                    Span::styled(status, Style::default().fg(status_color)),
                ]));
            }
        }
    } else {
        lines.push(Line::from("  No proposal data").style(theme::label()));
    }

    let para = Paragraph::new(lines)
        .block(block)
        .scroll((app.scroll_offset, 0));
    frame.render_widget(para, area);
}
