//! Metrics view — version, frame counts, candidate stats.

use crate::app::App;
use crate::theme;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render(app: &App, frame: &mut ratatui::Frame, area: Rect) {
    let block = Block::default()
        .title(" Metrics ")
        .title_style(theme::title())
        .borders(Borders::ALL)
        .border_style(theme::border());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let s = &app.state;

    let total_frames = s.focus_stack.frames.len();
    let active_frames = s
        .focus_stack
        .frames
        .iter()
        .filter(|f| f.status == "Active")
        .count();
    let paused_frames = s
        .focus_stack
        .frames
        .iter()
        .filter(|f| f.status == "Paused" || f.status == "Suspended")
        .count();
    let completed_frames = s
        .focus_stack
        .frames
        .iter()
        .filter(|f| f.status == "Completed" || f.status == "Archived")
        .count();
    let total_candidates = s.candidates.len();
    let surfaced = s
        .candidates
        .iter()
        .filter(|c| c.status == "Surfaced")
        .count();
    let suppressed = s
        .candidates
        .iter()
        .filter(|c| c.status == "Suppressed")
        .count();
    let pinned = s.candidates.iter().filter(|c| c.pinned).count();
    let max_pressure = s
        .candidates
        .iter()
        .map(|c| c.pressure)
        .fold(0.0f64, f64::max);
    let has_session = s.session.is_some();
    let event_count = s.events.len();
    let stack_depth = s.focus_stack.stack_path.len();

    // Pre-bind all formatted strings to extend lifetimes.
    let version_str = format!("{}", s.version);
    let total_frames_str = format!("{total_frames}");
    let active_str = format!("{active_frames}");
    let paused_str = format!("{paused_frames}");
    let completed_str = format!("{completed_frames}");
    let depth_str = format!("{stack_depth}");
    let candidates_str = format!("{total_candidates}");
    let surfaced_str = format!("{surfaced}");
    let suppressed_str = format!("{suppressed}");
    let pinned_str = format!("{pinned}");
    let pressure_str = format!("{max_pressure:.3}");
    let events_str = format!("{event_count}");

    let lines = vec![
        Line::raw(""),
        section("SESSION"),
        metric(
            "  Status",
            if has_session { "Active" } else { "None" },
            if has_session {
                theme::status_ok()
            } else {
                theme::label()
            },
        ),
        metric("  Version", &version_str, theme::value()),
        Line::raw(""),
        section("FOCUS STACK"),
        metric("  Total frames", &total_frames_str, theme::value()),
        metric("  Active", &active_str, theme::status_ok()),
        metric("  Paused/Suspended", &paused_str, theme::status_warn()),
        metric("  Completed/Archived", &completed_str, theme::label()),
        metric("  Stack depth", &depth_str, theme::value()),
        Line::raw(""),
        section("FOCUS GATE"),
        metric("  Candidates", &candidates_str, theme::value()),
        metric("  Surfaced", &surfaced_str, theme::highlight()),
        metric("  Suppressed", &suppressed_str, theme::label()),
        metric("  Pinned", &pinned_str, theme::status_warn()),
        metric(
            "  Max pressure",
            &pressure_str,
            pressure_style(max_pressure),
        ),
        Line::raw(""),
        section("EVENTS"),
        metric("  Recent count", &events_str, theme::value()),
    ];

    let para = Paragraph::new(lines).scroll((app.scroll_offset, 0));
    frame.render_widget(para, inner);
}

fn section(name: &str) -> Line<'_> {
    Line::from(Span::styled(format!("  ── {name} ──"), theme::highlight()))
}

fn metric<'a>(label: &'a str, value: &'a str, style: Style) -> Line<'a> {
    Line::from(vec![
        Span::styled(format!("{label}: "), theme::label()),
        Span::styled(value, style),
    ])
}

fn pressure_style(p: f64) -> Style {
    if p >= 0.8 {
        theme::status_err()
    } else if p >= 0.5 {
        theme::status_warn()
    } else {
        theme::value()
    }
}
