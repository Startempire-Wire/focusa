//! Contribution view — training data contribution status.
//!
//! Per 27-tui-spec §13: contribution status, queue items, review UI.

use crate::app::App;
use crate::theme;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render(app: &App, frame: &mut ratatui::Frame, area: Rect) {
    let block = Block::default()
        .title(" Training Data Contribution ")
        .title_style(theme::title())
        .borders(Borders::ALL)
        .border_style(theme::border());

    // Get contribution data from training endpoint
    let contrib_data = app.extra_data.get("training").and_then(|v| v.as_ref());

    if contrib_data.is_none() {
        let para = Paragraph::new("Loading contribution status...")
            .style(theme::label())
            .block(block);
        frame.render_widget(para, area);
        return;
    }

    let data = contrib_data.unwrap();
    
    let enabled = data.get("contribution_enabled").and_then(|v| v.as_bool()).unwrap_or(false);
    let queue_size = data.get("queue_size").and_then(|v| v.as_u64()).unwrap_or(0);
    let total = data.get("total_contributed").and_then(|v| v.as_u64()).unwrap_or(0);

    let status_indicator = if enabled {
        Span::styled("● ENABLED ", theme::status_ok())
    } else {
        Span::styled("○ DISABLED ", theme::status_warn())
    };

    let mut text = vec![
        Line::from(vec![
            Span::raw("Status: "),
            status_indicator,
            Span::styled(
                format!("│ Queue: {} items │ Total contributed: {}", queue_size, total),
                theme::label(),
            ),
        ]),
        Line::raw(""),
        Line::styled(
            "Contribution sends anonymized training examples to improve models.",
            theme::label(),
        ),
        Line::raw(""),
    ];

    if enabled {
        text.push(Line::styled(
            "Contribution is active. Items are queued for review before sending.",
            theme::label(),
        ));
    } else {
        text.push(Line::styled(
            "Contribution is paused. Enable with: focusa contribute enable",
            theme::status_warn(),
        ));
    }

    let para = Paragraph::new(text)
        .block(block)
        .wrap(Wrap { trim: true });

    frame.render_widget(para, area);
}
