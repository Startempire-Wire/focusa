//! Focusa TUI — terminal dashboard for cognitive runtime introspection.
//!
//! Read-only, event-driven, calm.
//! Polls the Focusa API and renders live state.

mod api;
mod app;
mod beginner_mode;
mod next_safe_action;
mod theme;
mod views;

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
    let mut args_iter = std::env::args().skip(1);
    let mut headless = false;
    for arg in args_iter.by_ref() {
        match arg.as_str() {
            "--headless-self-test" => headless = true,
            "--help" | "-h" => {
                println!("focusa-tui — Focusa Mission Deck");
                println!("Usage: focusa-tui [--headless-self-test]");
                println!("Env: FOCUSA_API_URL (default http://127.0.0.1:8787)");
                return Ok(());
            }
            other => {
                eprintln!(
                    "focusa-tui: unknown argument {other:?}; pass --headless-self-test or no args"
                );
                std::process::exit(2);
            }
        }
        if headless {
            break;
        }
    }
    let api_url =
        std::env::var("FOCUSA_API_URL").unwrap_or_else(|_| "http://127.0.0.1:8787".into());

    if headless {
        return run_headless_self_test(&api_url).await;
    }

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
                KeyCode::Char('d') | KeyCode::Char('n') => app.tab = app::Tab::DeckHome,
                KeyCode::Char('1') => app.tab = app::Tab::FocusState,
                KeyCode::Char('2') => app.tab = app::Tab::FocusStack,
                KeyCode::Char('3') => app.tab = app::Tab::Gate,
                KeyCode::Char('4') => app.tab = app::Tab::Events,
                KeyCode::Char('5') => app.tab = app::Tab::Metrics,
                KeyCode::Char('6') => app.tab = app::Tab::Lineage,
                KeyCode::Char('w') => app.tab = app::Tab::WorkLoop,
                KeyCode::Char('7') => app.tab = app::Tab::Autonomy,
                KeyCode::Char('8') => app.tab = app::Tab::Constitution,
                KeyCode::Char('9') => app.tab = app::Tab::Telemetry,
                KeyCode::Char('0') => app.tab = app::Tab::Rfm,
                KeyCode::Char('p') => app.tab = app::Tab::Proposals,
                KeyCode::Char('s') => app.tab = app::Tab::Skills,
                KeyCode::Char('u') => app.tab = app::Tab::Uxp,
                KeyCode::Char('x') => app.tab = app::Tab::Training,
                KeyCode::Char('r') => app.refresh().await,
                KeyCode::Char('h') | KeyCode::Char('?') => app.toggle_help(),
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

async fn run_headless_self_test(api_url: &str) -> Result<()> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()?;
    async fn fetch(client: &reqwest::Client, api: &str, path: &str) -> serde_json::Value {
        let url = format!("{}{}", api.trim_end_matches('/'), path);
        match client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => resp
                .json::<serde_json::Value>()
                .await
                .unwrap_or_else(|_| serde_json::json!({"raw_error": "decode_failed"})),
            Ok(resp) => serde_json::json!({"status": resp.status().as_u16(), "url": url}),
            Err(err) => serde_json::json!({"error": err.to_string(), "url": url}),
        }
    }
    let health = fetch(&client, api_url, "/v1/health").await;
    let focus_stack = fetch(&client, api_url, "/v1/focus/stack").await;
    let workpoint = fetch(&client, api_url, "/v1/workpoint/resume").await;
    let payload = serde_json::json!({
        "schema": "focusa.tui_headless_self_test.v1",
        "title": "Focusa Mission Deck",
        "default_tab": "DeckHome",
        "beginner_mode_decision_tree": crate::beginner_mode::DECISION_TREE,
        "help_overlay": {
            "toggle": ["h", "?"],
            "topics": crate::views::help_overlay::HELP_TOPICS,
        },
        "next_safe_action_model": crate::next_safe_action::HEADLESS_PROOF_STATES,
        "mission_ladder_levels": crate::views::mission_ladder::LADDER_LEVELS,
        "api_url": api_url,
        "health": health,
        "focus_stack": focus_stack,
        "workpoint": workpoint,
        "tabs": [
            "d:DeckHome", "1:FocusState", "2:FocusStack", "3:Gate", "4:Events", "5:Metrics",
            "6:Lineage", "w:WorkLoop", "7:Autonomy", "8:Constitution",
            "9:Telemetry", "0:Rfm", "p:Proposals", "s:Skills", "u:Uxp", "x:Training",
        ],
        "keybindings": {
            "quit": ["q", "Esc"],
            "refresh": ["r"],
            "help_overlay": ["h", "?"],
            "next_safe_action": ["n"],
            "next_tab": ["Tab"],
            "prev_tab": ["BackTab"],
            "scroll_down": ["Down", "j"],
            "scroll_up": ["Up", "k"],
        },
    });
    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}
