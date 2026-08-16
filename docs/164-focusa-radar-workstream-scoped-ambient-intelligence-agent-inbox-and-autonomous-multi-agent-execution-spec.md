# Spec 164 — Focusa Radar: Workstream-Scoped Ambient Intelligence, Agent Inbox, and Autonomous Multi-Agent Execution

**Status:** Proposed normative product and architecture specification  
**Spec number:** 164  
**Coordination issue:** `Startempire-Wire/focusa#130`  
**Normative product name:** **Focusa Radar**  
**Date:** 2026-08-04  
**Primary repository:** `Startempire-Wire/focusa`  
**Companion repository:** `WPUIAI/uiai-engine`  
**Companion coordination issue:** `WPUIAI/uiai-engine#43`  
**Foundation dependency:** Focusa Spec 158 / `Startempire-Wire/focusa#125`  
**Primary surfaces:** Focusa menubar, Focusa Desktop, Pi Work Surface, Focusa.work, UIAI Engine Cockpit  
**External implementation reference:** `Einsia/OpenChronicle`

> **Numbering decision:** Spec 164 is reserved for Focusa Radar after a repository collision check found no claimed Spec 159–164. Specs 159–163 remain intentionally unclaimed as the post-Spec-158 safety buffer.

---

## 0. Executive decision

Focusa SHALL gain **Focusa Radar**, a Workstream-scoped ambient intelligence and proactive execution system that can:

1. observe approved local and connected work context;
2. normalize observations into bounded Work Episodes;
3. detect unresolved work, risk, drift, commitments, repeated friction, and opportunities;
4. submit typed proactive proposals to the exact Workstream reducer;
5. deliver admitted work through a durable Agent Inbox;
6. wake and route eligible local, remote, scheduled, Pi, and UIAI agents;
7. admit agent-proposed execution graphs as canonical Workpoints;
8. supervise execution through the Workstream Work Loop;
9. require Evidence and Receipts before completion settlement; and
10. present notices, approvals, blocks, progress, and verified outcomes to the operator.

The normative naming hierarchy is:

- **Focusa Radar** — the complete market-facing product capability.
- **Radar Engine** — observation intake, Work Episode construction, detection, scoring, and policy evaluation.
- **Focusa Agent Inbox** — durable typed Handoffs, routing, claims, leases, thread events, graph proposals, results, and recovery.
- **Focusa Proactive Inbox** — the human-facing projection for notices, approvals, active work, blocks, verification, and outcomes.
- **Buzz Bridge** — optional conversational projection over typed Handoff threads; never canonical work state.

The governing loop is:

```text
Observe approved context
→ normalize, redact, and deduplicate
→ resolve exact Scope + Workstream or quarantine
→ construct a bounded Work Episode
→ detect a proactive signal
→ submit a typed proposal
→ Focusa reducer admits, rejects, merges, or downgrades meaning
→ create/update Workpoints and offer Handoffs
→ eligible agents claim bounded delivery responsibility
→ agents propose dependency graphs
→ Focusa admits graph nodes as Workpoints
→ Work Loop schedules governed execution
→ UIAI and other tools return actions, artifacts, Evidence, and Receipts
→ Focusa verifies and settles
→ Proactive Inbox, menubar, Pi, Focusa.work, and optional Buzz update
```

Market promise:

> **Stop remembering to ask AI.**

Architectural promise:

> **Focusa may notice and dispatch work, but only the exact Workstream reducer may canonize what the work means, who may execute it, and what proves completion.**

---

## 1. Spec 158 foundation

Spec 164 is subordinate to Spec 158.

### 1.1 Canonical identity

No proactive signal, Handoff, inbox item, graph, claim, approval, result, Evidence relationship, or completion claim may exist as daemon-global cognition.

Every canonical-capable object or operation SHALL carry or resolve:

```text
ScopeRef / ProjectRootKey
WorkstreamId
actor identity
causal and idempotency metadata
AttachmentKey where runtime binding matters
```

`ContinuityId` is lineage inside a Workstream. It is not Workstream identity.

Session, Instance, CWD, project path, selected tab, selected menu item, active window, current/last/latest state, or visual focus cannot authorize canonical work.

A global Proactive Inbox is an aggregate read model only. Every mutation from it resolves and echoes one exact Workstream before execution.

### 1.2 Agent Inbox is not a second Work Loop

Agent Inbox is a durable delivery and coordination plane. It SHALL NOT create a competing task authority.

- Handoffs reference or propose Workpoints.
- Agent graphs are proposals until reducer admission.
- Admitted graph nodes become canonical Workpoints and dependencies.
- Work Loop retains scheduling, budgets, retries, blockers, pause state, writer leases, fencing, and settlement authority.
- An inbox claim lease reserves responsibility for processing a Handoff; it does not grant repository, browser, OS, deployment, communication, or canonical mutation authority.

