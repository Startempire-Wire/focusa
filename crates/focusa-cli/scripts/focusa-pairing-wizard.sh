#!/usr/bin/env bash
# ============================================================================
# focusa pairing wizard — interactive VPS-side pairing flow (spec focusa-ui0y)
# Renders a terminal QR for the operator's phone camera to scan.
# ============================================================================
set -euo pipefail

DAEMON_URL="${FOCUSA_DAEMON_URL:-http://127.0.0.1:8787}"
TAILSCALE_HOSTNAME_HINT="${FOCUSA_TS_HOSTNAME:-focusa-vps}"
WIZARD_VERSION="0.9.34-dev"

step() { printf '\033[1;36m▶\033[0m  %s\n' "$*"; }
ok()   { printf '\033[1;32m✓\033[0m  %s\n' "$*"; }
warn() { printf '\033[1;33m!\033[0m  %s\n' "$*"; }
err()  { printf '\033[1;31m✗\033[0m  %s\n' "$*" >&2; }
ask() {
  local prompt="$1" default="$2" reply
  read -r -p "$(printf '\033[1;35m?\033[0m  %s [%s] ' "$prompt" "$default")" reply
  printf '%s' "${reply:-$default}"
}
pause() { read -r -p "$(printf '\033[2m  %s\033[0m' "$1")" _; }

render_terminal_qr() {
  python3 - "$1" <<'PY'
import sys, qrcode
url = sys.argv[1]
qr = qrcode.QRCode(
    version=None,
    error_correction=qrcode.constants.ERROR_CORRECT_L,
    box_size=1,
    border=1,
)
qr.add_data(url)
qr.make(fit=True)
matrix = qr.get_matrix()
# Use Unicode upper/lower block + space for higher density.
for r in matrix:
    line = ''.join('██' if c else '  ' for c in r)
    print(line)
PY
}

wizard_banner() {
  cat <<'EOF'

  ╔══════════════════════════════════════════════════════════╗
  ║          Focusa Pairing Wizard                           ║
  ║          focusa-pairing-wizard v0.9.34-dev               ║
  ╚══════════════════════════════════════════════════════════╝

EOF
}

detect_tailscale_hostname() {
  if command -v tailscale >/dev/null 2>&1; then
    tailscale status --json 2>/dev/null | python3 -c "
import json, sys
try:
    d = json.load(sys.stdin)
    name = d.get('Self', {}).get('DNSName', '').rstrip('.')
    if name:
        print(name + '|' + d.get('TailscaleIPs', ['?'])[0])
except Exception:
    pass
" 2>/dev/null || true
  fi
}

