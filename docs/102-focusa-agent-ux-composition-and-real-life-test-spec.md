# Focusa Agent UX Composition and Real-Life Test Spec

Status: iterable-spec-v0
Scope: Focusa agent experience, composition UX, contradiction handling, and exhaustive real-life agent-accessible surface testing
Authority: planning and evaluation only; no implementation changes authorized by this spec

## 0. Origin

This spec captures an experience report from actual Focusa use. The report was based on a representative tool battery, not only prior discussion or architecture theory.

Battery used:
- `focusa_project_identity`
- `focusa_project_verify`
- `focusa_project_card`
- `focusa_trajectory_view`
- `focusa_workpoint_resume`
- `focusa_tool_doctor`
- `focusa_resource_mode`
- `focusa_state_hygiene_doctor`
- `focusa_traverse` on ontology/workpoints/evidence/tool_registry
- `focusa_predict_stats/recent`
- `focusa_metacog_doctor`
- `focusa_work_loop_status`
- repo evidence via `git status`, spec/doc/script/test counts

## 1. Product read from use only

Focusa feels like an **agent cognition operating system**.

Not just memory. Not just task tracking. More like:
- project identity resolver
- goal/trajectory manager
- active work checkpoint system
- evidence/proof ledger
- agent prompt/context governor
- prediction/metacognition layer
- tool/router/intelligence surface
- safety boundary around “what is true vs what is advisory”

If inferred without prior explanation:

> Focusa helps coding agents avoid losing the plot across long sessions by binding work to project scope, Workpoints, Trajectory, evidence, and compact context packets.

## 2. What worked well

### 2.1 Project verification recovered unsafe `/root`

Initial injected card said root unsafe `/root`.

`focusa_project_identity` + `focusa_project_verify` correctly resolved:

- `/home/wirebot/focusa`
- project `Focusa`
- Rust monorepo
- local daemon URL `127.0.0.1:8787`

Good UX: it can self-correct broad cwd.

### 2.2 Workpoint model is strong

`focusa_workpoint_resume` returned canonical packet with:
- mission
- action
- exact next step
- do-not-drift boundaries

This is one of the best parts. It creates “continuation gravity.”

### 2.3 Project card is useful

`focusa_project_card` gave compact orientation:
- project
- current high-level goal
- inferred next event
- predictions count
- ontology object count
- outcomes

This is close to an “agent dashboard card.”

### 2.4 Tool doctor is valuable

`focusa_tool_doctor` surfaced:
- readiness ready
- contracts 63 / live 63
- drift yes
- token budget critical
- workpoint not_found
- UIAI standby/high

Good: it gave a health+next-action summary.

### 2.5 Traverse is bounded

`focusa_traverse` avoided dumping giant payloads.

Evidence surface had `19848` entries but returned bounded 10 with cursor. Correct philosophy.

## 3. Flaws / inconsistencies noticed

### 3.1 Identity mismatch vs verify contradiction

`focusa_project_identity` said:
- status `mismatch`
- confidence low

Then `focusa_project_verify` said:
- verified true
- confidence high

Same root. This is confusing. Agent UX should explain mismatch cause:

```text
identity=mismatch because persisted_project_root differs from requested root
verification=passed for requested root
```

### 3.2 Trajectory says no canonical packet, but Workpoint resume works

`trajectory_view` said:
- no canonical packet for current continuity
- create checkpoint then retry

But `workpoint_resume` succeeded with canonical Workpoint.

This creates “authority split brain.” Need clearer reconciliation:

```text
Trajectory lacks linked canonical packet, but Workpoint X is canonical. Suggested action: link/refresh trajectory association.
```

### 3.3 Tool doctor says drift=yes but live contracts 63/63

This is alarming but underexplained.

If contracts match 63/63, what drift?

Need top 3 drift causes in text:

```text
drift=yes: stale_live_contracts=[...], missing_docs=[...]
```

### 3.4 Ontology traverse returned 0, project card says ontology_objects=11

Possible selector issue, but UX feels inconsistent:
- `project_card`: ontology_objects=11
- `traverse ontology summaries`: returned 0

Need either:
- better default ontology selector
- “try selector=recent/search/head”
- explanation: “summaries unavailable; objects exist in registry X”

### 3.5 Workpoint mission can go stale

Current user ask was “experience report/gaps,” but active Workpoint mission still Bloatgaurd spec edit.

Focusa correctly says operator steering wins, but UX should offer:

```text
Current ask differs from active Workpoint. Create temporary report Workpoint? yes/no
```

No modal, just suggested route.

### 3.6 Evidence count huge

`evidence` has 19,848 entries. Great power, but likely retrieval UX problem.

Needs:
- top evidence by active object
- recent evidence by Workpoint
- confidence-changing evidence
- stale evidence
- duplicate evidence clusters

### 3.7 Token budget critical but resource normal

`tool_doctor`: token_budget=critical

`resource_mode`: normal/within_budget

This distinction makes sense technically, but UX should separate:
- context/token pressure
- daemon/system memory pressure

Right now it reads contradictory.

## 4. Product composition read

Focusa has many excellent primitives but needs stronger **composition UX**.

Current feel:
- powerful expert cockpit
- lots of surfaces
- great safety language
- many “next tools”
- but agents can get tool-route vertigo

Ideal composition:

### 4.1 One “Now” card

- project
- current ask
- active Workpoint
- trajectory gap
- next action
- proof needed
- risk
- current profile

### 4.2 One “Why” card

- why this context was included
- why other context excluded
- source authority ranking

### 4.3 One “Health” card

- scope
- workpoint
- trajectory
- evidence
- token pressure
- drift
- UIAI

### 4.4 One “Do” card

- exact next command/tool
- what it will mutate
- rollback/rehydrate refs

## 5. Agent UX “send it to the moon” recommendations

### 5.1 Profile selector everywhere

Use the Bloatgaurd profile names:
- Daily Driver
- Beast Mode
- Speedy
- Neat Freak
- Tightwad

Every packet should say:

