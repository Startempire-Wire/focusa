# 174 — Focusa Agent Workforce Browser Extension MVP Slice

**Status:** normative MVP implementation specification
**Release posture:** required before the held `v0.9.184-dev` release resumes
**Owner:** Sir V3 / Focusa runtime
**Supersedes:** the launch-scope ambiguity in the 2026-08-23 Spec 174 concept
**Related:** Spec 79, Spec 133, Spec 165, Spec 168, Spec 169, Spec 173, Spec 175, Production Consistency Policy

---

## 1. Operator outcome

Ship one reliable Chrome surface that lets an owner complete this loop without
opening a terminal after pairing approval:

> observe Chrome + Focusa → orient a bounded mission from the active tab →
> create a governed agent session → start and orchestrate it → watch durable
> progress/audit → reconnect without losing truth.

This is a **vertical MVP proof**, not the complete workforce product.
The browser extension is a control and observation client. The Focusa daemon
remains the runtime, scheduler, persistence authority, policy authority, and
source of truth. Closing Chrome must never stop an agent.

## 2. Four required capabilities

### 2.1 Observation

Observation means two bounded views:

1. **Chrome observation:** after an explicit owner gesture, read only the
   active tab's title, URL, and origin. No page body, cookies, form values,
   browsing history, DOM scraping, content script, or screenshot in MVP.
2. **Focusa observation:** display the paired daemon's health, Work Loop
   summary, authorized silent-session roster, and durable SSE audit events.

Observation is not control and does not mutate Focusa state.

### 2.2 Orientation

Orientation means converting an owner-approved active-tab observation plus an
explicit objective into a bounded mission packet. It answers:

- What page/context is relevant?
- What does the owner want done?
- Which Focusa project, continuity scope, work item, role, and model apply?
- What is explicitly excluded?

Orientation is not autonomous intent inference. The owner must review the
packet before creation.

### 2.3 Creation

Creation means preflighting and drafting one daemon-native Silent Session using
`focusa.silent_session_config.v1`, then obtaining a durable start approval and
starting the exact run. Creation does not introduce a browser-owned Agent type.
The created agent is the existing Silent Session projection.

### 2.4 Orchestration

Orchestration means exact-target lifecycle control over the created or selected
Silent Session:

- start,
- pause,
- resume,
- steer,
- cancel only through explicit step-up approval,
- refresh after stale-target rejection.

Every mutation uses daemon-issued `session_id`, `run_id`, `generation`, durable
approval where required, and a persisted idempotency key. The extension never
reconstructs canonical state locally.

## 3. Shared definitions

| Term | Normative definition |
|---|---|
| Owner | Human who pairs the Chrome profile to one or more Focusa daemons and supplies consent/direction. |
| Daemon connection | `{connection_id, label, base_url, device_id, token, granted_scopes, last_cursor}` stored locally in the Chrome profile. |
| Active daemon | The single daemon currently projected in the side panel. Other paired daemons remain stored but receive no implicit mutation. |
| Workforce | Authorized Silent Sessions visible through the active daemon. No second workforce database exists. |
| Agent | UI projection of one daemon-native Silent Session. |
| Manager | Session whose `role_profile_ref` resolves to a manager-capable existing Focusa role. MVP does not invent a new Manager runtime class. |
| Crew | Session with a bounded implementation role and task binding. MVP displays it identically except for role label. |
| Roster | Authorized projection returned by `GET /v1/silent-sessions`; never a client-maintained canonical list. |
| Browser observation | Owner-approved `{title, url, origin, captured_at}` from the active tab. |
| Orientation packet | Bounded, reviewed client contract that becomes the session mission and identity bindings. |
| Objective | Owner-authored desired outcome; required and nonblank. |
| Exclusion | Owner-authored constraint describing what the agent must not do. |
| Work item | Existing provider-neutral `WorkItemRef`/bead binding supplied as `identity.work_item_ref`. |
| Role profile | Existing `role_profile_ref`; MVP permits explicit input, not role invention. |
| Exact target | Current `{session_id, run_id, generation}` refreshed from the daemon before mutation. |
| Durable approval | Daemon-persisted, action-digest-bound consent record required by Spec 133. |
| Audit event | `focusa.stream_event.v1` envelope from durable SQLite replay plus live SSE tail. |
| Cursor | Last durably rendered SSE sequence. Persisted per daemon and supplied on reconnect. |
| Degraded | Read/control path failed but local connection state remains recoverable. Never displayed as success or disconnected truth. |
| Reconnecting | Bounded retry state after stream loss; daemon execution is presumed independent, not stopped. |
| Idempotency key | UUID created and persisted before a mutation request; reused only for an identical retry. |
| Task graph | This spec's executable bead dependency DAG. It is not a new product graph database. |

