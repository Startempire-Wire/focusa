# Workstream-Rooted Canonical Runtime — Design (#125)

**Status:** IR1 design. Parent program #252; Workset ledger owner #267-#274.
**Issue:** Startempire-Wire/focusa#125

## Problem

Runtime state is still organized around ad-hoc singletons (FocusaState
global, session-scoped caches) rather than rooted, typed workstreams. Every
project, remote binding, and team session needs one canonical runtime root
that owns its state, compaction, evidence, and continuation — without a
daemon on every host.

## Design

### Workstream root

```json
{
  "schema": "focusa.workstream_root.v1",
  "workstream_id": "ptm-main",
  "root_scope": {
    "scope_kind": "remote",
    "remote_binding_id": "binding-ptm",
    "canonical_root": "/home/planmarr/plan-the-marriage",
    "working_subpath": null
  },
  "continuity": { "continuity_id": "ptm-main", "principal": "team:planmarr" },
  "runtime": {
    "state_ref": "workstreams/ptm-main/state.sqlite",
    "evidence_ref": "workstreams/ptm-main/evidence",
    "compaction_ref": "workstreams/ptm-main/compaction"
  }
}
```

- Each workstream gets a rooted runtime partition (state, evidence,
  compaction) under the daemon data dir — no more shared global state.
- `RemoteWorkspaceBinding` (docs/162) supplies the remote authority; the
  workstream root consumes bindings rather than duplicating transport facts.

### Canonical lifecycle

```text
bind remote workspace (#89)
→ create workstream root
→ typed runtime facts bootstrap (project identity, trajectory, workpoint)
→ per-workstream compaction policy (bounded by the #112 controller lease)
→ continuation/rollover keyed by workstream_id
→ settlement receipts per workstream (evidence-owned)
```

### Invariants

1. State mutations are workstream-scoped: a write must name the
   workstream root; cross-workstream writes require an explicit transfer
   receipt.
2. Compaction never mixes workstreams: prompts, summaries, and receipts
   stay inside their root.
3. Continuation resolves the workstream root first, the session second —
   a transcript tail can never be treated as authority.
4. Remote workstreams hold no authority state on the remote host —
   runtime truth lives on the controller.

### Execution slices (IR2+)

1. Workstream root type + persistence + CRUD route.
2. FocusaState partition per workstream (singleton elimination).
3. Compaction scoping to the root.
4. Continuation/rollover keyed by workstream.
5. Migration of existing projects into workstream roots (preview + apply).

### Acceptance sketch

- PTM + a second remote project run concurrently under one daemon with
  zero cross-workstream state bleed.
- Compaction of one workstream never touches another's context.
- Continuation after daemon restart resumes the exact workstream root.

### Non-goals

- No daemon on remote hosts (bindings remain controller-owned).
- No new product-wide singletons; this design REMOVES them.
