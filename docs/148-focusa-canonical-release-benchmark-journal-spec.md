# 148 — Focusa Canonical Release Benchmark Journal Specification

**Status:** Active implementation specification  
**Owner:** Focusa release engineering  
**Canonical authority:** agent-kb-api `/v1/releases/journal`  
**Initial stable release:** `v0.9.136`

## 1. Purpose

Every canonical release must publish a structured lifecycle record to agent-kb-api so agents and operators can query how release quality, duration, reliability, and problems improve or degrade over time. The record begins before publication and is finalized only after production verification.

## 2. Authority and immutability

1. agent-kb-api owns the append-only canonical event ledger under its configured data directory.
2. Focusa is an authenticated event publisher and journal reader; repository projections are advisory exports only.
3. Existing events are immutable. Corrections append a new event referencing the superseded event.
4. Event identity is deterministic and idempotent. Reusing an event ID with different content fails closed.
5. Published tags and assets remain immutable.
6. Unknown historical measurements are `null` with `not_comparable`; they are never estimated retroactively.
7. Every accepted event receives an API-computed hash linked to the prior event hash.

## 3. API contract

### Publish

```http
POST /v1/releases/journal
Authorization: Bearer <agent-kb-token>
Content-Type: application/json
```

### Query

```http
GET /v1/releases/journal?project_id=focusa&release_id=focusa:v0.9.136&phase=final
GET /v1/releases/journal?project_id=focusa&view=releases&limit=50
```

Required event fields:

- `schema`: `agent-kb.release_journal.event.v1`
- `event_id`
- `release_id`
- `project_id`
- `tag`
- `phase`: `plan`, `benchmark`, `progress`, `problem`, `final`, or `correction`
- `sequence`
- `observed_at`
- `protocol_version`
- `estimates`
- `measurements`
- `problems`
- `comparison`
- `evidence_refs`

The API adds `received_at`, `previous_event_hash`, and `event_hash`, fsyncs the append, and returns a bounded receipt. `view=releases` groups events by release ID and projects planned estimates, final actuals, benchmark status, problem counts, and comparison classifications.

## 4. Required lifecycle events

### 4.1 Plan — before release mutation

Record before tag creation or push:

- estimated total elapsed time
- estimated CI, Release, and Deploy workflow durations
- expected asset count and required workflow count
- expected benchmark thresholds and release-gate score
- risk inventory and expected problem count
- estimate source: historical median, explicit operator estimate, or unavailable
- candidate commit and intended tag

### 4.2 Benchmark — before publication

Protocol v1 records:

1. Agent Intelligence eval: case count, category count, aggregate score, threshold, status.
2. Live performance proof: daemon health p95 and budget, passed/total runtime checks, status.
3. Final release gap gate status.
4. Release gate allow/block and score.
5. Proposed-tag version-surface verification.
6. Benchmark elapsed time and command evidence refs.

### 4.3 Progress and problem events — during release

Record stage start/completion timestamps for stamp, commit, tag, push, CI, Release, and Deploy. Every retry, failure, timeout, override, manual intervention, or unexpected result appends a `problem` event with:

- stage
- bounded diagnosis
- impact
- recovery action
- elapsed-time impact when known
- evidence refs

No-problem releases still record an empty problem list in the final event.

### 4.4 Final — after release finalization

Only after exact-tag CI, Release, Deploy, signed assets, checksums, and production health succeed, record:

- final commit, publication time, workflow run IDs/conclusions/durations
- total elapsed time and benchmark statistics
- asset count, checksum/signature status, production version
- all encountered problems and recovery outcomes
- actual-versus-estimate deltas
- historical comparison against the most recent protocol-compatible canonical release
- per-metric classification: `improved`, `degraded`, `unchanged`, or `not_comparable`

Each comparison contains current value, baseline value, delta, unit, direction, and baseline release ID. Aggregate prose without raw values is prohibited.

## 5. Historical bootstrap

The API journal may backfill immutable metadata for `v0.9.134-dev`, `v0.9.135-dev`, and `v0.9.136-dev`: publication time, commit, asset count, workflow conclusions/durations, deploy receipt, and known problems. Benchmark scores not measured under protocol v1 remain null and not comparable.

Historical backfill events are explicitly marked `source=historical_backfill`; they never claim contemporaneous estimates that did not exist.

## 6. Automation

`scripts/canonical-release-journal.py` is the authenticated Focusa client and benchmark normalizer. Canonical release scripts call it automatically:

```bash
python3 scripts/canonical-release-journal.py plan --tag v0.9.136 --channel stable
python3 scripts/canonical-release-journal.py benchmark --tag v0.9.136 --channel stable
python3 scripts/canonical-release-journal.py progress --tag v0.9.136 --stage tag-pushed
python3 scripts/canonical-release-journal.py problem --tag v0.9.136 --stage release --diagnosis "..."
python3 scripts/canonical-release-journal.py finalize --tag v0.9.136 --channel stable
python3 scripts/canonical-release-journal.py history --project-id focusa --limit 50
```

The client reads `AGENT_KB_API_URL` (default `http://127.0.0.1:8791`) and bearer credentials from `AGENT_KB_RELEASE_TOKEN`, `AGENT_KB_TOKEN`, `/etc/agent-kb/release-publisher.token`, or `/etc/agent-kb/token` in that order. The scoped publisher token is accepted only by the journal endpoint. Secrets never enter events or logs.

## 7. Stable v0.9.136 sequence

1. Verify exact candidate and absent stable tag.
2. Deploy and verify agent-kb-api journal endpoints.
3. Backfill immutable historical release metadata.
4. Verify the intended stable tag matches the Release workflow trigger before any immutable tag is created.
5. Stamp stable version surfaces and run protocol v1.
6. Publish `plan` and `benchmark` events before tag creation.
7. Commit benchmark tooling, docs, tests, and stable stamp.
8. Run final local gates; tag and push exactly once.
9. Publish progress/problem events while workflows run.
10. Verify CI, Release, Deploy, assets, signatures, and production.
11. Publish `final` event with actual-versus-estimate and historical comparison.
12. Query the grouped API history and capture the receipt as release evidence.

## 8. Acceptance criteria

- [ ] API rejects malformed, unauthorized, duplicate-conflicting, or out-of-sequence events.
- [ ] API appends hash-linked events durably and supports bounded filters plus grouped release history.
- [ ] Plan captures time/statistic estimates before release publication.
- [ ] Benchmark runs before publication and blocks on required-check failure.
- [ ] Pre-tag verification proves the Release workflow trigger matches the intended stable or preview tag.
- [ ] Problems during release are appended with diagnosis and recovery.
- [ ] Final event records all results and actual-versus-estimate deltas.
- [ ] Historical comparisons expose raw values and comparability boundaries.
- [ ] Release tooling publishes lifecycle events automatically when agent-kb-api is configured.
- [ ] Offline tests cover API append/query/idempotency and Focusa normalization.
- [ ] v0.9.136 is not complete until API final receipt, release workflows, and production all pass.

## 9. Failure and rollback

Before publication, normal Git rollback may remove unshipped Focusa changes. API events remain append-only and receive a correction/cancellation event if a planned release is abandoned. After publication, never rewrite tags, assets, or journal history. API unavailability blocks canonical finalization unless the event is durably spooled and later acknowledged with the original event ID.
