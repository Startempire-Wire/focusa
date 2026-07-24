# Spec 130 — HLT-Aware Compaction Mission Packet and Bloatgaurd Context Firewall

Status: implementation-reopened — original §§0–37 closure preserved; bounded-persistence/forever-session amendment §§38–54 specified 2026-07-13 after real Pi replay OOM; implementation and proof pending
Target file: `docs/130-hlt-aware-compaction-mission-packet-and-bloatgaurd-context-firewall-spec.md`  
Canonical label: Spec 130 Compaction Mission Packet  
Depends on: Spec 88, Spec 96, Spec 98, Spec 100, Spec 101, Spec 104, Spec 111, Spec 112, Spec 115, Spec 116, Spec 119, Spec 125, Spec 129  
Primary implementation surfaces: Focusa core, API, CLI, Pi extension, tool contracts, Context Cognition, Bloatgaurd, Workpoint, Trajectory, Evidence/ECS, Receipts, recent-turn daemon, utility cards, tests, docs

---

## 0. Purpose

Spec 130 defines Focusa’s compaction and bloat-control architecture for long-running agent sessions.

It ensures that compaction:

1. preserves mission cohesion,
2. avoids transcript-tail authority,
3. keeps bulky context out of hot prompts,
4. carries Workpoint and Trajectory through compression,
5. exposes HLT posture loudly,
6. prevents generic or missing HLT from becoming route authority,
7. keeps raw evidence rehydratable,
8. preserves active blockers,
9. supports provider prompt caching,
10. makes post-compaction recovery auditable.

This spec does **not** create a new authority surface. It defines a bounded prompt-facing projection of existing authority surfaces.

---

## 1. Core invariant

```text
Compaction must preserve mission cohesion without inventing mission authority.

A compacted Focusa session is recoverable only when the resumed agent receives:

1. verified scope,
2. HLT posture,
3. TrajectoryResumePacketV3,
4. WorkpointResumePacketV2,
5. AttentionRecallVerdict,
6. Evidence / ECS / Receipt expectations,
7. exact next tool,
8. do-not-use boundaries,
9. omitted context handles,
10. rehydrate routes.

If HLT is missing, generic, stale, conflicted, or fallback-mismatched,
the post-compaction packet must warn loudly before ordinary route guidance.
```

---

## 2. Authority hierarchy

Compaction packets are bounded projections only.

They must not override, infer, promote, or mutate canonical authority.

```text
Scope lives in typed ProjectIdentity / HostIdentity / ScopeRef.
Direction lives in Trajectory.
HLT is required.
Meaning lives in Focus State.
Immediate continuation lives in Workpoint.
Permission to mutate lives in Context Authority.
Proof lives in Evidence Refs.
Completion truth lives in Focusa Receipts.
Provider closure truth lives in Closure Authority.
Provider task display lives in adapters.
Ontology gives every surface shared semantic structure.
Operator steering wins.
```

### 2.1 Compaction authority rule

```text
Compaction packets are not authority.
Compaction summaries are not authority.
Provider summaries are not authority.
Transcript tails are not authority.
Pi local shadow is not authority.
Generic bootstrap text is not authority.

Compaction may only render authority posture from existing Focusa surfaces.
```

---

## 3. Relationship to Spec 101 Bloatgaurd

Spec 101 Bloatgaurd owns bloat-control policy.

Spec 130 uses Bloatgaurd for:

- output firewall,
- tool-call compression,
- prompt/context diet,
- tokenbloat control,
- stable-prefix layout,
- tool-call history elision,
- recent-turn slices,
- semantic cache,
- context handles,
- cold rehydration,
- duplicate-block dedupe,
- Bloatgaurd profiles and routines.

### 3.1 Bloatgaurd invariant imported

```text
Rich context lives in indexed handles, evidence stores, compact packets,
and rehydrate refs — not in hot prompts, raw transcript tails,
duplicated docs, proof logs, shell sprawl, or verbose tool-output dumps.
```

### 3.2 Bloatgaurd domains extended by this spec

Spec 130 adds or tightens these Bloatgaurd findings:

```text
raw_tool_history_in_hot_prompt
tool_summary_missing_evidence_ref
stable_prefix_churn
recent_turns_missing_evidence_refs
workpoint_resume_json_over_budget
trajectory_resume_json_over_budget
provider_cache_breakpoint_missing
uncapped_focus_slice_section
full_payload_used_by_default
active_blocker_elided_without_exact_error
generic_hlt_rendered_as_authority
missing_hlt_silently_suppressed
last_trajectory_clarity_backfilled_without_provenance
receipt_completion_claim_without_hlt_posture
compaction_summary_used_as_canonical_source
```

---

## 4. Relationship to Spec 125

Spec 125 governs mandatory Trajectory, non-lazy HLT, Pi bootstrap, receipts, and ontology interlock.

Spec 130 imports these rules:

```text
Never invent the north star.
Never hide that the north star is missing.
Never call generic text a trajectory.
Never treat fallback as fresh session truth.
Always query previous valid trajectory before asking for a new HLT.
Always show the HLT status before durable work.
Always make Pi carry Trajectory through bootstrap and compaction.
Always prove completion with Evidence and Receipts.
```

### 4.1 HLT status is mandatory in compaction

Every compaction packet must classify HLT as one of:

```text
canonical_explicit
previous_valid_fallback
supersession_pending
missing_required
generic_degraded
conflicted
```

Only these count as route-action-ready:

```text
canonical_explicit
previous_valid_fallback
```

Even `previous_valid_fallback` has limited authority:

```text
HLT may be reused as the prior valid project north star.
MLG / STG / Waypoints / current_state must be refreshed
for the active continuity/session before durable action.
```

### 4.2 Missing or generic HLT does not block compaction

If HLT is missing or generic:

```text
Compaction still proceeds.
Workpoint must still be preserved.
TrajectoryResumePacketV3 must include loud warning.
Post-compaction message must tell the model to restore/define HLT
before durable multi-step work.
```

### 4.3 Generic trajectory is not fallback

Fallback trajectory means:

```text
the most recent previous explicitly set valid trajectory
for the same verified scope family.
```

Fallback trajectory does not mean:

```text
generic bootstrap text
project name default
current task title
transcript summary
local Pi shadow
foreign project trajectory
unverified stale continuity packet
```

### 4.4 Spec 131 temporal preservation and refresh

Compaction consumes but does not own Spec 131 temporal authority. A CompactionMissionPacket and resume projection preserve bounded refs for HumanCalendarContext, TimeAwarenessPacket, TemporalPriorityFrame, civil/fixed deadline and readiness-target revisions, boundary/uncertainty posture, deadline conflicts, TemporalExecutionGuards, material progress, no-progress/lost-time incidents, temporal claims/forecast invalidations, cancellation/reconciliation, and factual completion/operator disposition.

Rules:

- compaction, model switch, handoff, resume, and rehydration never move an external deadline or reset elapsed/breach history;
- per-boot monotonic segments remain per-boot; transcript timestamps or packet generation time cannot bridge reboot as exact elapsed time;
- the pre-compaction frame becomes stale at the compaction boundary and MUST be refreshed before new durable/consequential action;
- a still-valid deterministic TemporalExecutionGuard may be observed/reconciled but cannot be reissued or extended by compaction;
- raw private calendar content remains behind authorized refs; the hot packet carries only the bounded projection/context hash needed for temporal decisions;
- degraded/missing temporal context blocks verified closure and new consequential dispatch but preserves bounded repair, cleanup, cancellation, reconciliation, evidence, and truthful operator notification;
- provider/agent completion and operator disposition cannot replace Spec 131 verified completion.

Canonical source: `docs/131-focusa-workpoint-item-timing-velocity-and-closure-authority-spec.md`.

---

## 5. Non-lazy inference rule

```text
Trajectory may be intelligently inferred as a candidate.
Trajectory must never be lazily inferred as authority.
```

Intelligent inference must be:

```text
scope-locked
current-ask aware
evidence-aware
previous-HLT aware
provenance-rich
explicitly labeled
operator-confirmable
noncanonical until promotion
```

Lazy inference is forbidden.

Lazy inference includes:

```text
silently filling HLT from project name
silently filling HLT from Workpoint/current_focus
silently reusing stale ladder lines
silently using transcript tail as route context
silently carrying old session clarity into a new session
silently treating generic bootstrap text as HLT
silently setting operator_confirmed=true for an inferred draft
```

### 5.1 Compaction-specific inference rule

```text
Compaction may summarize HLT posture.
Compaction may not invent HLT.
Compaction may not promote inferred HLT.
Compaction may only render candidate/fallback/canonical status.
```

---

## 6. Durable work definition

Spec 130 defines durable work as any action that creates or changes durable project state.

Durable work includes:

```text
writing code
editing config
changing deployment files
deploying
running production-affecting scripts
modifying database/schema
filing closure receipts
closing Workpoint / bead / provider work item
claiming completion
creating public proof
performing risky mutation
updating HLT / Trajectory
updating canonical Workpoint state
changing Context Authority
```

Simple read-only Q&A and triage may proceed with warnings.

Durable work requires one of:

```text
HLT_STATUS=canonical_explicit
HLT_STATUS=previous_valid_fallback with refreshed session-specific state
explicit degraded-mode receipt posture
operator override with recorded reason where allowed
```

Generic HLT can never be made canonical through override alone.

---

## 7. Compaction state machine

Spec 130 requires a formal state machine.

### 7.1 States

```text
session_start
session_switch
before_compaction
manual_compaction
auto_compaction
hard_context_pressure
provider_overflow
micro_compaction
after_compaction
model_switch
fork
handoff
resume_verified
resume_degraded
resume_blocked
completion_claim
```

### 7.2 State transition contract

Every transition must declare:

```text
required inputs
required packets
allowed fallbacks
warnings
exact next tool
forbidden actions
receipt expectation
omitted context policy
rehydrate policy
telemetry events
```