### 1.3 Focusa and UIAI remain separate authorities

Focusa owns:

- verified Project/Scope and Workstream identity;
- Focus Stack and Focus State;
- Workpoints, dependencies, Trajectory, and Work Loop;
- proactive signal admission;
- Agent Inbox canonical state;
- budgets, authority, completion, Evidence association, and Focusa Receipts.

UIAI Engine owns:

- browser and approved OS observation and actuation;
- UIAI sessions, contexts, targets, documents, navigations, frames, actions, and visual work objects;
- accessibility snapshots, visual snapshots, diagnostics, renders, artifacts, UIAI Receipts, and Evidence proposals.

Cockpit owns UIAI presentation and steering. It never becomes the Focusa reducer.

---

## 2. Current-state and transition constraints

### 2.1 Focusa

Spec 158 and the Focusa Desktop transition are architectural and documentation work, not completed production migration. Agent Inbox SHALL NOT be attached to the existing mixed daemon-global cognition snapshot or implemented beside it as permanent dual authority.

The existing Rust `focusa` CLI remains the product CLI foundation, but legacy Thread terminology and commands whose APIs infer global/current state must migrate to exact Workstream routing.

### 2.2 Focusa Desktop and menubar

Focusa Desktop is the primary rich local environment for Focusa cognition, Mission Canvas, Pi, Workpoints, Evidence, and governed work.

The menubar remains a compact quick-entry, lifecycle, status, privacy, approval, and emergency-control surface. It SHALL NOT become another full Mission Canvas or a separate Radar implementation.

### 2.3 UIAI Cockpit

The current Cockpit is still the Slice 0 shell: static scope chips, primitive navigator, route slot, placeholder Inspector, process ribbon, and Phase 0 card grid. It has no completed Work Surface store, tab/pane/window system, generated Focusa Workstream contract consumption, or universal semantic control plane.

UIAI-COCKPIT-003, 004, and 005 define the planned navigation, exact Work Surfaces, and universal agent-control architecture. Spec 164 and UIAI-COCKPIT-006 SHALL consume those plans rather than create a disconnected UI or authority.

The current all-optional Cockpit `ScopeRef` and any project-path-plus-continuity binding are insufficient. Generated Spec 158 Workstream and Attachment contracts govern.

### 2.4 UIAI CLI

The current `scripts/uiai` Bash/curl/jq wrapper is useful development tooling but is not the final registry-driven agent CLI. UIAI SHALL gain a compiled Go CLI sharing schemas, capability IDs, guards, errors, Receipts, and parity fixtures with the server and Cockpit. The script becomes a thin compatibility launcher with no unique business logic or legacy Focusa scope authority.

### 2.5 OpenChronicle

OpenChronicle is an implementation reference for accessibility-first capture, event-driven collection, debounce, deduplication, timeline blocks, episode/session cutting, inspectable local storage, and model-neutral access.

It SHALL NOT own Focusa memory, Workstream identity, proactive meaning, Agent Inbox state, or action authority. Any integration is an adapter producing provenance-bearing observation candidates.

---

## 3. Product topology

```text
Focusa menubar
  Radar state · source/privacy posture · quick approvals · emergency stop

Focusa Desktop
  Proactive Inbox · Agent Inbox · Radar timeline · Workpoint graph
  approvals · execution supervision · Evidence · Receipts

Pi Work Surface
  inbox watch/pull/claim · planning · graph proposal · execution
  steering · result/Evidence submission · headless fallback

Focusa daemon
  Workstream reducer · policies · Agent Inbox · Workpoints · Work Loop
  routing · event streams · Receipts · replay · recovery

Focusa.work
  remote/mobile projection · approvals · monitoring · results

UIAI Engine
  observation · browser/OS actuation · semantic UI state
  verification · artifacts · UIAI Receipts · Evidence proposals

UIAI Engine Cockpit
  Live/Activity/Evidence/Capabilities projections
  exact Handoff, Workpoint, Episode, target, and verification Work Surfaces

Buzz Bridge
  optional human-agent conversation projection over typed Handoff threads
```

No additional standalone desktop application is introduced.

---

## 4. Ownership and state placement

### 4.1 Workstream-owned

A Workstream may own:

```text
ProactiveSignalState
ProactivePolicyBinding
AgentInboxState
Handoffs and typed thread events
claim/execution associations
Workpoint graph and dependency projections
Work Loop scheduling/writer authority
Evidence and Receipt relationships
proactive outcome history
```

### 4.2 Project-owned

Project Scope may own explicitly shared policy:

