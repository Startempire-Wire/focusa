#!/usr/bin/env bash
# Configure/validate the Public Focusa URL used by the Phone Bridge Flow.
#
# This script is intentionally portable: it does not mutate webserver config.
# It prints reverse-proxy snippets and writes /etc/focusa/public-url only with --write.

set -euo pipefail

URL=""
DAEMON_URL="${FOCUSA_DAEMON_URL:-http://127.0.0.1:8787}"
CONFIG_PATH="${FOCUSA_PUBLIC_URL_FILE:-/etc/focusa/public-url}"
WRITE=0
CHECK=0
JSON=0
PRINT_PROXY=0

usage() {
  cat <<'EOF'
Usage: scripts/setup-phone-bridge-url.sh --url https://focusa.example.com [options]

Options:
  --url URL           Public Focusa URL phones will open.
  --daemon-url URL    Local daemon URL to proxy to (default: http://127.0.0.1:8787).
  --config-path PATH  URL config file (default: /etc/focusa/public-url).
  --write            Write URL to config path.
  --check            Validate public /connect and /v1/connect/room/start.
  --print-proxy      Print nginx/apache/caddy/litespeed proxy snippets.
  --json             Machine-readable validation summary.
  -h, --help         Show help.

Install contract:
  Public URL must proxy:
    /connect/*      -> ${DAEMON_URL}/connect/*
    /v1/connect/*   -> ${DAEMON_URL}/v1/connect/*

After setup:
  focusa pair
EOF
}

normalize_url() {
  printf '%s' "$1" | sed 's:/*$::'
}

json_escape() {
  python3 -c 'import json,sys; print(json.dumps(sys.stdin.read()))'
}

require_url() {
  if [[ -z "$URL" ]]; then
    echo "--url is required" >&2
    usage >&2
    exit 2
  fi
  URL="$(normalize_url "$URL")"
}

print_proxy_snippets() {
  cat <<EOF
# Phone Bridge Flow reverse proxy examples
# Public URL: ${URL:-https://focusa.example.com}
# Local daemon: ${DAEMON_URL}

## nginx
location /connect/ {
  proxy_pass ${DAEMON_URL}/connect/;
  proxy_set_header Host \$host;
  proxy_set_header X-Forwarded-Proto \$scheme;
  proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
}
location /v1/connect/ {
  proxy_pass ${DAEMON_URL}/v1/connect/;
  proxy_set_header Host \$host;
  proxy_set_header X-Forwarded-Proto \$scheme;
  proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
}

## Apache httpd
ProxyPass        /connect/ ${DAEMON_URL}/connect/
ProxyPassReverse /connect/ ${DAEMON_URL}/connect/
ProxyPass        /v1/connect/ ${DAEMON_URL}/v1/connect/
ProxyPassReverse /v1/connect/ ${DAEMON_URL}/v1/connect/

## Caddy
handle /connect* {
  reverse_proxy ${DAEMON_URL}
}
handle /v1/connect* {
  reverse_proxy ${DAEMON_URL}
}

## LiteSpeed / OpenLiteSpeed
# Add Context type=Proxy for URI /connect/  -> ${DAEMON_URL}/connect/
# Add Context type=Proxy for URI /v1/connect/ -> ${DAEMON_URL}/v1/connect/
EOF
}

write_config() {
  require_url
  local dir
  dir="$(dirname "$CONFIG_PATH")"
  mkdir -p "$dir"
  printf '%s\n' "$URL" > "$CONFIG_PATH"
  echo "Wrote Public Focusa URL: $CONFIG_PATH -> $URL"
}

check_connect_page() {
  require_url
  local body
  body="$(curl -kfSs --max-time 8 "$URL/connect" 2>/dev/null || true)"
  [[ "$body" == *"Focusa Connect"* && "$body" == *"Connect Mac to Focusa"* ]]
}

check_room_api() {
  require_url
  local payload
  payload="$(curl -kfSs --max-time 8 -X POST "$URL/v1/connect/room/start" \
    -H 'content-type: application/json' \
    -d "{\"server_url\":\"$URL\"}" 2>/dev/null || true)"
  printf '%s' "$payload" | jq -e '.status == "waiting_for_mac" and (.connect_url|type == "string")' >/dev/null 2>&1
}

validate() {
  require_url
  local connect_ok=0 room_ok=0
  check_connect_page && connect_ok=1 || true
  check_room_api && room_ok=1 || true

  if [[ "$JSON" -eq 1 ]]; then
    jq -n \
      --arg url "$URL" \
      --arg daemon_url "$DAEMON_URL" \
      --arg config_path "$CONFIG_PATH" \
      --argjson connect_ok "$connect_ok" \
      --argjson room_ok "$room_ok" \
      '{url:$url, daemon_url:$daemon_url, config_path:$config_path, connect_page_ok:($connect_ok==1), room_api_ok:($room_ok==1), ok:(($connect_ok==1) and ($room_ok==1))}'
  else
    echo "Public Focusa URL: $URL"
    echo "  /connect page:        $([[ "$connect_ok" -eq 1 ]] && echo OK || echo MISSING)"
    echo "  /v1/connect room API: $([[ "$room_ok" -eq 1 ]] && echo OK || echo MISSING)"
  fi

  [[ "$connect_ok" -eq 1 && "$room_ok" -eq 1 ]]
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --url) URL="${2:?--url requires URL}"; shift 2 ;;
    --daemon-url) DAEMON_URL="$(normalize_url "${2:?--daemon-url requires URL}")"; shift 2 ;;
    --config-path) CONFIG_PATH="${2:?--config-path requires PATH}"; shift 2 ;;
    --write) WRITE=1; shift ;;
    --check) CHECK=1; shift ;;
    --print-proxy) PRINT_PROXY=1; shift ;;
    --json) JSON=1; shift ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown argument: $1" >&2; usage >&2; exit 2 ;;
  esac
done

if [[ "$PRINT_PROXY" -eq 1 ]]; then
  print_proxy_snippets
fi
if [[ "$WRITE" -eq 1 ]]; then
  write_config
fi
if [[ "$CHECK" -eq 1 ]]; then
  validate
fi
if [[ "$PRINT_PROXY$WRITE$CHECK" == "000" ]]; then
  usage
fi
