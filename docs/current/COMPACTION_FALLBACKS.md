# Compaction Fallbacks

Pi extension compaction replaces default Pi compaction for Focusa sessions, so blank `none` fields are not acceptable.

## Policy

Compaction summaries must use intelligent related fallbacks, not random filler and not bare `none`.

Fallback order:

1. Focus State slot value.
2. Canonical WorkpointResumePacket fields:
   - `mission`
   - `next_slice`
   - `project_root`
   - `session_id`
   - `active_object_refs`
   - `verification_records`
   - `blockers`
3. Pi bridge local shadow:
   - recent decisions
   - constraints
   - failures
4. Current ask / active frame goal/title.
5. Session metadata such as current project root.
6. Only if no related canonical source exists: explicit explanatory sentence such as `No open questions recorded by Focusa or Workpoint.`

## Non-goals

- Do not hallucinate decisions, constraints, artifacts, or test results.
- Do not fill slots with unrelated repo facts.
- Do not emit bare `none` for cognitive summary fields.

## Operator-visible observability

Compaction must never look like a frozen conversation. While native compaction is active, the Pi status surface reports phase, elapsed seconds, context pressure, and attempt number; long attempts emit a bounded visible heartbeat. Retry notices include the bounded primary error and retry delay. Terminal, coordinator, and resume-context failures are shown in the UI as well as durable telemetry or console logs. Timers are cleared on completion, failure, compact reset, session start, and shutdown.

Pi owns queued operator input and native continuation after manual or automatic compaction. Focusa queues its hidden resume projection with explicit `deliverAs:"nextTurn", triggerTurn:false`; it never uses the default steering mode and never starts a competing post-compaction turn. Because Pi's API returns `void`, Focusa records `unknown_completion`, performs no blind retry, and relies on the next-turn context path. The harness still offers a truthful receipt signal: the next agent turn consumes the queued message, so its start is the delivery event. Focusa acknowledges that event with a same-session-scoped `agent_start` handler (pure `compactionDeliveryAckEligible` decision; only the session that queued the delivery may ack it) that flips `unknown_completion`/`deferred_to_next_turn` to `delivered` and writes a durable `focusa-compaction-delivery-acknowledged` entry. Settled deliveries (`delivered`, `superseded_by_operator`, `failed`) are never re-acknowledged. Operator text submitted during compaction therefore remains authoritative and flows into Pi's native queue. Agents must use bounded polling rather than long blocking `--watch` commands so steering can be observed and acted on promptly.

## Guard

```bash
node scripts/validate-compaction-fallbacks.mjs
```

This static guard fails if legacy bare `none` summary fallbacks return or if Workpoint/current-ask/session fallback hooks disappear.
