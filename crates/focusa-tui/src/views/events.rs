//! Events view — live scrollable event stream.

use crate::app::App;
use crate::theme;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render(app: &App, frame: &mut ratatui::Frame, area: Rect) {
    let block = Block::default()
        .title(" Events ")
        .title_style(theme::title())
        .borders(Borders::ALL)
        .border_style(theme::border());

    let events = &app.state.events;
    if events.is_empty() {
        let para = Paragraph::new("\n  No events recorded yet.")
            .style(theme::label())
            .block(block);
        frame.render_widget(para, area);
        return;
    }

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(Span::styled(
        format!("  {} events (latest first)", events.len()),
        theme::label(),
    )));
    lines.push(Line::raw(""));

    // Show events in reverse (newest first).
    for event in events.iter().rev() {
        let ts = if event.timestamp.len() > 19 {
            &event.timestamp[11..19] // HH:MM:SS
        } else {
            &event.timestamp
        };

        let type_style = event_type_style(&event.event_type);

        lines.push(Line::from(vec![
            Span::styled(format!("  [{ts}] "), theme::label()),
            Span::styled(
                format!("{:>8}… ", &event.event_id[..8.min(event.event_id.len())]),
                theme::label(),
            ),
            Span::styled(&event.event_type, type_style),
        ]));
    }

    let para = Paragraph::new(lines).scroll((app.scroll_offset, 0));
    frame.render_widget(para, inner);
}

fn event_type_style(event_type: &str) -> Style {
    if event_type.contains("Error") || event_type.contains("Violated") {
        theme::status_err()
    } else if event_type.contains("Session") {
        theme::highlight()
    } else if event_type.contains("Focus") || event_type.contains("Frame") {
        theme::status_ok()
    } else if event_type.contains("Candidate") || event_type.contains("Gate") {
        theme::status_warn()
    } else {
        theme::value()
    }
}
