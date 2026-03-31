//! Cache view — cache status and recent busts.
//!
//! Per 27-tui-spec §12: cache classes, live hit/miss feed, recent bust reasons.

use crate::app::App;
use crate::theme;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render(app: &App, frame: &mut ratatui::Frame, area: Rect) {
    let block = Block::default()
        .title(" Cache Status ")
        .title_style(theme::title())
        .borders(Borders::ALL)
        .border_style(theme::border());

    // Get cache data from extra_data
    let cache_data = app.extra_data.get("cache").and_then(|v| v.as_ref());

    if cache_data.is_none() {
        let para = Paragraph::new("Loading cache status...")
            .style(theme::label())
            .block(block);
        frame.render_widget(para, area);
        return;
    }

    let data = cache_data.unwrap();
    
    // Build stats text
    let entry_count = data.get("entry_count").and_then(|v| v.as_u64()).unwrap_or(0);
    let hit_rate = data.get("hit_rate").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let bust_events = data.get("bust_events").and_then(|v| v.as_array()).cloned().unwrap_or_default();

    let stats_text = format!(
        "Entries: {}  │  Hit Rate: {:.1}%  │  Recent Busts: {}\n\n",
        entry_count,
        hit_rate * 100.0,
        bust_events.len()
    );

    // Build bust events list
    let bust_text: String = bust_events
        .iter()
        .take(10)
        .map(|e| {
            let category = e.get("category").and_then(|v| v.as_str()).unwrap_or("?");
            let reason = e.get("reason").and_then(|v| v.as_str()).unwrap_or("?");
            format!("[{}] {}\n", category, reason)
        })
        .collect();

    let full_text = if bust_text.is_empty() {
        format!("{}\nNo recent cache bust events.", stats_text)
    } else {
        format!("{}\nRecent Cache Busts:\n{}", stats_text, bust_text)
    };

    let para = Paragraph::new(full_text)
        .style(theme::label())
        .block(block)
        .wrap(Wrap { trim: true });

    frame.render_widget(para, area);
}
