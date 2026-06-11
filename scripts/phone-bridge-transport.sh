#!/usr/bin/env bash
# Phone Bridge Transport Resolver.
#
# Focusa-contained helper for finding/validating a phone-reachable transport
# for the Phone Bridge Flow. It never mutates a live webserver by default.

set -euo pipefail

URL=""
DAEMON_URL="${FOCUSA_DAEMON_URL:-http://127.0.0.1:8787}"
CONFIG_PATH="${FOCUSA_PUBLIC_URL_FILE:-/etc/focusa/public-url}"
JSON=0
MODE="help"

usage() {
  cat <<'EOF'
Usage: scripts/phone-bridge-transport.sh <mode> [options]

Modes:
  detect            List candidate Phone Bridge transports.
  check --url URL   Validate a candidate transport.
  write --url URL   Write URL to /etc/focusa/public-url (or --config-path).
  options           Print adaptive setup options.
  proxy-snippets    Print optional reverse-proxy snippets.

Options:
  --url URL           Public/remote URL candidate.
  --daemon-url URL    Local daemon URL (default: http://127.0.0.1:8787).
  --config-path PATH  URL config file (default: /etc/focusa/public-url).
  --json             Machine-readable output where supported.
  -h, --help         Show help.

Resolution philosophy:
  Focusa tries configured URLs, non-local API URLs, verified hostname/IP/private
  routes, and optional operator-chosen tunnel/proxy paths. Live webserver
  mutation is not required and is never done implicitly.
EOF
}

normalize_url() { printf '%s' "$1" | sed 's:/*$::'; }

candidate_json() {
  local url="$1" source="$2" note="${3:-}"
  jq -n --arg url "$url" --arg source "$source" --arg note "$note" '{url:$url, source:$source, note:$note}'
}

host_is_public_v4() {
  python3 - "$1" <<'PY'
import ipaddress, sys
ip = ipaddress.ip_address(sys.argv[1])
print('1' if ip.version == 4 and ip.is_global else '0')
PY
}

host_is_private_or_tailscale_v4() {
  python3 - "$1" <<'PY'
import ipaddress, sys
ip = ipaddress.ip_address(sys.argv[1])
# Tailscale/CGNAT: 100.64.0.0/10. Loopback remains local-only fallback.
cg = ipaddress.ip_network('100.64.0.0/10')
print('1' if ip.version == 4 and not ip.is_loopback and (ip.is_private or ip in cg) else '0')
PY
}

emit_candidates_json() {
  local tmp
  tmp="$(mktemp)"
  : > "$tmp"

  for key in FOCUSA_PAIRING_URL FOCUSA_PUBLIC_URL FOCUSA_API_URL FOCUSA_BASE_URL; do
    local value="${!key:-}"
    if [[ -n "$value" ]]; then
      candidate_json "$(normalize_url "$value")" "$key" "configured environment" >> "$tmp"
    fi
  done

  for path in /etc/focusa/pairing-url /etc/focusa/public-url .focusa-pairing-url .focusa-public-url; do
    if [[ -f "$path" ]]; then
      local value
      value="$(normalize_url "$(grep -v '^#' "$path" | head -1 || true)")"
      [[ -n "$value" ]] && candidate_json "$value" "config_file" "$path" >> "$tmp"
    fi
  done

  local fqdn
  fqdn="$(hostname -f 2>/dev/null || hostname 2>/dev/null || true)"
  fqdn="${fqdn%.}"
  if [[ -n "$fqdn" && "$fqdn" != localhost && "$fqdn" != *.local && "$fqdn" != *.localdomain ]]; then
    candidate_json "https://${fqdn}" "hostname_https" "verified only if Focusa Connect responds" >> "$tmp"
    candidate_json "http://${fqdn}" "hostname_http" "verified only if Focusa Connect responds" >> "$tmp"
    candidate_json "http://${fqdn}:8787" "hostname_daemon_port" "works only when daemon is reachable from phone" >> "$tmp"
  fi

  for ip in $(hostname -I 2>/dev/null || true); do
    case "$ip" in *:*) continue ;; esac
    if [[ "$(host_is_public_v4 "$ip")" == "1" ]]; then
      candidate_json "https://${ip}" "public_ip_https" "verified only if Focusa Connect responds" >> "$tmp"
      candidate_json "http://${ip}" "public_ip_http" "verified only if Focusa Connect responds" >> "$tmp"
      candidate_json "http://${ip}:8787" "public_ip_daemon_port" "works only when daemon is reachable from phone" >> "$tmp"
    elif [[ "$(host_is_private_or_tailscale_v4 "$ip")" == "1" ]]; then
      candidate_json "http://${ip}:8787" "private_or_tailscale_ip" "works when phone shares this network/Tailscale" >> "$tmp"
    fi
  done

  candidate_json "$DAEMON_URL" "local_daemon" "local/dev only; not phone-reachable from another device" >> "$tmp"
  jq -s 'unique_by(.url)' "$tmp"
  rm -f "$tmp"
}

