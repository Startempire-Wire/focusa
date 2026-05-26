#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
WORKFLOW="${ROOT_DIR}/.github/workflows/release.yml"

fail() {
  echo "✗ FAIL: $*" >&2
  exit 1
}

pass() {
  echo "✓ PASS: $*"
}

rg -n 'Functional Dogfood Release|What this release proves|Commits since previous tag|Release workflow plus attached assets|macOS artifacts' "$WORKFLOW" >/dev/null \
  || fail "release workflow notes missing current functional dogfood proof language"
pass "release workflow notes use current functional dogfood proof language"

if rg -n '38 routes|96 unit tests|Cognitive Governance Framework|Full spec: 67|~13,000 LOC|14 command domains' "$WORKFLOW" >/tmp/focusa-release-notes-stale.txt; then
  cat /tmp/focusa-release-notes-stale.txt >&2
  fail "release workflow notes contain stale fixed-count/generic claims"
fi
pass "release workflow notes avoid stale fixed-count/generic claims"

rg -n 'COMMITS=\$\(git log --oneline.*\$\{PREV_TAG\}\.\.\$\{TAG\}|\$\{COMMITS\}' "$WORKFLOW" >/dev/null \
  || fail "release workflow notes missing tag-delta commit section"
pass "release workflow notes include tag-delta commit section"

echo "Release notes workflow static test: PASS"