```text
BLOATGAURD_PROFILE: Daily Driver
CONTEXT_POSTURE: balanced
FULL_PAYLOAD: cold opt-in
```

### 5.2 Routine buttons / commands

Named routines become agent affordances:
- The Scout: choose route
- The Librarian: compile context
- The Squeezer: compact tool history
- The Deep Dive: rehydrate exact proof
- The Gatekeeper: strict check

### 5.3 Contradiction resolver

When surfaces disagree, emit:

```text
CONFLICT:
- project_identity=mismatch
- project_verify=verified
RESOLUTION:
- requested root verified; persisted identity likely stale
NEXT:
- refresh identity binding
```

### 5.4 Confidence-changing evidence

Instead of recent evidence:

```text
Evidence that changed belief:
- test X pass
- spec Y added
- browser diag Z failed
```

### 5.5 Ask-to-Workpoint bridge

If current ask differs from active Workpoint:

```text
Operator ask changed from spec-edit → UX report.
Suggested: checkpoint temporary report Workpoint.
```

### 5.6 Context receipt

Every injected Focus Slice should include:

```text
included: 7 items
excluded: 214 items
omitted_bytes: 480k
rehydrate_refs: 5
reason: current ask + Workpoint + trajectory gap
```

### 5.7 Explain this packet

A built-in X-Ray:

```text
Why did I see this?
Why was this excluded?
Why is this canonical/advisory?
What would Beast Mode include?
```

## 6. Biggest gap

Focusa has the right architecture, but the UX needs **one compositional layer above the tools**.

Not more tools. Better orchestration:

```text
Scout → Now Card → Context Compiler → Workpoint/Evidence → X-Ray
```

## 7. Recommended next spec/theme

Add or implement:

> Focusa Operator Now Card + Conflict Resolver

This should become the main agent-facing composition layer over existing primitives.

## 8. Next-phase full real-life test plan

After this initial Spec102 seed, run a full extensive real-life test suite across all agent-accessible surfaces. Do not exclude surfaces simply because they are inconvenient. Record notes and iterate this spec.

### 8.1 Project/bootstrap surfaces

Test:
- project identity from safe cwd
- project identity from broad `/root`
- project verify with expected id/name/repo
- project card bootstrap
- project card outcome flow
- session transfer save/status/continue where safe

Notes to capture:
- mismatch explanations
- bootstrap clarity
- stale persisted identity behavior
- cross-project guard quality

### 8.2 Workpoint surfaces

Test:
- checkpoint
- resume canonical packet
- resume missing packet
- link evidence
- active object resolve
- degraded/noncanonical behavior
- ask-to-Workpoint mismatch flow

Notes to capture:
- continuation clarity
- do-not-drift usability
- exact next-action quality
- conflict with trajectory wording

### 8.3 Trajectory surfaces

Test:
- view/resume
- define goal
- assess
- propose Workpoint
- checkpoint
- prior-project fallback
- stale/missing evidence behavior

Notes to capture:
- HLT/MLG/STG clarity
- fallback labeling
- gap/action usefulness
- conflict with canonical Workpoint

### 8.4 Ontology/traverse surfaces

Test:
- ontology summaries
- ontology search
- workpoint recent
- evidence recent/search
- tool registry summaries
- telemetry slices
- tags verification
- include_rehydrate_refs

Notes to capture:
- selector discoverability
- empty result explanations
- cursor/rehydrate usefulness
- mismatch with project-card counts

### 8.5 Evidence/proof surfaces

Test:
- evidence capture
- workpoint evidence link
- browser diagnostics intake
- proof bundle refs
- confidence-changing evidence view if available

Notes to capture:
- evidence target clarity
- duplicate/stale evidence risks
- proof boundary clarity
- large evidence-store navigation

### 8.6 Prediction/metacog surfaces

Test:
- predict record/recent/stats/evaluate
- metacog retrieve/capture/doctor/reflect where safe
- end-of-task learning loop

Notes to capture:
- whether predictions improve next action
- stale prediction cleanup
- metacog retrieval relevance
- operator-visible value

### 8.7 Work-loop surfaces

Test:
- writer status
- status
- context update if safe
- checkpoint if safe
- preflight control only unless explicitly authorized
- select-next only if appropriate

Notes to capture:
- writer ownership clarity
- mutation risk clarity
- idle/enabled/budget rendering

### 8.8 State hygiene surfaces

Test:
- hygiene doctor
- hygiene plan
- apply only with explicit approval if ever needed

Notes to capture:
- stale/duplicate signal clarity
- safe/no-delete guarantees

### 8.9 Resource/doctor surfaces

Test:
- tool doctor scoped/all
- resource mode status
- LowMem preflight/status
- token-budget warnings
- UIAI browser pressure reporting

Notes to capture:
- health contradiction clarity
- top causes for drift
- token pressure vs resource pressure distinction

### 8.10 UIAI/web/browser surfaces

Test:
- UIAI agent card
- health
- tool search/graph
- browser open/read/snapshot/diagnostics/close
- source/markdown/search route where available
- Focusa diagnostics packet handoff
- saturated browser pressure flow

Notes to capture:
- UIAI-first affordance clarity
- session cleanup
- diagnostics usefulness
- evidence handoff quality

### 8.11 Pi extension / Focus Slice surfaces

Test:
- context injection from safe project
- context injection from unsafe `/root`
- current ask scope verdict
- attention recall verdict
- tool affordances
- UIAI-first focus slice
- resource/LowMem slice
- Workpoint resume injection after compaction

Notes to capture:
- prompt bloat
- visible contradiction handling
- whether agent knows exact next action
- whether included/excluded context receipt exists

### 8.12 CLI/API/docs surfaces

Test:
- CLI status/current commands where available
- API health/status/packet routes where available
- docs references for each surfaced route
- parity between docs, CLI, API, Pi tools

Notes to capture:
- missing docs
- stale docs
- command discoverability
- parity gaps

### 8.13 Bloatgaurd/Context Cognition surfaces

Test current state, even if spec-only:
- Spec100 Context Cognition implementation presence
- Spec101 Bloatgaurd implementation presence
- profiles availability
- routines availability
- tokenbloat metrics availability
- context compiler path availability

