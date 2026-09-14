SPEC:  # Spec 125 V3 — Mandatory Trajectory, Non-Lazy HLT, Pi Bootstrap/Compaction, Receipt/Ontology Interlock

Status: implementation-complete — revalidated 2026-07-12 (16 trajectory tests, 22 Workpoint reconciliation tests, §15.1 static suite, §15.2 isolated runtime suite)
Target file: `docs/125-mandatory-trajectory-nonlazy-hlt-pi-receipt-ontology-interlock-spec.md`
Supersedes: Spec 125 V2 draft
Strong dependencies: Spec 88, Spec 96, Spec 98, Spec 102, Spec 104, Spec 106, Spec 112, Spec 115, Spec 116, Spec 119, ontology docs 58–77, Spec 80.
Primary implementation surfaces: Focusa core, API, CLI, Pi extension, tool contracts, utility cards, compaction packets, session bootstrap, menubar/TUI, receipt ledger, tests, docs.

## 0. V3 correction summary

V3 tightens four points that V2 did not state strongly enough:

```text id="e5rob2"
1. Trajectory is mandatory for Focusa-guided project work.
2. HLT is mandatory and required.
3. Generic trajectory is never fallback and never authority.
4. Fallback trajectory is always the previous explicitly set valid trajectory.
```

V3 also adds:

```text id="ghqjrh"
5. Historic trajectories must be queryable by session through API and CLI.
6. Pi bootstrap and post-compaction trajectory delivery are first-class implementation surfaces.
7. Pi tool cards must loudly communicate missing/generic/fallback trajectory state.
8. Lazy inference is forbidden; intelligent inference is candidate generation with provenance, evidence, and explicit promotion gates.
```

## 1. Final V3 invariant

