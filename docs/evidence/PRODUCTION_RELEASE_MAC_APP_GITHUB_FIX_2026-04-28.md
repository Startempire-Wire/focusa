# Production Release / Mac App / GitHub Fix — 2026-04-28

## Scope

Operator requested failed GitHub releases fixed, Mac app brought current, production daemon rebuilt/restarted, release pushed, and residual junk cleaned.

## Fixes

- Fixed workspace clippy failures blocking GitHub CI/release:
  - `crates/focusa-cli/src/commands/metacognition.rs` now uses a loop args struct instead of a 14-argument function.
  - Clippy auto-fixes applied to API route files.
  - `crates/focusa-api/src/routes/capabilities.rs` uses `.to_vec()` instead of `iter().cloned().collect()`.
  - `apps/pi-extension/src/compaction.ts` restores pending-gated compaction auto-resume retry wiring with no hard max-attempt marker.
  - `crates/focusa-api/src/routes/ontology.rs` now always projects a safe `blocks_execution_of` link for destructive-confirmation requirements.
- Updated Mac menubar app version to `0.9.9`:
  - `apps/menubar/package.json`
  - `apps/menubar/package-lock.json`
  - `apps/menubar/bun.lock`
  - `apps/menubar/src-tauri/Cargo.toml`
  - `apps/menubar/src-tauri/tauri.conf.json`
  - `apps/menubar/src/lib/components/Settings.svelte`
- Updated stale Mac app runtime surfaces to current Focusa core/API:
  - `apps/menubar/src/lib/api.ts` centralizes local daemon API access.
  - `apps/menubar/src/lib/stores/runtime.svelte.ts` tracks health, Workpoint, Work-loop, ontology contract count/version, and recent events.
  - `apps/menubar/src/routes/+page.svelte` now polls current `/v1/health`, `/v1/ontology/tool-contracts`, `/v1/workpoint/current`, `/v1/work-loop/status`, and `/v1/events/recent` alongside `/v1/state/dump`.
  - `apps/menubar/src/lib/stores/focus-canvas.svelte.ts` now loads live focus stack/events instead of mock demo frames.
  - `apps/menubar/src/routes/canvas/+page.svelte` now displays live timeline data and keeps replay read-only.
- Updated GitHub release notes template to use the active tag variable instead of stale `v0.2.10` examples.

## Local validation

Frontend/Mac app checks:

```bash
cd apps/menubar && bun install && bun run check && bun run build
```

Result: passed; Svelte reported existing accessibility warnings only. After stale-surface update, the app still builds against current live daemon endpoints.

Rust/tests:

```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
./scripts/ci/run-spec-gates.sh
```

Result: passed.

Spec90/Spec91 live proof:

```bash
node scripts/validate-focusa-tool-contracts.mjs
node scripts/prove-focusa-tool-contracts-live.mjs --safe-fixtures
```

Result:

```text
Spec90 tool contracts: passed
tools=43 contracts=43
Spec91 live tool contract proof: passed
payload_equal=true
fixture_checks=workpoint:passed,work_loop:passed,tree_lineage:passed,metacognition:passed,focus_state:passed
```

## Production actions

Completed after commit:

```bash
cargo build --release --bins
systemctl restart focusa-daemon
systemctl is-active focusa-daemon
readlink -f /proc/$(systemctl show -p MainPID --value focusa-daemon)/exe
curl -sS http://127.0.0.1:8787/v1/health | jq .
curl -sS http://127.0.0.1:8787/v1/ontology/tool-contracts | jq '.version, (.contracts|length)'
node scripts/prove-focusa-tool-contracts-live.mjs --safe-fixtures
```

Result:

```text
active
/home/wirebot/focusa/target/release/focusa-daemon
{"ok":true,"version":"0.1.0"}
"spec90.tool_contracts.v1"
43
Spec91 live tool contract proof: passed
fixture_checks=workpoint:passed,work_loop:passed,tree_lineage:passed,metacognition:passed,focus_state:passed
```

A later local health timeout after spec-gate daemon churn was resolved by restarting the production daemon and re-running Spec91 proof; `/v1/health` and all safe fixtures passed.

Remaining release action: push commit/tag and wait for GitHub CI/release workflows to pass, then remove temporary local proof logs.
