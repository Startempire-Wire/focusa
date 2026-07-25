# Focusa Locked Next-Dev Release Requirement Trace Matrix

**Status:** SCOPE LOCKED — IN PROGRESS — NO RELEASE PASS
**Lock date:** 2026-07-25
**Project:** Focusa (`/home/wirebot/focusa`)
**Release train:** `next-dev` until the canonical tag script assigns the immutable version
**Root Bead:** `focusa-vbcqu`
**First Workpoint Bead:** `focusa-vbcqu.1.1`
**Operator authority:** bundle all currently open Focusa issues and Beads plus the approved master Release Cycle implementation into one dev release; admit no unrelated scope after lock

## 1. Truth correction

The prior revision declared broad release requirements `PASS`. That is not current truth.

At scope lock:

- 14 GitHub issues remain open;
- 23 pre-existing Beads remain open;
- `v0.9.121-dev` is published but failed terminal/live completion and remains incident evidence;
- installed and running components have not proven parity with a successfully completed new candidate;
- Project Genesis, Trajectory Ladder integrity, Flow Kernel, surface parity, temporal authority, and master Release Cycle implementation remain incomplete;
- current runtime diagnostics still show project/scope guidance drift, capability-contract drift, missing Workpoint/work-loop authority, degraded awareness, and critical token/resource pressure;
- the current branch contains pre-existing uncommitted release/Silent Session work that must not be overwritten.

No row may become `PASS` until its implementation, tests, migration, exact-SHA evidence, and live acceptance are linked.

## 2. Immutable scope rule

Included after this lock:

1. every GitHub issue open at lock: `#15`, `#44`, `#45`, `#47`, `#48`, `#49`, `#50`, `#52`, `#53`, `#54`, `#55`, `#56`, `#58`, `#59`;
2. every Bead open at lock, listed in §7;
3. Focusa-owned gaps discovered before lock;
4. the approved master Release Cycle, Trajectory Ladder, Project Genesis, marker guard, HLT Impasse, deliberate inference, and Frictionless Project Flow Kernel;
5. migrations, compatibility, tests, documentation, evidence, release automation, deployment, rollback, telemetry, and polish necessary to satisfy the included scope.

After lock, only a defect or missing acceptance requirement necessary to satisfy an included item may join this release. Unrelated ideas are post-release backlog.

**Accepted post-lock closure item:** `focusa-vbcqu.6.3` adds intelligent evidence-backed release-page generation. It is admitted because the current generic template fails the included #56 master Release Cycle acceptance and obscures candidate/deployment truth; it is not unrelated feature scope.

**Accepted post-lock closure item:** `focusa-vbcqu.4.4` protects the frozen Spec135–135K Mission Canvas contract from unknowns introduced by included primitive/schema/authority/storage/event/workflow changes. It is necessary #52/#53/#59 compatibility acceptance, not unrelated feature scope.

The Focusa release may implement the global release-control kernel and UIAI adapter contract. Mutating the UIAI repository requires separate verified project authority and is not silently absorbed into this release.

## 3. Candidate and pipeline rule

`v0.9.121-dev` remains immutable failed-incident evidence. It is not rewritten or clobbered.

The new candidate is created only after exact-SHA acceptance using:

```bash
scripts/create-dev-release-tag.sh --base 0.9 --push
```

Required live chain:

```text
CI
→ Release
→ Deploy Live Daemon
→ audit
→ self-heal
→ watchdog
→ source/artifact/installed/running truth verification
```

Local release builds, local release-binary deployment, partial deployment shortcuts, mutable accepted candidates, and false `SHIPPED` declarations are prohibited.

## 4. Locked Trajectory Ladder

### HLT — High Level Trajectory

Bring Focusa to uncompromising MVP launch readiness with no cross-scope authority leaks or stale release binaries.

### MLG — Mid Level Goal

Implement and prove the locked Focusa next-dev release train across all included issue, architecture, migration, integration, UX, runtime, and release-control waves.

### STG — Short Term Goal

Complete W0 by producing the immutable requirement→issue→bead→code→test→evidence manifest, detailed implementation specification, compatibility/migration plan, call stacks, and final pre-implementation gap/acceptance review.

### Waypoints