- approved observation sources and application allow/deny lists;
- data classifications, redaction, retention, and egress policy;
- agent allowlist, trust floor, default budgets, and autonomy ceilings;
- business hours and interruption posture.

Project policy does not create project-wide mutation authority. Every Handoff still targets one Workstream.

### 4.3 Daemon infrastructure

Daemon infrastructure may own agent presence, transport connections, capability catalogs, host resources, SSE cursors, wake timers, cloud clients, and non-cognitive telemetry.

It SHALL NOT own a global cognitive inbox or global current Workstream.

### 4.4 UIAI-owned

UIAI owns mutable Observation Objects, Episode artifacts, accessibility/visual snapshots, browser/session/target refs, action and verification Receipts, artifacts, and Evidence candidates.

A UIAI object may be unbound, exactly Workstream-bound, Workpoint-bound, detached, or frozen as an immutable Evidence candidate.

---

## 5. Core contracts

### 5.1 Observation candidate

```json
{
  "schema": "focusa.observation_candidate.v1",
  "observation_id": "obs_...",
  "source": {
    "authority": "uiai|focusa|git|terminal|agent|calendar|connector",
    "source_ref": "...",
    "adapter_id": "...",
    "captured_at": "..."
  },
  "binding": {
    "scope_ref": {"project_root_key": "prk_..."},
    "workstream_id": "ws_...",
    "attachment_ref": "attachment:..."
  },
  "classification": "terminal_error|browser_state|agent_loop|commitment|workflow_pattern|other",
  "content_digest": "sha256:...",
  "summary": "Repeated authentication callback failure",
  "structured_refs": [],
  "visual_refs": [],
  "trust": {
    "untrusted_content": true,
    "redaction_applied": true,
    "local_only": true
  }
}
```

Observation candidates are not canonical meaning. Missing or ambiguous binding enters quarantine and cannot trigger mutation.

### 5.2 Work Episode

A Work Episode groups observations sharing exact Workstream, temporal proximity, task/object relationship, and provenance. It carries observation refs/digests, participants, tools/objects, bounded summary, confidence, ambiguity, supersession, and untrusted-content posture.

Episode construction SHALL be inspectable, bounded, reversible, and deterministic for the same ordered inputs and configuration.

### 5.3 Proactive signal proposal

Required initial signal kinds:

```text
stuckness
repeated_failure
agent_loop
interrupted_work
unclosed_commitment
mission_drift
missing_verification
repeated_manual_workflow
context_recovery
emergent_opportunity
risk_or_policy_exception
```

A proposal carries exact Workstream, source Episode refs, confidence, expected value, urgency, interruption cost, risk, privacy cost, recommended autonomy level, and proposed Workpoint/Handoff action.

The reducer may reject, merge, supersede, downgrade, quarantine, or admit it.

### 5.4 Handoff

```json
{
  "schema": "focusa.handoff.v1",
  "handoff_id": "hnd_...",
  "revision": 1,
  "workstream": {
    "scope_ref": {"project_root_key": "prk_..."},
    "workstream_id": "ws_..."
  },
  "continuity_id": "cont_...",
  "workpoint_ref": "workpoint:wp_...",
  "parent_handoff_ref": null,
  "kind": "offer|assignment|decomposition_request|verification_request|context_request|approval_request|result|escalation",
  "source": {"actor_type": "radar|operator|agent|pi|service|uiai|buzz_bridge", "actor_id": "..."},
  "target_selector": {
    "agent_ids": [],
    "roles": ["code_investigator"],
    "capability_ids": ["repository.read", "tests.run"],
    "execution_domains": ["local", "focusa_cloud"],
    "trust_floor": "operator_owned"
  },
  "authority_grant_ref": "grant:...",
  "context_refs": [],
  "acceptance_criteria_refs": [],
  "budget": {"max_cost_usd": 5, "max_runtime_seconds": 2700, "max_attempts": 2},
  "scheduling": {"not_before": null, "expires_at": "...", "priority": "normal"},
  "claim_policy": "single|parallel_quorum|broadcast_advisory",
  "state": "offered",
  "created_event_head": "event:...",
  "reply_thread_ref": "handoff-thread:..."
}
```

Large context remains in referenced stores. Handoffs carry bounded summaries, constraints, authority references, acceptance criteria, and stable refs.

### 5.5 Handoff lifecycle

```text
proposed
→ admitted
→ offered
→ claimed
→ planning
→ executing
→ blocked | awaiting_approval | awaiting_verification
→ settled | rejected | cancelled | expired | quarantined
```

Handoff settlement does not automatically settle the parent Workpoint. The reducer evaluates results and Evidence against acceptance criteria.

### 5.6 Typed thread events

Required event family:

```text
handoff.offered
handoff.claimed
handoff.heartbeat
handoff.released
handoff.question
handoff.answer
handoff.context_requested
handoff.context_added
handoff.graph_proposed
handoff.graph_admitted
handoff.authority_requested
handoff.authority_changed
handoff.blocked
handoff.result_submitted
handoff.verification_submitted
handoff.settled
handoff.cancelled
```

Free-form text may accompany a typed event as supporting evidence. It cannot replace the typed transition.

---

## 6. Agent directory, routing, and claims

### 6.1 Agent descriptor

Agents register infrastructure descriptors containing stable agent identity, roles, capability IDs, execution domain, trust class, availability mode, supported protocols, cost/resource profiles, and public identity reference.

Registration does not attach an agent to every Workstream.

### 6.2 Routing

Routing evaluates:

- exact Project/Workstream eligibility;
- required capabilities and roles;
- data locality and sensitivity;
- local, remote, hosted, or UIAI execution domain;
- trust, entitlement, availability, workload, model/tool suitability, and cost;
- continuity/cache locality;
- verifier independence;
- writer leases and conflicts.

### 6.3 Claims and leases

A claim uses atomic compare-and-swap and returns an expiring claim lease. Heartbeats renew it within policy.

A claim means the agent is responsible for planning or returning the Handoff while valid. Every consequential operation still requires the referenced capability and authority grant.

If a heartbeat stops:

- the lease expires;
- Workpoints and partial Evidence remain durable;
- idempotency prevents repeated side effects;
- Focusa may re-offer, resume through another agent, escalate, or hold for review;
- recovery never uses daemon-last or visually active Workstream state.

---

## 7. Graph decomposition and autonomous execution

Agents may propose a graph; they cannot create a parallel canonical task engine.

A graph proposal carries temporary node IDs, objectives, dependencies, capability requirements, side-effect classes, budgets, acceptance criteria, target execution domains, parent Workpoint/event head, and idempotency metadata.

Focusa validates:

- exact Workstream ownership;
- parent Workpoint and revision/event head;
- authority and budget containment;
- dependency validity and supported loops;
- capability availability;
- side-effect and approval posture;
- cross-Workstream references;
- verification completeness.

Admitted nodes become canonical Workpoints and dependency edges. Ready nodes may be offered in parallel when authority, leases, Evidence, and conflict policy permit.

Cross-Workstream plans decompose into independently authorized Workstream operations. Different Workstreams never merge canonical graph state or ambient authority.

Verification nodes may target tests/builds, static analysis, UIAI browser/OS behavior, visual comparison, document/data validation, independent model review, operator approval, or release proof.

Completion is accepted only when required criteria and Evidence settle through Focusa.

---

## 8. Transport and wake-up

SSE, polling, cron, cloud messaging, desktop notifications, and Buzz are wake-up or projection mechanisms. The durable Workstream event and projection state is committed before notification.

### 8.1 SSE

```text
GET /v1/agent-inbox/events
  ?agent_id=agent_...
  &scope_ref=prk_...
  &workstream_id=ws_...
```

Requirements include `Last-Event-ID`, stable cursors, reconnect/backfill, heartbeat frames, reauthorization, exact eligibility filters, zero foreign Workstream content, and bounded event payloads with pull-by-ref.

### 8.2 Polling

```text
POST /v1/agent-inbox/pull
```

The request carries agent identity, capabilities, supported schemas, execution domain, resource posture, maximum work, and optional exact Workstream filters.

### 8.3 Cron and scheduled workers

Cron wakes an agent process. The agent authenticates, registers availability, pulls eligible work, atomically claims, executes under Workstream contracts, submits results/Evidence/Receipts, and exits when idle.

Cron SHALL NOT execute raw inbox payloads directly.

### 8.4 Buzz

Buzz may subscribe to typed Handoff events and publish human-readable projections. A response becomes conversation evidence, a bridge-generated typed proposal, a confirmation request, or a validated reducer operation.

Buzz never owns leases, graphs, Workpoints, authority, or settlement.

---

## 9. Radar observation pipeline

### 9.1 Source families

Radar may consume approved:

- Focusa Workpoint and Work Loop events;
- Pi and agent activity;
- terminal commands/outcomes;
- repository, build, and test state;
- UIAI accessibility/semantic and bounded visual observations;
- browser state and diagnostics;
- document, calendar, communication, and connector events;
- user commitments and TODOs;
- recurring workflow signatures.

Each source declares capability, privacy, retention, egress, and Workstream-binding policy.

### 9.2 Capture strategy

Default:

```text
event-driven structured observation
→ accessibility/semantic state first
→ content-fingerprint deduplication
→ adaptive minimum gap and unchanged-state backoff
→ bounded visual capture where structured state is insufficient
```

