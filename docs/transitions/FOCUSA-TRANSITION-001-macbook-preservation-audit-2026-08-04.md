# FOCUSA-TRANSITION-001 MacBook preservation and transition audit

Date: 2026-08-04

Authority read from: `origin/transition/spec158-desktop-pivot` at `47a9134143d044dc17268d1f30f1b3776282f7b5`

Coordination issues read: #125 and #128

## A. Worktree path, branch, and HEAD

- Worktree: `/Volumes/Macintosh HD/Users/vsmith/focusa-mission-canvas`
- Preserved source branch: `feature/spec-135-context-connectors-b2`
- Preserved source HEAD: `e5c8cdfd8852aabdcffc06bf84cfdb65dee9adf5`
- Local checkpoint branch: `checkpoint/spec-135-pi-mission-canvas-pre-desktop-pivot-2026-08-04`
- Checkpoint commit: `d45e2439a3ff4cb9ba07ae11fe36f45365f374c2`
- Audit branch: `local/spec158-desktop-pivot-audit-2026-08-04`
- Current `origin/main`: `eb0fcccaca9eca2c730b881ff45c9f8b0e87509b`
- Merge base with current main: `9decd212c153f83862de5daa3fa0cc786a2518fd`
- The checkpoint is an empty tree-preserving commit; its tree exactly matches `e5c8cdfd`.

## B. Uncommitted and untracked inventory

At preservation time:

- tracked modifications: none;
- staged modifications: none;
- untracked files: none;
- stashes: none;
- mission-relevant ignored files outside tracked proof/contracts: none found.

Machine-local/reproducible exclusions (not preservation assets):

- `.focusa/` (764 KiB): ignored local runtime state;
- `apps/pi-extension/node_modules/` (226 MiB): dependency cache;
- `apps/menubar/node_modules/` (80 MiB): dependency cache;
- `apps/menubar/.svelte-kit/` (900 KiB): generated preview cache;
- `apps/menubar/src-tauri/target/` (1.7 GiB): generated Rust/Tauri output;
- repository `target/` (16 GiB): generated Rust output;
- `.pytest_cache/` and `.ruff_cache/`: generated test caches.

No irreplaceable ignored mission artifact required a sanitized archive. The list above is the exclusion manifest; generated caches and build outputs were intentionally not archived or committed.

## C. Unpushed commit inventory

- `feature/spec-135-context-connectors-b2` is exactly equal to its remote branch at `e5c8cdfd`; there were zero unpushed source-branch commits before preservation.
- The local checkpoint has one intentionally unpushed commit: `d45e2439 chore: checkpoint Pi Mission Canvas before Desktop pivot`.
- The audit branch will contain only this report commit until review assigns implementation.
- Relative to current `origin/main`, preserved source HEAD has 122 unique commits and is missing 56 main commits. This divergence must not be rebased, merged, or pushed as part of preservation.
- The latest source-branch slice after its last main merge is:
  - `f7126d5e fix(project): restore canonical marker root`
  - `19ae1f6c feat(canvas): make work surfaces durable and actionable`
  - `59b343aa feat(canvas): stream live state and persist visual mode`
  - `f45a617a feat(canvas): add live refresh and native theme controls`
  - `e5c8cdfd feat(canvas): restore durable surface layouts`

## D. Mission Canvas source-file inventory

### Pi extension runtime (15 files)

- `apps/pi-extension/src/mission-canvas-accessibility.ts`
- `apps/pi-extension/src/mission-canvas-layout.ts`
- `apps/pi-extension/src/mission-canvas-model.ts`
- `apps/pi-extension/src/mission-canvas-session-inventory.ts`
- `apps/pi-extension/src/mission-canvas-shell.ts`
- `apps/pi-extension/src/mission-canvas-tool.ts`
- `apps/pi-extension/src/mission-canvas-view.ts`
- `apps/pi-extension/src/mission-canvas-widget.ts`
- `apps/pi-extension/src/work-rail-interactions.ts`
- `apps/pi-extension/src/work-rail-widget.ts`
- `apps/pi-extension/src/commands.ts`
- `apps/pi-extension/src/config.ts`
- `apps/pi-extension/src/project-binding.ts`
- `apps/pi-extension/src/scoped-surface-refresh.ts`
- `apps/pi-extension/src/semantic-surface-truth.ts`