## 4. MVP boundary

### 4.1 In scope

- Chrome Manifest V3 side panel.
- One active connection at a time; multiple saved daemon connections allowed.
- Existing device pairing flow with `read` + `write` scopes.
- Active-tab metadata after explicit owner gesture.
- Reviewed orientation packet.
- Silent Session preflight, create, durable start approval, and start.
- Roster with lifecycle, mission label, role, work item, update time, and health.
- Pause/resume and bounded steering.
- Cancel only after a second explicit confirmation and durable approval.
- Durable cursor-replay SSE timeline with reconnect.
- Work Loop summary for orientation (current work item, status, blocker).
- Clear token/revoke/local-forget behavior.
- Chrome unpacked artifact built in canonical CI/release output.

### 4.2 Explicitly deferred

- Firefox/Safari.
- Chrome Web Store publication.
- Content scripts or arbitrary DOM/page-body capture.
- Screenshots, cookies, password capture, form capture, session sharing.
- Secret Broker integration (Spec 173).
- Voice/STT, Radar, marketplace, templates marketplace.
- Rich freeform DAG editing.
- Fanout speed dial; Spec 169 route is plan-only today and must not be
  misrepresented as session creation.
- Cross-daemon aggregate mutation or replicated ledgers.
- New role/capability ontology; Spec 175 remains authority.
- Background service-worker ownership of long-running execution.

## 5. Reused Focusa primitives

| Need | Existing authority |
|---|---|
| Pairing | `POST /v1/device/pair/start`, `GET /v1/device/pair/status`, operator CLI completion, SQLite token durability |
| Authentication | paired device bearer token + device scopes |
| Health | `GET /v1/health` |
| Work orientation | `GET /v1/work-loop/status?summary_only=true` |
| Roster | `GET /v1/silent-sessions` |
| Session preflight/create | `POST /v1/silent-sessions/preflight`, `POST /v1/silent-sessions` |
| Lifecycle | existing start/pause/resume/cancel routes |
| Steering | `POST /v1/silent-sessions/{session_id}/steer` |
| Audit/live state | `GET /v1/events/stream?cursor=...` using `focusa.stream_event.v1` |
| Canonical completion | Silent Session completion stream + receipts; extension only projects it |
| Agent runtime | daemon-native Silent Sessions governed by Specs 133/168 |

No browser-owned scheduler, roster store, audit ledger, agent runtime, or
completion inference may be introduced.

## 6. Required small daemon addition: durable approval issuance

Existing lifecycle routes correctly require durable approvals, but no bounded
HTTP issuance route exists for an authenticated operator client. MVP adds one
canonical route rather than bypassing approval checks.

### 6.1 Route

`POST /v1/silent-sessions/{session_id}/approvals`

Required request:

```json
{
  "schema": "focusa.silent_session_approval_request.v1",
  "action": "start",
  "run_id": "<uuid>",
  "generation": 1,
  "idempotency_key": "<uuid>",
  "risk_acknowledged": true
}
```

Allowed MVP actions: `start`, `send_input`, `cancel`.

The daemon MUST:

1. authenticate the paired-device principal;
2. require the action's existing exact route scope;
3. load session, run, active config revision, and exact generation;
4. reject stale targets before writing;
5. derive project, continuity, work item, config hash, model binding,
   workspace, risk class, and permitted side effects from canonical records;
6. compute `action_digest` server-side using existing authorization code;
7. persist `DurableApprovalRecord` before responding;
8. cap expiry at 5 minutes;
9. return only approval id, action, exact target, expiry, and receipt ref;
10. replay an identical idempotency key and reject changed payload reuse.

