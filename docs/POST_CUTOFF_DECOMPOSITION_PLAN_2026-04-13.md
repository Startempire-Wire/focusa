# Post-Cutoff Decomposition Plan — 2026-04-13

This plan assumes the cutoff finding from `docs/IMPLEMENTATION_CUTOFF_AUDIT_2026-04-13.md`:
- docs 51-78 are the post-cutoff newer trajectory
- they must be decomposed as implementation/reconciliation work, not assumed-working functionality
- pass count is open-ended until decomposition is truly complete

---

## Ordered implementation/decomposition sequence

### Sequence 0 — cutoff and matrix artifacts
Already created:
- `docs/IMPLEMENTATION_CUTOFF_AUDIT_2026-04-13.md`
- `docs/IMPLEMENTATION_STATUS_MATRIX_2026-04-13.md`

### Sequence 1 — first true implementation frontier
Parent epic:
- `focusa-7u1f` — Implement first frontier: operator-first minimal-slice Pi context pipeline

Child beads:
- `focusa-020u` — persist explicit CurrentAsk and QueryScope in Pi hot path
- `focusa-j18z` — implement first real RelevantContextSet selection for Pi injection
- `focusa-ti0t` — implement explicit ExcludedContextSet tracking and reasons
- `focusa-7j4u` — add named scope-routing trace events to Pi/API telemetry
- `focusa-c851` — enforce operator-priority reset rules in Pi context injection
- `focusa-qovg` — align Pi injection outputs with shared interface substrate
- `focusa-mrob` — add replay/eval surfaces for scope-routing regression

Grandchildren created so far:
- `focusa-94s7` — define CurrentAsk classification rules for Pi input
- `focusa-4i1x` — persist QueryScope and reset semantics across input/context boundary
- `focusa-1l3x` — select relevant decisions for first minimal-slice injection
- `focusa-90rd` — select relevant constraints for first minimal-slice injection
- `focusa-cb3e` — track excluded context labels with explicit reasons
- `focusa-zd6t` — emit current_ask_determined and query_scope_built trace events
- `focusa-orh2` — emit relevant_context_selected and irrelevant_context_excluded trace events
- `focusa-w91s` — add regression tests for fresh-question and correction reset behavior

Why first:
- this is the earliest point where real behavior can shift from broad carryover to truthful operator-first routing
- it unblocks meaningful implementation of docs 52-57 and gives later substrate work a real consumer

---

### Sequence 2 — Pi contract / behavior / tools / trace / eval track
Parent epic:
- `focusa-e3id` — Implement Pi contract + action/eval trajectory docs 52-57 after routing substrate

Children:
- `focusa-210z` — enforce bounded Pi input/output contract surfaces
- `focusa-imqt` — implement measurable behavioral alignment hooks for decisions/constraints
- `focusa-vhbq` — formalize tool/action contracts across Pi and Focusa routes
- `focusa-qs4c` — complete trace/checkpoint/eval discipline for Pi integration

Grandchildren created so far:
- `focusa-murm` — bound Pi input slice fields to current-ask relevance only
- `focusa-r9mo` — bound Pi output surface to typed proposal/observation/failure classes
- `focusa-yt3x` — consult relevant constraints before risky shell/edit actions
- `focusa-9ux2` — consult relevant decisions in repeated-pattern zones
- `focusa-sode` — define route/tool contract schema coverage table
- `focusa-nfdx` — define golden-task replay set for routing + behavioral alignment

Why second:
- these docs only become truthful once current-ask relevance and minimal-slice routing exist
- otherwise contract/eval work would harden the wrong behavior

---

### Sequence 3 — shared ontology substrate track
Parent epic:
- `focusa-jz89` — Implement shared ontology substrate docs 70-77 after first frontier

Children:
- `focusa-3zav` — implement verifiable/actionable/scoped object field discipline
- `focusa-2m6e` — implement projection/view-profile surfaces
- `focusa-eczn` — implement retention/decay semantics beyond local heuristics
- `focusa-v2n5` — implement governance/versioning/migration checks for ontology changes
- `focusa-ru3s` — decompose governing priors + identity/self-model + intention docs into implementable runtime surfaces

Grandchildren created so far:
- `focusa-40pg` — normalize Verifiable and Scoped fields for new routing objects
- `focusa-ue1o` — define first Projection and ViewProfile objects for Pi
- `focusa-ystw` — define active-vs-historical retention policy for decisions and constraints
- `focusa-1euv` — define shared-layer compatibility checks for ontology changes

Why third:
- this track becomes much easier to implement honestly once a real routing consumer exists
- it also supports later bounded views and migration discipline for the newer ontology stack

---

### Sequence 4 — affordance/environment ontology track
Parent epic:
- `focusa-wmw7` — Implement affordance/execution environment ontology doc 66

Child:
- `focusa-2w17` — define executable environment facts and affordance surfaces

Grandchild created so far:
- `focusa-u7ck` — define first affordance object and execution-environment facts

Why here:
- affordances feed tool/action selection and UI-to-implementation handoff
- but should be grounded after routing and shared-interface work start to become real

---

### Sequence 5 — visual/UI trajectory
Parent epic:
- `focusa-s2z6` — Implement visual/UI cognition trajectory docs 58-65

Children:
- `focusa-6pd1` — define implemented core object frontier for docs 58-60
- `focusa-ost9` — define evidence/invention/handoff frontier for docs 62-65

Grandchildren created so far:
- `focusa-5bqc` — define smallest truthful visual core object set
- `focusa-piwx` — define evidence capture and critique loop for visual objects

Why later:
- current repo evidence suggests this track is sparse and likely blocked on stronger shared substrate and affordance semantics
- it still must be decomposed fully, but should not hijack the core routing frontier

---

## Open-ended pass rule

This decomposition is **not complete just because a numbered pass list exists**.
Continue adding passes, refinements, child beads, and grandchild beads until:
- every remaining doc in 51-78 has a clear place in the hierarchy
- blocked vs unblocked work is explicit
- tests/evals/telemetry/migration obligations are attached where needed
- no broad ambiguous beads remain in the active execution frontier

---

## Immediate next decomposition refinements

1. Reconcile existing pre-existing beads against the new hierarchy to avoid duplicate work.
2. Add telemetry/eval/migration annotations to each parent/child branch.
3. Expand the shared-substrate branch for docs 71-74 where vocabulary is still too broad.
4. Expand the visual/UI branch until docs 58-65 each have at least one truthful implementation frontier bead.
5. Expand affordance/environment branch with first consumer mapping.
