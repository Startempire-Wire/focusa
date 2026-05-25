# Tauri Menubar Up-to-Speed Spec

**Product:** Focusa Tauri menubar app (`apps/menubar`)  
**Target runtime:** Focusa `v0.9.12-dev` current state  
**Status:** implementation spec  
**Source audit:** [`TAURI_MENUBAR_FUNCTIONALITY_AUDIT.md`](TAURI_MENUBAR_FUNCTIONALITY_AUDIT.md)

## 1. Goal

Ship a Tauri menubar app that acts as a compact **Focusa Runtime Cockpit**: an operator can see whether the current agent/project is oriented, canonical, evidence-backed, healthy, and safe to continue without falling back to CLI for routine state checks.

The app is not a replacement for Pi or the CLI. It is the always-visible control/status surface for Focusa's current cognitive state.

## 2. Product principles

1. **Project-scoped first.** Every primary view is anchored by `project_root + continuity_id`; global Focus State is secondary.
2. **Canonical over narrative.** Workpoint/Trajectory packets beat transcript tail or legacy state dumps.
3. **Evidence visible by default.** Important statuses should show supporting evidence refs/handles.
4. **Hot path stays bounded.** Poll lightweight health/readiness routes; deep diagnostics are explicit user actions.
5. **Safe mutations only.** Any write operation shows confirmation, payload summary, and returned `tool_result_v1` status.
6. **Degraded is a first-class state.** `pending`, `blocked`, `canonical=false`, `degraded=true`, writer conflicts, and retry posture are visible, not hidden.

## 3. Target navigation

Replace current tabs (`Focus`, `Mission`, `Gate`, `Sync`, `Settings`) with this cockpit layout:

| Tab | Purpose | Primary routes |
|---|---|---|
| **Cockpit** | One-screen health/orientation summary. | `/v1/health`, `/v1/doctor`, `/v1/project/identity`, `/v1/trajectory/view`, `/v1/workpoint/resume`, `/v1/work-loop/health`, `/v1/telemetry/memory` |
| **Trajectory** | Project goals, current state, active gap, drift boundaries, proposed next Workpoint. | `/v1/trajectory/view`, `/v1/trajectory/assess`, `/v1/trajectory/propose-workpoint`, `/v1/trajectory/checkpoint` |
| **Workpoint** | Canonical continuation packet, next action, blockers, target objects, drift check, linked evidence. | `/v1/workpoint/resume`, `/v1/workpoint/current`, `/v1/workpoint/drift-check`, `/v1/workpoint/active-object/resolve`, `/v1/workpoint/evidence/link` |
| **Proof** | Evidence, predictions, metacognition, lineage/snapshots in one proof/recovery area. | `/v1/traverse`, `/v1/predictions/recent`, `/v1/predictions/stats`, `/v1/metacognition/status`, `/v1/metacognition/evaluations/recent`, `/v1/focus/snapshots/recent`, `/v1/lineage/tree` |
| **Work Loop** | Dispatch readiness, writer ownership, active task, pause/degraded state, checkpoint history. | `/v1/work-loop/health`, `/v1/work-loop/status?summary_only=true`, `/v1/work-loop/checkpoints`, `/v1/work-loop/context`, `/v1/work-loop/checkpoint` |
| **Focus/Gate** | Legacy Focus State and Focus Gate, with safe update/suppress/pin controls. | `/v1/focus/frame/current`, `/v1/focus/stack`, `/v1/focus/update`, `/v1/focus-gate/candidates`, `/v1/focus-gate/*` |
| **Sync/Settings** | Connection, sync peers, API base, build/runtime version, security guidance. | `/v1/sync/*`, `/v1/info`, `/v1/ontology/tool-contracts`, `/v1/resource/mode` |

## 4. Cockpit tab requirements

### 4.1 Header status strip

Show these chips at all times:

| Chip | Good state | Warning/bad state |
|---|---|---|
| Daemon | `/v1/health.ok=true` | unavailable, version unknown |
| Project | identity `status=verified` | ambiguous/broad root/mismatch |
| Trajectory | active goal + gap present | missing/unclear trajectory |
| Workpoint | resume `canonical=true` | pending, blocked, degraded, scope mismatch |
| Work Loop | `dispatch_ready=true` or equivalent readiness | paused, writer conflict, degraded, boundary reason |
| Memory | normal pressure | LowMem/emergency pressure |

### 4.2 Cockpit cards

Required cards:

1. **Project Identity**
   - canonical name, project id, root, confidence, repo remote.
   - action: verify current project root.
2. **Trajectory Gap**
   - long-term goal, current short-term goal, current state, gap, recommended action.
   - action: assess trajectory.
