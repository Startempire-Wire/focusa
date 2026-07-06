# Spec 117 TUI Welcome Intro Splash proof — 2026-07-06

Scope: focusa-117-arch.35 TUI Welcome Intro splash.

## Canonical content
- LOGO: `FOCUSA`
- TAGLINE: `FOCUSA - Local-first mission cohesion for AI coding agents.`

## Changes
- `crates/focusa-tui/src/views/intro.rs` exposes FOCUSA_LOGO + FOCUSA_TAGLINE constants, intro_lines() builder, and render(app, frame, area).
- intro::render draws a centered responsive popup; mobile-safe (all lines ≤ 72 chars).
- `App` gains `show_intro: bool`, `dismiss_intro()`, `tick_intro_dismiss(elapsed_ms)`.
- `views/mod.rs` calls `intro::render` last so splash renders above the regular Mission Deck.
- `main.rs` adds `--no-intro` flag, auto-dismisses after 2500ms, dismisses on any keypress.
- Headless proof exposes intro_splash { logo, tagline, version_line }.

## Tests/gates
- cargo test --release -p focusa-tui -- intro: PASS (4 tests)
- cargo build --release -p focusa-tui: PASS
- tests/spec_focusa_117_tui_intro_splash_static_test.sh: PASS
- tests/release_deploy_automation_static_test.sh: PASS
