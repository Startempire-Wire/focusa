//! Focus Gate view — candidate list with pressure visualization.

use crate::app::App;
use crate::theme;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render(app: &App, frame: &mut ratatui::Frame, area: Rect) {
    let block = Block::default()
        .title(" Focus Gate ")
        .title_style(theme::title())
        .borders(Borders::ALL)
        .border_style(theme::border());

    let candidates = &app.state.candidates;
    if candidates.is_empty() {
        let para = Paragraph::new("\n  No gate candidates.\n  Candidates emerge from intuition signals.")
            .style(theme::label())
            .block(block);
        frame.render_widget(para, area);
        return;
    }

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Split: header + table.
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(0),
        ])
        .split(inner);

    // Header.
    let surfaced = candidates.iter().filter(|c| c.status == "Surfaced").count();
    let pinned = candidates.iter().filter(|c| c.pinned).count();
    let header = Line::from(vec![
        Span::styled(format!("  {} candidates", candidates.len()), theme::label()),
        Span::styled(format!("  │  {} surfaced", surfaced), theme::highlight()),
        Span::styled(format!("  │  {} pinned", pinned), theme::status_warn()),
    ]);
    frame.render_widget(Paragraph::new(header), chunks[0]);

    // Candidate rows.
    let mut lines: Vec<Line> = Vec::new();
    for c in candidates {
        let pressure_bar = render_pressure_bar(c.pressure);
        let pin_marker = if c.pinned { "📌" } else { "  " };
        let id_short = &c.id[..8.min(c.id.len())];

        let status_style = match c.status.as_str() {
            "Surfaced" => theme::highlight(),
            "Suppressed" => theme::label(),
            "Resolved" => theme::status_ok(),
            _ => theme::value(),
        };

        lines.push(Line::from(vec![
            Span::styled(format!("  {pin_marker} "), theme::value()),
            Span::styled(format!("[{id_short}…] "), theme::label()),
            Span::styled(format!("{:<12} ", c.kind), theme::value()),
            Span::styled(&c.label, theme::value()),
        ]));
        lines.push(Line::from(vec![
            Span::raw("       "),
            Span::styled(pressure_bar, pressure_color(c.pressure)),
            Span::styled(format!(" {:.2}", c.pressure), theme::label()),
            Span::styled(format!("  {}", c.status), status_style),
        ]));
    }

    let para = Paragraph::new(lines).scroll((app.scroll_offset, 0));
    frame.render_widget(para, chunks[1]);
}

/// ASCII pressure bar: ▓░ scale from 0.0 to 1.0+.
fn render_pressure_bar(pressure: f64) -> String {
    let filled = (pressure * 20.0).min(20.0) as usize;
    let empty = 20 - filled;
    format!("│{}{}│", "▓".repeat(filled), "░".repeat(empty))
}

fn pressure_color(pressure: f64) -> Style {
    if pressure >= 0.8 {
        theme::status_err()
    } else if pressure >= 0.5 {
        theme::status_warn()
    } else {
        theme::label()
    }
}
