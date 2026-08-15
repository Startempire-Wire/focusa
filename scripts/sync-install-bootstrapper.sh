#!/usr/bin/env bash
# scripts/sync-install-bootstrapper.sh
# Sync scripts/install-focusa.sh (single source of truth) to the live
# install.focusa.dev docroot, where it is served as the public install URL.
#
# Why this exists: previously the in-repo scripts/install-focusa.sh and
# /home/focusadev/public_html/install.focusa.dev/installers/install-focusa.sh
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
<<<<<<< HEAD
DOCROOT="/home/focusadev/public_html/install.focusa.dev"
LIVE_DIR="${DOCROOT}/installers"
=======
LIVE_DIR="/home/focusadev/public_html/install.focusa.dev/installers"
>>>>>>> 33e62229 (fix: align bootstrapper parity with cPanel docroot)
LIVE="${LIVE_DIR}/install-focusa.sh"
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
  as_focusadev install -m 0755 "$SRC" "$LIVE"
  as_focusadev install -m 0755 "$SRC" "$ALIAS"
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
  for target in "$LIVE" "$ALIAS"; do
    if ! as_focusadev test -f "$target"; then
      [ "$quiet" = "1" ] || echo "[sync-install-bootstrapper] live not found: $target" >&2
      exit 1
    fi
    target_sha="$(as_focusadev sha256sum "$target" | awk '{print $1}')"
    if [ "$source_sha" != "$target_sha" ]; then
      [ "$quiet" = "1" ] || echo "[sync-install-bootstrapper] DRIFT: $SRC != $target" >&2
      exit 1
    fi
  done
  [ "$quiet" = "1" ] || echo "[sync-install-bootstrapper] OK: in-repo matches live + docroot alias" >&2
  exit 0
fi

sync_copy
echo "[sync-install-bootstrapper] synced: $SRC → $LIVE (+ $ALIAS)"
