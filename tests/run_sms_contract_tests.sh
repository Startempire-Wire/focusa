#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
if [[ "$(id -u)" == 0 ]]; then
  exec /usr/local/bin/as-user wirebot "$ROOT/tests/run_sms_contract_tests.sh"
fi
cd "$ROOT"
bun scripts/generate-agent-capability-descriptors.ts --check
bun scripts/generate-agent-tool-docs.ts --check
bun tests/spec141_agent_conformance_test.ts
python3 tests/spec138_operation_client_parity_test.py
python3 tests/spec152f_cross_presenter_parity_test.py