### Core and API runtime (11 files)

- `crates/focusa-core/src/mission_canvas/{mod,layout,memory,model,persistence,profiles,reducer,resolver}.rs`
- `crates/focusa-api/src/routes/mission_canvas.rs`
- `crates/focusa-api/src/routes/mission_canvas_surfaces.rs`
- route registration in the API server/router surfaces

### Other presentation/runtime sources

- `apps/menubar/src/lib/components/MissionCanvasView.svelte`
- `packages/a2ui-renderer/proof/{mission-surfaces,work-rail}.{html,ts}`
- `packages/a2ui-renderer/fixtures/work-rail-tui-snapshots.json`
- Pi Mission Canvas skills/runbooks under `.pi/skills/` and `apps/pi-extension/skills/`

### Contracts, schemas, scripts, and proof

- 25 Mission Canvas schemas under `schemas/spec135/mission-canvas/`;
- generated JSON Schema/OpenAPI/TypeScript/operation-registry contracts under `docs/contracts/spec135/`;
- completion DAG, portability, host-renderer, compatibility, and proof artifacts under `docs/contracts/`;
- Spec 135 generator/materializer scripts under `scripts/`;
- tracked screenshots and text proof under `docs/contracts/evidence/`.

The repository query found 127 tracked Mission Canvas/Work Rail/related paths. Generated artifacts remain generated/replaceable and are not candidates for semantic-core ownership.

## E. Mission Canvas test inventory and latest results

### Passing Pi/TypeScript checks

- `npm run check`: PASS.
- Eight tests in `npm run test:mission-canvas`: each passes individually.
- `mission-canvas-accessibility.test.mjs`: PASS.
- `work-rail-widget-width.test.mjs`: PASS.
- `scoped-surface-refresh.test.mjs`: PASS.
- `scoped-surface-refresh-runtime.test.mjs`: PASS.
- Project-binding set (`project-binding-decision`, `session-project-classification`, `north-star-gate`, `session-health-lifecycle-race`, `unbound-project-binding-guidance`): PASS.

The combined `npm run test:mission-canvas` process exceeded the initial 180-second harness limit after its second test because the shell-runtime process did not exit promptly; rerunning all eight files independently produced eight passes.

### Passing repository Spec 135 checks

- `spec135_mission_canvas_api_static_test.py`: PASS.
- `spec135_mission_canvas_completion_dag_test.py`: PASS (308 tasks, 381 nodes, 623 edges).
- `spec135_mission_canvas_generated_parity_test.py`: PASS.
- `spec135_mission_canvas_operation_registry_test.py`: PASS.
- `spec135_mission_canvas_portability_test.py`: PASS.
- `spec135_mission_canvas_surfaces_test.py`: PASS.
- `spec135_mission_canvas_naming_and_multiplexing_static_test.sh`: PASS.

### Failing or blocked checks

- `spec135_mission_canvas_full_shell_gui_test.py`: FAIL. The static firewall still requires the removed literal `Authoritative Pi-native Mission Canvas` in `mission-canvas-shell.ts`; this is a real test/behavior-contract drift requiring transition disposition, not a reason to restore rich Pi-primary authority.
- `spec135_mission_canvas_surfaces_e2e_test.py`: BLOCKED/FAIL because its local daemon did not become available. The orphan test daemon was terminated after the failed harness.
- `cargo test -p focusa-core mission_canvas -- --nocapture`: INCONCLUSIVE; compilation remained at `Compiling focusa-core` and exceeded 900 seconds without a compiler diagnostic.
- `cargo test -p focusa-api mission_canvas`: not rerun after the earlier concurrent attempt waited on Cargo package/build locks. Native Rust verification is a blocker, not a pass.

