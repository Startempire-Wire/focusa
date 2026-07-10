# Focusa Trajectory Audit Phase 2 — Generic and Flailing Patterns

**Work item:** `focusa-h7rro.2`  
**Prereq:** `docs/130-focusa-trajectory-audit-inventory.md`  
**Goal:** identify where Trajectory is descriptive, generic, overblocking, underblocking, or shuttling fields without a strong opinion.

## 1. Summary verdict

Trajectory has the right surface area, but the current behavior is too often an advisory projection rather than a strong mission guide. The strongest code is scope safety and Workpoint-vs-Trajectory reconciliation. The weakest code is gap/action specificity, define-goal validation, placeholder intelligence fields, and assess/propose guidance.

## 2. Placeholder or non-actionable `intelligence_view` fields

| Field | Current pattern | Why it flails | Desired direction |
|---|---|---|---|
| `learning_refs` | Always `[]`. | Looks like learning is considered, but no retrieval or rehydrate ref exists. | Either populate from metacog/prediction context or omit/degraded-note it. |
| `prediction_refs` | Always `[]`. | Same as learning refs; it implies prediction awareness without carrying proof. | Populate recent prediction ids or return `not_integrated`. |
| `stale_refs` | Always empty vector. | Clarity gate can emit `stale_or_missing_evidence_refs`, but stale refs never identify what is stale. | Return concrete missing/stale evidence handles or drop stale wording. |
| `tool_affordances` | Static list: view, Workpoint resume, object resolve, evidence capture. | Generic suggestions do not change with gap state. | Pick one `recommended_tool_call` based on status/gap. |
| `ask_operator_if` | Broad operator questions. | Does not always say exact field/value needed. | Include exact missing fields and example command. |
| `do_not_use` | Guard list is sometimes generic. | It warns but does not always block downstream action. | Pair each guard with enforcement status or repair command. |
| `recent_results` / `decisions` / `constraints` | Copied from Focus State. | Useful context, but no judgement on relevance or freshness. | Add source/freshness and avoid treating copied slots as trajectory truth. |

## 3. Vague or non-actionable gap descriptions

| Gap text | Source pattern | Problem | Stronger behavior |
|---|---|---|---|
| `Current verified state differs from desired end state` | Fallback when desired/current differ but Workpoint has no next/action. | True but not actionable; no next tool call or target. | Include `next_tool=focusa_trajectory_assess` or `focusa_workpoint_checkpoint` with exact missing action. |
| `Trajectory gap unclear until desired end state and current verified state are both present` | Desired/current missing. | Good diagnosis, weak recovery. | Return required field names and `focusa trajectory define-goal ... --current-state ...`. |
| `Trajectory definition required before ladder projection` | HLT bootstrap placeholder. | Good warning, but repeats without exact minimal command. | Include `focusa trajectory define-goal --long-term-goal ... --desired-end-state ...`. |
| Workpoint next/action as gap | Uses Workpoint next slice/action when available. | Can inherit vague Workpoint wording and call it trajectory gap. | Validate next slice has verb + target + proof hook before using it. |
| Waypoint `Close active gap: ...` | Generated from active gap. | Repackages vague gap as waypoint; no new information. | Generate waypoint from concrete state delta and proof command. |

## 4. Clarity gate blocks too often or too rarely

### 4.1 Blocks too often

| Pattern | Why it may overblock |
|---|---|
| `next_workpoint` is a missing fact. | A trajectory can be valid before a Workpoint exists; absence should trigger `propose_workpoint`, not necessarily unclear/operator-required. |
| `stale_or_missing_evidence_refs` fires when evidence count is zero and definition is not unclear. | A newly defined goal can be clear enough to create a first Workpoint even before proof exists. |
| `current_verified_state` missing always contributes to unclear/provisional. | For initial planning, missing current state should ask for assessment, not block all trajectory definition. |

### 4.2 Blocks too rarely