Notes to capture:
- spec-to-implementation gaps
- useful first implementation slice
- UX naming clarity

## 9. Test output format for iteration notes

For each tested surface, record:

```yaml
surface:
tool_or_route:
status: pass | degraded | blocked | missing | confusing
observed_output:
ux_issue:
composition_issue:
authority_issue:
tokenbloat_issue:
evidence_ref:
recommendation:
spec102_update_needed:
```

## 10. Acceptance criteria for Spec102 iteration 1

- all agent-accessible surfaces tested or explicitly marked unavailable with reason
- contradictions cataloged
- top 10 UX improvements ranked
- Now/Why/Health/Do card requirements refined
- conflict resolver requirements refined
- ask-to-Workpoint bridge requirements refined
- evidence navigation requirements refined
- Focus Slice context receipt requirements refined
- Bloatgaurd/Context Cognition integration requirements refined

## 11. Iteration 1 breadth-pass evidence

Breadth-pass notes from the first real-life agent-accessible surface battery are captured in:

- `docs/evidence/SPEC102_REAL_LIFE_SURFACE_BATTERY_2026-06-06.md`

High-level findings from that pass:

1. Identity mismatch and project verification can disagree without a reconciliation explanation.
2. Trajectory fallback/no-canonical-packet messaging can conflict with canonical Workpoint resume.
3. Missing Workpoint-id resume can silently fall back to the active Workpoint.
4. Project card ontology counts can disagree with ontology traverse results.
5. Tool doctor drift/token/workpoint statuses need compact cause lines.
6. Token budget and daemon resource pressure need separate Health Card fields.
7. UIAI tool search needs fuzzy/multi-intent behavior; UIAI pressure needs current-vs-historical split.
8. Prediction/metacog compact renders need actionable ids and top one-line content.
9. Spec100 Context Cognition and Spec101 Bloatgaurd are not yet clearly runtime-available surfaces.
10. Focusa needs a Now Card, Conflict Resolver, Context Receipt, and Ask-to-Workpoint Bridge as the primary composition layer.

## 12. Iteration 1 deeper-pass failure notes

Deeper failure notes are appended to:

- `docs/evidence/SPEC102_REAL_LIFE_SURFACE_BATTERY_2026-06-06.md`

Deeper-pass ranked failures:

1. Missing Workpoint id silently falls back to active Workpoint.
2. Trajectory/Workpoint reconciliation gap.
3. Doctor ready-vs-blocked category ambiguity.
4. Ontology/project-card count mismatch.
5. Evidence search failing for newly created Spec102 evidence.
6. Focus State write blocked despite canonical Workpoint.
7. Prediction/metacog Pi compact renders hide actionable details.
8. UIAI multi-word tool search failure.
9. UIAI current-vs-historical pressure ambiguity.
10. Work-loop `[object Object]` render.

New explicit requirements to define in the next Spec102 iteration:

- `RequestedIdFallbackDisclosure`
- `TrajectoryWorkpointReconciliation`
- `DoctorReadinessCategories`
- `OntologyCountSourceParity`
- `EvidenceSearchIndexHealth`
- `FocusStateWorkpointBridge`
- `PredictionCompactActionability`
- `MetacogCompactLessonLine`
- `UIAITokenizedToolSearch`
- `UIAIPressureSplit`
- `WorkLoopBudgetRenderSchema`
- `DiagnosticsSeverityClassifier`
- `SpecAvailabilityRegistry`

## 13. Spec98/99 singleton-remnant mapping

Spec98/99 establish that Focusa's canonical authority must be addressed by verified project/workstream scope, not daemon-global singleton `current` / `active` / `last` pointers. The required authority keys are:

```text
ProjectRootKey = verified_project_root + project_fingerprint
WorkstreamKey = ProjectRootKey + continuity_id
AttachmentKey = WorkstreamKey + instance_id + session_id + attachment_id
```

Spec98 explicitly forbids canonical authority from daemon-global active project/session/frame/Workpoint/Trajectory/current-task/last identity fields, fallback from scoped query to global active state, ambiguous cwd fallback, trajectory similarity as merge authority, and session id as project identity.

Spec99 found the implementation still partially centers canonical runtime shape around singleton `current` / `active` / `last` fields, with later scope guards patched around them. Spec102's real-life UX failures are therefore not random polish issues: many are visible remnants of the old singleton model leaking through newer scoped authority surfaces.

### 13.1 Direct singleton-remnant failures

| Spec102 issue | Singleton-remnant diagnosis | Spec98/99 foundation relation | UX implication |
| --- | --- | --- | --- |
| `RequestedIdFallbackDisclosure` | Requesting a missing Workpoint id returned the active Workpoint. This looks like fallback from scoped query to global/current active state. | Spec98 forbids daemon-global active Workpoint and fallback from scoped query to global active state. | Agent may falsely believe requested id resolved; lineage confidence becomes unsafe. |
| `TrajectoryWorkpointReconciliation` | Trajectory view reports fallback/no canonical packet while Workpoint resume returns canonical state. Active Trajectory and active Workpoint appear as separate patched singleton-era planes. | Spec98 requires active Workpoint and active Trajectory scoped under WorkstreamKey; Spec98 §13.4/13.6 require explicit handoff contracts across Focus State/Workpoint/Trajectory. | Agent sees authority split brain and may checkpoint/recover unnecessarily. |
| `FocusStateWorkpointBridge` | Focus State write rejected due frame binding while Workpoint was canonical. Focus State active frame and Workpoint continuation are not reconciled under one scoped workstream view. | Spec98 flags Focus State fallback to daemon active frame and requires active frame/slots scoped inside workstream/thread. Spec99 calls Focus Stack active pointer singleton critical. | Agent cannot tell whether Focus State or Workpoint is the active continuation authority. |
| `OntologyCountSourceParity` | Project card reports ontology object count while ontology traverse returns 0. This suggests separate read indexes/global arrays with route-specific filters rather than shared scoped source. | Spec98 §13.3/13.12 warn global arrays/read indexes and ontology advisory promotion can masquerade as authority without clear scope/source. | Agent cannot trust ontology availability or know which ontology layer is real. |
| `EvidenceSearchIndexHealth` | Recent evidence sees 19,848 entries but search cannot find just-linked Spec102 evidence. Evidence appears globally accumulated with insufficient scoped/indexed lookup. | Spec98 §13.9 requires Reference Store/ECS handle identity include project/session/root scope and explicit auditable rehydration. | Agent must rely on transcript memory/exact handles instead of scoped evidence search. |
| `DoctorReadinessCategories` | Pi doctor says ready while CLI doctor says blocked. Different global/runtime/source-build scopes are collapsed into one readiness word. | Spec98 requires routes distinguish canonical scoped result, advisory result, blocked unscoped request, telemetry, runtime, and source-build planes. | Agent may continue or stop based on wrong readiness plane. |

