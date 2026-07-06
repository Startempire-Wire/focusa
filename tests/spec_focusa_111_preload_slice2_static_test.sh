#!/usr/bin/env bash
# Spec 111 Slice 2 — core packet types and renderers static guard.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

PRE="$ROOT_DIR/crates/focusa-api/src/routes/preload.rs"
[[ -f "$PRE" ]] || fail "preload.rs missing"

for needle in \
  'pub enum RenderMode' \
  'pub struct AgentBootstrapProfile' \
  'pub const AGENT_BOOTSTRAP_PROFILES' \
  'pub fn profile_by_id' \
  'pub struct AgentBootstrapPacket' \
  'pub fn build_packet' \
  'pub fn render_packet' \
  'static_rule_lines' \
  'dynamic_context_lines' \
  'acceptance_prompt' \
  'bounded_dynamic_items' \
  'StaticRule' \
  'DynamicContext' \
  'AcceptancePrompt' \
  'includes_dynamic_context' \
  'includes_acceptance_prompt' \
  'max_dynamic_items' \
  'Focusa does not bypass install' \
  'Canonical Workpoint authority requires operator approval' \
  'Acknowledge Focusa rules'; do
  grep -qF -- "$needle" "$PRE" || fail "preload slice 2 missing: $needle"
done
pass "preload slice 2 covers profiles, packet, render, and bounded context"

echo "focusa-111 preload slice2 static test: PASS"
