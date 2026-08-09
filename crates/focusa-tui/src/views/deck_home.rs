//! Mission Deck home surface (Spec 117 §8).

use crate::app::App;
use crate::beginner_mode;
use crate::next_safe_action;
use crate::theme;
use crate::views::mission_ladder;
use crate::views::proof_status;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub const BEAUTIFICATION_CHECKLIST: &[&str] = &[
    "clear_mission_headline",
    "visible_scope_badge",
    "visible_proof_meter",
    "one_primary_next_action",
    "plain_language_why",
    "discoverable_hotkeys",
    "explicit_unavailable_states",
];

pub fn render(app: &App, frame: &mut ratatui::Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),
            Constraint::Length(12),
            Constraint::Min(0),
        ])
        .split(area);

    render_mission_card(app, frame, chunks[0]);
    render_next_safe_action(app, frame, chunks[1]);
    mission_ladder::render(app, frame, chunks[2]);
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
    let proof = proof_status::proof_meter(app);
    let scope = proof_status::scope_badge(app);
    // Spec 152E §21 shared presenter posture: the TUI renders the same
    // activation/entitlement states and allowed actions as the menubar,
    // the daemon REST license routes, and lifecycle receipts for the same
    // canonical registration; it never re-decides a transition.
    let activation_line = app
        .activation
        .as_ref()
        .map(|view| format!("Activation    {}", view.status_line()))
        .unwrap_or_else(|| "Activation    unavailable (no registration snapshot)".into());
    let entitlement_line = app
        .license
        .as_ref()
        .map(|posture| format!("Entitlement   {}", posture.status_line()))
        .unwrap_or_else(|| "Entitlement   unavailable (no signed authority snapshot)".into());
    // Spec 152F §11.5/§13 accessibility fixture: the TUI shows the same
    // next-action guide and always-reachable set as the menubar presenter;
    // denied value actions stay explained and never trap the customer.
    let entitlement_guide_line = app
        .license
        .as_ref()
        .map(|posture| format!("Entitlement   {}", posture.action_guide()))
        .unwrap_or_default();
    // Spec 172 §11/§15 presenter projection: License Type display,
    // Operator/Bundle upgrade accuracy, node semantics, and the frozen
    // locked-state accessibility fixture. Fail closed: no signed posture
    // snapshot renders as unavailable rather than an invented License Type.
    let spec172_line = app
        .spec172
        .as_ref()
        .map(|posture| format!("Spec 172      {}", posture.status_line()))
        .unwrap_or_else(|| "Spec 172      unavailable (no canonical posture snapshot)".into());
    let spec172_fixture_line = app
        .spec172
        .as_ref()
        .map(|posture| format!("Spec 172      {}", posture.locked_state_fixture()))
        .unwrap_or_default();
    let text = vec![
        Line::from(vec![
            Span::styled("Mission Deck", theme::title()),
            Span::raw(" — keep the mission, prove the handoff."),
        ]),
        Line::from(format!("Session       {session}")),
        Line::from(format!("Active frame  {active_frame}")),
        Line::from(activation_line),
        Line::from(entitlement_line),
        Line::from(entitlement_guide_line),
        Line::from(spec172_line),
        Line::from(spec172_fixture_line),
        Line::from(format!("Scope badge   {}  {}", scope.visual, scope.label)),
        Line::from(format!("Proof meter   {}  {}", proof.visual, proof.label)),
        Line::from(format!(
            "frames: {}  events: {}",
            app.state.focus_stack.frames.len(),
            app.state.events.len()
        )),
    ];
    let block = Block::default()
        .title(" Deck Home · Mission + Proof ")
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
    let mode_state = beginner_mode::assess(app);
    let next = next_safe_action::recommend(app);
    let intent = focus_state
        .map(|state| state.intent.as_str())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("No Focus State intent yet");
    let current = focus_state
        .and_then(|state| state.current_state.as_deref())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("Bind project, resume workpoint, capture proof");

    let text = vec![
        Line::from(Span::styled(
            "Next safe action — one move, with why",
            theme::label(),
        )),
        Line::from(format!("State          {}", mode_state.id())),
        Line::from(mode_state.explanation()),
        Line::from(format!("Primary action {}", next.label)),
        Line::from(format!("Command        {}", next.command)),
        Line::from(format!(
            "Authority     {} · context {}",
            next.authority_posture, next.walkthrough_context
        )),
        Line::from(format!("Why           {}", next.why)),
        Line::from(format!("Intent        {intent}")),
        Line::from(format!("Focus         {current}")),
        Line::from("Keys          d Deck · n next · / recall · h help · r refresh · q quit"),
    ];
    let block = Block::default()
        .title(" Mission Control · Do This Next ")
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
