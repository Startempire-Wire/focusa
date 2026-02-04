//! CLT (Context Lineage Tree) view.

use crate::app::App;
use crate::theme;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render(app: &App, frame: &mut ratatui::Frame, area: Rect) {
    let block = Block::default()
        .title(" Context Lineage Tree ")
        .title_style(theme::title())
        .borders(Borders::ALL)
        .border_style(theme::border());

    let clt = &app.extra_data.get("clt");
    let mut lines = Vec::new();

    if let Some(Some(data)) = clt {
        let total = data["total"].as_u64().unwrap_or(0);
        let head = data["head_id"].as_str().unwrap_or("none");
        lines.push(Line::from(vec![
            Span::styled("Head: ", theme::label()),
            Span::styled(head, theme::value()),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Total nodes: ", theme::label()),
            Span::styled(total.to_string(), theme::value()),
        ]));
        lines.push(Line::default());

        if let Some(nodes) = data["nodes"].as_array() {
            for n in nodes.iter().rev().take(20) {
                let id = n["node_id"].as_str().unwrap_or("?");
                let ntype = n["node_type"].as_str().unwrap_or("?");
                let parent = n["parent_id"].as_str().unwrap_or("root");
                lines.push(Line::from(vec![
                    Span::styled(format!("{id} "), theme::value()),
                    Span::styled(format!("[{ntype}] "), theme::label()),
                    Span::styled(format!("← {parent}"), Style::default().fg(Color::DarkGray)),
                ]));
            }
        }
    } else {
        lines.push(Line::from("  No CLT data available").style(theme::label()));
    }

    let para = Paragraph::new(lines)
        .block(block)
        .scroll((app.scroll_offset, 0));
    frame.render_widget(para, area);
}