wizard_main() {
  wizard_banner

  step "Welcome to Focusa pairing."

  # 1. Daemon check
  if curl -fsS -m 3 "$DAEMON_URL/v1/health" >/tmp/.focusa-health.json 2>/dev/null; then
    local daemon_ver
    daemon_ver="$(python3 -c "import json; print(json.load(open('/tmp/.focusa-health.json')).get('version','?'))")"
    ok "Focusa daemon detected (${daemon_ver}) at ${DAEMON_URL}"
  else
    err "Cannot reach Focusa daemon at $DAEMON_URL"
    err "  Run: systemctl --user status focusa-daemon"
    err "  Or:  $DAEMON_URL is wrong; set FOCUSA_DAEMON_URL"
    exit 1
  fi

  # 2. Tailscale detection
  echo
  step "Resolving phone-reachable URL…"
  local ts_info hostname ts_ip public_url
  ts_info="$(detect_tailscale_hostname || true)"
  if [[ -n "$ts_info" ]]; then
    hostname="${ts_info%|*}"
    ts_ip="${ts_info#*|}"
    public_url="https://${hostname}"
    ok "Tailscale MagicDNS resolves: ${hostname} → ${ts_ip}"
  else
    warn "Tailscale not detected. Falling back to FOCUSA_PUBLIC_URL or 127.0.0.1."
    public_url="${FOCUSA_PUBLIC_URL:-${DAEMON_URL}}"
    hostname="${public_url#https://}"
    hostname="${hostname#http://}"
    hostname="${hostname%%/*}"
  fi
  printf '\033[1m   Pairing URL: %s\033[0m\n' "$public_url"
  echo

  # 3. Confirm pairing
  local proceed
  proceed="$(ask "Pair your Mac now?" "Y")"
  if [[ ! "$proceed" =~ ^[Yy]?$ ]]; then
    echo "  Skipped. Run 'focusa pairing wizard' any time."
    exit 0
  fi

  # 4. Create room
  echo
  step "Creating pairing room…"
  local room_resp room_id pair_url
  room_resp="$(curl -fsS -m 5 -X POST "$DAEMON_URL/v1/connect/room/firstrun" \
    -H 'content-type: application/json' \
    -d "$(printf '{"mac_name":"operator-mac","server_url":"%s"}' "$public_url")")"
  room_id="$(printf '%s' "$room_resp" | python3 -c 'import json,sys; print(json.load(sys.stdin)["room_id"])')"
  pair_url="$(printf '%s' "$room_resp" | python3 -c 'import json,sys; print(json.load(sys.stdin)["pair_url"])')"
  ok "Room ${room_id:0:8}…  expires in 5 min"
  echo

  # 5. Print terminal QR
  echo "  Scan this QR with your iPhone or Android camera:"
  echo
  render_terminal_qr "$pair_url" | sed 's/^/  /'
  echo
  printf '  \033[2mURL: %s\033[0m\n' "$pair_url"
  echo

  # 6. Poll for Mac join
  echo
  step "Waiting for Mac to join the room…"
  local saw_mac=0 saw_phone=0 saw_completed=0
  for i in {1..60}; do
    local status_resp status device_name
    status_resp="$(curl -fsS -m 3 "$DAEMON_URL/v1/connect/room/$room_id/status" 2>/dev/null || true)"
    status="$(printf '%s' "$status_resp" | python3 -c 'import json,sys; d=json.load(sys.stdin); print(d.get("status","?"))' 2>/dev/null || echo '?')"
    case "$status" in
      waiting_for_phone)
        if [[ $saw_mac -eq 0 ]]; then
          printf '\r\033[K  \033[2m[%02ds] waiting for phone to scan…\033[0m' "$i"
        else
          device_name="$(printf '%s' "$status_resp" | python3 -c 'import json,sys; d=json.load(sys.stdin); print(d.get("device_name") or "Mac")' 2>/dev/null || echo 'Mac')"
          printf '\r\033[K  \033[2m[%02ds] phone not yet approved %s…\033[0m' "$i" "$device_name"
        fi
        saw_mac=1
        ;;
      mac_seen)
        printf '\r\033[K  \033[2m[%02ds] phone opened Connect Page…\033[0m' "$i"
        saw_phone=1
        ;;
      completed)
        saw_completed=1
        printf '\r\033[K\033[1;32m  ✓ Phone approved. Token issued.\033[0m\n'
        break
        ;;
      *)
        printf '\r\033[K  \033[2m[%02ds] status=%s\033[0m' "$i" "$status"
        ;;
    esac
    sleep 1
  done
  echo

  # 7. Optional: simulate the phone-side actions for an end-to-end demo
  if [[ "${FOCUSA_WIZARD_DEMO:-0}" == "1" ]]; then
    echo "  \033[2m[FOCUSA_WIZARD_DEMO=1 — simulating phone-side approval]\033[0m"
    curl -fsS -m 3 -X POST "$DAEMON_URL/v1/connect/room/$room_id/mac-offer" \
      -H 'content-type: application/json' \
      -d '{"mac_name":"operator-mac"}' >/dev/null || true
    curl -fsS -m 3 -X POST "$DAEMON_URL/v1/connect/room/$room_id/approve" \
      -H 'content-type: application/json' \
      -d '{"host":"127.0.0.1","operator_id":"phone","completed_by":"phone"}' >/dev/null || true
    saw_completed=1
  fi

  if [[ $saw_completed -eq 1 ]]; then
    echo
    ok "Pairing complete."
    echo
    echo "  Next:"
    echo "    1. On your Mac: open /Applications/Focusa.app"
    echo "       (the wizard will detect this VPS and connect automatically)"
    echo "    2. Verify:      focusa doctor"
    echo
    exit 0
  else
    err "Timed out waiting for phone approval."
    err "  Recovery: re-run 'focusa pairing wizard' and scan a fresh QR."
    exit 1
  fi
}

wizard_main "$@"