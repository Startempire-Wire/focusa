#!/usr/bin/env bash
# spec_focusa_112_about_command_static_test.sh
#
# Static guard for focusa-112-about-command: `focusa about` first-impressions card.
# Closes the transcript gap "evaluator had to reverse-engineer what focusa is FOR".
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MOD="$ROOT_DIR/crates/focusa-cli/src/commands/about.rs"
MODS_RS="$ROOT_DIR/crates/focusa-cli/src/commands/mod.rs"
MAIN_RS="$ROOT_DIR/crates/focusa-cli/src/main.rs" 2>/dev/null || MAIN_RS="$ROOT_DIR/crates/focusa-cli/src/main.rs"

fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

[ -f "$MOD" ] || fail "about.rs not found at $MOD"
pass "crates/focusa-cli/src/commands/about.rs exists"

grep -q 'pub mod about' "$MODS_RS" \
  || fail "pub mod about missing from commands/mod.rs"
pass "commands/mod.rs exposes about module"

# Main.rs registers About as top-level command
grep -q 'Commands::About' "$MAIN_RS" \
  || fail "main.rs not registering Commands::About"
grep -q 'About,' "$MAIN_RS" \
  || fail "main.rs About variant missing"
pass "main.rs registers About top-level command"

# Card has all 6 transcript-driven sections
for section in "Core concepts" "Try next" "Recover" "When to use" "When not" "anti.patterns"; do
  grep -qiE "$section" "$MOD" \
    || fail "about.rs missing section reference: $section"
done
pass "about.rs covers all 6 transcript-driven concept categories"

# Links to /llms.txt for LLM agents
grep -q '/llms.txt' "$MOD" \
  || fail "about.rs must cross-reference /llms.txt for LLM agents"
pass "about.rs cross-references /llms.txt for LLM agents"

echo "✓ All focusa-112-about-command static checks passed"