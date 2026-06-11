#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
SCRIPT="${ROOT_DIR}/scripts/setup-phone-bridge-url.sh"
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

bash -n "$SCRIPT"
assert_has "$SCRIPT" '/connect/\*' 'proxy helper documents /connect route'
assert_has "$SCRIPT" '/v1/connect/\*' 'proxy helper documents /v1/connect route'
assert_has "$SCRIPT" '/etc/focusa/public-url' 'proxy helper writes canonical public URL file'
assert_has "$SCRIPT" 'Focusa Connect' 'proxy helper validates actual Focusa Connect page'
assert_has "$SCRIPT" 'room/start' 'proxy helper validates Bridge Room API'
assert_has "$GLOSSARY" 'Phone Bridge Flow' 'glossary defines Phone Bridge Flow'
assert_has "$GLOSSARY" 'Public Focusa URL' 'glossary defines Public Focusa URL'
assert_has "$PLAN" 'scripts/setup-phone-bridge-url\.sh' 'Phone Bridge plan references setup helper'

echo "Phone Bridge public URL static test: PASS"
