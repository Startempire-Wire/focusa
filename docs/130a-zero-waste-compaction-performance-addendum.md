# Spec 130A — Zero-Waste Compaction Performance Addendum

Status: specified — implementation and proof pending  
Parent spec: `docs/130-hlt-aware-compaction-mission-packet-and-bloatgaurd-context-firewall-spec.md`  
Canonical label: Spec 130A Zero-Waste Compaction  
Primary issues: GitHub #11, #13  
Imported dependencies: GitHub #12, #14  
External authority dependencies: GitHub #4, #10  
Primary surfaces: Focusa core, API, Pi adapter, Context Cognition, Bloatgaurd, Evidence/ECS, persistence, telemetry, tests

---

## 0. Purpose

Spec 130A defines the performance and net-value requirements for Focusa compaction.

Spec 130 already establishes that compaction is controlled projection across prompt context, semantic state, native persistence, replay, and crash-safe continuation. Spec 130A adds one further closure condition:

```text
Compaction must produce a measurable net improvement in useful agent execution.
```

A compaction implementation is incomplete when it preserves semantic state but:

```text
duplicates model calls,
invalidates reusable prompt cache,
blocks daemon hot routes,
reconstructs state through redundant calls,
retains raw tool payloads in hot memory,
injects a large replacement prompt,
triggers unnecessary continuation turns,
writes unchanged state repeatedly,
or costs more than it saves.
```

### 0.1 Product objective

```text
Maximize verified productive progress retained per:

- prompt token,
- model call,
- wall-clock millisecond,
- hot-memory byte,
- native-session byte,
- durable write,
- cache miss,
- rehydrated byte,
- and recovery operation.
```

### 0.2 Net-positive optimization rule

A compaction change is net-positive only when:

```text
1. verified task success is preserved or improved;
2. authority, scope, evidence, blocker, and receipt fidelity do not regress;
3. at least one meaningful cost is materially reduced;
4. no other cost materially worsens without an explicit accepted tradeoff;
5. recovery remains deterministic and inspectable;
6. replay and live-session evidence prove the result.
```

Meaningful costs include:

```text
prompt tokens
re-billed historical tokens
compaction model tokens
resume-projection tokens
model calls
prepare latency
verify latency
time to productive continuation
process heap
hot artifact memory
native-session growth
sidecar writes
daemon queue depth
rehydration count
rehydrated bytes
repeat-error rate
operator intervention
```

---

## 1. Relationship to Spec 130

Spec 130A is a normative companion to Spec 130. It does not replace, renumber, or weaken any Spec 130 requirement.

Spec 130 remains authoritative for:

```text
scope and HLT posture
Trajectory and Workpoint authority
CompactionMissionPacket
persistence anchors and sidecars
native-session pressure
rollover and migration
crash recovery
cross-agent continuity
security and redaction
fidelity and receipt gates
```

Spec 130A governs:

```text
single-attempt ownership
prompt-cache preservation
control-plane call count
resume-projection size
artifact externalization efficiency
conditional continuation
persistence hot-path efficiency
pressure prediction
post-compaction effectiveness
compaction ROI
```

When requirements conflict, the stricter authority, evidence, security, or recovery rule wins.

---

## 2. Incident basis and issue mapping

### 2.1 GitHub #11 — auto-compaction re-entrancy

Spec 130A fully governs:

```text
process-wide registration lease
one coordinator
one epoch
one native invocation
duplicate extension detection
retry serialization
primary failure preservation
single completion notice
canonical recovery preservation
```

### 2.2 GitHub #13 — provider cache destruction

Spec 130A fully governs:

```text
stable system prefix
historical-prefix preservation
dynamic current-turn tail
cache telemetry
cache-miss classification
cache-safe degraded mode
runtime proof of materially reduced re-billing
```

### 2.3 GitHub #14 — lifecycle prompt races

Spec 130A imports only:

```text
serialized automatic message delivery
non-triggering advisories
operator-message priority
unknown-completion handling
next-turn deferral
```

Project-binding behavior remains governed by GitHub #14.

### 2.4 GitHub #12 — daemon persistence stalls

Spec 130A imports only:

```text
non-blocking compaction prepare/verify persistence
dedicated writer or governed blocking pool
bounded checkpoint acknowledgement
persistence saturation telemetry
```

The general daemon persistence redesign remains governed by GitHub #12.

### 2.5 GitHub #4 and #10

These remain external authority prerequisites.

```text
GitHub #4 provides verified project-directory identity.
GitHub #10 provides canonical parent and working-subpath identity.
```

Compaction consumes their typed output and fails closed when it is unavailable.

---

## 3. Zero-Waste invariants

