#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MAIN="$ROOT_DIR/crates/focusa-api/src/main.rs"
AUTH="$ROOT_DIR/crates/focusa-api/src/middleware/auth.rs"

for needle in \
  "fn enforce_bind_auth_guard" \
  "INSECURE_BIND_WITHOUT_AUTH" \
  "bind_is_loopback" \
  "auth_token_configured" \
  "enforce_bind_auth_guard(&config)?"; do
  if ! grep -Fq "$needle" "$MAIN"; then
    echo "missing non-loopback auth guard marker: $needle" >&2
    exit 1
  fi
done

if ! grep -Fq "FOCUSA_AUTH_TOKEN" "$AUTH"; then
  echo "auth middleware must document/use FOCUSA_AUTH_TOKEN" >&2
  exit 1
fi

echo "✓ non-loopback auth guard static markers present"