### 7.3 Resume states

#### `resume_verified`

Allowed when:

```text
scope verified
HLT canonical_explicit OR valid previous_valid_fallback
Workpoint resume packet scoped to current project_root + continuity_id
TrajectoryResumePacketV3 present
Evidence refs present or explicit missing-evidence warning rendered
no Workpoint / Trajectory conflict
```

#### `resume_degraded`

Required when:

```text
HLT missing_required
HLT generic_degraded
HLT previous_valid_fallback with session/continuity mismatch
Trajectory stale
Workpoint missing
Evidence partial
current_state missing
```

Allowed actions:

```text
read-only triage
focusa_hlt_history
focusa_trajectory_assess
focusa_trajectory_define_goal
focusa_workpoint_resume
focusa_tool_doctor
focusa_traverse bounded reads
```

Forbidden actions:

```text
durable work
closure claims
provider task closure
public proof snapshot
risky mutation
canonical HLT mutation without verified gate
```

#### `resume_blocked`

Required when:

```text
scope unsafe
project_root mismatch
trajectory scope conflict
Workpoint / Trajectory disagreement cannot be reconciled
Context Authority denies mutation
provider closure truth is inconsistent
active blocker lacks exact evidence or rehydrate ref
```

Allowed actions:

```text
focusa_project_identity
focusa_context_authority_check
focusa_hlt_history
focusa_trajectory_view
focusa_workpoint_resume
focusa_tool_doctor
operator clarification
```

---

## 8. Compaction Mission Packet

The Compaction Mission Packet is the compact prompt-facing render used at bootstrap, compaction, model switch, fork/handoff, and recall-intent recovery.

It is not canonical state.

### 8.1 Schema

```json
{
  "schema_version": "focusa.compaction_mission_packet.v1",
  "packet_id": "uuid",
  "generated_at": "iso8601",
  "resume_source": "session_start|session_switch|before_compaction|after_compaction|model_switch|fork|handoff|manual|provider_overflow",
  "status": "verified|degraded|blocked",
  "canonical": false,
  "advisory": true,
  "scope": {
    "scope_kind": "project|host",
    "project_root": "/path",
    "host_scope_id": null,
    "continuity_id": "focusa-cont-...",
    "session_id": "pi-session-...",
    "scope_status": "verified|unsafe|mismatch|missing"
  },
  "current_ask": {
    "text": "...",
    "ask_kind": "question|instruction|correction|meta|unknown",
    "source_turn_id": "pi-turn-..."
  },
  "trajectory": {
    "packet_ref": "trajectory_resume_packet_v3:...",
    "hlt": "...",
    "hlt_status": "canonical_explicit|previous_valid_fallback|supersession_pending|missing_required|generic_degraded|conflicted",
    "hlt_required": true,
    "hlt_source": "operator|trajectory_define_goal|previous_valid_trajectory|generic_bootstrap|none",
    "generic_bootstrap": false,
    "fallback": "none|previous_valid_trajectory",
    "fallback_level": "same_session|same_continuity_any_session|same_project_any_continuity|none",
    "action_authority_from_trajectory": true,
    "desired_end_state": "...",
    "current_verified_state": "...",
    "active_gap": "...",
    "warnings": []
  },
  "workpoint": {
    "packet_ref": "workpoint_resume_packet_v2:...",
    "workpoint_id": "...",
    "mission": "...",
    "next_slice": "...",
    "action_authority": true,
    "status": "ready|missing|stale|conflict"
  },
  "focus_state": {
    "intent": "...",
    "current_focus": "...",
    "decisions": [],
    "constraints": [],
    "failures": [],
    "recent_results": []
  },
  "active_blocker": {
    "present": false,
    "error_class": null,
    "test_name": null,
    "file_path": null,
    "line_range": null,
    "exact_blocker_excerpt": null,
    "rehydrate_ref": null
  },
  "evidence": {
    "evidence_refs": [],
    "ecs_handles": [],
    "receipt_refs": [],
    "proof_refs": [],
    "missing_evidence_warning": null
  },
  "bloatgaurd": {
    "omitted_sections": [],
    "omitted_bytes": 0,
    "omitted_tokens": 0,
    "rehydrate_refs": [],
    "full_payload_policy": "cold_opt_in",
    "tool_history_elided": true
  },
  "recent_turns": {
    "count": 0,
    "refs": []
  },
  "next": {
    "exact_next_tool": "focusa_workpoint_resume",
    "allowed_next_tools": [],
    "do_not_use": []
  },
  "receipt_expectation": {
    "required_before_completion": true,
    "trajectory_hlt_status_required": true,
    "evidence_required": true,
    "closure_authority_required": true
  }
}
```

### 8.2 Prompt render format

The compact prompt render must begin with authority and warning state, not prose.

```text
## CompactionMissionPacket
STATUS: verified|degraded|blocked
SCOPE_STATUS: verified|unsafe|mismatch|missing
PROJECT_ROOT: <...>
CONTINUITY_ID: <...>
SESSION_ID: <...>

HLT_STATUS: canonical_explicit|previous_valid_fallback|missing_required|generic_degraded|conflicted
HLT_REQUIRED: true
GENERIC_BOOTSTRAP: true|false
FALLBACK_SOURCE: previous_valid_trajectory|none
ACTION_AUTHORITY_FROM_TRAJECTORY: true|false
LOUD_WARNING: <message or none>

WORKPOINT_STATUS: ready|missing|stale|conflict
WORKPOINT_NEXT_SLICE: <...>

EXACT_NEXT_TOOL: <tool>
DO_NOT_USE: transcript_tail, generic_hlt_as_authority, stale_session_trajectory, full_payload_without_rehydrate_ref
REHYDRATE_REFS: <refs>
RECEIPT_EXPECTATION: required_before_completion
```

---

## 9. TrajectoryResumePacketV3

Spec 130 requires TrajectoryResumePacketV3 in all post-compaction auto-resume messages.

### 9.1 Required schema

```json
{
  "schema_version": "focusa.trajectory_resume_packet.v3",
  "packet_id": "uuid",
  "generated_at": "iso8601",
  "resume_source": "session_start|session_switch|before_compaction|after_compaction|model_switch|fork|manual",
  "canonical": false,
  "degraded": true,
  "scope": {
    "scope_kind": "project|host",
    "project_root": "/path",
    "continuity_id": "focusa-cont-...",
    "session_id": "pi-session-..."
  },
  "hlt": {
    "value": null,
    "status": "canonical_explicit|previous_valid_fallback|missing_required|generic_degraded|conflicted",
    "source": "operator|trajectory_define_goal|previous_valid_trajectory|generic_bootstrap|none",
    "generic_bootstrap": false,
    "required": true,
    "loud_warning_required": true
  },
  "fallback": {
    "used": false,
    "source": "previous_valid_trajectory|none",
    "level": "same_session|same_continuity_any_session|same_project_any_continuity|none",
    "source_session_id": null,
    "source_continuity_id": null,
    "source_trajectory_id": null
  },
  "ladder": {
    "mlg": null,
    "stg": null,
    "waypoints": []
  },
  "state": {
    "desired_end_state": null,
    "current_verified_state": null,
    "active_gap": "missing_verified_state",
    "evidence_refs": []
  },
  "authority": {
    "trajectory_route_authority": false,
    "workpoint_remains_immediate_authority": true,
    "operator_steering_wins": true
  },
  "warnings": [],
  "next_tools": [
    "focusa_hlt_history",
    "focusa_trajectory_view",
    "focusa_trajectory_define_goal",
    "focusa_workpoint_resume"
  ]
}
```

### 9.2 Required prompt first lines

The first visible lines of the Trajectory section must be:

```text
## TrajectoryResumePacketV3
HLT_STATUS: <...>
HLT_REQUIRED: true
GENERIC_BOOTSTRAP: true|false
FALLBACK_SOURCE: previous_valid_trajectory|none
CANONICAL: true|false
DEGRADED: true|false
LOUD_WARNING: <message or none>
NEXT_REQUIRED_TOOL: <tool>
```

### 9.3 Loud warning classes

```text
HLT_REQUIRED
GENERIC_HLT_DEGRADED
PREVIOUS_TRAJECTORY_FALLBACK
TRAJECTORY_SESSION_MISMATCH
TRAJECTORY_CONTINUITY_MISMATCH
TRAJECTORY_SCOPE_CONFLICT
TRAJECTORY_WORKPOINT_DISAGREEMENT
HLT_SUPERSESSION_REQUIRED
```

---

## 10. Pre-compaction pipeline

Before any manual, automatic, hard-pressure, or provider-overflow compaction, Pi must attempt:

```text
1. Focus State delta push
2. Workpoint checkpoint
3. Trajectory checkpoint
4. HLT history query for exact session
5. Trajectory resume
6. Workpoint resume
7. Persist authoritative state
8. Build CompactionMissionPacket
9. Store omitted raw material in ECS/Evidence handles
10. Verify active blocker preservation
```

### 10.1 Required failure handling

If Focusa is unavailable:

```text
record local non-canonical Workpoint fallback
record local non-canonical compaction packet
mark canonical=false
mark degraded=true
render loud warning
never treat local fallback as HLT authority
```

### 10.2 HLT missing/generic during pre-compaction

If HLT is missing or generic:

```text
compaction proceeds
Workpoint is preserved
TrajectoryResumePacketV3 hlt.status is missing_required or generic_degraded
post-compaction exact_next_tool is focusa_hlt_history
if no previous valid HLT exists, next tool becomes focusa_trajectory_define_goal
durable work remains blocked until HLT posture is resolved or degraded receipt posture is explicit
```

---

## 11. Post-compaction auto-resume contract

The post-compaction auto-resume message must include, in this order:

```text
1. Compaction status line
2. Loud Trajectory warning if any
3. AttentionRecallVerdict
4. TrajectoryResumePacketV3
5. WorkpointResumePacketV2
6. CompactionMissionPacket
7. Exact next tool
8. Do-not-use list
9. Evidence / Receipt expectations
10. Rehydrate refs
11. End-of-task reporting contract
```

### 11.1 Required message skeleton

```text
# Compaction Complete

## Trajectory Warning
<warning block or "none">

## AttentionRecallVerdict
<bounded attention recall packet>

## TrajectoryResumePacketV3
HLT_STATUS: <...>
HLT_REQUIRED: true
GENERIC_BOOTSTRAP: <...>
FALLBACK_SOURCE: <...>
CANONICAL: <...>
DEGRADED: <...>
LOUD_WARNING: <...>
NEXT_REQUIRED_TOOL: <...>

## WorkpointResumePacketV2
<bounded workpoint packet>

## CompactionMissionPacket
<bounded mission packet>

## Directive
<status-specific directive>

## Do Not Use
- transcript tail as authority
- generic HLT as trajectory authority
- stale lastTrajectoryClarity as HLT authority
- raw tool logs unless rehydrated by explicit handle
- full lineage / ontology / telemetry by default

## Receipt / Evidence Expectations
- completion requires receipt posture
- HLT status must appear in relevant receipts
- evidence refs must support completion claim
- Closure Authority must validate provider closure truth where relevant

## Rehydrate Routes
- focusa_traverse
- focusa_workpoint_resume
- focusa_trajectory_view
- focusa_hlt_history
- focusa_evidence_get
- focusa_ecs_rehydrate
```

---

## 12. Tool-output elision and structured rehydration

Historical tool calls must not remain in the model-visible prompt as raw transcripts after checkpoint.

Core transform:

```text
ToolCallHistory → ToolRunSummary + EvidenceRef + RehydrateRef + Failure/Decision/Constraint/Workpoint links
```

### 12.1 ToolRunSummary schema

```json
{
  "schema_version": "focusa.tool_run_summary.v1",
  "tool_name": "...",
  "target": "...",
  "action_type": "read|edit|test|search|diagnostics|proof|failure",
  "result": "pass|fail|found|changed|no-op|blocked",
  "summary": "...",
  "changed_files": [],
  "error_class": null,
  "test_name": null,
  "evidence_refs": [],
  "rehydrate_ref": "ecs:text:...",
  "omitted_bytes": 0,
  "omitted_tokens": 0,
  "active_blocker": false,
  "linked_workpoint_id": null,
  "linked_decisions": [],
  "linked_constraints": [],
  "linked_failures": []
}
```

### 12.2 Prompt-visible fields

Prompt-visible tool summaries may include only:

```text
tool or route name
target object/path/endpoint
action type
compact result
exact evidence handle
omitted byte/token count
rehydrate route
linked decision/constraint/failure/workpoint when relevant
exact active blocker lines when blocking current work
```

### 12.3 Raw preview policy

Default:

```text
No raw shell logs in hot prompt.
No full file reads in hot prompt.
No long test output in hot prompt.
No duplicate proof logs in hot prompt.
No raw transcript tail in hot prompt.
```

Allowed exception:

```text
Active blocker may include exact error class, test name, file path,
and minimal exact line excerpt required for the next action.
```

### 12.4 ECS replacement policy

When output exceeds configured threshold:

```text
store raw output in ECS/Evidence
replace model-visible content with ToolRunSummary
include rehydrate_ref
include omitted_bytes and omitted_tokens
include exact blocker excerpt only when active blocker
```

Default thresholds:

```text
externalizeThresholdBytes = 8192
externalizeThresholdTokens = 800
toolOutputFloodWindowMs = 120000
toolOutputFloodResultThreshold = 4
toolOutputFloodBytesThreshold = 24000
toolOutputFloodTokensThreshold = 4000
toolOutputFloodLargeResultBytes = 8192
toolOutputFloodLargeResultThreshold = 2
```

---

## 13. Recent-turn slice contract

Recent turns are orientation hints, not authority.

### 13.1 Required schema

```json
{
  "schema": "focusa.recent_turns.v1",
  "turn_id": "pi-turn-...",
  "continuity_id": "focusa-cont-...",
  "mission_at_turn": "...",
  "outcome": "committed|filed_bead|observed|blocked|ack|tooled",
  "evidence_refs": [],
  "tool_call_count": 0,
  "emitted_at": 0
}
```

### 13.2 Hard rules

```text
default emitted count = 4
hard cap = 8
drop status-only turns
drop tool-empty ack turns
never include assistant prose
never include raw tool output
include evidence_refs when evidence exists
do not emit until enough meaningful turns exist
idempotency guard prevents duplicate emission per turn
```

### 13.3 Recall-intent trigger

Recall intent must force recent-turn slice re-emission.

Trigger categories:

```text
direct recall
implicit prior
coherence loss
repetition
operator steering
```

If recent-turn ring is empty:

```text
surface focusa_lineage_tree
surface focusa_awareness_packet
surface focusa_workpoint_resume
surface focusa_trajectory_view
```

---

## 14. Stable prefix and provider prompt cache

Spec 130 requires stable-prefix + dynamic-slice prompt layout.

### 14.1 Stable prefix

Stable prefix may contain:

```text
provider/system/developer policy
Focusa cognitive rules
deterministic tool contract summaries
verified project identity summary when stable
Workpoint / Trajectory authority law
Bloatgaurd safety boundaries
canonical instructions without timestamps or random IDs
```

Stable prefix must not contain:

```text
current ask
tool output
timestamps
random IDs
per-turn counters
raw diagnostics
ECS raw payloads
latest report snippets
current blocker lines
recent-turn slices
```

### 14.2 Dynamic slice

Dynamic slice may contain:

```text
current ask
CompactionMissionPacket
AttentionRecallVerdict
TrajectoryResumePacketV3
WorkpointResumePacketV2
active blocker
visible recap requirement
recent turns
evidence handles
omitted counts
rehydrate refs
exact next tool
receipt expectation
```

### 14.3 Cache telemetry

Every provider call should record when available:

```text
stable_prefix_hash
stable_prefix_bytes
stable_prefix_token_estimate
dynamic_slice_bytes
dynamic_slice_token_estimate
cache_breakpoint_count
cache_read_tokens
cache_write_tokens
cache_hit_rate
cache_miss_reason
dynamic_content_before_breakpoint
provider_cache_hint_emitted
provider_cache_hint_supported
```

### 14.4 Finding IDs

```text
stable_prefix_churn
dynamic_content_before_cache_breakpoint
provider_cache_breakpoint_missing
provider_cache_skipped_in_safe_auto
cache_hint_unsupported_without_finding
recent_turns_in_stable_prefix
tool_output_in_stable_prefix
```

---

## 15. Focus Slice pressure policy

The Pi Focus Slice remains bounded.

Default budget:

```text
maxTokens = min(max(floor(headroom * 0.15), 200), 1500)
```

Pressure modes:

```text
normal: up to 1500 tokens
pressure: 700-900 tokens
critical: 400-600 tokens
blocked: only warning + exact next tool + rehydrate refs
```

### 15.1 Pressure mode content priority

Under pressure, preserve in order:

```text
1. current operator ask
2. HLT warning / HLT status
3. Workpoint next action
4. active blocker exact line
5. constraints that affect current action
6. evidence refs
7. receipt expectations
8. rehydrate refs
9. omitted context receipt
```

Drop or handle-only:

```text
semantic memory
ontology graph
historical context
decayed context
proof logs
raw docs
long Workpoint JSON
long Trajectory JSON
recent result prose
tool output previews
```

---

## 16. Context Cognition / Context Compiler

Spec 130 requires Context Cognition for broad reads and context selection.

### 16.1 Context Compiler preference order

Prefer:

```text
symbol map
route map
tool contract map
proof bundle map
active object map
selected snippets
evidence handles
codemap summaries
```

Avoid by default:

```text
whole repo
whole files
whole docs trees
raw grep dumps
raw logs
raw transcript tail
full lineage tree
full ontology graph
deep work-loop status
```

### 16.2 Required packet labels

Every Context Cognition packet used in compaction must include:

```text
status
scope_status
canonical=false unless promoted elsewhere
advisory=true
stale
degraded
source_refs
evidence_refs
next_tools
do_not_drift
side_effects
misuse_hint
selected_context
excluded_context
omitted counts
rehydrate refs
```

---

## 17. Subagent isolation

Spec 130 requires noisy work isolation.

Use subagents/fresh-context workers for:

```text
broad repo exploration
large log review
test-output triage
dependency research
multi-file audits
dead-code scans
duplicate scans
web/repo research
large proof runs
```

### 17.1 Subagent result schema

```json
{
  "schema": "focusa.subagent_result.v1",
  "task": "...",
  "scope": {
    "project_root": "...",
    "continuity_id": "...",
    "session_id": "..."
  },
  "summary": "...",
  "inspected_refs": [],
  "evidence_refs": [],
  "changed_files": [],
  "active_blockers": [],
  "confidence": "low|medium|high",
  "omitted_raw_refs": [],
  "recommended_next": "...",
  "must_not_infer": [],
  "rehydrate_refs": []
}
```

### 17.2 Subagent return policy

Subagents must return:

```text
bounded summary
evidence refs
exact blockers
confidence
recommended next action
omitted raw refs
```

Subagents must not return:

```text
raw logs
full grep dumps
full file contents
unbounded prose history
authority claims without evidence
```

---

## 18. Receipt / closure interlock

Compaction must preserve receipt expectations.

### 18.1 Required receipt trajectory frame

Relevant receipts must include:

```json
{
  "trajectory": {
    "hlt": "...",
    "hlt_status": "canonical_explicit|previous_valid_fallback|missing_required|generic_degraded|conflicted",
    "hlt_required": true,
    "generic_bootstrap": false,
    "fallback": {},
    "mlg": "...",
    "stg": "...",
    "waypoints": [],
    "active_gap": "...",
    "posture": "canonical|advisory|degraded|blocked|stale"
  }
}
```

### 18.2 Completion claim gate

A final report, work session, work item closure, install verification, risky mutation, or public proof snapshot must not claim full completion if:

```text
HLT is missing_required
HLT is generic_degraded
Trajectory and Workpoint conflict
current_state is missing and not explicitly accepted
evidence is partial/surrogate/missing
active blocker remains unresolved
Closure Authority has not validated provider closure truth where relevant
```

### 18.3 Post-compaction final report rule

```text
No post-compaction final answer may claim completion unless:

1. Workpoint status is reconciled,
2. HLT posture is included,
3. evidence refs exist,
4. receipt status permits completion,
5. active blocker is none or explicitly unresolved,
6. Closure Authority permits closure where provider task display is involved.
```

---

## 19. `lastTrajectoryClarity` provenance rule

`lastTrajectoryClarity` is dangerous unless provenance-backed.

### 19.1 Required fields

`lastTrajectoryClarity` must carry:

```text
project_root
continuity_id
session_id
trajectory_id
hlt_status
hlt_source
fallback.used
fallback.level
fallback.source_trajectory_id
fallback.source_session_id
fallback.source_continuity_id
fallback_source_timestamp
generic_bootstrap
canonical
degraded
provenance_ref
evidence_refs
```

### 19.2 Forbidden behavior

Pi must not backfill HLT from:

```text
unproven local memory
generic default
stale packet
mismatched session
mismatched continuity
transcript tail
Workpoint mission alone
Focus State current_focus alone
project folder name
package name
```

### 19.3 Allowed behavior

Pi may backfill HLT only from:

```text
validated previous_valid_trajectory record
matching project scope family
visible fallback metadata
loud warning unless same exact session + continuity
```

---

## 20. Security and privacy redaction

Compaction intersects with security because logs and tool output often contain secrets.

### 20.1 Never prompt-visible

The following must never be injected into hot prompt:

```text
API keys
tokens
session cookies
auth headers
private keys
.env contents
database passwords
OAuth secrets
webhook secrets
cloud credentials
raw production customer data
private operator knowledge marked private
```

### 20.2 Restricted handles

If a tool output contains suspected secrets:

```text
store as restricted ECS handle
redact prompt-visible summary
emit secret_redaction_applied finding
require explicit secure rehydrate path
do not image
do not include preview
```

### 20.3 Finding IDs

```text
secret_in_tool_output
secret_preview_attempted
restricted_handle_missing
raw_env_prompt_visible
auth_header_prompt_visible
private_operator_knowledge_leak
```

---

## 21. Agent instruction diet

Spec 130 adds an Agent Instruction Diet domain.

Covered files/surfaces:

```text
AGENTS.md
CLAUDE.md
GEMINI.md
CONTEXT.md
.cursorrules
.pi/settings.json prompts
Focusa tool docs
adapter READMEs
agent internal docs
```

### 21.1 Findings

```text
oversized_agent_instruction_file
conflicting_agent_instruction
duplicated_agent_rule
tool_catalog_instruction_bloat
stale_agent_instruction
unscoped_instruction_loaded_globally
private_operator_knowledge_in_public_instruction
instruction_file_without_scope_header
```

### 21.2 Requirements

Agent instruction files should:

```text
be scoped
be concise
avoid duplicated tool catalogs
link to canonical docs instead of pasting long specs
separate public repo instructions from private operator knowledge
include last-reviewed metadata
avoid carrying stale launch/implementation history
```

---

## 22. Anti-recursive summary bloat

Compaction summaries must not recursively summarize prior compaction summaries.

### 22.1 Rule

```text
Never summarize a prior compaction summary as prose.

Recompose every compaction packet from canonical sources:

- ProjectIdentity / HostIdentity / ScopeRef
- TrajectoryResumePacketV3
- WorkpointResumePacketV2
- Focus State
- Evidence / ECS handles
- Receipts
- RecentTurnSlice
- Context Cognition packet
- Bloatgaurd report
```

### 22.2 Finding IDs

```text
recursive_compaction_summary
summary_of_summary_detected
canonical_source_missing
transcript_tail_used_for_compaction
provider_summary_used_as_authority
```

---

## 23. Reversible checkpoint and replay

Compaction must be auditable and partially replayable.

### 23.1 Required commands

```bash
focusa compaction inspect --packet-id <id>
focusa compaction diff --before <id> --after <id>
focusa compaction replay --packet-id <id>
focusa compaction restore-context --handle <id>
focusa compaction why --packet-id <id>
```

### 23.2 Inspect must answer

```text
What did we keep?
What did we omit?
Why was it safe to omit?
Where is the exact raw evidence?
What authority surface supports the next action?
What HLT posture governs durable work?
What is the exact next tool?
What receipt/evidence expectation blocks completion?
```

---

## 24. API requirements

### 24.1 New or extended routes

```http
GET  /v1/compaction/packet/{packet_id}
POST /v1/compaction/build
POST /v1/compaction/evaluate
GET  /v1/compaction/inspect/{packet_id}
POST /v1/compaction/replay
POST /v1/compaction/diff
```

### 24.2 Required route behavior

`POST /v1/compaction/build` must:

```text
verify scope
query HLT history by session
fetch TrajectoryResumePacketV3
fetch WorkpointResumePacketV2
fetch Focus State
fetch recent turns
fetch relevant evidence handles
build omitted context receipt
classify HLT posture
classify resume state
return CompactionMissionPacket
```

### 24.3 `/v1/trajectory/resume`

Must return TrajectoryResumePacketV3 and:

```text
check current ask scope conflict
query previous valid trajectory fallback before generic bootstrap
include loud warning state
include Workpoint reconciliation hint
never use generic as fallback
```

### 24.4 `/v1/hlt/history`

Must support:

```text
session_id filter
include_cross_session_fallbacks flag
include_generic flag
fallback candidate computation
latest_valid_for_session
latest_valid_for_continuity
latest_valid_for_project
```

---

## 25. CLI requirements

### 25.1 New commands

```bash
focusa compaction inspect --packet-id <id>
focusa compaction why --packet-id <id>
focusa compaction replay --packet-id <id>
focusa compaction diff --before <id> --after <id>
focusa compaction restore-context --handle <id>
focusa hlt fallback --project-root <path> --continuity-id <id> --session-id current
focusa hlt sessions --project-root <path>
focusa hlt posture --project-root <path> --session-id current
focusa bloat where
focusa context why
```

### 25.2 CLI output requirements

Human output must show:

```text
scope
continuity
session
HLT status
fallback status
Workpoint status
resume state
exact next tool
warnings
evidence refs
omitted counts
rehydrate refs
receipt expectation
```

### 25.3 Loud warning format

```text
╔══════════════════════════════════════════════════════════════╗
║  FOCUSA TRAJECTORY WARNING                                  ║
╠══════════════════════════════════════════════════════════════╣
║  HLT is missing/generic/fallback/stale/conflicted.           ║
║  Do not treat this Trajectory as canonical route authority.  ║
║  Next: focusa hlt history --session-id current               ║
║        focusa trajectory define-goal ...                     ║
╚══════════════════════════════════════════════════════════════╝
```

---

## 26. Pi implementation requirements

### 26.1 Files likely requiring changes

```text
apps/pi-extension/src/session.ts
apps/pi-extension/src/compaction.ts
apps/pi-extension/src/turns.ts
apps/pi-extension/src/awareness.ts
apps/pi-extension/src/state.ts
apps/pi-extension/src/tool-contracts.ts
apps/pi-extension/src/config.ts
docs/current/FOCUSA_AGENT_UTILITY_CARD.md
docs/focusa-tools/tools/focusa_trajectory_view.md
docs/focusa-tools/tools/focusa_trajectory_define_goal.md
docs/focusa-tools/tools/focusa_trajectory_resume.md
docs/focusa-tools/tools/focusa_hlt_history.md
```

### 26.2 `session.ts`

Required changes:

```text
run bootstrap order from Spec 125
query previous-valid HLT before prompt
add session_id to HLT history query
do not suppress HLT prompt unless previous fallback is valid and warning is rendered
replace generic trajectoryDraftOptions with candidate scaffolds
require explicit operator edit/confirm for candidate HLT
require current_state/evidence or explicit override reason for define-goal
emit high-priority warning when HLT is missing/generic/fallback
```

### 26.3 `compaction.ts`

Required changes:

```text
pre-compaction must query HLT history by session
TrajectoryResumePacket must be V3
formatTrajectoryPacketForPrompt must include HLT_STATUS, HLT_REQUIRED, GENERIC_BOOTSTRAP, FALLBACK_SOURCE, LOUD_WARNING
do not backfill HLT from lastTrajectoryClarity unless provenance proves previous_valid_trajectory
post-compaction steer message must put trajectory warning above ordinary route guidance
build and inject CompactionMissionPacket
mask tool output previews under pressure
preserve active blocker exact lines
include receipt/evidence expectations
```

### 26.4 `turns.ts`

Required changes:

```text
recent-turn slice must include evidence_refs when evidence exists
do not emit recent turns too early
do not include assistant prose or raw tool output
capture ToolRunSummary handles from tool_result
record provider cache telemetry when usage exposes it
emit compaction fidelity telemetry
detect completion claims after compaction and check receipt posture
```

### 26.5 `awareness.ts`

Required changes:

```text
Utility Card must distinguish canonical HLT, previous-valid fallback, generic degraded, missing required, and conflicted
MISSION_PACKET must expose HLT status
NOW_CARD must expose exact next action
WHY_CARD must explain excluded generic/bootstrap/transcript authority
HEALTH_CARD must show hlt posture and fallback warning
DO_CARD must route to hlt_history/define_goal/assess depending on HLT status
RECONCILIATION_ENVELOPE must list trajectory warning as blocked/stale surface
```

