# 178 — Focusa Agent Workforce Browser Extension MVP Execution Task Graph

**Status:** normative execution companion to Spec 174
**Authority:** `docs/174-focusa-agent-workforce-browser-extension-mvp-slice-spec.md`
**Audience:** weak implementation models and adversarial reviewers

This file contains the mechanical build graph and proof obligations. It may
not widen Spec 174. A worker must read Spec 174 first, then execute only one
ready bead from this graph.

---

## 1. Side-panel layout

One page, four bounded regions:

1. **Orientation** — daemon selector, connection state, active-tab capture,
   objective/exclusions, project/work-item/role/model fields, review button.
2. **Creation** — preflight result, redacted config hash, explicit Create &
   Start button, current failure/recovery text.
3. **Workforce** — roster cards with exact lifecycle and Pause/Resume/Steer;
   Cancel behind confirmation.
4. **Audit** — cursor, stream state, filters, last 200 redacted events.

Accessibility:

- keyboard-operable;
- status conveyed by text plus color;
- `aria-live=polite` for stream/creation state;
- no auto-focus stealing during SSE updates;
- reduced-motion compatible;
- minimum 4.5:1 text contrast.

## 2. Audit projection

Render only bounded fields from `focusa.stream_event.v1`:

- timestamp;
- event type;
- session/work-item correlation when present;
- status/summary if primitive string;
- cursor;
- receipt/payload reference, never dereferenced automatically.

Do not dump arbitrary event payload JSON into the DOM. Keep 200 rendered rows;
older truth remains in daemon durable history and is recovered by cursor.

## 3. Reliability invariants

1. Daemon is always source of truth.
2. Chrome close does not cancel or pause sessions.
3. Every mutation is idempotent.
4. Every mutation targets exact generation.
5. Approval-required actions never bypass durable approval.
6. SSE cursor advances only after successful parse + render.
7. Reconnect replays before live tail.
8. Token is persisted before paired success.
9. Unknown/malformed payloads degrade visibly; they never become success.
10. No client-side completion inference.
11. No hidden host permission acquisition.
12. No broad page observation.
13. No raw secret/event payload logging.
14. Existing CLI, Pi extension, menubar, and older daemon clients retain behavior.

## 4. Failure UX

Every failure must show:

- operation;
- HTTP/status class;
- daemon-provided failure class if available;
- whether retry is safe;
- exact recovery action.

Required mappings:

| Failure | UI action |
|---|---|
| Pair code expired | Start a new pair flow; never reuse code. |
| One-shot token consumed before persistence | Explain loss and require re-pair. |
| 401 | Mark token invalid; offer re-pair. |
| 403 | Show missing scope; do not ask for admin token. |
| 409 stale target | Refresh status and require reconfirmation for risky action. |
| SSE network loss | Reconnect from durable cursor. |
| Malformed event | Visible degraded row; preserve cursor. |
| Create timeout | Reconcile using persisted idempotency key before retry. |
| Approval expired | Reconfirm and issue a new approval. |
| Daemon unavailable | Preserve form and connection; never claim session stopped. |

## 5. Production-consistency proofs

Feature is not complete without all five:

1. **Versioned contracts:** observation, orientation, connection, approval
   request/response, and consumed daemon envelopes.
2. **Producer tests:** daemon approval route, auth/scope, persistence,
   idempotency, exact-target and expiry tests.
3. **Consumer tests:** extension validators, SSE parser/reconnect, storage,
   orientation, create reconciliation, lifecycle controls.
4. **Cross-version interop:** new extension against prior supported daemon must
   fail capability detection clearly; new daemon must preserve old menubar/CLI
   pairing and Silent Session behavior.
5. **Live E2E:** real Chrome unpacked extension pairs to a test daemon, observes
   active tab, creates/starts a session, pauses/resumes, receives durable event,
   closes/reopens side panel, and resumes from cursor.

## 6. Acceptance scenario

One release-blocking golden path:

1. Load unpacked extension artifact in Chrome.
2. Enter daemon URL and grant that exact origin.
3. Extension starts pairing with read/write scopes.
4. Operator completes code using existing Focusa CLI.
5. Extension captures one-shot token and persists it before success.
6. Extension health, Work Loop summary, roster, and SSE become live.
7. Owner opens a harmless HTTPS page and clicks `Use current tab`.
8. Owner enters objective, exclusions, project, work item, role and model fields.
9. Extension displays the exact orientation packet.
10. Owner approves review.
11. Extension preflights config and displays redacted hash.
12. Owner clicks `Create & Start`.
13. Extension drafts session, issues exact durable start approval, starts run.
14. Roster and audit show canonical lifecycle/event updates.
15. Owner pauses and resumes using refreshed exact targets.
16. Side panel closes while session remains active.
17. Side panel reopens, replays from cursor, and renders no duplicate events.
18. Token/event logs contain no secret.

## 7. Executable task graph

Canonical dependency notation: `A -> B` means **B depends on A**.

```text
174-00 Spec + contract freeze
 ├─> 174-01 Extension shell/build
 ├─> 174-02 Approval API contract
 └─> 174-03 Release definition ledger

174-01 -> 174-04 Connection storage + URL/permission validation
174-04 -> 174-05 Pairing state machine
174-01 -> 174-06 SSE parser + cursor/reconnect core
174-01 -> 174-07 Chrome observation + orientation packet
174-02 -> 174-08 Approval API implementation + producer tests
174-05 -> 174-09 Authenticated API client + Focusa observation
174-06 -> 174-09
174-07 -> 174-10 Safe config/preflight/create reconciliation
174-09 -> 174-10
174-08 -> 174-11 Start/control/steer/cancel orchestration
174-10 -> 174-11
174-09 -> 174-12 Side-panel integrated projection
174-11 -> 174-12
174-12 -> 174-13 Consumer/static/accessibility tests
174-08 -> 174-14 Cross-version interop tests
174-13 -> 174-15 Live Chrome E2E + proof bundle
174-14 -> 174-15
174-03 -> 174-16 CI/artifact/release wiring
174-15 -> 174-16
174-16 -> 174-17 Final adversarial review + release verdict
```