The focused Rust source contains 15 `#[test]`/`#[tokio::test]` markers across Mission Canvas core/API paths.

## F. Unique work not present on current main

A direct tree comparison from current `origin/main` to preserved `e5c8cdfd` reports 417 changed paths. Mission Canvas-specific unique work includes:

- Pi-native shell, layout, accessibility, interaction, UIAI harness, and reference-design files;
- durable/actionable Work Surface model and layout persistence;
- live refresh, visual-mode persistence, theme controls, and overlay viewport positioning;
- Mission Canvas core model/reducer/resolver/persistence modules;
- Mission Canvas API routes and generated contracts;
- Spec 135 completion DAG and proof tests.

Current main has also advanced independently by 56 commits. Therefore this audit treats the preserved branch as a source/provenance corpus, not a merge-ready Desktop base.

Other registered worktrees:

- `focusa` on `local/mission-deck-interactions`: tracked modifications to `.beads/issues.jsonl` and `.focusa-project.json`; 33 commits ahead and 1045 behind current main.
- `focusa-cockpit-004` on `feat/cockpit-004-menubar-handoff`: tracked-clean; one commit ahead and 40 behind.
- `focusa-workloop` on `local/work-loop-completion`: tracked-clean; 16 commits ahead and 416 behind; branch-specific changes include `crates/focusa-core/src/runtime/daemon.rs` and `silent_sessions/platform_backends.rs`.
- Two `/private/tmp` worktrees are marked prunable and were not modified.

No code from another worktree is assumed authoritative or merged by this report.

## G. Preservation checkpoint ref

`checkpoint/spec-135-pi-mission-canvas-pre-desktop-pivot-2026-08-04` at `d45e2439a3ff4cb9ba07ae11fe36f45365f374c2`.

The checkpoint is local only and has not been pushed, tagged, rebased, or merged.

## H. File and behavior migration ledger