### 26.6 `state.ts`

Required changes:

```text
lastTrajectoryClarity must carry provenance fields
lastTrajectoryClarity must not be canonical unless scope/session/provenance match
store previous_valid_trajectory fallback metadata separately from current exact-session clarity
reset session-scoped trajectory state on session boundary unless fallback is explicitly revalidated
recentTurns must preserve evidence_refs
store last CompactionMissionPacket handle
store cache telemetry counters
```

### 26.7 `tool-contracts.ts`

Required changes:

```text
add focusa_hlt_history as trajectory family tool contract if missing
mark focusa_trajectory_define_goal as canonical mutation only when verified gate passes
update focusa_trajectory_view purpose to say HLT is required
update focusa_trajectory_resume purpose to mention loud warning and previous-valid fallback
add focusa_compaction_inspect / focusa_compaction_why if exposed to Pi
```

---

## 27. Provider-neutral adapter contract

Spec 130 defines one adapter contract for Pi, Claude Code, Codex, Gemini CLI, Cursor, Cline/Roo, Aider, and future adapters.

### 27.1 Adapter responsibilities

```text
capture_turn()
capture_tool_result()
record_tool_summary()
externalize_raw_output()
inject_compaction_packet()
inject_recent_turns()
emit_cache_split()
mark_visible_recap()
rehydrate_handle()
emit_receipt_expectation()
detect_recall_intent()
detect_completion_claim()
```

### 27.2 Adapter must not

```text
mutate canonical ring buffer directly
promote inferred HLT
treat generic HLT as fallback
hide missing HLT
inject raw output by default
use transcript tail as authority
claim completion without receipt posture
```

---

## 28. Compaction fidelity eval

Every compaction must be evaluable against what it replaced.

### 28.1 Required preserved fields

```text
operator current ask
verified scope
project_root
continuity_id
session_id
HLT status
fallback source and level
Workpoint next action
active blocker
exact failing test/error class
constraints
decisions
evidence refs
receipt expectations
do-not-use boundaries
rehydrate refs
```

### 28.2 Metrics

```text
preserved_required_fields_count
missing_required_fields_count
hallucinated_authority_count
lost_blocker_count
lost_hlt_warning_count
stale_hlt_backfill_detected
missing_rehydrate_ref_count
raw_tool_output_leak_count
generic_hlt_authority_count
completion_claim_without_receipt_count
```

### 28.3 Eval result schema

```json
{
  "schema": "focusa.compaction_fidelity_eval.v1",
  "packet_id": "...",
  "status": "pass|warn|fail",
  "required_fields": {
    "expected": 0,
    "preserved": 0,
    "missing": []
  },
  "authority_failures": [],
  "bloat_failures": [],
  "trajectory_failures": [],
  "receipt_failures": [],
  "score": 0.0,
  "evidence_refs": []
}
```

---

## 29. Cascading compaction detection

Spec 130 must detect repeated compaction without progress.

### 29.1 Signals

```text
compactions_in_last_hour
turns_since_last_verified_progress
same_next_tool_repeated
same_blocker_repeated
summary_size_growth
mission_packet_churn
stable_prefix_churn
workpoint_not_advancing
hlt_warning_repeated_without_action
```

### 29.2 Triggered behavior

When cascading compaction is detected:

```text
pause autonomous durable work
emit visible recap
force focusa_compaction_inspect
recommend focusa_hlt_history / focusa_trajectory_assess / focusa_tool_doctor
surface exact blocker
ask operator only if authority cannot be resolved by tools
```

### 29.3 Finding IDs

```text
cascading_compaction_detected
same_blocker_after_compaction
mission_packet_churn_high
summary_size_growth_high
hlt_warning_unresolved_after_n_turns
```

---

## 30. Optical context compression boundary

Optical context compression may exist only as a gated Bloatgaurd transport optimization.

Spec 130 imports these hard boundaries:

Never image:

```text
operator current ask
recent live turns
Workpoint action authority
Trajectory HLT status
Trajectory warnings
Evidence refs themselves
secrets
tokens
hashes
UUIDs
file paths needed for edits
exact diffs
active error lines
test names currently blocking work
package versions involved in a fix
security-sensitive content
```

Only candidate for imaging:

```text
old dense non-verbatim-critical tool output
old command logs
old collapsed history after checkpoint
large non-current tool docs
large structured JSON already preserved behind rehydrate ref
diagnostic dumps where gist is enough
```

If provider policy, model capability, canary, profitability, or recoverability gate fails:

```text
fallback=text_passthrough
```

---

## 31. Bloatgaurd profiles

Spec 130 should use existing Bloatgaurd profiles.

### 31.1 Recommended default

```text
daily_driver
```

### 31.2 Heavy coding / token-sensitive

```text
speedy
```

### 31.3 Audit/refactor

```text
neat_freak
```

### 31.4 Release / CI strict

```text
tightwad
```

### 31.5 Exact raw inspection

```text
deep_dive
```

---

## 32. Named routines

Spec 130 uses existing Bloatgaurd routines:

```text
Squeezer: compaction and token pressure
Librarian: context selection and bounded bundles
Pantry: scoped cache
Scout: adaptive bloat routing
Deep Dive: explicit handle rehydration
Gatekeeper: strict CI/static enforcement
Janitor: dedupe
Patrol: scan
Brief: report
X-Ray: explain
```

### 32.1 Squeezer automatic triggers

```text
after_checkpoint
before_compaction
token_pressure_high
tool_output_flood
provider_overflow
manual_compaction
model_switch
current_blocker_resolved
```

### 32.2 Scout automatic triggers

```text
before_broad_read
before_audit
token_pressure_high
large_repo_search
large_test_output
long_session
```

---

## 33. Configuration

### 33.1 Suggested config keys

```json
{
  "bloatgaurd": {
    "compaction_mission_packet": true,
    "mission_packet_max_tokens": 1200,
    "mission_packet_pressure_tokens": 800,
    "mission_packet_critical_tokens": 500,
    "tool_history_elision": "after_checkpoint",
    "full_payload_policy": "cold_opt_in",
    "recent_turns_default": 4,
    "recent_turns_hard_cap": 8,
    "provider_prompt_cache": "safe_auto",
    "stable_prefix_churn_gate": "advisory",
    "compaction_fidelity_eval": "advisory",
    "cascading_compaction_detection": true,
    "agent_instruction_diet": "advisory"
  },
  "trajectory": {
    "hlt_required": true,
    "generic_hlt_never_canonical": true,
    "previous_valid_fallback_only": true,
    "loud_warning_required": true,
    "session_hlt_history_required": true
  },
  "receipts": {
    "completion_requires_hlt_posture": true,
    "completion_requires_evidence_refs": true,
    "closure_authority_required": true
  }
}
```

---

## 34. Static tests

Required static tests:

```text
spec130_compaction_packet_schema_static_test
spec130_post_compaction_order_static_test
spec130_trajectory_packet_v3_required_static_test
spec130_hlt_warning_before_route_guidance_static_test
spec130_no_generic_hlt_authority_static_test
spec130_no_last_trajectory_clarity_backfill_static_test
spec130_tool_run_summary_schema_static_test
spec130_tool_output_masking_static_test
spec130_recent_turns_evidence_refs_static_test
spec130_provider_cache_split_static_test
spec130_agent_instruction_diet_static_test
spec130_receipt_completion_gate_static_test
spec130_compaction_fidelity_eval_static_test
spec130_cascading_compaction_detection_static_test
spec130_subagent_result_schema_static_test
spec130_security_redaction_static_test
spec130_no_recursive_summary_static_test
```

---

## 35. Runtime / eval tests

Required runtime/eval tests:

```text
1. Run compaction with valid canonical HLT:
   expect TrajectoryResumePacketV3 with HLT_STATUS=canonical_explicit.

2. Run compaction with missing HLT:
   expect HLT_REQUIRED warning before route guidance.

3. Run compaction with generic bootstrap HLT:
   expect GENERIC_HLT_DEGRADED and no route authority.

4. Run compaction with previous valid HLT from same project but different session:
   expect PREVIOUS_TRAJECTORY_FALLBACK warning and session-specific refresh requirement.

5. Run compaction after large tool output:
   expect ToolRunSummary + ECS handle + omitted counts, not raw log.

6. Run compaction with active failing test:
   expect exact test name/error class preserved.

7. Run provider overflow:
   expect Workpoint checkpoint, Trajectory checkpoint, CompactionMissionPacket, and degraded fallback if Focusa unavailable.

8. Trigger recall-intent after compaction:
   expect recent-turn slice + CompactionMissionPacket re-emission.

9. Attempt completion after missing/generic HLT:
   expect completion blocked or degraded receipt posture.

10. Trigger model switch:
   expect recent-turn slice and CompactionMissionPacket injection.

11. Repeat compaction without progress:
   expect cascading_compaction_detected.

12. Try to use lastTrajectoryClarity as HLT without provenance:
   expect blocked / warning.

13. Try define-goal with generic HLT and operator_confirmed=true:
   expect validation rejected.

14. Run subagent large log triage:
   expect bounded SubagentResult and raw log omitted behind handle.

15. Run stable-prefix cache-compatible provider path:
   expect stable_prefix_hash and cache telemetry.
```

---

## 36. Acceptance criteria

Spec 130 is accepted only when:

```text
1. Compaction has a formal state machine.
2. CompactionMissionPacket schema exists and validates.
3. Post-compaction auto-resume includes TrajectoryResumePacketV3.
4. HLT warning appears before ordinary route guidance when needed.
5. Generic HLT never becomes route authority.
6. Missing HLT always triggers loud warning.
7. Previous valid HLT is the only fallback trajectory source.
8. lastTrajectoryClarity cannot supply HLT without previous-valid provenance.
9. Workpoint is preserved even when HLT is missing/generic.
10. Missing/generic HLT blocks durable work unless degraded receipt posture is explicit.
11. Tool output over threshold is replaced by ToolRunSummary + handle.
12. Active blocker exact error/test/file information survives compaction.
13. RecentTurnSlice includes evidence_refs when evidence exists.
14. Stable-prefix and dynamic-slice split exists.
15. Cache telemetry records cache hit/miss data where provider exposes it.
16. Agent instruction files are covered by Bloatgaurd diet checks.
17. Subagent outputs use a bounded result envelope.
18. Completion claims require receipt/evidence posture.
19. Compaction fidelity eval proves required fields survive.
20. Rehydrate refs exist for every omitted raw tool output.
21. Compaction packet is recomposed from canonical sources, not transcript tail.
22. Cascading compaction detection exists.
23. Security redaction prevents secret-bearing raw logs from prompt injection.
24. API/CLI/Pi render the same authority/warning vocabulary.
25. All tests in §§34–35 pass.
```

---

## 37. Final operating rule

```text
Compaction is not forgetting.
Compaction is controlled projection.

Never invent the north star.
Never hide that the north star is missing.
Never call generic text a trajectory.
Never treat fallback as fresh session truth.
Never let transcript tail become authority.
Never let raw tool output flood the prompt.
Never claim completion without receipts and evidence.

Always query previous valid trajectory before asking for a new HLT.
Always show HLT status before durable work.
Always preserve Workpoint immediate action authority.
Always preserve active blocker exact evidence.
Always store raw evidence behind handles.
Always provide rehydrate refs.
Always make omissions auditable.
Always make Pi carry Trajectory through bootstrap and compaction.
Always prove completion with Evidence and Receipts.
```


---

## 38. 2026-07-13 bounded-persistence and forever-session amendment

This amendment is normative. It extends §§0–37 without deleting, weakening, or
renaming any existing requirement.

The original implementation closure proved prompt compaction, semantic fidelity,
context-firewall behavior, bounded route payloads, and runtime memory telemetry.
It did **not** prove that a native coding-agent session file or its replay working
set remained bounded over an indefinitely long workstream.

### 38.1 Reopening evidence

A real Pi session failed near the V8 heap limit after repeated successful
compactions:

```text
native session bytes:             1,366,100,514
native session entries:           70,279
compaction entries:               19
Focusa custom entries:            54,401
focusa-state entries:             27,985
focusa-state bytes:               1,268,921,044
project-switch ledger entries:    26,359
project-switch ledger bytes:      55,138,510
observed V8 heap failure:          approximately 3.1 GiB
```

The semantic compactor ran, but Pi still had to deserialize the append-only
session tree. Compaction reduced model context while repeated full Focusa state
snapshots continued growing the physical session and replay heap.

The implementation defect includes a semantic-deduplication failure:
`persistState()` included a fresh timestamp inside the serialized payload before
hash comparison, so an otherwise unchanged state could never have the same hash.

### 38.2 Corrected scope

Spec 130 now governs all four compaction boundaries:

```text
1. model prompt/context projection,
2. Focusa semantic state projection,
3. native coding-agent persistence and replay working set,
4. crash-safe continuation across native session/agent boundaries.
```

Passing only boundary 1 or 2 is insufficient for closure.

---

## 39. Forever-session invariant

```text
Logical mission history may grow without a fixed lifetime limit.
Prompt size, hot Focusa state, native active-segment size, replay working set,
and process heap must remain bounded independently of total historical volume.
Every omitted payload must remain integrity-verifiable and rehydratable.
A crash, OOM kill, compaction, model switch, native session rollover, or agent
handoff must resume from a verified Workpoint/Trajectory checkpoint rather than
from transcript-tail inference.
```

“Forever session” means continuous logical mission/workstream continuity. It does
not require one immortal provider-owned JSONL/database object. Physical native
sessions are bounded segments and may rotate transactionally.

### 39.1 Complexity requirements

Let `H` be total historical events and `B` be the configured hot budget.

```text
normal resume heap:            O(B), not O(H)
normal resume latency:         O(latest manifest + latest checkpoint + B)
prompt compilation:            O(selected packet budget)
Focusa native-session writes:  O(unique semantic revisions)
raw evidence storage:          O(unique content)
duplicate unchanged state:     O(1), with zero additional native entries
```

Historical inspection may be `O(H)` only behind an explicit cold-path operation
using streaming/pagination and a declared resource budget.

---

## 40. Authority and identity with rotating continuity IDs

This amendment does not introduce or assume a permanent Pi `continuity_id`.

### 40.1 Existing authority hierarchy remains

```text
ProjectRootKey = verified_project_root + project_fingerprint
WorkstreamKey  = ProjectRootKey + current continuity_id
AttachmentKey  = WorkstreamKey + instance_id + session_id + attachment_id
```

The verified `ProjectRootKey` is stable project authority. A Pi or other adapter
may generate a new `continuity_id` when creating a new native workstream/session.
`session_id` and `attachment_id` remain temporal runtime metadata.

### 40.2 Durable meaning across rotation

Cross-continuity continuation is carried by existing typed authority objects:

```text
- source Project Session Transfer packet,
- source Workpoint id + revision + checkpoint id,
- source Trajectory id + HLT provenance/status,
- CompactionMissionPacket id,
- CLT/lineage refs,
- evidence and receipt refs,
- verified target ProjectRootKey,
- reducer-approved target Workpoint materialization.
```

No target adapter may silently reuse an old `continuity_id`. No project-only,
name-similarity, transcript-summary, or latest-global-pointer lookup may promote
a target Workpoint.

### 40.3 Continuity transition rule

A continuity transition is valid only when:

```text
1. source project scope is verified;
2. source Workpoint checkpoint is accepted and reducer-visible;
3. source Trajectory/HLT posture is captured;
4. a Project Session Transfer packet is durably saved;
5. target project scope is independently verified;
6. target adapter supplies its current generated continuity_id/session_id;
7. reducer materializes or rebinds the target Workpoint from the explicit transfer;
8. target Workpoint resume returns canonical=true for the target authority key;
9. transfer receipt links source and target refs without claiming either id is static.
```

Cross-continuity HLT fallback from Spec 125 remains advisory at the documented
fallback level. It does not substitute for Workpoint transfer authority.

---

## 41. Four-plane persistence architecture

### 41.1 Canonical cognition plane

Focusa daemon/core remains authoritative for:

```text
ProjectIdentity, Trajectory, Workpoint, Focus State, CLT, Evidence/ECS,
Receipts, predictions, metacognition, and reducer-approved transitions.
```

### 41.2 Adapter hot-state plane

Each coding-agent adapter holds only the bounded state required for the active
turn, pressure detection, current attachment, and recovery initiation.

### 41.3 Native session plane

Provider/native sessions store:

```text
- native conversation entries required by that agent,
- bounded Focusa anchor entries,
- compaction entries required by the native runtime,
- segment/transfer refs,
- no repeated full canonical Focusa snapshot.
```

The native transcript is never Focusa authority.

### 41.4 Cold content plane

Large/raw material is content-addressed in Evidence/ECS or a provider-neutral
artifact store:

```text
raw tool output, full Focus State snapshots when needed for audit, logs,
large diagnostics, old native segments, migration indexes, screenshots,
and exact replay artifacts.
```

The hot plane carries digest, byte count, media/type classification, encryption
posture, and rehydrate handle—not the payload.

---

## 42. Bounded persistence schemas

### 42.1 `CompactionPersistenceAnchorV1`

The provider-native Focusa custom entry must use a bounded reference envelope:

```json
{
  "schema": "focusa.compaction_persistence_anchor.v1",
  "anchor_revision": 42,
  "semantic_digest": "sha256:...",
  "project_root": "/verified/project",
  "project_fingerprint": "sha256:...",
  "continuity_id": "current-generated-continuity",
  "session_id": "current-native-session",
  "workpoint_id": "wp_...",
  "workpoint_revision": 7,
  "checkpoint_id": "checkpoint_...",
  "trajectory_id": "trajectory:...",
  "hlt_status": "canonical_explicit",
  "compaction_packet_id": "cmp_...",
  "session_transfer_id": null,
  "focus_state_ref": "ecs:json:...",
  "evidence_refs": ["evidence:..."],
  "rehydrate_refs": ["ecs:..."],
  "created_at": "iso8601"
}
```

Rules:

```text
hard serialized size: 8 KiB
normal target:         4 KiB
created_at excluded from semantic_digest
arrays deterministically ordered before hashing
secrets/raw tokens/raw logs forbidden
missing refs represented explicitly, never by copied raw state
```

### 42.2 `NativeSessionPressureV1`

```json
{
  "schema": "focusa.native_session_pressure.v1",
  "adapter": "pi|claude|codex|opencode|other",
  "native_session_ref": "opaque-provider-ref",
  "session_bytes": 0,
  "entry_count": 0,
  "focusa_custom_bytes": 0,
  "focusa_custom_entries": 0,
  "duplicate_anchor_count": 0,
  "heap_used_bytes": 0,
  "heap_limit_bytes": 0,
  "headroom_ratio": 1.0,
  "posture": "normal|soft_pressure|hard_pressure|emergency|oversized_at_start",
  "recommended_action": "continue|checkpoint|compact|rollover|stream_migrate|refuse_full_load",
  "measured_at": "iso8601"
}
```

### 42.3 Project-switch observation

A project-switch observation is persisted only when its semantic tuple changes:

```text
(project_root, project_fingerprint, confidence class, authority status,
 conflict status, source class)
```

Repeated observations update daemon telemetry/counters but do not append another
native custom entry. The native envelope hard limit is 2 KiB.

### 42.4 Transfer receipt extension

