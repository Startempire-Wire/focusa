#!/usr/bin/env bash
# Focusa Daemon Installer — properly deploys the daemon binary
# Fixes the manual-cp portability issue: cargo install → systemd reload
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
INSTALL_ROOT="${FOCUSA_INSTALL_ROOT:-/usr/local}"
SERVICE_NAME="focusa-daemon"

echo "Installing Focusa daemon from $ROOT_DIR to $INSTALL_ROOT..."

# 1. Build release binary (release preferred, debug fallback)
if [ -d "$ROOT_DIR/target/release" ] && [ -f "$ROOT_DIR/target/release/focusa-daemon" ]; then
  BINARY="$ROOT_DIR/target/release/focusa-daemon"
elif [ -f "$ROOT_DIR/target/debug/focusa-daemon" ]; then
  BINARY="$ROOT_DIR/target/debug/focusa-daemon"
  echo "  note: using debug binary (run cargo build --release for production)"
else
  echo "  building daemon (debug)..."
  export PATH="/root/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin:$PATH"
  cd "$ROOT_DIR"
  cargo build -p focusa-api --bin focusa-daemon
  BINARY="$ROOT_DIR/target/debug/focusa-daemon"
fi

# 2. Stop running daemon (if any)
if pgrep -f "focusa-daemon" > /dev/null; then
  echo "  stopping running daemon..."
  systemctl stop "$SERVICE_NAME" 2>/dev/null || pkill -TERM -f focusa-daemon || true
  sleep 3
fi

# 3. Install binary (handle "Text file busy" by removing first if needed)
if [ -f "$INSTALL_ROOT/bin/focusa-daemon" ]; then
  rm -f "$INSTALL_ROOT/bin/focusa-daemon"
fi
install -m 0755 "$BINARY" "$INSTALL_ROOT/bin/focusa-daemon"
echo "  installed: $INSTALL_ROOT/bin/focusa-daemon"

# 4. Reload systemd + restart daemon (if service exists)
if systemctl list-unit-files "$SERVICE_NAME.service" >/dev/null 2>&1; then
  systemctl daemon-reload
  systemctl start "$SERVICE_NAME"
  echo "  service restarted: $SERVICE_NAME"
  sleep 3
fi

# 5. Verify health
echo "  verifying daemon health..."
for i in $(seq 1 10); do
  if curl -sf "$FOCUSA_DAEMON_URL/v1/health" >/dev/null 2>&1; then
    echo "  ✓ daemon healthy"
    curl -s "$FOCUSA_DAEMON_URL/v1/health" | jq -r '"  version=" + .version' 2>/dev/null || true
    exit 0
  fi
  sleep 1
done

echo "  ✗ daemon not responding after install"
exit 1