#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
API="$ROOT/crates/focusa-api/src/routes/compaction.rs"
PI="$ROOT/apps/pi-extension/src/compaction.ts"
RECENT="$ROOT/crates/focusa-api/src/routes/turn_recent.rs"
MOD="$ROOT/crates/focusa-api/src/routes/mod.rs"
SERVER="$ROOT/crates/focusa-api/src/server.rs"
CLI="$ROOT/crates/focusa-cli/src/commands/compaction.rs"
MAIN="$ROOT/crates/focusa-cli/src/main.rs"

require() { grep -Fq "$2" "$1" || { echo "missing: $2 in $1" >&2; exit 1; }; }

require "$API" 'focusa.compaction_mission_packet.v1'
require "$API" '/v1/compaction/build'
require "$API" '/v1/compaction/packet/{packet_id}'
require "$API" '/v1/compaction/inspect/{packet_id}'
require "$API" '/v1/compaction/evaluate'
require "$API" '/v1/compaction/replay'
require "$API" '/v1/compaction/diff'
require "$API" 'focusa.compaction_fidelity_eval.v1'
require "$API" 'RESUME_SOURCES'
require "$API" 'COMP-CASCADE-001'
require "$API" 'resume_state'
require "$API" 'canonical": false'
require "$API" 'advisory": true'
require "$API" 'transcript_tail_as_authority'
require "$API" 'HltStatus::GenericDegraded'
require "$API" 'PACKET_CAP: usize = 64'
require "$API" 'read_recent_turns_bounded'
require "$API" 'recent_turn:'
require "$RECENT" 'pub(crate) fn read_recent_turns_bounded'
require "$MOD" 'pub mod compaction;'
require "$SERVER" '.merge(routes::compaction::router())'
require "$PI" 'buildCompactionMissionPacket("before_compaction")'
require "$PI" 'buildCompactionMissionPacket("after_compaction")'
require "$PI" 'renderCompactionMissionPacket(missionPacket)'
require "$PI" 'focusa.compaction_mission_packet.v1'
require "$CLI" 'CompactionCmd'
require "$CLI" '/v1/compaction/inspect/'
require "$CLI" '/v1/compaction/evaluate'
require "$CLI" '/v1/compaction/replay'
require "$CLI" '/v1/compaction/diff'
require "$CLI" '/v1/ecs/rehydrate/'
require "$CLI" 'RestoreContext'
require "$CLI" 'CompactionCmd::Why'
require "$MAIN" 'Compaction(commands::compaction::CompactionCmd)'

echo 'PASS: Spec 130 CompactionMissionPacket API and Pi wiring'
