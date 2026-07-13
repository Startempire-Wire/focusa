#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

bash tests/commit_message_policy_test.sh >/dev/null
bash tests/spec125_hlt_status_model_static_test.sh >/dev/null
bash tests/spec130_compaction_mission_packet_static_test.sh >/dev/null
bash tests/spec130_bounded_persistence_test.sh >/dev/null
bash tests/spec130_native_session_pressure_test.sh >/dev/null
bash tests/spec130_auto_compaction_test.sh >/dev/null
bash tests/spec82_low_resource_efficiency_static_test.sh >/dev/null
bash tests/spec94_response_size_and_metadata_contract_test.sh >/dev/null
bash tests/spec128_update_status_static_test.sh >/dev/null
bash tests/software_currency_inventory_static_test.sh >/dev/null

rg -q 'Status: implementation-complete' docs/125-mandatory-trajectory-nonlazy-hlt-pi-receipt-ontology-interlock-spec.md
rg -q 'Status: implementation-complete' docs/130-hlt-aware-compaction-mission-packet-and-bloatgaurd-context-firewall-spec.md
rg -q 'Status:\*\* implementation-complete' docs/82-focusa-memory-optimization-spec.md
rg -q 'Status:\*\* implementation-complete' docs/94-focusa-intent-preserving-memory-rpc-optimization-sow.md
rg -q 'Implementation-complete' docs/128-focusa-over-the-air-auto-update-and-dev-mode-license-spec.md
rg -q 'Release/deploy boundary: not authorized or implied' docs/134-focusa-pre-mvp-stg-closure-evidence.md

printf 'PASS: combined pre-MVP STG strictness/compaction/optimization/OTA/currency closure gate\n'
