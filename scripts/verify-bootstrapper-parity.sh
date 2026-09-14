#!/usr/bin/env bash
# scripts/verify-bootstrapper-parity.sh
# CI guard: the live bootstrapper served from install.focusa.dev MUST be a
# byte-identical copy of scripts/install-focusa.sh. Exits non-zero on drift.
# Used as a release-gate step and from the operator's pre-merge hook.
set -euo pipefail

REPO_ROOT="${REPO_ROOT:-$(cd "$(dirname "$0")/.." && pwd)}"
SRC="${REPO_ROOT}/scripts/install-focusa.sh"
LIVE="/home/focusadev/public_html/install.focusa.dev/installers/install-focusa.sh"

if [ ! -f "$SRC" ]; then
  echo "[verify-bootstrapper-parity] FAIL: source missing ($SRC)" >&2
  exit 2
fi
if [ ! -f "$LIVE" ]; then
  echo "[verify-bootstrapper-parity] FAIL: live missing ($LIVE)" >&2
  exit 2
fi
if ! cmp -s "$SRC" "$LIVE"; then
  echo "[verify-bootstrapper-parity] FAIL: drift between $SRC and $LIVE" >&2
  echo "[verify-bootstrapper-parity] run: scripts/sync-install-bootstrapper.sh" >&2
  diff "$SRC" "$LIVE" >&2 || true
  exit 1
fi
echo "[verify-bootstrapper-parity] OK: live bootstrapper == in-repo bootstrapper"