```text id="x59gx5"
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

## 2. Corrected operating statement

Focusa is not a task manager.

Focusa is not bd.

Focusa is not cloud memory.

Focusa is not a raw transcript keeper.

Focusa is the local-first mission cohesion and proof layer:

```text id="lj87cu"
it scopes the work,
requires a real HLT,
routes the work through Trajectory,
preserves meaning through Focus State,
anchors continuation through Workpoint,
checks authority through Context Authority,
captures evidence,
produces receipts,
validates closure truth,
and projects only bounded safe views outward.
```

## 3. Mandatory Trajectory doctrine

### 3.1 Trajectory is mandatory for Focusa-guided project work

A Focusa-guided project/session/workstream is not fully oriented unless it has:

```text id="i3equ2"
verified scope
canonical or previous-valid HLT
desired end state
current verified state or explicit missing-state warning
MLG/STG/Waypoints or explicit reason they are not yet derivable
Workpoint alignment or explicit Workpoint-missing warning
evidence posture
```

Focusa may still answer simple operator questions without a defined HLT.

Focusa must not treat durable/multi-step/risky project work as fully Focusa-guided when HLT is missing, generic, stale, or conflicted.

### 3.2 HLT is mandatory

HLT is the required project north star.

HLT must be one of:

```text id="yilzz5"
canonical_explicit
previous_valid_fallback
supersession_pending
missing_required
generic_degraded
conflicted
```

Only these count as action-ready route context:

```text id="f7x54m"
canonical_explicit
previous_valid_fallback
```

Even then, `previous_valid_fallback` has limited authority:

```text id="w45otj"
HLT may be reused as the prior valid project north star.
MLG/STG/Waypoints/current_state must be refreshed for the active continuity/session before durable action.
```

### 3.3 Missing HLT is a loud state

If HLT is missing:

```json id="wmhsrm"
{
  "hlt_status": "missing_required",
  "trajectory_required": true,
  "hlt_required": true,
  "canonical": false,
  "degraded": true,
  "action_authority_from_trajectory": false,
  "recommended_action": "restore_previous_valid_hlt_or_define_goal",
  "loud_warning": "HLT_REQUIRED: no valid High-Level Trajectory is set for this verified scope."
}
```

### 3.4 Generic HLT is a louder state

Generic HLT examples:

```text id="go3j5c"
Maintain and improve <project> within verified project scope
Improve <project>
Continue <project> work
Project-level default
Strengthen project intelligence
```

Generic HLT must never be silently injected as route authority.

If generic HLT is present:

```json id="id8w51"
{
  "hlt_status": "generic_degraded",
  "generic_bootstrap": true,
  "trajectory_required": true,
  "hlt_required": true,
  "canonical": false,
  "degraded": true,
  "needs_definition": true,
  "action_authority_from_trajectory": false,
  "recommended_action": "query_hlt_history_then_define_goal",
  "loud_warning": "GENERIC_HLT_DEGRADED: this is a placeholder, not a real project trajectory."
}
```

## 4. Non-lazy intelligent inference contract

### 4.1 Core rule

```text id="s2eh9x"
Trajectory may be intelligently inferred as a candidate.
Trajectory must never be lazily inferred as authority.
```

Intelligent inference means:

```text id="ggh7du"
scope-locked
current-ask aware
evidence-aware
previous-HLT aware
provenance-rich
explicitly labeled
operator-confirmable
noncanonical until promotion
```

Lazy inference means:

```text id="reuiwa"
silently filling HLT from project name
silently filling HLT from Workpoint/current_focus
silently reusing stale ladder lines
silently using transcript tail as route context
silently carrying old session clarity into a new session
silently treating generic bootstrap text as HLT
silently setting operator_confirmed=true for an inferred draft
```

Lazy inference is forbidden.

### 4.2 Field-level inference policy

| Field                    | Candidate inference allowed? | Canonical promotion allowed when                                                          |
| ------------------------ | ---------------------------: | ----------------------------------------------------------------------------------------- |
| HLT                      |          Yes, candidate only | Explicit operator definition, durable project/spec source, or previous valid HLT fallback |
| desired_end_state        |          Yes, candidate only | Operator-defined or evidence-backed durable source                                        |
| MLG                      |                          Yes | Derived from valid HLT + current milestone evidence                                       |
| STG                      |                          Yes | Derived from valid HLT/MLG + current Workpoint/current ask/evidence                       |
| Waypoints                |                          Yes | Derived from valid HLT/MLG/STG + proof requirements                                       |
| current_state            |                          Yes | Evidence-backed or explicitly marked unverified                                           |
| active_gap               |                          Yes | Desired state minus verified current state                                                |
| next Workpoint candidate |                          Yes | Advisory until Workpoint checkpoint/resume accepts it                                     |

### 4.3 HLT cannot be inferred from these alone

The following may support HLT candidate generation, but cannot define canonical HLT by themselves:

```text id="kjpxe0"
Workpoint mission
Workpoint next_slice
Focus State current_focus
Focus Frame title
current task title
provider WorkItem title
bd/Beads issue title
transcript tail
Pi local fallback
project folder name
package name
generic project card
```

### 4.4 Required verified state gate

Before any HLT mutation or HLT-derived MLG/STG/Waypoint mutation, Focusa must verify:

```text id="y8cfgp"
1. typed scope is verified
2. project_root/host scope is safe
3. current_ask or mission exists
4. current_state is present or explicitly missing
5. evidence refs exist OR operator explicitly overrides with reason
6. previous HLT history has been checked
7. candidate is non-generic
8. source/provenance is recorded
```

If the gate fails:

```json id="f91kg9"
{
  "status": "blocked",
  "failure_class": "trajectory_verified_state_gate_failed",
  "canonical": false,
  "degraded": true,
  "active_gap": "missing_verified_state",
  "warning": "Cannot infer or mutate HLT without verified project scope, current ask/mission, and evidence or explicit operator override.",
  "next_tools": [
    "focusa_project_identity",
    "focusa_hlt_history",
    "focusa_trajectory_assess",
    "focusa_trajectory_define_goal"
  ]
}
```

### 4.5 Operator override is not a generic bypass

`operator_confirmed=true` is valid only when the HLT text is:

```text id="k0fq8n"
specific
non-generic
scope-bound
desired-end-state paired
source-recorded
current-state/evidence-aware or explicitly override-reasoned
```

`operator_confirmed=true` must not make a generic HLT canonical.

## 5. Previous-valid trajectory fallback contract

### 5.1 Definition

Fallback trajectory means:

```text id="tmx03c"
the most recent previous explicitly set valid trajectory for the same verified scope family.
```

Fallback trajectory does not mean:

```text id="e3rs4r"
generic bootstrap text
project name default
current task title
transcript summary
local Pi shadow
foreign project trajectory
unverified stale continuity packet
```

### 5.2 Fallback search order

When the active session has no exact valid HLT, Focusa must search:

```text id="l0vb7i"
1. same project_root + same continuity_id + same session_id
2. same project_root + same continuity_id, any session
3. same project_root, most recent previous valid HLT across continuities
4. no fallback available
```

For host scope, replace `project_root` with typed `host_scope_id`.

### 5.3 Fallback authority rules

If fallback is found at level 1:

```text id="a5qp1p"
HLT may be canonical.
MLG/STG/Waypoints/current_state may be canonical only if fresh for the same session and evidence-backed.
```

If fallback is found at level 2:

```text id="dmdeqv"
HLT may be project/workstream canonical.
Session-specific MLG/STG/Waypoints/current_state are advisory until refreshed.
```

If fallback is found at level 3:

```text id="ld2v3f"
HLT may be project-root fallback.
MLG/STG/Waypoints/current_state are advisory.
A visible warning is required.
```

If no fallback is found:

```text id="n9mb73"
HLT is missing_required.
Generic bootstrap is not allowed as fallback.
```

### 5.4 Required fallback payload

```json id="tdbk5b"
{
  "fallback_trajectory": true,
  "fallback_source": "previous_valid_trajectory",
  "fallback_level": "same_session|same_continuity_any_session|same_project_any_continuity|none",
  "fallback_source_trajectory_id": "trajectory:...",
  "fallback_source_session_id": "pi-session-...",
  "fallback_source_continuity_id": "focusa-cont-...",
  "fallback_source_timestamp": "iso8601",
  "fallback_source_hlt": "...",
  "generic_bootstrap": false,
  "loud_warning_required": true
}
```

## 6. Loud warning contract

### 6.1 Trigger states

Loud warning is required when:

```text id="jp8s9m"
HLT missing
HLT generic
HLT stale
HLT conflicted
fallback trajectory loaded
fallback session mismatch
fallback continuity mismatch
current ask scope conflicts with trajectory scope
Workpoint and Trajectory disagree
generic bootstrap HLT is generated
Pi restores lastTrajectoryClarity from prior session
trajectory.define_goal receives generic candidate
trajectory.view returns bootstrap_default=true
```

### 6.2 Warning surfaces

The warning must appear in:

```text id="vfj3p7"
API response warnings[]
tool_result_v1 failure_class or warning_class
CLI human output
CLI JSON output
Pi Utility Card
Pi session bootstrap message
Pi post-compaction auto-resume message
TrajectoryResumePacket
WorkpointResumePacket route context section
Menubar/TUI mission ladder
Receipt preview when relevant
```

### 6.3 Standard warning classes

```text id="fitirn"
HLT_REQUIRED
GENERIC_HLT_DEGRADED
PREVIOUS_TRAJECTORY_FALLBACK
TRAJECTORY_SESSION_MISMATCH
TRAJECTORY_CONTINUITY_MISMATCH
TRAJECTORY_SCOPE_CONFLICT
TRAJECTORY_WORKPOINT_DISAGREEMENT
HLT_SUPERSESSION_REQUIRED
```

### 6.4 CLI loud warning format

```text id="w5ti75"
╔══════════════════════════════════════════════════════════════╗
║  FOCUSA TRAJECTORY WARNING                                  ║
╠══════════════════════════════════════════════════════════════╣
║  HLT is missing/generic/fallback/stale/conflicted.           ║
║  Do not treat this Trajectory as canonical route authority.  ║
║  Next: focusa hlt history --session-id current               ║
║        focusa trajectory define-goal ...                     ║
╚══════════════════════════════════════════════════════════════╝
```

### 6.5 Pi loud warning format

```text id="sycm06"
TRAJECTORY_WARNING:
  class=HLT_REQUIRED|GENERIC_HLT_DEGRADED|PREVIOUS_TRAJECTORY_FALLBACK
  project_root=<...>
  continuity_id=<...>
  session_id=<...>
  hlt_status=<...>
  action_authority_from_trajectory=false
  required_next=focusa_hlt_history -> focusa_trajectory_define_goal