| path | current responsibility | branch/local/main provenance | current tests | current identity fields | required Workstream correction | new owner | disposition | parity criterion | retirement gate | notes |
|---|---|---|---|---|---|---|---|---|---|---|
| `apps/pi-extension/src/mission-canvas-model.ts` | Normalizes Work Surface and semantic-pair projections | modified on preserved branch vs current main | Pi surface, layout, session inventory, performance | `projectRoot`, `continuityId`, Workpoint, Instance, Session, Attachment, `workSurfaceId`; no Workstream | require `ScopeRef`, stable `workstreamId`, explicit Attachment; stop continuity fallback for `groupId` | shared semantic package after correction | `correct_identity_then_extract` | identical normalized semantics for exact Workstream/Attachment fixtures; continuity-only fixture fails closed | shared package and Pi adapter pass cross-Workstream adversarial tests | `SemanticPairPortfolio.scope` also uses only project root + continuity |
| `apps/pi-extension/src/mission-canvas-session-inventory.ts` | Joins discovered, Silent, and Work Surface sessions | present on current main and branch | session-inventory test | project root, continuity, Session, Attachment; discovered rows synthesize `filesystem-discovery` and `unbound` | inventory rows must carry exact WorkstreamKey and provenance; unbound rows remain observation-only/quarantined | shared runtime inventory + Pi adapter | `correct_identity_then_extract` | duplicate Session IDs in two Workstreams never dedupe together; unbound discovery grants no authority | Desktop and Pi consume same typed inventory with isolation proof | current dedupe is only `sessionId + attachmentId` |
| `apps/pi-extension/src/mission-canvas-view.ts` | Composes rich Pi canvas read model and panels | modified on preserved branch | Pi surface/reference/layout/performance | receives project/continuity and surface/session projections | consume shared Workstream-aware view model; retain terminal-specific layout only | Pi compatibility projection | `compatibility_projection` | bounded Pi view shows exact bound Workstream and no foreign data | Desktop parity for semantics plus explicit Pi compatibility acceptance | no new rich panels |
| `apps/pi-extension/src/mission-canvas-widget.ts` | Work Rail lifecycle, polling, scoped refresh, current packet projection | modified on preserved branch | scoped refresh, project binding, widget tests | process-wide module state; current project binding; CWD; continuity; active Workpoint | key polling/cache/signature by AttachmentKey; exact Workstream checks on every response | Pi extension | `keep_pi` | two simultaneous attachments cannot overwrite widget packet/signature; unbound fails closed | attachment-scoped lifecycle tests and Desktop control-plane parity | `latestUiContext`, `startupCwd`, semantic counters, and `lastWidgetSignature` are process-wide |
| `apps/pi-extension/src/work-rail-widget.ts` | Normalizes and renders tactical Work Rail | present on current main and branch | width test and Mission Canvas view tests | project root, continuity, Instance, Session, Attachment, Work Surface IDs; no Workstream | add WorkstreamKey and make aggregate/advisory modes provenance-only | shared semantic normalizer + Pi renderer | `correct_identity_then_extract` | same packet produces equivalent Desktop/Pi semantics; cross-Workstream packet rejected | shared normalizer stabilized and Pi width parity retained | rendering stays Pi-owned |
| `apps/pi-extension/src/commands.ts` | Mission Canvas commands, selection, refresh, actions | modified on preserved branch | project binding, shell, Pi surface | current binding, CWD/session state, continuity and active selections | command context must resolve exact AttachmentKey/WorkstreamContext; selection cannot create daemon authority | Pi extension and shared command graph | `correct_identity_then_extract` | GUI/CLI/agent invoke same typed operation and ambiguous commands return zero cognitive payload | typed operation parity and fail-closed tests | presentation open/close remains local |
| `apps/pi-extension/src/config.ts` | Pi interaction/display mode and local preferences | modified on preserved branch | mode precedence and project-binding tests | CWD/session-local configuration | keep display preferences noncanonical; any Workstream preference must be explicitly keyed | Pi extension | `keep_pi` | changing UI mode never changes canonical Workstream | config authority classification accepted | no broad rename |
| `apps/pi-extension/src/project-binding.ts` | Verifies project binding decision | current main/branch common | project-binding and cross-project tests | project root, CWD, remote/project evidence | return ScopeRef/ProjectRootKey and workspace binding candidate; never resolve Workstream from project alone | shared runtime binding | `correct_identity_then_extract` | same path across hosts/worktrees stays distinct; project binding grants no Workstream authority | ScopeRef and WorkspaceBinding contracts land | project binding remains necessary but insufficient |
| `apps/pi-extension/src/scoped-surface-refresh.ts` | Publishes/filters refresh receipts and truthful surface snapshots | current main/branch common | scoped refresh static/runtime tests | project root + continuity, startup CWD, latest receipt | receipt key must include WorkstreamId and Attachment where runtime-local; remove process-wide latest authority | shared runtime + Pi adapter | `correct_identity_then_extract` | receipt from another Workstream/Attachment cannot refresh current surface | Workstream-keyed event/receipt API is live | current `latestReceipt` is process-wide |
| `apps/pi-extension/src/semantic-surface-truth.ts` | Pure semantic truth normalization | current main/branch common | semantic integrity and Mission Canvas tests | largely payload/state based; caller scope is project+continuity | freeze typed Workstream-aware input before extraction | shared semantic package | `extract_shared` | pure deterministic fixtures pass in browser, Desktop, and Pi | typed input contract and package parity tests | safest pure extraction candidate after input freeze |
| `apps/pi-extension/src/mission-canvas-{shell,layout,tool,accessibility}.ts` | Pi TUI host lifecycle, terminal layout, tool projection, accessibility | branch-only relative to current main | shell/layout/accessibility/reference tests | Pi process/session and current UI context | preserve authentic Pi surface; route semantics through exact attachment-bound shared contracts | Pi extension | `keep_pi` | standalone and embedded Pi behavior stays usable without claiming primary Desktop ownership | Desktop semantic parity and compatibility matrix accepted | static Pi-authority wording test must transition, not be restored blindly |
| `crates/focusa-core/src/mission_canvas/*` | Composition model, reducer, resolver, memory, persistence | branch-only relative to current main | 15 focused Rust test markers; current run inconclusive | `MissionCanvasScope` is project root + continuity + Session + Attachment + optional Instance/subpath | replace canonical key with ScopeRef/WorkstreamId and typed AttachmentKey; preserve old records as migration input | shared/core runtime | `correct_identity_then_extract` | storage/replay isolation across two Workstreams; ambiguous legacy keys quarantine | Spec 158 identity/persistence foundations T021-T032 and rollback proof | current serialized scope is not permanent canonical identity |
| `crates/focusa-api/src/routes/mission_canvas.rs` | Mission Canvas composition API and scope parsing | branch-only relative to current main | API static test passes; Rust run inconclusive | required project root, continuity, Session, Attachment; optional Instance/subpath | use canonical WorkstreamContext extractor; compatibility route must explicitly resolve or fail closed | Focusa API | `correct_identity_then_extract` | REST/CLI/tool schemas agree and unbound response has zero foreign payload | T023/T050 contract parity | no daemon-global fallback may be introduced |
| `crates/focusa-api/src/routes/mission_canvas_surfaces.rs` | Work Surface list/mutation API | current main/branch common | surface static/e2e (e2e currently daemon-blocked) | scope/continuity-oriented surface bindings | exact WorkstreamKey and Attachment required for authority-bearing operations | Focusa API | `correct_identity_then_extract` | two Workstreams with same continuity remain isolated | runtime e2e and rollback evidence | inspect together with API route above |
| `apps/menubar/src/lib/components/MissionCanvasView.svelte` and `RuntimeView` | Menubar Mission Canvas/status content | current main/branch common | Spec96/Spec135 static tests | selected project/session/current runtime presentation | reduce to bounded status/handoff; no full primary Mission Canvas or remembered-selection authority | menubar | `retire_after_parity` | Desktop owns complete canvas; menubar opens/hands off exact Workstream safely | Desktop parity plus approved menubar compatibility matrix | current component delegates substantial content to `RuntimeView` |
| `apps/pi-extension/tests/mission-canvas*`, `work-rail*`, scoped refresh/project binding tests | Pi behavior and identity characterization | mixed current-main and branch-only | latest results in section E | mostly project root + continuity + Session/Attachment fixtures | add stable Workstream, workspace-binding, attachment and adversarial collision fixtures | test ownership follows shared/Pi boundaries | `preserve_as_is` | existing behavior recorded before extraction; new identity tests fail old fallback paths | replacement tests green in shared, Pi, browser, and native runs | preserve failing static firewall as migration evidence until disposition approved |
| `tests/spec135_mission_canvas*` | Repository contracts, generated parity, host/firewall, e2e | largely branch-only | 7 pass, 1 static fail, 1 daemon-blocked | Spec 135 terminology and old scope contracts | overlay Spec 158 ownership and milestone truth; do not mechanically rewrite generated artifacts | repository acceptance | `replace_generated` | regenerated contracts derive from approved Workstream-aware source and deterministic checks pass | source contracts approved, generator rerun bounded, migration matrix accepted | no lockfile regeneration required |
| `apps/pi-extension/tests/mission-canvas-uiai-server.mjs` and browser isolation fields | Browser proof harness/isolation projection | branch-only | layout/reference/performance coverage; no fresh UIAI session in audit phase | browser class + Work Surface/session; no Workstream | browser session capability must bind exact Workstream/Attachment; UIAI remains execution authority | Desktop preview harness + UIAI integration | `port_desktop` | same authored app runs browser/Tauri and browser evidence is Workstream-scoped | 5% UIAI/browser proof and later native parity | do not add Playwright |
| worktree/working-subpath fields across model, API, binding, and session paths | Distinguish runtime checkout topology | mixed | project-binding and session tests | path/subpath/worktree strings; host distinction incomplete | introduce WorkspaceBindingId under exact Scope/Workstream/Attachment relationships | shared runtime | `investigate` | identical paths on different hosts and sibling worktrees never share execution authority implicitly | T022 plus remote/worktree adversarial proof | other worktrees are evidence, not merge inputs |

