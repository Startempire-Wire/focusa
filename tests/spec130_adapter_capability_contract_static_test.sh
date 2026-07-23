#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MANIFEST="$ROOT_DIR/adapters/spec130-capability-manifests.json"
ROUTE="$ROOT_DIR/crates/focusa-api/src/routes/agent_capabilities.rs"
fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

[ -f "$MANIFEST" ] || fail "Spec130 adapter capability manifest missing"
jq -e '
  .schema == "focusa.adapter_capability_registry.v1" and
  (.registry_version | type == "string" and length > 0) and
  ([.adapters[].adapter] | sort) == ["claude", "codex", "opencode", "pi"] and
  all(.adapters[];
    (.manifest_version | type == "number") and
    (.measured_at | type == "string" and length > 0) and
    (.measured_against | type == "string" and length > 0) and
    (.tier | IN("tier_a", "tier_b", "tier_c", "tier_d")) and
    (.evidence_refs | type == "array" and length > 0) and
    (.limitations | type == "array" and length > 0) and
    ([
      .supports_compaction_hook,
      .supports_bounded_custom_entry,
      .supports_session_size_preflight,
      .supports_automatic_native_rollover,
      .supports_user_command_rollover,
      .supports_rpc_rollover,
      .supports_streaming_import,
      .supports_external_rehydrate,
      .supports_preload_receipt
    ] | all(type == "boolean"))
  )
' "$MANIFEST" >/dev/null || fail "registry schema or typed fields are invalid"
pass "registry has four typed, versioned, evidence-backed manifests"

jq -e '
  all(.adapters[];
    if .tier == "tier_a" then .supports_automatic_native_rollover
    elif .tier == "tier_b" then
      (.supports_automatic_native_rollover | not) and
      (.supports_user_command_rollover or .supports_rpc_rollover)
    elif .tier == "tier_c" then
      (.supports_automatic_native_rollover | not) and
      (.supports_user_command_rollover | not) and
      (.supports_rpc_rollover | not) and
      .supports_preload_receipt
    else
      (.supports_automatic_native_rollover | not) and
      (.supports_user_command_rollover | not) and
      (.supports_rpc_rollover | not)
    end
  )
' "$MANIFEST" >/dev/null || fail "adapter tier overclaims native rollover capability"
pass "tier invariants reject stronger-than-measured rollover claims"

jq -e '
  .adapters[] | select(.adapter == "pi") |
  .tier == "tier_b" and
  .supports_compaction_hook and
  .supports_bounded_custom_entry and
  .supports_session_size_preflight and
  (.supports_automatic_native_rollover | not) and
  .supports_user_command_rollover and
  .supports_rpc_rollover and
  .supports_streaming_import and
  .supports_external_rehydrate and
  .supports_preload_receipt
' "$MANIFEST" >/dev/null || fail "Pi posture does not match measured command/RPC boundary"
rg -F 'pi.on("session_before_compact"' "$ROOT_DIR/apps/pi-extension/src/compaction.ts" >/dev/null || fail "Pi compaction hook evidence missing"
rg -F 'pi.registerCommand("focusa-rollover"' "$ROOT_DIR/apps/pi-extension/src/commands.ts" >/dev/null || fail "Pi rollover command evidence missing"
rg -F 'await ctx.newSession({' "$ROOT_DIR/apps/pi-extension/src/commands.ts" >/dev/null || fail "Pi command-context replacement evidence missing"
rg -F 'pi_launch_migration.rs' "$ROOT_DIR/crates/focusa-cli/src/commands/pi_launch.rs" >/dev/null || fail "Pi streaming migration module evidence missing"
pass "Pi Tier B posture matches compaction, command, preflight, and migration surfaces"

jq -e '
  .adapters[] | select(.adapter == "claude") |
  .tier == "tier_d" and
  .supports_compaction_hook and
  .supports_bounded_custom_entry and
  ([
    .supports_session_size_preflight,
    .supports_automatic_native_rollover,
    .supports_user_command_rollover,
    .supports_rpc_rollover,
    .supports_streaming_import,
    .supports_external_rehydrate,
    .supports_preload_receipt
  ] | all(. == false))
' "$MANIFEST" >/dev/null || fail "Claude posture overclaims transfer capability"
rg -F 'PreCompact' "$ROOT_DIR/adapters/claude-code/bin/focusa-claude-code-hook.sh" >/dev/null || fail "Claude PreCompact evidence missing"
! rg -n 'newSession|switchSession|streaming.import|preload.receipt' "$ROOT_DIR/adapters/claude-code" >/dev/null || fail "Claude manifest must be remeasured after transfer support appears"
pass "Claude posture exposes bounded hooks without unsupported transfer claims"

for adapter in codex opencode; do
  jq -e --arg adapter "$adapter" '
    .adapters[] | select(.adapter == $adapter) |
    .tier == "tier_d" and
    ([
      .supports_compaction_hook,
      .supports_bounded_custom_entry,
      .supports_session_size_preflight,
      .supports_automatic_native_rollover,
      .supports_user_command_rollover,
      .supports_rpc_rollover,
      .supports_streaming_import,
      .supports_external_rehydrate,
      .supports_preload_receipt
    ] | all(. == false))
  ' "$MANIFEST" >/dev/null || fail "$adapter posture must remain observe-only without a dedicated adapter"
  [ ! -d "$ROOT_DIR/adapters/$adapter" ] || fail "$adapter adapter exists; remeasure its capability manifest"
done
pass "Codex and OpenCode remain conservative until dedicated adapters exist"

while IFS= read -r evidence_ref; do
  evidence_path="${evidence_ref%%#*}"
  [ -e "$ROOT_DIR/$evidence_path" ] || fail "missing evidence ref: $evidence_ref"
done < <(jq -r '.adapters[].evidence_refs[]' "$MANIFEST")
rg -F '/v1/agent/adapter-capabilities' "$ROUTE" >/dev/null || fail "typed adapter registry route missing"
pass "all evidence refs resolve and the registry is published"

echo "spec130 adapter capability contract static test: PASS"
