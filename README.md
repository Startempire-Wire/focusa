# Focusa

> **Local-first cognitive continuity and governance for AI agents.**
>
> Focusa helps coding agents remember what matters, recover after compaction, keep evidence attached to work, and make long-running sessions auditable instead of relying on fragile chat history.

**Current public snapshot:** `v0.9.0-dev`  
**Runtime state:** Rust daemon + HTTP API + CLI + Pi extension are implemented and live-tested.  
**Development state:** Focusa is still actively evolving; this README describes the current released snapshot, not a finished product.

---

## Why Focusa exists

Long agent sessions fail in predictable ways:

- **Conversation is mistaken for memory.** When the model context is compacted or overflows, decisions, constraints, evidence, and next steps become lossy prose.
- **The active task drifts.** Agents keep working, but not always on the same object, scope, or operator intent.
- **Proof gets buried in logs.** A test result, API response, or file path may be visible once and then disappear into transcript noise.
- **Learning is ungrounded.** Agents can record lessons, but without evidence, quality gates, or evaluation loops, those lessons become another pile of notes.
- **Autonomy is hard to trust.** The operator needs visible state, checkpoints, rollback points, and writer ownership instead of hidden memory writes.

Focusa was created to move durable meaning out of raw conversation and into typed, inspectable, local state.

---

## What Focusa is

Focusa is a local cognitive runtime that runs beside an agent harness such as Pi. It does not replace the agent or the model. It gives the agent structured memory, continuity, evidence handling, and governance surfaces.

In plain terms, Focusa gives an agent:

- a **current state of mind** (`Focus State`),
- a **continuation contract** after compaction (`Workpoint`),
- a way to **save proof without bloating prompts** (`Evidence` + handles),
- a **lineage/snapshot system** for recovery,
- a **metacognition loop** for reusable learning,
- a **work-loop control surface** with writer ownership,
- and a common result envelope so tools return predictable status, retry, evidence, and next-tool guidance.

Focusa is local-first. State lives on the machine running the daemon, under the project/data directory, and can be inspected through the CLI/API.

### What Focusa is not

- Not a model.
- Not a chatbot.
- Not a replacement for Pi, Claude Code, Codex, or other harnesses.
- Not a generic RAG system.
- Not a cloud memory service.
- Not finished or frozen; it is under active development.

---

## What a user can expect from Focusa-enhanced agents

When Focusa is working well, an agent should:

1. **Resume cleanly after compaction.** It should call `focusa_workpoint_resume` and continue from a typed packet instead of guessing from the transcript tail.
2. **Keep the current mission visible.** Intent, focus, constraints, failures, recent results, and next steps are stored in bounded fields.
3. **Preserve decisions and why they were made.** Decisions are concise architectural records, not buried paragraphs.
4. **Treat evidence as first-class.** Test output, API proof, release checks, and file references can be linked to the active Workpoint.
5. **Notice drift.** Workpoint drift checks can tell whether the agent is still working on the expected action/object.
6. **Avoid prompt bloat.** Large outputs become handles or evidence refs instead of raw transcript paste.
7. **Recover from uncertainty.** Tool results include status, retry posture, canonical/degraded state, and next-tool hints.
8. **Learn with discipline.** Metacognition tools include quality gates, evidence refs, and evaluation metrics instead of unconstrained note-taking.
9. **Respect ownership.** Work-loop mutation tools expose writer conflicts and preflight state instead of silently taking over.
10. **Remain inspectable.** The CLI/API expose state, lineage, snapshots, events, memory, ontology, Workpoints, and tool health.

---

## Current architecture snapshot

