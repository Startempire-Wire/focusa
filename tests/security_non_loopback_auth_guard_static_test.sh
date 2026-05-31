#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MAIN="$ROOT_DIR/crates/focusa-api/src/main.rs"
AUTH="$ROOT_DIR/crates/focusa-api/src/middleware/auth.rs"
DYNAMIC="$ROOT_DIR/tests/security_non_loopback_auth_guard_dynamic_test.sh"
GATE="$ROOT_DIR/scripts/ci/run-spec-gates.sh"

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


[[ -x "$DYNAMIC" ]] || { echo "non-loopback dynamic smoke missing or not executable" >&2; exit 1; }
for needle in \
  "0.0.0.0" \
  "INSECURE_BIND_WITHOUT_AUTH" \
  "FOCUSA_AUTH_TOKEN" \
  "Authorization: Bearer" \
  "x-focusa-permissions: project:read" \
  "DAEMON_BIN"; do
  if ! grep -Fq "$needle" "$DYNAMIC"; then
    echo "missing non-loopback dynamic marker: $needle" >&2
    exit 1
  fi
done

for needle in \
  "security_non_loopback_auth_guard_static_test.sh" \
  "security_non_loopback_auth_guard_dynamic_test.sh"; do
  if ! grep -Fq "$needle" "$GATE"; then
    echo "missing non-loopback CI gate marker: $needle" >&2
    exit 1
  fi
done

echo "✓ non-loopback auth guard static and dynamic CI markers present"
