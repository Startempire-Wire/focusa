#!/usr/bin/env bash
# V2 P0 round 2: room_claim_secret enforcement + persistence restart proof.
#
# Verifies that /v1/connect/room/{id}/join requires room_claim_secret
# when the room was created by /v1/connect/room/create, and that the
# secret survives daemon restart (via SQLite ledger rehydration).
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

LOG_DIR="${TMPDIR:-/tmp}/focusa-claim-test-$$"
mkdir -p "$LOG_DIR"
DAEMON_BIN="$ROOT_DIR/target/debug/focusa-daemon"
[ -x "$DAEMON_BIN" ] || { echo "✗ FAIL: $DAEMON_BIN not built"; exit 1; }

PORT=18985
DATA_DIR="$LOG_DIR/data"
mkdir -p "$DATA_DIR"

start_daemon() {
    local data_dir="$1"
    # First-call flag preserved via env: if no existing SQLite is in
    # the data dir, this is the fresh-create. Reuses the existing
    # data_dir on restart so the SQLite ledger survives.
    mkdir -p "$data_dir"
    FOCUSA_BIND="127.0.0.1:$PORT" FOCUSA_DATA_DIR="$data_dir" \
        nohup "$DAEMON_BIN" >"$LOG_DIR/daemon.log" 2>&1 &
    DAEMON_PID=$!
    for _ in $(seq 1 20); do
        curl -fs -m 1 "http://127.0.0.1:$PORT/v1/health" >/dev/null 2>&1 && return 0
        sleep 0.5
    done
    return 1
}

stop_daemon() {
    if [ -n "${DAEMON_PID:-}" ]; then
        kill "$DAEMON_PID" 2>/dev/null || true
        wait "$DAEMON_PID" 2>/dev/null || true
    fi
    # Also free the port in case anything else is bound
    fuser -k "$PORT/tcp" 2>/dev/null || true
    sleep 1
}

trap 'stop_daemon; rm -rf "$LOG_DIR"' EXIT

start_daemon "$DATA_DIR" || { echo "✗ FAIL: daemon failed to start"; exit 1; }

echo "=== test 1: room_create returns 43-char room_claim_secret ==="
J=$(curl -fs -m 5 -X POST "http://127.0.0.1:$PORT/v1/connect/room/create" \
    -H 'Content-Type: application/json' \
    -d "{\"server_url\":\"http://127.0.0.1:$PORT\"}")
RID=$(echo "$J" | python3 -c 'import sys,json; print(json.load(sys.stdin)["room_id"])')
SECRET=$(echo "$J" | python3 -c 'import sys,json; print(json.load(sys.stdin)["room_claim_secret"])')
SECRET_LEN=${#SECRET}
echo "  room=$RID secret_len=$SECRET_LEN"
[ "$SECRET_LEN" = "43" ] || { echo "✗ FAIL: expected 43-char secret, got $SECRET_LEN"; exit 1; }
echo "  ✓ secret generated"

echo ""
echo "=== test 2: /join without secret -> 401 room_claim_secret_missing ==="
HTTP=$(curl -s -o /tmp/claim-1.json -w "%{http_code}" -m 5 \
    -X POST "http://127.0.0.1:$PORT/v1/connect/room/$RID/join" \
    -H 'Content-Type: application/json' \
    -d '{"mac_name":"op","mac_nonce":"n1","mac_pubkey":null,"mac_callback":""}')
FC=$(python3 -c 'import json; print(json.load(open("/tmp/claim-1.json")).get("failure_class",""))')
echo "  HTTP=$HTTP failure_class=$FC"
[ "$HTTP" = "401" ] && [ "$FC" = "room_claim_secret_missing" ] \
    || { echo "✗ FAIL: expected 401 room_claim_secret_missing"; exit 1; }
echo "  ✓ missing secret rejected"

echo ""
echo "=== test 3: /join with WRONG secret -> 403 room_claim_secret_mismatch ==="
HTTP=$(curl -s -o /tmp/claim-2.json -w "%{http_code}" -m 5 \
    -X POST "http://127.0.0.1:$PORT/v1/connect/room/$RID/join" \
    -H 'Content-Type: application/json' \
    -d '{"mac_name":"op","mac_nonce":"n1","mac_pubkey":null,"mac_callback":"","room_claim_secret":"wrong-secret-12345"}')
FC=$(python3 -c 'import json; print(json.load(open("/tmp/claim-2.json")).get("failure_class",""))')
echo "  HTTP=$HTTP failure_class=$FC"
[ "$HTTP" = "403" ] && [ "$FC" = "room_claim_secret_mismatch" ] \
    || { echo "✗ FAIL: expected 403 room_claim_secret_mismatch"; exit 1; }
echo "  ✓ wrong secret rejected"

echo ""
echo "=== test 4: /join with CORRECT secret -> 200 mac_seen ==="
HTTP=$(curl -s -o /tmp/claim-3.json -w "%{http_code}" -m 5 \
    -X POST "http://127.0.0.1:$PORT/v1/connect/room/$RID/join" \
    -H 'Content-Type: application/json' \
    -d "{\"mac_name\":\"op\",\"mac_nonce\":\"n1\",\"mac_pubkey\":null,\"mac_callback\":\"\",\"room_claim_secret\":\"$SECRET\"}")
S=$(python3 -c 'import json; print(json.load(open("/tmp/claim-3.json")).get("status",""))')
echo "  HTTP=$HTTP status=$S"
[ "$HTTP" = "200" ] && [ "$S" = "mac_seen" ] \
    || { echo "✗ FAIL: expected 200 mac_seen"; exit 1; }
echo "  ✓ correct secret accepted"

echo ""
echo "=== test 5: persistence restart proof — restart daemon, secret still required ==="
stop_daemon
start_daemon "$DATA_DIR" || { echo "✗ FAIL: daemon failed to restart"; exit 1; }
HTTP=$(curl -s -o /tmp/claim-4.json -w "%{http_code}" -m 5 \
    -X POST "http://127.0.0.1:$PORT/v1/connect/room/$RID/join" \
    -H 'Content-Type: application/json' \
    -d '{"mac_name":"attacker","mac_nonce":"new","mac_pubkey":null,"mac_callback":""}')
FC=$(python3 -c 'import json; print(json.load(open("/tmp/claim-4.json")).get("failure_class",""))')
echo "  HTTP=$HTTP failure_class=$FC (after restart)"
[ "$HTTP" = "401" ] && [ "$FC" = "room_claim_secret_missing" ] \
    || { echo "✗ FAIL: secret did not survive restart"; exit 1; }
echo "  ✓ secret persisted across restart"

echo ""
echo "=== test 6: post-restart join with correct secret still works ==="
HTTP=$(curl -s -o /tmp/claim-5.json -w "%{http_code}" -m 5 \
    -X POST "http://127.0.0.1:$PORT/v1/connect/room/$RID/join" \
    -H 'Content-Type: application/json' \
    -d "{\"mac_name\":\"op\",\"mac_nonce\":\"n1\",\"mac_pubkey\":null,\"mac_callback\":\"\",\"room_claim_secret\":\"$SECRET\"}")
S=$(python3 -c 'import json; print(json.load(open("/tmp/claim-5.json")).get("status",""))')
echo "  HTTP=$HTTP status=$S"
[ "$HTTP" = "200" ] && [ "$S" = "mac_seen" ] \
    || { echo "✗ FAIL: post-restart join failed"; exit 1; }
echo "  ✓ post-restart join still works with persisted secret"

echo ""
echo "✓ ALL V2 P0 ROOM_CLAIM_SECRET ASSERTIONS PASSED"