No implementation node may begin until all listed dependencies are closed.

## 8. Weak-model execution packet (mandatory on every bead)

Every child bead must contain these fields verbatim and completely:

- **Spec anchor:** exact section(s) in this file.
- **Purpose:** one sentence.
- **Dependencies:** bead ids; executor verifies closed before editing.
- **Allowed files:** exhaustive list.
- **Forbidden files/scope:** explicit.
- **Inputs/contracts:** exact schemas/routes/types consumed.
- **Steps:** numbered, mechanical sequence; no design decisions left open.
- **Tests:** exact commands and expected terminal lines.
- **Acceptance:** binary checklist.
- **Evidence:** exact additional allowed output path; attach the specified proof.
- **Failure stop:** conditions requiring blocker rather than improvisation.
- **Handoff:** fields next bead needs.

Weak models MUST NOT:

- choose a new framework;
- rename contracts/routes/files;
- widen permissions;
- add page capture;
- bypass approval;
- duplicate canonical Focusa state;
- modify unrelated release work;
- close a bead because code compiles without its specified tests/evidence.

## 9. Bead-level done conditions

### 174-00 — Spec and contracts

Done when Spec 174 is committed, contract names are frozen, and the parent epic
links this file. No code.

### 174-01 — Shell/build

Done when deterministic build creates an unpacked MV3 directory, manifest has
only allowed permissions, background action opens side panel, and build test is
green.

### 174-02 — Approval API contract

Done when route/action mapping, request/response schemas, server-derived fields,
expiry/idempotency semantics, and capability descriptor are frozen in tests.
No persistence mutation yet.

### 174-03 — Release definition ledger

Done when every commit in `v0.9.183-dev..release-HEAD` and every Spec 174 term
has a plain-language definition, owning file/contract, user impact, failure
meaning, verification reference, and compatibility note. No vague labels.

### 174-04 — Storage/validation

Done when connection records round-trip, tokens never enter sync storage/logs,
URL rules pass, and exact optional origin permission is required.

### 174-05 — Pairing

Done when pair start/status, one-shot token persistence, expiry, revoke/forget,
and restart survival are tested. Pair success may render only after storage
commit succeeds.

### 174-06 — SSE reliability

Done when chunk-split SSE parsing, malformed events, cursor persistence,
deduplication, replay-before-live, bounded backoff, 401/403 termination and
panel reopen are deterministic tests.

### 174-07 — Observation/orientation

Done when activeTab-only metadata capture, sanitization, owner review, packet
validation, mission rendering and size rejection are tested.

### 174-08 — Approval API producer

Done when daemon route persists exact digest-bound approvals and fails closed
for every §6.3 case. Existing authorization tests stay green.

### 174-09 — Focusa observation

Done when health, Work Loop summary, roster and events are fetched with bearer
and bounded read/write permission headers; projections never expose raw token
or arbitrary event payload.

### 174-10 — Create

Done when safe fixed config, preflight display, persisted idempotency key,
create reconciliation, and draft projection pass consumer tests.

### 174-11 — Orchestration

Done when start approval/start, pause/resume, steer approval/steer, and confirmed
cancel approval/cancel refresh exact targets and handle stale/expired approval.

### 174-12 — Integrated UI

Done when the four panel regions implement the state machines without parallel
canonical state or optimistic lifecycle claims.

### 174-13 — Consumer quality

Done when all app tests, static manifest security gate, keyboard path, aria-live,
contrast and reduced-motion checks pass.

### 174-14 — Interop

Done when old CLI/menubar pairing tests and Silent Session tests stay green and
new extension reports unsupported daemon capability instead of guessing.

### 174-15 — Live E2E

Done only with the full §17 flow against a real test daemon and real Chrome,
including panel close/reopen and cursor replay. Screenshots/logs are evidence,
not substitutes for assertions.

### 174-16 — Release wiring

Done when GitHub/AppVeyor gates build/test the unpacked artifact, checksum it,
include it in release assets, and release notes link the definition ledger.
No Chrome Web Store claim.

### 174-17 — Final verdict

Adversarial reviewer attempts to disprove every §3 invariant and §6 step.
Release remains blocked on any critical objection, missing evidence, unknown
compatibility result, or unsupported claim. Machine verdict must contain:
`closure_supported=true`, `evidence_sufficiency=sufficient`, and
`critical_objections=[]`.

## 10. Rollback

- Remove/disable the Chrome artifact from release manifest; daemon remains
  compatible because approval route is additive.
- Revoke paired browser device token.
- Extension local forget deletes token/connection/cursor.
- Do not delete daemon audit/session records.
- Approval route can be disabled from client discovery, but persisted approvals
  remain audit evidence until normal retention.
- Rollback proof: old CLI and menubar pairing plus Silent Session flows remain
  green with the extension absent.

## 11. Release gate

The held release may resume only when:

- beads 174-00 through 174-17 are closed in dependency order;
- no task graph cycle exists;
- Spec 174 static gate passes;
- production-consistency five proofs exist;
- live Chrome E2E passes;
- release definition ledger is complete;
- adversarial verdict is `closure_supported=true`,
  `evidence_sufficiency=sufficient`, and `critical_objections=[]`.
