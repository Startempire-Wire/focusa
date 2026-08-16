# E2E verification — 2026-08-16 (whole ledger + all features)

Machine gates: scripts/e2e-live-route-matrix.mjs (live daemon) +
scripts/e2e-workspace-gate.sh (full workspace tests, pipefail-safe).

## Live route matrix — 18/18 PASS (verified against the deployed daemon)

| Check | Surface |
| --- | --- |
| health | daemon |
| constitution | #256 (hash-bound, served) |
| bg create/complete + wait | #311-family (completion envelope + output_tail) |
| completion claim evaluate | #276/#277 (deterministic verdict) |
| workstream migrate preview | #125 |
| callgraph validate/export jsonl/todo.txt | #254/#287 |
| callgraph item envelope | #289 |
| adapter registry | #254 slice 10 |
| fanout (3 lanes: 1 orchestrator/agent + 2 workers) | #312 |
| direction operations (record + envelope) | #291 |
| compaction epoch | #112 |
| error envelope parity | #261 |
| remote workspace bindings | #89 |
| closure validate gates on verdict | #276 settlement |
| silent sessions list + completion sweep | #195/#311 |
| workset ledger + transitions | #269/#271/#274 |
| bg wait + settlement idempotency | #311-family |

## Workspace test gate

cargo test --workspace --all-targets -- --test-threads=1 (serial) — green
(after the fanout-input fix; no false greens — set -euo pipefail).

## Extension surfaces

tsc green with the bg/fast-forward tools; runtime tests green after the
fixture-path parity fix; deployed extension synced back into
apps/pi-extension (parity restored).

## Security/perf fixes this round

Idempotent bg settlement (no re-settlement), bounded adapter
registration, single-connection bg wait, partition-path sanitization,
fail-closed snapshot loading. Full audit:
docs/current/IMPROVEMENT_AUDIT_2026-08-16.md.

## Release-readiness (Workstream/Workset/CallGraph)

Both gates join the release-tag revalidation (#280). The canon
big-squash release lands after the remaining issue closures.
