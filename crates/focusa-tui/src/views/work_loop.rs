//! Work-loop replay dashboard view.

use crate::app::App;
use crate::theme;
use ratatui::prelude::*;
use ratatui::widgets::*;
use serde_json::Value;

pub fn render(app: &App, frame: &mut ratatui::Frame, area: Rect) {
    let block = Block::default()
        .title(" Work Loop Replay ")
        .title_style(theme::title())
        .borders(Borders::ALL)
        .border_style(theme::border());

    let status_payload = app
        .extra_data
        .get("work_loop_status")
        .and_then(|value| value.as_ref());
    let replay_payload = app
        .extra_data
        .get("work_loop_replay")
        .and_then(|value| value.as_ref());
    let closure_bundle = app
        .extra_data
        .get("work_loop_closure_bundle")
        .and_then(|value| value.as_ref());

    let loop_status = closure_bundle
        .and_then(|value| value.get("work_loop"))
        .or(status_payload);
    let replay_consumer = closure_bundle
        .and_then(|value| value.get("secondary_loop_replay_consumer"))
        .or(replay_payload);
    let continuity_gate = closure_bundle
        .and_then(|value| value.get("secondary_loop_continuity_gate"))
        .or_else(|| status_payload.and_then(|value| value.get("secondary_loop_continuity_gate")))
        .or_else(|| replay_consumer.and_then(|value| value.get("secondary_loop_continuity_gate")));
    let objective_profile = closure_bundle
        .and_then(|value| value.get("secondary_loop_eval_bundle"))
        .and_then(|value| value.get("secondary_loop_objective_profile"))
        .or_else(|| {
            status_payload
                .and_then(|value| value.get("secondary_loop_eval_bundle"))
                .and_then(|value| value.get("secondary_loop_objective_profile"))
        });

    let mut lines: Vec<Line<'static>> = Vec::new();

    if loop_status.is_none() && replay_consumer.is_none() {
        lines.push(Line::from("  No work-loop replay data").style(theme::label()));
    } else {
        if let Some(doc) = closure_bundle
            .and_then(|value| value.get("doc"))
            .and_then(Value::as_str)
        {
            lines.push(metric("  Closure bundle doc", doc, theme::value()));
        }

        let enabled = loop_status
            .and_then(|value| value.get("enabled"))
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let status = loop_status
            .and_then(|value| value.get("status"))
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let project = loop_status
            .and_then(|value| value.get("project_status"))
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let tranche = loop_status
            .and_then(|value| value.get("tranche_status"))
            .and_then(Value::as_str)
            .unwrap_or("unknown");

        lines.push(metric(
            "  Loop",
            format!("{status} (enabled={})", if enabled { "yes" } else { "no" }),
            if enabled {
                theme::status_ok()
            } else {
                theme::label()
            },
        ));
        lines.push(metric("  Project", project, theme::value()));
        lines.push(metric("  Tranche", tranche, theme::value()));

        if let Some(task) = loop_status
            .and_then(|value| value.get("current_task"))
            .and_then(|value| value.get("work_item_id"))
            .and_then(Value::as_str)
        {
            lines.push(metric("  Current task", task, theme::highlight()));
        }

        let replay_status = replay_consumer
            .and_then(|value| value.get("status"))
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        lines.push(metric(
            "  Replay consumer",
            replay_status,
            replay_status_style(replay_status),
        ));

        let continuity_gate_state = continuity_gate
            .and_then(|value| value.get("state"))
            .and_then(Value::as_str)
            .unwrap_or_else(|| {
                if replay_status == "ok" {
                    "open"
                } else {
                    "fail-closed"
                }
            });
        lines.push(metric(
            "  Continuity gate",
            continuity_gate_state,
            if continuity_gate_state == "open" {
                theme::status_ok()
            } else {
                theme::status_err()
            },
        ));

        let pair_observed = replay_consumer
            .and_then(|value| value.get("secondary_loop_closure_replay_evidence"))
            .and_then(|value| value.get("evidence"))
            .and_then(|value| value.get("current_task_pair_observed"))
            .and_then(Value::as_bool);
        let pair_label = match pair_observed {
            Some(true) => "observed",
            Some(false) => "missing",
            None => "unknown",
        };
        lines.push(metric("  Current task pair", pair_label, theme::value()));

        let comparative_pairs = replay_consumer
            .and_then(|value| value.get("secondary_loop_replay_comparative"))
            .and_then(|value| value.get("summary"))
            .and_then(|value| value.get("comparative_improvement_pairs"))
            .and_then(Value::as_u64)
            .unwrap_or(0);
        lines.push(metric(
            "  Comparative pairs",
            comparative_pairs.to_string(),
            if comparative_pairs > 0 {
                theme::status_ok()
            } else {
                theme::label()
            },
        ));

        let replay_scanned = replay_consumer
            .and_then(|value| value.get("secondary_loop_closure_replay_evidence"))
            .and_then(|value| value.get("evidence"))
            .and_then(|value| value.get("replay_events_scanned"))
            .and_then(Value::as_u64)
            .unwrap_or(0);
        lines.push(metric(
            "  Replay events scanned",
            replay_scanned.to_string(),
            theme::value(),
        ));

        if let Some(non_closure_events) = objective_profile
            .and_then(|value| value.get("non_closure_objective_events"))
            .and_then(Value::as_u64)
        {
            lines.push(metric(
                "  Non-closure objectives",
                non_closure_events.to_string(),
                if non_closure_events > 0 {
                    theme::status_ok()
                } else {
                    theme::label()
                },
            ));
        }

        if let Some(rate) = objective_profile
            .and_then(|value| value.get("non_closure_objective_rate"))
            .and_then(Value::as_f64)
        {
            lines.push(metric(
                "  Non-closure objective rate",
                format!("{:.1}%", rate * 100.0),
                if rate > 0.0 {
                    theme::status_ok()
                } else {
                    theme::label()
                },
            ));
        }

        if let Some(dominant_objective) = objective_profile
            .and_then(|value| value.get("dominant_objective"))
            .and_then(Value::as_str)
        {
            lines.push(metric(
                "  Dominant objective",
                dominant_objective,
                theme::value(),
            ));
        }

        if let Some(reason) = continuity_gate
            .and_then(|value| value.get("reason"))
            .and_then(Value::as_str)
        {
            lines.push(metric("  Gate reason", reason, theme::label()));
        }
    }

    let para = Paragraph::new(lines)
        .block(block)
        .scroll((app.scroll_offset, 0));
    frame.render_widget(para, area);
}

fn metric(label: &str, value: impl Into<String>, style: Style) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label}: "), theme::label()),
        Span::styled(value.into(), style),
    ])
}

fn replay_status_style(status: &str) -> Style {
    match status {
        "ok" => theme::status_ok(),
        "error" => theme::status_err(),
        _ => theme::label(),
    }
}
