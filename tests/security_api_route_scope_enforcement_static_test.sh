#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCOPE="$ROOT_DIR/crates/focusa-api/src/middleware/route_scope.rs"
SERVER="$ROOT_DIR/crates/focusa-api/src/server.rs"
DOC="$ROOT_DIR/docs/current/API_ROUTE_PERMISSION_MATRIX.md"
[[ -f "$SCOPE" ]] || { echo "missing route scope middleware" >&2; exit 1; }

for marker in \
  "route_scope_layer" \
  "request_principal" \
  "append_capability_authorization_audit" \
  "can(&principal, &capability, &context)" \
  "state:write" \
  "workpoint:write" \
  "trajectory:write" \
  "metacog:write" \
  "prediction:write" \
  "work_loop:control" \
  "admin:service" \
  "mutation_routes_require_write_or_control_scopes"; do
  if ! grep -Fq "$marker" "$SCOPE"; then
    echo "route scope enforcement missing marker: $marker" >&2
    exit 1
  fi
done

if ! grep -Fq "middleware::route_scope::route_scope_layer" "$SERVER"; then
  echo "server router missing route-scope middleware layer" >&2
  exit 1
fi

if ! grep -Fq "Every mutation route checks a non-read scope" "$DOC"; then
  echo "permission matrix acceptance marker missing" >&2
  exit 1
fi

echo "✓ API route scope enforcement static markers present"