```text
1. One attachment-scoped CompactionCoordinator owns all compaction decisions.

2. One CompactionEpoch permits at most one live native compaction invocation.

3. One successful compaction performs at most:
   - one preparation transaction,
   - one native compaction call,
   - one verification transaction,
   - and one continuation delivery when continuation is required.

4. Focusa does not make a separate summarizer model call by default.

5. Provider-native summarization may preserve tactical context,
   but Focusa canonical packets remain authority.

6. Tool payloads are externalized once into complete,
   content-addressed artifacts.

7. Raw tool payloads do not remain in hot prompt or hot process memory.

8. Post-compaction context receives one globally budgeted resume projection,
   not several expanded packet renderings.

9. Historical provider messages remain byte- and order-stable whenever
   the provider adapter permits cache reuse.

10. Dynamic Focusa context is placed after the reusable historical prefix,
    adjacent to the current ask.

11. Ordinary compaction persistence is coalesced and non-blocking.

12. Native-session pressure uses exact or integrity-verifiable counters,
    not an unqualified partial sample.

13. Manual or idle compaction does not automatically trigger another model call.

14. Fixed every-N-turn micro-compaction is disabled by default.

15. Compaction is triggered by measured or predicted pressure,
    not arbitrary cadence alone.

16. Unchanged semantic packets, anchors, and projections are reused.

17. Every compaction proves that live context was actually reduced.

18. Every failure preserves one primary error classification and suppresses
    secondary retry or re-entrancy noise.

19. Operator input always outranks automatic compaction continuation.

20. No optimization may weaken fail-closed scope or completion gates.
```

---

## 4. CompactionEpoch and coordinator ownership

### 4.1 CompactionEpoch schema

```json
{
  "schema": "focusa.compaction_epoch.v1",
  "epoch_id": "sha256:...",
  "adapter": "pi",
  "instance_id": "pi-process-...",
  "attachment_id": "native-attachment-...",
  "project_root": "/verified/project",
  "continuity_id": "focusa-cont-...",
  "session_id": "native-session-...",
  "source_turn_id": "pi-turn-...",
  "source_workpoint_id": "wp_...",
  "source_workpoint_revision": 7,
  "source_trajectory_id": "trajectory:...",
  "trigger_class": "predicted_pressure|hard_pressure|provider_overflow|manual|rollover",
  "created_at": "iso8601"
}
```

The semantic epoch identity is deterministic from:

```text
adapter
instance_id
attachment_id
verified scope
source Workpoint revision
source Trajectory revision
source turn
trigger class
```

Time alone must not create another logical attempt.

### 4.2 One CompactionCoordinator

All of the following route through one coordinator:

```text
native automatic threshold compaction
Focusa proactive compaction
Focusa hard-pressure compaction
provider-overflow compaction
manual Focusa compaction
micro-compaction
session rollover preparation
emergency recovery
```

No module may invoke `ctx.compact()` independently once coordinator ownership is active.

### 4.3 Process-wide registration lease

Module-local state is insufficient.

Every adapter establishes a process-wide, versioned lease, such as:

```text
Symbol.for("focusa.compaction.coordinator.v1")
```

The lease binds:

```text
adapter instance
extension build
native session
attachment id
registration source
registered handlers
active epoch
```

A second discovered Focusa extension must:

```text
not register another compaction handler,
not invoke native compaction,
emit one bounded duplicate-install diagnostic,
identify the active owner,
and recommend installation repair.
```

### 4.4 Coordinator states

```text
idle
observing
prepare_requested
preparing
prepared
native_compaction_requested
native_compaction_active
native_compaction_failed
native_compaction_complete
verifying
verified
resume_pending
resume_delivered
deferred_to_next_turn
rollover_required
blocked
cooldown
```

### 4.5 Trigger precedence

```text
emergency native pressure
provider overflow
hard live-context pressure
native rollover requirement
predicted next-turn overflow
semantic-noise pressure
ordinary proactive threshold
manual request
optional optimization request
```

A higher-priority trigger may upgrade the active epoch. It may not create a concurrent epoch.

### 4.6 Invocation rule

```text
For each CompactionEpoch:
native_compaction_call_count <= 1
```

A failed native invocation may be retried only through a linked retry epoch after:

```text
the first attempt is fully settled,
the primary error is persisted,
the cooldown permits retry,
the live context still requires action,
and no native/manual compaction completed meanwhile.
```

### 4.7 Primary-error preservation

When provider summarization fails, that error remains the primary failure.

Secondary errors caused by:

```text
re-entrancy
undefined abort state
duplicate completion
blind retry
duplicate extension registration
concurrent message delivery
```

must be separately classified and must not obscure the initiating failure.

---

## 5. Adaptive pressure prediction

### 5.1 Required inputs

```text
current live-context tokens
provider context window
configured reserve
absolute token cap
recent input growth
recent tool-output growth
resume-projection budget
native-session bytes
Focusa custom bytes
heap headroom
same-error repetition
Workpoint advancement
scope conflict
operator corrections
cache behavior
```

### 5.2 Predicted peak

```text
predicted_peak =
    current_context_tokens
  + p95_recent_next_turn_input_growth
  + p95_recent_tool_output_growth
  + required_provider_reserve
  + required_resume_projection_tokens
```

The predictor uses a bounded recent sample and exposes:

```text
sample_count
p50 growth
p95 growth
confidence
fallback reason
```

When history is insufficient, Focusa uses the existing safe threshold policy.

### 5.3 Trigger rule

Proactive compaction may begin when:

```text
predicted_peak >= safe_context_limit
```

It must not begin solely because an arbitrary number of turns elapsed.

### 5.4 Semantic pressure

Semantic pressure exists when one or more occur:

