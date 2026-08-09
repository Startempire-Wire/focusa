#![recursion_limit = "256"]
//! Focusa TUI — terminal dashboard for cognitive runtime introspection.
//!
//! Read-only, event-driven, calm.
//! Polls the Focusa API and renders live state.

mod activation_presenter;
mod api;
mod app;
mod beginner_mode;
mod mission_control;
mod next_safe_action;
mod startup_perf;
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
    let mut no_intro = false;
    for arg in args_iter.by_ref() {
        match arg.as_str() {
            "--headless-self-test" => headless = true,
            "--no-intro" => no_intro = true,
            "--version" | "-V" => {
                println!("focusa-tui {}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
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

    let mut app = app::App::new_with_intro(api_url, !no_intro);

    // Initial fetch.
    app.refresh().await;

    // TTY guard: crossterm's enable_raw_mode() crashes with ENXIO on macOS
    // when stdout isn't a terminal (SSH, tmux pane, background job, CI).
    // Detect non-interactive contexts and route to headless output rather
    // than crashing. Also gate EnterAlternateScreen — it's only valid for TTYs.
    let stdout_is_tty = std::io::IsTerminal::is_terminal(&std::io::stdout());
    if !stdout_is_tty {
        eprintln!(
            "FOCUSA_TUI_NON_TTY: stdout is not a terminal; interactive raw mode is unavailable.\n\
             Recovery: run `focusa tui --headless-self-test` for structured diagnostics,\n\
             or launch `focusa tui` from a real terminal. If installed discovery fails,\n\
             run `focusa install --dry-run`, reinstall, or set FOCUSA_TUI_BIN."
        );
        std::process::exit(64);
    }

    // Terminal setup.
    // Defensive: enable_raw_mode can still fail on some SSH/tmux setups where
    // is_terminal() returns true but the underlying device doesn't support raw
    // mode (ENXIO/os error 6). Catch and convert to a clean exit instead of panic.
    if let Err(e) = enable_raw_mode() {
        eprintln!("focusa-tui: failed to enable raw mode: {e}");
        eprintln!("Run with --headless-self-test for structured output, or use a real terminal.");
        std::process::exit(65);
    }
    let mut stdout = io::stdout();
    if let Err(e) = execute!(stdout, EnterAlternateScreen) {
        let _ = disable_raw_mode();
        eprintln!("focusa-tui: failed to enter alternate screen: {e}");
        eprintln!("Run with --headless-self-test for structured output, or use a real terminal.");
        std::process::exit(66);
    }
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
                _ if app.show_intro => app.dismiss_intro(),
                KeyCode::Char('1') if matches!(app.modal, Some(crate::app::ModalKind::Learn)) => {
                    app.modal_selection = 0;
                }
                KeyCode::Char('2') if matches!(app.modal, Some(crate::app::ModalKind::Learn)) => {
                    app.modal_selection = 1;
                }
                KeyCode::Char('3') if matches!(app.modal, Some(crate::app::ModalKind::Learn)) => {
                    app.modal_selection = 2;
                }
                KeyCode::Char('4') if matches!(app.modal, Some(crate::app::ModalKind::Learn)) => {
                    app.modal_selection = 3;
                }
                KeyCode::Char('5') if matches!(app.modal, Some(crate::app::ModalKind::Learn)) => {
                    app.modal_selection = 4;
                }
                KeyCode::Char('d') | KeyCode::Char('n') => app.tab = app::Tab::DeckHome,
                KeyCode::Char('1') => app.tab = app::Tab::FocusState,
                KeyCode::Char('2') => app.tab = app::Tab::FocusStack,
                KeyCode::Char('3') => app.tab = app::Tab::Gate,
                KeyCode::Char('4') => app.tab = app::Tab::Events,
                KeyCode::Char('5') => app.tab = app::Tab::Metrics,
                KeyCode::Char('6') => app.tab = app::Tab::Lineage,
                KeyCode::Char('w') => app.tab = app::Tab::WorkLoop,
                KeyCode::Char('/') => app.open_modal(crate::app::ModalKind::Recall),
                KeyCode::Char('A') | KeyCode::Char('a') => {
                    app.open_modal(crate::app::ModalKind::About)
                }
                KeyCode::Char('7') => app.tab = app::Tab::Autonomy,
                KeyCode::Char('8') => app.tab = app::Tab::Constitution,
                KeyCode::Char('9') => app.tab = app::Tab::Telemetry,
                KeyCode::Char('0') => app.tab = app::Tab::Rfm,
                KeyCode::Char('p') => app.tab = app::Tab::Proposals,
                KeyCode::Char('s') => app.tab = app::Tab::Skills,
                KeyCode::Char('u') => app.tab = app::Tab::Uxp,
                KeyCode::Char('L') | KeyCode::Char('l') => {
                    app.open_modal(crate::app::ModalKind::Learn)
                }
                KeyCode::Char('x') => app.tab = app::Tab::Training,
                KeyCode::Char('r') => app.refresh().await,
                KeyCode::Char('h') | KeyCode::Char('?') => app.toggle_help(),
                KeyCode::Down | KeyCode::Char('j') => app.scroll_down(),
                KeyCode::Up | KeyCode::Char('k') => app.scroll_up(),
                _ => {}
            }
        }

        // Animation tick.
        app.tick_throbber();

        // Periodic refresh.
        if last_refresh.elapsed() >= refresh_rate {
            app.refresh().await;
            last_refresh = std::time::Instant::now();
            if app.show_intro {
                app.tick_intro_dismiss(last_refresh.elapsed().as_millis());
            }
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
        "ui_architecture": "mission_control_canvas_plus_modal_overlays",
        "mission_control_mobile_breakpoint_cols": crate::mission_control::MOBILE_BREAKPOINT_COLS,
        "mission_control": {
            "architecture": "mission_control_canvas_plus_modal_overlays",
            "mobile_breakpoint_cols": crate::mission_control::MOBILE_BREAKPOINT_COLS,
            "compact_keys_hint": crate::mission_control::COMPACT_KEYS_HINT,
            "full_keys_hint": crate::mission_control::FULL_KEYS_HINT,
        },
        "modal_shortcuts": {
            "recall": "/",
            "learn": "l",
            "help": "?",
            "command_palette": ":",
            "about": "a",
            "close": "Esc"
        },
        "about_logo": crate::views::intro::FOCUSA_LOGO,
        "about_tagline": crate::views::intro::FOCUSA_TAGLINE,
        "focusa_tags": crate::views::intro::FOCUSA_TAGS,
        "focusa_tags_line": crate::views::intro::FOCUSA_TAGS_LINE,
        "about_version": crate::views::about::ABOUT_VERSION,
        "about_credits_count": crate::views::about::ABOUT_CREDITS.len(),
        "intro_splash": {
            "logo": crate::views::intro::FOCUSA_LOGO,
            "tagline": crate::views::intro::FOCUSA_TAGLINE,
            "version_line": crate::views::intro::INTRO_VERSION_LINE,
        },
        "default_tab": "DeckHome",
        "deck_home_beautification_checklist": crate::views::deck_home::BEAUTIFICATION_CHECKLIST,
        "beginner_mode_decision_tree": crate::beginner_mode::DECISION_TREE,
        "beginner_mode_affordance_by_state": crate::beginner_mode::AFFORDANCE_REALITY_BY_BEGINNER_STATE
            .iter()
            .map(|(state, affordance)| serde_json::json!({"state": state, "affordance_reality": affordance}))
            .collect::<Vec<_>>(),
        "help_overlay": {
            "toggle": ["h", "?"],
            "topics": crate::views::help_overlay::HELP_TOPICS,
        },
        "startup": {
            "first_paint_budget_ms": crate::startup_perf::FIRST_PAINT_BUDGET_MS,
            "shell_render_phases": crate::startup_perf::SHELL_RENDER_PHASES,
            "progressive_loading_plan": crate::startup_perf::PROGRESSIVE_LOADING_PLAN,
        },
        "next_safe_action_model": crate::next_safe_action::HEADLESS_PROOF_STATES,
        "next_safe_action_recovery_tool_cap": crate::next_safe_action::HEADLESS_PROOF_RECOVERY_TOOL_CAP,
        "mission_ladder_levels": crate::views::mission_ladder::LADDER_LEVELS,
        "proof_meter_states": crate::views::proof_status::PROOF_METER_STATES,
        "affordance_reality_states": [
            crate::views::proof_status::AFFORDANCE_REALITY_POSSIBLE,
            crate::views::proof_status::AFFORDANCE_REALITY_LIMITED,
            crate::views::proof_status::AFFORDANCE_REALITY_UNAVAILABLE,
        ],
        "scope_badge_states": crate::views::proof_status::SCOPE_BADGE_STATES,
        "precedence_frames": [
            crate::views::proof_status::PRECEDENCE_FRAME_PROJECT,
            crate::views::proof_status::PRECEDENCE_FRAME_AUTHORITY,
            crate::views::proof_status::PRECEDENCE_FRAME_OPERATOR,
        ],
        "walkthrough_education": {
            "ids": crate::views::walkthroughs::WALKTHROUGH_IDS,
            "titles": crate::views::walkthroughs::WALKTHROUGH_TITLES,
            "audiences": crate::views::walkthroughs::WALKTHROUGH_AUDIENCES,
            "why_it_matters": crate::views::walkthroughs::WALKTHROUGH_WHY_IT_MATTERS,
            "success_signals": crate::views::walkthroughs::WALKTHROUGH_SUCCESS_SIGNALS,
            "anti_drift_rules": crate::views::walkthroughs::WALKTHROUGH_ANTI_DRIFT_RULES,
        },
        "recall_tab": {
            "hotkey": "/",
            "sources": crate::views::recall::RECALL_SEARCH_SOURCES,
            "card_fields": crate::views::recall::RECALL_CARD_FIELDS,
            "authority_rule": crate::views::recall::RECALL_AUTHORITY_RULE,
            "memory_status_values": crate::views::recall::MEMORY_STATUS_VALUES,
            "scope_status_values": crate::views::recall::SCOPE_STATUS_VALUES,
            "proof_status_values": crate::views::recall::PROOF_STATUS_VALUES,
            "allowed_use_values": crate::views::recall::ALLOWED_USE_VALUES,
            "workpoint_candidate_promotion_flow": crate::views::recall::WORKPOINT_CANDIDATE_PROMOTION_FLOW,
            "workpoint_candidate_preview_state": crate::views::recall::WorkpointCandidatePromotion::recall_default().preview_state,
            "workpoint_candidate_preview_only": crate::views::recall::WorkpointCandidatePromotion::recall_default().is_preview_only(),
            "workpoint_candidate_forbidden": crate::views::recall::WORKPOINT_CANDIDATE_FORBIDDEN,
        },
        "api_url": api_url,
        "health": health,
        "focus_stack": focus_stack,
        "workpoint": workpoint,
        "tabs": [
            "d:DeckHome", "1:FocusState", "2:FocusStack", "3:Gate", "4:Events", "5:Metrics",
            "6:Lineage", "w:WorkLoop", "/:RecallModal", "a:AboutModal", "l:LearnModal",
            "?:HelpModal", "::CommandPalette", "7:Autonomy", "8:Constitution",
            "9:Telemetry", "0:Rfm", "p:Proposals", "s:Skills", "u:Uxp", "x:Training",
        ],
        "keybindings": {
            "quit": ["q", "Esc when no modal is open"],
            "modal_close": ["Esc"],
            "refresh": ["r"],
            "help_overlay": ["h"],
            "help_modal": ["?"],
            "command_palette": [":"],
            "next_safe_action": ["n"],
            "recall": ["/"],

            "scroll_down": ["Down", "j"],
            "scroll_up": ["Up", "k"],
        },
    });
    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}
