#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
if [[ "$(id -u)" == 0 ]]; then
  exec /usr/local/bin/as-user wirebot "$ROOT/tests/run_sms_pair_once_tests.sh"
fi
cd "$ROOT"
python3 tests/sms_appliance_checkpoint_test.py
python3 tests/sms_broker_contract_test.py
python3 tests/sms_pair_once_lifecycle_static_test.py
python3 tests/sms_supervisor_lifecycle_test.py
python3 tests/sms_credential_provision_test.py
python3 -m py_compile scripts/focusa-sms-appliance.py scripts/focusa-sms-supervisor.py scripts/focusa-google-messages-broker.py
node --check scripts/focusa-sms-ready-probe.mjs
bash -n scripts/install-focusa-sms-appliance-service.sh
rustfmt --edition 2021 --check crates/focusa-cli/src/commands/sms.rs
apps/pi-extension/node_modules/.bin/tsc --noEmit -p apps/pi-extension/tsconfig.json
printf 'sms pair-once targeted gates: passed\n'
