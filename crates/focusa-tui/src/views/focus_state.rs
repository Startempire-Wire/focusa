//! Focus State view — displays current canonical cognition.

use crate::app::App;
use crate::theme;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render(app: &App, frame: &mut ratatui::Frame, area: Rect) {
    let block = Block::default()
        .title(" Focus State ")
        .title_style(theme::title())
        .borders(Borders::ALL)
        .border_style(theme::border());

    let state = match &app.state.focus_state {
        Some(s) => s,
        None => {
            let para = Paragraph::new("\n  No active Focus State.\n  Push a frame to begin.")
                .style(theme::label())
                .block(block);
            frame.render_widget(para, area);
            return;
        }
    };

    // Split into panels: Intent (4) + Constraints + Decisions + Next Steps + Current State.
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Intent
            Constraint::Length(2 + state.constraints.len().min(5) as u16), // Constraints
            Constraint::Length(2 + state.decisions.len().min(5) as u16),  // Decisions
            Constraint::Length(2 + state.next_steps.len().min(5) as u16), // Next steps
            Constraint::Min(3),   // Current state
        ])
        .split(inner);

    // Intent.
    let intent = Paragraph::new(Line::from(vec![
        Span::styled("Intent: ", theme::label()),
        Span::styled(&state.intent, theme::highlight()),
    ]));
    frame.render_widget(intent, chunks[0]);

    // Constraints.
    render_list("Constraints", &state.constraints, frame, chunks[1]);

    // Decisions.
    render_list("Decisions", &state.decisions, frame, chunks[2]);

    // Next steps.
    render_list("Next Steps", &state.next_steps, frame, chunks[3]);

    // Current state.
    let current = state.current_state.as_deref().unwrap_or("—");
    let cs = Paragraph::new(Line::from(vec![
        Span::styled("State: ", theme::label()),
        Span::styled(current, theme::value()),
    ]));
    frame.render_widget(cs, chunks[4]);
}

fn render_list(title: &str, items: &[String], frame: &mut ratatui::Frame, area: Rect) {
    let header = Line::from(Span::styled(
        format!("{title} ({}):", items.len()),
        theme::label(),
    ));

    let mut lines = vec![header];
    for item in items.iter().take(5) {
        lines.push(Line::from(vec![
            Span::styled("  • ", theme::label()),
            Span::styled(item.as_str(), theme::value()),
        ]));
    }
    if items.len() > 5 {
        lines.push(Line::from(Span::styled(
            format!("  … and {} more", items.len() - 5),
            theme::label(),
        )));
    }

    let para = Paragraph::new(lines);
    frame.render_widget(para, area);
}