## I. Spec 158 identity conflicts

1. `WorkSurfaceProjection` has no `workstreamId`; it treats project root + continuity as its scope and even uses continuity as a grouping fallback.
2. `MissionCanvasScope` persistence/API identity is project root + continuity + runtime metadata, not ScopeRef + stable WorkstreamId.
3. Session inventory deduplicates by SessionId + AttachmentId without WorkstreamKey and synthesizes unbound discovery identities.
4. Work Rail snapshots omit WorkstreamId and can normalize active packets using project/continuity alone.
5. Widget polling sends project root + continuity, accepts matching responses on only those fields, and stores process-wide `latestUiContext`, `startupCwd`, semantic counters, and signature.
6. Scoped refresh receipts and matching use project root + continuity and keep one module-global latest receipt.
7. Project binding and CWD are correctly treated as verification inputs in some paths but are still sufficient inputs for downstream Mission Canvas polling/projection.
8. SessionId, InstanceId, AttachmentId, and WorkSurfaceId appear in models but are not arranged under an exact Workstream-rooted AttachmentKey.
9. Menubar/runtime wording and selection center a current project/session; remembered UI selection must remain presentation-only.
10. UIAI browser isolation is represented as a string/class on a surface, not proven as an exact Workstream/Attachment capability binding.
11. Existing static contracts assert Pi-native Mission Canvas authority; Spec 158 now bounds Pi as compatibility projection, so the failing firewall requires governed replacement.
12. Core/API persistence keys serialize the incomplete legacy scope, creating migration/quarantine obligations before shared extraction.

