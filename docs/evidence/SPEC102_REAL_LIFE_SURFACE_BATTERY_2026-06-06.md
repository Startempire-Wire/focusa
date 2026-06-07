# Spec102 Real-Life Agent Surface Battery Notes — 2026-06-06

Status: breadth-pass-1
Scope: Agent-accessible Focusa/UIAI/CLI/API/repo surfaces used in a real Pi session
Authority: evidence notes for iterating `docs/102-focusa-agent-ux-composition-and-real-life-test-spec.md`

## Summary

The battery confirmed the core product read: Focusa is an agent cognition operating system that binds project identity, Workpoints, Trajectory, evidence, predictions, metacog, resource pressure, and UIAI handoff into scoped agent continuity.

The strongest UX pattern is canonical Workpoint continuation plus bounded evidence handles. The weakest pattern is cross-surface composition: several surfaces are individually useful but disagree, omit causes, or fail to explain how to reconcile state.

## Top findings

1. **Identity contradiction needs resolver.** `focusa_project_identity` can report mismatch/low confidence while `focusa_project_verify` reports verified/high confidence for the same requested root.
2. **Trajectory/Workpoint authority split needs reconciliation.** `trajectory_view` reported no canonical packet for current continuity while `workpoint_resume` returned a canonical Workpoint.
3. **Missing Workpoint id resume fell back silently.** Requesting an impossible id returned the active canonical Workpoint, which is useful but should explicitly say fallback occurred.
4. **Ontology counts disagree.** `project_card` reported `ontology_objects=11`; `focusa_traverse surface=ontology selector=summaries/search` returned 0.
5. **Tool doctor drift lacks immediate causes.** `contracts=63 live_contracts=63 drift=yes` needs top drift causes in the compact text.
6. **Token pressure vs resource pressure is confusing.** `tool_doctor` reported token_budget=critical while `resource_mode` reported normal/within_budget.
7. **UIAI tool search is too literal.** Query `search markdown diagnostics browser read` returned 0, while graph/card clearly know those tools.
8. **Work-loop status leaks `[object Object]`.** `budget_remaining=[object Object]` is not operator/agent-friendly.
9. **Prediction recent/stats are too terse.** `predictions recent → 8` and stats counts lack actionable top open predictions or evaluation candidates.
10. **Context Cognition and Bloatgaurd are spec-only or mostly spec-only.** Repo search found Spec100/101 docs but no CLI/API command for `context-cognition` and only a temporary `scripts/focusa-bloat-gate` wrapper.

## Surface notes

### Project/bootstrap surfaces

```yaml
surface: project_identity_from_broad_root
tool_or_route: focusa_project_identity cwd=/root project_root=/home/wirebot/focusa
status: pass
observed_output: verified high confidence root=/home/wirebot/focusa
ux_issue: good recovery from unsafe cwd when explicit candidate supplied
composition_issue: prior run showed mismatch possible for same root; needs reason when persisted identity conflicts
authority_issue: project_root+continuity_id boundary clear
recommendation: add identity mismatch explanation and persisted-vs-requested comparison
spec102_update_needed: yes
```

```yaml
surface: project_verify
tool_or_route: focusa_project_verify
status: pass
observed_output: verified=true confidence=high
ux_issue: concise and clear
composition_issue: should pair with identity mismatch resolver when identity says mismatch
recommendation: Now Card should include both identity and verify status with reconciliation line
spec102_update_needed: yes
```

```yaml
surface: project_card
tool_or_route: focusa_project_card
status: pass
observed_output: compact project dashboard; hlg/stg, inferred_wp, predictions, ontology_objects
ux_issue: very useful but dense
composition_issue: ontology_objects count later disagreed with traverse ontology results
recommendation: Project Card should expose source/selector for ontology count
spec102_update_needed: yes
```