A fixed screenshot every ten seconds is not the default architecture.

### 9.3 Detection and interruption

A proposed intervention score may combine:

```text
expected value × confidence × urgency × reversibility
− interruption cost − execution risk − privacy cost − compute cost
```

Scoring is advisory. Policy determines whether Focusa remembers, notices, prepares, proposes, or executes.

### 9.4 Autonomy levels

1. **Remember** — store bounded episode refs.
2. **Notice** — surface a proactive signal.
3. **Prepare** — diagnose, research, or draft without consequential mutation.
4. **Execute safely** — use an isolated worktree, browser context, sandbox, or equivalent.
5. **Apply approved playbook** — execute a previously authorized low-risk class.
6. **Escalate on exception** — continue within explicit limits and interrupt only on block, risk, or required approval.

Autonomy is configured per Project, Workstream, capability, data class, cost, time window, and agent trust class.

---

## 10. Product surfaces

### 10.1 Menubar

The menubar owns:

- Radar active/paused/private posture;
- explicitly attached Project and Workstream;
- observation-source and local/cloud posture;
- high-value notices and pending approvals;
- active-agent state;
- quick approve, dismiss, pause, and open-details actions;
- **Pause observation and all agents** emergency control;
- exact deep links into Focusa Desktop.

It SHALL NOT implement the full timeline, graph editor, or Agent Inbox workspace.

### 10.2 Focusa Desktop

Focusa Desktop gains a **Proactive** workspace or Mission Canvas area:

```text
Proactive
  Inbox
  Signals
  Timeline
  Candidate Workpoints
  Agent Inbox
  Active Graphs
  Awaiting Verification
  Completed
  Policies and Sources
```

Operator-friendly views appear by default; raw routing, events, and protocol details use progressive disclosure.

### 10.3 Pi

Pi exposes compact commands and Work Rail state for eligible Handoffs, claims and leases, graph proposals, active Workpoints/dependencies, blockers, authority requests, result/Evidence submission, and exact handoff to Desktop or Cockpit.

### 10.4 Focusa.work

Focusa.work projects remote/mobile notices, approvals, authority changes, progress, blocks, Evidence, completion summaries, and pause/revoke controls.

Connected-local mode keeps raw observation data local unless policy permits sync.

### 10.5 UIAI Engine Cockpit

Cockpit SHALL NOT add a competing top-level **Proactive** authority.

Use existing IA:

- **Live** — active observer/browser/OS sessions and captures;
- **Activity** — observations, actions, approvals, errors, and execution events;
- **Evidence** — immutable snapshots, verification, Receipts, and reports;
- **Capabilities** — observation/actuation capability and entitlement posture;
- exact **Work Surfaces** — one Episode, UIAI object, Focusa Handoff, Workpoint, target, or verification run.

Canonical approval, cancellation, reassignment, authority, Workpoint, and settlement operations route to Focusa.

---

## 11. Agent-controllable CLI

### 11.1 One semantic command graph per authority

```text
Focusa Command Registry
  → Focusa CLI
  → Focusa Desktop command palette
  → Pi and Focusa agent tools
  → Focusa.work controls
  → REST/OpenAPI/MCP

UIAI Capability Registry
  → compiled UIAI CLI
  → Cockpit GUI
  → UIAI REST/OpenAPI/MCP/Pi
  → agent cards and discovery
```

The registries federate through typed catalogs. They do not collapse authorities.

### 11.2 Required CLI behavior

Every agent-safe command SHALL provide:

- stable command/capability ID and schema version;
- complete JSON and JSONL streams;
- exact resolved Workstream echo where applicable;
- operation ID and Receipt ref;
- typed warnings, denial, conflict, and recovery;
- deterministic exit class;
- typed stdin/file input;
- idempotency for mutation;
- preview/dry-run where supported;
- `--wait`, `--timeout`, `--follow`, and noninteractive behavior;
- shell completion and capability/schema discovery;
- truthful headless/offline posture.

Minimum stable exit classes:

```text
success
usage_or_schema_error
service_unavailable
authentication_failed
entitlement_denied
scope_missing_or_ambiguous
approval_required
conflict_or_stale_revision
claim_or_lease_conflict
blocked_with_recovery
partial_success
transport_timeout
internal_failure
```

Agents never infer these by parsing prose.

### 11.3 Focusa CLI target