The client MUST NOT submit or override action digest, permitted side effects,
config hash, model binding, workspace, risk class, or operator actor.

### 6.2 Response

```json
{
  "schema": "focusa.silent_session_approval_response.v1",
  "status": "approved",
  "approval_id": "<uuid>",
  "action": "start",
  "session_id": "<uuid>",
  "run_id": "<uuid>",
  "generation": 1,
  "expires_at": "<RFC3339>",
  "receipt_ref": "approval:<uuid>"
}
```

### 6.3 Fail-closed cases

Return structured 4xx for unauthenticated principal, missing scope, unknown
session/run, stale generation, invalid lifecycle, unsupported action, blank or
reused-conflicting idempotency key, and non-boolean consent.

## 7. Chrome extension architecture

### 7.1 Location and stack

Create `apps/workforce-extension/`.

Use:

- Chrome Manifest V3;
- browser-native side panel;
- plain standards-based ES modules;
- no runtime framework and no runtime npm dependency;
- Node build script that copies/validates static assets;
- Node built-in test runner for pure modules;
- JSDoc contract typedefs plus runtime validators.

Reason: smallest deterministic surface for a release speedrun. Do not add React,
Svelte, CRXJS, a state library, router, CSS framework, or test framework.

### 7.2 Exact files

```text
apps/workforce-extension/
  package.json
  manifest.json
  scripts/build.mjs
  src/background.mjs
  src/sidepanel.html
  src/sidepanel.mjs
  src/styles.css
  src/lib/contracts.mjs
  src/lib/validation.mjs
  src/lib/storage.mjs
  src/lib/api-client.mjs
  src/lib/sse-parser.mjs
  src/lib/reconnect.mjs
  src/lib/orientation.mjs
  src/lib/projections.mjs
  src/lib/views.mjs
  tests/validation.test.mjs
  tests/storage.test.mjs
  tests/api-client.test.mjs
  tests/sse-parser.test.mjs
  tests/orientation.test.mjs
  tests/projections.test.mjs
```

Repository integration:

```text
tests/174_workforce_extension_static_test.py
scripts/ci/run-spec-gates.sh
.appveyor.yml
.github/workflows/ci.yml
scripts/generate-release-notes.py (definition/feature input only if required)
```

Approval route implementation:

```text
crates/focusa-api/src/routes/silent_sessions_approvals.rs
crates/focusa-api/src/routes/silent_sessions.rs
crates/focusa-api/src/routes/silent_sessions_contract.rs
crates/focusa-api/src/routes/mod.rs (only if module registration requires it)
crates/focusa-api/src/middleware/route_scope.rs
crates/focusa-api/src/routes/agent_capabilities.rs
```

Do not touch files outside this list without amending this spec first.

## 8. Manifest security contract

Required:

- `manifest_version: 3`
- action opens the side panel;
- permissions: `storage`, `sidePanel`, `activeTab`;
- optional host permissions: `http://*/*`, `https://*/*`;
- no content scripts;
- no `tabs`, `cookies`, `history`, `webRequest`, `scripting`, or `<all_urls>`
  persistent host permission;
- extension CSP remains self-only;
- exact daemon origin permission requested from a user click before pairing.

Remote daemon URLs MUST use HTTPS. HTTP is permitted only for
`localhost`, `127.0.0.1`, and `[::1]` development origins.

## 9. Client contracts

### 9.1 Connection record

```json
{
  "schema": "focusa.workforce_connection.v1",
  "connection_id": "<uuid>",
  "label": "KH Focusa",
  "base_url": "https://daemon.example",
  "device_id": "<uuid>",
  "token": "<secret>",
  "granted_scopes": ["read", "write"],
  "last_cursor": "42",
  "created_at": "<RFC3339>",
  "last_connected_at": "<RFC3339|null>"
}
```

Token rules:

- `chrome.storage.local`, never sync storage;
- never log, render, export, place in URL, or include in audit projection;
- redact to `••••` in diagnostics;
- delete on local forget;
- call daemon revoke when authorized before deleting during full unpair;
- no fallback to admin token.

### 9.2 Browser observation

```json
{
  "schema": "focusa.browser_observation.v1",
  "title": "Page title",
  "url": "https://example.com/path?bounded=true",
  "origin": "https://example.com",
  "captured_at": "<RFC3339>"
}
```