### 13.2 Partial singleton-remnant / authority-plane clarity failures

| Spec102 issue | Diagnosis | Relation |
| --- | --- | --- |
| `PredictionCompactActionability` | Pi render hides scoped prediction ids/context while CLI JSON exposes project_root/continuity_id and unmatched scope binding. | Not a singleton bug by itself, but compact render fails to surface scoped authority metadata that Spec98 requires. |
| `MetacogCompactLessonLine` | Pi render hides useful lesson content/rehydrate refs while CLI JSON exposes scoped capture metadata. | Mostly render/actionability, but relies on clear read-index vs canonical cognition distinction from Spec98. |
| `WorkLoopBudgetRenderSchema` | `[object Object]` is a render bug, but work-loop current task/writer ownership is an orchestration plane that Spec98 says must stay separate from cognition authority. | Partial authority-plane clarity failure. |
| `SpecAvailabilityRegistry` | Spec-only features can look like available surfaces. | Related to Spec98's requirement that every route/surface declare mutation class, status taxonomy, and availability rather than letting agents infer authority from docs. |

### 13.3 Not primarily singleton-remnant failures

| Spec102 issue | Diagnosis |
| --- | --- |
| `UIAITokenizedToolSearch` | Search/ranking UX issue in UIAI discovery; not caused by Focusa singleton state. |
| `UIAIPressureSplit` | Telemetry presentation issue: current capacity vs historical pressure should be split. Related to telemetry-plane clarity, not singleton authority. |
| `DiagnosticsSeverityClassifier` | Browser diagnostics severity/classification issue; not singleton-related, though evidence handoff remains scoped. |

### 13.4 Extrapolated root cause pattern

The old singleton model leaves three kinds of agent-visible residue:

1. **Silent fallback residue** — a scoped request fails or is ambiguous, but the system returns an active/current object anyway.
2. **Split-plane residue** — Workpoint, Trajectory, Focus State, Ontology, Evidence, Doctor, and CLI/API/Pi each have valid local truth but no shared reconciliation envelope.
3. **Global-index residue** — large global arrays or read indexes are filtered per route; counts, search, and recent views disagree because scope is not the storage/index invariant.

### 13.5 Spec102 requirement updates from Spec98/99 review

Add the following foundation-aware requirements to Spec102 implementation planning:

#### 13.5.1 Scoped fallback disclosure

Any route that falls back from a requested id/scope to an active/current object must render:

```yaml
requested_scope:
requested_found: false
fallback_used: true
fallback_source: active_workstream | active_session | prior_project | none
fallback_object_id:
canonical_for_requested_scope: false
canonical_for_fallback_scope: true | false
misuse_hint:
```

If fallback would use daemon-global active/current state, canonical must be false unless the fallback object is proven under the same `ProjectRootKey + WorkstreamKey`.

#### 13.5.2 Authority-plane reconciliation card

Whenever Workpoint, Trajectory, Focus State, Ontology, Evidence, Doctor, or Work-loop disagree, render a reconciliation card:

```yaml
surface_states:
  workpoint:
  trajectory:
  focus_state:
  ontology:
  evidence:
  telemetry:
resolution:
authority_for_next_action:
supporting_context:
blocked_or_stale_surfaces:
next_repair_tool:
```

#### 13.5.3 Scoped index/source parity

Project card counts, traverse counts, evidence search, ontology search, and recent views must declare:

```yaml
source_index:
scope_key:
selector:
freshness:
count_semantics:
why_zero_if_empty:
try_next_selectors:
```

#### 13.5.4 Doctor readiness taxonomy

Doctor outputs must split readiness by authority plane:

```yaml
runtime_readiness:
project_scope_readiness:
workpoint_readiness:
trajectory_readiness:
focus_state_readiness:
source_build_readiness:
release_readiness:
telemetry_readiness:
ui_browser_readiness:
```

A single `ready`/`blocked` word is insufficient after Spec98/99.

#### 13.5.5 Post-compaction one-envelope handoff

Spec98 says post-compaction resume must remain simple: one Workpoint continuation contract with supporting ASCC, CLT, and Trajectory Ladder refs. Spec102 should require the Now Card to be that human/agent-facing one-envelope handoff:

```yaml
now_card:
  authority: workpoint | operator_current_ask | blocked
  project_root:
  continuity_id:
  workpoint_id:
  trajectory_status:
  focus_state_status:
  evidence_status:
  conflicts:
  exact_next_action:
```

### 13.6 Clean-repair UX acceptance bar

Repairs must feel like the issue never existed. A passing fix removes the underlying authority/render/index defect and also removes any user-visible residue from the repair itself.

Clean-repair criteria:

- No stale notices, temporary warnings, debug labels, fallback banners, or scar text remain after the normal state is restored.
- Error/fallback disclosures appear only while the condition is active and collapse back to the ordinary happy-path view when resolved.
- Canonical happy-path cards stay calm and minimal; they should not narrate past failures or mention internal repair history.
- Reconciliation/doctor/detail cards expose cause and recovery only on demand or when the current state is conflicting, blocked, stale, or non-canonical.
- Counts, search results, and readiness labels must agree without requiring the agent/operator to remember previous mismatches.
- Post-fix UI should preserve existing task flow, visual hierarchy, keyboard flow, and copy tone; no extra confirmation steps unless safety requires them.

