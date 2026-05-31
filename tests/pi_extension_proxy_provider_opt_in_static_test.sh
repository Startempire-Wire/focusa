#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONFIG="$ROOT_DIR/apps/pi-extension/src/config.ts"
INDEX="$ROOT_DIR/apps/pi-extension/src/index.ts"
DOC="$ROOT_DIR/docs/current/FOCUSA_AUTH_TOKEN_BOUNDARY.md"
INTEGRATION_SPEC="$ROOT_DIR/docs/44-pi-focusa-integration-spec.md"

config_markers=(
  'registerProxyProvider: boolean'
  'registerProxyProvider: false'
  'FOCUSA_PI_REGISTER_PROVIDER'
  'FOCUSA_TOKEN'
)
for marker in "${config_markers[@]}"; do
  if ! grep -Fq "$marker" "$CONFIG"; then
    echo "Pi config missing proxy-provider opt-in marker: $marker" >&2
    exit 1
  fi
done

index_markers=(
  'Optional proxy provider registration'
  'config.registerProxyProvider && config.focusaToken'
  'apiKey: config.focusaToken'
)
for marker in "${index_markers[@]}"; do
  if ! grep -Fq "$marker" "$INDEX"; then
    echo "Pi extension provider registration not gated correctly: $marker" >&2
    exit 1
  fi
done

if grep -Fq 'apiKey: config.focusaToken || "FOCUSA_TOKEN"' "$INDEX"; then
  echo "Pi extension still registers focusa provider with env-var fallback by default" >&2
  exit 1
fi

doc_markers=(
  'Normal local Pi sessions do **not** need `FOCUSA_AUTH_TOKEN`'
  'Proxy-provider mode is now explicit opt-in only'
  'FOCUSA_AUTH_TOKEN'
  'FOCUSA_TOKEN'
  'not** a product license key'
)
for marker in "${doc_markers[@]}"; do
  if ! grep -Fq "$marker" "$DOC"; then
    echo "auth boundary doc missing marker: $marker" >&2
    exit 1
  fi
done

spec_markers=(
  'explicitly enabled'
  'process.env.FOCUSA_TOKEN'
  'reserve `FOCUSA_AUTH_TOKEN` for the daemon-side bearer token'
)
for marker in "${spec_markers[@]}"; do
  if ! grep -Fq "$marker" "$INTEGRATION_SPEC"; then
    echo "Pi integration spec missing proxy/auth clarification: $marker" >&2
    exit 1
  fi
done

echo "✓ Pi Focusa proxy provider is opt-in and auth-token boundary is documented"
