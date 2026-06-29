#!/usr/bin/env bash
# V2 P1.4 nonce mismatch rejection.
#
# Verifies that /v1/connect/room/{id}/mac-offer refuses to overwrite an
# already-bound mac_nonce with a different one. Without this guard, an
# attacker who intercepts the PWA tab could swap in their own nonce and
# redirect the mac_callback fast path to a URL they control.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

LOG_DIR="${TMPDIR:-/tmp}/focusa-nonce-test-$$"
mkdir -p "$LOG_DIR"
trap 'rm -rf "$LOG_DIR"' EXIT

DAEMON_BIN="$ROOT_DIR/target/debug/focusa-daemon"
[ -x "$DAEMON_BIN" ] || {
  echo "✗ FAIL: $DAEMON_BIN not built"
  exit 1
}

PORT=18976
DATA_DIR="$LOG_DIR/data"
mkdir -p "$DATA_DIR"

export FOCUSA_BIND="127.0.0.1:$PORT"
export FOCUSA_DATA_DIR="$DATA_DIR"
nohup "$DAEMON_BIN" >"$LOG_DIR/daemon.log" 2>&1 &
DAEMON_PID=$!
trap 'rm -rf "$LOG_DIR"; kill $DAEMON_PID 2>/dev/null || true' EXIT

for _ in $(seq 1 20); do
  curl -fs -m 1 "http://127.0.0.1:$PORT/v1/health" >/dev/null 2>&1 && break
  sleep 0.5
done

R=$(curl -fs -m 5 -X POST "http://127.0.0.1:$PORT/v1/connect/room/create" \
  -H 'Content-Type: application/json' \
  -d "{\"server_url\":\"http://127.0.0.1:$PORT\"}" \
  | python3 -c 'import sys,json; print(json.load(sys.stdin)["connect_id"])')
echo "  room=$R"

# First mac-offer with nonce A: should succeed (status=mac_seen)
J1=$(curl -fs -m 5 -X POST "http://127.0.0.1:$PORT/v1/connect/room/$R/mac-offer" \
  -H 'Content-Type: application/json' \
  -d '{"mac_name":"op-mac","mac_nonce":"nonce-A-original","mac_pubkey":null}')
S1=$(echo "$J1" | python3 -c 'import sys,json; print(json.load(sys.stdin)["status"])')
echo "  first mac-offer (nonce A): status=$S1"

# Second mac-offer with different nonce B: must be 409 nonce_mismatch
HTTP=$(curl -s -o /tmp/mac_offer2.json -w "%{http_code}" -m 5 \
  -X POST "http://127.0.0.1:$PORT/v1/connect/room/$R/mac-offer" \
  -H 'Content-Type: application/json' \
  -d '{"mac_name":"attacker-mac","mac_nonce":"nonce-B-hostile","mac_pubkey":null}')
echo "  second mac-offer (nonce B): HTTP=$HTTP"
echo "  body: $(cat /tmp/mac_offer2.json)"
FAIL_CLASS=$(python3 -c "import json; print(json.load(open('/tmp/mac_offer2.json')).get('failure_class',''))")
[ "$HTTP" = "409" ] && [ "$FAIL_CLASS" = "mac_nonce_mismatch" ] \
  && echo "  ✓ V2 P1.4 verified: nonce mismatch rejected with 409 mac_nonce_mismatch" \
  || { echo "  ✗ FAIL: expected 409 mac_nonce_mismatch, got HTTP=$HTTP failure_class=$FAIL_CLASS"; exit 1; }

# Third mac-offer with the SAME nonce A: should succeed (idempotent retry)
HTTP2=$(curl -s -o /tmp/mac_offer3.json -w "%{http_code}" -m 5 \
  -X POST "http://127.0.0.1:$PORT/v1/connect/room/$R/mac-offer" \
  -H 'Content-Type: application/json' \
  -d '{"mac_name":"op-mac","mac_nonce":"nonce-A-original","mac_pubkey":null}')
S3=$(python3 -c "import json; print(json.load(open('/tmp/mac_offer3.json')).get('status',''))")
echo "  same-nonce retry (nonce A again): HTTP=$HTTP2 status=$S3"
[ "$HTTP2" = "200" ] && [ "$S3" = "mac_seen" ] \
  && echo "  ✓ idempotent retry with same nonce accepted" \
  || { echo "  ✗ FAIL: same-nonce retry should succeed"; exit 1; }

echo
echo "✓ ALL V2 P1.4 NONCE MISMATCH ASSERTIONS PASSED"