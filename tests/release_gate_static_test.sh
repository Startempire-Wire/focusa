#!/usr/bin/env bash
# Static guard for programmatic ReleaseGate: expensive release builds must be
# gated by significant delta, scheduled window, or explicit override reason.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
GATE="$ROOT_DIR/scripts/release-gate.py"
HELPER="$ROOT_DIR/scripts/create-dev-release-tag.sh"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

[[ -f "$GATE" ]] || fail "missing scripts/release-gate.py"
[[ -f "$HELPER" ]] || fail "missing scripts/create-dev-release-tag.sh"
python3 -m py_compile "$GATE" || fail "release-gate.py must compile"
pass "release-gate.py exists and compiles"

for token in \
  'SIGNIFICANT_SCORE = 8' \
  'RELEASE_WORKFLOW_SCORE = 8' \
  'WINDOW_SCORE = 4' \
  'STALE_HOURS = 24' \
  'DEFAULT_WINDOWS_PT = ("11:00", "16:00")' \
  'plain_language_error' \
  'Blocked: not enough significant app delta since last release' \
  'critical_security_install_signing_checksum' \
  'release_deploy_system' \
  'app_runtime_code' \
  'user_visible_ui' \
  'docs_only'; do
  grep -q "$token" "$GATE" || fail "release-gate missing token: $token"
done
pass "release-gate encodes weighted significant-delta policy and plain blocker"

python3 - "$GATE" <<'PY'
import importlib.util
import sys

spec = importlib.util.spec_from_file_location("release_gate", sys.argv[1])
module = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = module
spec.loader.exec_module(module)
scored = module.score_path(".github/workflows/release.yml")
assert scored.category == "release_deploy_system"
assert scored.score >= module.SIGNIFICANT_SCORE
PY
pass "canonical release workflow repairs independently meet the significant-delta gate"

grep -q 'python3 scripts/release-gate.py' "$HELPER" || fail "tag helper must invoke ReleaseGate"
grep -q -- '--force-release' "$HELPER" || fail "tag helper missing --force-release override"
grep -q -- '--release-reason' "$HELPER" || fail "tag helper missing --release-reason"
grep -q 'Blocked: --force-release requires --release-reason' "$HELPER" || fail "force override must require plain-language reason"
grep -q 'ReleaseGate override accepted' "$HELPER" || fail "override must be explicit/auditable"
pass "create-dev-release-tag enforces ReleaseGate with explicit override reason"

echo "✓ release_gate_static_test passed"