```text
Agent harness / Pi session
        │
        │ focusa_* tools, commands, lifecycle hooks
        ▼
Pi Focusa extension ── thin adapter, no parallel memory
        │
        │ HTTP JSON calls
        ▼
Focusa daemon / API (Rust)
        │
        ├─ Focus State: bounded current cognitive state
        ├─ Workpoint: typed continuation and evidence spine
        ├─ Core reducer: deterministic state transitions
        ├─ Ontology: objects, links, working sets, active refs
        ├─ Lineage / CLT: branch-aware context history
        ├─ Tree snapshots: recoverable state checkpoints
        ├─ Metacognition: capture/retrieve/reflect/adjust/evaluate
        ├─ Work-loop: continuous execution state and writer control
        ├─ ECS / references: externalized handles for large content
        └─ CLI/API parity surfaces
```

### Core crate: `focusa-core`

The core crate owns data types and reducer logic. Important current state includes:

- `FocusaState` — session, focus stack, Focus State, gate, memory, telemetry, ontology, Workpoint, and continuous work state.
- `FocusState` — bounded slots for intent, current focus, decisions, constraints, failures, next steps, open questions, recent results, notes, and artifacts.
- `WorkpointState` — active Workpoint ID, records, resume events, drift events, degraded fallbacks.
- `OntologyState` — proposals, objects/links/status changes, working-set refreshes, verification records, and delta log.
- `FocusaEvent` — reducer-owned event taxonomy for Focus State, ontology, Workpoint, continuous work, telemetry, memory, and related state transitions.

The reducer is the authority for state mutation. API routes and Pi tools should submit typed events or commands; they should not become alternate memory systems.

### API crate: `focusa-api`

The daemon exposes local HTTP endpoints under `/v1`. Current important namespaces include:

- `/v1/health`, `/v1/status`
- `/v1/focus/*`
- `/v1/workpoint/*`
- `/v1/work-loop/*`
- `/v1/lineage/*`
- `/v1/ontology/*`
- `/v1/metacognition/*`
- `/v1/threads/*`, `/v1/instances/*`
- `/v1/capabilities/*`
- telemetry, memory, ECS/reference, gate, proposals, autonomy, cache, and token surfaces

The Workpoint release path now waits for reducer-visible state before reporting success:

- `POST /v1/workpoint/checkpoint` returns `accepted` only after the new active Workpoint is visible to `/current` and `/resume`.
- `POST /v1/workpoint/evidence/link` returns `accepted` only after linked evidence is visible in Workpoint state.
- If reducer state has not materialized yet, the route returns `pending` with retry guidance instead of pretending the operation is complete.

### CLI crate: `focusa-cli`

The CLI is the operator/debug surface for the daemon. Current command domains include:

```text
start, stop, status, focus, stack, gate, memory, ecs, env, events,
turns, state, clt, lineage, autonomy, constitution, telemetry, rfm,
proposals, reflect, metacognition, ontology, skills, thread, export,
contribute, cache, workpoint, tokens, wrap
```

Most commands support human-readable output, and the top-level CLI supports `--json` for machine-readable workflows.

### Pi extension

The Pi extension is the main agent-facing integration. It registers 43 `focusa_*` tools grouped into these families:

- **Focus State:** scratch, decide, constraint, failure, intent, current focus, next step, open question, recent result, note.
- **Workpoint:** checkpoint, resume, link evidence, active object resolve, evidence capture.
- **Work-loop:** writer status, status, control, context, checkpoint, select next.
- **Tree/lineage:** head, path, snapshot, diff, restore, recent snapshots, compare latest, lineage tree, LI extraction.
- **Metacognition:** capture, retrieve, reflect, plan adjustment, evaluate outcome, recent reflections, recent adjustments, loop run, doctor.
- **State hygiene:** doctor, plan, approval-safe apply.
- **Tool doctor:** diagnostic entrypoint for Focusa tool readiness and likely recovery path.

Every `focusa_*` tool is expected to expose a common `tool_result_v1` result envelope with status, canonical/degraded flags, retry guidance, side effects, evidence refs, and next-tool hints.

---

## Workpoint continuity

A Workpoint is a typed continuation record. It preserves:

