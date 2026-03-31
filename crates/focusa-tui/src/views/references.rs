//! References view — ECS handle listing.
//!
//! Per 27-tui-spec §6: Table view with Ref ID, Type, Size, Linked.

use crate::app::App;
use crate::theme;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render(app: &App, frame: &mut ratatui::Frame, area: Rect) {
    let block = Block::default()
        .title(" References (ECS Handles) ")
        .title_style(theme::title())
        .borders(Borders::ALL)
        .border_style(theme::border());

    // Get handles from extra_data or empty vec
    let handles: Vec<HandleRow> = app
        .extra_data
        .get("references")
        .and_then(|v| v.as_ref())
        .and_then(|v| v.get("handles"))
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .map(|h| HandleRow {
                    id: h.get("id").and_then(|v| v.as_str()).unwrap_or("?").to_string(),
                    kind: h.get("kind").and_then(|v| v.as_str()).unwrap_or("?").to_string(),
                    size: h.get("size").and_then(|v| v.as_u64()).unwrap_or(0),
                    label: h.get("label").and_then(|v| v.as_str()).unwrap_or("?").to_string(),
                    pinned: h.get("pinned").and_then(|v| v.as_bool()).unwrap_or(false),
                })
                .collect()
        })
        .unwrap_or_default();

    if handles.is_empty() {
        let msg = if app.extra_data.get("references").is_none() {
            "Loading references..."
        } else {
            "No handles in reference index.\n\nArtifacts are stored externally (ECS) and referenced via handles."
        };
        let para = Paragraph::new(msg)
            .style(theme::label())
            .block(block);
        frame.render_widget(para, area);
        return;
    }

    // Build table rows
    let rows: Vec<Row> = handles
        .iter()
        .skip(app.scroll_offset as usize)
        .take(20)
        .map(|h| {
            let pin_indicator = if h.pinned { "📌 " } else { "  " };
            Row::new(vec![
                Cell::from(format!("{}{}", pin_indicator, &h.id[..8.min(h.id.len())])),
                Cell::from(h.kind.clone()),
                Cell::from(format_bytes(h.size)),
                Cell::from(h.label.clone()),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Length(10),
            Constraint::Min(20),
        ],
    )
    .header(
        Row::new(vec!["Handle ID", "Type", "Size", "Label"])
            .style(theme::value().add_modifier(Modifier::BOLD)),
    )
    .block(block)
    .row_highlight_style(theme::highlight());

    frame.render_widget(table, area);
}

struct HandleRow {
    id: String,
    kind: String,
    size: u64,
    label: String,
    pinned: bool,
}

fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}
