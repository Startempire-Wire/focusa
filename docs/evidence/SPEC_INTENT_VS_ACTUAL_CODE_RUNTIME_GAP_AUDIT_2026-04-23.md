# Spec Intent vs Actual Code/Runtime Gap Audit — 2026-04-23

## Scope

This audit intentionally does **not** use status matrices or prior completion notes as authority.

Authority order used here:
1. original intent specs
2. actual implementation code
3. live runtime behavior

Primary sources:
- `docs/44-pi-focusa-integration-spec.md:10-17`
- `docs/45-ontology-overview.md:5-25`
- `docs/61-domain-general-cognition-core.md:3-17`
- `docs/66-affordance-and-execution-environment-ontology.md:3-23,63-284`
- `docs/67-query-scope-and-relevance-control.md:3-29`
- `docs/68-current-ask-and-scope-integration.md:28-34,57-59`
- `docs/73-intention-commitment-and-self-regulation.md:24-35,53-133`
- `docs/75-projection-and-view-semantics.md:22-39,44-82`
- `docs/76-retention-forgetting-and-decay-policy.md:17-38,44-74`
- `apps/pi-extension/src/index.ts:20-41`
- `apps/pi-extension/src/tools.ts:269-560`
- `apps/pi-extension/src/state.ts:103-105,1003-1008`
- `apps/pi-extension/src/turns.ts:137-138,368-376`
- `apps/pi-extension/src/compaction.ts:76-92`
- `apps/pi-extension/src/session.ts:272-286`
- `crates/focusa-api/src/routes/ontology.rs:20-107,208-216,228,307-308,3813-4023`
- `crates/focusa-api/src/routes/turn.rs:207-231,296`

Live runtime samples were taken from the running daemon on `2026-04-23`.

---

## Bottom line

Focusa is **real** today as a first-class Pi tool layer, scope-control layer, and durable state/runtime.

Focusa is **not yet fully real** as the larger intended system described by the specs:
- the Pi bridge is not yet truly extension-thin / single-authority in practice
- the ontology exists, but it is still only partially acting as Pi’s primary typed working world
- affordance reasoning is mostly schema presence without live materialization
- self-regulation and retention/decay are still largely vocabulary-only, not object-complete runtime layers
- projection/view semantics exist, but are still thin compared to the spec’s intended per-role read models

---

## 1. Delivered: first-class Focusa tool surface is real in code and runtime

### Spec intent
`docs/44-pi-focusa-integration-spec.md:12-17` says Focusa should be the cognitive authority and the Pi extension should provide UX glue, operator controls, and observability.

### Actual code
- `apps/pi-extension/src/index.ts:35-41` wires tools, commands, WBM, compaction, session, and turn integration.
- `apps/pi-extension/src/tools.ts:269-560` registers the first-class Focusa operator tools, including `focusa_scratch`, `focusa_decide`, `focusa_constraint`, `focusa_failure`, `focusa_intent`, `focusa_current_focus`, `focusa_next_step`, `focusa_open_question`, `focusa_recent_result`, and `focusa_note`.

### Live runtime
- Pi extension bind probe showed `33` active `focusa_*` tools.
- Live tool execution in-session succeeded for `focusa_scratch`, `focusa_current_focus`, `focusa_recent_result`, and `focusa_decide`.
- `GET /v1/telemetry/tools` returned live persisted usage including `focusa_decide: 33`, `focusa_scratch: 32`, `focusa_failure: 8`, `focusa_current_focus: 8`, `focusa_recent_result: 6`.

### Judgment
**Delivered.** The document/tool layer is not hypothetical; it exists and is being used.

---

## 2. Delivered: current-ask and query-scope control are materially implemented

### Spec intent
- `docs/67-query-scope-and-relevance-control.md:3-29` requires protection against scope contamination and adjacent-thread leakage.
- `docs/68-current-ask-and-scope-integration.md:28-34,57-59` requires every active answer path to have a `CurrentAsk`, governed by a `QueryScope`, with relevance filtering and explicit exclusion.

### Actual code
- `apps/pi-extension/src/turns.ts:368-376` classifies incoming user input, derives a query scope, and stores both before the turn runs.
- `apps/pi-extension/src/turns.ts:137-138` injects `CURRENT_ASK` and `QUERY_SCOPE` directly into the bounded prompt slice.
- `crates/focusa-api/src/routes/ontology.rs:78-80` includes `relevant_context_set`, `excluded_context_set`, and `scope_failure` object types.
- `crates/focusa-api/src/routes/ontology.rs:2547+` materializes current ask / query scope projection objects in the ontology view.

