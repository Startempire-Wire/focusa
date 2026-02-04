//! Focusa TUI — terminal dashboard for cognitive runtime introspection.
//!
//! Read-only, event-driven, calm.
//! Polls the Focusa API and renders live state.

mod app;
mod api;
mod views;
mod theme;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::prelude::*;
use std::io;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    let api_url = std::env::var("FOCUSA_API_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8787".into());

    let mut app = app::App::new(api_url);

    // Initial fetch.
    app.refresh().await;

    // Terminal setup.
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Main loop.
    let tick_rate = Duration::from_millis(250);
    let refresh_rate = Duration::from_secs(2);
    let mut last_refresh = std::time::Instant::now();

    loop {
        terminal.draw(|f| views::render(&app, f))?;

        if event::poll(tick_rate)?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Char('1') => app.tab = app::Tab::FocusState,
                KeyCode::Char('2') => app.tab = app::Tab::FocusStack,
                KeyCode::Char('3') => app.tab = app::Tab::Gate,
                KeyCode::Char('4') => app.tab = app::Tab::Events,
                KeyCode::Char('5') => app.tab = app::Tab::Metrics,
                KeyCode::Char('6') => app.tab = app::Tab::Lineage,
                KeyCode::Char('7') => app.tab = app::Tab::Autonomy,
                KeyCode::Char('8') => app.tab = app::Tab::Constitution,
                KeyCode::Char('9') => app.tab = app::Tab::Telemetry,
                KeyCode::Char('0') => app.tab = app::Tab::Rfm,
                KeyCode::Char('p') => app.tab = app::Tab::Proposals,
                KeyCode::Char('s') => app.tab = app::Tab::Skills,
                KeyCode::Char('u') => app.tab = app::Tab::Uxp,
                KeyCode::Char('x') => app.tab = app::Tab::Training,
                KeyCode::Char('r') => app.refresh().await,
                KeyCode::Tab => app.next_tab(),
                KeyCode::BackTab => app.prev_tab(),
                KeyCode::Down | KeyCode::Char('j') => app.scroll_down(),
                KeyCode::Up | KeyCode::Char('k') => app.scroll_up(),
                _ => {}
            }
        }

        // Periodic refresh.
        if last_refresh.elapsed() >= refresh_rate {
            app.refresh().await;
            last_refresh = std::time::Instant::now();
        }
    }

    // Cleanup.
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
