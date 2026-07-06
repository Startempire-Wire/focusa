#!/usr/bin/env bash
# Live proof guard for gaps 4-9: ensures the captured tmux + daemon evidence exists.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
EVIDENCE="$ROOT_DIR/docs/evidence/freshop-q-gaps-4-9"
fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

[[ -d "$EVIDENCE" ]] || fail "evidence directory missing: $EVIDENCE"
[[ -s "$EVIDENCE/00-tmux-transcript.txt" ]] || fail "tmux transcript missing"
[[ -s "$EVIDENCE/02-focusa-daemon.log" ]] || fail "focusa-daemon log missing"

python3 - <<PY
import json, urllib.request
# /v1/about is reachable on live local daemon.
with urllib.request.urlopen("http://127.0.0.1:18787/v1/about", timeout=3) as r:
    body = json.loads(r.read().decode())
for k in ("schema", "project", "version", "quickstart", "interactive_first_run", "next_commands"):
    assert k in body, f"missing {k}: {body}"
assert body["schema"] == "focusa.about.v1", body
assert body["project"] == "Focusa", body
assert any("init" in c["commands"][2] for c in [body["quickstart"]]) or "focusa init --quickstart" in body["quickstart"]["commands"][-1], body
PY
pass "/v1/about live probe returns canonical schema with quickstart + next_commands"

# focusa CLI: wordmark on --help and about.
out=$(/home/wirebot/focusa/target/release/focusa about 2>&1 | head -8)
echo "$out" | grep -q "cognitive governance runtime" || fail "focusa about missing wordmark tagline"
echo "$out" | grep -q "Focusa turns long AI chat" || fail "focusa about missing one-liner"
pass "focusa CLI rendered wordmark + tagline + one-line live"

# focusa init --quickstart --dry-run
js=$(/home/wirebot/focusa/target/release/focusa init --quickstart --dry-run 2>&1)
echo "$js" | python3 -c "
import json, sys
d = json.loads(sys.stdin.read())
assert d['schema'] == 'focusa.init.v1', d
assert d['mode'] == 'dry_run', d
assert d['marker_preview']['schema'] == 'focusa.project.v1', d
" || fail "focusa init --quickstart output shape mismatch"
pass "focusa init --quickstart --dry-run schema verified"

echo "focusa gaps 4-9 live proof guard: PASS"