```text
focusa workstream list|show|create|attach|detach|context

focusa radar status|start|pause|resume|scan
focusa radar sources list|describe|allow|deny
focusa radar policy show|validate|apply
focusa radar episode list|show
focusa radar signal list|show|dismiss|promote

focusa inbox list|show|watch|pull
focusa inbox offer|claim|heartbeat|release
focusa inbox question|answer|context-add
focusa inbox graph propose|show|admit|reject|patch
focusa inbox block|authority-request
focusa inbox result submit
focusa inbox verify submit
focusa inbox cancel|retry

focusa proactive list|show|approve|dismiss|snooze|execute
focusa agent register|describe|capabilities|presence|doctor

focusa desktop manifest|status|state|present|watch
focusa desktop surface list|open|focus|split|move|close
focusa desktop command list|describe|invoke
```

Mutations require an exact Workstream or Attachment. Local selection may prefill display but is never authority. Legacy `focusa thread` becomes a migration/compatibility reader, not a parallel owner.

### 11.4 UIAI CLI target

Prefer a compiled `cmd/uiai` Go binary sharing generated contracts with the server:

```text
uiai discover|status|health|doctor

uiai cockpit discover|status|state|watch
uiai cockpit workspaces list
uiai cockpit capabilities list|search|describe
uiai cockpit attach|detach|switch
uiai cockpit surface list|open|focus|split|move|close
uiai cockpit object list|open|inspect
uiai cockpit query <capability-id>
uiai cockpit call <capability-id>
uiai cockpit proposal list|preview|accept|reject
uiai cockpit receipt show
uiai cockpit undo|retry|present

uiai observe status|start|pause|resume
uiai observe sources list|describe|allow|deny
uiai observe capture once
uiai observe episode list|show|export
uiai observe policy show|validate|apply

uiai browser ...
uiai research ...
uiai evidence ...
```

Generic `query` and `call` are the stable low-level agent path. Ergonomic domain commands are generated aliases over the same capability IDs.

### 11.5 Semantic verification

Agents control Desktop and Cockpit through semantic state, stable IDs, object refs, Work Surface IDs, exact Workstream context, operation status, and Receipts. Screenshots remain Evidence and diagnostics, not the primary control protocol.

---

## 12. API families

Representative Focusa routes:

```text
GET  /v1/workstreams/{workstream_id}/radar/status
GET  /v1/workstreams/{workstream_id}/radar/episodes
GET  /v1/workstreams/{workstream_id}/radar/signals
POST /v1/workstreams/{workstream_id}/radar/signals/{signal_id}/admit

GET  /v1/workstreams/{workstream_id}/agent-inbox/handoffs
POST /v1/workstreams/{workstream_id}/agent-inbox/handoffs
GET  /v1/workstreams/{workstream_id}/agent-inbox/handoffs/{handoff_id}
POST /v1/workstreams/{workstream_id}/agent-inbox/handoffs/{handoff_id}/claim
POST /v1/workstreams/{workstream_id}/agent-inbox/handoffs/{handoff_id}/heartbeat
POST /v1/workstreams/{workstream_id}/agent-inbox/handoffs/{handoff_id}/release
POST /v1/workstreams/{workstream_id}/agent-inbox/handoffs/{handoff_id}/graph-proposals
POST /v1/workstreams/{workstream_id}/agent-inbox/handoffs/{handoff_id}/events
POST /v1/workstreams/{workstream_id}/agent-inbox/handoffs/{handoff_id}/results

GET  /v1/agent-inbox/events
POST /v1/agent-inbox/pull

GET  /v1/agents
POST /v1/agents/register
POST /v1/agents/{agent_id}/heartbeat
GET  /v1/agents/{agent_id}/eligible-handoffs
```

Every route uses the canonical Workstream context extractor. A path parameter does not replace authority validation.

UIAI observation, episode, capture, actuation, render, export, and Evidence-candidate routes are generated from its capability registry rather than frozen as disconnected one-off APIs.

---

## 13. Privacy, security, and trust

### 13.1 Visible state

The operator must always be able to determine whether observation is active; included sources/apps; bound Project/Workstream; screenshot posture; egress posture; active agents; UIAI control owner; and how to pause all observation and actuation.

### 13.2 Local-first default

- raw screenshots remain local;
- structured observations remain local unless policy permits sync;
- redaction precedes model/cloud egress;
- Focusa receives bounded summaries and refs;
- encrypted sync is explicit;
- retention is source- and classification-specific.

### 13.3 Prompt injection

Browser content, accessibility text, terminal output, documents, chat, email, whiteboards, spreadsheets, screenshots, and generated UI are untrusted content.

They may inform a proposal. They cannot alter Workstream binding, grants, capability schemas, approvals, agent selectors, budgets, entitlements, or canonical completion criteria.

### 13.4 Private context

Required controls include application/window denylists; password-manager, banking, and private-profile defaults; secret/token redaction; clipboard classification; private mode; screenshot region exclusion/blur; source pause; deletion/export; and separation of mutable observation retention from immutable Evidence retention.

