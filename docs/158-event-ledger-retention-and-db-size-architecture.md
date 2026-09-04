# Event-Ledger Retention and Database-Size Architecture

**Issues:** #309-era DB architecture follow-up (operator directive 2026-08-15)
**Components:** `focusa-core` retention engine, `focusa-api` prune route + daily sweep, `focusa-cli` prune command

## Diagnosis (2026-08-15, live production DB)

The daemon SQLite ledger (`$FOCUSA_DATA_DIR/focusa.sqlite`) had grown to
**11.1 GB** with 2,717,078 pages (4096 B/page), WAL journaling healthy,
freelist nearly empty (no recoverable bloat — the size was real data):

| Table | Rows | Note |
| --- | --- | --- |
| `events` | 6,260,101 | generic event ledger |
| `event_hash_chain` | 6,260,071 | per-event hash chain |
| everything else | < 1k total | silent-session tables etc. were empty |

Rowid milestones mapped to timestamps showed the story:

- rows 1 → ~3.26M: real events, 2026-07-07 → 2026-08-01 (~250k/day);
- rows ~3.26M → 6.26M: **placeholder junk** with `ts = 1970-01-01`
  (`temporal_action_envelope.v1` with `temporal:unavailable` /
  `action:unavailable` payloads) — a hot error loop of the retired temporal
  fallback writer.

The writer that emitted those placeholders no longer exists in the current
codebase (no `temporal:unavailable` matches; events stopped 2026-08-01),
so the junk is purely historical. Modern indexes (`idx_events_ts`, machine,
session, thread, chain) were already present.

## Architecture

### 1. Retention engine (`crates/focusa-core/src/runtime/event_retention.rs`)

- `prune_epoch_junk(conn, batch)` — deletes epoch-timestamped placeholder
  events + their chain rows in bounded short transactions (daemon-writer
  friendly). Never exported (junk carries no signal).
- `prune_before(conn, cutoff, export_dir, batch)` — exports events older than
  the hot window to append-only JSONL cold files
  (`<data>/events-cold/events-cold-YYYYMMDD.jsonl`), deletes them from the
  hot ledger, then anchors the hash chain and returns freed pages via
  incremental vacuum.
- `anchor_hash_chain(conn, keep=2000)` — drops chain rows below
  `max(chain_index) - keep` and persists the head checkpoint hash in `meta`
  (`event_chain_anchor`), so forward integrity stays provable after pruning.
- `incremental_vacuum(conn, pages)` — no-op unless the DB was created with
  `PRAGMA auto_vacuum=INCREMENTAL`.
- `retention_cutoff(days)` — ISO-8601 UTC cutoff (lexicographically
  comparable with `events.ts`).

### 2. Daemon surface

- `POST /v1/events/prune` — `{epoch_junk?, before_days?, export?, dry_run?, batch_size?}`;
  defaults to dry-run; mutation requires a fresh, restore-proven, off-host-settled
  Spec 181 recovery point and cold export. Long work runs on `spawn_blocking`,
  with batches clamped to [100, 100_000].
- The daemon’s single Spec 181 maintenance coordinator evaluates at most one
  mutating sweep per 24 hours. It writes planned/settled receipts and refuses
  event deletion while backup health is degraded.

### 3. CLI

- `focusa events prune --epoch-junk | --before-days N [--no-export] [--dry-run]`
  → calls the daemon route and prints the summary.

### 4. Steady-state expectation

Hot window 30 days at observed ~250k events/day ≈ 7.5M events worst-case
≈ the pre-prune size — so the default window should be tuned per-host
(recommend 14 days for the anchor server). With the window enforced plus
cold export, the SQLite ledger stays bounded and the filesystem grows only
with compressed-if-wanted JSONL cold files (currently uncompressed).

## Historical live pruning runbook (executed 2026-08-15; do not repeat)

The 2026-08-15 maintenance-window procedure stopped the daemon, dropped
indexes, issued direct deletes/VACUUM, and repaired ownership. It is retained
only as historical evidence. It is **not** current authority:

- `focusa stop` and broad daemon termination are forbidden while #486 remains open;
- direct root SQLite mutation/VACUUM is outside the governed retention contract;
- current mutation must flow through the authenticated route, backup health
  gate, cold-export fsync, bounded transactions, and durable receipts;
- missing backup/off-host/restore proof fails closed without deleting data.


## Verification

- `cargo test -p focusa-core event_retention` — 4 tests: junk pruning,
  cold export + chain anchoring, lexicographic cutoff, orphan sweep.
- `cargo test -p focusa-api retention` — route dry-run/cadence and recovery-gate coverage.
- `tests/181-focusa-backup-runtime-contract-test.sh` — backup-gated route, receipt, and cold-export fsync wiring.
- Released/installed live retention acceptance remains open; source tests are not production proof.
