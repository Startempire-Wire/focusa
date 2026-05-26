#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MAIN="$ROOT_DIR/crates/focusa-api/src/main.rs"
AUTH="$ROOT_DIR/crates/focusa-api/src/middleware/auth.rs"

for needle in \
  "fn enforce_bind_auth_guard" \
  "INSECURE_BIND_WITHOUT_AUTH" \
  "bind_is_loopback" \
  "enforced_auth_token_configured" \
  "enforce_bind_auth_guard(&config)?"; do
  if ! grep -Fq "$needle" "$MAIN"; then
    echo "missing non-loopback auth guard marker: $needle" >&2
    exit 1
  fi
done

if ! grep -Fq "FOCUSA_AUTH_TOKEN" "$AUTH" || grep -Fq "Config token check" "$AUTH"; then
  echo "auth middleware must enforce/document FOCUSA_AUTH_TOKEN only until config-token middleware exists" >&2
  exit 1
fi

if ! grep -Fq "bind_auth_guard_rejects_non_loopback_with_config_only_token" "$MAIN"; then
  echo "non-loopback guard must reject config-only token mismatch" >&2
  exit 1
fi

echo "✓ non-loopback auth guard static markers present"