END_TRAJECTORY_WARNING
```

## 7. Historic trajectory by session: API and CLI

### 7.1 Current gap

The HLT ledger entries already carry:

```text id="q4ngw4"
project_root
continuity_id
session_id
old_hlt
new_hlt
source
reason
evidence_refs
lamport_ts
timestamp
```

But current history query accepts only:

```text id="jm97pi"
project_root
continuity_id
limit
```

V3 requires session filtering.

### 7.2 API requirement

Add:

```http id="nsupqx"
GET /v1/hlt/history?project_root=<path>&continuity_id=<id>&session_id=<id>&limit=50
```

Supported query params:

```text id="o67scv"
project_root: required unless host scope is used
scope_kind: project|host
scope_id: optional typed scope id
continuity_id: optional filter
session_id: optional filter
include_cross_session_fallbacks: false by default
include_generic: false by default
limit: default 50, max 500
```

### 7.3 API response shape

```json id="w8f2bj"
{
  "status": "completed",
  "project_root": "/path",
  "continuity_id": "focusa-cont-...",
  "session_id": "pi-session-...",
  "count": 3,
  "entries": [],
  "fallback_candidates": [],
  "latest_valid_for_session": null,
  "latest_valid_for_continuity": null,
  "latest_valid_for_project": null,
  "warnings": [],
  "ledger_file": "..."
}
```

### 7.4 CLI requirement

Add to current HLT CLI:

```bash id="vqmftz"
focusa hlt history \
  --project-root /path/to/project \
  --continuity-id focusa-cont-... \
  --session-id pi-session-... \
  --limit 20