### 13.5 Emergency stop

**Pause observation and all agents** SHALL stop new capture, suspend/revoke delegated UIAI control, stop new claims, pause eligible Work Loops, preserve durable state/Receipts, avoid corrupting in-flight transactions, and expose recovery.

---

## 14. Deployment modes

### 14.1 Local-only

Focusa daemon owns Workstream state and Agent Inbox; local UIAI observes/acts; local agents use SSE/polling; raw context remains local; Desktop and menubar provide the operator experience.

### 14.2 Connected-local SaaS

Local daemon/Desktop retains Workstream authority unless explicitly migrated. Cloud provides notifications, mobile approval, routing, hosted workers, and encrypted projections. It SHALL NOT become a silent second canonical writer.

### 14.3 Hosted Workstream

Focusa Cloud owns the exact hosted reducer and Agent Inbox. Local edge services attach through typed Attachments. UIAI product, feature, node, time, sequence, and limit authority remains independently validated.

### 14.4 Self-hosted

The same contracts support self-hosted Focusa, UIAI, agents, Buzz bridge, and notification adapters.

---

## 15. Cross-repository specification family

### 15.1 Focusa

Normative document:

```text
docs/164-focusa-radar-workstream-scoped-ambient-intelligence-agent-inbox-and-autonomous-multi-agent-execution-spec.md
```

Planned companions:

```text
docs/spec164/01-observation-episodes-and-signal-admission.md
docs/spec164/02-agent-inbox-handoffs-claims-and-thread-events.md
docs/spec164/03-graph-admission-work-loop-and-multi-agent-execution.md
docs/spec164/04-cli-api-mcp-desktop-menubar-and-work-contracts.md
docs/spec164/05-security-privacy-saas-migration-and-closure.md
docs/contracts/spec164-complete-feature-and-contract-ledger.v1.yaml
```

### 15.2 UIAI Engine

Coordination issue: `WPUIAI/uiai-engine#43`

Planned companion:

```text
UIAI-COCKPIT-006 — Focusa Radar Observation Adapter, Episode Timeline,
Handoff Projection, and Proactive Verification Amendment
```

It SHALL integrate—not duplicate—UIAI-COCKPIT-003 navigation, UIAI-COCKPIT-004 exact Work Surfaces/Attachments, UIAI-COCKPIT-005 universal control and CLI/API/MCP/Pi parity, and issues #12, #18, #21, #27, #31, #35, #36, #37, #40, and #42.

Required machine companion:

```text
UIAI-COCKPIT-006-C01 — observation, episode, capture, Handoff projection,
verification, CLI, event, security, recovery, and Evidence capability ledger
```

---

## 16. Dependency-ordered implementation

```text
F158-1  Stable WorkstreamId, WorkstreamKey, ScopeRouter
F158-2  Scoped events, snapshots, replay, and quarantine
F158-3  Workpoint, Work Loop, writer lease, and client-envelope cutover
F158-4  Generated Workstream/Attachment client contracts

S164-1  Normative docs and machine-readable ledger
S164-2  Signal, Handoff, thread-event, claim, and result schemas
S164-3  Agent Directory and capability-catalog federation
S164-4  Workstream-owned Agent Inbox reducer state and persistence
S164-5  Claim/heartbeat/release/idempotency/recovery
S164-6  Focusa CLI/API/MCP/Pi parity
S164-7  Graph validation and Workpoint admission
S164-8  Multi-agent scheduling, verification, and settlement

R164-1  Observation adapter and quarantine
R164-2  Structured capture, dedupe, and adaptive backoff
R164-3  Work Episode builder and provenance
R164-4  Signal detectors and scoring
R164-5  Autonomy policy and Candidate Workpoint admission

D164-1  Menubar Radar/privacy/quick approval
D164-2  Desktop Proactive/Agent Inbox
D164-3  Pi Work Rail and commands
D164-4  Focusa.work projection

U003    Cockpit navigation and Activity/Live/Evidence homes
U004    Scoped Work Surfaces, panes, windows, exact Attachments
U005    Universal registry, semantic state, compiled CLI, parity
U006-1  UIAI observation objects, episodes, local retention
U006-2  Focusa Handoff/Workpoint Work Surface adapter
U006-3  UIAI actuation and proactive verification
U006-4  Desktop live-reference and continuation integration
```

Critical path:

```text
F158-1 → F158-2 → F158-3 → F158-4
→ S164-1 → S164-2 → S164-4 → S164-5 → S164-6
→ S164-7 → S164-8
```

The UIAI headless bridge may precede the full Cockpit GUI. Cockpit presentation depends on U003/U004/U005.

---

## 17. Mandatory acceptance matrix