### Live runtime
`GET /v1/ontology/world` returned live object materialization for this layer:
- `current_ask: 1`
- `query_scope: 1`
- `relevant_context_set: 2`
- `excluded_context_set: 1`
- `scope_failure: 10`

### Judgment
**Delivered enough to count as real product behavior.** This is one of the clearest places where spec intent is actually present in live behavior.

---

## 3. Partial: the Pi bridge is not yet truly “extension-thin / single cognitive authority”

### Spec intent
`docs/44-pi-focusa-integration-spec.md:12-17` explicitly says:
- Focusa should be the single cognitive authority
- Pi should be UX glue
- Pi should not maintain parallel memory / compaction state

### Actual code
The extension still maintains meaningful shadow state and fallback behavior:
- `apps/pi-extension/src/state.ts:103-105` keeps `localDecisions`, `localConstraints`, and `localFailures`.
- `apps/pi-extension/src/state.ts:1003-1008` builds compaction instructions from that local shadow.
- `apps/pi-extension/src/compaction.ts:76-92` syncs local shadow before compaction and persists backup state into Pi session entries.
- `apps/pi-extension/src/session.ts:272-286` does soft resync after reconnect and can push locally accumulated shadow state back into Focusa with note `Reconciled after Focusa outage`.

### Runtime implication
This is useful operationally, but it means Pi is still carrying meaningful shadow cognition/continuity behavior instead of being only a thin shell over Focusa.

### Judgment
**Partial.** The bridge works, but the “single authority / thin extension” intent is not yet fully achieved.

---

## 4. Partial: the ontology exists, but it is not yet clearly Pi’s main typed working world

### Spec intent
`docs/45-ontology-overview.md:5-25` says the ontology should be canonical in semantics and give Pi a typed, bounded, interruptible working world rather than just preserved cognitive fragments.

### Actual code
- `crates/focusa-api/src/routes/ontology.rs:3813-4023` builds a combined projection and slice payloads over a large typed world.
- `crates/focusa-api/src/routes/turn.rs:207-231` only injects `active_mission_slice_summary(...)` into prompt assembly as `directive`.
- `crates/focusa-api/src/routes/turn.rs:296` exposes only a lightweight `ontology_slice` status marker in turn status.
- `apps/pi-extension/src/turns.ts` still assembles its hot-path slice primarily from Focus State, semantic memory summaries, ECS handle summaries, and scope headers, not from a rich ontology projection.

### Live runtime
`GET /v1/ontology/world` returned a large live graph:
- `object_count: 7114`
- `link_count: 11833`

So the world exists. But the prompt/runtime path still uses it mostly as a summarized sidecar, not as the dominant working-world substrate the spec describes.

### Judgment
**Partial.** The ontology is real, but it is not yet fully occupying the role the specs intended.

---

## 5. Missing in practice: affordance reasoning is mostly schema presence, not live product behavior

### Spec intent
`docs/66-affordance-and-execution-environment-ontology.md:3-23` says this layer should answer:
- what can be done right now
- what is blocked by authority, tools, resources, or prerequisites
- which action path is safest / cheapest / fastest / most reversible

It defines `Capability`, `ToolSurface`, `Permission`, `AuthorityBoundary`, `Precondition`, `Resource`, `CostModel`, `LatencyProfile`, `ReliabilityProfile`, `ReversibilityProfile`, and `Affordance` at `docs/66-affordance-and-execution-environment-ontology.md:63-284`.

### Actual code
`crates/focusa-api/src/routes/ontology.rs:33-45` includes these object types in `OBJECT_TYPES`, so the vocabulary is present.

### Live runtime
The same live `/v1/ontology/world` sample showed **no materialized objects** of these types:
- no `affordance`
- no `capability`
- no `tool_surface`
- no `permission`
- no `authority_boundary`
- no `precondition`
- no `resource`
- no `cost_model`
- no `latency_profile`
- no `reliability_profile`
- no `reversibility_profile`

### Judgment
**Missing as an experienced system.** The schema exists, but the intended affordance layer is not yet visibly functioning in live runtime.

---

## 6. Missing in practice: self-regulation / commitment is still mostly vocabulary-only

### Spec intent
`docs/73-intention-commitment-and-self-regulation.md:24-35` says Focusa needs operational structures for:
- intention formation
- commitment maintenance
- inhibition
- persistence under interruption
- controlled abandonment
- finishing loops instead of drifting

It defines `Intention`, `Commitment`, `InhibitionRule`, `DistractionCandidate`, `PersistencePolicy`, `AbandonmentCondition`, `CompletionDrive`, and `SelfRegulationState` at `docs/73-intention-commitment-and-self-regulation.md:53-133`.