3. **Workpoint Resume**
   - canonical/degraded, mission, current action, next action, blockers, target objects, evidence refs.
   - action: drift check.
4. **Dispatch Readiness**
   - dispatch ready, boundary reason, pause flags, writer owner, degraded transport.
   - action: open Work Loop tab.
5. **Health/Doctor**
   - daemon health, doctor summary, memory RSS/pressure, route budget warnings.
   - action: run doctor refresh.
6. **Recent Proof**
   - latest evidence/prediction/metacog evaluation/snapshot handles.
   - action: open Proof tab.

## 5. Trajectory tab requirements

### Read-only baseline

- show active Trajectory ID, source, supersession state.
- show long-term goal, desired end state, current short-term goal.
- show current verified state, gap, recommended next action.
- show required evidence refs, required checks, acceptance risks, `not_done_if` boundaries.
- show advisory Workpoint candidate if available.

### Safe actions

| Action | Guard | Route |
|---|---|---|
| Assess current state | User confirms observed state/evidence refs. | `POST /v1/trajectory/assess` |
| Propose Workpoint | Read-only proposal first. | `POST /v1/trajectory/propose-workpoint` |
| Checkpoint trajectory | Confirmation + summary. | `POST /v1/trajectory/checkpoint` |
| Define/update goal | High-friction confirmation; show operator-confirmed boundary. | `POST /v1/trajectory/define-goal` |

## 6. Workpoint tab requirements

### Resume packet display

Show the Workpoint as a continuation contract:

- packet id, Workpoint id, project root, continuity id, session id.
- canonical/degraded status, warnings, scope recovery hints.
- mission, current action, next action.
- target objects, active object refs.
- blockers and do-not-drift rules.
- verified evidence refs.
- next-tools/hints when present.

### Workpoint tools

| Action | Guard | Route |
|---|---|---|
| Resume refresh | Safe read. | `POST /v1/workpoint/resume` |
| Drift check | Safe read/write-free verification. | `POST /v1/workpoint/drift-check` |
| Resolve active object | Input hint required. | `POST /v1/workpoint/active-object/resolve` |
| Link evidence | Confirmation, target ref + evidence ref required. | `POST /v1/workpoint/evidence/link` |
| Create checkpoint | Confirmation, all required fields previewed. | `POST /v1/workpoint/checkpoint` |

## 7. Proof tab requirements

Unify evidence, prediction, metacognition, and lineage because these answer: “Why should we trust this state?”

### Sections

1. **Evidence**
   - recent evidence refs linked to active Workpoint/project.
   - capture/link form for target ref, result, evidence ref.
2. **Predictions**
   - recent predictions with confidence, status, evaluation score.
   - calibration stats.
   - evaluate action for selected prediction.
3. **Metacognition**
   - status/caps/eviction telemetry.
   - recent reflections, adjustments, evaluations.
   - highlight promoted learning from successful evaluations.
4. **Lineage/Snapshots**
   - current lineage head, recent snapshots, diff action.
   - restore is displayed as CLI guidance only unless explicitly approved in a later design.

### Routes

- `/v1/predictions/recent`
- `/v1/predictions/stats`
- `/v1/metacognition/status`
- `/v1/metacognition/reflections/recent`
- `/v1/metacognition/adjustments/recent`
- `/v1/metacognition/evaluations/recent`
- `/v1/focus/snapshots/recent`
- `/v1/focus/snapshots/diff`
- `/v1/lineage/head`
- `/v1/lineage/tree`
- `/v1/traverse`

## 8. Work Loop tab requirements

### Read surfaces

- `/v1/work-loop/health` polled on hot path.
- `/v1/work-loop/status?summary_only=true` polled less frequently or on tab open.
- `/v1/work-loop/checkpoints` on tab open.
- deep status/replay surfaces only behind explicit “Deep diagnostics” button.

### Display

- dispatch readiness.
- active work item/task id.
- writer owner and conflict state.
- pause flags and boundary reason.
- degraded status and transport degradation.
- last heartbeat/checkpoint.

### Mutations

Default mode is read-only. Mutations require explicit confirmation and show result envelope:

- pause/resume/stop preflight first.
- context update.
- manual checkpoint.
- select next ready work.

## 9. Focus/Gate tab requirements

Keep legacy Focus State but make it current and useful:

- display active frame current fields: intent, current_focus/current_state, decisions, constraints, failures, next steps, open questions, recent results, notes.
- display Focus Stack depth and active path.
- display Gate candidates/signals from direct routes, not only `/v1/state/dump`.
- support safe `focus/update` writes for single-slot additions.
- support gate suppress/pin/surface with confirmation and result envelope.

## 10. Sync/Settings requirements