| Pattern | Why it underblocks |
|---|---|
| `define_goal` basic validation only checks non-empty `long_term_goal` and `desired_end_state`. | Generic text like `make it better` can pass the first gate. |
| Desired/current equality can produce no active gap without proof validation. | A user can claim current equals desired without evidence or required checks. |
| Workpoint next/action can become active gap without judging specificity. | Vague Workpoint text can make Trajectory seem actionable. |
| `operator_confirmed` defaults true when `goal_source=operator`. | Direct operator source is important, but still should reject generic or self-contradictory goals. |

## 5. define_goal validation weaknesses

| Weakness | Current behavior | Risk |
|---|---|---|
| Non-empty only for core fields | `basic_valid` checks trim-empty goal/end-state only. | Accepts vague, circular, or non-verifiable goals. |
| Supersession check is narrow | Requires confirmation/evidence only when `supersedes_trajectory_id` is present. | A caller can alter goal text without explicit supersession metadata. |
| Current-state gate is soft | Route comments mention current ask/state/supersession evidence, but output can still carry advisory active gap rather than crisp rejection. | Agents may proceed with a weak trajectory. |
| Required checks/evidence are optional | `required_checks` and `required_evidence_refs` can be empty. | Desired end state may have no proof contract. |

## 6. assess output weaknesses

| Weakness | Current behavior | Risk |
|---|---|---|
| Assessment records state delta, then returns view. | The returned view is rich but not centered on the exact next call. | Agents must infer whether to define goal, checkpoint Workpoint, or capture evidence. |
| observed_state can be absent. | The request accepts optional observed state. | Empty assessment can look like an action without improving clarity. |
| Evidence refs are optional. | Assessment may not prove observed state. | Trajectory can update narrative without proof. |

## 7. propose_workpoint wrong-next-step risks

| Weakness | Current behavior | Risk |
|---|---|---|
| Proposal mission is active gap. | If active gap is generic, Workpoint mission is generic. | Proposed Workpoint inherits the weak gap. |
| Candidate is advisory only, but easy to over-trust. | It includes guard text but no hard confidence/specificity score. | Agents may act on an advisory proposal instead of checkpointing a canonical Workpoint. |
| Target/action defaults can be absent or broad. | `target_ref` and `action_type` are optional. | Proposal may lack object and verb. |

## 8. Field-shuttling without strong opinion

| Code/output area | Shuttled fields | Why it matters |
|---|---|---|
| `focus_trajectory_sync` | Focus State current focus and trajectory STG. | It projects alignment but does not decide if focus contradicts trajectory. |
| `relevance_rationale` | Inclusion reasons for project/workpoint/frame/evidence. | It explains sources but does not rank which source wins beyond reconciliation. |
| `context_sufficiency` | Missing facts/conflicts/recommended action. | Useful, but lacks a single command-shaped next step. |
| `durable_lifecycle` | Checkpoints, state deltas, history, DOD, milestones. | Good audit data, but no policy says what lifecycle state permits. |

## 9. Strongest existing parts to preserve

- Project scope safety rejects unsafe roots and mismatches before canonical use.
- `trajectory_workpoint_reconciliation` correctly says canonical Workpoint controls immediate next action.
- Route set is complete: view, define-goal, assess, propose-workpoint, checkpoint, resume.
- Lifecycle metadata exists for source precedence, refresh triggers, checkpoints, state deltas, and goal provenance.

## 10. Phase 3 design implications

The strong Trajectory spec should require:

1. Every gap has `gap_description`, `next_tool`, `next_command`, and `proof_needed`.
2. `define_goal` rejects vague/circular/unverifiable goals with specific recovery hints.
3. `assess` either records an evidence-backed state or blocks with exact missing fields.
4. `propose_workpoint` requires concrete verb + target + proof hook or returns blocked/advisory with repair command.
5. Placeholder fields must either be populated, renamed as not-integrated, or omitted.
6. Clarity gate should distinguish `can_define_goal`, `can_assess`, `can_propose_workpoint`, and `can_execute` instead of one broad proceed/block posture.
