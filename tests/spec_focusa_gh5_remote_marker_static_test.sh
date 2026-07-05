#!/usr/bin/env bash
# Static/functional guard for GH #5 remote marker onboarding.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ONBOARD="$ROOT_DIR/crates/focusa-cli/src/commands/onboard.rs"
fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }
for needle in \
  'pub remote: Option<String>' \
  'ensure_project_marker' \
  'slug_from_remote' \
  'title_from_slug' \
  'detect_workspace_kind' \
  '.focusa-project.json' \
  'refusing to overwrite' \
  'repo_remote' \
  'Project marker:'; do
  grep -nF "$needle" "$ONBOARD" >/dev/null || fail "onboard remote marker missing: $needle"
done
pass "onboard exposes low-risk --remote marker creation path"
if grep -nE '/home/focusadev/perpetua|/home/wirebot/focusa' "$ONBOARD" >/dev/null; then
  fail "remote marker onboarding contains project-specific hardcoded root"
fi
pass "remote marker onboarding avoids hardcoded project roots"
python3 - <<'PY'
from pathlib import Path
text = Path('crates/focusa-cli/src/commands/onboard.rs').read_text()
assert 'Remote git URL to record in a local `.focusa-project.json` marker' in text
assert '"schema": "focusa.project.v1"' in text
assert '"created_at"' in text
PY
pass "remote marker schema fields are statically present"
echo "GH5/remote marker static test: PASS"
