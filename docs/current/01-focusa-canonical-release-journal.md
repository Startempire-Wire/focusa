# 01 — Focusa Canonical Release Journal

Focusa publishes every canonical release lifecycle to agent-kb-api. The journal is the structured authority for release estimates, benchmark measurements, execution problems, final results, and historical trends.

## Authority

- API: `GET/POST /v1/releases/journal`
- Project filter: `project_id=focusa`
- Schema: `agent-kb.release_journal.event.v1`
- Release ID: `focusa:<tag>`
- Benchmark protocol: `focusa.release_benchmark.v1`

The agent-kb-api append-only hash-linked ledger is canonical. GitHub releases, Actions runs, deploy receipts, benchmark artifacts, and production health are evidence sources linked from journal events.

## What each release records

1. **Plan before publication:** candidate commit, intended tag, estimated total time, workflow durations, expected artifacts, benchmark threshold, risks, and estimate source.
2. **Benchmark before publication:** Agent Intelligence score, health latency p95, runtime-check totals, release-gap status, release-gate score, version-surface consistency, and benchmark duration.
3. **Progress during execution:** stamp, commit, tag, push, CI, Release, and Deploy stage receipts.
4. **Problems during execution:** failures, retries, timeouts, interventions, impact, recovery, and evidence.
5. **Final verification:** actual timings and statistics, signed assets, production version, all problems, estimate deltas, and comparison with the preceding comparable release.

## Query current history

```bash
python3 scripts/canonical-release-journal.py history --project-id focusa --limit 50
```

Direct API query:

```bash
curl -fsS \
  -H "Authorization: Bearer $(cat /etc/agent-kb/token)" \
  "${AGENT_KB_API_URL:-http://127.0.0.1:8791}/v1/releases/journal?project_id=focusa&view=releases&limit=50" | jq .releases
```

## Interpret comparisons

Every comparable metric includes baseline, current value, delta, unit, and direction:

- `improved`
- `degraded`
- `unchanged`
- `not_comparable`

Lower is better for durations and problem counts. Benchmark scores use higher-is-better. Artifact count is reported with raw deltas; completeness checks remain the quality authority. Measurements from different protocols are not treated as equivalent.

## Historical bootstrap snapshot

The initial immutable metadata backfill establishes this pre-stable baseline:

| Release | Assets | Remote pipeline | Problems evidenced on exact commit | Timing vs prior |
|---|---:|---:|---:|---|
| `v0.9.134-dev` | 60 | 3152 s | 0 | not comparable; first sample |
| `v0.9.135-dev` | 60 | 1150 s | 0 | improved by 2002 s |
| `v0.9.136-dev` | 60 | 1638 s | 0 | degraded by 488 s |

Source: agent-kb-api historical events ending in hashes `837c0feb…`, `45d7d9d…`, and `bbe45f7a…`. Exact values remain queryable through `view=releases`; this table is a documentation projection, not authority.

## Historical honesty

Older releases have GitHub timing, asset, deploy, and exact-commit workflow evidence without protocol-v1 benchmark scores or contemporaneous pre-release estimates. Those fields remain null and `not_comparable`. The journal never reconstructs estimates after outcomes are known. A zero historical problem count means no failed exact-commit canonical workflow was found by the backfill protocol; it does not prove that no human difficulty occurred.

## Release completion rule

A release is not finalized until:

- candidate benchmark passed before publication
- exact-tag CI passed
- Release workflow passed
- Deploy workflow passed
- required assets, checksums, and signatures exist
- production reports the intended version
- final journal event was accepted and can be queried by release ID

Full contract: `docs/148-focusa-canonical-release-benchmark-journal-spec.md`.