1. **W0:** lock this trace manifest, detailed specification, call stacks, migrations, and acceptance matrix.
2. **W1:** implement Trajectory Ladder integrity, HLT Impasse/inference, temporal/prediction foundations, complete history/query/fallback, Waypoint migration, and marker guard.
3. **W2:** implement atomic Project Genesis, bootstrap/onboarding recovery, task-path decomposition, and first Workpoint creation.
4. **W3:** implement Frictionless Project Flow Kernel, continuous Workpoint advance, Pi ambient mode, toggleable complete Spec135 Mission Canvas journey, and surface parity.
5. **W4:** close Pi UX/menu/Footer, contract drift, awareness/preload, output/token, prediction/metacog, and other pre-lock runtime regressions.
6. **W5:** implement the master release orchestrator and close v0.9.121 incident requirements without rewriting its history.
7. **W6:** close or evidence-supersede every Bead open at scope lock.
8. **W7:** run integrated acceptance, dogfood the canonical next-dev pipeline, verify live truth, measure speed/friction/cost, and issue the final release decision.

### First Workpoint

```text
Bead: focusa-vbcqu.1.1
Action: build the locked requirement→issue→bead→code→test→evidence manifest
Target: docs/142-focusa-release-requirement-trace-matrix.md
Done: every locked requirement has an owner, dependency route, implementation/proof destination, disposition rule, and no-pass acceptance state
Status: COMPLETED — coverage and DAG validation passed
```

Canonical Focusa Workpoint persistence remains blocked by the included first-Workpoint coordination defect. The verified project, persisted Trajectory, operator steering, Bead, and this validated artifact supplied degraded execution authority without exposing internal coordination to the operator.

Next Workpoint candidate: `focusa-vbcqu.2.1` — implement Trajectory Ladder integrity after an isolated clean implementation workspace is authorized.

## 5. Release wave DAG

| Wave | Bead | Purpose | Gate | Current state |
|---|---|---|---|---|
| W0 | `focusa-vbcqu.1` | Scope/spec/closure matrix | First | IN PROGRESS |
| W1 | `focusa-vbcqu.2` | Trajectory/temporal/prediction/marker | W0 | BLOCKED |
| W2 | `focusa-vbcqu.3` | Genesis/bootstrap/onboarding/first Workpoint | W0 | BLOCKED |
| W3 | `focusa-vbcqu.4` | Flow Kernel/Canvas/surface parity/Spec135 compatibility | W0 | BLOCKED |
| W4 | `focusa-vbcqu.5` | Pi UX/contracts/runtime regressions | W0 | BLOCKED |
| W5 | `focusa-vbcqu.6` | Release orchestrator/incident closure | W0 | BLOCKED |
| W6 | `focusa-vbcqu.7` | Organizational ownership for all open-at-lock Beads | W0; leaf completion enforced by W7 acceptance | BLOCKED |
| W7 | `focusa-vbcqu.8` | Acceptance/dogfood/speed/final decision | W0 final review + every locked implementation/open-at-lock leaf | BLOCKED |
| Release | `focusa-vbcqu` | Locked next-dev train | W7 | BLOCKED |

`bd dep cycles` reports no dependency cycles.

## 6. Open GitHub issue mapping

| Issue | Owning Bead | Wave | Required disposition | Status |
|---|---|---|---|---|
| #15 Prediction/metacognitive authority | `focusa-vbcqu.2.3` | W1 | Implement, prove, close | OPEN |
| #44 Programmatic project binding/recovery | `focusa-vbcqu.3.3` | W2 | Implement, prove, close | OPEN |
| #45 Stale Canvas/advisory propagation | `focusa-vbcqu.4.3` | W3 | Implement, prove, close | OPEN |
| #47 Footer hints crash | `focusa-vbcqu.5.3` | W4 | Reproduce, fix, regression proof, close | OPEN |
| #48 Pi menu functional audit | `focusa-vbcqu.5.2` | W4 | Audit/fix/prove every surface, close | OPEN |
| #49 Pi menu IA/bloat | `focusa-vbcqu.5.1` | W4 | Rationalize, usability proof, close | OPEN |
| #50 Genesis/readiness engine | `focusa-vbcqu.3.2` | W2 | Implement, prove, close | OPEN |
| #52 Consolidation/dead-road removal | `focusa-vbcqu.4.2`, `focusa-vbcqu.4.4` | W3 | Consolidate, protect/amend Spec135 dependencies, remove dead roads, prove, close | OPEN |
| #53 Canvas-off terminal mode/toggle | `focusa-vbcqu.4.1` | W3 | Implement shared-state toggle/parity, close | OPEN |
| #54 Temporal authority | `focusa-vbcqu.2.2` | W1 | Implement across flow/release, prove, close | OPEN |
| #55 v0.9.121 incident | `focusa-vbcqu.6.2` | W5 | Preserve truth, close root causes with new release proof | OPEN |
| #56 Reusable release orchestrator | `focusa-vbcqu.6.1`, `focusa-vbcqu.6.3` | W5 | Implement generic kernel/adapters/metrics plus intelligent evidence-backed release pages, close | OPEN |
| #58 HLT/spec/tasks to first Workpoint | `focusa-vbcqu.3.1` | W2 | Implement atomic Project Genesis, close | OPEN |
| #59 Trajectory Ladder integrity | `focusa-vbcqu.2.1` | W1 | Implement complete closure/migration, close | OPEN |

