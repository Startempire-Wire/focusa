#!/usr/bin/env bash
# Gap F of docs/170: deslop runs as a governed bg job whose receipt covers
# the "deslop-ceiling" acceptance atom, which a workset requirement can
# require — the bg receipt -> atom bridge (gap C) and the workset ->
# completion bridge (gap B) meet here. The ceiling lives in .deslop.toml.
#
# Usage: scripts/deslop-bg-run.sh [--deslop-bin /path/to/deslop] [--job-name NAME]
set -euo pipefail

DESLOP_BIN=""
JOB_NAME_OVERRIDE=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --deslop-bin)
      [[ $# -ge 2 ]] || { echo "missing value for --deslop-bin" >&2; exit 2; }
      DESLOP_BIN="$2"
      shift 2
      ;;
    --job-name)
      [[ $# -ge 2 ]] || { echo "missing value for --job-name" >&2; exit 2; }
      JOB_NAME_OVERRIDE="$2"
      shift 2
      ;;
    *)
      echo "unknown option: $1" >&2
      exit 2
      ;;
  esac
done
JOB_NAME="${JOB_NAME_OVERRIDE:-deslop-scan}"
PROJECT_ROOT="$(git rev-parse --show-toplevel)"

if [[ -n "$DESLOP_BIN" ]]; then
  [[ -x "$DESLOP_BIN" ]] || {
    echo "deslop binary not found at $DESLOP_BIN" >&2
    exit 2
  }
  SCAN_COMMAND="'$DESLOP_BIN' ."
else
  [[ -x "$PROJECT_ROOT/scripts/deslop" ]] || {
    echo "canonical local Deslop entry point missing: $PROJECT_ROOT/scripts/deslop" >&2
    exit 2
  }
  SCAN_COMMAND="'$PROJECT_ROOT/scripts/deslop' ."
fi

# The bg job is the ONLY background mechanism (AGENTS.md TBQ rule).
focusa bg run --name "$JOB_NAME" -- bash -c "\
  set -e; set -o pipefail; \
  $SCAN_COMMAND > /tmp/deslop-report.log 2>&1 || { cat /tmp/deslop-report.log; echo DESLOP=FAIL; exit 1; }; \
  cat /tmp/deslop-report.log; \
  echo 'receipt covers acceptance atom: deslop-ceiling'"