```

Also support:

```bash id="oc2s3w"
focusa hlt history --project-root /path --session-id current
focusa trajectory history --project-root /path --session-id current
focusa hlt sessions --project-root /path
focusa hlt fallback --project-root /path --continuity-id <id> --session-id current
```

### 7.5 CLI output requirements

Human output must show:

```text id="qalqbe"
Project
Continuity
Session
Latest valid HLT for exact session
Latest valid HLT for continuity
Latest valid HLT for project
Whether fallback would be exact, cross-session, cross-continuity, or unavailable
Whether any generic HLT was skipped
```

### 7.6 Session alias resolution

`--session-id current` resolves to:

```text id="zckm6j"
Pi sessionFrameKey when called from Pi
current Focusa session id when available
local CLI profile session id when configured
else returns session_id_required_or_unknown
```

No command may silently treat `current` as “any session.”

## 8. Pi session bootstrap contract

### 8.1 Bootstrap order

On Pi `session_start` and `session_switch`, Focusa must run this route:

```text id="k1w2n7"
1. bind/verify project_root or host scope
2. resolve continuity_id
3. query HLT history for current session
4. query HLT history for current continuity
5. query latest valid project HLT fallback
6. call trajectory_view
7. classify HLT state
8. loudly warn if HLT missing/generic/fallback
9. prompt only if prior valid fallback is unavailable or insufficient
10. refresh Workpoint
11. inject Utility Card with explicit HLT status
```

### 8.2 Pi must not lazily define HLT

Pi must not send `trajectory/define-goal` with `operator_confirmed=true` from generated draft options unless the operator explicitly chooses or edits a non-generic candidate and the body includes:

```text id="n9o5qw"
current_ask or mission
current_state or explicit missing-current-state override
desired_end_state
non-generic HLT
source/provenance
session_identity
project_root
continuity_id
session_id
```

### 8.3 Draft options must be relabeled

Current Pi draft options such as “Project-level default” or “Infer from current task” must be treated as candidate scaffolds only.

Required labels:

```text id="xihcld"
A) Candidate from project evidence — requires confirmation
B) Candidate from current ask — requires confirmation
C) Candidate from Workpoint gap — cannot define HLT alone
D) Restore previous valid HLT — preferred when available
E) Custom HLT / desired end state
F) Skip — leaves HLT_REQUIRED warning active
```

Forbidden label behavior:

```text id="uj2y7o"
Do not present generic defaults as safe.
Do not set operator_confirmed=true just because a draft was selected.
Do not suppress trajectory prompt merely because generic or stale fallback exists.
```

### 8.4 Previous valid fallback at bootstrap

If a previous valid HLT exists:

```text id="pzvps8"
Pi may load it as previous_valid_fallback.
Pi must display a loud warning if session or continuity differs.
Pi must require refresh of MLG/STG/Waypoints/current_state for current session.
Pi must not ask the operator to redefine HLT unless fallback is conflicted or superseded.
```

### 8.5 Bootstrap utility card must expose HLT state

The Utility Card MISSION_PACKET must include:

```text id="yo2g6o"
trajectory_hlt_status=canonical_explicit|previous_valid_fallback|missing_required|generic_degraded|conflicted
hlt_source=operator|trajectory_define_goal|previous_valid_trajectory|generic_bootstrap|none
hlt_required=true|false
generic_bootstrap=true|false
fallback_level=same_session|same_continuity_any_session|same_project_any_continuity|none
action_authority_from_trajectory=true|false
```

## 9. Pi post-compaction contract

### 9.1 Pre-compaction

Before compaction, Pi must attempt:

```text id="nztzds"
1. Focus State delta push
2. Workpoint checkpoint
3. Trajectory checkpoint
4. HLT history query for exact session
5. Trajectory resume
6. Workpoint resume
7. Persist authoritative state
```

If HLT is missing/generic:

```text id="b9ri9p"
Compaction must still preserve Workpoint.
TrajectoryResumePacket must include loud warning.
Post-compaction message must tell the model to restore/define HLT before durable multi-step work.
```

### 9.2 Post-compaction

The auto-resume message must include:

```text id="ltt16t"
AttentionRecallVerdict
WorkpointResumePacketV2
TrajectoryResumePacketV3
Trajectory warning if any
Exact next tool
Do-not-use list
Receipt/evidence expectations
```

### 9.3 TrajectoryResumePacketV3

Replace or extend current TrajectoryResumePacket with:

```json id="ewqw4m"
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

