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

## 9. Verified deployment topology

Audit date: 2026-07-28. These are observed boundaries, not inferred defaults.

| Surface | KnownHost production VPS (KH) | OVH service host |
|---|---|---|
| Role | Pi/Focusa authority, GitHub deploy runner, local agent-kb cache, central audit | intended agent-kb master, PostgreSQL, Mem0 and private AI services |
| agent-kb-api | current journal-capable binary on loopback `127.0.0.1:8791` | July-12 master binary on Tailscale; no journal route |
| SQLite | `/var/lib/agent-kb-api/agent-kb.sqlite` local document index | independent `/var/lib/agent-kb-api/agent-kb.sqlite` master index |
| Journal | 27+ local events in hash-linked JSONL | absent |
| Focusa daemon | authoritative production daemon and learning API | non-authoritative/stale; must not receive release authority implicitly |
| Release execution | self-hosted GitHub runner label `focusa-deploy` | none |
| Connectivity | reaches OVH SSH and Tailscale IPv6 relay | IPv4 API listener is not reachable from KH; IPv6 relay is reachable |
| Disk pressure | 98% used at audit | 92% used at audit |

Observed faults that block cross-server canonicality:

1. `agent-kb-local.service` restart-loops because it passes unsupported `--master-url` while another service owns port 8791.
2. `agent-kb-local-cache` claims mirror refresh but only writes a timestamp; it performs no replication.
3. `agent-kb-refresh.timer` restarts the API every five minutes, introducing avoidable availability gaps.
4. No dedicated release-journal backup or remote replica exists.
5. OVH and KH SQLite indexes are independent and expose different generations.

Remediation applied after the audit:

- journal-capable agent-kb-api parity deployed to OVH and KH
- 29 existing events, 6 release rows and 9 problem rows replicated with zero pending acknowledgements
- duplicate KH API service and stamp-only refresh/cache timers disabled
- SQLite WAL projections active on both hosts
- daily bounded backups active on both hosts; isolated restore rebuilt all 29 events
- bounded cleanup reduced KH disk to 88.8% and OVH to 89.7%; the dual-server resource gate passed before and after v0.9.139
- v0.9.139 finalized with 60 uploaded assets, green CI/Release/Deploy workflows, live daemon version 0.9.139, and all five planned predictions scored 1.0
- both journal projections reached 39 events with zero replication backlog; recovered resource and dependency failures remain retained as future recurrence lessons

## 10. Target dual-server storage architecture

### 10.1 OVH master

OVH is the canonical release-journal write authority after parity deployment and health proof. It stores:

- hash-linked append-only JSONL authority
- SQLite WAL projection for bounded query and trend analysis
- replication acknowledgements and source-host identity
- backup snapshots and restore receipts

### 10.2 KH release outbox and read cache

KH owns release execution and therefore writes first to a durable local outbox. It then replicates idempotently to OVH over verified Tailscale IPv6. KH retains a read projection and the unacknowledged outbox, not an independent competing authority.

Required states: `local_durable`, `replication_pending`, `master_accepted`, `projection_applied`, `backup_verified`. Finalization requires `master_accepted` unless an explicitly documented degraded spool policy is active.

### 10.3 SQLite projection

Minimum tables:

- `release_events`: event ID/hash, release ID, phase, sequence, timestamps, source host, payload JSON, previous hash
- `release_runs`: tag, commit, channel, lifecycle status, plan/final event refs
- `release_metrics`: metric name, protocol, value, unit, comparison direction, baseline release
- `release_problems`: failure fingerprint, stage, diagnosis, impact, recovery, recurrence count
- `release_predictions`: Focusa prediction ID, confidence, predicted/actual outcome, score
- `release_lessons`: Focusa metacog capture/reflection/adjustment refs and reuse count
- `release_integrations`: GitHub run, Focusa evidence, audit receipt, backup and replication refs
- `release_replication_state`: source/master sequence, lag, attempts, last acknowledgement

SQLite uses WAL, foreign keys, bounded busy timeout, indexed release/phase/time/fingerprint fields, and transactional projection. JSONL remains the audit authority; SQLite is rebuildable.

## 11. Software and integration boundaries

- **agent-kb-api (Go):** authenticated append/query, JSONL authority, SQLite projections, replication and grouped history.
- **Focusa daemon (Rust, KH):** Predictions, evaluation, Metacognition, adjustments, evidence and trajectory authority.
- **Focusa release client (Python):** plan/benchmark/progress/problem/final publication and durable outbox.
- **GitHub Actions/CLI:** exact-commit CI, Release, signed assets and KH Deploy run evidence.
- **KH self-hosted runner:** production installation and post-install trust proof.
- **Tailscale:** private KH↔OVH replication; IPv6 is the verified API path until IPv4 routing is repaired.
- **central audit:** receives bounded mutation and replication receipts; never substitutes for the journal.
- **PostgreSQL/Mem0 on OVH:** available integrations but not release-journal authority in v1.
- **systemd:** one agent-kb service per host plus bounded replication/backup timers; duplicate restart loops are prohibited.

## 12. Predictive metacognitive learning loop

The journal must change future behavior rather than merely accumulate incidents.

1. **Retrieve before planning:** query prior problem fingerprints, Metacog lessons, adjustment outcomes and prediction calibration for the project/release protocol.
2. **Predict before risk:** record predictions for trigger compatibility, benchmark success, CI, signed asset completeness, Deploy and production health.
3. **Settle after evidence:** evaluate each prediction with exact workflow, asset or production evidence.
4. **Learn from each distinct failure:** capture a Focusa Metacog lesson and create or revise a reusable prevention adjustment.
5. **Guard recurrence:** the next plan lists retrieved lesson IDs and executes their prevention checks before immutable tagging.
6. **Measure reuse:** final events record lessons retrieved, guards applied, repeated failure fingerprints, avoided failures and adjustment effectiveness.
7. **Promote only proven learning:** adjustments become canonical release gates only after outcome evaluation shows improvement.

A repeated failure fingerprint without a retrieved lesson and explicit prevention guard blocks the next tag. Prediction and Metacog refs are stored in agent-kb journal events; Focusa remains their canonical learning authority.

## 13. Resource, backup and availability gates

- Warn at 85% disk usage; block benchmark/build/release mutation at 90% until bounded cleanup or capacity recovery is evidenced.
- Bound JSONL event size, API query count, SQLite WAL growth and local outbox retention.
- Back up JSONL plus SQLite snapshot on both hosts; periodically restore into an isolated directory and verify hashes/projections.
- Expose master/cache generation, replication lag, oldest pending event, backup age and hash-chain health.
- API refresh uses online reindex or atomic projection swap; five-minute unconditional service restarts are prohibited.
- Release completion requires journal master acknowledgement, backup health and no disk-pressure block.

## 14. Failure and rollback

Before publication, normal Git rollback may remove unshipped Focusa changes. API events remain append-only and receive a correction/cancellation event if a planned release is abandoned. After publication, never rewrite tags, assets, or journal history. API unavailability blocks canonical finalization unless the event is durably spooled and later acknowledged with the original event ID. Server-role changes, service stops, firewall changes and destructive cleanup follow the operator confirmation and backup rules in `/root/.agent-kb/SAFETY_RULES.md`.
