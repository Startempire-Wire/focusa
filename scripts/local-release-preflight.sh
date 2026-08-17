#!/bin/bash
set -euo pipefail
# Local preflight — fast mirror of CI Spec Gates (strict) without 6m wait.
# Usage: bash scripts/local-release-preflight.sh --strict
# Strict runs: version surfaces + parity + gap gate + daemon spec gates under FOCUSA_TEST_MODE.
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
STRICT=0
if [[ "${1:-}" == "--strict" ]]; then STRICT=1; fi

echo "=== local preflight: version surfaces ==="
# pick current stamped version if present, else Cargo
if [[ -f docs/current/.release-version-stamp ]]; then
  V="$(tr -d '[:space:]' < docs/current/.release-version-stamp)"
  TAG="v${V}"
else
  V="$(grep -m1 '^version' Cargo.toml | sed -E 's/.*"(.*)"/\1/')"
  TAG="v${V}"
fi
echo "checking $TAG (from $V)"
python3 scripts/verify-version-surfaces.py "$TAG" || { echo "FAIL verify-version-surfaces"; exit 1; }
node scripts/validate-docs-runtime-parity.mjs || { echo "FAIL docs/runtime parity"; exit 1; }
echo "version surfaces: PASS"

if [[ "$STRICT" -eq 1 ]]; then
  echo "=== local preflight: gap gate ==="
  bash tests/final_release_gap_gate.sh || { echo "FAIL final_release_gap_gate"; exit 1; }
  echo "gap gate: PASS"
  echo "=== local preflight: spec gates (FOCUSA_TEST_MODE) ==="
  # reuse CI script but keep data dir for inspection
  export FOCUSA_TEST_MODE="${FOCUSA_TEST_MODE:-1}"
  # run only fast gates if BUILD env says skip heavy compile
  if [[ "${PREFLIGHT_FAST:-0}" == "1" ]]; then
    echo "(fast mode: skip daemon build, run static gates only)"
    python3 tests/spec104_singleton_inventory_gate.py --closure
    python3 scripts/verify-version-surfaces.py "$TAG"
  else
    bash scripts/ci/run-spec-gates.sh
  fi
  echo "spec gates: PASS"
fi
echo "=== local preflight: done ==="