Acceptance test framing: after each repair, rerun the original failing workflow plus the restored happy path. The happy path passes only if a fresh tester would not notice that a previous issue had been fixed.

### 13.7 Updated classification count

From the deeper-pass failure classes:

- Direct singleton-remnant / foundational-scope issues: 6
- Partial authority-plane/render fallout: 4
- Non-singleton UI/telemetry/search issues: 3

So **10 of 13** Spec102 issue classes are at least partly downstream of the Spec98/99 move away from singleton authority, and **6 of 13** are direct remnants of singleton/current-active behavior or global-index patching.

## 14. Repair backlog and invisible-UX acceptance matrix

This section converts the Spec102 findings into implementation-grade repair work. Every repair must satisfy Section 13.6: the final happy path must feel like the issue never existed.

### 14.1 Priority order

Repair order should remove authority-risk first, then index/search trust, then compact-render polish:

1. **P0 `RequestedIdFallbackDisclosure`** — prevent silent wrong-object continuation.
2. **P0 `TrajectoryWorkpointReconciliation`** — remove Workpoint/Trajectory split-brain in resume flows.
3. **P0 `FocusStateWorkpointBridge`** — reconcile Focus State write authority with canonical Workpoint continuation.
4. **P1 `DoctorReadinessCategories` + doctor drift cause lines** — split runtime/source/release/project readiness and make drift causes visible only when relevant.
5. **P1 `OntologyCountSourceParity`** — make project-card/traverse ontology counts explain source, selector, freshness, and zero semantics.
6. **P1 `EvidenceSearchIndexHealth`** — make just-linked evidence discoverable by scoped search or explain index lag/selector alternatives.
7. **P1 `PredictionCompactActionability` + `MetacogCompactLessonLine`** — include actionable ids and one useful line in Pi compact output.
8. **P2 `WorkLoopBudgetRenderSchema`** — remove `[object Object]` render and normalize budget display.
9. **P2 `UIAITokenizedToolSearch` + `UIAIPressureSplit` + `DiagnosticsSeverityClassifier`** — improve UIAI discovery, pressure semantics, and diagnostic severity.
10. **P2 `SpecAvailabilityRegistry` + Spec100/101 availability labels** — prevent spec-only capabilities from reading as implemented runtime features.
11. **P2 `ProjectIdentityMismatchSemantics`** — explain persisted/requested/verified identity differences without alarming normal happy paths.
12. **P2 `WrongIdConsistency`** — align missing Workpoint id and wrong continuity id behavior.

### 14.2 Implementation target map

| Requirement | Likely target surface | Output contract to change | Primary test shape |
| --- | --- | --- | --- |
| `RequestedIdFallbackDisclosure` | Workpoint resume route/tool + Pi render | requested id, found=false, fallback_used, fallback object, canonical flags | Request impossible Workpoint id; verify no silent canonical-for-requested result. |
| `TrajectoryWorkpointReconciliation` | Trajectory view/resume + Workpoint resume composition | reconciliation card with trajectory_status + workpoint_status + authority_for_next_action | Resume canonical Workpoint while Trajectory is provisional/fallback; verify one calm authority line. |
| `FocusStateWorkpointBridge` | Focus State write/read model + Workpoint checkpoint/resume bridge | focus_state_status, workpoint_status, block reason, repair route | Try Focus State write with canonical Workpoint; verify clear bridge or on-demand repair, not contradictory authority. |
| `DoctorReadinessCategories` | Pi tool doctor render + CLI doctor JSON/schema | runtime/project/workpoint/trajectory/source/release/telemetry readiness categories | Compare Pi doctor and CLI doctor; verify categories explain ready vs blocked planes. |
| Doctor drift cause lines | Tool doctor compact renderer | drift cause counts and source refs when drift=true only | Trigger/current drift; verify compact cause line, no happy-path scar. |
| `OntologyCountSourceParity` | Project card, traverse ontology selectors, ontology read index | source_index, scope_key, selector, freshness, count_semantics, why_zero_if_empty | Project card count >0 and traverse selector 0; verify explanation and next selector. |
| `EvidenceSearchIndexHealth` | Evidence capture/search/recent index | scoped search health, index freshness, fallback selectors, exact-handle lookup | Link Spec102 evidence; search by target/ref/text; verify discoverable or explicit lag. |
| `PredictionCompactActionability` | Pi prediction tool render | prediction ids, confidence, project/continuity scope, evaluation hint | Record/list prediction; verify id usable by evaluate without CLI JSON. |
| `MetacogCompactLessonLine` | Pi metacog retrieve/capture render | top lesson one-liner, why relevant, rehydrate id | Retrieve lessons; verify compact result tells agent what to do differently. |
| `WorkLoopBudgetRenderSchema` | Work-loop status renderer/schema | explicit budget fields or omitted empty budget | Read work-loop status; verify no `[object Object]`. |
| `UIAITokenizedToolSearch` | UIAI tool search ranking/tokenization | fuzzy/multi-token matches and split-query suggestions | Search `visual failure diagnostics`; verify useful tools returned. |
| `UIAIPressureSplit` | UIAI health renderer | current_capacity separate from historical_pressure | Read UIAI health; verify capacity is not confused with history. |
| `DiagnosticsSeverityClassifier` | UIAI diagnostics + Focusa intake | page_breaking / workflow_blocking / benign_asset / unknown severity | Capture diagnostics with benign asset failures; verify not over-alarming. |
| `SpecAvailabilityRegistry` | CLI/API/docs discovery + project card/spec cards | spec_only / partial / implemented / deprecated availability | Ask for Spec100/101 runtime state; verify spec-only is prominent. |
| `ProjectIdentityMismatchSemantics` | Project identity/verify/project card composition | persisted/requested/verified comparison and calm mismatch reason | Verify from `/root` vs project root; verify no unsafe cwd inference. |
| `WrongIdConsistency` | Workpoint resume validation | missing id and wrong continuity use same not_found/fallback taxonomy | Compare missing Workpoint id vs wrong continuity id behavior. |