### 9.4 Format rules for injected prompt

The first visible lines of the post-compaction Trajectory section must be:

```text id="y6f760"
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

### 9.5 LastTrajectoryClarity cannot silently backfill HLT

Current Pi behavior may preserve previous `lastTrajectoryClarity` fields when a fresh trajectory response omits them.

V3 rule:

```text id="l52h1r"
Pi may backfill HLT only from a validated previous_valid_trajectory record.
Pi may not backfill HLT from unproven local memory, generic default, stale packet, or mismatched session.
Every backfill must set fallback.used=true and render a loud warning unless same exact session+continuity.
```

## 10. Tool card / Utility Card V3 contract

### 10.1 MISSION_PACKET additions

Utility Card must include:

```text id="xwyevb"
- trajectory_hlt_status=<state>
- hlt_required=true
- generic_bootstrap=<true|false>
- fallback=<none|previous_valid_trajectory>
- fallback_level=<...>
- hlt_source=<...>
- action_authority_from_trajectory=<true|false>
```

### 10.2 NOW_CARD additions

```text id="rqn5b1"
- exact_next_action=focusa_hlt_history when HLT is missing/generic/fallback-stale
- exact_next_action=focusa_trajectory_define_goal when no previous valid HLT exists
- exact_next_action=focusa_trajectory_assess when HLT exists but current_state/gap is missing
```

### 10.3 WHY_CARD additions

```text id="t5qytm"
- why=Trajectory included because HLT is required route context.
- excluded=generic bootstrap as authority
- excluded=transcript tail as trajectory source
- excluded=stale session trajectory unless previous_valid_fallback warning is rendered
```

### 10.4 HEALTH_CARD additions

```text id="gkuggo"
- hlt=canonical|previous_valid_fallback|missing_required|generic_degraded|conflicted
- trajectory_clarity=clear|provisional|unclear|conflicted
- fallback_warning=none|session_mismatch|continuity_mismatch|project_fallback
```

### 10.5 DO_CARD additions

```text id="ynfy1k"
- if hlt missing/generic: focusa_hlt_history -> focusa_trajectory_define_goal
- if previous fallback: focusa_trajectory_assess -> focusa_workpoint_resume/checkpoint
- if exact canonical HLT: focusa_workpoint_resume or active object execution path
```

## 11. API implementation requirements

### 11.1 `/v1/trajectory/view`

Must return:

```json id="wgxpwh"
{
  "trajectory_required": true,
  "hlt_required": true,
  "hlt_status": "canonical_explicit|previous_valid_fallback|missing_required|generic_degraded|conflicted",
  "generic_bootstrap": false,
  "loud_warning_required": false,
  "previous_valid_fallback": {},
  "action_authority_from_trajectory": true,
  "warnings": []
}
```

If generic HLT exists, the response must not make it look like canonical `long_term_goal`.

Preferred shape:

```json id="h9gjuh"
{
  "trajectory": {
    "long_term_goal": null,
    "generic_placeholder": "Maintain and improve ...",
    "hlt_status": "generic_degraded",
    "needs_definition": true
  }
}
```

### 11.2 `/v1/trajectory/define-goal`

Must reject canonical mutation when:

```text id="tb6d4f"
project_root missing
continuity_id missing
scope unsafe
HLT generic
desired_end_state missing
current_ask/mission missing
current_state missing without explicit override reason
evidence missing without explicit operator override reason
```

`operator_confirmed=true` is not enough if the HLT is generic.

### 11.3 `/v1/trajectory/resume`

Must:

```text id="k68v94"
check current ask scope conflict
query previous valid trajectory fallback before generic bootstrap
return TrajectoryResumePacketV3
include loud warning state
include Workpoint reconciliation hint
never use generic as fallback
```

### 11.4 `/v1/hlt/history`

Must support:

```text id="qlemv5"
session_id filter
include_cross_session_fallbacks flag
include_generic flag
fallback candidate computation
latest_valid_for_session
latest_valid_for_continuity
latest_valid_for_project
```

### 11.5 Rename confusing fallback fields

Current names such as:

```text id="o7s7f8"
allow_prior_project_trajectory
fallback_prior_project_trajectory
```

are ambiguous.

V3 should introduce clearer fields:

```text id="kvmr11"
allow_previous_valid_trajectory
previous_valid_trajectory_fallback
fallback_level
fallback_source_scope
```

Foreign project fallback must remain impossible as canonical route context.

## 12. CLI implementation requirements

### 12.1 Add session filter

Current command:

```bash id="gepjra"
focusa hlt history --project-root /path --continuity-id <id>
```

Required command:

```bash id="komqs7"
focusa hlt history \
  --project-root /path \
  --continuity-id <id> \
  --session-id <session-id|current> \
  --limit 20
