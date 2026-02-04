//! Focus Stack view — tree visualization of nested frames.

use crate::app::App;
use crate::theme;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render(app: &App, frame: &mut ratatui::Frame, area: Rect) {
    let block = Block::default()
        .title(" Focus Stack ")
        .title_style(theme::title())
        .borders(Borders::ALL)
        .border_style(theme::border());

    let frames = &app.state.focus_stack.frames;
    if frames.is_empty() {
        let para = Paragraph::new("\n  No frames. Start a session and push a focus frame.")
            .style(theme::label())
            .block(block);
        frame.render_widget(para, area);
        return;
    }

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let active_id = app.state.focus_stack.active_id.as_deref();

    let mut lines: Vec<Line> = Vec::new();

    // Header.
    lines.push(Line::from(vec![
        Span::styled(
            format!("  {} frames", frames.len()),
            theme::label(),
        ),
        Span::styled("  │  ", theme::label()),
        Span::styled(
            format!("path: [{}]", app.state.focus_stack.stack_path.join(" → ")),
            theme::label(),
        ),
    ]));
    lines.push(Line::raw(""));

    // Render each frame as a tree node.
    for f in frames {
        let indent = "  ".repeat(f.depth as usize);
        let connector = if f.depth > 0 { "└─ " } else { "" };
        let is_active = Some(f.frame_id.as_str()) == active_id;

        let status_style = match f.status.as_str() {
            "Active" => theme::status_ok(),
            "Paused" | "Suspended" => theme::status_warn(),
            "Completed" | "Archived" => theme::label(),
            _ => theme::value(),
        };

        let marker = if is_active { "▸ " } else { "  " };
        let id_short = &f.frame_id[..8.min(f.frame_id.len())];

        lines.push(Line::from(vec![
            Span::styled(marker, if is_active { theme::highlight() } else { theme::label() }),
            Span::styled(format!("{indent}{connector}"), theme::label()),
            Span::styled(format!("[{id_short}…] "), theme::highlight()),
            Span::styled(&f.intent, if is_active { theme::highlight() } else { theme::value() }),
        ]));

        lines.push(Line::from(vec![
            Span::raw("    "),
            Span::styled(format!("{indent}   "), theme::label()),
            Span::styled(&f.status, status_style),
            Span::styled(format!("  beads:{}", &f.beads_id[..8.min(f.beads_id.len())]), theme::label()),
        ]));
    }

    let para = Paragraph::new(lines)
        .scroll((app.scroll_offset, 0));
    frame.render_widget(para, inner);
}
