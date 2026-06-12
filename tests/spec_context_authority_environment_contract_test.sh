#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

CARGO_BIN="${CARGO:-cargo}"
CONTRACT="${TMPDIR:-/tmp}/focusa-env-contract-test-$$.json"
trap 'rm -f "$CONTRACT" /tmp/focusa-env-contract-out-$$.json' EXIT

"$CARGO_BIN" run -q -p focusa-cli --locked -- --json env contract init \
  --path "$CONTRACT" \
  --role live_build_host \
  --project-root /home/wirebot/focusa \
  --owner wirebot \
  --machine-kind vps \
  --preferred-source local_repo_build > /tmp/focusa-env-contract-out-$$.json

jq -e '.schema == "focusa.environment_contract.v1"' /tmp/focusa-env-contract-out-$$.json >/dev/null
jq -e '.install_role == "live_build_host"' /tmp/focusa-env-contract-out-$$.json >/dev/null
jq -e '.binary_policy.release_asset_install_allowed == false' /tmp/focusa-env-contract-out-$$.json >/dev/null
jq -e '.binary_policy.local_build_required == true' /tmp/focusa-env-contract-out-$$.json >/dev/null

"$CARGO_BIN" run -q -p focusa-cli --locked -- --json env contract show --path "$CONTRACT" > /tmp/focusa-env-contract-out-$$.json
jq -e '.schema == "focusa.environment_contract.v1" and .owner == "wirebot"' /tmp/focusa-env-contract-out-$$.json >/dev/null

echo "environment contract test passed"
