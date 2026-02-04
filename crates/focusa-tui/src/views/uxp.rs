//! UXP / UFI view.

use crate::app::App;
use crate::theme;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render(app: &App, frame: &mut ratatui::Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    render_uxp(app, frame, chunks[0]);
    render_ufi(app, frame, chunks[1]);
}

fn render_uxp(app: &App, frame: &mut ratatui::Frame, area: Rect) {
    let block = Block::default()
        .title(" UXP Profile ")
        .title_style(theme::title())
        .borders(Borders::ALL)
        .border_style(theme::border());

    let data = &app.extra_data.get("uxp");
    let mut lines = Vec::new();

    if let Some(Some(d)) = data {
        for dim in ["autonomy_tolerance", "verbosity_preference", "interruption_sensitivity",
                     "explanation_depth", "confirmation_preference", "risk_tolerance", "review_cadence"] {
            if let Some(dim_data) = d.get(dim) {
                let val = dim_data["value"].as_f64().unwrap_or(0.5);
                let conf = dim_data["confidence"].as_f64().unwrap_or(0.0);
                let frozen = dim_data["frozen"].as_bool().unwrap_or(false);
                let bar_len = (val * 15.0) as usize;
                let bar = "█".repeat(bar_len) + &"░".repeat(15 - bar_len);
                let freeze_icon = if frozen { " 🔒" } else { "" };
                lines.push(Line::from(vec![
                    Span::styled(format!("{:24} ", dim.replace('_', " ")), theme::label()),
                    Span::styled(bar, theme::value()),
                    Span::styled(format!(" {val:.2} c={conf:.2}{freeze_icon}"), Style::default().fg(Color::DarkGray)),
                ]));
            }
        }
    } else {
        lines.push(Line::from("  No UXP data").style(theme::label()));
    }

    let para = Paragraph::new(lines)
        .block(block)
        .scroll((app.scroll_offset, 0));
    frame.render_widget(para, area);
}

fn render_ufi(app: &App, frame: &mut ratatui::Frame, area: Rect) {
    let block = Block::default()
        .title(" UFI Signals ")
        .title_style(theme::title())
        .borders(Borders::ALL)
        .border_style(theme::border());

    let data = &app.extra_data.get("ufi");
    let mut lines = Vec::new();

    if let Some(Some(d)) = data {
        let aggregate = d["aggregate"].as_f64().unwrap_or(0.0);
        let count = d["signal_count"].as_u64().unwrap_or(0);

        lines.push(Line::from(vec![
            Span::styled("Aggregate: ", theme::label()),
            Span::styled(format!("{aggregate:.3}"), theme::value()),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Signals: ", theme::label()),
            Span::styled(count.to_string(), theme::value()),
        ]));

        if let Some(signals) = d["signals"].as_array() {
            lines.push(Line::default());
            for s in signals.iter().rev().take(15) {
                let stype = s["signal_type"].as_str().unwrap_or("?");
                let tier = s["weight_tier"].as_str().unwrap_or("?");
                let tier_color = match tier {
                    "high" => Color::Red,
                    "medium" => Color::Yellow,
                    _ => Color::DarkGray,
                };
                lines.push(Line::from(vec![
                    Span::styled(format!("[{tier}] "), Style::default().fg(tier_color)),
                    Span::styled(stype, theme::value()),
                ]));
            }
        }
    } else {
        lines.push(Line::from("  No UFI data").style(theme::label()));
    }

    let para = Paragraph::new(lines)
        .block(block)
        .scroll((app.scroll_offset, 0));
    frame.render_widget(para, area);
}
