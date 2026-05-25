#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
MANIFEST="$ROOT_DIR/Cargo.toml"
TUI_MANIFEST="$ROOT_DIR/crates/focusa-tui/Cargo.toml"
TOOLS="$ROOT_DIR/apps/pi-extension/src/tools.ts"

fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

if rg -n 'tokio = \{ version = "1", features = \["full"\]' "$MANIFEST" "$TUI_MANIFEST" >/dev/null; then
  fail "tokio full feature set is enabled on hot-path manifests"
fi
pass "Tokio uses narrow feature set, not full"

rg -n 'reqwest = \{ version = "0\.12", default-features = false, features = \["json", "rustls-tls"\] \}' "$MANIFEST" >/dev/null || fail "workspace reqwest must disable default features and avoid blocking/stream by default"
rg -n 'reqwest = \{ workspace = true, features = \["stream"\] \}' "$ROOT_DIR/crates/focusa-api/Cargo.toml" >/dev/null || fail "Only focusa-api should opt into reqwest stream for proxy streaming"
pass "Reqwest disables defaults globally; stream is API-only"

rg -n 'tokio = \{ workspace = true \}|reqwest = \{ workspace = true \}' "$TUI_MANIFEST" >/dev/null || fail "TUI should share optimized workspace dependency policy"
pass "TUI uses workspace dependency policy"

rg -n 'terseToolText|timeoutPreservedText' "$TOOLS" >/dev/null || fail "Pi tool degradation messages lack centralized terse helpers"
pass "Pi degradation messages use terse helpers"

rg -n 'compactApiEcho|params\.summary\.slice\(0, 240\)|next_tools\?\.length.*slice\(0, 4\)' "$TOOLS" >/dev/null || fail "Pi tool envelopes must cap summaries, next tools, and raw API echoes"
pass "Pi tool envelopes cap token-heavy fields"

rg -n 'timeoutBudgetForRoute|compactFallbackPacket' "$TOOLS" >/dev/null || fail "Pi tools must use route-tier timeout budgets and compact fallback packets"
pass "Pi tools use route-tier timeout budgets and compact fallback packets"

rg -n 'capToolText|capToolOutputText' "$TOOLS" >/dev/null || fail "Pi tools must cap model-visible text output globally"
pass "Pi tools cap model-visible text output globally"

if rg -n 'response: (res\.body|body|b)|request: payload|fallback_packet: fallback|resume_packet_v2: v2|rendered_summary: res\.body|health: health\.body|workpoint: workpoint\.body|work_loop: loop\.body' "$TOOLS" >/dev/null; then
  fail "Pi tool details contain raw unbounded response/request/diagnostic echoes"
fi
pass "Pi tool details avoid raw unbounded response/request/diagnostic echoes"

echo "SPEC82 low-resource efficiency static test: PASS"
