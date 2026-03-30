//! Reliability Focus Mode view.

use crate::app::App;
use crate::theme;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render(app: &App, frame: &mut ratatui::Frame, area: Rect) {
    let block = Block::default()
        .title(" Reliability Focus Mode ")
        .title_style(theme::title())
        .borders(Borders::ALL)
        .border_style(theme::border());

    let data = &app.extra_data.get("rfm");
    let mut lines = Vec::new();

    if let Some(Some(d)) = data {
        let level = d["level"].as_str().unwrap_or("R0");
        let ais = d["ais_score"].as_f64().unwrap_or(1.0);
        let regen = d["needs_regeneration"].as_bool().unwrap_or(false);
        let ensemble = d["needs_ensemble"].as_bool().unwrap_or(false);

        let level_color = match level {
            "R0" => Color::Green,
            "R1" => Color::Yellow,
            "R2" => Color::Rgb(255, 165, 0),
            _ => Color::Red,
        };

        lines.push(Line::from(vec![
            Span::styled("Level: ", theme::label()),
            Span::styled(
                level,
                Style::default()
                    .fg(level_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::styled("AIS: ", theme::label()),
            Span::styled(format!("{ais:.2}"), theme::value()),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Regeneration: ", theme::label()),
            Span::styled(if regen { "needed" } else { "not needed" }, theme::value()),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Ensemble: ", theme::label()),
            Span::styled(
                if ensemble { "needed" } else { "not needed" },
                theme::value(),
            ),
        ]));

        if let Some(results) = d["validator_results"].as_array() {
            lines.push(Line::default());
            lines.push(Line::from("  Validators:").style(theme::label()));
            for r in results {
                let validator = r["validator"].as_str().unwrap_or("?");
                let passed = r["passed"].as_bool().unwrap_or(false);
                let icon = if passed { "✓" } else { "✗" };
                let color = if passed { Color::Green } else { Color::Red };
                lines.push(Line::from(vec![
                    Span::styled(format!("    {icon} "), Style::default().fg(color)),
                    Span::styled(validator, theme::value()),
                ]));
            }
        }
    } else {
        lines.push(Line::from("  No RFM data").style(theme::label()));
    }

    let para = Paragraph::new(lines)
        .block(block)
        .scroll((app.scroll_offset, 0));
    frame.render_widget(para, area);
}
