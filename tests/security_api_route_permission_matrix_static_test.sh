#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DOC="$ROOT_DIR/docs/current/API_ROUTE_PERMISSION_MATRIX.md"
PERM="$ROOT_DIR/crates/focusa-api/src/routes/permissions.rs"
CAPS="$ROOT_DIR/crates/focusa-api/src/routes/capabilities_extra.rs"
[[ -f "$DOC" ]] || { echo "missing API route permission matrix doc" >&2; exit 1; }

for needle in \
  "public:health" \
  "state:write" \
  "workpoint:write" \
  "trajectory:write" \
  "metacog:write" \
  "prediction:write" \
  "work_loop:control" \
  "admin:service" \
  "Every mutation route checks a non-read scope"; do
  if ! grep -Fq "$needle" "$DOC"; then
    echo "permission matrix missing marker: $needle" >&2
    exit 1
  fi
done

for marker in "requested_scopes" "permission_context" "Compatibility shim" "forbid"; do
  if ! grep -Fq "$marker" "$PERM"; then
    echo "permission helper missing marker: $marker" >&2
    exit 1
  fi
done

if ! grep -Fq "require_scope" "$CAPS"; then
  echo "expected existing scoped route enforcement helper in capabilities_extra" >&2
  exit 1
fi

echo "✓ API route permission matrix static markers present"
