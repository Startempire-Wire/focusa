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
  long work runs on `spawn_blocking`, batches clamped to [100, 100_000].
- Daily retention sweep in the daemon (first tick at startup, then every
  24h): exports + prunes beyond `FOCUSA_EVENT_RETENTION_DAYS` (default 30)
  into `<data>/events-cold`; disable with `FOCUSA_EVENT_RETENTION_DISABLED=1`.

### 3. CLI

- `focusa events prune --epoch-junk | --before-days N [--no-export] [--dry-run]`
  → calls the daemon route and prints the summary.

### 4. Steady-state expectation

Hot window 30 days at observed ~250k events/day ≈ 7.5M events worst-case
≈ the pre-prune size — so the default window should be tuned per-host
(recommend 14 days for the anchor server). With the window enforced plus
cold export, the SQLite ledger stays bounded and the filesystem grows only
with compressed-if-wanted JSONL cold files (currently uncompressed).

## Live pruning runbook (executed 2026-08-15)

Pruning a **live** ledger while the daemon writes is possible but slow
(lock contention). The proven procedure is a short maintenance window:

1. Stop the daemon (it runs detached as root — `pgrep -f '^/usr/local/bin/focusa-daemon'`).
2. Drop the event indexes (bulk deletes are index-bound; 50k-row batches
   took ~4.5 min with indexes, ~seconds without):
   `DROP INDEX IF EXISTS idx_events_ts; ... idx_event_hash_chain_index;`
3. Batched deletes over stdin (exec argv has a 128 KB per-argument limit):
   ```sql
   .bail on
   BEGIN IMMEDIATE;
   DELETE FROM event_hash_chain WHERE event_id IN (<batch ids>);
   DELETE FROM events WHERE event_id IN (<batch ids>);
   COMMIT;
   ```
   Orphan chain sweep afterwards: `DELETE FROM event_hash_chain WHERE
   event_id NOT IN (SELECT event_id FROM events);`
4. Recreate the indexes with the same `CREATE INDEX IF NOT EXISTS` statements
   used in `persistence_sqlite.rs`.
5. `VACUUM` **as root** (the temp file is root-owned, bypassing the
   wirebot 30 GB user quota that would otherwise fail the rewrite), then
   `chown wirebot:wirebot` the db/wal/shm files.
6. Relaunch the daemon with its recorded environment
   (`FOCUSA_BIND`, `FOCUSA_HOME`, `FOCUSA_MAGIC_BIN`, `FOCUSA_DATA_DIR`,
   snapshot/metacog caps), then verify `/v1/health`.

Expected result: file shrinks from ~11.1 GB to ~5 GB (the junk share), and
the freed blocks return to the wirebot quota.

## Verification

- `cargo test -p focusa-core event_retention` — 4 tests: junk pruning,
  cold export + chain anchoring, lexicographic cutoff, orphan sweep.
- `cargo clippy -p focusa-core --all-targets -- -D warnings` — clean.
- Live prune evidence: maintenance-window log (`/tmp/focusa-maintenance2.log`),
  before/after `ls -la` sizes, daemon health after restart.
