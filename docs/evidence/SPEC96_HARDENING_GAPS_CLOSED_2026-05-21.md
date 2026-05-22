# Spec96 Hardening Gaps Closed — 2026-05-21

Operator directive: close all Spec96 hardening gaps without waiting.

## Closure summary

Spec96 hardening is closed to the current acceptance surface: all Spec96 runtime/static/golden shell gates pass, tool contracts pass, TypeScript compiles, Rust API checks, and the live daemon was rebuilt/restarted from the patched binary.

## Final repairs in this closure slice

- Post-compaction evidence tools now carry explicit `project_root`, `session_id`, and `continuity_id` through trajectory clarity checks and `FocusaSessionIdentity` instead of relying on ambient `/root` cwd (`apps/pi-extension/src/tools.ts:1042`, `apps/pi-extension/src/state.ts:1131`).
- Workpoint evidence/checkpoint dispatch uses nonblocking `try_send` under LowMem and returns a bounded `resource_exhausted` safe-retry envelope if the daemon command channel is saturated (`crates/focusa-api/src/routes/workpoint.rs:946`).
- `/v1/status/deep` moves persistence/process diagnostics to `spawn_blocking`, so cold status cannot hold hot `/v1/health` callers hostage (`crates/focusa-api/src/routes/session.rs:173`).
- `/v1/work-loop/status/deep` copies bounded WorkLoop state before awaiting cold worktree diagnostics, so work-loop deep status cannot hold hot health/status readers hostage (`crates/focusa-api/src/routes/work_loop.rs:2068`).

## Gap map from the critical re-audit

| Re-audit gap family | Closure evidence |
|---|---|
| unsafe post-compaction `/root` carryover | explicit scoped session identity + Workpoint resume hard gates; `tests/spec96_broad_root_scope_isolation_static_test.sh`; `tests/spec96_compaction_resume_injection_v2_static_test.sh` |
| ProjectIdentity / FocusaSessionIdentity integration | `tests/spec96_project_identity_quorum_static_test.sh`; `tests/spec96_session_identity_envelope_static_test.sh` |
| durable Trajectory lifecycle / clarity gate | `tests/spec96_trajectory_reducer_lifecycle_static_test.sh`; `tests/spec96_trajectory_clarity_gate_static_test.sh` |
| LowMem Focus Slice / tool affordances | `tests/spec96_lowmem_focus_slice_wpv2_static_test.sh`; `tests/spec96_tool_affordance_catalog_golden_eval_test.sh` |
| traverse schema / bounded surfaces | `tests/spec96_traverse_schema_runtime_test.sh`; `tests/spec96_traverse_adapters_static_test.sh`; `tests/spec96_traversal_budget_golden_eval_test.sh` |
| work-loop health/deep route split | `tests/spec96_work_loop_route_split_runtime_test.sh`; `tests/spec96_work_loop_route_split_static_test.sh` |
| CLI parity | `tests/spec96_cli_parity_static_test.sh` |
| ResourceMode preflight/hysteresis/full-payload gating | `tests/spec96_resource_mode_envelope_static_test.sh`; `tests/spec96_resource_mode_hysteresis_static_test.sh`; `tests/spec96_lowmem_full_payload_gate_static_test.sh` |
| Workpoint/evidence LowMem timeout | `tests/spec96_workpoint_evidence_lowmem_timeout_static_test.sh`; live proof `/tmp/spec96-lowmem-evidence-link-final-proof.out` |
| static false-positive guardrails | `tests/spec96_static_false_positive_guard_test.sh`; `tests/spec96_focus_slice_runtime_injection_test.sh` |

## Validation transcript handles

- `apps/pi-extension`: `npx tsc --noEmit` — pass.
- Rust API: `CARGO_TARGET_DIR=/tmp/focusa-cargo-target /root/.cargo/bin/cargo check -p focusa-api` — pass.
- Tool registry: `node scripts/validate-focusa-tool-contracts.mjs` — pass, 58 tools / 58 contracts.
- All `tests/spec96_*_test.sh` except cargo-backed status split were run as `wirebot`; all completed with `PASS` after the WorkLoop deep-route repair.
- Cargo-backed status split was run with `CARGO_BIN=/root/.cargo/bin/cargo CARGO_TARGET_DIR=/tmp/focusa-status-test tests/spec96_status_hot_deep_split_runtime_test.sh`; it passed and wrote build artifacts outside `/home`.
- Live daemon restarted from `target/release/focusa-daemon`; `/v1/health` returned HTTP 200 in ~1ms after warmup.
- ResourceMode restored to `normal forced=false` after LowMem proofs.

## Operational posture

Controlled alpha/dogfood is acceptable after agents reload the Pi extension. Broad GTM remains a separate product/support decision, not a Spec96 hardening gap.

## Emergency follow-up: Utility Card cross-session leak

