# Proposal: direct agent messaging through the Focusa Pi extension

**Status: draft design; not implemented or enabled.** This proposal changes no
runtime permissions, tool registration, service, installed extension, or release
version. The names below are proposed contracts, not available commands.

## Outcome

An operator working with a build agent and a separate review agent should not
have to copy messages between their terminals. Explicitly enrolled sessions can
exchange bounded requests, questions, findings, and acknowledgements through
Focusa. A busy receiver gets a queued follow-up, not an interrupted tool call.
The operator remains able to observe, pause, and revoke the exchange.

Messaging must preserve independent review: the builder can request a review and
receive findings, but cannot author the reviewer's verdict or turn delivery into
acceptance. Existing task records remain the source of task status. A message
neither claims a task nor closes it.

## Existing architecture to reuse

- [Spec 168](../168-multi-agent-silent-session-orchestration-workflow.md) owns
  multi-agent task execution through the existing work loop and Silent Sessions.
  Messaging does not create a second orchestrator or spawn background agents.
- [Spec 165](../165-background-execution-and-completion-notification.md) provides
  durable-before-broadcast delivery and exact attachment routing. Its background
  completion events remain progress-only: this proposal does **not** change their
  prohibition on sending progress into model context.
- [Spec 169](../169-fast-forward-session-multiplier-design.md) preserves per-lane
  execution budgets. Messages must not multiply or reset those budgets.
- The [Pi extension](../../apps/pi-extension/README.md) already has typed scope,
  attachment lifecycle, tool results, discovery, and an event-stream consumer.
- The [production consistency policy](../current/PRODUCTION_CONSISTENCY_POLICY.md)
  requires producer, consumer, compatibility, and live delivery proof, not merely
  a successful send endpoint.

Pi's extension API supports custom messages and queued follow-up delivery. Its
upstream `file-trigger.ts` example illustrates receiving an external notification;
it is not an appropriate production transport. A public temporary file, terminal
keystroke injection, process ID, or modified session transcript supplies neither
recipient authorization nor reliable delivery.

## Bounded first implementation

Start with two independently running, operator-enrolled Pi sessions attached to
one supported Focusa daemon, one verified project/continuity scope, and the same
admitted operator principal. Sessions may have different build/review roles.

This is not cross-user or cross-machine acceptance. Those routes need separately
verified enrollment, transport authentication, data-disclosure policy, and remote
recipient proof. A shared operating-system user or project directory alone is
not an authenticated operator principal. If the installed daemon cannot bind
both sessions to the required identities, enrollment is blocked rather than
inferred from their filesystem access.

Default is off. Joining requires an explicit operator action in each session
(or a pre-existing, applicable enrollment grant). Discovery lists only peers
already visible under that grant, not every process, account, or transcript.
Enrollment does not restore itself after a fork, new session, or scope change.

## Proposed call path and ownership

```text
sender Pi tool
  -> existing Focusa authenticated request and scope checks
  -> daemon messaging owner: enrollment, policy, budgets, durable record
  -> recipient-authorized event notification / bounded replay
  -> receiving extension: exact attachment and enrollment validation
  -> Pi custom-message follow-up queue
  -> explicit receipt / optional authorized response through the same owner
```

The daemon owns enrollment, revocation, routing, expiry, idempotency, durable
status, and aggregate exchange limits. The extension is a thin sender/consumer;
it must not create a parallel mailbox service, persistent polling process, or
second task ledger. Reuse canonical error and `tool_result_v1` constructors.

A shared event stream must not expose message bodies or sensitive peer metadata
to unrelated subscribers. Use recipient-authorized delivery, or a minimal
non-sensitive notification followed by an authenticated fetch. Client-side
filtering after a broadcast is not a confidentiality boundary.

## Proposed contract: `focusa.agent_message.v1`

Strictly validate a versioned envelope. Resolve identity and policy fields from
the authenticated enrollment; never trust client-supplied sender attribution.

| Field group | Required meaning |
| --- | --- |
| Identity | Daemon-assigned message ID, authenticated sender and exact recipient attachment, enrollment/grant revision |
| Scope | Verified project root plus continuity, work item reference, conversation ID |
| Session lifecycle | Exact session/attachment instance; run and generation where applicable |
| Content | Enumerated kind (`request`, `question`, `finding`, `acknowledgement`), bounded UTF-8 text, optional authorized evidence references |
| Causality | Client idempotency key, optional reply-to message, monotonically bounded exchange/hop accounting |
| Time | Creation time, expiry, and delivery/acknowledgement timestamps assigned by the owner |
| Policy | Explicit non-authoritative peer provenance and applicable delivery/wake budget |

The sender selects only an enrolled recipient and permitted message fields.
Transport credentials are never part of message content or receipts. Limit both
encoded bytes and the resulting model-context contribution. Reject unknown
versions, malformed fields, unexpected attachments, oversized content, and
unsupported content types before queueing. Do not fetch evidence references
automatically or interpret text as an executable command.

### Receipt semantics

Keep distinct facts instead of a single misleading `sent` flag:

1. **Accepted:** authorization passed and the daemon durably recorded the message.
2. **Queued:** available for the exact enrolled recipient; not yet seen by Pi.
3. **Recorded in recipient session:** consumer reports successful insertion of
   that message ID into the correct session. A `sendMessage()` return value is
   not sufficient proof of this state.
4. **Acknowledged:** explicit receipt for the same message from the authenticated
   recipient. This does not prove the requested task ran or the finding is true.
5. **Rejected, expired, or revoked:** explicit terminal outcome with a bounded
   reason; never reported as delivered or acknowledged.