### 14.3 Per-repair acceptance tests

Each repair must include two tests or proof steps:

```yaml
repair_id:
original_failing_workflow:
expected_failing_state_now:
repair_behavior:
restored_happy_path:
clean_repair_checks:
  no_stale_notice: true
  no_debug_label: true
  no_scar_text: true
  no_prior_issue_reference: true
  no_extra_step_unless_safety_required: true
  ordinary_copy_tone: true
  task_flow_preserved: true
proof_refs:
```

A repair is not complete until both paths pass:

1. **Original failing workflow** proves the defect is actually handled.
2. **Restored happy path** proves the normal UI/agent flow has no residual scar.

### 14.4 No-residual UX checklist

Apply this checklist to every CLI, Pi-tool, Focus Slice, browser/UIAI, and docs-facing repair:

- Normal state uses ordinary product language, not repair language.
- No banner, warning, or fallback disclosure remains after state is canonical and non-conflicting.
- Debug terms such as `fallback`, `stale`, `blocked`, `degraded`, `not_found`, or `index lag` appear only when currently true or inside explicit detail/debug views.
- The fix does not add an extra confirmation step to the happy path unless required by safety/destructive-action policy.
- The user/agent does not need to know the prior bug to interpret the output.
- Copy stays short enough for Pi compact surfaces and structured enough for CLI/API consumers.
- If a warning appears, it names the current condition and the next repair/action; it does not narrate historical failure.
- If a surface has both compact and full modes, compact mode shows the calm next action; full mode carries diagnostics.
- Screenshots/browser flows must not show temporary debug UI or old-state placeholders after repair.

### 14.5 Output examples

#### 14.5.1 Good Workpoint happy path

```text
WORKPOINT <id>: canonical=true · project=<name> · next=<exact next action>
```

No mention of fallback, prior missing id, or repair history.

#### 14.5.2 Good Workpoint requested-id failure

```text
Workpoint not found for requested id <requested_id> in this project/workstream.
Active Workpoint available: <active_id>.
Use active Workpoint? canonical_for_requested_scope=false; canonical_for_active_scope=true.
```

This disclosure appears only for the failing requested-id path.

#### 14.5.3 Good reconciliation happy path

```text
Now: Workpoint <id> is canonical. Trajectory is aligned. Next: <exact next action>.
```

#### 14.5.4 Good reconciliation conflict path

```yaml
surface_states:
  workpoint: canonical <id>
  trajectory: provisional; stale_or_missing_evidence_refs
resolution: use Workpoint for immediate next action; refresh Trajectory after evidence capture
```

#### 14.5.5 Good doctor happy path

```yaml
runtime_readiness: ready
project_scope_readiness: ready
workpoint_readiness: ready
source_build_readiness: not_checked
release_readiness: not_checked
```

No `blocked` headline if only source-build maintenance tooling is unavailable and the user is doing runtime work.

#### 14.5.6 Good evidence search happy path

```text
Evidence found: <evidence_ref> · target=<target_ref> · scope=<project/workstream>
```

#### 14.5.7 Good evidence search index-lag path

```yaml
search_status: index_lag
exact_handle_lookup: found
recent_view: found
retry_after: short
next_selector: target_ref
```

### 14.6 Promotion of remaining deeper-pass findings

The following deeper-pass findings were not fully covered by Section 13 and are now explicit Spec102 requirements:

#### 14.6.1 `ProjectIdentityMismatchSemantics`

Project identity, project verify, and project card outputs must explain mismatches as a scoped comparison:

```yaml
requested_project_root:
persisted_project_root:
verified_project_root:
matched_axes:
mismatched_axes:
authority_decision:
safe_next_action:
```

Happy path rule: when all axes match, render only the verified project and confidence; do not show mismatch machinery.

#### 14.6.2 `WrongIdConsistency`

Missing Workpoint id, wrong Workpoint id, wrong continuity id, and wrong project root must share one status taxonomy:

```yaml
requested_found: true | false
scope_found: true | false
fallback_available: true | false
fallback_used: true | false
canonical_for_requested_scope: true | false
canonical_for_fallback_scope: true | false
```

Happy path rule: valid id + matching scope renders only the canonical Workpoint card.

#### 14.6.3 `DoctorDriftCauseLine`

Doctor compact output must explain drift with bounded cause counts:

```text
drift=yes causes=stale_live_contracts:<n>, missing_docs:<n>, version_mismatch:<n>
```

Happy path rule: when drift=false, omit drift-cause machinery.

#### 14.6.4 `SpecRuntimeAvailabilityLabel`

Spec100/101 and future spec-only capabilities must show availability state whenever surfaced as possible tools/features:

```yaml
feature:
availability: spec_only | partial | implemented | deprecated
runtime_entrypoint:
docs_ref:
first_implementation_slice:
```

Happy path rule: implemented runtime features do not show spec-only caveats.

### 14.7 Iteration 1 done definition

Spec102 iteration 1 is done only when:

- Every Section 12 requirement has a Section 14 target surface, priority, and acceptance test.
- Every P0 repair has implementation proof and restored invisible happy-path proof.
- P1/P2 repairs have either implementation proof or an explicit deferred status with reason and next owner/slice.
- The Now/Why/Health/Do card contracts include authority, scope, readiness, and exact next action without cross-surface contradictions.
- Focus State, Workpoint, and Trajectory either reconcile under one envelope or clearly state current blocked/stale plane.
- Evidence and ontology surfaces expose count/search/source parity or a scoped reason why not.
- Pi compact renders provide enough ids/one-line content to act without CLI JSON for normal workflows.
- UIAI health/search/diagnostics outputs separate current condition from history and severity.
- Clean-repair checklist passes for every completed repair.
- A fresh tester can complete onboarding → checkpoint Workpoint → link evidence → compact/resume → follow next action without noticing any previous issue existed.

### 14.8 Repair report template

Use this template for each implementation PR/patch:

```yaml
repair_id:
priority:
target_files_or_routes:
operator_visible_change:
original_failure_proof:
happy_path_proof:
clean_repair_checklist_result:
residual_ui_risk:
residual_authority_risk:
follow_up_needed:
evidence_refs:
```

`residual_ui_risk` must be `none` for a completed repair. If risk remains, the repair is partial and must not be marked done.

## 15. Other-agent UX backlog not yet covered by Spec102

These improvements are known useful agent-UX upgrades that were not fully covered by Sections 13-14. They should follow the same invisible clean-repair standard from Section 13.6: normal happy paths stay calm, short, and free of residual repair/scar language.

### 15.1 Multi-agent ownership board

Focusa should expose a scoped ownership board for concurrent agents:

```yaml
ownership_board:
  project_root:
  continuity_id:
  workpoint_id:
  bead_id:
  active_agents:
    - agent_id:
      owns:
      touched_files:
      last_activity:
      lease_status: active | stale | released
  collision_risk:
  safe_next_action:
```

Purpose: prevent two agents from editing the same bead/file/surface or interpreting ownership from transcript tail.

Happy path: if there is only one active agent and no collision, show a compact `ownership: clear` line or omit details.

### 15.2 Agent handoff quality score

Every Workpoint/Focus Slice/Trajectory handoff should say whether the next agent can continue safely:

```yaml
handoff_quality:
  score: 0-100
  status: ready | partial | unsafe
  missing:
  stale:
  authority:
  next_action_quality:
  proof_quality:
```

Purpose: turn handoff from raw context into an explicit continuation safety signal.

Happy path: show `handoff: ready` plus exact next action; detailed missing/stale lists only when partial/unsafe.

### 15.3 Proof/artifact browser

Focusa should provide a browsable proof surface grouped by project, Workpoint, bead, spec, file, test, and confidence change:

```yaml
artifact_browser:
  group_by: workpoint | bead | spec | file | test | confidence_change
  filters:
  artifacts:
    - evidence_ref:
      target_ref:
      kind: test | screenshot | cli | file | api | browser_diag | report
      confidence_delta:
      freshness:
      rehydrate_ref:
```

Purpose: let agents find proof without transcript memory or huge global evidence counts.

Happy path: exact scoped proof appears quickly; no stale/index caveat unless currently true.

### 15.4 Dry-run / preview mode for Focusa mutations

State-changing Focusa actions should support preview/dry-run where feasible:

```yaml
mutation_preview:
  route:
  would_create:
  would_update:
  would_link:
  authority_scope:
  risk:
  irreversible: true | false
  safe_to_apply:
```

Targets: Workpoint checkpoint, evidence capture/link, trajectory define/assess promotion, state hygiene apply, work-loop control, project-card outcome, prediction evaluation, metacog capture.

Happy path: preview is optional and does not add a mandatory step for low-risk routine actions.

### 15.5 Undo / rollback affordance

Focusa should offer scoped rollback handles for accidental or low-confidence mutations:

```yaml
rollback_card:
  latest_safe_snapshot:
  reversible_actions:
  irreversible_actions:
  restore_tool:
  restore_scope:
  expected_after_restore:
```

Purpose: make recovery obvious after a wrong checkpoint, bad evidence link, stale trajectory definition, or hygiene mistake.

Happy path: rollback is available in details/review mode, not displayed as an alarming banner during normal work.

### 15.6 Trust badges per surface

Every agent-facing surface should use consistent short trust badges:

```text
canonical · advisory · projected · stale · degraded · blocked · spec_only · partial · verified · unsafe_scope
```

Purpose: replace inconsistent prose with a recognizable trust language across Pi tools, CLI/API, Focus Slice, UIAI intake, docs cards, and reports.

Happy path: show only positive badge(s), e.g. `canonical · verified`; avoid explaining stale/degraded taxonomy unless relevant.

### 15.7 Agent command palette

Focusa should expose a compact set of recommended commands/actions:

```yaml
command_palette:
  - label: Resume work
    tool:
    args_preview:
    when:
  - label: Link proof
  - label: Explain conflict
  - label: Start next bead
  - label: Make repair report
  - label: Run clean-repair check
```

Purpose: reduce tool-choice friction and make common routines discoverable.

Happy path: show top 3 commands only; full palette on request.

### 15.8 Route recommender

Focusa should recommend the best next tool route with why and expected output:

```yaml
route_recommendation:
  recommended_tool:
  why:
  expected_output:
  confidence:
  alternatives:
  avoid:
```

Purpose: prevent agents from calling broad/cold/deep tools when a bounded route exists.

Happy path: one concise recommended route, no verbose decision tree unless uncertainty is high.

### 15.9 Stuck-loop detector

Focusa should detect repeated cycles that do not change confidence or state:

```yaml
stuck_loop:
  detected: true | false
  repeated_actions:
  last_confidence_change:
  likely_cause:
  break_glass_action:
```

Examples: repeated resume/checkpoint/doctor calls, repeated evidence search misses, repeated trajectory assess without new evidence.

Happy path: silent when no loop exists.

### 15.10 Review mode before bead closure

Before closing a bead, Focusa should assemble a review packet:

```yaml
bead_review:
  bead_id:
  changed_files:
  linked_evidence:
  tests_run:
  clean_repair_checklist:
  residual_ui_risk:
  residual_authority_risk:
  next_follow_up:
  close_recommendation: close | keep_open | split_followup
```

Purpose: prevent premature closure without proof or invisible happy-path validation.

Happy path: if all checks pass, render a compact close recommendation.

### 15.11 Notification/change feed

Focusa should show what changed since the last agent turn/session/checkpoint:

```yaml
change_feed:
  since:
  files_changed:
  beads_changed:
  workpoints_changed:
  evidence_changed:
  predictions_changed:
  agents_changed:
  attention_required:
```

Purpose: improve multi-agent continuity without scanning git, beads, evidence, and Workpoint surfaces separately.

Happy path: if no relevant changes, show `changes: none relevant` or omit.

### 15.12 Agent-safe empty states

