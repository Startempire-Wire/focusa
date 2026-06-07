#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="${ROOT_DIR:-/home/wirebot/focusa}"
cd "$ROOT_DIR"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

required_tests=(
  tests/spec102_multi_agent_ownership_board_test.sh
  tests/spec102_agent_handoff_quality_test.sh
  tests/spec102_proof_artifact_browser_test.sh
  tests/spec102_mutation_dry_run_preview_test.sh
  tests/spec102_undo_rollback_affordance_test.sh
  tests/spec102_trust_badges_per_surface_test.sh
  tests/spec102_agent_command_palette_test.sh
  tests/spec102_route_recommender_test.sh
  tests/spec102_stuck_loop_detector_test.sh
  tests/spec102_bead_review_mode_test.sh
  tests/spec102_notification_change_feed_test.sh
  tests/spec102_agent_safe_empty_states_test.sh
  tests/spec102_personalized_verbosity_profiles_test.sh
  tests/spec102_evidence_diffing_test.sh
  tests/spec102_recovery_playbooks_test.sh
)

for test_file in "${required_tests[@]}"; do
  [[ -x "$test_file" ]] || fail "missing executable Section 15 regression: $test_file"
done
pass "all Section 15 regression scripts are executable"

open_s15=$(bd --no-daemon list --parent focusa-pm2b --all --json --no-pager | jq '[.[] | select(.id != "focusa-pm2b.42") | select(.title|test("S15|Section 15")) | select(.status != "closed")] | length')
[[ "$open_s15" == "0" ]] || fail "Section 15 has non-suite open feature beads: $open_s15"
pass "all Section 15 feature beads are closed"

for test_file in "${required_tests[@]}"; do
  "$test_file"
done
pass "all Section 15 regressions pass together"

scripts/spec102-repair-report focusa-pm2b docs/evidence/SPEC102_REPAIR_REPORT_CURRENT.md >/tmp/spec102-section15-report.log
rg -F "missing_required_fields=0" /tmp/spec102-section15-report.log >/dev/null || fail "repair report incomplete"
closed_repairs=$(awk -F= '/closed_repairs=/{print $2}' /tmp/spec102-section15-report.log | tail -1)
[[ "${closed_repairs:-0}" -ge 35 ]] || fail "repair report closed_repairs too low: ${closed_repairs:-missing}"
pass "repair report has no missing required fields"

tests/spec102_prep_packet_enforcement_test.sh
tests/spec102_proof_matrix_enforcement_test.sh
tests/spec102_supersession_policy_test.sh
pass "Section 13.6 prep/proof/supersession gates pass"

echo "SPEC102 Section 15 acceptance suite: PASS"