Commit before notifying. Retries with the same idempotency key and content return
one receipt; changed content under that key is rejected. Reconnects must reconcile
message IDs with the recipient session before redelivery. Do not claim exactly-once
execution across a crash between transcript insertion and receipt persistence:
expose ambiguous delivery and suppress automatic task re-execution. Consumer
acknowledgements and task-result evidence remain separate.

## Pi extension integration

- Proposed operator command: `/focusa peers` with join, status, pause, leave, and
  revoke actions. Joining selects the already-verified scope and permitted peers;
  it never binds a project merely because Pi launched in a directory.
- Proposed agent tools: narrow send and receipt/status operations, available only
  under the enrolled policy. Agents cannot use them to enroll peers, grant
  themselves permission, or expand the recipient set.
- Incoming text uses a distinct custom message type such as
  `focusa-agent-message`, with visible sender role, task, and message ID. Do not
  impersonate a human with `sendUserMessage()` or place peer text in a system
  prompt. Mark the body as untrusted peer-supplied data and potential instructions
  subject to the recipient's existing task and authority boundaries.
- Validate the active attachment again immediately before insertion, not only
  when the event first arrives. Queue behind current work; no automatic abort,
  steering injection, editor replacement, or hidden terminal input.
- Session switch, fork, compaction/rollover, reload, and shutdown must cancel or
  revalidate pending delivery through the existing attachment lifecycle. Never
  attach a delayed callback to whichever session happens to be current.
- Receiving and waking the model are separate permissions. An explicit opt-in
  may permit automatic turns, charged against the existing work-loop policy and
  a finite shared exchange budget. Otherwise messages remain visible and queued
  without a model call. Both paths need consumer proof.
- Acknowledgements never trigger replies to acknowledgements. Enforce bounded
  queue depth, per-peer rate, message expiry, conversation hops, automatic turns,
  and wall-clock duration at the daemon. Budget exhaustion yields a visible pause;
  sending a new message ID cannot reset the conversation budget.
- Existing completion/progress rendering remains unchanged. Messaging must work
  without enabling messaging on every installed Pi session.

Register implemented tools one-to-one in runtime, discovery, generated Pi and
capability descriptors, cross-harness adapters, and per-tool documentation.
Do not advertise proposed names as installed tools before their owner exists.
Apply the existing permissions and product-feature entitlement policy; no local
file or extension-only fallback when the daemon denies or lacks the capability.

## Security and privacy boundaries

- A request, finding, shared room, peer role, or successful acknowledgement is
  not operator approval, identity transfer, task ownership, or acceptance truth.
- Existing destructive, privileged, financial, tenant, and publication gates
  remain intact. Neither peer gains the other's tool permissions or credentials.
- Enrollment authorizes bounded project collaboration, not transcript sharing.
  Never auto-attach private memory, session history, environment variables,
  authentication artifacts, screenshots, or arbitrary files. Evidence references
  require the receiver's own permission to resolve them.
- Define retention and deletion policy before enabling the message store. Store
  only necessary bounded content with access controls; ordinary logs and audit
  receipts contain identifiers, status, sizes, and content hashes rather than
  message bodies or secrets. Do not promise perfect secret detection.
- Revoke must block new sends, fetches, and undelivered queued messages promptly.
  It cannot erase text already seen by a recipient; report that limit honestly.
- Discord or another shared inbound channel must not inherit this enrollment or
  an administrative tool path. A same-project match is not sufficient admission.

## Implementation and acceptance checklist

No item below is claimed complete by this documentation PR.

- [ ] Resolve existing identity/enrollment, recipient stream authorization,
  entitlement, storage, and budget owners; specify the additive versioned routes
  and public schemas before implementing adapters.
- [ ] Producer tests: exact identity/scope checks, denied peer, revoked grant,
  expiry, retry deduplication, conflicting idempotency payload, durable-before-event
  ordering, replay bounds, retention, and concurrent send limits.
- [ ] Consumer tests with a real Pi adapter: busy follow-up, idle receipt without
  wake permission, opted-in wake, duplicate event, wrong scope/recipient,
  disconnect, switch/fork/rollover/reload, and cancelled stale callbacks.
- [ ] Crash/reconnect tests distinguish durable queueing, session insertion,
  ambiguous receipt, explicit acknowledgement, and actual task completion.
- [ ] Negative tests: malicious peer instructions do not grant execution rights;
  unrelated stream subscribers cannot retrieve content; private context is not
  automatically exported; shared-channel callers cannot enroll or send.
- [ ] Loop controls: two peers cannot sustain acknowledgement storms, reset hop
  limits with new IDs, or exceed aggregate wake/turn budgets.
- [ ] Compatibility: older daemon/extension pairs report messaging unavailable,
  unknown message versions fail closed, and existing progress events keep their
  non-model behavior. No terminal/file-watcher fallback.
- [ ] Live two-session proof: build agent sends a synthetic review request;
  independent reviewer receives it at a safe pause and returns a bounded finding;
  both receipts agree; operator is not a relay; task status is not changed merely
  by delivery. Include revocation and restart recovery in the same bounded proof.
- [ ] Tool/documentation parity and supported platform/installation proof satisfy
  the production consistency policy before release. No untested cross-account,
  cross-host, or other-harness support claims.

## Disposition

This PR asks upstream to review the integration boundary and acceptance contract.
It intentionally contains no transport scaffolding or partially advertised tool.
After agreement, implement one owner-backed end-to-end slice with the existing
Focusa work-loop/task workflow, then enable it only for explicitly enrolled
sessions. No live extension, service, credential, or operator task is changed by
merging this proposal.
