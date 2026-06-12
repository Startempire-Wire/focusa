#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FIXTURE="$ROOT/tests/golden/context_authority/live_build_host_pairing_release_asset_block.json"
cd "$ROOT"

CARGO_BIN="${CARGO:-cargo}"

"$CARGO_BIN" test -p focusa-cli blocks_release_asset_install_on_live_build_host_during_pairing --locked
"$CARGO_BIN" run -p focusa-cli --locked -- --json action preflight \
  --current-ask "$(jq -r .facts.current_ask "$FIXTURE")" \
  --kind "$(jq -r .proposed_action.kind "$FIXTURE")" \
  --target "$(jq -r .proposed_action.target "$FIXTURE")" \
  --source "$(jq -r .proposed_action.source "$FIXTURE")" \
  --install-role "$(jq -r .facts.install_role "$FIXTURE")" \
  --project-root "$(jq -r .facts.project_root "$FIXTURE")" \
  --repo-version "$(jq -r .facts.repo_version "$FIXTURE")" \
  --cli-version "$(jq -r .facts.cli_version "$FIXTURE")" \
  --daemon-version "$(jq -r .facts.daemon_version "$FIXTURE")" > /tmp/focusa-context-authority-preflight.json

jq -e '.schema == "focusa.operational_context_gate.v1"' /tmp/focusa-context-authority-preflight.json >/dev/null
jq -e '.verdict == "block"' /tmp/focusa-context-authority-preflight.json >/dev/null
jq -e '.conflicts[] | select(.class == "consumer_install_path_conflicts_with_live_build_host")' /tmp/focusa-context-authority-preflight.json >/dev/null
jq -e '.safe_alternative | contains("local Focusa repo")' /tmp/focusa-context-authority-preflight.json >/dev/null

echo "context authority preflight golden test passed"
