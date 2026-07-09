#!/usr/bin/env bash
# scripts/sync-install-bootstrapper-windows.sh
# Sync scripts/install-focusa.ps1 (single source of truth) to the live
# install.focusa.dev docroot, where it is served at /focusa.ps1 and /install.ps1.
#
# Mirrors scripts/sync-install-bootstrapper.sh (bash counterpart).
#
# Usage:
#   scripts/sync-install-bootstrapper-windows.sh           # copies in-repo → live
#   scripts/sync-install-bootstrapper-windows.sh --check   # exits 1 if drift detected
#   scripts/sync-install-bootstrapper-windows.sh --check --quiet  # silent check
#
# Operates as the focusadev user (the live docroot is owned by focusadev).
set -euo pipefail

REPO_ROOT="${REPO_ROOT:-$(cd "$(dirname "$0")/.." && pwd)}"
SRC="${REPO_ROOT}/scripts/install-focusa.ps1"
LIVE_DIR="/home/focusadev/install.focusa.dev/public_html/installers"
LIVE="${LIVE_DIR}/install-focusa.ps1"
DOCROOT="/home/focusadev/install.focusa.dev/public_html"

# Live docroot is owned by focusadev; copy via sudo -u focusadev cp so we don't
# break ownership.
sync_copy() {
  install -m 0644 -o focusadev -g focusadev "$SRC" "$LIVE"
  install -m 0755 -o focusadev -g focusadev "$SRC" "$DOCROOT/install.ps1"
  install -m 0755 -o focusadev -g focusadev "$LIVE" "$DOCROOT/focusa.ps1"
}

mode="sync"
quiet="0"
for arg in "$@"; do
  case "$arg" in
    --check) mode="check" ;;
    --quiet) quiet="1" ;;
  esac
done

[ -f "$SRC" ] || { echo "[sync-install-windows] missing source: $SRC" >&2; exit 2; }

if [ "$mode" = "check" ]; then
  if [ ! -f "$LIVE" ]; then
    [ "$quiet" = "1" ] || echo "[sync-install-windows] live not found: $LIVE" >&2
    exit 1
  fi
  if ! cmp -s "$SRC" "$LIVE"; then
    [ "$quiet" = "1" ] || echo "[sync-install-windows] DRIFT: $SRC != $LIVE" >&2
    exit 1
  fi
  if [ ! -L "$DOCROOT/focusa.ps1" ] || [ ! -L "$DOCROOT/install.ps1" ]; then
    [ "$quiet" = "1" ] || echo "[sync-install-windows] docroot aliases missing" >&2
    exit 1
  fi
  [ "$quiet" = "1" ] || echo "[sync-install-windows] OK: in-repo matches live + docroot aliases present" >&2
  exit 0
fi

mkdir -p "$LIVE_DIR"
sync_copy
echo "[sync-install-windows] synced: $SRC → $LIVE (+ $DOCROOT/install.ps1, $DOCROOT/focusa.ps1)"
