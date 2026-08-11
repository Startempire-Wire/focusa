//! Mission Control canvas (Spec 117 §8 launch polish).
//!
//! Single-canvas dashboard that replaces the flat tab strip. Around a
//! central Mission Card we render proof meter, scope badge, mission ladder,
//! and walkthroughs as adjacent widgets. Detail and recall/learn/about
//! surfaces open as modal overlays (see views::modal), not as tab bodies.
//!
//! Responsive: collapses to a single stacked column when viewport < 80 cols
//! so it stays usable on mobile/small terminals.

use crate::app::App;
use crate::beginner_mode;
use crate::next_safe_action;
use crate::theme;
use crate::views::proof_status;
use ratatui::prelude::*;
use ratatui::widgets::*;
use serde_json::Value;

pub const MOBILE_BREAKPOINT_COLS: u16 = 80;
pub const COMPACT_KEYS_HINT: &str = "n next · / recall · l learn · ? help · a about · q quit";
pub const FULL_KEYS_HINT: &str =
    "n next · / recall · l learn · ? help · a about · : cmd · Esc close · q quit";

pub fn is_mobile(area: Rect) -> bool {
    area.width < MOBILE_BREAKPOINT_COLS
}

pub fn render(app: &App, frame: &mut ratatui::Frame, area: Rect) {
    // Layout: header + body only (global views footer handles key hints).
    let header = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(3)])
        .split(area);

    render_header(app, frame, header[0]);

    if is_mobile(area) {
        render_stack(app, frame, header[1]);
    } else {
        render_grid(app, frame, header[1]);
    }

    // Active modal overlays: replace canvas, not stack.
    if let Some(modal) = app.modal {
        render_modal_overlay(modal, app, frame, area);
    }
}

fn render_header(app: &App, frame: &mut ratatui::Frame, area: Rect) -> Rect {
    let state = if app.connected {
        "connected"
    } else {
        "offline"
    };
    let proj = project_label(app);
    let line = Line::from(vec![
        Span::styled(
            "MISSION CONTROL",
            theme::title().add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(format!("{proj}"), theme::label()),
        Span::raw("  "),
        Span::styled(
            state,
            if app.connected {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red)
            },
        ),
    ]);
    frame.render_widget(
        Paragraph::new(line).block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(theme::border()),
        ),
        area,
    );
    area
}

fn project_label(app: &App) -> String {
    if let Some(workpoint) = app
        .extra_data
        .get("workpoint_resume")
        .and_then(|v| v.as_ref())
    {
        if let Some(id) = workpoint
            .get("id")
            .or_else(|| workpoint.get("workpoint_id"))
            .and_then(|v| v.as_str())
        {
            return id.to_string();
        }
    }
    "focusa".to_string()
}

fn temporal_status_fields(temporal: Option<&Value>) -> [String; 4] {
    let field = |name: &str, fallback: &str| {
        temporal
            .and_then(|value| value.get(name))
            .and_then(Value::as_str)
            .unwrap_or(fallback)
            .to_string()
    };
    let urgency = temporal
        .and_then(|value| value.get("urgency"))
        .and_then(|value| {
            value
                .get("subject_ref")
                .or_else(|| value.get("reason_code"))
        })
        .and_then(Value::as_str)
        .unwrap_or("none")
        .to_string();
    let conformance = temporal
        .and_then(|value| value.get("conformance"))
        .and_then(|value| value.get("full_conformance_status"))
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    [
        field("deadline_status", "unavailable"),
        field("deadline_conflict_state", "unknown"),
        urgency,
        conformance,
    ]
}

fn render_grid(app: &App, frame: &mut ratatui::Frame, area: Rect) -> Vec<Rect> {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
        .split(area);

    let left = render_mission_column(app, frame, cols[0]);
    let right = render_status_column(app, frame, cols[1]);
    left.into_iter().chain(right).collect()
}

fn render_stack(app: &App, frame: &mut ratatui::Frame, area: Rect) {
    // Mobile: keep it simple. Mission Card + beginner hint.
    // Key hints are in the global footer.
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(8), Constraint::Length(3)])
        .split(area);

    render_mission_column(app, frame, rows[0]);
    render_beginner_hint_mobile(app, frame, rows[1]);
}

