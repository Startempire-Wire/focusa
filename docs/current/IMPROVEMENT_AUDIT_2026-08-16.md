# Improvement audit — 2026-08-16 (security + performance first)

Ranked findings from the E2E verification pass + surface review.
Fixes landed this round marked [FIXED].

## Security (top)

1. [FIXED] bg completion was re-settlable — forged/duplicate completions
   could overwrite a settled job. Now idempotent: completed_at set →
   `already_settled`, never re-settled.
2. [FIXED] Adapter registry accepted unbounded/empty registrations that
   feed ROUTING decisions. Now bounded (≤64 capability refs, non-empty
   identity).
3. [FIXED earlier] partition_paths could escape the data dir via absolute
   workstream-id components (sanitized).
4. [FIXED earlier] load_state silently started fresh over unparsable
   canonical snapshots (fail-closed now).
5. [NOTED] error envelopes carry raw error strings — acceptable for the
   loopback-only daemon; redaction for remote surfaces lands with #299.
6. [NOTED] direction/adapter/fanout routes are loopback-trust-bound;
   cross-host authorization arrives with #299 (credential authority).

## Performance (top)

1. [FIXED] bg wait route opened a new SQLite connection every 500ms poll —
   now one reused connection per wait.
2. [NOTED] Remote build host serializes every check on one lock — the
   dominant throughput bottleneck; parallel runners keyed by target dir
   would cut verification latency ~5x (infra change, tracked).
3. [NOTED] Checkpoint persistence serializes the full 25MB state each
   cycle — inherent to the snapshot model; the daily retention sweep
   bounds growth (already landed).
4. [NOTED] route_frame/team are O(frames×adapters) with hash sets —
   fine at current scale; indexing only if the registry grows past ~100.

## Consistency / half-baked surfaces (e2e-driven)

1. [FIXED] Deployed extension ≠ repo apps/pi-extension (session/tools/
   turns drifted) — parity tests failed; synced back into the repo.
2. [FIXED] Three extension tests had stale fixture paths from the old
   /root/.pi/agent layout — now FOCUSA_REPO_ROOT-aware.
3. [FIXED] closure_validate previously passed unconditionally — gated on
   the #276 verdict.
4. [IN FLIGHT] daemon rebuild carrying the closure gating (last matrix
   check pending deploy).

## False-green discipline (hard-won this round)

`cmd 2>&1 | tail -N && echo GREEN` masks failures — the pipe's exit is
tail's, not cmd's. Every gate chain must use `set -o pipefail` (or run
the command with full logging + an explicit EXIT marker). Two false
greens were caught this round (workspace gate, daemon build) and both
chains were replaced with pipefail-safe scripts.

## Deslop first run (2026-08-16, v0.32.0)

- Before: 26.2% duplicated (98k/374k LOC), top clone = a committed
  6.8MB extension backup (26k-node identical copy) — REMOVED.
- Generated/fixture surfaces report-hidden (tool-contracts registry,
  reducer test fixtures) per the generated-code philosophy.
- After: 18.2% duplicated (61k/338k LOC) — under the 20% committed
  ceiling. This is the honest baseline.
- The toolResult constructor (pi-native shape) now backs the six new
  tools; the remaining 102 `} as any;` escapes in pre-existing tools
  are the tracked convergence target.

## Release-readiness for the Workstream/Workset/CallGraph release

- e2e-live-route-matrix.mjs gates every bug-fix + feature surface
  (17/18 live; the last resolves with the closure-gating deploy).
- Workspace test gate running (all-targets, serial).
- These gates join the release-tag revalidation audits (#280).
