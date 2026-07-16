#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PRESSURE="$ROOT/apps/pi-extension/src/session-pressure.ts"
SESSION="$ROOT/apps/pi-extension/src/session.ts"
STATE="$ROOT/apps/pi-extension/src/state.ts"
COMMANDS="$ROOT/apps/pi-extension/src/commands.ts"
COMPACTION="$ROOT/apps/pi-extension/src/compaction.ts"

rg -q 'focusa\.native_session_pressure\.v1' "$PRESSURE"
rg -q 'native_segment_soft_bytes' "$PRESSURE"
rg -q 'native_segment_hard_bytes' "$PRESSURE"
rg -q 'native_startup_migration_bytes' "$PRESSURE"
rg -q 'oversized_at_start' "$PRESSURE"
rg -q 'refuse_full_load' "$PRESSURE"
rg -q 'SAMPLE_LIMIT = 2_048' "$PRESSURE"
rg -q 'migrateNativeSessionBounded' "$PRESSURE"
rg -q 'focusa\.native_session_migration_manifest\.v1' "$PRESSURE"
rg -q 'createReadStream' "$PRESSURE"
rg -q 'resume_immutable_source' "$PRESSURE"
rg -q 'linkSync' "$PRESSURE"
rg -q 'getSessionFile' "$SESSION"
rg -q 'refreshNativeSessionPressure' "$SESSION"
rg -q 'lastNativeSessionPressure' "$STATE"
rg -Fq 'registerCommand("focusa-rollover"' "$COMMANDS"
rg -Fq 'execute [output-dir]' "$COMMANDS"
rg -q 'prepareCompactionRollover' "$COMMANDS"
rg -q 'migrateNativeSessionBounded' "$COMMANDS"
rg -q 'typed verified project/workstream scope required' "$COMMANDS"
rg -q 'export async function prepareCompactionRollover' "$COMPACTION"

cd "$ROOT"
npx --yes tsx tests/spec130_native_session_pressure_runtime_test.mts
printf 'PASS: Spec 130 native session pressure static/runtime contract\n'
