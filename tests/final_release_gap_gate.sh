#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

fail() { printf '✗ FINAL RELEASE GAP: %s\n' "$*" >&2; exit 1; }
pass() { printf '✓ %s\n' "$*"; }
require() {
  local pattern="$1" file="$2" message="$3"
  rg -n "$pattern" "$file" >/dev/null || fail "$message"
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
require 'ctx\.reload\(\)' apps/pi-extension/src/ota-activation.ts 'Pi OTA reload is automatic'
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

# Worktree/authority aggregate proof.
bash tests/authority_scope_static_test.sh
bash tests/spec96_project_identity_quorum_static_test.sh
python3 tests/spec104_mismatch_semantic_static_test.py
bash tests/spec130_rotating_continuity_transfer_static_test.sh
pass 'worktree and authority gates'

# Cache-safe prefix stabilization and automatic Pi activation runtime proof.
(
  cd apps/pi-extension
  npm run test:cache-safe-context
  npm run test:ota-activation
  npm run test:spec104-attachment
)
pass 'cache miss mitigation and Pi OTA activation gates'

# Compaction provider-overflow, retry, automatic rollover, persistence, crash, and rotating-agent proof.
require 'pi\.sendUserMessage\("/focusa-rollover execute"' apps/pi-extension/src/auto-compaction.ts 'transport retry exhaustion queues governed rollover automatically'
bash tests/spec130a_proactive_compaction_runtime_test.sh
bash tests/spec130a_persistence_actor_static_test.sh
python3 tests/spec130a_release_stress_static_test.py
npx --yes tsx tests/spec130a_release_stress_runtime_test.mts
pass 'compaction and session-recovery gates'

printf 'FINAL RELEASE GAP GATE: PASS\n'
