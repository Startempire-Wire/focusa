#!/usr/bin/env bash
# V2 P0.1/P0.2/P0.3/P0.4 hybrid proof:
# - scan page extracts room_claim_secret from location.hash
# - scan page scrubs #secret from the visible URL
# - scan page posts /mac-offer with room_claim_secret
# - /mac-offer rejects missing/wrong secret and accepts correct secret
# - FirstRunWizard no longer self-posts /join
# - approval receipt logic does not copy room_claim_secret
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

LOG_DIR="${TMPDIR:-/tmp}/focusa-secret-client-test-$$"
mkdir -p "$LOG_DIR"
DAEMON_BIN="$ROOT_DIR/target/debug/focusa-daemon"
[ -x "$DAEMON_BIN" ] || { echo "✗ FAIL: $DAEMON_BIN not built"; exit 1; }

PORT=$(python3 - <<'PY'
import socket
s = socket.socket()
s.bind(('127.0.0.1', 0))
print(s.getsockname()[1])
s.close()
PY
)
DATA_DIR="$LOG_DIR/data"
mkdir -p "$DATA_DIR"

start_daemon() {
  mkdir -p "$DATA_DIR"
  FOCUSA_BIND="127.0.0.1:$PORT" FOCUSA_DATA_DIR="$DATA_DIR" \
    nohup "$DAEMON_BIN" >"$LOG_DIR/daemon.log" 2>&1 &
  DAEMON_PID=$!
  for _ in $(seq 1 120); do
    if python3 - <<'PY' "$PORT"
import socket, sys
port = int(sys.argv[1])
s = socket.socket()
s.settimeout(0.2)
try:
    s.connect(('127.0.0.1', port))
except OSError:
    sys.exit(1)
finally:
    s.close()
PY
    then
      return 0
    fi
    sleep 0.25
  done
  echo "✗ FAIL: daemon failed to start"
  tail -50 "$LOG_DIR/daemon.log" || true
  exit 1
}

cleanup() {
  if [ -n "${DAEMON_PID:-}" ] && kill -0 "$DAEMON_PID" 2>/dev/null; then
    kill "$DAEMON_PID" 2>/dev/null || true
    wait "$DAEMON_PID" 2>/dev/null || true
  fi
}
trap cleanup EXIT

start_daemon

BASE="http://127.0.0.1:$PORT"
CREATE_JSON=$(curl -fsS -X POST "$BASE/v1/connect/room/create" -H 'content-type: application/json' -d '{}')
ROOM_ID=$(echo "$CREATE_JSON" | python3 -c 'import sys,json; print(json.load(sys.stdin)["room_id"])')
SECRET=$(echo "$CREATE_JSON" | python3 -c 'import sys,json; print(json.load(sys.stdin)["room_claim_secret"])')
PAIR_QR=$(echo "$CREATE_JSON" | python3 -c 'import sys,json; print(json.load(sys.stdin)["pair_url_qr_payload"])')

[ ${#SECRET} -eq 43 ] || { echo "✗ FAIL: expected 43-char secret"; exit 1; }
echo "$PAIR_QR" | grep -Fq '#secret=' || { echo "✗ FAIL: pair_url_qr_payload missing #secret"; exit 1; }
echo "✓ PASS: room_create returns secret-bearing QR payload"

PAGE=$(curl -fsS "$BASE/connect/room/$ROOM_ID/scan#secret=$SECRET")
printf '%s' "$PAGE" | grep -Fq "location.hash.replace(/^#/, '')" || { echo "✗ FAIL: scan page missing hash secret extraction"; exit 1; }
printf '%s' "$PAGE" | grep -Fq "history.replaceState(null, '', location.pathname + location.search)" || { echo "✗ FAIL: scan page missing URL hash scrub"; exit 1; }
printf '%s' "$PAGE" | grep -Fq "room_claim_secret" || { echo "✗ FAIL: scan page missing room_claim_secret field"; exit 1; }
printf '%s' "$PAGE" | grep -Fq "/mac-offer" || { echo "✗ FAIL: scan page not posting to /mac-offer"; exit 1; }
echo "✓ PASS: scan page source carries secret flow"

python3 - <<'PY' "$PAGE"
import sys
page = sys.argv[1]
assert "localStorage.setItem('room_claim_secret'" not in page, "room_claim_secret persisted to localStorage"
assert "sessionStorage.setItem('room_claim_secret'" not in page, "room_claim_secret persisted to sessionStorage"
assert "approval_receipt" not in page, "unexpected receipt flow found in scan page"
PY
echo "✓ PASS: scan page does not persist room_claim_secret in browser storage"

HTTP=$(curl -sS -o "$LOG_DIR/missing.json" -w '%{http_code}' -X POST \
  "$BASE/v1/connect/room/$ROOM_ID/mac-offer" \
  -H 'content-type: application/json' \
  -d '{"mac_name":"Mac","mac_nonce":"nonce-1"}')
FC=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["failure_class"])' "$LOG_DIR/missing.json")
[ "$HTTP" = "401" ] && [ "$FC" = "room_claim_secret_missing" ] || { echo "✗ FAIL: expected /mac-offer missing secret -> 401 room_claim_secret_missing"; cat "$LOG_DIR/missing.json"; exit 1; }
echo "✓ PASS: /mac-offer rejects missing secret"

HTTP=$(curl -sS -o "$LOG_DIR/wrong.json" -w '%{http_code}' -X POST \
  "$BASE/v1/connect/room/$ROOM_ID/mac-offer" \
  -H 'content-type: application/json' \
  -d '{"mac_name":"Mac","mac_nonce":"nonce-1","room_claim_secret":"wrong-secret"}')
FC=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["failure_class"])' "$LOG_DIR/wrong.json")
[ "$HTTP" = "403" ] && [ "$FC" = "room_claim_secret_mismatch" ] || { echo "✗ FAIL: expected /mac-offer wrong secret -> 403 room_claim_secret_mismatch"; cat "$LOG_DIR/wrong.json"; exit 1; }
echo "✓ PASS: /mac-offer rejects wrong secret"

HTTP=$(curl -sS -o "$LOG_DIR/correct.json" -w '%{http_code}' -X POST \
  "$BASE/v1/connect/room/$ROOM_ID/mac-offer" \
  -H 'content-type: application/json' \
  -d "{\"mac_name\":\"Mac\",\"mac_nonce\":\"nonce-1\",\"room_claim_secret\":\"$SECRET\"}")
STATUS=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["status"])' "$LOG_DIR/correct.json")
[ "$HTTP" = "200" ] && [ "$STATUS" = "mac_seen" ] || { echo "✗ FAIL: expected /mac-offer correct secret -> 200 mac_seen"; cat "$LOG_DIR/correct.json"; exit 1; }
echo "✓ PASS: /mac-offer accepts correct secret"

if rg -n "connect/room/.*/join|/v1/connect/room/.*/join|mac_offer_posted" apps/menubar/src/lib/components/FirstRunWizard.svelte >/dev/null 2>&1; then
  echo "✗ FAIL: FirstRunWizard still contains self-join flow"
  rg -n "connect/room/.*/join|/v1/connect/room/.*/join|mac_offer_posted" apps/menubar/src/lib/components/FirstRunWizard.svelte || true
  exit 1
fi

echo "✓ PASS: FirstRunWizard no longer self-joins canonical rooms"
echo "Phone Bridge secret client-flow hybrid test: PASS"
