//! Autonomy Calibration view.

use crate::app::App;
use crate::theme;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render(app: &App, frame: &mut ratatui::Frame, area: Rect) {
    let block = Block::default()
        .title(" Autonomy Calibration ")
        .title_style(theme::title())
        .borders(Borders::ALL)
        .border_style(theme::border());

    let data = &app.extra_data.get("autonomy");
    let mut lines = Vec::new();

    if let Some(Some(d)) = data {
        let level = d["level"].as_str().unwrap_or("?");
        let ari = d["ari_score"].as_f64().unwrap_or(0.0);
        let samples = d["sample_count"].as_u64().unwrap_or(0);

        lines.push(Line::from(vec![
            Span::styled("Level: ", theme::label()),
            Span::styled(level, theme::value()),
        ]));
        lines.push(Line::from(vec![
            Span::styled("ARI Score: ", theme::label()),
            Span::styled(format!("{ari:.1}"), theme::value()),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Samples: ", theme::label()),
            Span::styled(samples.to_string(), theme::value()),
        ]));

        // Dimensions.
        if let Some(dims) = d.get("dimensions") {
            lines.push(Line::default());
            lines.push(Line::from("  Dimensions:").style(theme::label()));
            for dim in [
                "correctness",
                "stability",
                "efficiency",
                "trust",
                "grounding",
                "recovery",
            ] {
                let val = dims[dim].as_f64().unwrap_or(0.0);
                let bar_len = (val * 20.0) as usize;
                let bar = "█".repeat(bar_len) + &"░".repeat(20 - bar_len);
                lines.push(Line::from(vec![
                    Span::styled(format!("    {dim:12} "), theme::label()),
                    Span::styled(bar, theme::value()),
                    Span::styled(format!(" {val:.2}"), Style::default().fg(Color::DarkGray)),
                ]));
            }
        }
    } else {
        lines.push(Line::from("  No autonomy data").style(theme::label()));
    }

    let para = Paragraph::new(lines)
        .block(block)
        .scroll((app.scroll_offset, 0));
    frame.render_widget(para, area);
}
