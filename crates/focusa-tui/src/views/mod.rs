//! View rendering — all panels composable, read-only.

pub mod about;
mod autonomy;
mod cache;
mod constitution;
mod contribution;
pub mod deck_home;
mod events;
mod focus_stack;
mod focus_state;
mod gate;
pub mod help_overlay;
pub mod intro;
mod intuition;
mod lineage;
mod metrics;
pub mod mission_ladder;
pub mod modal;
pub mod proof_status;
mod proposals;
pub mod recall;
mod references;
mod rfm;
pub mod semantic_pair;
mod skills;
mod telemetry;
mod training;
mod uxp;
pub mod walkthroughs;
mod work_loop;

use crate::app::{App, Tab};
use crate::theme;
use ratatui::prelude::*;
use ratatui::widgets::*;
use throbber_widgets_tui::symbols::throbber;

/// Root render — layout + dispatch to active view.
pub fn render(app: &App, frame: &mut ratatui::Frame) {
    let area = frame.area();

    // Global layout: header (1) + body + footer (1).
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    render_header(app, frame, chunks[0]);
    render_body(app, frame, chunks[1]);
    render_footer_keys(app, frame, chunks[2]);
    if app.show_help {
        help_overlay::render(frame, area);
    }
    if app.show_intro {
        intro::render(app, frame, area);
    }
    if matches!(app.tab, crate::app::Tab::About) {
        about::render(frame, area);
    }
    if matches!(app.tab, crate::app::Tab::Walkthroughs) {
        walkthroughs::render(frame, area);
    }
    if matches!(app.tab, crate::app::Tab::DeckHome)
        && app.modal.is_none()
        && !app.show_intro
        && !app.show_help
    {
        crate::mission_control::render(app, frame, chunks[1]);
    }
    if let Some(modal) = app.modal {
        modal::render_modal(modal, app, frame, area);
    }
}

fn render_footer_keys(app: &App, frame: &mut ratatui::Frame, area: Rect) {
    let hint = if app.connected {
        let ts = app
            .last_refresh_at
            .map(|t| format!(" · updated {}", t.format("%H:%M:%S")))
            .unwrap_or_default();
        let update = app
            .state
            .update_notification
            .as_ref()
            .filter(|notice| !notice.stale_parts.is_empty())
            .map(|notice| format!(" · UPDATE {}", notice.stale_parts.len()))
            .unwrap_or_default();
        format!("n=deck  /=recall  l=learn  ?=help  a=about  :=cmd  q=quit{update}{ts}")
    } else {
        "waiting…  r=retry  q=quit".to_string()
    };
    frame.render_widget(Paragraph::new(Span::styled(hint, theme::label())), area);
}

fn render_header(app: &App, frame: &mut ratatui::Frame, area: Rect) {
    let logo_color = if app.connected {
        theme::title()
    } else {
        theme::status_err()
    };

    let status = if app.connected {
        Span::styled("●", Style::default().fg(Color::Green))
    } else {
        let spinner_len = throbber::CLOCK.symbols.len() as i8;
        let i = app.throbber_state.index().rem_euclid(spinner_len) as usize;
        Span::styled(
            format!("{}", throbber::CLOCK.symbols[i]),
            Style::default().fg(Color::Yellow),
        )
    };

    let status_label = if app.connected {
        "online"
    } else {
        "connecting"
    };
    let line = Line::from(vec![
        Span::styled("FOCUSA", logo_color.add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        status,
        Span::raw(" "),
        Span::styled(status_label, theme::label()),
        Span::raw("  "),
        Span::styled(
            "Local-first mission cohesion for AI coding agents.",
            theme::label(),
        ),
    ]);
    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(theme::border());
    frame.render_widget(Paragraph::new(line).block(block), area);
}

fn render_body(app: &App, frame: &mut ratatui::Frame, area: Rect) {
    if !app.connected {
        let msg = app.last_error.as_deref().unwrap_or("Not connected");
        let spinner_len = throbber::CLOCK.symbols.len() as i8;
        let i = app.throbber_state.index().rem_euclid(spinner_len) as usize;
        let spinner_char = throbber::CLOCK.symbols[i];
        let block = Block::default()
            .title(" Disconnected ")
            .title_style(theme::status_err())
            .borders(Borders::ALL)
            .border_style(theme::border());
        let para = Paragraph::new(format!(
            "\n   {} Waiting for Focusa daemon at {}...\n\n   {}\n\n   Press 'r' to retry, 'q' to quit.",
            spinner_char, app.api_url(), msg
        ))
        .style(theme::label())
        .block(block);
        frame.render_widget(para, area);
        return;
    }

    match app.tab {
        Tab::DeckHome => deck_home::render(app, frame, area),
        Tab::FocusState => focus_state::render(app, frame, area),
        Tab::FocusStack => focus_stack::render(app, frame, area),
        Tab::Gate => gate::render(app, frame, area),
        Tab::Events => events::render(app, frame, area),
        Tab::Metrics => metrics::render(app, frame, area),
        Tab::Lineage => lineage::render(app, frame, area),
        Tab::About => {}        // handled at root in render() with about::render
        Tab::Walkthroughs => {} // handled at root in render() with walkthroughs::render
        Tab::WorkLoop => work_loop::render(app, frame, area),
        Tab::Recall => recall::render(app, frame, area),
        Tab::Autonomy => autonomy::render(app, frame, area),
        Tab::Constitution => constitution::render(app, frame, area),
        Tab::Telemetry => telemetry::render(app, frame, area),
        Tab::Rfm => rfm::render(app, frame, area),
        Tab::Proposals => proposals::render(app, frame, area),
        Tab::Skills => skills::render(app, frame, area),
        Tab::Uxp => uxp::render(app, frame, area),
        Tab::Training => training::render(app, frame, area),
        Tab::References => references::render(app, frame, area),
        Tab::Cache => cache::render(app, frame, area),
        Tab::Contribution => contribution::render(app, frame, area),
        Tab::Intuition => intuition::render(app, frame, area),
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
            "  │  q:quit  Tab:switch  h/?:help  r:refresh  j/k:scroll ",
            theme::label(),
        ),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::border());
    let para = Paragraph::new(line).block(block);
    frame.render_widget(para, area);
}
