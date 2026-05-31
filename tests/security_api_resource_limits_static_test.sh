#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SERVER="$ROOT_DIR/crates/focusa-api/src/server.rs"
RATE_LIMIT="$ROOT_DIR/crates/focusa-api/src/middleware/rate_limit.rs"
JSON_GUARD="$ROOT_DIR/crates/focusa-api/src/middleware/json_guard.rs"
DOC="$ROOT_DIR/docs/current/API_RESOURCE_LIMITS.md"
[[ -f "$DOC" ]] || { echo "missing API resource limits doc" >&2; exit 1; }
[[ -f "$RATE_LIMIT" ]] || { echo "missing rate-limit middleware" >&2; exit 1; }
[[ -f "$JSON_GUARD" ]] || { echo "missing JSON guard middleware" >&2; exit 1; }

for marker in \
  "DefaultBodyLimit" \
  "FOCUSA_API_MAX_BODY_BYTES" \
  "1_048_576" \
  "mutation_rate_limit_layer" \
  "mutation_json_guard_layer"; do
  if ! grep -Fq "$marker" "$SERVER"; then
    echo "server missing resource-limit marker: $marker" >&2
    exit 1
  fi
done

for marker in \
  "FOCUSA_API_MUTATION_RATE_LIMIT_PER_WINDOW" \
  "FOCUSA_API_MUTATION_RATE_LIMIT_WINDOW_MS" \
  "TOO_MANY_REQUESTS" \
  "rate_key" \
  "is_mutation_request"; do
  if ! grep -Fq "$marker" "$RATE_LIMIT"; then
    echo "rate-limit middleware missing marker: $marker" >&2
    exit 1
  fi
done

for marker in \
  "FOCUSA_API_JSON_MAX_DEPTH" \
  "FOCUSA_API_JSON_MAX_ARRAY_ITEMS" \
  "FOCUSA_API_JSON_MAX_OBJECT_FIELDS" \
  "validate_json_shape" \
  "mutation_json_guard_layer"; do
  if ! grep -Fq "$marker" "$JSON_GUARD"; then
    echo "JSON guard middleware missing marker: $marker" >&2
    exit 1
  fi
done

for marker in \
  "OWASP API4" \
  "CWE-400" \
  "HTTP request body size" \
  "JSON depth posture" \
  "Rate-limit posture" \
  "Mutation route rate limit" \
  "Mutation JSON shape guard" \
  "FOCUSA_API_MUTATION_RATE_LIMIT_PER_WINDOW" \
  "FOCUSA_API_JSON_MAX_DEPTH"; do
  if ! grep -Fq "$marker" "$DOC"; then
    echo "resource limits doc missing marker: $marker" >&2
    exit 1
  fi
done

echo "✓ API resource-limit static markers present"
