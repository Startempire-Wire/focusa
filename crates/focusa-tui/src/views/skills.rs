//! Agent Skills view.

use crate::app::App;
use crate::theme;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render(app: &App, frame: &mut ratatui::Frame, area: Rect) {
    let block = Block::default()
        .title(" Agent Skills ")
        .title_style(theme::title())
        .borders(Borders::ALL)
        .border_style(theme::border());

    let data = &app.extra_data.get("skills");
    let mut lines = Vec::new();

    if let Some(Some(d)) = data {
        let total = d["total"].as_u64().unwrap_or(0);
        lines.push(Line::from(vec![
            Span::styled("Total Skills: ", theme::label()),
            Span::styled(total.to_string(), theme::value()),
        ]));
        lines.push(Line::default());

        if let Some(skills) = d["skills"].as_array() {
            let mut current_cat = String::new();
            for s in skills {
                let cat = s["category"].as_str().unwrap_or("?");
                if cat != current_cat {
                    lines.push(Line::default());
                    lines.push(Line::from(format!("  {cat}:")).style(theme::label()));
                    current_cat = cat.to_string();
                }
                let id = s["id"].as_str().unwrap_or("?");
                let name = s["name"].as_str().unwrap_or("?");
                let enabled = s["enabled"].as_bool().unwrap_or(false);
                let icon = if enabled { "✓" } else { "✗" };
                let color = if enabled { Color::Green } else { Color::DarkGray };
                lines.push(Line::from(vec![
                    Span::styled(format!("    {icon} "), Style::default().fg(color)),
                    Span::styled(format!("{id:20} "), theme::value()),
                    Span::styled(name, Style::default().fg(Color::DarkGray)),
                ]));
            }
        }

        if let Some(prohibited) = d["prohibited"].as_array() {
            lines.push(Line::default());
            lines.push(Line::from("  Prohibited:").style(theme::status_err()));
            for p in prohibited {
                lines.push(Line::from(vec![
                    Span::styled("    ✗ ", Style::default().fg(Color::Red)),
                    Span::styled(p.as_str().unwrap_or("?"), theme::value()),
                ]));
            }
        }
    } else {
        lines.push(Line::from("  No skills data").style(theme::label()));
    }

    let para = Paragraph::new(lines)
        .block(block)
        .scroll((app.scroll_offset, 0));
    frame.render_widget(para, area);
}
