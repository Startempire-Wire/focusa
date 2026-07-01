#!/usr/bin/env bash
# Spec104 runtime integrity proof for Pi typed scope authority helpers.
# Static singleton audits are not enough: these proofs exercise the runtime
# paths that build CurrentAsk scope verdicts, project-root inference, and the
# project-switch ledger without recursive scope-store access.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

run_tsx() {
  local test_file="$1"
  echo "--- Spec104 Pi runtime: ${test_file}"
  npx --yes tsx "$test_file"
}

run_tsx tests/scope_arbitration_runtime_test.mts
run_tsx tests/current_ask_project_override_runtime_test.mts
run_tsx tests/pi_project_root_inference_test.mts
run_tsx tests/pi_session_project_switch_ledger_runtime_test.mts

echo "Spec104 Pi runtime scope integrity: PASS"