```text
same blocker repeated without new evidence
same failing command or tool repeated
same exact next action repeated without Workpoint advancement
Workpoint revision stagnant across productive-looking turns
scope conflict or correction invalidates carried context
tool-output flood obscures current action
compaction summary or projection grows across generations
rehydration repeatedly requests the same omitted payload
agent output demonstrates context carryover contamination
```

The coordinator selects the least expensive sufficient action:

```text
projection rebuild
artifact externalization
micro-compaction
fresh subagent
operator recap
native compaction
session rollover
```

### 5.5 Fixed cadence

Default:

```text
fixed_every_n_turns_micro_compaction = disabled
```

Manual micro-compaction remains allowed. Automatic micro-compaction requires a measured semantic or token-pressure reason.

---

## 6. Incremental compaction readiness

### 6.1 Dirty domains

```text
scope
current_ask
workpoint
trajectory
focus_state
active_blocker
evidence
tool_artifacts
recent_turns
receipt_expectation
native_pressure
```

A domain receives a revision only when its recovery meaning changes.

### 6.2 Domain digests

```text
scope_digest
ask_digest
workpoint_digest
trajectory_digest
focus_digest
blocker_digest
evidence_digest
artifact_index_digest
receipt_digest
```

The root compaction digest is derived from these domain digests.

Unchanged domains must not be recursively rebuilt, serialized, or rewritten during ordinary preparation.

### 6.3 Coalescing

Within one turn or reducer transaction:

```text
multiple semantic mutations coalesce to the final revision
multiple warnings coalesce by finding id and posture
multiple tool-output observations coalesce into artifact metadata
multiple pressure observations update counters without creating recovery anchors
```

### 6.4 Always-compaction-ready posture

Focusa must not wait until critical pressure to discover:

```text
the active Workpoint
the HLT posture
the active blocker
the latest evidence refs
the current ask
the exact next action
```

A pressure event seals already-bounded recovery state. It does not reconstruct mission meaning from the full transcript.

---

## 7. Unified preparation and verification

### 7.1 Preparation route

```http
POST /v1/compaction/prepare
```

Request:

```json
{
  "schema": "focusa.compaction_prepare_request.v1",
  "epoch": {},
  "scope": {},
  "trigger": {},
  "current_ask": {},
  "local_semantic_deltas": {},
  "native_pressure": {},
  "adapter_capabilities": {}
}
```

Response:

```json
{
  "schema": "focusa.compaction_prepare_result.v1",
  "status": "prepared|degraded|blocked|rollover_required",
  "epoch_id": "sha256:...",
  "source_revision": 0,
  "semantic_digest": "sha256:...",
  "workpoint_checkpoint_ref": "workpoint-checkpoint:...",
  "trajectory_checkpoint_ref": "trajectory-checkpoint:...",
  "compaction_packet_ref": "compaction:...",
  "resume_projection": {},
  "native_compactor_instructions": "...",
  "fidelity_manifest": {},
  "persistence_ack": {},
  "warnings": []
}
```

### 7.2 Preparation transaction

`/compaction/prepare` must:

```text
1. verify scope once;
2. apply accepted local semantic deltas once;
3. capture Workpoint and Trajectory under one source revision;
4. capture blocker, evidence, and receipt posture under that revision;
5. construct or reuse the CompactionMissionPacket;
6. construct or reuse the CompactionResumeProjection;
7. persist one durable preparation record;
8. return one coherent result.
```

Independent reads may execute concurrently. Authority-bearing writes remain ordered through the canonical reducer/single-writer boundary.

### 7.3 Verification route

```http
POST /v1/compaction/verify
```

Request:

```json
{
  "schema": "focusa.compaction_verify_request.v1",
  "epoch_id": "sha256:...",
  "native_compaction_result": {},
  "context_usage_before": {},
  "context_usage_after": {},
  "native_pressure_after": {},
  "delivery_posture": "immediate|deferred|none"
}
```

Response:

```json
{
  "schema": "focusa.compaction_verify_result.v1",
  "status": "verified|ineffective|degraded|rollover_required|blocked",
  "context_release_ratio": 0.0,
  "required_fields_preserved": true,
  "workpoint_resume_status": "canonical|degraded|blocked",
  "resume_projection_ref": "compaction-resume:...",
  "recommended_next": "continue|defer|rollover|inspect|operator_action",
  "findings": []
}
```

### 7.4 Call budget

A normal successful compaction uses:

```text
prepare RPC count <= 1
verify RPC count <= 1
```

Additional cold rehydration is forbidden during normal preparation unless a required authority field is missing.

### 7.5 Non-blocking persistence

Prepare and verify routes must not perform large synchronous serialization or SQLite filesystem work on Tokio core workers.

They use:

```text
the dedicated persistence actor,
a governed blocking pool,
or an equivalent bounded single-writer mechanism.
```

Hard-pressure recovery records receive priority over ordinary background snapshot work.

---

## 8. Content-addressed tool and history artifacts

### 8.1 One complete artifact

Large output follows:

```text
stream input
→ redact while streaming
→ hash while streaming
→ store complete artifact
→ atomically publish
→ return one stable handle
```

The normal path must not:

