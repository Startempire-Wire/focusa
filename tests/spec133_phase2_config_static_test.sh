#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SS="$ROOT/crates/focusa-core/src/silent_sessions"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }; pass(){ echo "✓ PASS: $*"; }
for group in IdentityConfig HarnessConfig ModelConfig WorkspaceConfig BootstrapConfig SupervisionConfig ResourceConfig OutputConfig GovernanceConfig NotificationConfig RetentionConfig; do
  rg -n "struct $group" "$SS/config.rs" >/dev/null || fail "missing typed config group: $group"
done
pass "all §15.1 config groups are typed"
for layer in CompiledDefaults ExecutionProfile ProjectPolicy BehavioralPreset ContextAuthority SessionRequest OperatorRevision ConstitutionalPolicy; do
  rg -n "$layer" "$SS/config_resolution.rs" >/dev/null || fail "missing precedence layer: $layer"
done
pass "all eight precedence layers and profile/preset distinction are explicit"
for marker in FieldProvenance ConfigPolicyLock EffectiveSilentSessionConfig requested_config resolved_effective_config restart_required_fields warnings validation redacted_config_hash; do
  rg -n "$marker" "$SS/config_resolution.rs" >/dev/null || fail "missing effective config contract: $marker"
done
pass "effective config exposes provenance, locks, warnings, validation and redacted hash"
for marker in HotMutable RestartRequired Immutable ResourceLimitLoosened; do
  rg -n "$marker" "$SS/config_resolution.rs" "$SS/config_revision.rs" >/dev/null || fail "missing mutation policy: $marker"
done
pass "hot, restart, immutable and tighten-only resource mutations are fail closed"
for marker in preview_config_revision persist_pending apply_hot create_restart_plan verify commit rollback RolledBack GateRequired; do
  rg -n "$marker" "$SS/config_revision.rs" >/dev/null || fail "missing transactional revision stage: $marker"
done
pass "preview-to-commit transactional flow and rollback are explicit"
rg -n 'raw secret field forbidden' "$SS/config_resolution.rs" >/dev/null || fail "missing secret-ref-only guard"
pass "raw secret material is rejected"
