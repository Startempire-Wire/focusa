# Spec 132 Installer Animation Baseline — 20260712T083423Z

## Recorded state

- Starting SHA: `8ba4d11035618cf00d22a49dc1dde63bb80cf21e`
- Baseline SHA: `19e7049b7672b66ddbc0036c344c10024c19bfd7`
- Default remote branch: `origin/main`
- Remote SHA after safe `git fetch --no-prune origin main`: `8ba4d11035618cf00d22a49dc1dde63bb80cf21e`
- HEAD/remote merge base: `8ba4d11035618cf00d22a49dc1dde63bb80cf21e`
- Workspace version: `0.9.94-dev`
- Dirty worktree at capture: yes; preserved without reset/stash/overwrite.

```text
 M .beads/issues.jsonl
 M .github/workflows/release.yml
?? docs/evidence/INSTALL_ANIMATION_BASELINE_20260712T083423Z.md
```

The `.beads/issues.jsonl` change records active Spec 132 decomposition/status. The pre-existing `.github/workflows/release.yml` modification is operator-session work and remains untouched by this baseline task. This evidence file is the only new content created by A1.

## Files changed from the required baseline to starting HEAD

Total: **60 files**.

```text
M	.beads/issues.jsonl
M	.github/workflows/release.yml
M	Cargo.lock
M	Cargo.toml
M	apps/menubar/README.md
M	apps/menubar/package-lock.json
M	apps/menubar/package.json
M	apps/menubar/src-tauri/Cargo.lock
M	apps/menubar/src-tauri/Cargo.toml
M	apps/menubar/src-tauri/src/main.rs
M	apps/menubar/src-tauri/tauri.conf.json
M	apps/menubar/src/lib/components/Settings.svelte
M	apps/pi-extension/src/awareness.ts
M	apps/pi-extension/src/compaction.ts
M	apps/pi-extension/src/session.ts
M	apps/pi-extension/src/state.ts
M	apps/pi-extension/src/tool-contracts.ts
M	apps/pi-extension/src/tools.ts
M	crates/focusa-api/src/routes/agent_capabilities.rs
M	crates/focusa-api/src/routes/device_pairing.rs
M	crates/focusa-api/src/routes/focus.rs
M	crates/focusa-api/src/routes/license.rs
M	crates/focusa-api/src/routes/preload.rs
M	crates/focusa-api/src/routes/project.rs
M	crates/focusa-api/src/routes/training.rs
M	crates/focusa-api/src/routes/trajectory.rs
M	crates/focusa-api/src/routes/workpoint.rs
M	crates/focusa-cli/src/commands/cleanup.rs
M	crates/focusa-cli/src/commands/dxux.rs
M	crates/focusa-cli/src/commands/help.rs
M	crates/focusa-cli/src/commands/hlt.rs
M	crates/focusa-cli/src/commands/install.rs
M	crates/focusa-cli/src/commands/project.rs
M	crates/focusa-cli/src/commands/tui.rs
M	crates/focusa-cli/src/commands/work_item.rs
M	crates/focusa-cli/src/main.rs
M	crates/focusa-core/src/types.rs
M	crates/focusa-core/src/work_item/lifecycle.rs
M	crates/focusa-core/src/work_item/policy.rs
M	crates/focusa-tui/src/main.rs
M	crates/focusa-tui/src/mission_control.rs
A	docs/132-focusa-installer-animated-terminal-experience-spec.md
M	scripts/bd-close-with-evidence
M	scripts/enforce_bd_closure_evidence.sh
M	scripts/install-focusa.sh
A	scripts/magic/focusa-pi-shell-reminder.sh
A	tests/spec109_agent_tools_memory_turn_runtime_test.sh
A	tests/spec111_pi_tool_inventory_reconcile_test.sh
A	tests/spec112_agent_context_bundle_test.sh
A	tests/spec112_pi_extension_archive_smoke_test.sh
A	tests/spec124_newcomer_root_help_test.sh
A	tests/spec124_project_dashboard_runtime_test.sh
A	tests/spec125_hlt_status_model_static_test.sh
A	tests/spec125_runtime_eval_test.sh
M	tests/spec128_update_runtime_test.sh
A	tests/spec92_cleanup_authorization_runtime_test.sh
M	tests/spec_focusa_yixp_tui_usage_static_test.sh
M	tests/spec_install_path_walkthrough_static_test.sh
M	tests/spec_install_rust_static_test.sh
M	tests/spec_install_smoke_integration_test.sh
```

## Installer contract changes discovered

- Workspace advanced from the Spec 132 baseline version `0.9.91-dev` to `0.9.94-dev`.
- Rust installer now binds asset resolution/download to the selected release tag instead of implicitly mixing release state.
- Installer tests gained local release-fixture support and expanded smoke/path walkthrough coverage.
- Host target handling was corrected for Intel macOS asset selection.
- Uninstall now removes the managed Pi extension; Spec 132 must preserve that exact retention/removal contract.
- Current installer-related surfaces changed across `Cargo.toml`, `install.rs`, `main.rs`, Bash bootstrapper, release workflow, and multiple static/runtime tests.
- The committed release workflow added Apple Developer ID credential import/notarization plumbing. The dirty worktree contains a separate unfinished policy edit for an explicitly unnotarized preview; Spec 132 does not authorize completing, discarding, releasing, or deploying that change.
- Existing requirements remain authoritative: Rust orchestration, thin bootstrapper, staged downloads, atomic stash/rollback, smoke-test gate, trust ordering, PATH idempotency, sibling TUI discovery, platform matrix, bundled Pi support, single JSON document, durable walkthrough, and post-Spec-112 session/preload/receipt behavior.

## Reconciliation without weakening Spec 132

Spec 132 is applied additively to current HEAD. Later installer fixes are treated as protected behavior, not reasons to omit animation, fallbacks, safety, tests, Pi migration, streamed progress, or proof. Implementation must integrate through typed presentation events while retaining the current orchestrator and regression guards. No dirty operator work was reset, stashed, overwritten, or incorporated. No tag, release, deploy, installer sync, or live-host mutation occurred.

## Current-head drift gate status

- HEAD and `origin/main` match at capture.
- Baseline comparison and changed-file inventory are complete.
- Full governing-surface reread and contract traceability continue under `focusa-slxpz.1.2`; this evidence must be updated if HEAD moves before merge.