- mission / current ask,
- active object refs,
- action intent,
- verification records,
- blockers and drift boundaries,
- next slice / exact next action,
- canonical vs degraded state.

Use Workpoints whenever raw conversation becomes unreliable:

```text
Before compaction or risky handoff:
  focusa_workpoint_checkpoint

After compaction, model switch, fork, or uncertainty:
  focusa_workpoint_resume

After tests, release proof, API evidence, or file proof:
  focusa_workpoint_link_evidence
```

A non-canonical Workpoint is a fallback hint, not truth. The agent should say it is degraded and recover through a canonical Focusa read when possible.

---

## Metacognition and learning

Focusa metacognition is for reusable learning. It is not a dumping ground for every thought.

The loop is:

1. `focusa_metacog_capture` — store a reusable signal with rationale/evidence.
2. `focusa_metacog_retrieve` — search prior learning before planning.
3. `focusa_metacog_reflect` — generate hypotheses and strategy updates.
4. `focusa_metacog_plan_adjust` — turn reflection into a tracked adjustment.
5. `focusa_metacog_evaluate_outcome` — decide whether the adjustment improved results.

Spec89 added quality-gate details and suggested metrics so weak, vague, or low-evidence learning can be improved before it influences future behavior.

---

## State hygiene

Focusa state should be useful, not hoarded. The current hygiene tools are intentionally proposal-first:

- `focusa_state_hygiene_doctor` diagnoses stale or duplicate signals without mutation.
- `focusa_state_hygiene_plan` creates a proposed cleanup path.
- `focusa_state_hygiene_apply` is approval-gated and non-destructive in the current snapshot.

No Focusa tool should be silently deleted or demoted as a shortcut. Weak tools should be clarified, hardened, merged upward, or redesigned.

---

## Quick start

### Build and run locally

```bash
git clone <repo-url> focusa
cd focusa
cargo build --release -p focusa-api -p focusa-cli

# Start daemon in foreground
cargo run --bin focusa-daemon

# In another shell
cargo run --bin focusa -- status
cargo run --bin focusa -- workpoint current
```

Default API URL:

```text
http://127.0.0.1:8787
```

### Installed service pattern

A deployed local service typically runs:

```text
/home/wirebot/focusa/target/release/focusa-daemon
```

Health check:

```bash
curl -sS http://127.0.0.1:8787/v1/health | jq .
```

### CLI examples

```bash
# Daemon status
focusa status

# Current Workpoint
focusa workpoint current

# Resume packet
focusa workpoint resume

# Drift check
focusa workpoint drift-check \
  --latest-action 'release verify Spec89FocusaToolSuite live_api cli pi_tool' \
  --expected-action-type release_verify

# Ontology surfaces
focusa ontology primitives
focusa ontology world
focusa ontology slices
```

### API examples

```bash
# Health
curl -sS http://127.0.0.1:8787/v1/health | jq .

# Current Workpoint
curl -sS http://127.0.0.1:8787/v1/workpoint/current | jq .

# Resume Workpoint
curl -sS -X POST http://127.0.0.1:8787/v1/workpoint/resume \
  -H 'content-type: application/json' \
  -d '{"mode":"operator"}' | jq .
```

### Pi skill and tools

The current Focusa Pi skill lives in:

- project: `.pi/skills/focusa/SKILL.md`
- extension package: `apps/pi-extension/skills/focusa/SKILL.md`
- installed global copy: `~/.pi/skills/focusa/SKILL.md`

If Pi reports `description is required`, the skill is missing YAML frontmatter. A valid Focusa skill starts with:

```yaml
---
name: focusa
description: Use when preserving Focusa cognitive state, resuming after compaction/model switch/context overflow, linking evidence to Workpoints, using Focus State, work-loop, lineage/tree, metacognition, state-hygiene, or diagnosing Focusa tool readiness.
---
```