### Actual code
- `crates/focusa-api/src/routes/ontology.rs:20-107` does **not** include those object types in `OBJECT_TYPES`.
- But `crates/focusa-api/src/routes/ontology.rs:208-211` includes relation vocabulary like `commits_to`, `inhibits`, and `abandons_under`.
- `crates/focusa-api/src/routes/ontology.rs:228,307-308` includes action vocabulary like `maintain_commitment`, `form_intention`, and `promote_commitment`.

### Runtime implication
The naming/vocabulary exists, but the main self-regulation objects from the spec are not actually part of the live ontology object model.

### Judgment
**Mostly missing.** This is one of the clearest spec→code gaps.

---

## 7. Missing in practice: retention / forgetting / decay is also still mostly vocabulary-only

### Spec intent
`docs/76-retention-forgetting-and-decay-policy.md:17-38` requires explicit memory discipline between canonical, active, decayed, superseded, archived, and pruned knowledge.

It defines `RetentionPolicy`, `DecayProfile`, `ArchiveState`, and `PruningDecision` at `docs/76-retention-forgetting-and-decay-policy.md:44-74`.

### Actual code
- `crates/focusa-api/src/routes/ontology.rs:20-107` does **not** include these object types in `OBJECT_TYPES`.
- But `crates/focusa-api/src/routes/ontology.rs:214-216` includes relation vocabulary `retained_under`, `decays_via`, and `archived_as`.

### Live runtime
`GET /v1/ontology/world` did not surface these retention object families either.

### Judgment
**Mostly missing.** The spec names an important behavior layer that is not yet object-complete in code/runtime.

---

## 8. Partial: projection/view semantics exist, but only as a thin layer

### Spec intent
`docs/75-projection-and-view-semantics.md:22-39` says truth should be canonical while views should be contextual, role-aware, bounded, and traceable.

It defines `Projection`, `ViewProfile`, `ProjectionRule`, and `ProjectionBoundary` at `docs/75-projection-and-view-semantics.md:44-82`.

### Actual code
- `crates/focusa-api/src/routes/ontology.rs:86-89` includes `projection`, `view_profile`, `projection_rule`, and `projection_boundary` object types.
- `apps/pi-extension/src/turns.ts:137-138` injects `PROJECTION_KIND`, `VIEW_PROFILE`, `CURRENT_ASK`, and `QUERY_SCOPE` into the prompt slice.
- `crates/focusa-api/src/routes/ontology.rs:402+` derives view/profile kinds by slice type.

### Live runtime
`GET /v1/ontology/world` showed only minimal materialization:
- `projection: 1`
- `view_profile: 1`
- `projection_rule: 1`
- `projection_boundary: 1`

### Judgment
**Partial.** Projection semantics are present, but currently look more like a thin metadata/projection shell than the richer per-role read-model system the spec describes.

---

## 9. Delivered but bounded: identity/reference resolution is more real than the higher cognition layers

### Spec intent
The ontology direction requires typed canonical entities and reference resolution instead of loose artifact naming.

### Actual code
`crates/focusa-api/src/routes/ontology.rs:81-84` includes:
- `canonical_entity`
- `reference_alias`
- `resolution_candidate`
- `resolution_decision`

### Live runtime
The live world had real counts for this layer:
- `canonical_entity: 124`
- `reference_alias: 128`
- `resolution_candidate: 128`
- `resolution_decision: 128`

### Judgment
**Delivered enough to be real.** This layer is materially farther along than affordance, self-regulation, or retention.

---

## What this means for the product question

If the question is:

> “Why does Focusa still not fully feel like the intended software?”

The code/runtime evidence points to four main reasons:

1. **The bridge is still not fully extension-thin**
   - Pi still carries local shadow continuity and compaction behavior.

2. **The ontology is present but still secondary in the turn path**
   - it exists as a world/slice layer, but the live Pi hot path is still more Focus State + summaries than full typed working-world operation.

3. **Affordance reasoning is not yet materially alive**
   - practical possibility / authority / reversibility / cost are not showing up as live ontology objects.

4. **Conative memory discipline layers are still mostly absent as objects**
   - self-regulation and retention/decay are still mainly spec vocabulary, not fully implemented runtime strata.

---

## Completion priority implied by code, not by stale status docs

1. **Finish the single-authority bridge contract**
   - reduce remaining Pi shadow cognition/compaction behavior
2. **Make ontology slices more central to actual turn construction**
3. **Materialize affordance objects in live world + action selection**
4. **Implement doc 73 object layer, not just its links/actions**
5. **Implement doc 76 object layer, not just its links/actions**
6. **Deepen projection/view behavior from thin headers into real per-role read models**

That is the code-grounded explanation for why the intended experience still feels incomplete.
