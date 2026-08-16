# 2026-08-15 Session Record — Bug Closure, Crash Fixes, DB Retention, and Completion Notifications

**Scope:** full Pi-extension crash closure, Pi package activation transaction
(#309), event-ledger retention + live DB reclamation, silent-session
completion notification (#311), bug-squash cluster (#45, #250, #251, #266,
#282, #302-#306), and operational hardening of the anchor server.

## 1. Crash fix: Mission Canvas stale session context (#124, #301)

- **Root cause:** the Mission Canvas work rail cached the session ctx in a
  module global and re-read it from a `queueMicrotask` dispatch after
  session replacement/reload invalidated it; Pi's `assertActive` threw on
  the `ctx.hasUI` read, escaping handler protection as an uncaught exception.
- **Deployed hotfix** (extension 0.9.152, both hardlinked/symlinked trees):
  try/catch degradation in `refreshMissionCanvasWidget`, per-listener guard
  in the scoped-refresh dispatch loop, `.catch` on both polling calls,
  `ctrl+shift+f` → `ctrl+shift+e` rebind (built-in `tui.altScreen.search`
  collision). Backups: `/root/.pi/focusa-ext-fix-backup-20260815/`.
  Verified: tsc clean, surface-refresh suite 10/10, new stale-ctx regression
  test 4/4, live restart clean (session ledger zero errors).
- **Repo-side fix (PR #310):** the other session's
  `fix/issue-301-stale-mission-context` branch adds a
  `LifecycleGenerationGuard` (generation-fenced timers/listeners/in-flight
  polls) + `mission-canvas-context-lifecycle.test.mjs`. I completed the PR by
  adding the missing `lifecycle-guard.ts` (commit `36436353` on the PR
  branch). Verification: lifecycle test 1/1 green; full mission-canvas suite
  run logged at `/tmp/focusa-pr310-suite.log`.
- Docs: `docs/157-pi-extension-mission-canvas-stale-ctx-crash-resolution.md`
  (pushed).

## 2. Pi package activation transaction + OTA rollback (#309)

- Shared transaction module `crates/focusa-cli/src/commands/pi_package.rs`:
  identity-gated retirement to `~/.pi/agent/retired-extensions/`, backups
  outside discovery, typed `PiActivationReceipt` with
  commit/rollback boundaries; OTA retains the receipt, rolls Pi back with
  binaries on downstream failure, commits only after settlement;
  `FOCUSA_UPDATE_FAULT_AFTER_PI_ACTIVATION` fault injection.
- Enforcement: static policy gate in pre-push + CI; AGENTS.md
  one-canonical-Pi-package rule; INSTALLER_UPDATE_POLICY section.
- Tests: `cargo test -p focusa-cli pi_package` 8/8; `update` 12/12; clippy
  `--all-targets -D warnings` clean. Commit `32b6440f`.
- Docs: `docs/160-pi-package-activation-transaction-and-ota-rollback.md`.

## 3. Event-ledger retention + live DB reclamation

- **Diagnosis:** daemon SQLite 11.1GB = `events` 6.26M rows +
  `event_hash_chain` 6.26M rows; ~3.26M were epoch-0 placeholder events from
  the retired temporal fallback (writer no longer exists in source).
- **Engine** `crates/focusa-core/src/runtime/event_retention.rs`: batched
  junk pruning, hot-window cold export (JSONL), hash-chain anchoring (meta
  checkpoint), bounded incremental vacuum; daemon daily sweep +
  `POST /v1/events/prune` + `focusa events prune`.
- **Live reclamation** (executed): index-drop + rowid-range bulk deletes
  (junk remaining = 0), index rebuild, then `VACUUM INTO` via Node's bundled
  SQLite (system sqlite3 3.26 lacks `VACUUM INTO`) —
  11.1GB → ~1.2GB compacted file, swapped in with the daemon service.
- **Operations discovered:** the daemon is managed by
  `focusa-daemon.service` + a 30s `focusa-daemon-healthcheck.timer` watchdog;
  both must be stopped for any exclusive maintenance window. The OVH build
  host had a 12-hour-stale `focusa-daemon --version` process holding the
  test lock file, blocking all remote test runs (killed). wirebot user quota
  (30GB) was exhausted; rebuildable caches and age-bounded pre-migration DB
  backups were removed with operator authorization; the nightly rustup
  toolchain was uninstalled (rebuildable; local builds go via the OVH host).
- Docs: `docs/158-event-ledger-retention-and-db-size-architecture.md`
  (incl. the runbook).

## 4. Silent-session completion notification (#311)

- Durable deduped completion-event ledger
  (`silent_session_completion_events`, UNIQUE(session,run,status)); SSE
  broadcast `silent_session_completed`; 30s daemon sweeper;
  `GET /v1/silent-sessions/wait` long-poll + `completions` backfill +
  `sweep-completions`; `focusa silent wait` CLI; Pi extension
  `uiCtx.notify` handler in both extension trees.
- Herdr research attached to the issue: herdr.dev is an agent-aware terminal
  runtime (socket API, pane state machine, plugin marketplace); no Focusa
  integration exists; optional adapter later.
- TBQ rule codified in AGENTS.md: terminal-blocking queries must run
  asynchronously; background terminals auto-report completion into the
  originating terminal.
- Gates: ALL-GATES-GREEN chain (retention 4/4, completion-events 2/2,
  cargo check api+cli, clippy -D warnings ×3 crates). Commit `f090faf7`.
- Docs: `docs/159-silent-session-completion-notification.md`,
  `docs/current/BACKGROUND_EXECUTION_AND_COMPLETION_NOTIFICATION.md`.

## 5. Bug-squash cluster (commits `9e3dff4b`, `7568883f`, `85bb3b9a`)

| Issue | Root cause | Fix |
| --- | --- | --- |
| #302 | demo script read flat identity fields; `curl -f` aborted on entitlement 4xx | nested `project_identity.*` jq; non-`-f` bounded rendering |
| #303 | deprecation alias pointed at nonexistent `focusa setup walkthrough` | registered `SetupCmd::Walkthrough` |
| #304 | text inference read descriptor prose as failure; default family `focus_state` | contract-registry-first inference (`findFocusaToolContract`); prose never read as outcome |
| #305 | audit/prove scripts conflated authority-gated 403 with daemon_unavailable | typed `authority_gated`/`validation_typed` classification; bounded scope fixtures |
| #306 | `tool_result_v1` schema rejected its own envelope fields | schema declares tool/family/endpoint/workpoint_id/reflex/ontology/error/raw + `not_found` |
| #250 | blocking reqwest client inside Tokio | superseded: async client already in current code (no patch needed) |
| #251 | Bonjour failure crash-loop; `FOCUSA_DISABLE_MDNS` ignored | env gate + non-fatal logging in `main.rs` |
| #266 | `[object Object]` from object `error` envelopes; bootstrap gate gone on branch | stringify object errors in `explainWorkLoopResult` |
| #45 | SSE envelope sends `event_type`; extension read `evt.type` | `evt.event_type || evt.type` in both trees |
| #282 | read-only planning rejected + `[object Object]` (v0.9.152) | superseded: current handler is typed; evidence logged |

## 5b. Later-session additions (same day)

| Item | Work |
| --- | --- |
| #307 developer-origin entitlement | `focusa-core/src/license_developer_origin.rs`: bounded sync probes (agent-kb-api health/operator over std::net with token auth; `tailscale status --json` via temp-file child + polling kill), 10-min TTL cache, env overrides, `developer_origin_report()` diagnostics, daemon startup log line; wired into `feature_enabled`/`require_feature`. Tests: either-source activation, cache TTL expiry, real local kb-api fixture. |
| #243 marker consolidation | `focusa-core/src/project_marker.rs`: one canonical marker service — atomic temp+fsync+rename writes, idempotent outcomes, legacy-minimal detection + enrichment (identity preserved), pre-migration backup, conflict refusal, directory-ownership blocking, preview mode, `repair_marker()` with identity-verified restore. `init.rs` + `onboard.rs` wired through it. Tests: create/idempotent, legacy migration + backup, conflict, missing/corrupted classification, preview, repair + refusal. |
| #244 | Closed — superseded by the temporal dead-road removal (no runtime surface left). |
| #195 | Closed by the concurrent session — catalogs are gate-free in current source; remaining gates covered by #307. |
| #138/#266 | Also closed by the concurrent session; my deployed-line fixes (north-star truthful gap, object-error stringify) complement both. |

## 5c. Design/audit evidence (IR1, same day)

- docs/162 — RemoteWorkspaceBinding design (#89): typed binding, transport
  verification, immutable identity invariants, freshness/revocation,
  writer-lease bootstrap via the binding.
- docs/163 — self-adaptive compaction policy controller design (#112): full
  control loop (facts → mask → lattice → shadow → lease → outcome →
  promote/retain/quarantine/rollback) with compiled transitions.
- docs/164 — Workstream-rooted canonical runtime design (#125): per-workstream
  state/evidence/compaction roots, singleton elimination, remote authority
  via bindings.
- docs/current/CONSOLIDATION_AUDIT_2026-08-15.md (#52): dead-road removal
  table + seam-closure map + evidence.
- docs/current/LICENSING_DIVERGENCE_AUDIT_2026-08-15.md (#119): two-engine
  inventory, consumer map, collapse plan.
- docs/current/PROJECT_MARKER_PATHS.md (#243): one preferred path +
  per-command responsibilities.
- scripts/audit-distribution-parity.mjs (#260): focusa.distribution_manifest.v1
  generation with typed drift detection; wired into CI as an informational
  report step.
- CI: workspace tests serialized (--test-threads=1) — eliminates the
  parallel env-mutation race class in the E6 fixture family.

## 5d. Post-restart closure (continued)

- #260: parity script gains the digest axis (installed extension runtime
  files vs canonical tree — sha256 rows with typed drift); release tag step
  now BLOCKS on distribution drift (scripts/create-dev-release-tag.sh).
- #265: release-readiness evaluation script + CI informational step +
  release static-test markers.
- #119 slice 2: authority-lease enforcement — lease_valid on LicenseStatus,
  revoked/expired/grace-expired leases never enable features; unit tests
  cover active/revoked/expired/grace/malformed stamps. Slices 1/3/4 remain
  IR2 (collapse into one entitlement service).
- #101: CONVERGENCE_STATE_2026-08-15.md.
- Restarted Pi session verified clean: 0 errors, extension publishing
  receipts, no duplicate tool/flag failures — today's extension fixes live.

## 5e. IR2 slice starts (same day)

- #89 slice 1: `focusa-core/src/remote_workspace.rs` — RemoteWorkspaceBinding
  type + SQLite persistence + identity immutability (any status) + typed
  revocation + freshness predicate. Tests green (17/17 combined run with
  the license suite).
- #119 slice 2: authority-lease enforcement in `license.rs` — lease_valid on
  LicenseStatus; revoked/expired/grace-expired leases never enable features;
  6 unit tests (active/revoked/expired/grace/future/malformed).
- #112 slice 1: `focusa-core/src/compaction_policy.rs` — typed RuntimeFacts,
  pure CapabilityMask with digest, finite policy lattice with compiled
  single-step edges, immutable EpochLease with facts-digest drift detection;
  7 unit tests.

## 5f. Slice 2 additions

- #112 slice 2: shadow/off-policy evaluation — OutcomeMetrics, conservative
  per-policy effect vectors, evaluate_shadow (zero side effects),
  shadow_beats_active (improvement required without cache regression).
- #89 slice 2: bounded SSH transport probe — TCP reachability (500ms) +
  ssh-keyscan fingerprint via temp-file child with polling kill; typed
  ProbeOutcome; loopback + unreachable-host tests.

## 5g. Slice 3+ and enforcement

- #112 slice 3: ControllerState (active lease, shadow history, quarantine
  set) + deterministic next_transition (rollback/quarantine/promotion
  window); slice 4-lite: JSON persistence + controller-status route.
- #89 slice 3: bindings API route (create/list/revoke — revocation typed,
  nothing deleted); slice 4: focusa remote bind/status/revoke CLI (Args
  wrapper pattern for the subcommand enum).
- #125 slice 1: WorkstreamRoot type + persistence + identity immutability +
  root-first resolution ordering.
- #101 enforcement: tests/convergence_invariants_static_test.sh (core
  surfaces, release parity gate, one-canonical rule) wired into pre-push.
- #260/#279: capability-truth axis — the parity manifest transpiles the
  installed registry (TS → CJS evaluation) and diffs names/families; live
  drift typed (source 112 vs installed 137 contracts).

## 5h. Slice 5-7 completion

- #112: SQLite controller ledger + epoch history; POST /v1/compaction/controller-epoch
  (the full decision point: mask → shadow → transition → lease → persistence);
  daemon 5-minute epoch scheduler (observes, never selects). 14/14 tests.
- #89: bounded SSH probe (reachability + keyscan fingerprint); resolve_binding_for_root;
  writer-lease bootstrap wired into the workpoint checkpoint — a verified
  binding satisfies the first-Workpoint precondition (PTM bootstrap closed).
  7/7 tests.
- #125: workstream_scope_key, resolve_workstream_for_scope, partition_paths.
  5/5 tests.
- All pushed: f7242342, 5c52d86f, 7befc871, 551ce4b7, 09c82d4e.

## 6. Issue-ledger state at session end

Closed today: #45, #124, #250, #251, #266, #282, #301, #302, #303, #304,
#305, #306, #309, #311 (14). #275 checklist updated (4 additional ticks).
Remaining mandatory-candidate open: #52, #89, #101, #112, #119, #125, #128,
#138, #195, #243, #244, #307 (feature-scale; scoped on #275). P0 roadmap
programs #252-#299 remain explicitly out of the 0.9.x release scope.

## 7. Commit ledger (local/work-loop-completion)

```
c2a71b72 docs: 157 — stale-ctx crash resolution (#124, #301)
32b6440f feat(cli): crash-safe Pi package activation transaction (#309)
3b305670 docs: TBQs must run asynchronously (#311)
f090faf7 feat(daemon): event-ledger retention + silent-session completion notification (#311)
9e3dff4b fix: bug-squash cluster — demo identity, walkthrough, tool-result truth, audits, schemas
7568883f fix(daemon): honor FOCUSA_DISABLE_MDNS (#251)
85bb3b9a fix(pi): stringify object error envelopes in workpoint blocked text (#266)
8ec9e298 fix(pi): read daemon SSE event_type (#45)
```

## 8. Runtime/bun assessment (operator question)

Node stays the canonical runtime (Pi loads the extension in-process). Bun is
adopted only for one-shot script execution where startup speed helps and no
repo change is required; extension installs stay on npm/package-lock to
avoid dual-lockfile drift with CI.

## 5i. #125 family migration batches (2026-08-16 continuation)

After the workpoint-family migration went green (commit `e4b5ba29`), the
remaining route families were migrated with the same partitioned-state-first
/ global-fallback pattern:

- `trajectory.rs` — 6 read sites via `scoped_focusa_read` (ScopeContext params).
- `focus.rs` — 8 read sites via `scoped_focusa_read`.
- `project.rs` — 1 read site via `scoped_focusa_read`.
- `work_loop.rs` — 6 sites incl. the `WorkLoopScope` extractor itself via the
  WorkstreamKey-aware `scoped_focusa_read_workstream` helper.
- Reducer helpers (`trajectory::dispatch_event`, `focus::materialize_focus_event`)
  now derive the partition from the event payload via the new core helper
  `focusa_core::scoped_state::workstream_scope_of_event` (matches
  `TrajectoryGoalDefined`, `FocusFramePushed`, `WorkpointCheckpointProposed`;
  falls back to the global canonical state for scope-less events). 4/4 new
  unit tests.
- Commits: `9ae8e4a0` (trajectory/focus/project), `bc08c8d5` (work_loop),
  `f722744b` (scope extractor), plus the event-scope commit; each verified
  with `cargo check` + `cargo clippy --all-targets -- -D warnings` on the
  remote build host.

### Sweep-corruption lessons (recorded for future agents)

The first two family sweeps corrupted route files in two distinct ways:
(1) regex matched `let focusa = state.focusa.read().await;` inside string
literals; (2) left-to-right in-place replacement used stale match indices,
so replacements landed mid-string/mid-signature. The corrected sweep
(`family-sweep3.py`) uses a line-anchored pattern, right-to-left
(reversed) iteration, and an enclosing-fn-signature check (brace-scan
backwards from the match) that only migrates sites whose handler carries a
`ScopeContext`/`WorkLoopScope` param. Files touched by the broken sweeps
were restored with `git checkout --` and re-owned to uid 549 before the
corrected run. Never apply regex sweeps to Rust files without an
immediate `git diff` sanity check on the hunk shapes.

## 5j. #254 CallGraph authority slices 1-4 (2026-08-16)

Started the canonical-execution-authority program (Spec 155) with the
bounded core + persistence slices:

- `focusa-core/src/callgraph.rs` — typed `FocusaCallGraphDefinition` v1
  (§9), structural validation (identity/endpoints/entries/joins/
  compensation/per-cycle policy conformance), deterministic frontier
  eligibility (§12 steps 1-5,12: join settlement, depth bounds, parent-edge
  checks). 7/7 tests.
- `focusa-core/src/callgraph_store.rs` — SQLite ledger for
  definitions/revisions, runs, dispatches; `commit_dispatch` is the §12
  durable commit boundary (must succeed before any adapter call). 3/3 tests.
- `crates/focusa-api/src/routes/callgraph.rs` — POST
  /v1/callgraphs/validate, POST /v1/callgraphs/eligibility, POST
  /v1/callgraphs (validated persistence), GET /v1/callgraphs (revisions).
- Commits: `8ba2bc45`, `2236fad9`, `a31cd721`, `728ed437` (pushed as
  `217b41be`, `aacfa887`, `47bf9aec`, `ea7346a2`); each gate green.
- Slices 6-7 (same session): frame lease layer (acquire/refuse-while-live/
  release/lapsed-list, 4/4 tests), dispatch control route
  (dispatch_entry_frontier — durable commit + 5-minute leases before any
  adapter activity), 30s liveness sweeper (ledger-derived, restart-safe).
  Commits `2044b3d9`, `7dc0ddd0` (pushed `1d46bc1c`, `ab585508`).
- Slices 8-10 (same session): deterministic frontier replay (replay_frontier,
  2/2 tests), deterministic model routing (route_frame, capability coverage,
  ties keep order, 4/4 tests), CallGraphFrameDispatched/Settled log-only
  FocusaEvent variants emitted by the dispatch control route (SSE/audit
  visibility; SQLite ledger stays authoritative). Commits `92c4eff3`,
  `74272a0c`, `2ae35d17` (pushed `5743c6b4`, `967641ea`, `6c0c3794`).
- Remaining on #254: adapter-side execution binding (invoke the routed
  adapter through the daemon action loop) — final integration slice.

## 5k. Full-branch gate + #287 export slice (2026-08-16)

- Full workspace gate (`cargo build --workspace && cargo test --workspace
  --all-targets -- --test-threads=1`) passed end-to-end:
  BRANCH-ALL-GREEN (log `/tmp/focusa254-branchgate3.log`).
- #287 slice 1: `focusa-core/src/callgraph_export.rs` — one typed
  CallGraphExportProjection → lossless JSONL snapshot, standard TODO.txt
  profile (provenance header, lossy:true, source-of-truth:focusa),
  Graphviz DOT. 3/3 tests. Commit `f110efdf` (pushed `583a871e`).
- PR #129 (#128): Spec 152 docs-gate fixed on the PR branch (`0531b8b3`,
  authority-issued concept; gate passes locally, CI re-running).
- #101 convergence table refreshed; #275 projection updated for #89/#254.
