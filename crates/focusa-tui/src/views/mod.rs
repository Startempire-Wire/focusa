//! View rendering — all panels composable, read-only.

mod autonomy;
mod constitution;
mod events;
mod focus_stack;
mod focus_state;
mod gate;
mod lineage;
mod metrics;
mod proposals;
mod rfm;
mod skills;
mod telemetry;
mod training;
mod uxp;

use crate::app::{App, Tab};
use crate::theme;
use ratatui::prelude::*;
use ratatui::widgets::*;

/// Root render — layout + dispatch to active view.
pub fn render(app: &App, frame: &mut ratatui::Frame) {
    let area = frame.area();

    // Global layout: header (3) + body + status bar (3).
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area);

    render_header(app, frame, chunks[0]);
    render_body(app, frame, chunks[1]);
    render_status_bar(app, frame, chunks[2]);
}

fn render_header(app: &App, frame: &mut ratatui::Frame, area: Rect) {
    let tabs: Vec<Line> = Tab::ALL
        .iter()
        .map(|t| {
            let style = if *t == app.tab {
                theme::tab_active()
            } else {
                theme::tab_inactive()
            };
            Line::from(format!(" {} {} ", t.hotkey(), t.label())).style(style)
        })
        .collect();

    let tabs_widget = Tabs::new(tabs)
        .block(
            Block::default()
                .title(" Focusa — Cognitive Runtime ")
                .title_style(theme::title())
                .borders(Borders::ALL)
                .border_style(theme::border()),
        )
        .select(Tab::ALL.iter().position(|t| *t == app.tab).unwrap_or(0))
        .divider("│")
        .highlight_style(theme::tab_active());

    frame.render_widget(tabs_widget, area);
}

fn render_body(app: &App, frame: &mut ratatui::Frame, area: Rect) {
    if !app.connected {
        let msg = app.last_error.as_deref().unwrap_or("Not connected");
        let block = Block::default()
            .title(" Disconnected ")
            .title_style(theme::status_err())
            .borders(Borders::ALL)
            .border_style(theme::border());
        let para = Paragraph::new(format!(
            "\n  Waiting for Focusa daemon at {}...\n\n  {}\n\n  Press 'r' to retry, 'q' to quit.",
            "FOCUSA_API_URL", msg
        ))
        .style(theme::label())
        .block(block);
        frame.render_widget(para, area);
        return;
    }

    match app.tab {
        Tab::FocusState => focus_state::render(app, frame, area),
        Tab::FocusStack => focus_stack::render(app, frame, area),
        Tab::Gate => gate::render(app, frame, area),
        Tab::Events => events::render(app, frame, area),
        Tab::Metrics => metrics::render(app, frame, area),
        Tab::Lineage => lineage::render(app, frame, area),
        Tab::Autonomy => autonomy::render(app, frame, area),
        Tab::Constitution => constitution::render(app, frame, area),
        Tab::Telemetry => telemetry::render(app, frame, area),
        Tab::Rfm => rfm::render(app, frame, area),
        Tab::Proposals => proposals::render(app, frame, area),
        Tab::Skills => skills::render(app, frame, area),
        Tab::Uxp => uxp::render(app, frame, area),
        Tab::Training => training::render(app, frame, area),
    }
}

fn render_status_bar(app: &App, frame: &mut ratatui::Frame, area: Rect) {
    let conn = if app.connected {
        Span::styled("● connected", theme::status_ok())
    } else {
        Span::styled("○ disconnected", theme::status_err())
    };

    let session = app
        .state
        .session
        .as_ref()
        .map(|s| format!("session: {}…", &s.session_id[..8.min(s.session_id.len())]))
        .unwrap_or_else(|| "no session".into());

    let active = app
        .state
        .focus_stack
        .active_id
        .as_ref()
        .map(|id| format!("frame: {}…", &id[..8.min(id.len())]))
        .unwrap_or_else(|| "no active frame".into());

    let version = format!("v{}", app.state.version);

    let line = Line::from(vec![
        Span::raw(" "),
        conn,
        Span::styled(
            format!("  │  {session}  │  {active}  │  {version}"),
            theme::label(),
        ),
        Span::styled(
            "  │  q:quit  Tab:switch  r:refresh  j/k:scroll ",
            theme::label(),
        ),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::border());
    let para = Paragraph::new(line).block(block);
    frame.render_widget(para, area);
}
