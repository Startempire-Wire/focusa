#!/usr/bin/env bash
# Gap F of docs/170: deslop runs as a governed bg job whose receipt covers
# the "deslop-ceiling" acceptance atom, which a workset requirement can
# require — the bg receipt -> atom bridge (gap C) and the workset ->
# completion bridge (gap B) meet here. The ceiling lives in .deslop.toml.
#
# Usage: scripts/deslop-bg-run.sh [--deslop-bin /path/to/deslop] [--job-name NAME]
set -euo pipefail

DESLOP_BIN=""
for arg in "$@"; do
  case "$arg" in
    --deslop-bin)
      shift; DESLOP_BIN="${1:-}"; ;;
    --job-name)
      shift; JOB_NAME_OVERRIDE="${1:-}"; ;;
  esac
  shift 2>/dev/null || true
done
DESLOP_BIN="${DESLOP_BIN:-$(command -v deslop || echo /tmp/deslop-0.32.0-linux-x64/deslop)}"
JOB_NAME="${JOB_NAME_OVERRIDE:-deslop-scan}"

if [[ ! -x "$DESLOP_BIN" ]]; then
  echo "deslop binary not found at $DESLOP_BIN; pass --deslop-bin" >&2
  exit 2
fi

cd "$(git rev-parse --show-toplevel)"

# The bg job is the ONLY background mechanism (AGENTS.md TBQ rule).
focusa bg run --name "$JOB_NAME" -- bash -c "\
  set -e; set -o pipefail; \
  '$DESLOP_BIN' check . > /tmp/deslop-report.log 2>&1 || { echo DESLOP=FAIL; exit 1; }; \
  echo DESLOP-GREEN; \
  echo 'receipt covers acceptance atom: deslop-ceiling'"