check_connect_page() {
  local url="$1" body
  body="$(curl -kfSs --max-time 8 "$(normalize_url "$url")/connect" 2>/dev/null || true)"
  [[ "$body" == *"Focusa Connect"* && "$body" == *"Connect Mac to Focusa"* ]]
}

check_room_api() {
  local url="$1" payload
  payload="$(curl -kfSs --max-time 8 -X POST "$(normalize_url "$url")/v1/connect/room/start" \
    -H 'content-type: application/json' \
    -d "{\"server_url\":\"$(normalize_url "$url")\"}" 2>/dev/null || true)"
  printf '%s' "$payload" | jq -e '.status == "waiting_for_mac" and (.connect_url|type == "string")' >/dev/null 2>&1
}

validate_url() {
  local url="$1" connect_ok=0 room_ok=0
  check_connect_page "$url" && connect_ok=1 || true
  check_room_api "$url" && room_ok=1 || true
  if [[ "$JSON" -eq 1 ]]; then
    jq -n --arg url "$(normalize_url "$url")" --arg daemon_url "$DAEMON_URL" \
      --argjson connect_ok "$connect_ok" --argjson room_ok "$room_ok" \
      '{url:$url, daemon_url:$daemon_url, connect_page_ok:($connect_ok==1), room_api_ok:($room_ok==1), ok:(($connect_ok==1) and ($room_ok==1))}'
  else
    echo "Phone Bridge transport: $(normalize_url "$url")"
    echo "  /connect page:        $([[ "$connect_ok" -eq 1 ]] && echo OK || echo MISSING)"
    echo "  /v1/connect room API: $([[ "$room_ok" -eq 1 ]] && echo OK || echo MISSING)"
  fi
  [[ "$connect_ok" -eq 1 && "$room_ok" -eq 1 ]]
}

write_config() {
  local url="$1" dir
  dir="$(dirname "$CONFIG_PATH")"
  mkdir -p "$dir"
  printf '%s\n' "$(normalize_url "$url")" > "$CONFIG_PATH"
  echo "Wrote Phone Bridge transport URL: $CONFIG_PATH -> $(normalize_url "$url")"
}

print_options() {
  cat <<EOF
Phone Bridge transport options:

1. Existing public URL / reverse proxy
   - Point /connect/* and /v1/connect/* to ${DAEMON_URL}
   - Write URL with: sudo scripts/phone-bridge-transport.sh write --url https://focusa.example.com

2. Tailscale/private network
   - Ensure phone and VPS/Mac share the private network.
   - Use a verified http://<tailscale-ip>:8787 URL if reachable.

3. Temporary tunnel
   - Use your preferred tunnel (SSH, Cloudflare Tunnel, Tailscale Funnel, ngrok).
   - Set FOCUSA_PAIRING_URL or write /etc/focusa/public-url to the tunnel URL.

4. Local/dev fallback
   - http://127.0.0.1:8787 works only on the same machine.

Validate any option with:
  scripts/phone-bridge-transport.sh check --url <url>
EOF
}

print_proxy_snippets() {
  cat <<EOF
# Optional reverse-proxy snippets for Phone Bridge Flow
# Local daemon: ${DAEMON_URL}

## nginx
location /connect/ { proxy_pass ${DAEMON_URL}/connect/; }
location /v1/connect/ { proxy_pass ${DAEMON_URL}/v1/connect/; }

## Apache httpd
ProxyPass        /connect/ ${DAEMON_URL}/connect/
ProxyPassReverse /connect/ ${DAEMON_URL}/connect/
ProxyPass        /v1/connect/ ${DAEMON_URL}/v1/connect/
ProxyPassReverse /v1/connect/ ${DAEMON_URL}/v1/connect/

## Caddy
handle /connect* { reverse_proxy ${DAEMON_URL} }
handle /v1/connect* { reverse_proxy ${DAEMON_URL} }

## LiteSpeed / OpenLiteSpeed
# Add Context type=Proxy for URI /connect/    -> ${DAEMON_URL}/connect/
# Add Context type=Proxy for URI /v1/connect/ -> ${DAEMON_URL}/v1/connect/
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    detect|check|write|options|proxy-snippets) MODE="$1"; shift ;;
    --url) URL="$(normalize_url "${2:?--url requires URL}")"; shift 2 ;;
    --daemon-url) DAEMON_URL="$(normalize_url "${2:?--daemon-url requires URL}")"; shift 2 ;;
    --config-path) CONFIG_PATH="${2:?--config-path requires PATH}"; shift 2 ;;
    --json) JSON=1; shift ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown argument: $1" >&2; usage >&2; exit 2 ;;
  esac
done

case "$MODE" in
  detect) emit_candidates_json ;;
  check) [[ -n "$URL" ]] || { echo "check requires --url" >&2; exit 2; }; validate_url "$URL" ;;
  write) [[ -n "$URL" ]] || { echo "write requires --url" >&2; exit 2; }; write_config "$URL" ;;
  options) print_options ;;
  proxy-snippets) print_proxy_snippets ;;
  help) usage ;;
  *) echo "Unknown mode: $MODE" >&2; usage >&2; exit 2 ;;
esac
