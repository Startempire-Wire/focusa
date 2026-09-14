#!/usr/bin/env bash
# scripts/sync-install-bootstrapper.sh
# Sync scripts/install-focusa.sh (single source of truth) to the live
# install.focusa.dev docroot, where it is served as the public install URL.
#
# Why this exists: previously the in-repo scripts/install-focusa.sh and
# /home/focusadev/install.focusa.dev/public_html/installers/install-focusa.sh
# drifted apart (440-line live shell vs 173-line in-repo). Per the operator
# rule on 2026-07-07, the in-repo script is the canonical bootstrapper; the
# live shell must be a byte-identical copy.
#
# Usage:
#   scripts/sync-install-bootstrapper.sh           # copies in-repo → live
#   scripts/sync-install-bootstrapper.sh --check   # exits 1 if drift detected
#   scripts/sync-install-bootstrapper.sh --check --quiet  # silent check
#
# Operates as the focusadev user (the live docroot is owned by focusadev).
set -euo pipefail

REPO_ROOT="${REPO_ROOT:-$(cd "$(dirname "$0")/.." && pwd)}"
SRC="${REPO_ROOT}/scripts/install-focusa.sh"
LIVE_DIR="/home/focusadev/install.focusa.dev/public_html/installers"
LIVE="${LIVE_DIR}/install-focusa.sh"
DOCROOT="/home/focusadev/install.focusa.dev/public_html"
ALIAS="${DOCROOT}/focusa"

# Live docroot is owned by focusadev; mutate it as that user so the deploy
# runner never creates root/wirebot-owned files in a cPanel account.
as_focusadev() {
  if [ "$(id -un)" = "focusadev" ]; then
    "$@"
  else
    sudo -n -u focusadev -- "$@"
  fi
}

sync_copy() {
  as_focusadev mkdir -p "$LIVE_DIR"
  as_focusadev mkdir -p "$DOCROOT"
  as_focusadev install -m 0755 "$SRC" "$LIVE"
  as_focusadev install -m 0755 "$SRC" "$ALIAS"
  # Ensure both bootstrapper aliases stay in sync
  for target in "$LIVE" "$ALIAS"; do as_focusadev chmod 0755 "$target"; done
}

mode="sync"
quiet="0"
for arg in "$@"; do
  case "$arg" in
    --check) mode="check" ;;
    --quiet) quiet="1" ;;
  esac
done

[ -f "$SRC" ] || { echo "[sync-install-bootstrapper] missing source: $SRC" >&2; exit 2; }

if [ "$mode" = "check" ]; then
  if ! as_focusadev test -f "$LIVE"; then
    [ "$quiet" = "1" ] || echo "[sync-install-bootstrapper] live not found: $LIVE" >&2
    exit 1
  fi
  source_sha="$(sha256sum "$SRC" | awk '{print $1}')"
  live_sha="$(as_focusadev sha256sum "$LIVE" | awk '{print $1}')"
  if [ "$source_sha" != "$live_sha" ]; then
    [ "$quiet" = "1" ] || echo "[sync-install-bootstrapper] DRIFT: $SRC != $LIVE" >&2
    exit 1
  fi
  [ "$quiet" = "1" ] || echo "[sync-install-bootstrapper] OK: $SRC == $LIVE" >&2
  exit 0
fi

sync_copy
echo "[sync-install-bootstrapper] synced: $SRC → $LIVE"
