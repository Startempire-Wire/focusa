#!/usr/bin/env bash
# Static guard for GH #7 / Spec 110 Pi unbound-project nag.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SESSION="$ROOT_DIR/apps/pi-extension/src/session.ts"
fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }
for needle in \
  'queueUnboundProjectNag' \
  'markerExistsAtCwd' \
  '--nag-suppress' \
  '.focusa-project.json' \
  'focusa about' \
  'focusa init' \
  'focusa onboard --remote <git-url> --project-root <path>' \
  'pi_unbound_project_nag' \
  'session_start'; do
  grep -nF -- "$needle" "$SESSION" >/dev/null || fail "Pi unbound nag missing marker: $needle"
done
pass "Pi startup nag has marker check, suppress flag, commands, and telemetry"
python3 - <<'PY'
from pathlib import Path
text = Path('apps/pi-extension/src/session.ts').read_text()
assert 'if (pi.getFlag("--nag-suppress")) return;' in text
assert 'if (markerExistsAtCwd(cwd)) return;' in text
assert 'getAttachmentRuntime().vitalInfoPrompted[key]' in text
assert 'queueUnboundProjectNag(pi, ctx, "new_session_new_project")' in text
assert text.index('sessionProjectClassification === "new_session_new_project"') < text.index('queueUnboundProjectNag(pi, ctx, "new_session_new_project")')
assert 'sendUserMessage(' not in text
assert 'deferLifecycleAdvisory(ctx, key, prompt, reason)' in text
assert 'pi_lifecycle_advisory_deferred_to_next_turn' in text
PY
pass "Pi startup nag suppresses when marker present or already emitted"
echo "GH7/Pi unbound nag static test: PASS"
