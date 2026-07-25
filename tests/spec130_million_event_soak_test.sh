#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TEST="$ROOT/tests/spec130_million_event_soak_runtime_test.mts"

rg -q 'TOTAL_EVENTS = 1_000_000' "$TEST"
rg -q 'TOTAL_CYCLES = 10_000' "$TEST"
rg -q 'PHYSICAL_SEGMENTS = 10' "$TEST"
rg -q 'unchangedStateSuppressions === TOTAL_EVENTS - TOTAL_CYCLES' "$TEST"
rg -q 'replaySlopeBytesPerSegment' "$TEST"
rg -q 'required_ref_loss: 0' "$TEST"

cd "$ROOT"
npx --yes tsx tests/spec130_million_event_soak_runtime_test.mts
