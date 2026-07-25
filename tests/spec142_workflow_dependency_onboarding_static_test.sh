#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
AUTO="$ROOT_DIR/apps/pi-extension/src/auto-compaction.ts"
TURNS="$ROOT_DIR/apps/pi-extension/src/turns.ts"
STATE="$ROOT_DIR/apps/pi-extension/src/state.ts"
INSTALL_RS="$ROOT_DIR/crates/focusa-cli/src/commands/install.rs"
BOOTSTRAP="$ROOT_DIR/scripts/install-focusa.sh"
SPEC_GATES="$ROOT_DIR/scripts/ci/run-spec-gates.sh"
SPEC="$ROOT_DIR/docs/142-focusa-seamless-pi-continuation-and-workflow-dependency-onboarding-spec.md"

fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

rg -n 'input_passthrough_native_overflow_recovery|return \{ action: "continue" as const \}' "$AUTO" >/dev/null \
  || fail "high-pressure Pi input does not preserve native prompt flow"
if rg -n 'Focusa preserved this prompt instead of sending another over-limit model request|Run /focusa-rollover execute; then resend' "$AUTO" >/dev/null; then
  fail "manual rollover/resend blocker remains in normal input path"
fi
pass "high-pressure prompts remain owned by Pi native compaction/retry"

python3 - "$TURNS" <<'PY'
from pathlib import Path
import sys
s=Path(sys.argv[1]).read_text()
for name in ("before_agent_start", "context", "input"):
    marker=f'pi.on("{name}"'
    start=s.index(marker)
    end=s.find('\n  pi.on(', start+len(marker))
    body=s[start:] if end < 0 else s[start:end]
    assert 'await ' not in body, f'{name} contains awaited prompt-path work'
PY
pass "prompt-critical hooks contain no awaited daemon work"

rg -n 'NON_PROJECT_ARTIFACT_SUFFIXES|isPlausibleProjectAlias|\^\\d\+\(\?:\\\.\\d\+\)\+\$' "$STATE" >/dev/null \
  || fail "numeric/artifact project-alias rejection missing"
rg -n 'currentAskDeclaresProjectScope|durable_project_write_authority' "$STATE" >/dev/null \
  || fail "diagnostic-path filtering or durable-write authority split missing"
pass "pressure numbers, artifacts, and diagnostic paths cannot override operator flow"

if rg -n 'sendUserMessage\("/focusa-rollover execute"|rollover_auto_queued' "$AUTO" >/dev/null; then
  fail "transport failure still auto-queues session rollover"
fi
pass "rollover remains explicit and outside ordinary prompt recovery"

for marker in \
  '@earendil-works/pi-coding-agent@0.81.1' \
  '"node",' \
  '"npm",' \
  '"pi",' \
  '"uiai-engine",' \
  'engine-vw20-multipool-20260705-2119' \
  '963883a19eec91c81ee88bc70c23e8db77f0cc12c673be872f6ee3bda3bba5b5' \
  'input_passthrough_native_overflow_recovery'; do
  rg -n -F "$marker" "$INSTALL_RS" "$AUTO" >/dev/null || fail "missing pinned workflow marker: $marker"
done
rg -n 'dependency_report = build_preflight_report|dependency commands completed but required tools are still unavailable|agent workflow readiness' "$INSTALL_RS" >/dev/null \
  || fail "normal install does not enforce verified dependency readiness"
pass "Rust installer owns ordered Node/npm/Pi/UIAI readiness"

rg -n 'INSTALL_DEPENDENCIES="\$\{INSTALL_DEPENDENCIES:-1\}"|--install-dependencies|--assume-yes|/dev/tty|ARGS\+=\(--install-dependencies\)' "$BOOTSTRAP" >/dev/null \
  || fail "public bootstrap dependency consent/forwarding contract missing"
pass "public bootstrap offers full workflow dependencies with explicit consent"

rg -n 'EXPECTED_OWNER|find_owner_drift|fix-user-perms|workspace ownership drift' "$SPEC_GATES" >/dev/null \
  || fail "release ownership preflight/repair contract missing"
pass "release gate detects and repairs managed checkout ownership drift"

rg -n 'UIAI platform matrix|Node\.js|Focusa Pi extension|No installer may silently use an unverified|Ownership safety and automatic repair' "$SPEC" >/dev/null \
  || fail "Spec142 dependency/verification contract incomplete"
pass "Spec142 documents acceptance, platform fallback, and trust boundary"

echo "Spec142 seamless Pi continuation and workflow dependency onboarding static test: PASS"
