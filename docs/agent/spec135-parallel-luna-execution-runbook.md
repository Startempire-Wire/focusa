# Spec 135 Parallel Luna Execution Runbook

**Status:** active through the direct Pi/Luna workaround

**Machine plan:** `docs/contracts/spec135-parallel-execution-plan.v1.json`

**Source graph:** `docs/transitions/FOCUSA-TRANSITION-001-mission-canvas-executable-callgraph.yaml`

## Purpose

This runbook accelerates the remaining Mission Canvas work without bypassing dependency, authority, evidence, Cargo, UIAI, or release gates. While Focusa issue [#132](https://github.com/Startempire-Wire/focusa/issues/132) is being repaired, `scripts/spec135-direct-luna-runner.py` launches isolated native Pi/Luna workers directly and does not call Work Loop or Silent Sessions.

## Current shape

The executable graph has 133 tasks: 27 complete and 106 remaining. The remaining graph resolves into 29 dependency waves.

The identity train remains the initial bottleneck:

1. `ID-001`
2. `ID-002`
3. `ID-003`, `ID-004`, and `ID-006` in collision-safe batches
4. `ID-005` and `ID-007`
5. `ID-008`
6. `ID-009`
7. `ID-010`

After `ID-010` is integrated, parallelism opens materially:

| Wave | Tasks | Collision-safe maximum |
|---:|---:|---:|
| 8 | 5 | 3 |
| 9 | 9 | 3 |
| 10 | 8 | 3 |
| 19 | 14 | 5 |
| 20 | 10 | 4 |
| 21 | 7 | 4 |

The machine plan is authoritative for the complete wave and batch inventory.

## Roles

### Scheduler

Work Loop admits only the next dependency-ready batch. It never treats `implemented_partial` as complete and never selects speculative dependent work.

### Workers

Each Luna Max worker receives exactly one task packet, one branch, one worktree, and one evidence destination. Workers do not update the central task graph.

### Integration writer

One writer owns graph status, generated packet regeneration, shared registries, generated contracts, and integration commits.

### Verifier

A separate read-only Luna Max session checks acceptance criteria, negative cases, evidence, and diff scope. It does not repair the worker branch it evaluates.

## Immediate direct workaround

Start, inspect, tail, or stop a native Luna worker without Focusa orchestration:

```bash
python3 scripts/spec135-direct-luna-runner.py start ID-001
python3 scripts/spec135-direct-luna-runner.py status
python3 scripts/spec135-direct-luna-runner.py tail ID-001
python3 scripts/spec135-direct-luna-runner.py stop ID-001
```

The runner uses `openai-codex/gpt-5.6-luna` with maximum thinking, disables extensions/skills/templates/themes, allows only `read,bash,edit,write`, and creates a dedicated branch, worktree, session directory, prompt, PID record, exit record, and durable log for each task.

## Returning to Focusa orchestration after issue #132

The direct workaround is active now. Before switching back to Work Loop/Silent Sessions after the fix merges, run one canary and prove:

1. requested Luna Max model equals the observed effective model;
2. project, Workstream, branch, worktree, and task packet are exact;
3. only one writer owns the task;
4. event tail and output cursors resume correctly;
5. pause, resume, interrupt, and cancellation target the current run generation;
6. checkpoint and completion receipts survive process exit;
7. completed status requires checks and evidence rather than process exit alone.

Only then set `activation_gate.focusa_orchestration_enabled` through a reviewed plan regeneration. The direct runner remains independent of that gate.

## Worker launch contract

Use the task object from the machine plan. The worker instruction is:

```text
Execute exactly TASK_ID from TASK_PACKET_REF under CARDINAL-135-SVELTE-001.
Work only in WORKTREE_SLOT on BRANCH.
Read the packet and every required source before editing.
Do not infer missing identity, bindings, paths, operations, or acceptance criteria.
Do not edit the central executable graph or another task's evidence.
Run only the packet's permitted checks; the pre-50% Cargo prohibition remains binding.
Write EVIDENCE_REF, commit the bounded change, checkpoint, and stop.
If a dependency, authority, or exact target is missing, record the blocker and stop without selecting another task.
```

The scheduler substitutes `TASK_ID`, `TASK_PACKET_REF`, `WORKTREE_SLOT`, `BRANCH`, and `EVIDENCE_REF` from `spec135-parallel-execution-plan.v1.json`.

## Batch execution

For each wave:

1. Integration writer records the integrated base commit.
2. Create one worktree per task in the next batch from that exact base.
3. Start bounded Luna Max workers concurrently up to the batch size.
4. Tail structured events; do not infer progress from terminal silence.
5. Each worker commits its change and evidence or checkpoints a typed blocker.
6. Verifier evaluates each result independently.
7. Integration writer cherry-picks verified commits in machine-plan order.
8. Resolve shared registry or generated-contract changes only in the integration worktree.
9. Run the wave's combined checks once.
10. Update task statuses, regenerate packets and this plan, and begin the next ready batch.

## Collision policy

Tasks sharing an ownership lane or concrete target path are separated into different batches. In particular:

- operation bindings sharing API route and registry owners are serialized;
- generated OpenAPI, clients, validators, and operation registries have one integration writer;
- profile registries and PTY registry surfaces remain single-writer;
- domain and acceptance tasks may run concurrently only when target paths are disjoint;
- workers never merge, rebase, push upstream, or perform release actions.

## Cargo and release

Parallel source implementation does not waive the Cargo restriction. Rust tasks remain partial until their declared Cargo checks are permitted and pass.

Official release remains blocked until:

- all 133 executable tasks are complete;
- every evidence artifact is present and verified;
- generated parity and schema checks pass;
- required Cargo and native PTY checks pass;
- UIAI visual/browser evidence passes;
- acceptance, cutover, polish, rollback, and receipt gates pass;
- the operator authorizes the release action separately.

## Regeneration and proof

```bash
python3 scripts/generate-spec135-parallel-execution-plan.py
python3 tests/spec135_parallel_execution_plan_test.py
python3 tests/spec158_mission_canvas_executable_callgraph_test.py
```
