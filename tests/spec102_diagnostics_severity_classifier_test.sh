#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="${ROOT_DIR:-/home/wirebot/focusa}"
cd "$ROOT_DIR"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }
FILE="apps/pi-extension/src/tools.ts"
for term in page_breaking workflow_blocking benign_asset unknown diagnostics_severity severityClassification "Benign asset diagnostics will not trigger an unnecessary repair loop"; do
  rg -F "$term" "$FILE" >/dev/null || fail "diagnostics severity classifier missing $term"
done
pass "Pi diagnostics intake declares severity classes"

rg -n 'severity=\$\{severityClassification\.severity\} alarm=\$\{severityClassification\.alarm\}' "$FILE" >/dev/null || fail "compact output missing severity/alarm"
pass "compact diagnostics intake output includes severity and alarm"

rg -n 'png\|jpe\?g\|gif\|webp\|svg\|ico\|css\|woff2\?|favicon|analytics|pixel|beacon|tracking' "$FILE" >/dev/null || fail "benign asset classifier lacks asset-pattern guard"
pass "benign asset failures are classified separately"

rg -n 'blank page|page crashed|navigation failed|main frame|document failed|hydration failed' "$FILE" >/dev/null || fail "page-breaking classifier lacks page failure patterns"
pass "page-breaking failures still route to repair evidence"

echo "SPEC102 diagnostics severity classifier test: PASS"
