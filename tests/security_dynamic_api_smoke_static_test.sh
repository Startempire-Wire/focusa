#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPT="$ROOT_DIR/tests/security_dynamic_api_smoke_test.sh"
DOC="$ROOT_DIR/docs/current/DYNAMIC_API_SECURITY_SMOKE.md"
[[ -x "$SCRIPT" ]] || { echo "dynamic API smoke script missing or not executable" >&2; exit 1; }
[[ -f "$DOC" ]] || { echo "dynamic API smoke doc missing" >&2; exit 1; }

for marker in \
  "FOCUSA_API_MAX_BODY_BYTES=4096" \
  "127.0.0.1" \
  "/v1/health" \
  "/v1/telemetry/trace" \
  "malformed JSON" \
  "HTTP 413"; do
  if ! grep -Fq "$marker" "$SCRIPT" "$DOC"; then
    echo "dynamic API smoke marker missing: $marker" >&2
    exit 1
  fi
done

echo "✓ dynamic API security smoke static markers present"
