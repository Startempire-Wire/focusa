# Implementation Cutoff Audit — 2026-04-13

## Purpose

Identify the exact point where real implementation fidelity drops and the newer ontology/trajectory docs become mostly normative or docs-only.

This document is the foundation for rigorous decomposition work.
It is not the final decomposition itself.

---

## Pass 1 provisional finding

**Provisional cutoff:** real implementation does **not** cleanly continue into the newer trajectory starting at **doc 51**.

More precisely:
- pre-51 / older substrate docs still have substantial working-code correspondence in core runtime areas
- docs **51-78** are largely a **normative trajectory layer** with only scattered partial implementations, partial tests, and drifted hot-path behavior
- therefore decomposition should proceed on the assumption that the newer trajectory must be treated as **post-cutoff work**, not as already-implemented functionality with a few small gaps

This is an implementation-fidelity cutoff, not a claim that docs 51-78 have zero code touchpoints.
Many later docs have references, tests, partial telemetry, or isolated scaffolding.
The key finding is that they do **not** form a mostly-working contiguous implementation layer.

---

## Key evidence

### E1. Trust audit directly marks core newer-doc behavior as DOCS-ONLY

`docs/TRUST_RESTORATION_AUDIT_2026-04-12.md` states:
- newer operator-priority / non-coercive adoption requirements are still **DOCS-ONLY** in Pi hot paths
- current Pi extension still injects a broad always-on focus-state block
- closed beads were not reliable proof of actual runtime alignment

This is direct evidence against any claim that the newer Pi/scope-control trajectory is already substantially implemented.

### E2. Pi hot path still centered on broad focus-state injection

`apps/pi-extension/src/turns.ts` historically used:
- a monolithic `[Focusa Focus State — 10-slot live refresh]` block
- prepended before every LLM call

Recent work only began to reduce this through:
- transient ask/scope metadata
- basic bounded-slice suppression for `fresh_question` and `correction`
- trace instrumentation for objective route/slice metadata

That is meaningful progress, but still far short of docs 51/54a/54b/67/68/69’s target architecture.

### E3. Later-doc concepts exist mostly as vocabulary, enums, or tests — not as a coherent implemented layer

Examples:
- doc 56 trace dimensions existed partly as enums/types before consistent hot-path emission
- doc 70 shared-interface language is referenced in docs and some object naming, but not yet as a comprehensive implemented ontology substrate
- docs 75/76/77 strongly influence newer spec language, but are not yet a fully realized runtime layer with end-to-end migration/conformance enforcement

### E4. Workers illustrate the same pattern: partial evolution, not full later-doc conformance

Worker runtime evolved toward LLM-backed execution, but older docs still required reconciliation.
This confirms the repo pattern:
- real code exists
- later doctrine exists
- alignment is incomplete and often drifted

That is a post-cutoff trajectory problem, not evidence of mostly-implemented newer docs.

---

## Provisional status matrix (high-level)

Legend:
- **Implemented** = code/runtime mostly matches doc intent
- **Partial** = meaningful implementation exists, but material drift/gaps remain
- **Docs-only** = mostly normative/spec language with little or no faithful runtime implementation
- **Drifted** = code exists, but materially contradicts or bypasses the newer doc intent

| Docs | Theme | Status | Notes |
|---|---|---:|---|
| 51-54b | Pi injection / operator priority / subject preservation | Drifted / Docs-only | Trust audit says core behavior still DOCS-ONLY in hot path; broad focus-state injection dominated historically. |
| 55 | Tool/action contracts | Partial / Test-heavy | Contract language richer than current routed API/tool schema discipline. |
| 56 | Trace/checkpoints/recovery | Partial | Trace types/checkpoints exist; many required dimensions only recently began to be emitted. |
| 57 | Golden tasks/evals | Partial / Docs-only | Some tests and eval references exist; full fixed-eval discipline not yet a strong live implementation layer. |
| 58-65 | Visual/UI ontology track | Mostly Docs-only | Limited code/test references; not a clearly implemented trajectory layer. |
| 66 | Affordance/execution environment ontology | Mostly Docs-only | Minimal code correspondence so far. |
| 67-69 | Current ask / query scope / scope-failure tracing | Early Partial | Only recent bridge work started; true minimal-slice routing still not implemented. |
| 70-77 | Shared ontology substrate / projection / retention / governance | Mostly Docs-only with scattered partials | Strong normative influence, limited faithful end-to-end implementation. |
| 78 | Bounded secondary cognition / persistent autonomy | Partial spec synthesis | Spec now hardened; implementation still incomplete and split into multiple reconciliation beads. |

---

## Pass 1 conclusion

For decomposition purposes, treat **doc 51 as the start of the post-cutoff newer trajectory**.

That means:
- all docs **51-78** must be evaluated as implementation candidates or reconciliation candidates
- none should be assumed implemented merely because code references, enums, tests, or partial route surfaces exist
- decomposition must proceed from the first true dependency frontier within 51-78, not from the existence of prose or scattered scaffolding

---

## Immediate implication for Pass 2

Pass 2 must build a doc-by-doc matrix for docs 51-78 with these columns:
- required runtime objects
- required API surfaces
- required extension surfaces
- required telemetry/trace
- required eval/test surfaces
- actual current code coverage
- classification: implemented / partial / drifted / docs-only
- prerequisite docs
- downstream docs blocked on this one

Only after that matrix exists should detailed epic/child/grandchild decomposition begin.