Sanitization:

- remove URL username/password;
- remove fragment;
- limit URL to 2048 bytes and title to 300 bytes;
- reject `chrome:`, `chrome-extension:`, `file:`, `data:`, and `javascript:`;
- never capture body/selection in MVP;
- capture only after `Use current tab` click;
- show exact captured fields before creation.

### 9.3 Orientation packet

```json
{
  "schema": "focusa.browser_orientation.v1",
  "objective": "Owner-authored objective",
  "exclusions": ["Do not purchase", "Do not publish"],
  "observation": { "$ref": "focusa.browser_observation.v1" },
  "project_root": "/approved/project",
  "continuity_id": "project-continuity",
  "work_item_ref": "focusa-123",
  "role_profile_ref": "role:researcher",
  "agent_identity_ref": "agent:browser-created",
  "created_at": "<RFC3339>"
}
```

Validation:

- objective 1..4000 bytes;
- exclusions maximum 10, each 1..300 bytes;
- project root and continuity id required;
- work item and role may be null only after explicit review acknowledgement;
- observation required;
- packet immutable after preflight begins; editing creates a new packet/idempotency key.

### 9.4 Mission projection

Until the core config owns a structured orientation field, project the reviewed
packet into `identity.mission` with this exact bounded template:

```text
OBJECTIVE
<objective>

BROWSER ORIENTATION (OWNER-APPROVED)
Title: <title>
URL: <sanitized URL>
Captured: <timestamp>

EXCLUSIONS
- <exclusion>
```

Maximum rendered mission: 8192 bytes. Truncate nothing silently; reject and ask
the owner to shorten it.

## 10. Safe session template

MVP form requires owner-supplied:

- project root;
- continuity id;
- optional work item ref;
- role profile ref;
- provider;
- model;
- auth profile ref.

Fixed safe values:

- harness `pi`;
- native resume `prefer`;
- model selection `exact`;
- fallback disabled;
- workspace `read_only_shared` for MVP;
- integration policy `manual`;
- context authority required;
- risky mutation preflight required;
- destructive actions false;
- writer lease required;
- completion receipt required;
- maximum 12 turns;
- maximum 30 minutes;
- maximum output 16 MiB;
- default redaction/retention profiles.

The extension MUST call preflight and display the redacted config hash before
creation. It MUST NOT offer an unsafe override in MVP.

## 11. UI state machines

### 11.1 Connection

`unconfigured → permission_required → pairing_requested → awaiting_approval → paired`

Failure states: `expired`, `revoked`, `degraded`. Token delivery is one-shot;
the extension persists the token before rendering paired success.

### 11.2 Stream

`idle → replaying → live → reconnecting → live`

- persist cursor only after an event renders successfully;
- reconnect delays: 1s, 2s, 4s, 8s, then 15s maximum;
- use `fetch` streaming so Authorization and permission headers are present;
- send persisted cursor as query parameter;
- deduplicate by sequence/event id;
- a malformed event becomes one visible degraded audit row and does not advance
  cursor;
- 401/403 stops retry and moves connection to `revoked`/`scope_denied`;
- network loss never changes agent lifecycle locally.

### 11.3 Creation

`editing → reviewed → preflighting → preflight_ok → creating → drafted → approving_start → starting → running`

On any ambiguous network failure, query by roster/idempotency result before
retrying creation. Never create a second session speculatively.

### 11.4 Control

Before every mutation:

1. refresh session status;
2. obtain exact run id/generation;
3. persist idempotency key;
4. issue durable approval when action requires it;
5. submit action;
6. accept only canonical response;
7. refresh on 409 stale target.

No optimistic lifecycle transition may be displayed as canonical.


## 12. Execution authority

The executable dependency DAG, failure matrix, production-consistency
proofs, golden E2E, weak-model task packets, per-bead done conditions,
rollback, and final release gate are normative in:

`docs/178-focusa-agent-workforce-browser-extension-mvp-execution-taskgraph.md`.

Spec 174 defines **what must exist**. Doc 178 defines **exactly how weak
models build and prove it**. If they conflict, Spec 174 wins and Doc 178
must be reconciled before execution continues.