Validate skill loading with Pi's actual loader:

```bash
node --input-type=module - <<'NODE'
import { loadSkills } from '/opt/cpanel/ea-nodejs20/lib/node_modules/@mariozechner/pi-coding-agent/dist/core/skills.js';
const r = loadSkills({ cwd: process.cwd(), agentDir: '/root/.pi/agent', skillPaths: [], includeDefaults: true });
console.log(r.skills.map(s => [s.name, s.description.length, s.filePath]));
console.log(r.diagnostics);
NODE
```

---

## Current repository layout

```text
focusa/
├── README.md                         # GitHub-facing overview
├── Cargo.toml                        # Rust workspace
├── crates/
│   ├── focusa-core/                  # Reducer, state, event types, memory, workers
│   ├── focusa-api/                   # Local daemon / HTTP API binary focusa-daemon
│   ├── focusa-cli/                   # CLI binary focusa
│   └── focusa-tui/                   # TUI crate
├── apps/
│   └── pi-extension/                 # Pi integration and Focusa tools
├── docs/                             # Specs, evidence, audits, operator guides
├── tests/                            # Contract and live-stress scripts
└── .pi/skills/focusa/                # Project-local Focusa skill
```

Some older docs describe planned GUI/proxy/autonomy surfaces in more detail than the current runtime exposes. Treat those as design direction unless the README/current evidence says they are released in this snapshot.

---

## Current live proof

Current release proof is documented in:

```text
docs/evidence/SPEC89_REAL_RELEASE_LIVE_PROOF_2026-04-28.md
```

That proof verified a rebuilt installed daemon and CLI through direct live API/CLI/Pi tool probes, including:

- daemon health,
- Workpoint checkpoint/current/resume,
- Workpoint evidence link visible in resume,
- Focus State update,
- metacognition capture,
- work-loop status,
- CLI Workpoint current and drift-check,
- Pi `focusa_workpoint_resume`.

Proof marker:

```text
DIRECT_REAL_RELEASE_PROOF=PASS
```

---

## Design principles

1. **Meaning lives in typed state, not transcript luck.**
2. **Focusa is the cognitive authority; adapters stay thin.**
3. **Every important result should say whether it is canonical, degraded, retryable, or blocked.**
4. **Evidence should be linked, not pasted forever.**
5. **Agents should recover through explicit state reads, not memory vibes.**
6. **Operator steering wins over automation.**
7. **Local-first and inspectable beats hidden cloud memory.**
8. **Focusa remains evolvable; docs describe snapshots, not permanent completion.**

---

## Documentation map

Start here:

- `docs/README.md` — documentation index for the current snapshot.
- `docs/focusa-tools/README.md` — focused docs for every current `focusa_*` tool family, with tool descriptions and examples.
- `docs/focusa-tools/workpoint.md` — Workpoint checkpoint/resume/evidence/object-resolution tools.
- `docs/focusa-tools/focus-state.md` — Focus State and scratchpad tools.
- `docs/focusa-tools/work-loop.md` — continuous work-loop writer/status/control tools.
- `docs/focusa-tools/metacognition.md` — metacog capture/retrieve/reflect/adjust/evaluate tools.
- `docs/focusa-tools/tree-lineage.md` — lineage, tree, snapshot, diff, restore, and LI extraction tools.
- `docs/focusa-tools/diagnostics-hygiene.md` — tool doctor and state hygiene tools.
- `docs/SPEC89_HARDENED_FOCUSA_TOOL_OPERATOR_GUIDE_2026-04-28.md` — when to use each hardened Focusa tool.
- `docs/88-ontology-backed-workpoint-continuity.md` — Workpoint continuity design.
- `docs/89-focusa-tool-suite-improvement-hardening-spec.md` — current tool-suite hardening snapshot.
- `docs/evidence/SPEC89_REAL_RELEASE_LIVE_PROOF_2026-04-28.md` — released runtime proof.