## 7. Beads open at scope lock

W6 owns their disposition organizationally, and the W7 integrated-acceptance leaf directly depends on every item. None may disappear through cleanup or renaming without a linked supersession record and evidence.

| Bead | Locked responsibility | Status at lock |
|---|---|---|
| `focusa-w26jj.9.7.4` | Final strict no-pass release decision | OPEN |
| `focusa-w26jj.9.7.3` | Full release-candidate acceptance without publishing | OPEN |
| `focusa-w26jj.9.7.2` | Zero-deferral/forbidden-placeholder audit | OPEN |
| `focusa-w26jj.9.7.1` | Requirement-to-code-test-evidence trace matrix | OPEN |
| `focusa-w26jj.9.6.5` | Cross-platform integrated E2E | OPEN |
| `focusa-w26jj.9.6.4` | Offline/proxy/airgap/interruption E2E | OPEN |
| `focusa-w26jj.9.6.3` | Windows install→OTA→rollback E2E | OPEN |
| `focusa-w26jj.9.6.2` | macOS install→OTA→rollback E2E | OPEN |
| `focusa-w26jj.9.6.1` | Linux install→OTA→rollback E2E | OPEN |
| `focusa-w26jj.9.4.6` | Environment/dependency family gate | OPEN |
| `focusa-w26jj.9.3.5` | Onboarding/installer family gate | OPEN |
| `focusa-ux2qx.17` | Live bootstrapper parity | OPEN |
| `focusa-a6yq6.10.9` | Silent Sessions final proof | OPEN |
| `focusa-a6yq6.10.8` | Spec133 final report/pre-MVP gate | OPEN |
| `focusa-a6yq6.10.7` | Spec133 criterion/gap audit | OPEN |
| `focusa-a6yq6.10.5` | Spec133 cross-platform/real Pi proof | OPEN |
| `focusa-a6yq6.10.4` | Spec133 runtime E2E/security matrix | OPEN |
| `focusa-8305` | Contradictory unbound onboarding/init guidance | OPEN |
| `focusa-vfsrg` | Legacy Pi SDK migration | OPEN |
| `focusa-n68k` | Scope enforcement across middleware/Trajectory/Pi restore | OPEN |
| `focusa-pskm` | Spec114 observatory UI | OPEN |
| `focusa-nodn` | CLI/menubar/TUI/consumer scope migration | OPEN |
| `focusa-84px` | Tests/contracts/benchmarks/docs closure | OPEN |

Existing recovery Bead `focusa-yqoa2` remains in progress for v0.9.121 incident truth and is included through W6.

## 8. Construction requirements

The release implements—not merely documents—the accepted architecture in `/root/release-cycle/01-release-cycle-high-level-plan.md`:

- single project-level HLT authority;
- no lazy/generated/placeholder Trajectory;
- greenfield and brownfield HLT Impasse;
- deliberate evidence-scored MLG/STG/Waypoint inference;
- Waypoint-only Trajectory terminology and legacy migration;
- complete Trajectory event ledger, query, reconstruction, fallback, and supersession;
- ProjectIdentity + Trajectory marker footprint and integrity guard;
- atomic Project Genesis through task graph and first Workpoint;
- one ProjectFlowPacket and Frictionless Project Flow Kernel;
- continuous Workpoint verify/reassess/advance;
- Focus Stack/spec/task/evidence/release integration;
- Pi ambient baseline and toggleable complete Spec135–135K Mission Canvas journey over one substrate;
- reusable project-neutral release topology/profiles/state machine/adapters;
- immutable candidate, exact-SHA proof reuse, deployment truth, rollback, and speed/friction learning;
- a versioned `ReleaseIntelligencePacket` and release page that explain release-specific purpose, impact, scope, proof, unproven gaps, compatibility, artifacts, deployment truth, security, and measured deltas before commits, with anti-generic and unsupported-claim gates;
- a `Spec135CompatibilityPacket`, per-change impact classification, surgical amendment workflow, and Pi/Canvas/API/headless parity gate protecting the complete frozen 135–135K series.