Existing Project Session Transfer output must gain a typed transition receipt:

```json
{
  "schema": "focusa.continuity_transition_receipt.v1",
  "transfer_id": "transfer_...",
  "source": {
    "project_root_key": "...",
    "continuity_id": "source-generated-id",
    "session_id": "source-session",
    "workpoint_id": "wp_...",
    "workpoint_revision": 7,
    "checkpoint_id": "checkpoint_..."
  },
  "target": {
    "project_root_key": "...",
    "continuity_id": "target-generated-id",
    "session_id": "target-session",
    "workpoint_id": "wp_...",
    "workpoint_revision": 8
  },
  "trajectory_id": "trajectory:...",
  "compaction_packet_id": "cmp_...",
  "evidence_refs": ["evidence:..."],
  "status": "prepared|materialized|verified|degraded|blocked",
  "blocked_reason": null
}
```

This receipt links dynamic authority epochs. It is not a new static session id.

---

## 43. Semantic revision, deduplication, and coalescing

### 43.1 Stable digest

The persistence digest must include only semantic recovery fields. It must exclude:

```text
timestamps, last-observed times, telemetry counters, request ids, process ids,
heap samples, display-only animation state, repeated health polls, and other
volatile values that do not change recovery meaning.
```

### 43.2 Write rule

```text
if semantic_digest == last_persisted_semantic_digest:
    append nothing
else:
    externalize oversized/cold fields
    append exactly one bounded anchor for the new semantic revision
```

A time interval may rate-limit changed writes. Time passage alone must never force
an unchanged write.

### 43.3 Coalescing

Within one agent turn or one reducer transaction:

```text
- multiple state mutations coalesce to the final semantic revision;
- project-switch observations coalesce by semantic tuple;
- repeated warnings coalesce by finding id + posture;
- WBM and normal persistence may reference the same anchor instead of duplicating payload;
- tool-output pressure updates store counters separately from recovery anchors.
```

### 43.4 Offline shadow

Offline recovery state must be stored atomically in a bounded local sidecar or
Focusa artifact, not copied into every native session entry.

Requirements:

```text
0600-equivalent permissions where supported
atomic temp-write + fsync + rename
project/continuity/session scope in envelope
content digest verification on read
bounded generations with latest verified pointer
no canonical promotion while Focusa is unavailable
reconciliation on reconnect
```

---

## 44. Adaptive native-session budgets

Defaults are bounded by both absolute size and runtime heap:

```text
anchor payload hard cap:          8 KiB
project-switch payload hard cap:  2 KiB
Focusa custom bytes soft cap:     min(8 MiB, 0.5% heap limit)
Focusa custom bytes hard cap:     min(16 MiB, 1% heap limit)
native segment soft cap:          min(64 MiB, 5% heap limit)
native segment hard cap:          min(128 MiB, 10% heap limit)
native startup migration cap:     min(256 MiB, 20% heap limit)
soft heap headroom floor:         35%
hard heap headroom floor:         20%
emergency heap headroom floor:    10%
```

An operator may lower these values. Raising a hard cap requires explicit deep-dive
configuration and must not disable preflight/migration.

### 44.1 Postures

`normal`:

```text
bounded anchors; ordinary Spec 130 behavior
```

`soft_pressure`:

```text
checkpoint if semantic progress changed; suppress cold hydration; compact prompt;
prepare rollover/migration capability
```

`hard_pressure`:

```text
stop nonessential native persistence; checkpoint Workpoint and Trajectory;
build CompactionMissionPacket; seal current segment; request/perform rollover
```

`emergency`:

```text
abort cold/full-payload work; write the minimum transactional recovery record;
flush/fsync; stop agent loop before V8/OS exhaustion; supervisor restarts into a
verified target attachment
```

`oversized_at_start`:

```text
do not ask the native runtime to deserialize the full session; invoke streaming
migration/recovery first
```

---

## 45. Cross-agent capability contract

Every adapter must publish:

```json
{
  "adapter": "pi",
  "supports_compaction_hook": true,
  "supports_bounded_custom_entry": true,
  "supports_session_size_preflight": true,
  "supports_automatic_native_rollover": false,
  "supports_user_command_rollover": true,
  "supports_rpc_rollover": true,
  "supports_streaming_import": false,
  "supports_external_rehydrate": true,
  "supports_preload_receipt": true
}
```

Capabilities are measured, versioned, and evidence-backed. An adapter must never
claim a stronger tier than its actual API allows.

### 45.1 Adapter tiers

```text
Tier A — automatic bounded rollover:
  launcher/RPC/native API can checkpoint, replace physical session, inject target
  packet, and verify resume without operator data entry.

Tier B — command-gated rollover:
  native API permits replacement only from an explicit user command; Focusa
  checkpoints early and presents one exact command before hard pressure.

Tier C — restart handoff:
  adapter cannot replace sessions; Focusa saves transfer/preload receipt and
  restarts a fresh native session under supervisor control.

Tier D — observe-only:
  no reliable injection/transfer; durable work is blocked or explicitly degraded.
```

### 45.2 Pi 0.81.1 verified boundary

The tested Pi SDK provides:

```text
- pi.appendEntry() for custom persistence,
- session_start reasons plus session_before_switch/session_before_fork,
- session_before_compact/session_compact hooks with manual/threshold/overflow reason,
- session_shutdown reasons for quit/reload/new/resume/fork,
- read-only sessionManager and stable getSessionId() in ordinary extension contexts,
- ctx.compact() in ordinary extension contexts,
- typed newSession()/fork() with transactional withSession only in ExtensionCommandContext,
- RPC newSession/compact operations.
```

Therefore:

```text
- the Pi extension can immediately bound anchors and detect pressure;
- it must not unsafe-cast an event context to call newSession();
- foreground automatic Tier A rollover requires a launcher/RPC supervisor or a
  future Pi API that explicitly permits safe idle-time session replacement;
- /focusa-rollover may implement Tier B through ExtensionCommandContext;
- oversized startup recovery must run before Pi loads the JSONL.
```

### 45.3 Pi session/project classification boundary

Pi 0.81 emits `session_start` for startup, reload, new, resume, and fork transitions; it does not emit the legacy post-transition `session_switch` or `session_fork` events. The adapter must use `sessionManager.getSessionId()` plus verified project evidence and persisted Focusa anchors to classify:

```text
new_session_new_project
new_session_existing_project
resumed_session_resumed_project
resumed_session_recoverable_project
session_project_mismatch
forked_compacted_continuation
```

A stable Pi UUID is temporal evidence, not project authority. Matching root evidence may rehydrate non-blockingly; a missing marker with matching durable root evidence is recoverable; a root mismatch must fail closed before importing Workpoint, Trajectory, decisions, or evidence. Lifecycle guidance is queued as an idempotent advisory in the next real operator-turn tail and must never call `sendUserMessage()` from session hooks.

Claude, Codex, OpenCode, and future adapters must publish equivalent measured
capabilities rather than inheriting Pi assumptions.

---

## 46. Transactional rollover state machine

```text
monitoring
  -> soft_pressure
  -> checkpointing
  -> packet_built
  -> source_segment_sealed
  -> transfer_saved
  -> target_attachment_created
  -> target_workpoint_materialized
  -> target_resume_verified
  -> source_archived
  -> resumed
```

Failure states:

```text
checkpoint_failed
packet_build_failed
segment_seal_failed
transfer_save_failed
target_create_failed
target_scope_mismatch
target_workpoint_pending
target_resume_degraded
archive_integrity_failed
```

### 46.1 Transaction ordering

1. Stop accepting nonessential writes.
2. Wait for an idle boundary or safely abort the active operation.
3. Checkpoint Workpoint and Trajectory under source authority.
4. Build/evaluate the Spec 130 CompactionMissionPacket.
5. Flush and fsync the offline sidecar and native segment.
6. Record source checksum, byte count, last entry id, checkpoint, and packet refs.
7. Save Project Session Transfer.
8. Ask the adapter/supervisor to create the target native attachment.
9. Generate the target adapter’s current `continuity_id`; do not reuse source id.
10. Verify ProjectRootKey.
11. Materialize target Workpoint through reducer-approved transfer.
12. Inject one bounded resume/preload packet.
13. Require canonical target Workpoint resume before autonomous durable work.
14. Mark transfer verified and source segment archived/read-only.

A failed target transition leaves the source segment and checkpoint intact and
retryable. It never deletes or mutates the only recovery copy.

---

## 47. Streaming migration of oversized native sessions

### 47.1 Preflight

Launcher/supervisor checks native session metadata and file size before starting
the provider runtime. Files above the startup migration cap enter
`oversized_at_start`.

### 47.2 Migration algorithm

Migration must use bounded memory:

```text
1. acquire a migration lock;
2. hash and record the immutable source;
3. stream JSONL/events line by line;
4. build any parent/branch lookup in a bounded on-disk index;
5. identify active branch, latest native compaction, latest valid Focusa anchor,
   latest Workpoint/Trajectory/transfer refs, and bounded recent turns;
6. query canonical Focusa state when available;
7. build/evaluate a new CompactionMissionPacket;
8. write a temporary target segment containing native header, bounded recovery
   context, anchor/transfer refs, and the recent-turn budget;
9. fsync and validate target parse/fidelity/integrity;
10. atomically publish target segment/manifest;
11. retain source as immutable cold evidence with rehydrate ref;
12. start target adapter and verify canonical resume.
```

No migration path may parse the complete payload into one JavaScript object graph.

### 47.3 Degraded migration

If Focusa daemon is unavailable:

```text
use latest integrity-valid anchor + native compaction + bounded recent turns;
mark canonical=false and degraded=true;
retain exact source checksum/ref;
require project verify + Workpoint checkpoint/resume reconciliation before durable work.
```

---

## 48. Crash and OOM safety