```text
send only an arbitrary first slice to the daemon
keep a second full process-memory copy
produce unrelated local and daemon ids
discard the tail of the evidence source
```

### 8.2 HistoryArtifact schema

```json
{
  "schema": "focusa.history_artifact.v1",
  "artifact_id": "sha256:...",
  "kind": "tool_output|log|transcript_segment|diagnostic|migration_source",
  "media_type": "text/plain",
  "bytes": 0,
  "token_estimate": 0,
  "line_count": 0,
  "source": {
    "adapter": "pi",
    "session_id": "...",
    "turn_id": "...",
    "tool_name": "...",
    "target_refs": []
  },
  "security": {
    "restricted": false,
    "redaction_applied": false
  },
  "storage_ref": "ecs:...",
  "search_ref": "ecs-search:...",
  "created_at": "iso8601"
}
```

### 8.3 Prompt-visible ToolRunSummary

Default replacement includes only:

```text
tool name
target
result
changed files or produced artifacts
exact active blocker excerpt when necessary
artifact handle
byte/token count
bounded rehydrate instructions
```

Default raw preview:

```text
none
```

An exact preview is allowed only when required for the immediate active blocker.

### 8.4 Selective rehydration

Required operations:

```text
search(query, max_matches)
head(lines|bytes)
tail(lines|bytes)
range(start_line, end_line)
around(match_id, context_lines)
metadata
full_payload with explicit deep-dive authorization
```

Normal rehydration must not fetch the full artifact.

### 8.5 Hot-memory rule

The adapter hot state may retain:

```text
artifact id
kind
label
digest
byte count
token estimate
security posture
small search-index metadata
```

It must not retain complete externalized artifact payloads.

Default per-attachment hot artifact metadata budget:

```text
1 MiB
```

---

## 9. Provider-native tactical summarization boundary

### 9.1 No separate summarizer call by default

Focusa must not invoke an additional LLM solely to generate a second compaction summary on the ordinary path.

### 9.2 Canonical preservation context

Focusa supplies:

```text
verified scope
current ask
HLT posture
Workpoint authority
active blocker
constraints affecting the next action
evidence refs
receipt expectations
exact next action
do-not-use boundaries
```

### 9.3 Tactical provider delta

The provider-native compactor may preserve:

```text
changed files
tests run and exact outcomes
failed approaches worth avoiding
unresolved hypotheses
temporary environmental observations
promising next tactical action
```

### 9.4 Forbidden provider-summary authority

The provider-native summary must not:

```text
define or alter HLT
select project scope
promote Workpoint state
claim completion
invent evidence
remove an active blocker
override operator steering
become a receipt
```

### 9.5 Provider summarization failure

```text
1. persist the primary provider error;
2. do not launch overlapping retries;
3. retain the prepared canonical packet and projection;
4. classify current pressure;
5. defer retry when safe;
6. use deterministic bounded fallback when supported;
7. require rollover or stop before emergency exhaustion.
```

---

## 10. CompactionResumeProjectionV1

### 10.1 Schema

```json
{
  "schema": "focusa.compaction_resume_projection.v1",
  "projection_id": "sha256:...",
  "source_compaction_packet_id": "...",
  "semantic_digest": "sha256:...",
  "status": "verified|degraded|blocked",
  "current_ask": "...",
  "scope": {
    "project_root": "...",
    "continuity_id": "...",
    "session_id": "...",
    "scope_status": "verified"
  },
  "trajectory": {
    "hlt": "...",
    "hlt_status": "canonical_explicit",
    "warning": null
  },
  "workpoint": {
    "workpoint_id": "...",
    "mission": "...",
    "next_action": "...",
    "canonical": true
  },
  "active_blocker": null,
  "critical_constraints": [],
  "evidence_refs": [],
  "rehydrate_refs": [],
  "exact_next_tool": "...",
  "do_not_use": [],
  "receipt_expectation": "..."
}
```

### 10.2 Normal render

```text
## Focusa Resume
STATUS: verified
CURRENT_ASK: ...
SCOPE: project_root=... continuity_id=...
HLT: ...
WORKPOINT: ...
NEXT_ACTION: ...
ACTIVE_BLOCKER: none
CRITICAL_CONSTRAINTS: ...
EVIDENCE: ...
REHYDRATE: ...
EXACT_NEXT_TOOL: ...
DO_NOT_USE: ...
```

### 10.3 Global budgets

```text
normal:    900 tokens
pressure:  600 tokens
critical:  400 tokens
blocked:   250 tokens
```

These are total projection budgets. They are not separate budgets for Trajectory, Workpoint, attention, MissionPacket, recent turns, and receipts.

### 10.4 Mandatory order

```text
1. current operator ask
2. scope or authority warning
3. HLT posture
4. Workpoint next action
5. active blocker
6. action-critical constraints
7. evidence refs
8. exact next tool
9. rehydrate refs
10. receipt expectation
```

### 10.5 Budget compiler

Each candidate section carries:

```text
mandatory
priority
relevance
authority importance
freshness
estimated tokens
rehydration cost
```

Algorithm:

```text
1. reserve all mandatory fields;
2. fail closed if mandatory fields cannot fit;
3. rank optional fields by utility per token;
4. add fields while budget remains;
5. shorten individual fields deterministically;
6. emit handles for omitted material;
7. verify final token estimate;
8. persist an omission receipt.
```

