//! Telemetry view.

use crate::app::App;
use crate::theme;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render(app: &App, frame: &mut ratatui::Frame, area: Rect) {
    let block = Block::default()
        .title(" Cognitive Telemetry ")
        .title_style(theme::title())
        .borders(Borders::ALL)
        .border_style(theme::border());

    let data = &app.extra_data.get("telemetry");
    let mut lines = Vec::new();

    if let Some(Some(d)) = data {
        let events = d["total_events"].as_u64().unwrap_or(0);
        let prompt = d["total_prompt_tokens"].as_u64().unwrap_or(0);
        let completion = d["total_completion_tokens"].as_u64().unwrap_or(0);

        lines.push(Line::from(vec![
            Span::styled("Total Events: ", theme::label()),
            Span::styled(events.to_string(), theme::value()),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Prompt Tokens: ", theme::label()),
            Span::styled(format!("{prompt:>10}"), theme::value()),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Completion Tokens: ", theme::label()),
            Span::styled(format!("{completion:>10}"), theme::value()),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Total Tokens: ", theme::label()),
            Span::styled(format!("{:>10}", prompt + completion), theme::value()),
        ]));

        if let Some(tasks) = d["tokens_per_task"].as_array() {
            lines.push(Line::default());
            lines.push(Line::from("  Per-Task Breakdown:").style(theme::label()));
            for t in tasks {
                let id = t["task_id"].as_str().unwrap_or("?");
                let tp = t["prompt_tokens"].as_u64().unwrap_or(0);
                let tc = t["completion_tokens"].as_u64().unwrap_or(0);
                let turns = t["turns"].as_u64().unwrap_or(0);
                lines.push(
                    Line::from(format!("    {id}: {tp}+{tc} tokens, {turns} turns"))
                        .style(theme::value()),
                );
            }
        }
    } else {
        lines.push(Line::from("  No telemetry data").style(theme::label()));
    }

    let para = Paragraph::new(lines)
        .block(block)
        .scroll((app.scroll_offset, 0));
    frame.render_widget(para, area);
}
