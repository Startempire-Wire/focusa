# Post-Cutoff Code Locus Map — 2026-04-13

Purpose:
- tie decomposition beads to exact current code surfaces
- prevent vague implementation beads with no locus
- identify where post-cutoff doc work actually lands in the repo

## Core Pi hot path

### `apps/pi-extension/src/turns.ts`
Key current loci from search:
- line 9 — imports `classifyCurrentAsk` from `state.ts`
- line 74 — current `scopeKind` read from `S.queryScope`
- lines 89-96 — current section-suppression heuristic for `fresh_question` and `correction`
- line 101 — still emits `[Focusa Focus State — 10-slot live refresh]`
- around line 121 — `priorMissionReused`
- same section — objective-routing trace fields including `focus_slice`, `operator_subject`, `active_subject_after_routing`, and subject-hijack-related dimensions

Interpretation:
- docs 51/54a/54b/67/68/69 land primarily here first
- this file is the main first-frontier implementation surface for replacing broad carryover with true minimal-slice routing

Relevant beads:
- `focusa-7u1f`
- `focusa-020u`
- `focusa-j18z`
- `focusa-ti0t`
- `focusa-7j4u`
- `focusa-c851`
- `focusa-mrob`

### `apps/pi-extension/src/state.ts`
Current locus:
- defines `PiCurrentAsk`, `PiQueryScope`, `PiExcludedContext`
- holds mutable shared state for `currentAsk`, `queryScope`, and `excludedContext`
- contains `classifyCurrentAsk()`

Interpretation:
- docs 67/68 are currently rooted here for type/state scaffolding
- current implementation is still bridge-level and needs promotion into a stronger routing substrate

Relevant beads:
- `focusa-020u`
- `focusa-94s7`
- `focusa-4i1x`
- `focusa-cb3e`

## API contract / ontology route surfaces

### `crates/focusa-api/src/routes/ontology.rs`
Search hits indicate current normative surfaces for:
- tool/action contract metadata
- golden-task/eval route metadata
- trace/checkpoint-related contract examples
- projection-related route examples

Interpretation:
- docs 55, 56, 57, 75, 77 partially materialize here as route-level declared metadata
- risk: route metadata can outpace truthful runtime behavior unless tied back to real producers/consumers

Relevant beads:
- `focusa-vhbq`
- `focusa-93sn`
- `focusa-qs4c`
- `focusa-2m6e`
- `focusa-v2n5`

### `crates/focusa-api/src/routes/commands.rs`
Search hits indicate active command surfaces for:
- `ascc.checkpoint`
- `memory.decay_tick`
- session lifecycle commands
- autonomy/proposal-related commands

Interpretation:
- docs 56, 76, 77, and 78 have command-path hooks here
- these are candidate loci for truthful implementation of checkpoint, decay, migration, and autonomy behaviors

Relevant beads:
- `focusa-qs4c`
- `focusa-eczn`
- `focusa-v2n5`
- `focusa-o8vn`
- `focusa-6z1y`

## Existing test/eval surfaces

### `tests/trace_dimensions_test.sh`
Current role:
- infrastructure-style trace coverage for 18 doc-56 dimensions
- proves retrievability more than truthful end-to-end production semantics

Relevant beads:
- `focusa-qs4c`
- `focusa-zd6t`
- `focusa-orh2`

### `tests/golden_tasks_eval.sh`
Current role:
- infrastructure check for doc-57 eval surfaces
- explicitly says it does not prove full SPEC-57 success by itself

Relevant beads:
- `focusa-qs4c`
- `focusa-nfdx`
- `focusa-n4fo`

## Current code-locus implications

1. First-frontier work is concentrated in:
   - `apps/pi-extension/src/turns.ts`
   - `apps/pi-extension/src/state.ts`

2. Contract/trace/eval hardening is concentrated in:
   - `crates/focusa-api/src/routes/ontology.rs`
   - `crates/focusa-api/src/routes/commands.rs`
   - `tests/trace_dimensions_test.sh`
   - `tests/golden_tasks_eval.sh`

3. Shared-substrate docs 70-77 still need more exact loci identified across core object definitions and reducers; they currently appear more as distributed vocabulary than one settled implementation surface.

4. Visual/UI docs 58-65 and affordance doc 66 still require deeper code-locus discovery because current implementation evidence is sparse.

## Immediate next refinement

Create code-targeted beads beneath the first-frontier and second-track branches so implementation order is tied directly to:
- `turns.ts` injection/routing rewrite
- `state.ts` ask/scope state model hardening
- route contract hardening in `ontology.rs`
- command-path checkpoint/decay/autonomy hardening in `commands.rs`
- test/eval surface upgrades in `tests/trace_dimensions_test.sh` and `tests/golden_tasks_eval.sh`