### Current-build references

These docs describe only the current present build/snapshot surfaces:

- [`CHANGELOG.md`](CHANGELOG.md) — current snapshot change history.
- [`docs/current/CURRENT_RUNTIME_STATUS.md`](docs/current/CURRENT_RUNTIME_STATUS.md) — implemented runtime status and current limits.
- [`docs/current/API_REFERENCE_CURRENT.md`](docs/current/API_REFERENCE_CURRENT.md) — current API route inventory generated from route registrations.
- [`docs/current/CLI_REFERENCE_CURRENT.md`](docs/current/CLI_REFERENCE_CURRENT.md) — current CLI command inventory from `focusa --help`.
- [`docs/current/PI_EXTENSION_AND_SKILLS_GUIDE.md`](docs/current/PI_EXTENSION_AND_SKILLS_GUIDE.md) — Pi extension and skill locations/validation.
- [`docs/current/WORKPOINT_LIFECYCLE_GUIDE.md`](docs/current/WORKPOINT_LIFECYCLE_GUIDE.md) — current Workpoint usage and recovery flow.
- [`docs/current/TOOL_RESULT_ENVELOPE_V1.md`](docs/current/TOOL_RESULT_ENVELOPE_V1.md) — current structured tool result contract.
- [`docs/current/TROUBLESHOOTING_CURRENT.md`](docs/current/TROUBLESHOOTING_CURRENT.md) — current troubleshooting runbook.
- [`docs/current/VALIDATION_AND_RELEASE_PROOF.md`](docs/current/VALIDATION_AND_RELEASE_PROOF.md) — current validation and real runtime proof expectations.

### Individual Focusa tool docs

Each current `focusa_*` Pi tool has its own doc with purpose, usage guidance, example usage, expected result, recovery notes, and related tools.