fn render_beginner_hint_mobile(app: &App, frame: &mut ratatui::Frame, area: Rect) {
    let mode = beginner_mode::assess(app);
    let lines = vec![
        Line::from(Span::styled(format!("{}", mode.id()), theme::title())),
        Line::from(mode.explanation()),
        Line::from(format!("Run: {}", mode.primary_action())),
    ];
    let block = Block::default()
        .title(" Next ")
        .borders(Borders::ALL)
        .border_style(theme::border());
    frame.render_widget(
        Paragraph::new(lines).block(block).wrap(Wrap { trim: true }),
        area,
    );
}

fn render_mission_column(app: &App, frame: &mut ratatui::Frame, area: Rect) -> Vec<Rect> {
    let next = next_safe_action::recommend(app);
    let mode = beginner_mode::assess(app);
    let lines = vec![
        Line::from(Span::styled("Mission", theme::title())),
        Line::from(format!(
            "Intent       {}",
            short(
                app.state
                    .focus_state
                    .as_ref()
                    .map(|f| f.intent.as_str())
                    .unwrap_or(""),
                64
            )
        )),
        Line::from(format!("Current      {}", short(&current_state(app), 56))),
        Line::from(format!("Next action  {}", short(next.label, 56))),
        Line::from(format!(
            "Authority    {} · beginner state {}",
            next.authority_posture,
            mode.id()
        )),
        Line::from(format!("Why          {}", short(next.why, 64))),
        Line::from(""),
        Line::from(Span::styled(COMPACT_KEYS_HINT, theme::label())),
    ];
    let block = Block::default()
        .title(" Mission Card ")
        .borders(Borders::ALL)
        .border_style(theme::border());
    frame.render_widget(
        Paragraph::new(lines).block(block).wrap(Wrap { trim: true }),
        area,
    );
    vec![area]
}

fn render_status_column(app: &App, frame: &mut ratatui::Frame, area: Rect) -> Vec<Rect> {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Ratio(1, 5),
            Constraint::Ratio(2, 5),
            Constraint::Ratio(2, 5),
        ])
        .split(area);

    render_proof_scope_blocks(app, frame, rows[0]);
    let ladder = render_ladder_block(app, frame, rows[1]);
    let walkthroughs = render_walkthroughs_block(app, frame, rows[2]);
    vec![rows[0], ladder, walkthroughs]
}

fn render_proof_scope_blocks(app: &App, frame: &mut ratatui::Frame, area: Rect) -> Rect {
    let proof = proof_status::proof_meter(app);
    let scope = proof_status::scope_badge(app);
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
        .split(area);
    let p_block = Block::default()
        .title(" Proof Meter ")
        .borders(Borders::ALL)
        .border_style(theme::border());
    let proof_lines = vec![
        Line::from(format!("{}  {}", proof.visual, proof.label)),
        Line::from(format!("affordance  {}", proof.affordance_reality)),
    ];
    frame.render_widget(
        Paragraph::new(proof_lines)
            .block(p_block)
            .wrap(Wrap { trim: true }),
        cols[0],
    );
    let s_block = Block::default()
        .title(" Scope Badge ")
        .borders(Borders::ALL)
        .border_style(theme::border());
    let temporal = app
        .extra_data
        .get("workpoint_resume")
        .and_then(|value| value.as_ref())
        .and_then(|value| value.get("temporal_context"))
        .or_else(|| {
            app.extra_data
                .get("temporal")
                .and_then(|value| value.as_ref())
        });
    let [deadline, conflict, urgency, conformance] = temporal_status_fields(temporal);
    let prediction_authority = app
        .extra_data
        .get("prediction_authority")
        .and_then(|value| value.as_ref());
    let epistemic_conformance = prediction_authority
        .and_then(|value| value.get("profile_conformance"))
        .and_then(|value| value.get("full_conformance_status"))
        .and_then(|value| value.as_str())
        .unwrap_or("unknown");
    let epistemic_events = prediction_authority
        .and_then(|value| value.get("event_count"))
        .and_then(|value| value.as_u64())
        .unwrap_or(0);
    let instruction_integrity = app
        .extra_data
        .get("instruction_integrity")
        .and_then(|value| value.as_ref());
    let instruction_status = instruction_integrity
        .and_then(|value| value.get("status"))
        .and_then(|value| value.as_str())
        .unwrap_or("unavailable");
    let instruction_outage = instruction_integrity
        .and_then(|value| value.get("dynamic_authority_outage_posture"))
        .and_then(|value| value.as_str())
        .unwrap_or("unknown");
    let scope_lines = vec![
        Line::from(format!("{}  {}", scope.visual, scope.label)),
        Line::from(format!("precedence  {}", scope.precedence_frame)),
        Line::from(format!(
            "time  deadline={deadline} conflict={conflict} urgency={urgency} conformance={conformance}"
        )),
        Line::from(format!(
            "epistemic  events={epistemic_events} conformance={epistemic_conformance} operations={} daemon_authority=true",
            include_str!("../../../docs/contracts/spec138-generated-operation-contracts.v1.json")
                .matches("\"operation_id\":")
                .count()
        )),
        Line::from(format!(
            "instruction  guard={instruction_status} outage={instruction_outage} canvas_authority=false"
        )),
    ];
    frame.render_widget(
        Paragraph::new(scope_lines)
            .block(s_block)
            .wrap(Wrap { trim: true }),
        cols[1],
    );
    area
}

