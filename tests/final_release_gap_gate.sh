#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

fail() { printf '✗ FINAL RELEASE GAP: %s\n' "$*" >&2; exit 1; }
pass() { printf '✓ %s\n' "$*"; }
search() {
  if command -v rg >/dev/null 2>&1; then rg -n "$1" "$2"; else grep -En "$1" "$2"; fi
}
require() {
  local pattern="$1" file="$2" message="$3"
  search "$pattern" "$file" >/dev/null || fail "$message"
  pass "$message"
}
forbid() {
  local pattern="$1" file="$2" message="$3"
  if search "$pattern" "$file" >/dev/null; then fail "$message"; fi
  pass "$message"
}

# All-surface OTA: policy, atomic local promotion, delegated signed macOS
# activation, automatic Pi runtime reload, rollback, and release artifacts.
for surface in cli daemon tui pi_extension menubar installer; do
  require "${surface}: enabled" crates/focusa-core/src/update.rs "OTA policy includes ${surface}"
done
require 'execute_verified_apply\(&apply\.plan\)' crates/focusa-cli/src/commands/update.rs 'verified OTA executor is active'
require '"would_update" \| "would_install"' crates/focusa-cli/src/commands/update.rs 'binary and installer assets are atomically promoted'
require '"would_update_package" \| "would_install_package"' crates/focusa-cli/src/commands/update.rs 'Pi extension package is atomically promoted'
require 'registerAutomaticOtaActivation' apps/pi-extension/src/index.ts 'Pi OTA activation is registered'
require 'reloadWhenIdle' apps/pi-extension/src/ota-activation.ts 'Pi OTA uses non-conversational safe-idle reload when supported'
require 'process_start' apps/pi-extension/src/ota-activation.ts 'Pi OTA safely activates on natural process start fallback'
forbid 'sendUserMessage|focusa-activate-updated-extension' apps/pi-extension/src/ota-activation.ts 'Pi OTA activation must not inject conversation'
require 'pi-extension-activation-receipt' apps/pi-extension/src/ota-activation.ts 'Pi OTA activation receipt is durable'
require 'downloadAndInstall' apps/menubar/src/lib/updater.ts 'menubar signed update installs automatically'
require 'await relaunch\(\)' apps/menubar/src/lib/updater.ts 'menubar relaunches automatically'
require 'focusa\.menubar\.ota\.activation' apps/menubar/src/lib/updater.ts 'menubar activation receipt is durable'
require 'rollback_promoted_parts' crates/focusa-cli/src/commands/update.rs 'OTA rollback restores promoted local surfaces'
require 'focusa-pi-extension-\$\{VERSION\}\.tar\.gz|focusa-pi-extension-' .github/workflows/release.yml 'release packages Pi extension'
require 'focusa-installer-|install-focusa\.sh' .github/workflows/release.yml 'release packages installer'
require 'latest\.json|updater' .github/workflows/release.yml 'release publishes signed menubar updater metadata'
require 'focusa-daemon' .github/workflows/release.yml 'release packages daemon/API surface'
require 'focusa-tui' .github/workflows/release.yml 'release packages TUI surface'
bash tests/spec132_public_uninstall_preservation_test.sh
bash tests/spec132_public_bootstrap_dry_run_static_test.sh
bash tests/installer_explicit_target_alias_test.sh
bash tests/spec114_public_benchmark_flywheel_static_test.sh
python3 tests/spec114_observatory_ui_static_test.py
pass 'public uninstall preserves user data unless purge is explicit'

# Every Focusa Pi tool, skill, runbook, machine projection, and public agent
# entry point must remain one-to-one and release-current.
python3 scripts/generate-agent-skills.py --check
bun scripts/generate-agent-capability-descriptors.ts --check
bun scripts/generate-agent-tool-docs.ts --check
bun tests/spec141_agent_conformance_test.ts
python3 tests/spec141_agent_first_tool_audit_test.py
bash tests/spec129_agent_docs_surface_static_test.sh
pass 'all-Pi-tool, Agent Card, skill/runbook, and onboarding documentation gates'

# Worktree/authority aggregate proof. Provider status never substitutes for
# technical evidence; administrative closure replays fail closed.
python3 scripts/reduce-locked-release-technical-closure.py --check
python3 tests/165_focusa_locked_release_technical_closure_reducer_test.py
python3 tests/166_focusa_locked_release_candidate_ancestry_test.py
python3 tests/167_focusa_locked_release_governance_receipt_test.py
python3 tests/check_workset_evidence_integrity.py
python3 tests/168_focusa_windows_native_ota_workflow_test.py
python3 tests/169_focusa_rel4_candidate_artifact_workflow_test.py
bash tests/authority_scope_static_test.sh
bash tests/spec96_project_identity_quorum_static_test.sh
python3 tests/spec104_mismatch_semantic_static_test.py
bash tests/spec130_rotating_continuity_transfer_static_test.sh
pass 'worktree and authority gates'

# Cache-safe prefix stabilization and automatic Pi activation runtime proof.
# Run locally with the routed toolchain bypassed (the remote build host does not
# carry the pi-extension node_modules).
(
  cd apps/pi-extension
  FOCUSA_ROUTE_DRY_RUN=1 npm run test:cache-safe-context
  FOCUSA_ROUTE_DRY_RUN=1 npm run test:ota-activation
  FOCUSA_ROUTE_DRY_RUN=1 npm run test:spec104-attachment
  FOCUSA_ROUTE_DRY_RUN=1 npm run test:unbound-project
)
pass 'cache miss mitigation and Pi OTA activation gates'

# Compaction provider-overflow, native recovery, persistence, crash, and rotating-agent proof.
forbid 'pi\.sendUserMessage\("/focusa-rollover execute"' apps/pi-extension/src/auto-compaction.ts 'transport retry exhaustion must not auto-queue rollover'
bash tests/spec130a_proactive_compaction_runtime_test.sh
bash tests/spec130a_persistence_actor_static_test.sh
python3 tests/spec130a_release_stress_static_test.py
npx --yes tsx tests/spec130a_release_stress_runtime_test.mts
pass 'compaction and session-recovery gates'

python3 tests/spec145_canonical_release_cycle_static_test.py
python3 tests/spec146_release_intelligence_workflow_gate.py
python3 ./tests/spec137_138_full_conformance_invocation_test.py
python3 ./tests/run_spec137_138_full_conformance_gates.py
python3 scripts/generate-spec150-complete-feature-ledger.py --check
python3 scripts/generate-cross-spec-tool-grounding-matrix.py --check
python3 scripts/audit-cross-spec-reality-grounding.py
pass 'canonical release kernel, cross-spec reality grounding, tool/runbook awareness, and architecture gates'

printf 'FINAL RELEASE GAP GATE: PASS\n'
