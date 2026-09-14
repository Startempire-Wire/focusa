# Safe Self-Adaptive Compaction Policy Controller — Design (#112)

**Status:** IR1 design (control-system formalization). Implementation pending #294 patch packets.
**Issue:** Startempire-Wire/focusa#112

## Control loop

```text
typed runtime facts
→ capability and safety mask
→ finite validated policy lattice
→ shadow/off-policy evaluation
→ conservative adaptive selection
→ immutable epoch policy lease
→ measured outcome
→ promote, retain, quarantine, or rollback
```

## 1. Typed runtime facts

Inputs collected per epoch (turn-boundary snapshots, no continuous probing):

- provider + API adapter + model (from the active session);
- transport posture (websocket-cached / streaming / retry budget);
- task phase (preload, tool-loop, review, rollover);
- context-growth pattern (tokens/turn slope, prompt-cache hit rate);
- cache posture (prefix stability, dynamic-slice volatility);
- active Bloatgaurd intent (diet, firewall, compaction pressure).

## 2. Capability and safety mask

A pure function over facts producing a mask of legal controller actions:

- which policies the provider actually supports (tool-boundary compaction,
  native lifecycle, ASCC, prompt-rewrite, none);
- safety invariants that must hold regardless of policy (one queued
  compaction request per tool boundary, never mid-tool abort, keep the
  extension's restart-receipt semantics);
- read-only facts (facts never select a mutation policy directly).

## 3. Finite validated policy lattice

Policies form a partial order from conservative to aggressive:

```text
none < warn-only < native_lifecycle < tool_boundary_compaction
      < prompt_rewrite < ascc_pressure_route
```

Each transition has a precondition set (e.g., `tool_boundary_compaction`
requires the daemon lease + verified provider capability). Transitions are
compiled, not inferred: a static validator proves every edge against the
capability mask before the lattice is instantiated.

## 4. Shadow/off-policy evaluation

The controller continuously evaluates the next-more-aggressive policy in
shadow mode (no side effects): same facts, simulated outcome, recorded
against the active policy's measured outcome. Shadow results feed the
selection but can never trigger promotion by themselves.

## 5. Conservative adaptive selection

Selection moves at most one lattice edge per epoch, and only when:

- the target policy's shadow evaluation beat the active policy's measured
  outcome on the last N epochs (N configurable, default 5);
- the capability mask still permits the target;
- no regression was recorded for the target in the quarantine window.

## 6. Immutable epoch policy lease

The selected policy is sealed into an epoch lease (epoch id, policy,
facts digest, mask digest, selected-at, expires-at). During an epoch the
policy cannot change — drift between the lease's facts digest and current
facts forces a new epoch rather than a mid-epoch override.

## 7. Measured outcome + transitions

Each epoch records: outcome metrics (latency, cache hit rate, token
growth, error count, operator interruptions). Transition rules:

- `promote` — outcome beat the shadowed target for N epochs.
- `retain` — no significant difference.
- `quarantine` — outcome regressed; policy is banned for the quarantine
  window and the controller steps back one lattice edge.
- `rollback` — hard regression (crash, lifecycle violation); immediate
  step back + quarantine + receipt with evidence.

## Persistence + observability

- Epoch leases + outcome records persist in the daemon SQLite (bounded —
  retention follows the event-ledger window, doc 158).
- `focusa compaction controller-status --json` exposes the current lease,
  mask, shadow results, and quarantine set.
- Every transition writes a typed receipt (evidence refs for #277
  Completion Authority).

## Acceptance sketch

- Controller never mutates policy mid-epoch; every change is a sealed lease.
- No policy is selected that the capability mask rejects (compile-time +
  runtime double check).
- Injected regression forces rollback within one epoch with a receipt.
- Shadow evaluation produces zero side effects (audited by the event
  ledger).

## Non-goals

- Unconstrained ML tuning; provider-specific hacks; policy selection from
  raw transcripts.