```yaml
surface: session_transfer
tool_or_route: focusa_session_transfer status
status: pass
observed_output: saved=false resume=completed inferred_wp=verify_or_fix_tests
ux_issue: useful status but ambiguous difference between saved=false and resume=completed
recommendation: add short explanation: no saved transfer packet, but project-card resume guidance available
spec102_update_needed: yes
```

### Workpoint surfaces

```yaml
surface: canonical_resume
tool_or_route: focusa_workpoint_resume id=019e9f27...
status: pass
observed_output: canonical=true with mission/action/next/do-not-drift
ux_issue: excellent continuation gravity
recommendation: preserve as core Now Card source
spec102_update_needed: yes
```

```yaml
surface: missing_id_resume
tool_or_route: focusa_workpoint_resume id=00000000-0000-0000-0000-000000000000
status: confusing
observed_output: returned active canonical Workpoint instead of not_found or explicit fallback
ux_issue: can mislead agent into thinking requested id existed
authority_issue: fallback should be explicit and tagged
recommendation: render `requested_workpoint_not_found; fell_back_to_active_workpoint=<id>`
spec102_update_needed: yes
```

```yaml
surface: active_object_resolve
tool_or_route: focusa_active_object_resolve hint=docs/102...
status: pass
observed_output: count=1 verified=false refs=docs/102...
ux_issue: useful but should explain verified=false means unresolved canonical ontology binding, not bad file
recommendation: add verification meaning hint
spec102_update_needed: yes
```

```yaml
surface: workpoint_link_evidence_noop
tool_or_route: focusa_workpoint_link_evidence attach_to_workpoint=false
status: pass
observed_output: no_op attach_to_workpoint=false
ux_issue: safe; terse
recommendation: include whether payload was valid but intentionally not linked
spec102_update_needed: minor
```

### Trajectory surfaces

```yaml
surface: trajectory_view
tool_or_route: focusa_trajectory_view
status: degraded/confusing
observed_output: prior project fallback; says no canonical packet and suggests checkpoint
composition_issue: canonical Workpoint exists from resume tool
authority_issue: conflict between Trajectory advisory fallback and Workpoint canonical continuation
recommendation: add trajectory-workpoint reconciliation text and link/refresh suggestion
spec102_update_needed: yes
```

```yaml
surface: trajectory_assess
tool_or_route: focusa_trajectory_assess
status: pass
observed_output: gaps=1 action=propose_workpoint canonical=false
ux_issue: too terse; no gap text in compact render
recommendation: include top gap summary in compact result
spec102_update_needed: yes
```

```yaml
surface: trajectory_propose_workpoint
tool_or_route: focusa_trajectory_propose_workpoint
status: pass
observed_output: advisory=true checkpoint_required=true no_execution=true
ux_issue: good safety labeling
recommendation: preserve and include candidate next-action text when possible
spec102_update_needed: minor
```

```yaml
surface: trajectory_checkpoint
tool_or_route: focusa_trajectory_checkpoint
status: pass
observed_output: status=completed persisted=true canonical=true
ux_issue: clear but could include checkpoint id/reference
recommendation: return durable handle/id in compact text
spec102_update_needed: yes
```

### Traverse/ontology/evidence/tool registry surfaces

```yaml
surface: ontology_summaries
tool_or_route: focusa_traverse surface=ontology selector=summaries
status: confusing
observed_output: returned=0/0 while project_card says ontology_objects=11
composition_issue: count/source mismatch
recommendation: if ontology has no summaries, suggest valid selectors or explain object source from project_card
spec102_update_needed: yes
```

```yaml
surface: ontology_search
tool_or_route: focusa_traverse surface=ontology selector=search query=Workpoint Trajectory Evidence ProjectIdentity
status: confusing
observed_output: returned=0/0
ux_issue: search over core ontology terms returning nothing feels broken
recommendation: improve default ontology search/index or render `ontology search index unavailable`
spec102_update_needed: yes
```