fn render_ladder_block(app: &App, frame: &mut ratatui::Frame, area: Rect) -> Rect {
    let trajectory = app
        .extra_data
        .get("trajectory_view")
        .and_then(|v| v.as_ref());
    let block = Block::default()
        .title(" Mission Ladder ")
        .borders(Borders::ALL)
        .border_style(theme::border());
    // Spec 125 §6.2/§13/§14: HLT status warnings in mission ladder.
    let hlt_status = trajectory
        .as_ref()
        .and_then(|t| t.get("hlt_status").or(t.get("status")))
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let hlt_required = trajectory
        .as_ref()
        .and_then(|t| t.get("hlt_required"))
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let generic_bootstrap = trajectory
        .as_ref()
        .and_then(|t| t.get("generic_bootstrap"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let fallback_level = trajectory
        .as_ref()
        .and_then(|t| t.get("fallback_level"))
        .and_then(Value::as_str)
        .unwrap_or("none");
    let action_authority = trajectory
        .as_ref()
        .and_then(|t| t.get("action_authority_from_trajectory"))
        .and_then(Value::as_bool)
        .unwrap_or(false);

    let mut text = vec![Line::from(format!(
        "HLT  {}",
        ladder_value(trajectory, &["long_term_goal", "hlt"])
    ))];

    // Spec 125 §6.2: loud warning when HLT is missing/generic/fallback.
    if hlt_required
        && (hlt_status == "missing_required" || generic_bootstrap || fallback_level != "none")
    {
        let warning = if generic_bootstrap {
            "⚠ HLT is generic bootstrap — define project-specific HLT"
        } else if fallback_level != "none" {
            &format!(
                "⚠ HLT fallback level: {} — verify trajectory",
                fallback_level
            )
        } else {
            "⚠ HLT missing — use focusa_trajectory_define_goal"
        };
        text.push(Line::from(Span::styled(
            format!(" {}", warning),
            theme::status_warn(),
        )));
    }

    // Spec 125 §6.2: action authority indicator.
    if !action_authority {
        text.push(Line::from(Span::styled(
            " ⚠ No action authority from trajectory — Workpoint is immediate authority",
            theme::status_warn(),
        )));
    }

    text.extend(vec![
        Line::from(format!(
            " └─ MLG  {}",
            ladder_value(trajectory, &["mid_level_goal", "mlg"])
        )),
        Line::from(format!(
            "     └─ STG  {}",
            ladder_value(trajectory, &["short_term_goal", "stg"])
        )),
        Line::from(format!(
            "         └─ WP  {}",
            ladder_value(trajectory, &["workpoint_id", "wp"])
        )),
        Line::from(format!(
            "             └─ Evidence  {}",
            ladder_value(trajectory, &["evidence_count", "evidence"])
        )),
    ]);
    frame.render_widget(
        Paragraph::new(text).block(block).wrap(Wrap { trim: true }),
        area,
    );
    area
}

fn ladder_value(source: Option<&Value>, keys: &[&str]) -> String {
    let Some(value) = source else {
        return "unavailable".to_string();
    };
    for k in keys {
        if let Some(found) = value.get(*k).and_then(Value::as_str) {
            if !found.trim().is_empty() {
                return short(found, 56);
            }
        }
    }
    "unavailable".to_string()
}

fn current_state(app: &App) -> String {
    if let Some(workpoint) = app
        .extra_data
        .get("workpoint_resume")
        .and_then(|v| v.as_ref())
    {
        if let Some(s) = workpoint.get("mission").and_then(Value::as_str) {
            if !s.trim().is_empty() {
                return s.to_string();
            }
        }
    }
    "Bind project, resume workpoint, capture proof".to_string()
}

fn render_walkthroughs_block(app: &App, frame: &mut ratatui::Frame, area: Rect) -> Rect {
    let block = Block::default()
        .title(" Learn · walkthroughs available ")
        .borders(Borders::ALL)
        .border_style(theme::border());
    let cats = crate::views::walkthroughs::WALKTHROUGH_TITLES;
    let lines: Vec<Line> = std::iter::once(Line::from(Span::styled(
        "Press 'l' to open Learn modal.",
        theme::label(),
    )))
    .chain(
        cats.iter()
            .enumerate()
            .map(|(i, t)| Line::from(format!("  {}. {}", i + 1, t))),
    )
    .collect();
    frame.render_widget(
        Paragraph::new(lines).block(block).wrap(Wrap { trim: true }),
        area,
    );
    area
}

fn render_footer(app: &App, frame: &mut ratatui::Frame, area: Rect) -> Rect {
    let line = if is_mobile(area_for_footer(app, area)) {
        Line::from(Span::styled(
            format!(" ⌘ {}", COMPACT_KEYS_HINT.replace(" · ", "  ")),
            theme::label(),
        ))
    } else {
        Line::from(Span::styled(format!(" ⌘ {FULL_KEYS_HINT}"), theme::label()))
    };
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(theme::border());
    frame.render_widget(Paragraph::new(line).block(block), area);
    area
}

fn render_footer_compact(app: &App, frame: &mut ratatui::Frame, area: Rect) -> Rect {
    let line = Line::from(Span::styled(
        format!(" ⌘ {}", COMPACT_KEYS_HINT.replace(" · ", "  ")),
        theme::label(),
    ));
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(theme::border());
    frame.render_widget(Paragraph::new(line).block(block), area);
    area
}

fn area_for_footer(_app: &App, area: Rect) -> Rect {
    area
}

fn short(text: &str, max_chars: usize) -> String {
    let chars: String = text.chars().collect();
    if chars.chars().count() <= max_chars {
        return chars;
    }
    let mut cut = String::new();
    for (i, c) in chars.chars().enumerate() {
        if i + 1 >= max_chars {
            cut.push('…');
            break;
        }
        cut.push(c);
    }
    cut
}

fn render_modal_overlay(
    modal: crate::app::ModalKind,
    app: &App,
    frame: &mut ratatui::Frame,
    area: Rect,
) {
    crate::views::modal::render_modal(modal, app, frame, area);
}

// Public constant so other modules (e.g. status) can reference the FOCUSA mark
pub const FOCUSA_BRAND: &str = "FOCUSA";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mobile_breakpoint_matches_spec() {
        assert_eq!(MOBILE_BREAKPOINT_COLS, 80);
    }

    #[test]
    fn mobile_detection_uses_width_threshold() {
        assert!(is_mobile(Rect::new(0, 0, 79, 24)));
        assert!(!is_mobile(Rect::new(0, 0, 80, 24)));
        assert!(!is_mobile(Rect::new(0, 0, 120, 30)));
    }

    #[test]
    fn short_truncates_long_strings_with_ellipsis() {
        assert_eq!(short("abcdef", 4).chars().count(), 4);
        assert!(short("abcdefghi", 4).ends_with('…'));
    }

    #[test]
    fn bounded_temporal_context_renders_exact_status_without_inferred_urgency() {
        let temporal = serde_json::json!({
            "deadline_status":"none",
            "deadline_conflict_state":"feasible",
            "urgency":null,
            "conformance":{"full_conformance_status":"blocked_live_proof_required"}
        });
        assert_eq!(
            temporal_status_fields(Some(&temporal)),
            ["none", "feasible", "none", "blocked_live_proof_required"]
        );
        assert_eq!(
            temporal_status_fields(None),
            ["unavailable", "unknown", "none", "unknown"]
        );
    }

    #[test]
    fn compact_keys_hint_is_mobile_safe() {
        for line in [COMPACT_KEYS_HINT, FULL_KEYS_HINT] {
            assert!(line.chars().count() <= 80, "line too long: {line}");
        }
    }
}