Bottom-truncating a fully rendered packet is forbidden.

### 10.6 Prompt exclusions

The normal resume projection must not contain:

```text
full Workpoint JSON
full Trajectory JSON
full CompactionMissionPacket JSON
duplicate schema renderings
general Focusa tool tutorials
end-of-task learning instructions
large recent-turn blocks
raw logs
raw evidence
duplicated authority prose
```

---

## 11. Cache-preserving prompt layout

### 11.1 Stable prefix

May contain:

```text
provider/system/developer policy
Focusa authority laws
stable security boundaries
deterministic tool contracts
stable project identity when unchanged
```

Must not contain:

```text
current ask
recent turns
Workpoint revision
Trajectory revision
current blocker
timestamps
random ids
pressure state
Utility Card output
visible recap state
live WBM context
generated receipts
```

### 11.2 Dynamic tail

Dynamic Focusa context is placed:

```text
after the reusable historical conversation prefix
and adjacent to the newest/current user turn.
```

Where adapter semantics permit, merge the bounded dynamic projection into the newest user message.

The adapter must not prepend a changing Focusa message before all prior conversation history.

### 11.3 History stability

Across adjacent ordinary turns with the same provider, model, branch, and session:

```text
prior historical messages remain byte-identical
prior message ordering remains identical
stable system prefix remains byte-identical
only the current-turn dynamic tail changes
```

### 11.4 Explicit discontinuities

Cache-prefix changes are allowed for:

```text
provider or model change
new native session
branch/fork change affecting serialized history
security-policy change
tool-contract version change
operator-requested cache bust
compaction or rollover where native history necessarily changes
```

Every discontinuity is classified.

### 11.5 Cache telemetry

```text
provider
model
hashed session/cache key
stable_system_prefix_hash
history_prefix_hash
dynamic_slice_hash
dynamic_slice_tokens
input_tokens
cache_read_tokens
cache_write_tokens
estimated_rebilled_tokens
idle_duration
miss_reason
layout_mode
```

### 11.6 Cache-safe degraded mode

Enter `cache_safe_degraded` when:

```text
same provider/model/session
below effective cache TTL
large cache miss by absolute and ratio threshold
two consecutive misses
cache-read tokens plateau near stable-prefix floor
```

In degraded mode include only:

```text
current ask
verified scope
HLT posture
canonical Workpoint next action
active blocker
critical constraints
evidence refs
exact next tool
```

Suppress:

```text
optional Utility Card
recent-turn prose
optional ontology context
historical context
telemetry prose
WBM live prose
noncritical receipts
```

Exit only after measured cache reuse improves, an explicit discontinuity occurs, or the operator changes the mode.

### 11.7 Cache performance gate

For supported provider fixtures, the optimized layout must:

```text
preserve reuse through the prior assistant turn
and reduce re-billed historical tokens by at least 50%
relative to the existing volatile-prefix baseline.
```

No authority or evidence gate may be removed to achieve this result.

---

## 12. Conditional continuation and delivery arbitration

### 12.1 Resume policy

```text
active autonomous work interrupted by compaction:
  immediate continuation may occur.

manual compaction while idle:
  defer projection to next natural turn.

automatic compaction after completed or parked work:
  do not trigger a model turn.

session rollover:
  inject preload/resume projection when target attachment starts.

operator message arrives before automatic continuation:
  operator message wins.
```

### 12.2 ResumeDeliveryArbiter states

```text
none
pending
queued
delivered
deferred_to_next_turn
superseded_by_operator
failed
unknown_completion
```

### 12.3 Delivery key

```text
compaction_resume:<epoch_id>:<target_attachment_id>
```

One key may produce at most one triggered model continuation.

### 12.4 Unknown completion

When the adapter API returns no reliable completion promise:

```text
do not pretend delivery succeeded
do not attach a meaningless Promise.catch
do not blindly retry
record unknown_completion
inject the projection on the next natural turn
```

### 12.5 Non-triggering advisory

A compaction warning that does not require immediate agent execution uses:

```text
UI notification
non-triggering custom message
next-turn dynamic context
```

It must not impersonate an operator-authored user prompt.

### 12.6 Duplicate suppression

Each epoch may render:

```text
at most one visible completion notice
at most one automatic continuation
at most one persisted delivery outcome
```

---

## 13. Persistence and pressure efficiency

### 13.1 Coalesced adapter writer

```text
dirty-domain mark
→ debounce/coalesce
→ final semantic projection
→ digest
→ asynchronous atomic sidecar write
→ bounded native anchor
```

### 13.2 Synchronous flush boundaries

Permitted only for:

```text
before native compaction
hard or emergency pressure
session rollover
process shutdown
risky durable mutation
explicit operator checkpoint
accepted continuity transfer
```

### 13.3 No pre-debounce sidecar churn

The adapter must not write a new sidecar generation before determining that the coalesced revision is ready to anchor.

A recovery boundary forces final flush.

### 13.4 Daemon persistence boundary

The daemon isolates:

```text
whole-state serialization
SQLite writes
WAL/fsync work
checkpoint compaction
```