No audited Mission Canvas type is ready to become permanent Desktop canonical identity without correction or exact Workstream resolution.

## J. Risks and blockers

- Preserved branch and current main are heavily divergent (122/56); merge/rebase is explicitly prohibited and would be high risk.
- Spec 158 foundation dependencies T021/T022/T023 are not yet available; T060 semantic extraction is blocked.
- Current core/API identity would reproduce the project-root + continuity defect if copied into Desktop.
- Rust toolchain contract is unresolved: this worktree has no `rust-toolchain.toml`; current `rustc`/Cargo are Homebrew 1.91.1. No additional toolchain was installed.
- Focused Rust tests could not finish compilation within 900 seconds; API native tests remain unproven.
- One GUI static firewall is stale against current source and one daemon e2e cannot establish the daemon.
- Combined Node Mission Canvas script has a process-exit/harness issue even though every constituent test passes independently.
- Existing menubar, work-loop, and main worktrees contain separate branch/local state; none may be silently adopted.
- `target/` is 16 GiB and Tauri target output is 1.7 GiB; resource pressure and stale Git lock cleanup were observed. Builds must remain bounded and use one approved cache/toolchain strategy.
- No browser preview or Tauri milestone was claimed during preservation/audit.

Recorded environment baseline:

- Node `v22.23.1`
- npm `10.9.8`
- Rust/Cargo `1.91.1` (Homebrew; not yet approved as project pin)
- Tauri manifest major version `2`
- macOS `14.7.6`

