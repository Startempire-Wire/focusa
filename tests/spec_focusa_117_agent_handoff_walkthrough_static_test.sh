#!/usr/bin/env bash
# Spec 117 §13.2 — Agent Handoff walkthrough static guard.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

WT="$ROOT_DIR/crates/focusa-cli/src/commands/walkthrough.rs"
[[ -f "$WT" ]] || fail "walkthrough.rs missing"

for needle in \
  "agent-handoff" \
  "pub fn agent_handoff" \
  "Show current mission" \
  "Show current Workpoint" \
  "Render the handoff packet" \
  "Show what a new agent receives" \
  "Show drift boundaries" \
  "Show evidence and proof expectations" \
  "do-not-drift boundaries" \
  "proof expectations" \
  "agent_handoff_round_trips"; do
  grep -qF -- "$needle" "$WT" || fail "agent handoff walkthrough missing: $needle"
done
pass "agent-handoff walkthrough covers Spec 117 §13.2 required steps"

grep -qF "vec![\"first-mission\", \"agent-handoff\"]" "$WT" || fail "catalog missing agent-handoff"
grep -qF '"agent-handoff" => agent_handoff().steps[0].id.clone()' "$WT" || fail "start action missing agent-handoff first step"
grep -qF '"agent-handoff" => Ok(serde_json::to_value(agent_handoff())?)' "$WT" || fail "render action missing agent-handoff"
pass "agent-handoff walkthrough wired into catalog/start/show"

echo "focusa-117 agent-handoff walkthrough static test: PASS"
