# Focusa Agent Docs Index

This is the bounded, public-safe starting point for AI agents working in the Focusa repo. Use it before broad code changes or after context loss.

## 1. What Focusa is

Focusa is the local-first proof and continuity layer for AI coding agents. It keeps long-running work attached to a typed Workpoint, linked Evidence, and a next safe action so agents do not rely on chat tail memory.

**Current distribution boundary:** Focusa v0.9.142 still contains legacy self-issued Evaluation paths. New evaluator/customer distribution is release-blocked until Spec 152 mandatory authority-issued licensing, Spec 150A lifecycle binding, and applicable Spec 152A protected-component work are implemented. Agents must not restore, recommend, or infer a local Evaluation bypass.

## 2. Architecture map

| Layer | Purpose | Key locations |
| --- | --- | --- |
| CLI | Operator and agent command surface | `crates/focusa-cli/src/commands/` |
| API daemon | Local typed HTTP API and future canonical entitlement gate | `crates/focusa-api/src/routes/` |
| Core | reducers, Workpoints, Evidence, runtime state, persistence, lifecycle | `crates/focusa-core/src/` |
| Entitlement | current divergent implementations to be collapsed into signed-lease verifier | `crates/focusa-license/`, `crates/focusa-core/src/license.rs`, Spec 152 |
| Work loop + Silent Sessions | governed execution, durable runs, steering, receipts | `crates/focusa-core/src/silent_sessions/`, `docs/133-silent-sessions-final-release-proof.md` |
| Mission Canvas + Work Rail | scoped work surfaces, interviews, artifacts, generated UI | `docs/135-series-current-manifest.md`, `apps/menubar/` |
| Connectors + domains | provider-neutral context, auth lifecycle, software/domain projections | `crates/focusa-core/src/connectors.rs`, `docs/contracts/spec135/` |
| TUI / Mission Deck | terminal cockpit | `crates/focusa-tui/` |
| Pi extension | Focusa Pi tools, authority hooks, compaction/OTA/runtime bridge | `apps/pi-extension/` |
| Agent machine contracts | Pi/MCP/OpenAI/CLI/REST schemas and Agent Card | `docs/contracts/spec141/generated-capability-v2/` |
| Skills + runbooks | progressive agent onboarding and recovery playbooks | `.pi/skills/`, `apps/pi-extension/skills/` |
| Menubar preview | macOS/Tauri Mission Canvas and lifecycle cockpit | `apps/menubar/` |
| Protected components | proposed private workers/capsules for selected commercial functionality | Spec 152A plus private implementation repositories |
| Public docs | current reference, onboarding, lifecycle, and specs | `README.md`, `docs/`, `docs/current/` |

### 2.1 Current authority and recovery model

- Exact project authority is `project_root + continuity_id`; worktrees are verified working subpaths.
- Workpoint is immediate action authority; Trajectory supplies destination, current state, gap, and waypoints.
- Focus State is the bounded decision/constraint/failure journal, not a transcript replacement.
- Silent Sessions are daemon-native. Exact `session_id`, `run_id`, `generation`, approval, and idempotency values govern mutations.
- Proactive compaction preserves canonical Workpoint/Trajectory packets and queues governed automatic rollover after bounded transport exhaustion.
- Cache-safe context keeps stable prefixes and current user-tail authority while classifying degraded fallbacks explicitly.
- Mission Canvas binds Work Surfaces to canonical operations and project scope.
- Browser/UIAI actions require session/origin authority **and** an independent product/feature entitlement.
- Customer lifecycle requires verified install/repair, trusted update/rollback, and data-preserving uninstall.
- Missing/invalid/expired/revoked entitlement must become recovery-only, not Evaluation.

### 2.2 Licensing/onboarding authority fast path

Before touching install, license, Evaluation, protected modules, UIAI entitlement, or first-run code, read in order:

1. `docs/152-mandatory-authority-licensing-evaluation-entitlements-and-unified-onboarding-spec.md`
2. `docs/150a-spec152-entitlement-overlay-and-lifecycle-integration.md`
3. `docs/152a-protected-distribution-private-feature-capsules-and-anti-tamper-spec.md`
4. `docs/contracts/spec152-supersession-and-integration-matrix.v1.yaml`
5. `docs/current/INSTALLER_UPDATE_POLICY.md`
6. `docs/current/FIRST_RUN_FLOW.md`

Private authority/server implementation belongs in the private authority repository. Never place commercial caps, customer records, anti-abuse logic, raw authority proof, signing material, or admin URLs in this public tree.

### 2.3 All-Pi-tool and skill discovery

1. `focusa_agent_card` reports runtime tool count, installed skill/runbook inventory, interfaces, auth, and registry digest.
2. `focusa_tool_search` finds the narrowest capability without hot-loading every schema.
3. `focusa_tool_describe` cold-loads one strict contract; `focusa_tool_graph` or `focusa_tool_bundle` expands only the selected workflow.
4. `docs/contracts/spec141/generated-capability-v2/pi-tools.json` is the machine projection for every Focusa Pi tool.
5. `docs/focusa-tools/tools/focusa_<name>.md` is the human reference for each tool.
6. Load the matched `.pi/skills/<skill>/SKILL.md`, then its numbered runbook.
7. Future descriptors for mutable/execution tools must include product-qualified `license_feature` and optional limit metadata.

