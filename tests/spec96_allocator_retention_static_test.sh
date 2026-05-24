#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
SERVER_RS="${ROOT_DIR}/crates/focusa-api/src/server.rs"
SPEC96="${ROOT_DIR}/docs/96-trajectory-projection-and-daemon-stability-spec.md"

if rg -n 'fn malloc_trim|trim_allocator_once|allocator_trim_loop|FOCUSA_ALLOCATOR_TRIM_INTERVAL_SECS' "$SERVER_RS" >/dev/null; then
  echo "✓ PASS: daemon has allocator trim loop wiring"
else
  echo "✗ FAIL: allocator trim loop wiring missing" >&2
  exit 1
fi

if rg -n 'tokio::task::spawn_blocking\(trim_allocator_once\)|allocator_trim_loop\(\)\.await' "$SERVER_RS" >/dev/null; then
  echo "✓ PASS: allocator trim runs off hot async workers and is spawned at startup"
else
  echo "✗ FAIL: allocator trim loop should spawn blocking trim and start with server" >&2
  exit 1
fi

if rg -n 'FOCUSA_ALLOCATOR_TRIM_INTERVAL_SECS' "$SPEC96" >/dev/null; then
  echo "✓ PASS: allocator trim interval is documented in Spec96"
else
  echo "✗ FAIL: allocator trim interval missing from Spec96" >&2
  exit 1
fi

SYSTEMD_CAT_FILE="$(mktemp /tmp/focusa-daemon-systemd-cat.XXXXXX)"
trap 'rm -f "$SYSTEMD_CAT_FILE"' EXIT
if command -v systemctl >/dev/null 2>&1 && systemctl cat focusa-daemon.service >"$SYSTEMD_CAT_FILE" 2>/dev/null; then
  if rg -n 'MALLOC_ARENA_MAX=2|MALLOC_TRIM_THRESHOLD_=131072' "$SYSTEMD_CAT_FILE" >/dev/null; then
    echo "✓ PASS: live systemd service caps glibc arenas/trim threshold"
  else
    echo "✗ FAIL: live systemd service missing allocator env caps" >&2
    cat "$SYSTEMD_CAT_FILE" >&2
    exit 1
  fi
else
  echo "↪ SKIP: systemd service not available in this environment"
fi

echo "SPEC96 allocator retention static test: PASS"