| Tool | Family | Doc |
| --- | --- | --- |
| `focusa_scratch` | Focus State | [`docs/focusa-tools/tools/focusa_scratch.md`](docs/focusa-tools/tools/focusa_scratch.md) |
| `focusa_decide` | Focus State | [`docs/focusa-tools/tools/focusa_decide.md`](docs/focusa-tools/tools/focusa_decide.md) |
| `focusa_constraint` | Focus State | [`docs/focusa-tools/tools/focusa_constraint.md`](docs/focusa-tools/tools/focusa_constraint.md) |
| `focusa_failure` | Focus State | [`docs/focusa-tools/tools/focusa_failure.md`](docs/focusa-tools/tools/focusa_failure.md) |
| `focusa_intent` | Focus State | [`docs/focusa-tools/tools/focusa_intent.md`](docs/focusa-tools/tools/focusa_intent.md) |
| `focusa_current_focus` | Focus State | [`docs/focusa-tools/tools/focusa_current_focus.md`](docs/focusa-tools/tools/focusa_current_focus.md) |
| `focusa_next_step` | Focus State | [`docs/focusa-tools/tools/focusa_next_step.md`](docs/focusa-tools/tools/focusa_next_step.md) |
| `focusa_open_question` | Focus State | [`docs/focusa-tools/tools/focusa_open_question.md`](docs/focusa-tools/tools/focusa_open_question.md) |
| `focusa_recent_result` | Focus State | [`docs/focusa-tools/tools/focusa_recent_result.md`](docs/focusa-tools/tools/focusa_recent_result.md) |
| `focusa_note` | Focus State | [`docs/focusa-tools/tools/focusa_note.md`](docs/focusa-tools/tools/focusa_note.md) |
| `focusa_work_loop_writer_status` | Work-loop | [`docs/focusa-tools/tools/focusa_work_loop_writer_status.md`](docs/focusa-tools/tools/focusa_work_loop_writer_status.md) |
| `focusa_work_loop_status` | Work-loop | [`docs/focusa-tools/tools/focusa_work_loop_status.md`](docs/focusa-tools/tools/focusa_work_loop_status.md) |
| `focusa_work_loop_control` | Work-loop | [`docs/focusa-tools/tools/focusa_work_loop_control.md`](docs/focusa-tools/tools/focusa_work_loop_control.md) |
| `focusa_work_loop_context` | Work-loop | [`docs/focusa-tools/tools/focusa_work_loop_context.md`](docs/focusa-tools/tools/focusa_work_loop_context.md) |
| `focusa_work_loop_checkpoint` | Work-loop | [`docs/focusa-tools/tools/focusa_work_loop_checkpoint.md`](docs/focusa-tools/tools/focusa_work_loop_checkpoint.md) |
| `focusa_work_loop_select_next` | Work-loop | [`docs/focusa-tools/tools/focusa_work_loop_select_next.md`](docs/focusa-tools/tools/focusa_work_loop_select_next.md) |
| `focusa_state_hygiene_doctor` | Diagnostics / Hygiene | [`docs/focusa-tools/tools/focusa_state_hygiene_doctor.md`](docs/focusa-tools/tools/focusa_state_hygiene_doctor.md) |
| `focusa_state_hygiene_plan` | Diagnostics / Hygiene | [`docs/focusa-tools/tools/focusa_state_hygiene_plan.md`](docs/focusa-tools/tools/focusa_state_hygiene_plan.md) |
| `focusa_state_hygiene_apply` | Diagnostics / Hygiene | [`docs/focusa-tools/tools/focusa_state_hygiene_apply.md`](docs/focusa-tools/tools/focusa_state_hygiene_apply.md) |
| `focusa_tool_doctor` | Diagnostics / Hygiene | [`docs/focusa-tools/tools/focusa_tool_doctor.md`](docs/focusa-tools/tools/focusa_tool_doctor.md) |
| `focusa_active_object_resolve` | Workpoint | [`docs/focusa-tools/tools/focusa_active_object_resolve.md`](docs/focusa-tools/tools/focusa_active_object_resolve.md) |
| `focusa_evidence_capture` | Workpoint | [`docs/focusa-tools/tools/focusa_evidence_capture.md`](docs/focusa-tools/tools/focusa_evidence_capture.md) |
| `focusa_workpoint_checkpoint` | Workpoint | [`docs/focusa-tools/tools/focusa_workpoint_checkpoint.md`](docs/focusa-tools/tools/focusa_workpoint_checkpoint.md) |
| `focusa_workpoint_link_evidence` | Workpoint | [`docs/focusa-tools/tools/focusa_workpoint_link_evidence.md`](docs/focusa-tools/tools/focusa_workpoint_link_evidence.md) |
| `focusa_workpoint_resume` | Workpoint | [`docs/focusa-tools/tools/focusa_workpoint_resume.md`](docs/focusa-tools/tools/focusa_workpoint_resume.md) |
| `focusa_tree_head` | Tree / Lineage | [`docs/focusa-tools/tools/focusa_tree_head.md`](docs/focusa-tools/tools/focusa_tree_head.md) |
| `focusa_tree_path` | Tree / Lineage | [`docs/focusa-tools/tools/focusa_tree_path.md`](docs/focusa-tools/tools/focusa_tree_path.md) |
| `focusa_tree_snapshot_state` | Tree / Lineage | [`docs/focusa-tools/tools/focusa_tree_snapshot_state.md`](docs/focusa-tools/tools/focusa_tree_snapshot_state.md) |
| `focusa_tree_restore_state` | Tree / Lineage | [`docs/focusa-tools/tools/focusa_tree_restore_state.md`](docs/focusa-tools/tools/focusa_tree_restore_state.md) |
| `focusa_tree_diff_context` | Tree / Lineage | [`docs/focusa-tools/tools/focusa_tree_diff_context.md`](docs/focusa-tools/tools/focusa_tree_diff_context.md) |
| `focusa_metacog_capture` | Metacognition | [`docs/focusa-tools/tools/focusa_metacog_capture.md`](docs/focusa-tools/tools/focusa_metacog_capture.md) |
| `focusa_metacog_retrieve` | Metacognition | [`docs/focusa-tools/tools/focusa_metacog_retrieve.md`](docs/focusa-tools/tools/focusa_metacog_retrieve.md) |
| `focusa_metacog_reflect` | Metacognition | [`docs/focusa-tools/tools/focusa_metacog_reflect.md`](docs/focusa-tools/tools/focusa_metacog_reflect.md) |
| `focusa_metacog_plan_adjust` | Metacognition | [`docs/focusa-tools/tools/focusa_metacog_plan_adjust.md`](docs/focusa-tools/tools/focusa_metacog_plan_adjust.md) |
| `focusa_metacog_evaluate_outcome` | Metacognition | [`docs/focusa-tools/tools/focusa_metacog_evaluate_outcome.md`](docs/focusa-tools/tools/focusa_metacog_evaluate_outcome.md) |
| `focusa_tree_recent_snapshots` | Tree / Lineage | [`docs/focusa-tools/tools/focusa_tree_recent_snapshots.md`](docs/focusa-tools/tools/focusa_tree_recent_snapshots.md) |
| `focusa_tree_snapshot_compare_latest` | Tree / Lineage | [`docs/focusa-tools/tools/focusa_tree_snapshot_compare_latest.md`](docs/focusa-tools/tools/focusa_tree_snapshot_compare_latest.md) |
| `focusa_metacog_recent_reflections` | Metacognition | [`docs/focusa-tools/tools/focusa_metacog_recent_reflections.md`](docs/focusa-tools/tools/focusa_metacog_recent_reflections.md) |
| `focusa_metacog_recent_adjustments` | Metacognition | [`docs/focusa-tools/tools/focusa_metacog_recent_adjustments.md`](docs/focusa-tools/tools/focusa_metacog_recent_adjustments.md) |
| `focusa_metacog_loop_run` | Metacognition | [`docs/focusa-tools/tools/focusa_metacog_loop_run.md`](docs/focusa-tools/tools/focusa_metacog_loop_run.md) |
| `focusa_metacog_doctor` | Metacognition | [`docs/focusa-tools/tools/focusa_metacog_doctor.md`](docs/focusa-tools/tools/focusa_metacog_doctor.md) |
| `focusa_lineage_tree` | Tree / Lineage | [`docs/focusa-tools/tools/focusa_lineage_tree.md`](docs/focusa-tools/tools/focusa_lineage_tree.md) |
| `focusa_li_tree_extract` | Tree / Lineage | [`docs/focusa-tools/tools/focusa_li_tree_extract.md`](docs/focusa-tools/tools/focusa_li_tree_extract.md) |

### Focusa skills

The main `focusa` skill is the router and mental model. Focused companion skills provide progressive-disclosure playbooks for high-value workflows:

- `.pi/skills/focusa/SKILL.md` / `apps/pi-extension/skills/focusa/SKILL.md` — main Focusa router skill.
- `.pi/skills/focusa-workpoint/SKILL.md` — Workpoint recovery, evidence linking, drift-safe handoff.
- `.pi/skills/focusa-metacognition/SKILL.md` — reusable learning and quality-gated reflection.
- `.pi/skills/focusa-work-loop/SKILL.md` — continuous work-loop ownership/control.
- `.pi/skills/focusa-cli-api/SKILL.md` — daemon, CLI, API, release-proof operations.
- `.pi/skills/focusa-troubleshooting/SKILL.md` — degraded/offline/pending/blocked recovery.
- `.pi/skills/focusa-docs-maintenance/SKILL.md` — public docs, skills, evidence, and snapshot wording.

---

## License

Proprietary — Startempire Wire

Part of the Startempire Wire ecosystem.