Absolute prevention of every process or hardware crash is not claimable. Spec 130
requires that compaction/persistence pressure does not cause unrecoverable mission
loss and that known OOM trajectories are intercepted before heap exhaustion.

### 48.1 Required guards

```text
startup size preflight
continuous native/custom byte accounting
heap/headroom telemetry when exposed
write amplification and duplicate ratio telemetry
minimum recovery checkpoint before hard-pressure action
watchdog/supervisor restart contract
atomic sidecar/manifest writes
checksummed immutable source segments
idempotent transfer/migration ids
bounded retry/circuit-breaker behavior
```

### 48.2 Forbidden behavior

```text
increase NODE_OPTIONS heap as the primary fix
load a known oversized native session before migration
append full Focusa state per hook/poll/turn
hash volatile timestamp fields as semantic state
silently reuse stale continuity authority
rewrite/delete the sole source session during migration
claim forever-session support from prompt compaction alone
```

A larger heap may be used once as a controlled forensic/migration aid, never as
the product architecture.

---

## 49. Backward compatibility and rollback

### 49.1 Dual read, bounded write

Adapters must read legacy `focusa-state`/`focusa-wbm-state` entries for migration,
but all new normal writes use bounded anchors/sidecars.

```text
legacy read: supported through a bounded latest-valid lookup or streaming migrator
legacy write: forbidden after amendment activation
new anchor read: required
new anchor write: required
```

### 49.2 Rollback

Rollback may disable automatic rollover or target-materialization changes, but it
must not restore repeated full-state native writes.

Rollback procedure:

```text
1. pause rollover initiation;
2. preserve source/target segments and transfer receipt;
3. select last verified source or target checkpoint;
4. restore bounded anchor/sidecar reader;
5. verify ProjectRootKey + Workpoint resume;
6. resume in degraded Tier B/C if Tier A is unavailable.
```

---

## 50. Implementation plan and exact surfaces

### 50.1 Phase A — immediate write-amplification stop

Primary files:

```text
apps/pi-extension/src/state.ts
apps/pi-extension/src/persistence.ts
apps/pi-extension/src/session.ts
apps/pi-extension/src/config.ts
apps/pi-extension/src/compaction.ts
tests/pi_extension_contract_test.sh
tests/pi_extension_runtime_authority_test.mts
tests/spec130_compaction_mission_packet_static_test.sh
```

Required changes:

```text
- separate semantic payload from volatile metadata;
- stable deterministic digest;
- zero append on unchanged digest;
- bounded anchor writer and legacy reader;
- WBM references the same anchor instead of duplicating full payload;
- project-switch semantic dedupe/coalescing;
- custom-entry byte/duplicate telemetry;
- anchor payload hard-cap rejection/externalization;
- regression fixture reproducing timestamp-hash defect.
```

### 50.2 Phase B — pressure and migration substrate

Primary files/surfaces:

```text
apps/pi-extension/src/session.ts
apps/pi-extension/src/session-pressure.ts
apps/pi-extension/src/auto-compaction.ts
apps/pi-extension/src/compaction.ts
apps/pi-extension/src/commands.ts
apps/pi-extension/src/tools.ts
crates/focusa-api/src/routes/compaction.rs
crates/focusa-cli/src/commands/compaction.rs
Project Session Transfer API/storage
Focusa launcher/install integration
```

Required changes:

```text
- NativeSessionPressureV1 measurement/render;
- /focusa-rollover Tier B command;
- launcher preflight before Pi/native agent load;
- streaming migration command and inspect/dry-run mode;
- segment manifest and immutable archive refs;
- pressure-triggered Workpoint/Trajectory/Compaction packet transaction;
- no hidden native-session mutation from unsupported event contexts.
```

### 50.3 Phase C — rotating-continuity transfer

Primary existing systems:

```text
Project Session Transfer route/tool
Workpoint checkpoint/resume reducer
Trajectory checkpoint/resume
CLT lineage
Preload packet/receipt
agent capability registry
```

Required changes:

```text
- source-to-target transition receipt;
- explicit target continuity/session binding;
- reducer-approved target Workpoint materialization;
- canonical target resume gate;
- Pi launcher/RPC integration;
- Claude/Codex/OpenCode capability manifests and adapter conformance fixtures.
```

### 50.4 Phase D — proof and closure

```text
real oversized-session migration
multi-segment soak
cross-agent handoff matrix
crash injection at every transaction boundary
bounded startup/replay memory profile
semantic fidelity and exact evidence rehydration
operator-visible recovery UX
```

---

## 51. Security and privacy extension

Segment manifests, anchors, sidecars, and transition receipts inherit §§20 and 27.

Additional requirements:

```text
- source native session paths are never prompt-visible by default;
- archived raw sessions use restricted local permissions;
- secret-bearing tool output remains behind secure handles;
- transfer receipts contain refs/digests, not raw credentials;
- cross-device transfer authenticates device/operator and verifies project scope;
- migration logs redact raw payload and local private topology;
- content hashes do not become authorization tokens;
- deletion/retention follows canonical retention policy and requires proof that a
  verified successor plus immutable recovery reference exists.
```

---

## 52. Amendment static, runtime, and soak tests

### 52.1 Static tests

```text
spec130_no_volatile_fields_in_semantic_digest_static_test
spec130_bounded_anchor_schema_static_test
spec130_anchor_hard_cap_static_test
spec130_no_full_state_native_write_static_test
spec130_project_switch_semantic_dedupe_static_test
spec130_dynamic_continuity_transition_static_test
spec130_no_static_continuity_assumption_static_test
spec130_adapter_capability_contract_static_test
spec130_pi_command_context_boundary_static_test
spec130_startup_preflight_static_test
spec130_streaming_migration_static_test
spec130_release_closure_reopened_static_test
```

### 52.2 Runtime tests

```text
1. Call persistence repeatedly with unchanged semantic state:
   expect one native anchor, zero duplicate appends.

2. Change only timestamp/telemetry/process fields:
   expect unchanged semantic digest and zero append.

3. Change Workpoint revision:
   expect one bounded anchor with new revision and valid refs.

4. Enable WBM:
   expect reference reuse, not a second full payload.

5. Repeat identical project observations 100,000 times:
   expect bounded/coalesced native persistence.

6. Exceed anchor payload cap:
   expect externalization or fail-closed rejection, never oversized append.

7. Hit soft/hard/emergency pressure:
   expect the exact state-machine transitions and checkpoint ordering.

8. Start with an oversized native session:
   expect launcher refusal to full-load and streaming migration.

9. Kill migration after every transaction step:
   expect source remains intact and retry is idempotent.

10. Rotate Pi continuity_id:
    expect explicit transfer and canonical target Workpoint; no stale id reuse.

11. Hand off Pi -> Claude -> Codex/OpenCode -> Pi:
    expect same mission/Workpoint lineage, current target attachment authority,
    exact blocker/evidence, and no transcript-tail inference.

12. Run without daemon:
    expect bounded degraded sidecar/anchor recovery and blocked canonical claims.
```

### 52.3 Real regression and soak gates

The amendment cannot close without:

```text
- successful bounded-memory recovery of the observed 1.366 GB failure artifact;
- source artifact checksum preserved;
- no complete JavaScript object-graph load during migration;
- peak migration RSS within declared budget;
- active native segment remains below configured hard cap;
- at least 1,000,000 synthetic semantic events across multiple physical segments;
- at least 10,000 compaction/checkpoint/rollover cycles in stress simulation;
- zero required-field fidelity loss;
- zero lost blocker/evidence/receipt refs;
- zero unchanged-state native appends;
- startup/replay memory slope statistically flat relative to total history;
- crash recovery succeeds at every transaction boundary;
- adapter conformance passes for each claimed capability tier.
```

---

## 53. Amended acceptance criteria

All original §36 criteria remain mandatory. Spec 130 remains reopened until all of
the following also pass:

```text
26. Full Focusa state is not repeatedly persisted into native agent sessions.
27. Semantic dedupe excludes volatile fields and produces zero unchanged writes.
28. Every native Focusa entry satisfies its hard byte cap.
29. Project-switch persistence is semantic-change driven and coalesced.
30. Native session/custom bytes and heap headroom have typed pressure telemetry.
31. Startup preflight prevents full loading of known oversized sessions.
32. Oversized migration is streaming, atomic, checksummed, and reversible.
33. Logical continuity survives physical segment rotation.
34. Pi continuity_id rotation is handled explicitly; no static-id assumption exists.
35. Cross-continuity Workpoint authority requires reducer-approved transfer.
36. Every adapter publishes measured capability posture.
37. Unsupported event contexts never invoke native session replacement by cast/hack.
38. Tier A/B/C behavior is truthful and operator-visible.
39. Old native segments remain rehydratable and integrity-verifiable.
40. The real 1.366 GB OOM artifact resumes under the declared memory budget.
41. Million-event/multi-segment soak keeps hot memory and startup cost bounded.
42. Crash injection proves deterministic recovery from every rollover boundary.
43. Cross-agent handoff preserves mission, blocker, evidence, and receipt fidelity.
44. Rollback never restores unbounded full-state native writes.
45. Closure evidence includes exact commands, metrics, artifacts, and receipts.
```

---

## 54. Amended final operating rule

```text
Compaction is controlled projection across prompt, semantic state, persistence,
and replay—not only a model summary.

Keep canonical meaning in Focusa.
Keep provider-native hot state bounded.
Keep raw history immutable, content-addressed, and rehydratable.
Treat continuity_id as current workstream authority, not a permanent global id.
Transfer Workpoint authority explicitly when continuity rotates.
Checkpoint before pressure becomes failure.
Migrate oversized sessions before native deserialization.
Never trade semantic memory for heap safety.
Never trade heap safety for transcript retention.
Never claim forever-session support until real long-history, crash, migration, and
cross-agent gates prove bounded continuation.
```
