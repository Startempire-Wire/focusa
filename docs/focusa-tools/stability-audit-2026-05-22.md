# Focusa Pi Tool Stability Audit — 2026-05-22

Scope: `apps/pi-extension/src/tools.ts`, `apps/pi-extension/src/state.ts`, and the 58 `focusa_*` tools exposed to Pi agents.

## Weaknesses found

1. **Unsafe-cwd sessions could not recover Focus State writes.** Fresh/compacted Pi sessions launched from `/root` lacked a scoped frame and session-local Workpoint packet, so `focusa_decide`/`focusa_failure` fell back to scratchpad with `no_active_frame`.
2. **Scope-sensitive tools defaulted to unsafe cwd.** Workpoint/Trajectory/evidence tools used `S.sessionCwd || process.cwd()` directly, making `/root` a common bad default after compaction or cross-session recovery.
3. **Evidence 409 conflicts were opaque.** `focusa_evidence_capture` collapsed Workpoint scope mismatches into `request failed (409)` instead of showing the mismatched field and recovery action.
4. **Many registered tools lacked prompt snippets.** The tool suite had descriptions, but many tools did not provide compact action guidance to the agent at registration time.
5. **Contract tests did not enforce agent-facing guidance coverage.** Runtime contract tests verified selected request/response behavior but not prompt snippet availability across the whole Focusa suite.

## Improvements applied

1. **Frame recovery now adopts safe Workpoint authority.** `ensurePiFrame` can use a session Workpoint or daemon active canonical Workpoint to recover safe `project_root + continuity_id` before creating a Pi frame.
2. **Scope-sensitive tools use a safe project resolver.** Trajectory, Workpoint, and evidence tools now resolve safe project scope from explicit input, safe session cwd, session Workpoint, or daemon active Workpoint before falling back.
3. **409 evidence conflicts report actionable scope diagnostics.** Tool output now includes expected vs packet scope, `scope_recovery_context`, `request_scope`, and next tools.
4. **All Focusa tools get prompt snippets.** A default family-aware prompt snippet is injected for any `focusa_*` registration missing explicit guidance.
5. **Runtime/static coverage tightened.** Tests now assert prompt snippet coverage, schema expectations, missing-frame recovery, scoped-tool unsafe-cwd recovery, and evidence 409 clarity.

## Verification

- `cd apps/pi-extension && npx tsc --noEmit`
- `bash tests/spec80_impl_pi_extension_runtime_contract_test.sh`
- `bash tests/spec96_stale_focus_frame_validation_static_test.sh`
- `bash tests/focus_frame_write_contract_test.sh`
- `node scripts/validate-focusa-tool-contracts.mjs`

## Operational note

Existing Pi sessions must reload/restart to pick up extension source changes. New sessions should still pass explicit `project_root` when known; automatic Workpoint adoption is a recovery path, not a substitute for a clear project folder.

## Second-pass hardening

Additional weaknesses found after the first patch:

1. **Focus State write failures lacked structured recovery fields.** The human text said a write failed, but tool-result consumers did not receive consistent `failure_class`, `retry_posture`, `recovery_hint`, and `next_tools`.
2. **Tool result envelopes discarded tool-specific next-tool hints.** Inferred envelopes defaulted `next_tools` by family and could miss more precise recovery suggestions already produced by a tool.
3. **`focusa_tool_doctor` summarized health but did not prescribe action.** It exposed health/resource/workpoint status without a concise recommended action, session-scope safety, or next-tool routing.

Second-pass improvements:

1. Added `pushDeltaFailureRecovery` to map Focus State write failures to structured recovery posture and next tools.
2. Added structured recovery data to `focusa_decide`, `focusa_constraint`, `focusa_failure`, and bounded slot fallbacks.
3. Made inferred tool envelopes preserve `details.next_tools` when present.
4. Expanded `focusa_tool_doctor` with session-scope diagnostics, recommendations, recommended action, and next-tool routing.
5. Extended static coverage to require structured Focus State recovery and actionable tool-doctor diagnostics.

Second-pass verification used the same gate set:

- `cd apps/pi-extension && npx tsc --noEmit`
- `bash tests/spec80_impl_pi_extension_runtime_contract_test.sh`
- `bash tests/spec96_stale_focus_frame_validation_static_test.sh`
- `bash tests/focus_frame_write_contract_test.sh`
- `node scripts/validate-focusa-tool-contracts.mjs`

## Project-root inference refinement

Operator correction: the Focusa project root must not default to the Pi agent install/root directory or to the Focusa repo itself. The coding agent runtime location and the active project root are separate concepts.

Refinement applied:

- Project root inference is now marker-based and directory-layout agnostic.
- Markers considered include `.focusa-project.json`, `.git`, `.beads`, and common root-level workspace files (`Cargo.toml`, `package.json`, `pyproject.toml`, `go.mod`, `composer.json`, lock/workspace files).
- Each inference carries a confidence label and numeric score.
- Roots below 90% confidence set `requiresOperatorConfirmation=true`; the agent should use an operator menu/interview to select the correct root before Focusa writes.
- Daemon-global active Workpoint scope is no longer used as a fallback for unrelated `/root` sessions.

Additional verification:

- `bash tests/pi_project_root_inference_test.sh`
- `bash tests/spec96_utility_card_session_isolation_static_test.sh`

## Project-root authority gate

Additional correction: correct project root determines Focus State, Workpoint, Trajectory, evidence, and progress authority. Therefore low-confidence root guesses must not silently bind state.

Third-pass refinement:

- Automatic Pi session/frame/Workpoint/Trajectory binding is blocked when project-root confidence is below 90%.
- Focus State frame creation returns no frame under unconfirmed project-root scope and emits bounded telemetry.
- Scope-sensitive Focusa tools return a blocked envelope with candidate roots and `next_tools` including `interview` so the operator can confirm the correct root.
- Utility Card shows confidence/source and directs the agent to menu confirmation before Focusa writes.

Additional verification:

- `bash tests/pi_project_root_inference_test.sh` now proves low-confidence package-only roots expose candidates and block automatic frame creation.
- `bash tests/spec96_stale_focus_frame_validation_static_test.sh` now asserts low-confidence root gates exist.

## Core enforcement for project-root authority

Final accuracy pass: Pi-side gating is not enough because tools or future clients could call Focusa APIs directly.

Core updates:

- `FocusaSessionIdentity` now carries project-root confidence metadata: confidence label, numeric confidence score, resolution source, candidate roots, and `requires_operator_confirmation`.
- Workpoint checkpoint and evidence-link routes reject session identities whose project root is unsafe, below 90% confidence, or explicitly requires operator confirmation.
- Rejection envelope uses `project_root_confirmation_required`, `failure_class=scope_mismatch`, `retry_posture=operator_required`, candidate roots, and `next_tools=[interview, focusa_project_identity, focusa_workpoint_checkpoint]`.

This makes project-root accuracy a daemon-enforced authority boundary, not only Pi client behavior.

## Navigation metaphor correction

Operator clarification: `project_root` is the project folder/container holding related files. As a metaphor, it is the vessel or hull: a better-confirmed vessel improves navigation/travel reliability, but it is not the route or destination. Focusa Trajectory carries the navigation model: current functional state, desired destination/outcome, and waypoint goals. Utility Cards should require both: confirmed project folder plus clear trajectory.