from Tokio core workers.

### 13.5 Exact native pressure manifest

```text
native_session_bytes
native_entry_count
focusa_custom_bytes
focusa_custom_entries
unique_anchor_count
duplicate_anchor_count
latest_entry_offset
latest_compaction_offset
manifest_revision
```

Counters update incrementally when entries are appended and are periodically verified against file metadata or streaming inspection.

A sampled count is labeled estimated and is not treated as exact global pressure.

### 13.6 Migration semantic selection

The bounded recovery segment prioritizes:

```text
native session header
latest valid native compaction
latest valid Focusa anchor
latest Workpoint checkpoint ref
latest Trajectory checkpoint ref
latest continuity-transfer receipt
bounded meaningful recent turns
active blocker refs
```

A byte-bounded transcript tail alone is insufficient.

---

## 14. Semantic packet and projection reuse

### 14.1 Packet digest

Compaction packet semantic digest excludes:

```text
packet id
generated timestamp
request id
telemetry counters
display-only fields
```

It includes recovery-meaningful fields only.

### 14.2 Reuse rule

```text
if semantic_digest == latest_packet_semantic_digest:
    reuse packet and projection refs
    append no duplicate packet
    record observation telemetry separately
else:
    persist one new semantic packet revision
```

### 14.3 Projection digest

The prompt-facing projection is content-addressed separately.

It is rebuilt only when:

```text
a visible semantic field changes
pressure mode changes its budget
provider layout changes
a required warning changes
```

### 14.4 Stable section digests

```text
scope_section_digest
trajectory_section_digest
workpoint_section_digest
blocker_section_digest
evidence_section_digest
```

Unchanged sections may be reused during compilation.

---

## 15. Effectiveness verification and ROI

### 15.1 Required measurements

```text
tokens_before
tokens_after
context_window
context_release_ratio
projection_tokens
native_compaction_latency
prepare_latency
verify_latency
native_session_bytes_before/after
heap_before/after when meaningful
cache posture
delivery posture
```

### 15.2 Context release

```text
context_release_ratio =
  1 - (tokens_after / tokens_before)
```

A compaction is `ineffective` when:

```text
live context remains above the safe operating threshold
context release is materially below adapter/model expectation
the next call immediately reintroduces avoidable Focusa bulk
```

### 15.3 Verified continuation gate

Before autonomous durable work resumes:

```text
scope remains verified
Workpoint resume remains canonical
HLT posture remains valid or explicitly degraded
active blocker is preserved
exact next action is present
required evidence/receipt posture is present
```

### 15.4 Ineffective response

```text
first ineffective event:
  rebuild minimal projection and inspect layout.

repeated ineffective event:
  disable optional dynamic context and enter cache-safe degraded mode.

hard pressure persists:
  checkpoint and request session rollover.

emergency pressure:
  stop agent loop after minimum recovery record.
```

### 15.5 Productive continuation

Within a bounded evaluation window, record whether the resumed agent:

```text
invoked the correct next tool
advanced the Workpoint
resolved or correctly preserved the blocker
avoided a repeated failed approach
produced accepted evidence
```

### 15.6 ROI metrics

```text
compaction_overhead_input_tokens
compaction_overhead_output_tokens
resume_projection_tokens
context_tokens_removed
historical_tokens_rebilled
historical_tokens_cache_read
model_calls_caused_by_compaction
prepare_latency_ms
verify_latency_ms
time_to_first_productive_action_ms
productive_turns_until_next_compaction
rehydrate_calls
rehydrated_bytes
repeat_error_count
workpoint_revision_delta
task_success
authority_failure_count
```

Derived metrics:

```text
net_token_savings =
    avoided_historical_tokens
  - compaction_overhead_tokens
  - resume_projection_tokens
  - avoidable_rebilled_tokens

productive_efficiency =
    productive_progress_units
  / total_tokens_since_prior_compaction

compaction_break_even_turns =
    compaction_overhead_tokens
  / expected_per_turn_token_savings
```

### 15.7 Adaptation boundary

Focusa may recommend model/adapter-specific trigger changes only after:

```text
a minimum sample count
successful fidelity gates
stable provider/model identity
bounded confidence
```

Default adaptive range:

```text
minimum proactive trigger: 55%
maximum proactive trigger: 80%
hard-pressure threshold: never automatically raised
```

Every adjustment exposes previous value, new value, evidence window, expected improvement, confidence, and rollback condition.

---

## 16. Configuration

```json
{
  "compaction": {
    "coordinator_enabled": true,
    "process_wide_registration_lease": true,
    "predictive_pressure_enabled": true,
    "fixed_micro_compaction_enabled": false,
    "separate_summarizer_call_enabled": false,
    "resume_projection_normal_tokens": 900,
    "resume_projection_pressure_tokens": 600,
    "resume_projection_critical_tokens": 400,
    "resume_projection_blocked_tokens": 250,
    "auto_resume_policy": "active_work_only",
    "retry_max_per_pressure_crossing": 1,
    "retry_cooldown_ms": 60000,
    "prepare_rpc_budget": 1,
    "verify_rpc_budget": 1,
    "cache_safe_degraded_enabled": true,
    "artifact_raw_preview_chars": 0,
    "active_blocker_preview_chars": 400,
    "artifact_hot_metadata_bytes": 1048576,
    "semantic_packet_dedupe": true,
    "exact_native_pressure_manifest": true,
    "roi_telemetry": true,
    "adaptive_trigger_enabled": false
  }
}
```

