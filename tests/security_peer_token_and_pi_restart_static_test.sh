#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SYNC="$ROOT_DIR/crates/focusa-api/src/routes/sync.rs"
STATE="$ROOT_DIR/apps/pi-extension/src/state.ts"
CONFIG="$ROOT_DIR/apps/pi-extension/src/config.ts"
BOUNDARY="$ROOT_DIR/docs/current/SECURITY_COMMAND_BOUNDARY.md"

for marker in \
  "peer auth_token persistence is disabled" \
  "unsupported sync credential configuration" \
  "body.auth_token"; do
  if ! grep -Fq "$marker" "$SYNC"; then
    echo "sync peer-token rejection marker missing: $marker" >&2
    exit 1
  fi
done

if grep -Fq 'S.pi!.exec("bash", ["-lc", cmd])' "$STATE"; then
  echo "Pi daemon kickstart still uses bash -lc" >&2
  exit 1
fi
for marker in \
  'S.pi!.exec("systemctl", ["restart", "focusa-daemon"])' \
  'DEFAULT_DAEMON_RESTART_COMMAND = "systemctl restart focusa-daemon"' \
  'custom shell restart commands are refused'; do
  if ! grep -Fq "$marker" "$STATE" "$CONFIG" "$BOUNDARY"; then
    echo "Pi fixed-argv restart marker missing: $marker" >&2
    exit 1
  fi
done

echo "✓ peer token persistence rejection and Pi fixed-argv restart markers present"