Operator observed another Pi session receiving the Spec96 LowMem Utility Card. Root cause: Pi `ensurePiFrame()` used unsafe-cwd recovery that adopted the daemon global active Workpoint, mutating that other session to the Spec96 `project_root + continuity_id`; the Utility Card then truthfully matched the contaminated in-memory scope.

Fix:

- Unsafe cwd now clears scoped Workpoint/continuity and returns no frame instead of adopting the daemon global active Workpoint (`apps/pi-extension/src/state.ts:1229`).
- Scoped Workpoint packets are stamped with the current Pi `sessionFrameKey` when loaded into the extension; Utility Card scope rejects mismatched Pi session stamps or mismatched packet `session_id` when no stamp exists (`apps/pi-extension/src/state.ts:1243`).
- Utility Card no longer prints an unverified stale continuity id when no scoped Workpoint is verified (`apps/pi-extension/src/awareness.ts:12`).
- Runtime test now proves unsafe `/root` cannot display or adopt the Spec96 LowMem mission/continuity (`tests/utility_card_session_isolation_test.mts:32`).

Validation:

- `apps/pi-extension npx tsc --noEmit` — pass.
- `tests/spec96_utility_card_session_isolation_static_test.sh` — pass.
- `tests/spec96_broad_root_scope_isolation_static_test.sh` — pass.
- `tests/spec96_workpoint_post_compaction_resume_static_test.sh` — pass.
- `tests/spec96_compaction_resume_injection_v2_static_test.sh` — pass.
- `tests/spec96_scope_recovery_feedback_static_test.sh` — pass.
- `node scripts/validate-focusa-tool-contracts.mjs` — pass, 58/58.

## Emergency follow-up 2: Pi Task/session title cross-session leak

Operator observed the `Pi Task` from this session appearing in another Pi session. Root cause: `session_start` and `session_switch` reset only selected fields, then restored `focusa-state` entries before checking `entry.data.sessionId`; frame title/goal/current ask could be copied from a different Pi session even though Workpoint adoption was later rejected.

Fix:

- Added `resetPiSessionScopedState()` to clear all session-scoped singleton state at every `session_start` and `session_switch`: current ask, frame title/goal, Workpoint packet/summary, continuity id, caches, compaction fields, local shadows, telemetry rings, and WBM flags (`apps/pi-extension/src/state.ts:203`).
- `session_start` and `session_switch` now call the reset before any persisted entry restore (`apps/pi-extension/src/session.ts:195`, `apps/pi-extension/src/session.ts:416`).
- Persisted `focusa-state` / `focusa-wbm-state` is restored only when `entry.data.sessionId === eventSessionId`; otherwise even frame title/current ask are ignored (`apps/pi-extension/src/session.ts:218`, `apps/pi-extension/src/session.ts:426`).
- Runtime isolation proof now seeds a fake `Pi Task: SPEC96 FROM OTHER SESSION`, runs the session reset, and asserts Utility Card/currentAsk/frame cache do not leak it (`tests/utility_card_session_isolation_test.mts:54`).

Validation:

- `apps/pi-extension npx tsc --noEmit` — pass.
- `tests/spec96_utility_card_session_isolation_static_test.sh` — pass.
- `tests/spec96_broad_root_scope_isolation_static_test.sh` — pass.

## Emergency follow-up 3: Pi session display name ownership

Operator reported `Pi Task:` replacing the Pi session name. Pi docs confirm `pi.setSessionName(name)` sets the session display name shown in the session selector, and `/name <name>` is the user-facing session naming command (`docs/extensions.md`, `docs/sessions.md`). Focusa frame titles are context metadata, not Pi session names.

Fix:

- Removed all automatic `pi.setSessionName(...)` calls from Focusa Pi extension source (`apps/pi-extension/src/state.ts`, `apps/pi-extension/src/session.ts`, `apps/pi-extension/src/turns.ts`).
- `session_start` now caches scoped Focusa frame title/goal for Focusa context only and explicitly documents Pi session display ownership (`apps/pi-extension/src/session.ts:271`).
- Updated §35.8 from "Session name from focus frame" to "Pi session display name ownership" (`docs/44-pi-focusa-integration-spec.md:1704`).
- Updated Pi extension contract to require no `setSessionName` usage in app source while preserving scoped frame metadata (`tests/pi_extension_contract_test.sh:260`).

Validation:

- `apps/pi-extension npx tsc --noEmit` — pass.
- `tests/spec96_utility_card_session_isolation_static_test.sh` — pass.
- `tests/spec96_broad_root_scope_isolation_static_test.sh` — pass.
- `node scripts/validate-focusa-tool-contracts.mjs` — pass, 58/58.
- `git diff --check` — pass.
- Focused static proof: `rg -n "setSessionName" apps/pi-extension/src` returns no matches.
- `tests/pi_extension_contract_test.sh` static/app sections pass, but full strict run currently has one unrelated daemon seed failure: `/v1/focus/push` rejected with `session_inactive`/`closed` after `/v1/session/start`.