```yaml
surface: evidence_recent
tool_or_route: focusa_traverse surface=evidence selector=recent limit=8
status: pass
observed_output: returned=8/19848 truncated=true cursor=8
ux_issue: bounded but lacks top/confidence-changing grouping
recommendation: add evidence views by Workpoint, active object, confidence-change, stale, duplicate cluster
spec102_update_needed: yes
```

```yaml
surface: workpoints_recent
tool_or_route: focusa_traverse surface=workpoints selector=recent limit=8
status: pass
observed_output: returned=8/32 truncated=true
ux_issue: bounded; needs richer card when used for active selection
recommendation: provide active/recent/stale grouped view
spec102_update_needed: minor
```

```yaml
surface: tool_registry_summary
tool_or_route: focusa_traverse surface=tool_registry selector=summaries
status: pass
observed_output: returned=1/1
ux_issue: too compact for parity diagnosis but safe
recommendation: add common selector hints for contract drift
spec102_update_needed: minor
```

### Prediction/metacog surfaces

```yaml
surface: prediction_record
tool_or_route: focusa_predict_record
status: pass
observed_output: recorded
ux_issue: no prediction id shown in compact text
recommendation: return prediction id so it can be evaluated later
spec102_update_needed: yes
```

```yaml
surface: prediction_recent
tool_or_route: focusa_predict_recent limit=8
status: confusing
observed_output: predictions recent → 8
ux_issue: no actual predictions visible
recommendation: compact render should include top 3 open predictions with ids/confidence
spec102_update_needed: yes
```

```yaml
surface: prediction_stats
tool_or_route: focusa_predict_stats
status: pass/minimal
observed_output: 288 predictions, 53 evaluated
ux_issue: useful but missing calibration by type and next evaluation candidates
recommendation: add top accuracy classes and stale open predictions
spec102_update_needed: yes
```

```yaml
surface: metacog_doctor_retrieve
工具_or_route: focusa_metacog_doctor + focusa_metacog_retrieve
status: pass/minimal
observed_output: candidates=5 top workflow_signal
ux_issue: candidates hidden; no immediate lesson text except top kind/id
recommendation: compact mode should include one-line top lesson and why relevant
spec102_update_needed: yes
```

### Work-loop and hygiene surfaces

```yaml
surface: work_loop_writer_status
tool_or_route: focusa_work_loop_writer_status
status: pass
observed_output: active_writer=daemon-supervisor status=idle preflight=read_only
ux_issue: clear
recommendation: preserve
spec102_update_needed: no
```

```yaml
surface: work_loop_status
tool_or_route: focusa_work_loop_status
status: degraded
observed_output: budget_remaining=[object Object]
ux_issue: object serialization leak
recommendation: render budget fields explicitly or omit from compact text
spec102_update_needed: yes
```

```yaml
surface: work_loop_pause_preflight
tool_or_route: focusa_work_loop_control action=pause preflight=true
status: pass
observed_output: route=/work-loop/pause writer=daemon-supervisor mutates=false
ux_issue: excellent mutation safety labeling
recommendation: preserve and use in Now/Do card for risky controls
spec102_update_needed: minor
```

```yaml
surface: state_hygiene
tool_or_route: focusa_state_hygiene_doctor + plan
status: pass
observed_output: signals=0 duplicate_groups=0 stale_candidates=0 recommended=no_hygiene_needed
ux_issue: clear and safe
recommendation: preserve
spec102_update_needed: no
```

### Doctor/resource surfaces

```yaml
surface: tool_doctor
tool_or_route: focusa_tool_doctor scope=all
status: pass/confusing
observed_output: readiness=ready contracts=63 live_contracts=63 drift=yes token_budget=critical resource=normal/within_budget workpoint=not_found
ux_issue: drift cause absent; token/resource distinction confusing; workpoint not_found conflicts with explicit Workpoint resume
recommendation: add top causes and scope-specific active Workpoint reconciliation
spec102_update_needed: yes
```