Every empty result must explain what kind of empty it is:

```yaml
empty_state:
  empty_because: none_exist | wrong_selector | wrong_scope | index_unavailable | permission_blocked | cold_path_disabled | not_checked
  scope:
  selector:
  next_selector:
  repair_or_retry:
```

Purpose: prevent agents from treating search/traverse/list emptiness as proof of absence.

Happy path: true empty states are calm and short.

### 15.13 Personalized verbosity profiles

Focusa should support consistent profile-specific renders:

```yaml
verbosity_profile:
  profile: operator | coding_agent | qa_agent | release_agent | debug_agent
  compact_fields:
  detail_fields:
  hidden_by_default:
  escalation_fields:
```

Purpose: show operators calm outcome/status, coding agents actionable ids/routes, QA agents proof/checklists, release agents readiness/risks, and debug agents internals.

Happy path: selected profile persists for the session/workstream and does not require repeated restating.

### 15.14 Evidence diffing

Focusa should compare old proof vs new proof and show what confidence changed:

```yaml
evidence_diff:
  before_ref:
  after_ref:
  changed_claims:
  confidence_delta:
  regressions:
  stale_refs_removed:
  new_followups:
```

Purpose: make evidence meaningful, not just accumulated.

Happy path: if new proof does not change confidence, state that clearly and suggest the next proof that would.

### 15.15 Recovery playbooks

Focusa should provide concise recovery playbooks for common agent failures:

```yaml
recovery_playbook:
  scenario:
  symptoms:
  first_safe_tool:
  next_tools:
  proof_to_capture:
  stop_conditions:
```

Required scenarios:

- project identity mismatch
- unsafe broad cwd/root
- stale or provisional Trajectory
- missing or wrong Workpoint id
- Focus State write blocked
- evidence index lag
- ontology selector empty
- doctor ready/blocked ambiguity
- UIAI pressure or diagnostics confusion
- stuck loop/no confidence change

Happy path: playbooks appear only when the scenario is active or when explicitly requested.

### 15.16 Section 15 acceptance criteria

Section 15 work is complete when:

- each backlog item has a bead and target surface;
- each completed item has original need proof and restored clean happy-path proof;
- profile-specific renders remain compact and non-alarming in normal states;
- multi-agent flows show ownership/change/handoff safety without requiring transcript memory;
- review/rollback/dry-run flows improve safety without adding needless happy-path friction;
- empty/search/proof surfaces never force agents to infer absence from silence;
- all completed improvements pass Section 13.6 no-residual UX criteria.

## 16. Full implementation assurance and no-deferral closure gate

Spec102 is not complete until the entire backlog is implemented or explicitly superseded by operator-approved spec change. Deferrals, partials, hidden gaps, and UX scars are not acceptable completion states.

### 16.1 No-deferral rule

Final Spec102 closure requires:

- all child beads under `focusa-pm2b` are closed;
- zero child beads remain `open`, `in_progress`, `blocked`, `deferred`, or `partial`;
- no repair report contains `residual_ui_risk` other than `none`;
- no repair report contains `residual_authority_risk` unless a follow-up bead remains open and the parent epic is not closed;
- no implementation note says `later`, `TODO`, `not implemented`, `stub`, `manual workaround`, or `known gap` without an open bead and operator-approved scope change;
- every spec-only or unavailable capability is either implemented, explicitly labeled as unavailable in runtime surfaces, or removed from happy-path presentation.

### 16.2 Missing-prep checklist before each implementation bead

Before coding any Spec102 repair bead, complete this prep packet:

```yaml
prep_packet:
  bead_id:
  requirement_id:
  target_routes_or_tools:
  target_files:
  current_failing_output_ref:
  current_happy_path_output_ref:
  regression_test_name:
  clean_repair_assertions:
  implementation_owner:
  collision_check:
  rollback_plan:
```

A repair bead may move to `in_progress` only after its prep packet has target files, failing proof, happy-path proof, and regression-test plan.

### 16.3 Full implementation proof matrix

Each bead must produce this proof matrix before closure:

```yaml
implementation_proof:
  bead_id:
  requirement_id:
  code_refs:
  test_refs:
  original_failure_before:
  fixed_failure_after:
  restored_happy_path_after:
  no_residual_ux_assertions:
    no_stale_notice:
    no_debug_label:
    no_scar_text:
    no_prior_issue_reference:
    no_extra_step_unless_safety_required:
    ordinary_copy_tone:
    task_flow_preserved:
  evidence_refs:
  residual_ui_risk: none
  residual_authority_risk: none
```

### 16.4 Final closure audit

Before closing `focusa-pm2b`, run a final audit that proves:

```yaml
spec102_final_audit:
  total_child_beads:
  closed_child_beads:
  open_child_beads: 0
  deferred_child_beads: 0
  blocked_child_beads: 0
  unimplemented_spec_items: 0
  missing_prep_packets: 0
  missing_proof_matrices: 0
  residual_ui_risk_items: 0
  residual_authority_risk_items: 0
  golden_flow_status: pass
  regression_suite_status: pass
  fresh_tester_invisible_repair_status: pass
```

The parent epic remains open until this audit passes.

### 16.5 Supersession-only escape hatch

If implementation discovers a Spec102 requirement is invalid, unsafe, or obsolete, the only valid non-implementation path is an operator-approved supersession:

```yaml
supersession:
  requirement_id:
  bead_id:
  reason:
  operator_approval_ref:
  replacement_requirement:
  updated_spec_ref:
  affected_tests:
```

Without supersession, missing work remains an open gap.

### 16.6 Assurance report template

Use this report when claiming full Spec102 completion:

```yaml
spec102_completion_report:
  epic_id: focusa-pm2b
  child_bead_count:
  all_closed: true
  no_deferrals: true
  no_known_gaps: true
  no_residual_ui: true
  no_residual_authority_risk: true
  golden_flow_evidence:
  regression_evidence:
  proof_matrix_index:
  supersessions: []
  operator_visible_summary:
```

Any `false`, non-empty supersession list, or missing evidence means Spec102 is not fully complete.
