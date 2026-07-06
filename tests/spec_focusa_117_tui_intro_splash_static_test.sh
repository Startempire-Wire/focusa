#!/usr/bin/env bash
# Spec 117 §6 polish — TUI Welcome Intro splash static guard.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

INTRO="$ROOT_DIR/crates/focusa-tui/src/views/intro.rs"
APP="$ROOT_DIR/crates/focusa-tui/src/app.rs"
MAIN="$ROOT_DIR/crates/focusa-tui/src/main.rs"
VIEWS="$ROOT_DIR/crates/focusa-tui/src/views/mod.rs"
[[ -f "$INTRO" ]] || fail "intro.rs missing"

for needle in \
  'FOCUSA_LOGO' \
  'FOCUSA_TAGLINE' \
  'pub const FOCUSA_LOGO: &str = "FOCUSA"' \
  'pub const FOCUSA_TAGLINE: &str = "Local-first mission cohesion for AI coding agents."' \
  'pub fn render' \
  'pub fn intro_lines' \
  'Press any key' \
  'auto-dismiss'; do
  grep -qF -- "$needle" "$INTRO" || fail "intro missing: $needle"
done
pass "intro exposes canonical FOCUSA LOGO + TAGLINE"

grep -qF 'pub show_intro: bool' "$APP" || fail "App missing show_intro state"
grep -qF 'pub fn dismiss_intro' "$APP" || fail "App missing dismiss_intro"
grep -qF 'pub fn tick_intro_dismiss' "$APP" || fail "App missing tick_intro_dismiss"
pass "App wires show_intro + dismiss + auto-tick"

grep -qF 'pub mod intro;' "$VIEWS" || fail "views mod missing intro export"
grep -qF 'intro::render' "$VIEWS" || fail "views render does not call intro::render"
pass "intro rendered above the regular Mission Deck surface"

grep -qF '"--no-intro"' "$MAIN" || fail "main missing --no-intro flag"
grep -qF 'new_with_intro(api_url, !no_intro)' "$MAIN" || fail "main does not honor --no-intro on launch"
grep -qF 'app.tick_intro_dismiss' "$MAIN" || fail "main does not call tick_intro_dismiss"
grep -qF 'app.dismiss_intro' "$MAIN" || fail "main does not dismiss intro on keypress"
pass "main honors --no-intro, auto-dismiss on timeout, dismiss on keypress"

echo "focusa-117 tui-intro-splash static test: PASS"