## 9. Spec135–135K protected downstream gate

Every locked release leaf declares:

```text
spec135_impact: none | indirect | direct | unknown
affected_135_specs[]
affected_primitives[]
affected_schemas_apis_events_storage[]
affected_pi_canvas_agent_surfaces[]
compatibility_behavior
migration_behavior
required_doc_amendments[]
required_tests[]
agent_handoff_refs[]
```

`unknown` blocks implementation promotion and release acceptance.

A bounded `Spec135CompatibilityPacket` provides exact-SHA old/new semantic diff, affected frozen-manifest clauses/docs/primitives, ownership/authority/storage/event/API/tool changes, Pi/Canvas/headless impact, migration/rollback/toggle behavior, exact files/tests/evidence, unresolved blockers, implementation order, and rehydrate refs.

Surgical amendment law:

1. do not create Spec135L or another companion;
2. update `135-series-current-manifest.md` when authoritative delivery behavior changes;
3. update only directly/indirectly affected existing 135–135K docs;
4. preserve unaffected wording and frozen ordering;
5. record changed contract, reason, superseded wording, migration, implementation/test refs, and compatibility status;
6. regenerate dependent contracts/fixtures/docs;
7. deliver the packet and amended refs to the Mission Canvas agent before affected implementation;
8. require runtime/generated-contract/doc and Pi/Canvas/API/MCP/headless parity before closure;
9. expose Mission Canvas compatibility and unresolved proof honestly in intelligent release notes.

Owner: `focusa-vbcqu.4.4`; GitHub #52 amendment comment `5080241893`.

## 10. Pre-lock runtime gaps

Owned by `focusa-vbcqu.5.4` unless an included issue proves a more specific owner:

- verified project still receives broad/unsafe-root guidance;
- static/live tool contracts disagree;
- agent preload awareness can degrade to zero visible lines;
- repeated context/tool output causes attention and token pressure;
- `focusa_metacog_doctor` can return `SCOPE_REQUIRED` without a project-scope input;
- `focusa_predict_record` can return HTTP 400 despite explicit verified scope;
- first Workpoint/internal coordination has a circular bootstrap dependency;
- LowMem/resource recovery can obscure core project flow.

## 11. Protected pre-existing work

At lock, branch `local/work-loop-completion` is at `760596d8794cff4201bafd6e0e70dd2ce7e89647`, tracks its remote with zero divergence, and is 16 commits behind `origin/main` (`a7574e4b71f4e925d2de01764a33b19158883fcf`).

Pre-existing uncommitted paths:

- `.beads/issues.jsonl`;
- `.github/workflows/release.yml`;
- `.github/workflows/spec132-terminal-matrix.yml`;
- `crates/focusa-core/src/silent_sessions/mod.rs`;
- `release-proof/audit/self-heal-result.json`.

The release-train work may append its authorized Beads records but must not overwrite, stash, restore, rebase across, or absorb the other source/workflow/proof changes without ownership and exact diff review.

## 12. Detailed-spec numbering gap

Two different top-level files currently use number 142:

- `142-focusa-release-requirement-trace-matrix.md`;
- `142-focusa-seamless-pi-continuation-and-workflow-dependency-onboarding-spec.md`.

The detailed master Release Cycle implementation spec will use the next available number, **143**, and must record how the duplicate-142 naming conflict is corrected without breaking references.

## 13. W0.3 implementation-readiness findings