Safe defaults favor:

```text
one attempt
one projection
no extra model call
no raw preview
no fixed cadence
deferred idle resume
stable cache prefix
complete durable artifacts
```

---

## 17. Implementation phases

### 17.1 P0-A — duplicate-call and delivery safety

Primary surfaces:

```text
apps/pi-extension/src/index.ts
apps/pi-extension/src/auto-compaction.ts
apps/pi-extension/src/compaction.ts
apps/pi-extension/src/session.ts
apps/pi-extension/src/state.ts
```

Required:

```text
process-wide coordinator lease
duplicate extension detection
single epoch state
single native compact call
single completion notice
serialized resume delivery
idle/manual resume deferral
primary error classification
```

### 17.2 P0-B — cache-preserving prompt layout

Primary surfaces:

```text
apps/pi-extension/src/turns.ts
apps/pi-extension/src/state.ts
apps/pi-extension/src/awareness.ts
provider serialization fixtures
cache telemetry routes
```

Required:

```text
remove volatile system-prompt sections
preserve historical message order
attach dynamic Focusa projection to current-turn tail
stable prefix hashing
history prefix hashing
cache-safe degraded mode
cache miss classification
```

### 17.3 P0-C — one bounded resume projection

Primary surfaces:

```text
apps/pi-extension/src/compaction.ts
crates/focusa-api/src/routes/compaction.rs
crates/focusa-core
```

Required:

```text
CompactionResumeProjectionV1
global token-budget compiler
remove expanded JSON prompt renderings
deterministic ordering
semantic projection digest
omission receipts
```

### 17.4 P1-A — unified preparation transaction

```text
POST /v1/compaction/prepare
POST /v1/compaction/verify
one source revision
one durable preparation record
one verify record
parallel independent reads
non-blocking persistence
```

### 17.5 P1-B — content-addressed artifact path

```text
streaming complete storage
one digest-derived handle
metadata-only hot cache
search/head/tail/range rehydration
zero default raw preview
active blocker exception
```

### 17.6 P1-C — pressure prediction and exact accounting

```text
bounded p95 growth predictor
semantic-pressure signals
fixed micro-compaction disabled
exact native counters
hysteresis
ineffective-compaction escalation
```

### 17.7 P2 — ROI-guided adaptation

Required only after P0/P1 proof:

```text
model/adapter result aggregation
break-even analysis
bounded threshold recommendations
optional operator-approved adaptation
long-session outcome comparison
```

No trained self-summarizer is required for closure.

---

## 18. Static tests

```text
spec130a_single_compaction_coordinator_static_test
spec130a_process_wide_registration_lease_static_test
spec130a_one_native_compaction_per_epoch_static_test
spec130a_no_separate_summarizer_default_static_test
spec130a_no_fixed_micro_compact_default_static_test
spec130a_one_resume_projection_static_test
spec130a_no_expanded_packet_json_in_prompt_static_test
spec130a_global_projection_budget_static_test
spec130a_cache_stable_prefix_static_test
spec130a_dynamic_current_turn_tail_static_test
spec130a_history_order_preserved_static_test
spec130a_cache_safe_degraded_static_test
spec130a_no_raw_artifact_hot_memory_static_test
spec130a_complete_content_addressed_artifact_static_test
spec130a_selective_rehydrate_contract_static_test
spec130a_prepare_verify_call_budget_static_test
spec130a_nonblocking_compaction_persistence_static_test
spec130a_semantic_packet_dedupe_static_test
spec130a_exact_native_pressure_manifest_static_test
spec130a_conditional_auto_resume_static_test
spec130a_unknown_delivery_no_blind_retry_static_test
spec130a_compaction_roi_telemetry_static_test
```

---

## 19. Runtime and performance tests

### 19.1 Coordinator

```text
1. Register the same extension twice:
   expect one coordinator and one ctx.compact call.

2. Trigger proactive and hard-pressure paths simultaneously:
   expect one upgraded epoch and one native call.

3. Trigger manual compaction during native compaction:
   expect coalesced or blocked result, not another call.

4. Fail provider summarization:
   expect one primary failure and zero abort-controller cascade.

5. Retry after cooldown:
   expect a linked retry epoch only after prior settlement.
```

### 19.2 Cache

```text
6. Run ten adjacent turns with changing Focusa state:
   expect stable system-prefix hash.

7. Compare serialized request history:
   expect all prior messages byte- and order-stable.

8. Change only current Workpoint next action:
   expect only current-turn dynamic tail to change.

9. Produce two same-model sub-TTL cache misses:
   expect one transition to cache_safe_degraded.

10. Benchmark against existing prefix layout:
    expect at least 50% fewer re-billed historical tokens.
```

### 19.3 Projection

```text
11. Normal projection <= 900 tokens.
12. Pressure projection <= 600 tokens.
13. Critical projection <= 400 tokens.
14. Optional overflow uses deterministic packing, not bottom truncation.
15. Mandatory overflow fails closed.
```

