#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
WORKFLOW="${ROOT_DIR}/.github/workflows/release.yml"
STALE_OUTPUT="$(mktemp /tmp/focusa-release-notes-stale.XXXXXX)"
trap 'rm -f "$STALE_OUTPUT"' EXIT

fail() {
  echo "✗ FAIL: $*" >&2
  exit 1
}

pass() {
  echo "✓ PASS: $*"
}

rg -n 'Functional Dogfood Release|What this release proves|Complete commit audit|Release workflow plus attached assets|macOS artifacts' "$WORKFLOW" >/dev/null \
  || fail "release workflow notes missing current functional dogfood proof language"
pass "release workflow notes use current functional dogfood proof language"

if rg -n '38 routes|96 unit tests|Cognitive Governance Framework|Full spec: 67|~13,000 LOC|14 command domains' "$WORKFLOW" >"$STALE_OUTPUT"; then
  cat "$STALE_OUTPUT" >&2
  fail "release workflow notes contain stale fixed-count/generic claims"
fi
pass "release workflow notes avoid stale fixed-count/generic claims"

for marker in \
  '### Features added' \
  '### Fixes shipped' \
  '### Issues resolved' \
  '### Other changes' \
  '### Full changelog' \
  '### Complete commit audit' \
  'gh issue list' \
  'closedAt > $previous'; do
  rg -F "$marker" "$WORKFLOW" >/dev/null || fail "release workflow notes missing durable changelog marker: $marker"
done
pass "release workflow notes include features, fixes, resolved issues, compare link, and complete commit audit"

if rg -n 'git log .*head -[0-9]+' "$WORKFLOW" >/dev/null; then
  fail "release workflow truncates commit history"
fi
pass "release workflow does not truncate tag-delta commit history"

python3 "${ROOT_DIR}/tests/release_notes_preview_test.py"

echo "Release notes workflow static test: PASS"