```yaml
surface: resource_mode
tool_or_route: focusa_resource_mode status
status: pass
observed_output: mode=normal forced=false reason=within_budget
ux_issue: clear but should be shown separately from token budget
recommendation: Health Card should separate daemon resource pressure from context/token pressure
spec102_update_needed: yes
```

### UIAI/browser surfaces

```yaml
surface: uiai_agent_card
tool_or_route: pi_uiai_agent_card
status: pass
observed_output: compact workflows and discovery endpoints
ux_issue: good, but long enough that card could have profile-specific compactness
recommendation: add Bloatgaurd profile-aware UIAI card verbosity later
spec102_update_needed: minor
```

```yaml
surface: uiai_tool_search
tool_or_route: pi_uiai_tool_search q="search markdown diagnostics browser read"
status: confusing
observed_output: count=0
ux_issue: multi-intent query failed despite graph containing all queried concepts
recommendation: fuzzy tokenized search or suggest splitting query
spec102_update_needed: yes
```

```yaml
surface: uiai_tool_graph
tool_or_route: pi_uiai_tool_graph
status: pass
observed_output: rich related-tools graph and Focusa integration metadata
ux_issue: very useful but large
recommendation: add compact workflow card and full graph cold opt-in
spec102_update_needed: yes
```

```yaml
surface: uiai_health
tool_or_route: uiai_health
status: degraded/confusing
observed_output: browser idle-off active_pages=0 but pressure=saturated due historical queue/errors
ux_issue: current availability and historical pressure are mixed
recommendation: split current_capacity from historical_pressure
spec102_update_needed: yes
```

```yaml
surface: uiai_browser_open_read_snapshot_diagnostics_close
tool_or_route: browser_open/read/snapshot/diagnostics/close example.com
status: pass
observed_output: read and snapshot bounded; diagnostics captured favicon 404; focusa_scope echoed; close succeeded
ux_issue: diagnostics can flag expected favicon 404 as failure; useful but needs severity classification
recommendation: diagnostics summary should distinguish page-breaking failures from benign asset failures
spec102_update_needed: yes
```

```yaml
surface: browser_diagnostics_intake
tool_or_route: focusa_browser_diagnostics_intake
status: pass
observed_output: completed evidence=uiai-diagnostics:session=... active_object_hints=browser:https://example.com/,unknown-url,browser_diagnostics
ux_issue: unknown-url hint is noisy when URL is known
recommendation: suppress unknown-url when target_ref is explicit
spec102_update_needed: minor
```

### CLI/API/docs surfaces

```yaml
surface: cli_discovery
tool_or_route: focusa --help
status: pass
observed_output: rich command catalog including project/resource/trajectory/traverse/workpoint/tokens
ux_issue: no context-cognition or bloatgaurd command yet
recommendation: once Spec100/101 implemented, add discoverable commands or docs pointer
spec102_update_needed: yes
```

```yaml
surface: cli_status_agent
tool_or_route: focusa status --agent
status: pass
observed_output: completed status envelope with next action, why, command, recovery, evidence, docs
ux_issue: excellent output pattern
recommendation: Now/Health/Do cards should reuse this envelope style
spec102_update_needed: yes
```

```yaml
surface: cli_tokens
tool_or_route: focusa tokens doctor + compact-plan
status: pass
observed_output: critical token class, compact-plan recommends ECS handles and checkpoint
ux_issue: good; should integrate with Bloatgaurd Squeezer routine
recommendation: connect token doctor to Spec101 profiles/routines later
spec102_update_needed: yes
```

```yaml
surface: api_health_status
tool_or_route: curl /v1/health + /v1/status
status: pass
observed_output: health ok; status huge but bounded with cold_omitted and resource budget details
ux_issue: /v1/status compact still large for agent prompt; contains stack_depth=12920 and telemetry totals
recommendation: keep status API but Pi prompt should consume smaller Now/Health card
spec102_update_needed: yes
```

