#!/usr/bin/env bash
# spec_about_command_static_test.sh
# Static guard for focusa-112-about-command.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ABOUT="$ROOT_DIR/crates/focusa-cli/src/commands/about.rs"
MOD="$ROOT_DIR/crates/focusa-cli/src/commands/mod.rs"
MAIN="$ROOT_DIR/crates/focusa-cli/src/main.rs"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

[ -f "$ABOUT" ] || fail "missing about.rs"
grep -q 'pub mod about;' "$MOD" || fail "commands/mod.rs missing pub mod about"
grep -q 'Commands::About' "$MAIN" || fail "main.rs missing Commands::About dispatch"
grep -q 'commands::about::run(cli.json)' "$MAIN" || fail "main.rs missing about::run dispatch"
pass "about command is wired into CLI"

for marker in 'What this is' 'Core concepts' 'Try next' 'Recover' 'focusa doctor' 'focusa workpoint current'; do
  grep -q "$marker" "$ABOUT" || fail "about.rs missing marker: $marker"
done
pass "about card contains required sections and recovery commands"

for concept in Workpoint Trajectory 'Focus Stack' Memory Constitution; do
  grep -q "$concept" "$ABOUT" || fail "about.rs missing core concept: $concept"
done
pass "about card includes 5 core concepts"

grep -q 'serde_json::json!' "$ABOUT" || fail "about command missing --json output"
grep -q 'core_concepts' "$ABOUT" || fail "about --json missing core_concepts"
pass "about supports JSON mode"

echo "✓ All about command static checks passed"