| Verified current-state gap | Evidence | Locked owner/closure |
|---|---|---|
| Dual Trajectory terminology/data shape | 44 source references to milestone across core/API/TUI; `TrajectoryProjectionRecord` has string `waypoints` plus rich `milestones` | `focusa-vbcqu.2.1`; typed Waypoint migration, compatibility read, static/runtime absence gate |
| Lazy/fallback lower-level projection | `trajectory.rs` uses ordered `first_nonempty` and fallback candidate derivation | `focusa-vbcqu.2.1`, `.3.1`; committed inference events only, HLT Impasse, forbidden-pattern tests |
| Fragmented history | four HLT JSONL ledgers, 10 records total, no complete Ladder event/query ledger | `focusa-vbcqu.2.1`; unified append-only ledger, dual-read/shadow verification, HLT projection |
| Identity-only marker | `.focusa-project.json` is `focusa.project.v1` without Trajectory binding/guard | `focusa-vbcqu.2.1`, `.3.1`; additive atomic marker migration and rollback |
| Scope/authority recovery drift | stale OVH Workpoint/gap survived current-state change; project-aware tools can still report broad cwd or `SCOPE_REQUIRED` | `focusa-vbcqu.2.1`, `.3.3`, `.5.4`; explicit project/continuity/revision binding and current-ask non-authority |
| Static/live tool-contract drift | Tool Doctor reports missing/stale live contracts; implementation/spec audit parser rejects existing object spread | `focusa-vbcqu.5.4`; repair parser/registry and regenerate Spec141 contracts before W7 |
| Stale deployment truth | source/version gate is `0.9.121-dev`; local daemon reports `0.9.120-dev` | `focusa-vbcqu.6.2`, `.8.1`; exact-SHA source/artifact/installed/running reconciliation |
| Spec135 downstream risk | core schemas/events/operations feed frozen Mission Canvas contracts | `focusa-vbcqu.4.4`; UNKNOWN blocker, packet, surgical amendments, Pi/Canvas/headless parity |

Feasibility baseline: 10/10 local requests returned HTTP 200; `trajectory view` observed 0.067–0.632 seconds and 50-record HLT history 0.002–0.042 seconds. Spec143 §18 locks measured p95/LowMem/write-lock budgets and W7 regression reporting.

Coverage check: all 14 currently open GitHub issues exactly match 14 manifest rows; the locked Bead family contains 32 items, has no dependency cycles, and only W0.3 is in progress. These counts are evidence, not completion claims.

## 14. Acceptance and no-pass rule

The release is not done if any of these is true:

- a locked issue or Bead lacks a proven disposition;
- architecture exists only as prose without runtime, migration, tests, and evidence;
- any surface disagrees on Project, Trajectory, Workpoint, task, evidence, or release state;
- HLT/Ladder content is lazily generated or placeholder-backed;
- milestone remains a canonical Trajectory term;
- bootstrap requires manual internal protocol choreography;
- spec/task/evidence/Focus Stack/release integration drifts;
- exact-SHA acceptance is incomplete;
- release artifacts are mutable or built/deployed outside the canonical pipeline;
- live deployment, install, browser/API smoke, rollback, audit, self-heal, or watchdog proof is missing;
- source, artifact, installed, and running versions differ;
- speed/friction/cost evidence is absent;
- `SHIPPED` is declared before all terminal truth is green.

## 15. Evidence status

| Evidence | Current state |
|---|---|
| Scope lock epic/DAG | CREATED — `focusa-vbcqu` |
| GitHub issue mapping | PASS — all 14 currently open issues exactly match 14 manifest rows |
| Locked acceptance dependencies | PASS — 19 direct W7 blockers; all listed dependencies open/visible and acyclic |
| Locked Bead family | PASS — 32 items, no dependency cycles; W0.3 is sole in-progress item |
| Focusa project verification | PASS — high confidence `/home/wirebot/focusa` |
| Locked Trajectory MLG/STG/Waypoints | PERSISTED — W0 scope/spec/readiness contract frozen |
| Canonical first Workpoint | BLOCKED by included Genesis coordination defect |
| Degraded first Workpoint authority | COMPLETED — `focusa-vbcqu.1.1`; coverage and DAG validation passed |
| Detailed implementation spec | COMPLETE / UNDER W0.3 REVIEW — `focusa-vbcqu.1.2`; Spec143 contract created and gated |
| Intelligent release-page requirement | LOCKED — `focusa-vbcqu.6.3`, GitHub #56 comment `5080213361` |
| Spec135 compatibility safeguard | LOCKED — `focusa-vbcqu.4.4`, GitHub #52 comment `5080241893` |
| Final gap/migration/security review | PASS — `focusa-vbcqu.1.3`; live gaps assigned, budgets/threats/migrations locked |
| Docs/runtime and version gates | PASS — runtime parity; source version consistency `0.9.121-dev` |
| Tool implementation/spec audit | BLOCKED — existing parser rejects `...PRELOAD_TOOL_CONTRACTS`; owner `.5.4` |
| Source implementation waves | READY BY CONTRACT; workspace blocked by protected uncommitted branch 16 commits behind `origin/main` |
| Integrated acceptance | BLOCKED by W1–W6 |
| New candidate | NOT CREATED |
| Live deployment truth | NOT PROVEN |
| Final release decision | BLOCKED |