```yaml
surface: docs_refs
tool_or_route: ls docs/100,101,102,current pointer
status: pass
observed_output: docs exist
ux_issue: current pointer works for discoverability
recommendation: add index link later if docs index exists
spec102_update_needed: minor
```

### MCP/SilentSession surfaces

```yaml
surface: mcp_gateway
tool_or_route: mcp status
status: unavailable
observed_output: 0/0 servers, 0 tools
ux_issue: agent-accessible but empty; should not imply failure
recommendation: utility card should say MCP unavailable/not configured when 0 servers
spec102_update_needed: minor
```

```yaml
surface: silent_sessions
tool_or_route: focusa_silent_sessions list
status: pass
observed_output: none
ux_issue: clear
recommendation: preserve
spec102_update_needed: no
```

### Spec100/Spec101 implementation presence

```yaml
surface: context_cognition_presence
tool_or_route: rg context-cognition + focusa help
status: missing/spec_only
observed_output: references only in docs/100; no CLI command found
ux_issue: Spec100 is strong but not operationally discoverable yet
recommendation: mark as implementation gap and avoid presenting as available runtime feature
spec102_update_needed: yes
```

```yaml
surface: bloatgaurd_presence
tool_or_route: rg bloatgaurd/bloat
status: mostly_spec_only
observed_output: Spec101 and current pointer exist; temporary scripts/focusa-bloat-gate exists; no full profile/routine implementation
ux_issue: names/profiles/routines are not yet operator-facing runtime surfaces
recommendation: first implementation slice should expose read-only Bloatgaurd profile/routine status card
spec102_update_needed: yes
```

## Ranked UX improvements from breadth-pass-1

1. Add **Now Card** compositional render.
2. Add **Conflict Resolver** for identity/verify, trajectory/workpoint, doctor/workpoint, token/resource contradictions.
3. Add **Context Receipt** showing included/excluded/omitted/rehydrate refs.
4. Add **Ask-to-Workpoint Bridge** when operator ask differs from active Workpoint.
5. Add **Tool Doctor Cause Lines** for drift/token/resource/workpoint statuses.
6. Add **Ontology Selector Hints** and count-source reconciliation.
7. Add **Prediction/Metacog Compact Details** with ids and top relevant one-liners.
8. Add **Evidence Navigation Views** by active object, Workpoint, confidence-change, stale, duplicates.
9. Add **UIAI current-vs-historical pressure split** and fuzzy multi-intent tool search.
10. Add **Spec100/101 availability labels** so spec-only capabilities do not look runtime-ready.

## Spec102 iteration recommendations

Add or refine requirements for:

- `FocusaNowCard` envelope
- `FocusaConflictResolver` envelope
- `FocusaContextReceipt` envelope
- `AskToWorkpointBridge` behavior
- `DoctorCauseSummary` compact text
- `OntologyTraverseHint` fallback behavior
- `EvidenceConfidenceView`
- `PredictionActionabilityView`
- `MetacogTopLessonView`
- `UIAIPressureSplit`
- `SpecAvailabilityRegistry` for planned/spec-only vs implemented/runtime features

## Follow-up test gaps

The breadth pass did not fully exercise destructive/mutating controls, full work-loop selection, state hygiene apply, branch/restore lineage rollback, or any external MCP server because those require explicit authorization, configured servers, or riskier mutation. These should be tested with fixtures or preflight-only harnesses in the next iteration.

---

# Deeper Agent UX Failure Pass — 2026-06-06

Status: deeper-pass-1
Purpose: expand from breadth findings into precise failure modes, reproduction probes, and UX impact.

## D1. Project identity behavior is good when explicit, but mismatch semantics remain opaque

### Probe

- `focusa_project_identity cwd=/root`
- `focusa_project_identity cwd=/root project_root=/home/wirebot/focusa`
- `focusa_project_identity cwd=/home/wirebot/focusa project_root=/home/wirebot/focusa`
- `focusa_project_verify cwd=/root project_root=/home/wirebot/focusa ...`

### Observed

