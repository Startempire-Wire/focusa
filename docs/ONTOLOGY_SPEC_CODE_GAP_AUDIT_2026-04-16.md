# Ontology Spec ↔ Code Gap Audit (Direct Read, No Summary Proxy)

Date: 2026-04-16
Scope read directly: docs 45,46,47,48,49,50,51,67,68,69,70,74,75,77 + code loci in `crates/focusa-api/src/routes/ontology.rs`, `crates/focusa-core/src/types.rs`, `crates/focusa-core/src/reducer.rs`, `crates/focusa-api/src/routes/proposals.rs`, `crates/focusa-api/src/routes/telemetry.rs`.

## Confirmed Gaps

1. **Ontology is projection-only, not canonical reducer-owned world state**
   - Spec: ontology truth canonized via reducer; canonical ontology state should evolve (`docs/45-ontology-overview.md`, `docs/50-ontology-classification-and-reducer.md:38-47`).
   - Code: ontology route explicitly read-only projection (`crates/focusa-api/src/routes/ontology.rs:1-5`, `:1591-1597`, `:1652-1657`).
   - Code: `FocusaState` has no ontology state container (`crates/focusa-core/src/types.rs:390-426`).

2. **Ontology reducer events exist but reducer does not apply ontology mutations**
   - Spec: reducer responsibilities include apply canonical deltas, update working-set membership canonically (`docs/50-ontology-classification-and-reducer.md:42-47`).
   - Code: ontology events are treated as audit-only no-op in reducer (`crates/focusa-core/src/reducer.rs:1089-1096`).

3. **Action policy contract violation (toolable + reducer-visible + constraint checked)**
   - Spec: actions must map to concrete toolable behavior and emit reducer-visible events (`docs/48-ontology-links-actions.md:60-64`).
   - Code: action catalog marks `reducer_visible=false`, `runtime_execution_supported=false`, `constraint_checked=false` (`crates/focusa-api/src/routes/ontology.rs:1614-1619`).

4. **Deterministic classifier coverage incomplete**
   - Spec requires import/call graph extraction (`docs/50-ontology-classification-and-reducer.md:16-17`).
   - Code exposes link types `imports`/`calls` but workspace projection never emits them (no `"type": "imports"` or `"type": "calls"` emissions in `ontology.rs`; emitted links are filesystem/package-derived only).

5. **Doc-70 shared status/lifecycle substrate not implemented in ontology primitives surface**
   - Spec statuses include `proposed,candidate,failed,superseded,retired,completed` (`docs/70-shared-interfaces-statuses-and-lifecycle.md:198-210`).
   - Code status vocabulary omits all six above (`crates/focusa-api/src/routes/ontology.rs:26-28`).

6. **Doc-67 query-scope ontology objects/actions missing**
   - Spec object types: `CurrentAsk, QueryScope, RelevantContextSet, ExcludedContextSet, ScopeFailure` (`docs/67-query-scope-and-relevance-control.md:44-107`).
   - Spec actions: `determine_current_ask, build_query_scope, select_relevant_context, exclude_irrelevant_context, verify_answer_scope, record_scope_failure` (`docs/67-query-scope-and-relevance-control.md:159-177`).
   - Code: none of these are in `OBJECT_TYPES`/`ACTION_TYPES` (`crates/focusa-api/src/routes/ontology.rs:18-49`).
   - Note: partial non-ontology representation exists only in work-loop decision context fields (`crates/focusa-core/src/types.rs:335-343`).

7. **Doc-74 identity-resolution model missing**
   - Spec objects/actions defined in doc-74 (`docs/74-identity-and-reference-resolution.md:44-163`).
   - Code: no `CanonicalEntity/ReferenceAlias/Resolution*` objects or actions in ontology constants (`crates/focusa-api/src/routes/ontology.rs:18-49`).

8. **Doc-75 projection/view ontology model missing**
   - Spec objects/actions defined in doc-75 (`docs/75-projection-and-view-semantics.md:42-145`).
   - Code: no `Projection/ViewProfile/ProjectionRule/ProjectionBoundary` objects or `build_projection/compress_projection/...` actions in ontology constants (`crates/focusa-api/src/routes/ontology.rs:18-49`).

9. **Doc-77 governance/versioning/migration ontology model missing**
   - Spec objects/actions defined in doc-77 (`docs/77-ontology-governance-versioning-and-migration.md:42-171`).
   - Code: no `OntologyVersion/CompatibilityProfile/MigrationPlan/DeprecationRecord/GovernanceDecision` objects nor governance actions in ontology constants (`crates/focusa-api/src/routes/ontology.rs:18-49`).

10. **Doc-51 operator-first slice assembly rule not encoded in ontology slice builder**
    - Spec: operator-intent classification must precede slice assembly (`docs/51-ontology-expression-and-proxy.md:36-52`).
    - Code: slice membership computed from projection object-type buckets only, no operator ask classification input in slice builder (`crates/focusa-api/src/routes/ontology.rs:1333-1425`).

## Result

Claim "all ontology specs transformed to working functionality" is currently false.

The implemented surface today is a useful **read-only projection and contract catalog**, but it does not yet satisfy the canonical reducer-owned ontology evolution model and multiple post-cutoff ontology domains (67/70/74/75/77).