### 19.4 Artifacts

```text
16. Externalize a 100 MB tool result:
    expect bounded process memory and complete artifact digest.

17. Re-store identical output:
    expect same artifact id and no duplicate payload.

18. Rehydrate search/head/tail/range:
    expect bounded returned bytes.

19. Request normal full payload:
    expect rejection or explicit deep-dive requirement.

20. Include an active blocker:
    expect only the exact bounded blocker excerpt in hot prompt.
```

### 19.5 Prepare and verify

```text
21. Prepare under one source revision.
22. Count one prepare and one verify call.
23. Saturate ordinary persistence; recovery record receives priority.
24. No large synchronous serialization/fsync on Tokio workers.
25. Verify measured tokens_after and release ratio.
```

### 19.6 Delivery

```text
26. Manual compaction while idle causes zero automatic model calls.
27. Active autonomous loop compacts and causes one continuation.
28. Operator input arrives first and supersedes automatic continuation.
29. Unknown delivery completion defers to next turn without blind retry.
30. Duplicate completion events produce one visible completion notice.
```

### 19.7 Outcome

```text
31. Resumed coding task invokes correct next tool and advances Workpoint.
32. Failing-test blocker survives without repeating disproven approach.
33. Repeated compaction keeps projection size stable.
34. Live cost comparison proves positive net token savings.
35. Authority and receipt failures remain zero.
```

---

## 20. Initial performance budgets

```text
resume projection build p95:       <= 20 ms
warm local prepare p50:            <= 75 ms
warm local prepare p95:            <= 300 ms
warm verify p95:                   <= 150 ms
normal resume projection:          <= 900 tokens
pressure resume projection:        <= 600 tokens
critical resume projection:        <= 400 tokens
automatic model calls while idle:  0
native compact calls per epoch:     1
prepare calls per epoch:            1
verify calls per epoch:             1
default raw artifact preview:       0 chars
hot artifact payload bytes:         0
duplicate unchanged packet writes:  0
duplicate completion notices:       0
```

A platform may declare a different latency class, but it must preserve the call, token, authority, and duplication budgets.

---

## 21. Acceptance criteria

All Spec 130 acceptance criteria remain mandatory.

Spec 130A additionally requires:

```text
1. One CompactionCoordinator owns every adapter compaction trigger.

2. Duplicate extension discovery cannot create another native compaction call.

3. Each CompactionEpoch invokes native compaction at most once.

4. Provider summarization failure retains one primary error and no secondary
   re-entrancy cascade.

5. The normal path makes no separate summarizer model call.

6. Fixed every-N-turn micro-compaction is disabled by default.

7. Preparation and verification each use at most one normal-path RPC.

8. Preparation captures one coherent source revision.

9. Post-compaction context receives one globally budgeted resume projection.

10. Full Workpoint, Trajectory, or MissionPacket JSON is not injected by default.

11. Mandatory resume content survives every pressure budget.

12. Dynamic Focusa context no longer shifts all prior history behind a changing
    first message.

13. Ordinary adjacent turns retain a byte-stable system prefix.

14. Provider fixtures prove cache reuse through the prior assistant turn.

15. Re-billed historical tokens fall by at least 50% against the existing
    volatile-prefix baseline.

16. Large tool outputs are stored completely under one content-addressed handle.

17. Externalized raw tool payloads consume zero normal hot-prompt bytes and zero
    normal hot process-memory payload bytes.

18. Rehydration supports bounded search and range operations.

19. Manual or idle compaction causes zero unnecessary model turns.

20. Operator input supersedes pending automatic continuation.

21. Unknown message-delivery completion causes no blind retry.

22. Compaction persistence does not block Tokio core workers with large synchronous
    SQLite or whole-state serialization.

23. Unchanged semantic domains produce zero duplicate packet, sidecar, and native
    writes.

24. Native pressure accounting is exact or explicitly integrity-verified.

25. Post-compaction verification proves live context release.

26. Ineffective compaction escalates deterministically toward minimal layout or
    rollover.

27. Compaction ROI telemetry reports overhead, savings, cache behavior, latency,
    productive continuation, and authority failures.

28. Every shipped optimization is net-positive under §0.2.

29. GitHub #11 and #13 cannot close without their corresponding Spec 130A gates.

30. Closure evidence includes live-session token, cache, latency, memory,
    persistence, recovery, and outcome proof.
```

---

## 22. Final operating rule

```text
Compaction must remove more waste than it creates.

One owner.
One epoch.
One preparation transaction.
One native compaction.
One verification transaction.
One bounded resume projection.
One continuation only when useful.

Keep stable context stable.
Put changing context at the current-turn tail.
Store raw payloads once.
Rehydrate only the needed slice.
Write only changed meaning.
Never summarize canonical authority into prose.
Never spend another model call when the native compactor can do the work.
Never trigger an idle model merely to announce recovery.
Never retry an unknown operation blindly.
Never optimize tokens by weakening scope, evidence, blockers, or receipts.

A successful compaction is not merely smaller.

It is cheaper to continue from,
faster to recover from,
safer to act from,
and at least as effective as the context it replaced.
```