- With only `cwd=/root`, identity preserved `/home/wirebot/focusa` from session and said incoming `/root` result rejected as different project.
- With explicit `project_root=/home/wirebot/focusa`, identity verified high confidence.
- Verify also passed high confidence.

### UX failure

The system can recover correctly, but it does not expose a reusable reconciliation structure. Earlier runs showed `project_identity` mismatch while `project_verify` verified. Deeper run shows there is useful internal logic (“incoming result rejected as different project”) that should become a standard conflict explanation.

### Impact on agent intelligence

Good preservation prevents cross-project drift, but inconsistent wording can cause the agent to over-trust or under-trust the current project scope.

### Recommendation

Add `ProjectIdentityConflictCard`:

```yaml
requested_root:
preserved_root:
incoming_root:
decision: preserved | switched | rejected | verified
why:
operator_action_needed: false | true
next_tool:
```

## D2. Missing Workpoint id silently falls back to active Workpoint

### Probe

`focusa_workpoint_resume workpoint_id=00000000-0000-0000-0000-000000000000 ...`

### Observed

Returned active canonical Workpoint `019e9f2f-c6d5-7101-9133-83aa2b4af872`, not an explicit not-found result.

### UX failure

This is a serious agent UX ambiguity. A caller asking for a specific id may believe that id resolved. The fallback behavior is useful, but must be made explicit.

### Impact on agent intelligence

Can produce false lineage confidence and hide a broken handoff reference.

### Recommendation

Return:

```yaml
status: fallback_to_active
requested_workpoint_id: 00000000-0000-0000-0000-000000000000
requested_found: false
fallback_workpoint_id: 019e9f2f-c6d5-7101-9133-83aa2b4af872
canonical: true
misuse_hint: requested id was not found; active Workpoint shown for recovery only
```

## D3. Wrong continuity id behavior is safer than wrong Workpoint id behavior

### Probe

`focusa_workpoint_resume continuity_id=totally-wrong-continuity`

### Observed

Returned `status=not_found canonical=false` with recovery instruction to checkpoint current mission.

### UX failure

This behavior is clearer than missing Workpoint id fallback. Inconsistency between the two cases is the failure.

### Recommendation

Align missing Workpoint id behavior with wrong continuity behavior: either hard not_found or explicit fallback status.

## D4. Trajectory fallback conflicts with canonical Workpoint existence

### Probe

- `focusa_trajectory_view allow_prior_project_trajectory=false`
- `focusa_trajectory_view allow_prior_project_trajectory=true`
- `focusa_workpoint_resume canonical Workpoint`

### Observed

- Without prior fallback: bootstrap default, needs `focusa_trajectory_define_goal`.
- With prior fallback: prior project fallback but says no canonical Focusa packet for project_root+continuity_id and suggests checkpoint.
- Workpoint resume is canonical for same project/continuity.

### UX failure

Trajectory and Workpoint are correctly separate authorities, but the user-facing text reads like Workpoint does not exist.

### Impact on agent intelligence

Agents may waste time re-checkpointing or mistrust canonical Workpoint state.

### Recommendation

Add `TrajectoryWorkpointReconciliation`:

```yaml
trajectory_status: bootstrap_default | prior_project_fallback | active
workpoint_status: canonical | missing | stale
relationship: unlinked | linked | mismatched
message: Trajectory is advisory fallback; Workpoint is canonical for immediate action.
recommended_action: refresh trajectory goal or link current Workpoint; do not recreate Workpoint unless current ask changed.
```

## D5. Ontology traversal appears empty across multiple selectors

### Probe

- `focusa_traverse surface=ontology selector=summaries`
- `selector=head`
- `selector=recent`
- `selector=window`
- `selector=search query="Workpoint Trajectory Evidence ProjectIdentity"`

### Observed

All returned `0/0`. Project Card says `ontology_objects=11`.

### UX failure

This is a concrete composition inconsistency. Either Project Card ontology_objects comes from a different source, or traverse ontology selectors do not expose that ontology layer.

