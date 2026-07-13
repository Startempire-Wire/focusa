#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
API="$ROOT/crates/focusa-api/src/routes/compaction.rs"
PI="$ROOT/apps/pi-extension/src/compaction.ts"
MOD="$ROOT/crates/focusa-api/src/routes/mod.rs"
SERVER="$ROOT/crates/focusa-api/src/server.rs"

require() { grep -Fq "$2" "$1" || { echo "missing: $2 in $1" >&2; exit 1; }; }

require "$API" 'focusa.compaction_mission_packet.v1'
require "$API" '/v1/compaction/build'
require "$API" '/v1/compaction/packet/{packet_id}'
require "$API" '/v1/compaction/inspect/{packet_id}'
require "$API" '/v1/compaction/evaluate'
require "$API" '/v1/compaction/replay'
require "$API" '/v1/compaction/diff'
require "$API" 'focusa.compaction_fidelity_eval.v1'
require "$API" 'canonical": false'
require "$API" 'advisory": true'
require "$API" 'transcript_tail_as_authority'
require "$API" 'HltStatus::GenericDegraded'
require "$API" 'PACKET_CAP: usize = 64'
require "$MOD" 'pub mod compaction;'
require "$SERVER" '.merge(routes::compaction::router())'
require "$PI" 'buildCompactionMissionPacket("before_compaction")'
require "$PI" 'buildCompactionMissionPacket("after_compaction")'
require "$PI" 'renderCompactionMissionPacket(missionPacket)'
require "$PI" 'focusa.compaction_mission_packet.v1'

echo 'PASS: Spec 130 CompactionMissionPacket API and Pi wiring'
