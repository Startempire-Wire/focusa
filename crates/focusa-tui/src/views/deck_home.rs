//! Mission Deck home surface (Spec 117 §8).

use crate::app::App;
use crate::theme;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render(app: &App, frame: &mut ratatui::Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),
            Constraint::Length(7),
            Constraint::Min(0),
        ])
        .split(area);

    render_mission_card(app, frame, chunks[0]);
    render_next_safe_action(app, frame, chunks[1]);
    render_orientation(app, frame, chunks[2]);
}

fn render_mission_card(app: &App, frame: &mut ratatui::Frame, area: Rect) {
    let session = app
        .state
        .session
        .as_ref()
        .map(|s| s.session_id.as_str())
        .unwrap_or("no session");
    let active_frame = app
        .state
        .focus_stack
        .active_id
        .as_deref()
        .unwrap_or("no active frame");
    let text = vec![
        Line::from(vec![
            Span::styled("Mission Deck", theme::title()),
            Span::raw(" — resume the right mission, with proof."),
        ]),
        Line::from(format!("session: {session}")),
        Line::from(format!("active frame: {active_frame}")),
        Line::from(format!(
            "frames: {}  events: {}",
            app.state.focus_stack.frames.len(),
            app.state.events.len()
        )),
    ];
    let block = Block::default()
        .title(" Deck Home ")
        .title_style(theme::title())
        .borders(Borders::ALL)
        .border_style(theme::border());
    frame.render_widget(
        Paragraph::new(text).block(block).wrap(Wrap { trim: true }),
        area,
    );
}

fn render_next_safe_action(app: &App, frame: &mut ratatui::Frame, area: Rect) {
    let focus_state = app.state.focus_state.as_ref();
    let intent = focus_state
        .map(|state| state.intent.as_str())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("No Focus State intent yet");
    let current = focus_state
        .and_then(|state| state.current_state.as_deref())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("Bind project, resume workpoint, capture proof");

    let text = vec![
        Line::from(Span::styled("Next safe action", theme::label())),
        Line::from(format!("intent: {intent}")),
        Line::from(format!("focus: {current}")),
        Line::from("keys: d Deck Home · 1 State · 2 Stack · Tab next · r refresh · q quit"),
    ];
    let block = Block::default()
        .title(" Mission Control ")
        .title_style(theme::title())
        .borders(Borders::ALL)
        .border_style(theme::border());
    frame.render_widget(
        Paragraph::new(text).block(block).wrap(Wrap { trim: true }),
        area,
    );
}

fn render_orientation(app: &App, frame: &mut ratatui::Frame, area: Rect) {
    let proof_status = if app.state.events.is_empty() {
        "proof not visible yet — capture or link evidence before handoff"
    } else {
        "proof activity visible in event stream"
    };
    let text = vec![
        Line::from("1. Project — verify the folder before trusting carryover state."),
        Line::from("2. Workpoint — checkpoint mission/current action/next slice."),
        Line::from(format!("3. Proof — {proof_status}.")),
        Line::from("4. Resume — another agent should recover mission, evidence, and next action."),
    ];
    let block = Block::default()
        .title(" Beginner Orientation ")
        .title_style(theme::title())
        .borders(Borders::ALL)
        .border_style(theme::border());
    frame.render_widget(
        Paragraph::new(text).block(block).wrap(Wrap { trim: true }),
        area,
    );
}