A release gate must prove runtime tool count = contracts = Pi descriptors = per-tool docs, installed skills/runbooks = packaged copies, and every mutable/execution surface = entitlement coverage ledger.

## 3. Canonical command surface

Start with read-only discovery:

```bash
focusa help all
focusa help migration
focusa project
focusa setup wizard --dry-run
focusa first-mission --project-root "$PWD" --dry-run --json
focusa status operator --json
```

Core continuity commands—only when canonical entitlement permits mutation:

```bash
focusa workpoint checkpoint --project-root "$PWD" --continuity-id demo --mission "Mission" --next-action "Next slice" --json
focusa workpoint evidence-link --target-ref tests --result "smoke passed" --evidence-ref "test:smoke" --json
focusa workpoint resume --project-root "$PWD" --continuity-id demo --copy-prompt
```

Background execution and lifecycle discovery:

```bash
focusa silent --help
focusa tui --headless-self-test
focusa update --help
focusa install --preflight --json          # no mutation; does not create Evaluation
bash scripts/install-focusa.sh --uninstall # preserves user data
focusa uninstall --dry-run --keep-data
```

Do not recommend `scripts/install-focusa.sh --eval` or equivalent current PowerShell/curl paths. They are legacy implementation surfaces and release blockers under Spec 152.

Target device-code commands described by Spec 152 are normative design, not shipped commands until implementation proof exists.

Safety and proof commands:

```bash
focusa action preflight --current-ask "change binary" --kind binary_replace --target /usr/local/bin/focusa --source github_release_asset --install-role live_build_host --project-root "$PWD" --json
focusa cleanup --safe --project-root "$PWD" --dry-run --json
scripts/guard-public-surface.sh
python3 tests/spec152_documentation_consistency_gate.py
bash tests/spec_cli_cross_phase_smoke_test.sh
```

## 4. API and daemon rules

- Default daemon URL: `http://127.0.0.1:8787`.
- Health route: `GET /v1/health`.
- Current license route: `GET /v1/license/status`; it must migrate to canonical signed-lease state.
- Workpoint resume route: `POST /v1/workpoint/resume`.
- Telemetry snapshot route: `GET /v1/telemetry/snapshot`.
- Project-scoped mutations require verified project scope and entitlement.
- Daemon-global advisory surfaces must say they are advisory and non-canonical.
- Recovery routes must remain available without active product execution rights.
- Authentication tokens, pairing, loopback, source checkout, and health do not imply entitlement.

## 5. Workpoints, Evidence, and Trajectory

- **Workpoint** is immediate continuation authority: mission, scope, current action, next action, blockers, and proof handles.
- **Evidence** is proof linked to the active Workpoint.
- **Trajectory** is advisory north-star context and does not override Workpoint or entitlement.
- **Context Authority** decides whether a proposed action matches task/project/environment/install role.
- **Entitlement Authority** separately decides whether product/feature/time/node/limit conditions permit execution.

Never treat transcript tail, UI state, local files, or agent memory as canonical project or license authority.

## 6. Update and release policy

- Use the canonical GitHub release pipeline for public artifacts.
- Keep CLI/daemon/TUI/Pi/menubar/worker/capsule contract versions coherent.
- Run focused tests, then broader smoke/coverage/adversarial gates.
- Public release gates include public-surface, cross-phase, Spec 150 lifecycle, Spec 152 documentation/entitlement, UIAI route coverage, and protected-component proof where applicable.
- Spec 150 `implementation_verified` proves lifecycle mechanics only until Spec 150A passes.
- Do not publish local-only runtime data or internal proof bundles.

## 7. Public/private boundary rules

Do not add:

- private host paths or admin URLs;
- secrets, tokens, keys, customer data, or full chat logs;
- local runtime databases, ledgers, pairing state, or raw authority evidence;
- internal launch strategy, anti-abuse signals, commercial calculations/caps;
- signing or capsule content keys;
- private proprietary worker implementation.

Use public-safe replacements:

| Unsafe category | Public-safe wording |
| --- | --- |
| host-specific paths | `~/projects/focusa-demo` or `$PWD` |
| backend/admin URLs | support or install/license public origin |
| full conversation dumps | bounded proof summaries or Evidence refs |
| license/customer records | public schema, synthetic fixtures, redacted status |
| proprietary module layout | stable public IPC/feature contract |

## 8. Software layout checklist for agents

1. `git fetch origin`
2. `git status --short --branch`
3. Read this index and the governing current/spec documents.
4. Identify/claim the active bead/work item.
5. Check the Spec 152 supersession matrix for conflicts.
6. Make the smallest scoped change.
7. Run focused proof and documentation consistency.
8. Update bead notes, commit, and push.

## 9. Helpful references

- Product overview: `README.md`
- Current CLI reference: `docs/current/CLI_REFERENCE_CURRENT.md`
- Entitlement authority: Spec 152
- Lifecycle integration: Spec 150A
- Protected distribution: Spec 152A
- Contradiction matrix: `docs/contracts/spec152-supersession-and-integration-matrix.v1.yaml`
- Public-surface guard: `scripts/guard-public-surface.sh`
- Cross-phase smoke: `tests/spec_cli_cross_phase_smoke_test.sh`
- Workpoint CLI: `crates/focusa-cli/src/commands/workpoint.rs`
- Project CLI: `crates/focusa-cli/src/commands/project.rs`
- API routes: `crates/focusa-api/src/routes/`