### Impact on agent intelligence

Ontology is supposed to be semantic spine; empty ontology traversal makes the agent think ontology is unavailable or broken.

### Recommendation

Add one of:

1. route-level explanation:
```text
ontology summaries unavailable; project_card ontology_objects counts tool-contract ontology objects, not traverse ontology index
```

2. selector hint:
```text
try surface=tool_registry or selector=active_context
```

3. shared ontology summary source so Project Card and traverse agree.

## D6. Evidence search misses newly linked Spec102 evidence while recent shows huge store

### Probe

- `focusa_traverse surface=evidence selector=recent limit=8`
- `focusa_traverse surface=evidence selector=search query=Spec102 limit=8`

### Observed

- recent: `8/19848`, truncated.
- search Spec102: `0/0`.

### UX failure

The evidence store is enormous, and search failing for a just-created evidence target makes evidence feel non-navigable.

### Impact on agent intelligence

Agents cannot reliably find proof they just captured unless they retain exact handle from transcript.

### Recommendation

Evidence search should index target_ref/result/evidence_ref and recent captures. Add `evidence_recent_for_workpoint` and `evidence_search_index_status`.

## D7. Prediction tool has rich CLI JSON but terse Pi tool render

### Probe

- Pi: `focusa_predict_recent limit=8` → `predictions recent → 8`
- CLI: `focusa --json predict recent --limit 3`

### Observed

CLI JSON includes prediction ids, outcomes, confidence, context refs, project roots, continuity ids, recommendations, and why. Pi compact render hides all of that.

### UX failure

The agent-facing Pi tool loses actionability. The CLI proves the data exists.

### Recommendation

Pi compact render should include top 1-3 predictions:

```text
predictions recent → 8
- 019e... ux_surface_battery conf=0.78 open: composition inconsistencies likely; action=record/rank Spec102 findings
```

## D8. Metacog Pi render is too terse while CLI JSON is good

### Probe

- Pi: `focusa_metacog_doctor/retrieve`
- CLI: `focusa --json metacognition doctor/retrieve`

### Observed

CLI JSON includes top lesson summary, confidence, strategy_class, capture_id, rehydrate route, tags. Pi compact render shows candidates count and top kind/id only.

### UX failure

The agent misses the actual reusable lesson unless it runs CLI JSON or extra rehydrate.

### Recommendation

Pi compact metacog render should include the top lesson summary line and capture id.

## D9. Tool doctor vs CLI doctor disagreement creates reliability ambiguity

### Probe

- Pi: `focusa_tool_doctor scope=all`
- CLI: `focusa --json doctor`

### Observed

- Pi tool doctor: `readiness=ready`, `contracts=63 live_contracts=63`, `drift=yes`, `token_budget=critical`, `resource=normal`.
- CLI doctor: `status=blocked`, summary `1 doctor check(s) blocked`; details include missing PATH helpers such as cargo/rustc/node/npm for source-build/maintainer paths.

### UX failure

Both outputs may be correct under different definitions, but they read as contradictory: ready vs blocked.

### Impact on agent intelligence

An agent may proceed or halt incorrectly depending on which doctor it consulted.

### Recommendation

Doctor surfaces need category-specific readiness:

```yaml
runtime_readiness: ready
source_build_readiness: blocked
pi_extension_readiness: blocked
contract_readiness: drift
release_readiness: blocked
recommended_scope: continue runtime work; do not claim source-build/release proof
```

## D10. Tool doctor drift is not explained compactly

### Probe

`focusa_tool_doctor scope=all`

### Observed

`contracts=63 live_contracts=63 drift=yes` with no top causes in compact render.

### UX failure

Drift without cause is an anxiety signal, not an action signal.

### Recommendation

Compact render must include top cause categories:

```text
drift=yes causes=stale_live_contracts:3, docs_missing:1, static/live_version_mismatch:1
```

## D11. UIAI tool search fails multi-word conjunctive phrases

