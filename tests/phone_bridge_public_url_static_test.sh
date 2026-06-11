#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
SCRIPT="${ROOT_DIR}/scripts/phone-bridge-transport.sh"
SHIM="${ROOT_DIR}/scripts/setup-phone-bridge-url.sh"
GLOSSARY="${ROOT_DIR}/docs/00-glossary.md"
PLAN="${ROOT_DIR}/docs/54-focusa-pairing-room-plan.md"

assert_has() {
  local file="$1"
  local pattern="$2"
  local label="$3"
  if rg -n "$pattern" "$file" >/dev/null; then
    echo "✓ PASS: ${label}"
  else
    echo "✗ FAIL: ${label}" >&2
    echo "Missing pattern '${pattern}' in ${file}" >&2
    exit 1
  fi
}

bash -n "$SCRIPT" "$SHIM"
assert_has "$SCRIPT" 'detect\|check\|write\|options\|proxy-snippets' 'transport resolver exposes adaptive modes'
assert_has "$SCRIPT" 'FOCUSA_PAIRING_URL' 'transport resolver includes configured URL candidates'
assert_has "$SCRIPT" 'private_or_tailscale_ip' 'transport resolver includes private/Tailscale candidates'
assert_has "$SCRIPT" '/connect/\*' 'transport resolver documents /connect route'
assert_has "$SCRIPT" '/v1/connect/\*' 'transport resolver documents /v1/connect route'
assert_has "$SCRIPT" '/etc/focusa/public-url' 'transport resolver writes canonical public URL file'
assert_has "$SCRIPT" 'Focusa Connect' 'transport resolver validates actual Focusa Connect page'
assert_has "$SCRIPT" 'room/start' 'transport resolver validates Bridge Room API'
assert_has "$SHIM" 'phone-bridge-transport\.sh' 'legacy setup helper delegates to transport resolver'
assert_has "$GLOSSARY" 'Phone Bridge Flow' 'glossary defines Phone Bridge Flow'
assert_has "$GLOSSARY" 'Public Focusa URL' 'glossary defines Public Focusa URL'
assert_has "$PLAN" 'Phone Bridge Transport Resolver' 'Phone Bridge plan defines transport resolver'
assert_has "$PLAN" 'scripts/phone-bridge-transport\.sh' 'Phone Bridge plan references transport resolver helper'

echo "Phone Bridge transport static test: PASS"
