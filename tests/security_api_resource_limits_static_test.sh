#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SERVER="$ROOT_DIR/crates/focusa-api/src/server.rs"
DOC="$ROOT_DIR/docs/current/API_RESOURCE_LIMITS.md"
[[ -f "$DOC" ]] || { echo "missing API resource limits doc" >&2; exit 1; }

for marker in \
  "DefaultBodyLimit" \
  "FOCUSA_API_MAX_BODY_BYTES" \
  "1_048_576"; do
  if ! grep -Fq "$marker" "$SERVER"; then
    echo "server missing request body bound marker: $marker" >&2
    exit 1
  fi
done

for marker in \
  "OWASP API4" \
  "CWE-400" \
  "HTTP request body size" \
  "JSON depth posture" \
  "Rate-limit posture"; do
  if ! grep -Fq "$marker" "$DOC"; then
    echo "resource limits doc missing marker: $marker" >&2
    exit 1
  fi
done

echo "✓ API resource-limit static markers present"