### 17.1 Workstream isolation

- [ ] Two Projects receive simultaneous observations with zero cross-project leakage.
- [ ] Two Workstreams inside one Project keep Episodes, signals, Handoffs, graphs, budgets, leases, and results isolated.
- [ ] Unbound/ambiguous observations quarantine with zero mutation.
- [ ] UI focus, CWD, Continuity, Session, or latest state cannot authorize a Handoff.

### 17.2 Observation quality and privacy

- [ ] Unchanged structured/visual state does not create repeated Episodes or signals.
- [ ] Adaptive backoff reduces capture frequency.
- [ ] Semantic/accessibility capture remains the primary path.
- [ ] Visual capture enriches rather than replaces structured state.
- [ ] Raw screenshots remain local by default.
- [ ] Retention, deletion, export, and immutable Evidence are distinct and proven.
- [ ] Denied applications/private profiles produce no observation content.
- [ ] Secrets redact before permitted egress.
- [ ] Untrusted content cannot alter authority, scope, budgets, or policy.
- [ ] Emergency stop suspends capture and delegated control.

### 17.3 Delivery and recovery

- [ ] SSE reconnect with `Last-Event-ID` loses no eligible Handoff and duplicates no transition.
- [ ] Polling and SSE expose equivalent schemas and eligibility.
- [ ] Cron workers can wake, pull, claim, execute, report, and exit.
- [ ] A single-claim race yields one valid claim.
- [ ] Expired claims re-offer without repeating completed side effects.
- [ ] Restart restores each Workstream inbox independently.

### 17.4 Graph and multi-agent execution

- [ ] Agent graphs cannot create canonical nodes before reducer admission.
- [ ] Admitted nodes become Workpoints with exact dependencies.
- [ ] Independent nodes execute in parallel with separate leases/Receipts.
- [ ] Conflicts create visible recovery or resolution state.
- [ ] Cross-Workstream references do not merge authority or state.
- [ ] Completion requires specified verification and Evidence.

### 17.5 CLI and surface parity

- [ ] Focusa CLI/API/Pi/MCP/Desktop/agent tools use the same IDs and schemas.
- [ ] UIAI compiled CLI/API/MCP/Cockpit use the same capability IDs and schemas.
- [ ] Every mutation echoes exact Workstream and operation/Receipt refs.
- [ ] Every denial has stable JSON and exit behavior.
- [ ] Headless commands work with Desktop/Cockpit closed where permitted.
- [ ] Supported semantic operations require no OCR, coordinate clicking, or label matching.
- [ ] `scripts/uiai` retains no unique authority/business logic after migration.
- [ ] Menubar remains compact and can pause observation and agents.
- [ ] Desktop is the rich Proactive/Agent Inbox surface.
- [ ] Cockpit uses Live/Activity/Evidence/Capabilities rather than a competing authority.
- [ ] Buzz cannot mutate state without typed proposal and reducer acceptance.

### 17.6 Deployment

- [ ] Local-only requires no cloud service.
- [ ] Connected-local does not create dual canonical authority.
- [ ] Hosted workers receive bounded context and grants.
- [ ] Hosted Workstreams preserve exact local Attachments and UIAI entitlement boundaries.
- [ ] Self-hosted passes the same protocol, isolation, and recovery tests.

---

## 18. No-false-closure rule

This specification is not complete because screenshots appear in a sidebar; a model summarizes the screen; one SSE event arrives; a cron shell script executes; an inbox table exists outside the reducer; a queue uses only project path or Continuity; a graph lives in an agent framework; Cockpit clicks its own buttons; the Bash wrapper emits JSON; one autonomous bug fix succeeds; Buzz agents converse; or global records are merely labeled legacy.

Closure requires:

1. required Spec 158 Workstream foundation is implemented;
2. Agent Inbox is physically and logically Workstream-partitioned;
3. Handoffs, claims, graphs, Workpoints, Work Loop, authority, Evidence, and Receipts have one ownership model;
4. Focusa and UIAI retain separate authorities with generated contracts;
5. CLI/API/GUI/MCP/Pi parity is runtime-proven;
6. privacy, injection, retention, and emergency-stop behavior is proven;
7. multi-agent execution survives crash, reconnect, restart, stale revisions, and conflict; and
8. every tranche has stable Evidence and a no-false-closure review.

---

## 19. Final architectural statement

> **Focusa Radar notices. The Focusa reducer decides what the observation means. Agent Inbox delivers bounded work. Agents propose how to decompose it. Workpoints and Work Loop remain canonical execution state. UIAI Engine observes, acts, and proves within its authority. Focusa Desktop presents the complete proactive mission system. The menubar keeps it ambient and controllable. Buzz communicates, but Focusa governs.**