```

### 12.2 Add fallback command

```bash id="t4bm3e"
focusa hlt fallback \
  --project-root /path \
  --continuity-id <id> \
  --session-id current
```

Returns:

```text id="zrd4g0"
exact session HLT
same continuity HLT
project-level HLT
generic entries skipped
selected fallback
fallback warning
next command
```

### 12.3 Add sessions list

```bash id="dxif5k"
focusa hlt sessions --project-root /path
```

Returns:

```text id="xz9zoo"
session_id
continuity_id
entry_count
latest_valid_hlt
latest_timestamp
generic_count
fallback_eligible
```

### 12.4 CLI define-goal guard

`focusa hlt set` and `focusa trajectory define-goal` must reject generic HLT even with `--confirm`.

Required extra flags when evidence is missing:

```bash id="c92h8k"
--operator-override-reason "..."
--current-state "..."
```

or:

```bash id="qlqor9"
--evidence-ref <ref>
```

## 13. Pi implementation requirements

### 13.1 Files requiring V3 changes

```text id="vy950y"
apps/pi-extension/src/session.ts
apps/pi-extension/src/compaction.ts
apps/pi-extension/src/awareness.ts
apps/pi-extension/src/state.ts
apps/pi-extension/src/tool-contracts.ts
docs/current/FOCUSA_AGENT_UTILITY_CARD.md
docs/focusa-tools/tools/focusa_trajectory_view.md
docs/focusa-tools/tools/focusa_trajectory_define_goal.md
docs/focusa-tools/tools/focusa_trajectory_resume.md
docs/focusa-tools/tools/focusa_hlt_history.md
```

### 13.2 `session.ts`

Required changes:

```text id="icy8be"
- Replace generic trajectoryDraftOptions with noncanonical candidate scaffolds.
- Add previous-valid HLT query before prompt.
- Add session_id to hlt history query.
- Do not suppress prompt unless previous fallback is valid and warning is rendered.
- Require explicit operator edit/confirm for candidate HLT.
- Require current_state/evidence or explicit override reason for define-goal.
- Emit high-priority warning when HLT is missing/generic/fallback.
```

### 13.3 `compaction.ts`

Required changes:

```text id="mgwuhc"
- Pre-compaction must query HLT history by session.
- TrajectoryResumePacket must be V3.
- formatTrajectoryPacketForPrompt must include HLT_STATUS, HLT_REQUIRED, GENERIC_BOOTSTRAP, FALLBACK_SOURCE, LOUD_WARNING.
- Do not backfill HLT from lastTrajectoryClarity unless provenance proves previous_valid_trajectory.
- Post-compaction steer message must put trajectory warning above ordinary route guidance.
```

### 13.4 `awareness.ts`

Required changes:

```text id="ayqqb0"
- Utility Card must distinguish canonical HLT, previous-valid fallback, generic degraded, missing required, and conflicted.
- MISSION_PACKET must expose HLT status.
- RECONCILIATION_ENVELOPE must list trajectory warning as a blocked/stale surface.
- DO_CARD must route to hlt_history/define_goal/assess depending HLT status.
```

### 13.5 `state.ts`

Required changes:

```text id="fd2j45"
- lastTrajectoryClarity must carry provenance fields.
- lastTrajectoryClarity must not be treated as canonical unless scope/session/provenance match.
- Store previous_valid_trajectory fallback metadata separately from current exact-session clarity.
- Reset session-scoped trajectory state on session boundary unless fallback is explicitly revalidated.
```

### 13.6 `tool-contracts.ts`

Required changes:

```text id="m26vxz"
- Add focusa_hlt_history as a trajectory family tool contract if missing from Pi-visible catalog.
- Mark focusa_trajectory_define_goal as canonical mutation only when verified gate passes; otherwise advisory candidate.
- Update focusa_trajectory_view purpose to say HLT is required.
- Update focusa_trajectory_resume purpose to mention loud warning and previous-valid fallback.
```

## 14. Receipt / ontology integration

### 14.1 Receipt must include HLT posture

Every relevant receipt must include:

```json id="k54deg"
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

