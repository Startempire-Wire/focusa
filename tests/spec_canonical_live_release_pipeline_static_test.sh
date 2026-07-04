#!/usr/bin/env bash
# Guard the operator directive: build/deploy ONLY via full live release pipeline.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
POLICY="$ROOT_DIR/docs/canonical-live-release-pipeline.md"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

[ -f "$POLICY" ] || fail "missing canonical live release policy doc"
pass "canonical live release policy doc exists"

for file in "$ROOT_DIR/AGENTS.md" "$ROOT_DIR/docs/live-release-automation.md" "$ROOT_DIR/docs/production-deployment-guide.md" "$ROOT_DIR/docs/deploy-runbook.md"; do
  grep -q 'canonical-live-release-pipeline.md' "$file" \
    || fail "$file must link canonical live release policy"
done
pass "agent/deploy docs link canonical live release policy"

for marker in \
  'scripts/create-dev-release-tag.sh --base 0.9 --push' \
  'CI' \
  'Release' \
  'Deploy Live Daemon' \
  'Auto Heal Release Pipeline' \
  'Release Pipeline Watchdog'; do
  grep -q "$marker" "$POLICY" \
    || fail "policy missing required marker: $marker"
done
pass "policy names full live pipeline chain"

# These docs must not instruct agents to deploy local artifacts or invoke only deploy workflow.
python3 - "$ROOT_DIR" <<'PY'
from pathlib import Path
import re, sys
root = Path(sys.argv[1])
files = [root/'AGENTS.md', root/'docs/live-release-automation.md', root/'docs/production-deployment-guide.md', root/'docs/deploy-runbook.md']
patterns = [
    'cargo build --release',
    'target/release/focusa-daemon',
    'install-daemon.sh --binary target/release',
    "gh workflow run 'Deploy Live Daemon'",
]
allow_words = ('do not', 'forbidden', 'not allowed', 'must not', 'never')
for file in files:
    for idx, line in enumerate(file.read_text().splitlines(), 1):
        lower = re.sub(r'[*_`]', '', line.lower())
        if any(p in line for p in patterns) and not any(w in lower for w in allow_words):
            raise SystemExit(f"{file}:{idx} contains actionable forbidden deploy instruction: {line}")
PY
pass "agent/deploy docs contain no actionable local build or partial deploy instructions"

# The canonical policy may mention forbidden commands only in its forbidden section.
grep -q 'Forbidden for release/deploy' "$POLICY" \
  || fail "policy must include explicit forbidden section"
grep -q 'Do not use these as release/deploy actions' "$POLICY" \
  || fail "policy must explain forbidden commands are examples, not instructions"
pass "policy explicitly forbids local build/partial deploy commands"

# Existing automation guards must include the watchdog and self-heal surfaces.
grep -q 'Release Pipeline Watchdog' "$ROOT_DIR/.github/workflows/release-pipeline-watchdog.yml" \
  || fail "watchdog workflow missing"
grep -q 'Auto Heal Release Pipeline' "$ROOT_DIR/.github/workflows/auto-retry-deploy.yml" \
  || fail "auto heal workflow missing"
pass "continuous auto-detect/auto-heal workflows exist"

echo "✓ Canonical live release pipeline static guard passed"