### API client unification

- `SyncPanel` and `AddPeerModal` must use shared `getApiUrl()` / `fetchJson()`; no hardcoded `http://127.0.0.1:8787` outside defaults.
- all fetches use timeout, structured error handling, and envelope normalization.

### Build/version

- show app package version, Tauri config version, daemon version, runtime snapshot string.
- update menubar version from stale `0.9.9` to the chosen app release version.
- decide Bun-only or npm parity:
  - Bun-only: remove stale npm lock or document npm unsupported.
  - npm parity: align Vite/Svelte plugin versions so `npm ci` passes.

### Security

- remote daemon guidance should prefer SSH tunnel.
- direct `0.0.0.0:8787` guidance must warn about network exposure.
- replace `csp: null` with a minimal compatible CSP if Tauri/Svelte build allows it.

## 11. Client architecture

### Stores

Create explicit stores instead of one loose runtime blob:

- `connectionStore`
- `projectStore`
- `trajectoryStore`
- `workpointStore`
- `proofStore`
- `workLoopStore`
- `healthStore`
- `focusGateStore`
- `syncStore`

### API layer

`src/lib/api.ts` should expose:

- `getApiUrl()`
- `setApiUrl(url)`
- `fetchJson(path, options)`
- `postJson(path, body, options)`
- `normalizeToolResult(payload)`
- `isDegraded(payload)`
- `summarizeError(error)`

Every card should render normalized fields when present:

- `status`
- `canonical`
- `degraded`
- `failure_class`
- `retry`
- `side_effects`
- `evidence_refs`
- `next_tools`

### Polling model

| Frequency | Surfaces |
|---|---|
| 2s | `/v1/health`, `/v1/work-loop/health` |
| 5s | project identity, trajectory view, Workpoint resume/current, memory telemetry |
| 10s | doctor, predictions stats, metacog status, snapshots, events |
| on tab open | deep work-loop, lineage tree, recent lists, sync peers |
| manual only | write routes, deep diagnostics, snapshot diff/restore |

## 12. Implementation phases

### Phase 0 — Build contract repair

- Make `bun run check` reliable from clean checkout.
- Resolve npm lock conflict or document Bun-only.
- Add static CI script for menubar build/check expectations.

**Acceptance:** clean checkout can run documented app proof without order-dependent failure.

### Phase 1 — Cockpit MVP

- Central API client.
- Cockpit tab with health, doctor, project identity, trajectory view, Workpoint resume, work-loop health, memory telemetry.
- Result envelope badges.

**Acceptance:** Cockpit answers project, trajectory, Workpoint canonicality, dispatch readiness, and memory pressure in one view.

### Phase 2 — Workpoint + Trajectory depth

- Full Trajectory tab.
- Full Workpoint tab.
- Drift check and evidence link flows.

**Acceptance:** operator can determine the exact next safe continuation action and supporting evidence without CLI.

### Phase 3 — Proof systems

- Prediction recent/stats/evaluate.
- Metacognition status/recent/evaluations.
- Lineage head/tree and recent snapshots/diff.

**Acceptance:** operator can inspect why state is trusted and what learning/prediction evidence supports it.

### Phase 4 — Work Loop controls

- Health/status/checkpoints.
- Preflighted pause/resume/stop/context/checkpoint/select-next.

**Acceptance:** app can safely show and control continuous work-loop posture without stealing writer ownership.

### Phase 5 — Polish and release readiness

- Version consistency.
- CSP/security hardening.
- App screenshots and macOS packaging proof.
- Docs update and release proof.

**Acceptance:** `bun install`, `bun run check`, `bun run build`, and Tauri package proof pass; docs match app behavior.

## 13. Non-goals for this version

- Full replacement for Pi extension tools.
- Editing arbitrary Focusa stores.
- Destructive snapshot restore from GUI by default.
- Background autonomous dispatch without explicit work-loop writer semantics.
- Cloud sync or hosted account system.

## 14. Final acceptance checklist

The app is “fully up to speed” when:

- [ ] project identity and trajectory are first-class surfaces.
- [ ] Workpoint resume packet is canonical source for continuation UI.
- [ ] evidence, predictions, metacognition evaluations, and lineage snapshots are visible.
- [ ] work-loop health dispatch readiness is visible on hot path.
- [ ] tool/result envelopes are normalized and shown consistently.
- [ ] all write actions are confirmation-gated and show canonical/degraded outcomes.
- [ ] API base is centralized; no stale hardcoded daemon URL except defaults.
- [ ] build/check/package proof is deterministic from a clean checkout.
- [ ] app version/runtime version/source-available boundary are current.
- [ ] remote connection guidance is tunnel-first and security-aware.