### 14.2 Completion claims require HLT state

A `final_report`, `work_session`, `work_item_closure`, `install_verification`, `risky_mutation`, or `public_proof_snapshot` receipt must not claim full completion if:

```text id="g8vyiz"
HLT is missing_required
HLT is generic_degraded
Trajectory and Workpoint conflict
current_state is missing and not explicitly accepted
evidence is partial/surrogate/missing
```

### 14.3 Ontology mapping

HLT maps into:

```text id="x3fps6"
Mission / Goal hierarchy
CurrentAsk / QueryScope boundary
GoverningPrior mission band
Projection/View route frame
Workpoint ActiveMissionSet
Receipt trajectory frame
```

Generic/missing/fallback warning maps into:

```text id="rjg8yf"
ScopeFailure
OpenLoop
Risk
Blocker
Precondition
Verification
ProjectionBoundary
```

## 15. Test requirements

### 15.1 Required static tests

```text id="gc6chd"
spec125_generic_hlt_loud_warning_static_test
spec125_hlt_required_no_lazy_inference_static_test
spec125_previous_valid_fallback_static_test
spec125_hlt_history_session_filter_static_test
spec125_pi_bootstrap_hlt_required_static_test
spec125_pi_compaction_trajectory_packet_v3_static_test
spec125_utility_card_hlt_status_static_test
spec125_define_goal_rejects_generic_static_test
spec125_no_mlg_stg_from_generic_hlt_static_test
spec125_last_trajectory_clarity_provenance_static_test
```