### Probe

- `pi_uiai_tool_search q="search"` → 4 results
- `q="markdown"` → 2 results
- `q="diagnostics"` → 10 results
- `q="read"` → 9 results
- `q="browser read"` → 0 results
- earlier `q="search markdown diagnostics browser read"` → 0 results

### UX failure

The tool search appears to use phrase/substring matching rather than tokenized OR/AND ranking. Agents naturally issue multi-word intent queries.

### Recommendation

Implement tokenized fuzzy search:

- exact phrase boost
- all-token match boost
- any-token fallback
- show suggestion: `No exact match for "browser read"; showing results for "browser" and "read"`

## D12. Work-loop budget renders `[object Object]`

### Probe

`focusa_work_loop_status`

### Observed

`budget_remaining=[object Object]`

### UX failure

Raw object stringification leak.

### Recommendation

Render fields explicitly or omit:

```text
budget_remaining: turns=..., ms=..., items=...
```

## D13. UIAI health mixes current capacity and historical pressure

### Probe

`uiai_health`

### Observed

- browser idle/off, active_pages=0
- overall_pressure=saturated
- queue_depth=0 but p95 historical 19289 and rejected=3
- errors stored_count high/saturated

### UX failure

Agent sees saturated despite no active pages and may avoid browser unnecessarily.

### Recommendation

Split:

```yaml
current_capacity: available | unavailable
historical_pressure: normal | high | saturated
error_backlog: normal | high | saturated
recommended_action:
```

## D14. Browser diagnostics severity needs classification

### Probe

Open/read/snapshot/diagnostics on `https://example.com`.

### Observed

Main page status 200, favicon 404 appears as failed_request/http_4xx=1.

### UX failure

Technically correct, but severity can be misleading. Favicon 404 is usually non-blocking.

### Recommendation

Classify diagnostics:

```yaml
page_health: ok
blocking_failures: 0
nonblocking_failures: 1
examples: favicon 404
```

## D15. Focus State note write failed despite canonical Workpoint

### Probe

`focusa_recent_result` after breadth-pass evidence capture.

### Observed

Rejected: “Attentive and awaiting operator direction… project-bound frame missing/fallback.” Scratchpad fallback used. Workpoint checkpoint then succeeded.

### UX failure

Focus State slots and Workpoint authority diverged. The tool guidance says verify project/checkpoint, but the canonical Workpoint already existed.

### Recommendation

Focus State write tools should render:

```yaml
blocked_reason: focus_frame_not_bound_to_project
workpoint_status: canonical_exists
safe_alternative: focusa_workpoint_checkpoint or evidence_capture
repair: bind/reopen Focus frame to project_root+continuity_id
```

## D16. Context Cognition/Bloatgaurd spec-only state is not prominently labeled in CLI

### Probe

- `rg context-cognition`
- `focusa --help`
- `rg bloatgaurd/bloat`

### Observed

Spec100/101 are in docs. No `focusa context-cognition` command visible. Bloatgaurd has spec and a small temporary script only.

### UX failure

Spec-rich features can sound available before runtime surfaces exist.

### Recommendation

Add `SpecAvailabilityRegistry`:

```yaml
feature: Context Cognition
status: spec_only | partial | implemented | deprecated
surfaces_available: []
first_runtime_slice:
```

## Deeper-pass ranked failures

1. Missing Workpoint id silent fallback.
2. Trajectory/Workpoint reconciliation gap.
3. Doctor ready-vs-blocked category ambiguity.
4. Ontology/project-card count mismatch.
5. Evidence search failing for just-created Spec102 evidence.
6. Focus State write blocked despite canonical Workpoint.
7. Prediction/metacog Pi compact renders hide actionable details.
8. UIAI multi-word tool search failure.
9. UIAI current-vs-historical pressure ambiguity.
10. Work-loop `[object Object]` render.

## Deeper-pass iteration requirements for Spec102

Add explicit requirements for:

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