## K. Proposed first cleanup or extraction task

First perform a bounded identity-characterization slice, not broad extraction:

1. add typed fixtures for `ScopeRef`, `WorkstreamKey`, `ContinuityId`, `AttachmentKey`, and `WorkSurfaceId` behind a compatibility adapter;
2. add adversarial tests proving identical project root + ContinuityId cannot bind or deduplicate Work Surfaces across different WorkstreamIds;
3. add tests proving unbound filesystem Sessions and remembered UI/CWD selection are observation-only;
4. only then extract `semantic-surface-truth.ts` as the first pure shared helper, with Desktop/Pi fixture parity.

This is prework for T060 and must wait for the approved T021/T022 identity contracts. It does not add panels, rename files broadly, or mutate canonical state.

## L. Task-graph nodes recommended for assignment

- **Current MacBook audit stream:** T010 (preservation, completed locally pending review), T011 (divergent/local inventory, completed by this report pending review), T012 (freeze rich Pi expansion, active boundary).
- **Spec 158 foundation owner:** T020, then T021 and T022; T023 follows only after both identity types are approved.
- **Desktop shell owner in a clean reviewed base:** T070 (`Extract or version product-neutral Desktop shell`), currently ready and the correct implementation node for the 5% shell.
- **Mission Canvas adapter owner after dependencies:** T060 only after T011, T021, and T022; then T061 after T050 and T060.
- Do not assign T071/T072/T073, T080, T090, or removal nodes from this old worktree yet; their dependencies are blocked.

## M. Proposed 5% Focusa Desktop milestone scope

### Product slice

Create the real `apps/desktop/` application as the primary Focusa Desktop identity, using one authored SvelteKit workspace consumed by both browser preview and a Tauri 2 shell. This is T070 shell work, not a mock dashboard and not code inside the Pi extension.

### Included

- product-neutral workspace/package manifest and bounded navigation model;
- real Focusa Desktop app name, bundle identifier, icons/config provenance, and Tauri configuration;
- baseline application frame and navigation for the agreed workspace map;
- truthful daemon `unavailable`, `read-only`, and `connected/read-only` presentation states with no canonical writes;
- explicit scope/Workstream binding status area that reports unbound until an exact contract exists;
- local presentation-only CLI/typed-agent operations for open, status, and navigation inspection; no domain mutation;
- one SvelteKit dev/preview entry used unchanged by browser and native shell;
- accessible keyboard navigation, visible focus, responsive frame, and visible recovery state;
- one pinned Rust/toolchain decision recorded once before native build; no extra toolchain/bootstrap and no local release build.

### Excluded

- canonical Workstream mutation;
- daemon-global current/latest fallback;
- full Mission Canvas domain integration;
- Pi PTY embedding;
- updater/release packaging;
- lockfile regeneration unless a bounded dependency addition proves it necessary;
- Playwright or any browser authority other than UIAI Engine;
- release tags, pushes, GitHub Releases, or shipping artifacts.

### Required browser proof

- start the real shared SvelteKit preview;
- open/retain one UIAI Engine browser session;
- capture screenshot, responsive layout, accessibility snapshot, interactions, console diagnostics, and failed-network/daemon-unavailable behavior;
- record concise Evidence references and prove no duplicate canonical state.

### Required native proof

- build and open the complete development Tauri shell using the single approved pinned toolchain/cache strategy (never `cargo build --release`);
- capture full-window native screenshots and launch logs;
- prove app identity, frame/navigation, truthful daemon state, and absence of canonical mutation;
- record Node, package manager, Rust, Cargo, Tauri, and macOS versions in the milestone Evidence packet.

### 5% acceptance

The milestone is claimable only when the same authored application visibly runs in browser and Tauri, UIAI proof is clean or truthfully degraded, the full native shell opens, no critical 5% behavior is a hidden placeholder, and all Evidence is attached. This audit does not claim the 5% milestone.