### 15.2 Required runtime/eval tests

```text id="t8249z"
1. Start Pi session with no HLT:
   expect HLT_REQUIRED warning and prompt/history route.

2. Start Pi session with generic bootstrap only:
   expect GENERIC_HLT_DEGRADED warning and no canonical trajectory.

3. Start Pi session with previous valid HLT in same project:
   expect previous_valid_fallback loaded with warning when session differs.

4. Run compaction with valid HLT:
   expect TrajectoryResumePacketV3 injected with HLT_STATUS canonical_explicit.

5. Run compaction with missing HLT:
   expect TrajectoryResumePacketV3 injected with loud warning and next tool hlt_history/define_goal.

6. Query hlt history by session:
   expect entries filtered by session_id.

7. define-goal with generic HLT and --confirm:
   expect validation_rejected.

8. define-goal with explicit HLT, desired state, current state, operator confirmation:
   expect persisted and ledger entry with session_id.

9. Workpoint/current_focus exists but HLT invalid:
   expect no MLG/STG population from Workpoint/current_focus.

10. Utility Card with fallback HLT:
   expect trajectory_hlt_status=previous_valid_fallback and refresh/assess next tool.
```

## 16. Acceptance criteria

Spec 125 V3 is accepted only when:

```text id="h1t3s8"
1. HLT is mandatory in all active project/host work packets.
2. Generic HLT never becomes canonical.
3. Generic HLT always triggers loud warning.
4. Missing HLT always triggers loud warning.
5. Previous valid HLT is the only fallback trajectory source.
6. Fallback source and level are visible in API/CLI/Pi/UI.
7. HLT history is queryable by session through API.
8. HLT history is queryable by session through CLI.
9. Pi session bootstrap queries or restores trajectory before presenting work as fully oriented.
10. Pi session bootstrap does not silently set generic HLT.
11. Pi post-compaction injects TrajectoryResumePacketV3.
12. Pi post-compaction puts HLT warning above ordinary guidance when needed.
13. Utility Card exposes trajectory_hlt_status.
14. Workpoint/current_focus cannot populate MLG/STG when HLT is invalid/generic.
15. define-goal rejects generic HLT even with operator_confirmed.
16. define-goal requires explicit current_state/evidence or override reason.
17. lastTrajectoryClarity cannot silently backfill HLT without previous-valid provenance.
18. Receipt trajectory frame includes HLT status and fallback state.
19. Provider closure receipts cannot claim completion from missing/generic trajectory when trajectory is relevant.
20. All tests in §15 pass.
```

## 17. Final V3 operating rule

```text id="whgux7"
Never invent the north star.
Never hide that the north star is missing.
Never call generic text a trajectory.
Never treat fallback as fresh session truth.
Always query previous valid trajectory before asking for a new HLT.
Always show the HLT status before durable work.
Always make Pi carry Trajectory through bootstrap and compaction.
Always prove completion with Evidence and Receipts.
```
