//! Mission Ladder visual panel (Spec 117 §6.4 / §14.1).

use crate::app::App;
use crate::theme;
use ratatui::prelude::*;
use ratatui::widgets::*;
use serde_json::Value;

pub const LADDER_LEVELS: &[&str] = &["HLT", "MLG", "STG", "Workpoint", "Evidence"];

pub fn render(app: &App, frame: &mut ratatui::Frame, area: Rect) {
    let lines = ladder_lines(app);
    let block = Block::default()
        .title(" Mission Ladder ")
        .title_style(theme::title())
        .borders(Borders::ALL)
        .border_style(theme::border());
    frame.render_widget(
        Paragraph::new(lines).block(block).wrap(Wrap { trim: true }),
        area,
    );
}

pub fn ladder_lines(app: &App) -> Vec<Line<'static>> {
    let trajectory = app
        .extra_data
        .get("trajectory_view")
        .and_then(|value| value.as_ref());
    let workpoint = app
        .extra_data
        .get("workpoint_resume")
        .and_then(|value| value.as_ref());

    vec![
        Line::from(format!(
            "HLT: {}",
            value_text(trajectory, &["long_term_goal", "hlt", "goal"])
        )),
        Line::from(format!(
            " └─ MLG: {}",
            value_text(trajectory, &["mid_level_goal", "mlg"])
        )),
        Line::from(format!(
            "     └─ STG: {}",
            value_text(trajectory, &["short_term_goal", "stg", "next"])
        )),
        Line::from(format!(
            "         └─ Workpoint: {}",
            workpoint_id(workpoint)
        )),
        Line::from(format!(
            "             └─ Evidence: {}",
            evidence_status(workpoint)
        )),
    ]
}

fn value_text(source: Option<&Value>, keys: &[&str]) -> String {
    let Some(value) = source else {
        return "unavailable — run focusa trajectory view".to_string();
    };
    for key in keys {
        if let Some(found) = value.get(*key).and_then(Value::as_str) {
            if !found.trim().is_empty() {
                return trim(found);
            }
        }
        let pointer = format!("/trajectory/{key}");
        if let Some(found) = value.pointer(&pointer).and_then(Value::as_str) {
            if !found.trim().is_empty() {
                return trim(found);
            }
        }
    }
    "unavailable — checkpoint mission hierarchy".to_string()
}

fn workpoint_id(source: Option<&Value>) -> String {
    let Some(value) = source else {
        return "unavailable — run focusa workpoint resume".to_string();
    };
    for key in ["id", "workpoint_id"] {
        if let Some(found) = value.get(key).and_then(Value::as_str) {
            if !found.trim().is_empty() && found != "none" {
                return trim(found);
            }
        }
    }
    "unavailable — checkpoint current mission".to_string()
}

fn evidence_status(source: Option<&Value>) -> String {
    let Some(value) = source else {
        return "unavailable — attach or declare proof gap".to_string();
    };
    let count = value
        .get("verified_evidence")
        .or_else(|| value.get("evidence_refs"))
        .or_else(|| value.pointer("/packet/verified_evidence"))
        .and_then(Value::as_array)
        .map(|items| items.len())
        .unwrap_or(0);
    if count > 0 {
        format!("linked ({count} refs)")
    } else {
        "missing — no proof visible yet".to_string()
    }
}

fn trim(value: &str) -> String {
    const MAX: usize = 72;
    let compact = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if compact.chars().count() > MAX {
        format!("{}…", compact.chars().take(MAX).collect::<String>())
    } else {
        compact
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;

    #[test]
    fn levels_match_spec() {
        assert_eq!(
            LADDER_LEVELS,
            &["HLT", "MLG", "STG", "Workpoint", "Evidence"]
        );
    }

    #[test]
    fn missing_state_renders_unavailable_recovery_text() {
        let app = App::new("http://127.0.0.1:8787".into());
        let text = ladder_lines(&app)
            .into_iter()
            .map(|line| line.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(text.contains("HLT:"));
        assert!(text.contains("unavailable"));
        assert!(text.contains("Evidence:"));
    }
}
