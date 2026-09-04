# 181 — Focusa Continuous Rolling Backup, Restore, and Retention Specification

**Status:** implementation in progress
**Authority:** GitHub #487; Bead `focusa-6r16j`; Sir V3 recovery baseline 2026-08-31
**Scope:** Focusa application data safety; no release or installed-acceptance claim

## 1. Goal

Provide application-consistent, continuously verified, bounded local and off-host recovery for the canonical Focusa runtime without stopping the daemon, creating a parallel scheduler/ledger, or deleting the last known-good generation.

## 2. Approved recovery policy

- Maximum data loss objective (RPO): 15 minutes.
- Recovery-time objective (RTO): 2 hours.
- Retention: 24 hourly, 14 daily, 8 weekly, 12 monthly recovery points.
- Local and off-host copies are required.
- Never delete the last verified generation.
- Restore drill: at least weekly.
- Backup freshness, integrity, off-host settlement, restore age, and target breaches must be machine-readable.

A 6.6 GB full database copy every 15 minutes is prohibited. A design that meets cadence only through uncontrolled full-copy storage or write amplification is non-conforming.

## 3. Measured storage input

The verified 5,910,990,848-byte SQLite snapshot compressed with zstd to 610,616,368 bytes: 10.33%. Compressed online full generations are viable for hourly/daily tiers. They do not alone satisfy the 15-minute RPO.

## 4. Current state and gap

- Canonical live DB: `<data_dir>/focusa.sqlite`, WAL mode.
- Source P1 now includes versioned contracts, online full snapshots, hashes, receipts, health, authenticated routes, and one daemon maintenance coordinator; released/installed/live acceptance is not established.
- The content-addressed snapshot-chunk prototype proves deduplication and reconstruction, but it first materializes a complete temporary SQLite snapshot. It is therefore explicitly experimental, disabled by default, and **cannot** satisfy the approved 15-minute RPO.
- Health remains `rpo_status=breach_incremental_not_implemented`; hourly full generations must not mask this gap.
- Source retention, restore, off-host, and event-retention gates exist, but no off-host rclone remote is configured for `wirebot`, no live generation has settled, and no live restore drill exists.
- cPanel/R2 archives remain secondary infrastructure copies, not application consistency proof.
- The installed nightly hygiene script is now read-only and fail-closed (SHA-256 `7131d646e1160cab4532fb78550a3d4454be7e1bf0e0062f5729d82749d7ce87`); it currently exits 1 on the installed daemon’s expected `404` because backup routes are not yet released.

## 5. Authority boundaries

1. Reuse the daemon lifecycle and one maintenance coordinator for backup, retention sweep, restore scheduling, and health projection.
2. Reuse Focusa evidence/error-envelope conventions. Generation manifests and receipts are append-only authority; no second database ledger.
3. SQLite online backup is the only accepted full-generation source while the daemon is live.
4. Backup work must never acquire the API write lock for filesystem/process I/O.
5. Existing event-chain and ECS authorities remain canonical. Backup code reads and binds them; it does not redefine them.
6. Off-host transport is an adapter behind the generation contract. cPanel aggregate counts never settle a generation.
7. `focusa stop` is forbidden while #486 remains open.
8. Retention cannot act until a newer verified generation exists and policy gates pass.

## 6. Versioned contracts

Implement serde contracts in `focusa-core`.

### 6.1 `focusa.backup_policy.v1`

Required fields:

- contract version;
- enabled;
- RPO/RTO seconds;
- full-generation cadence;
- incremental recovery-point cadence;
- hourly/daily/weekly/monthly counts;
- weekly restore cadence;
- local/off-host requirements;
- backup root;
- minimum free bytes and minimum free percent;
- maximum concurrent operations = 1;
- compression algorithm/level;
- incremental strategy identifier;
- policy digest.

Unknown or incomplete policy fails closed.

### 6.2 `focusa.backup_generation_manifest.v1`

Required fields:

- generation ID and immutable slot ID;
- generation kind: `full|incremental`;
- state: `staging|verified|off_host_settled|restore_proven|failed`;
- created/completed timestamps;
- source database canonical path, file identity, page size/page count;
- runtime version, schema version, platform, project root key, continuity scope when present;
- SQLite source data version and online-backup completion counters;
- event count, chain index/hash, persisted chain anchor;
- ECS object/handle inventory digest;
- cold-export inventory digest;
- artifact list with relative path, byte length, SHA-256, media type, compression;
- parent/full-base generation for incrementals;
- policy digest;
- manifest SHA-256;
- off-host settlement reference without credentials;
- latest restore receipt reference.

Staging manifests are never recovery points.

### 6.3 `focusa.backup_receipt.v1`

Phases: `planned|snapshot_started|snapshot_completed|verified|settled|failed`.

Bind run ID, generation ID, policy digest, source identity, timestamps, bytes, hashes, SQLite verification, event-chain/ECS binding, off-host status, error code, and bounded error text.

### 6.4 `focusa.backup_prune_decision.v1`

Bind policy digest, complete retained set, candidate set, tier assignment, last-known-good generation, restore-proven generations, off-host gates, disk state, decision, and reasons.

### 6.5 `focusa.backup_restore_receipt.v1`

Bind isolated target, source generation/chain, artifact hashes, decompression, `quick_check`, schema, event-chain proof, ECS/cold-export proof, reducer/replay checks, elapsed seconds, RTO result, and cleanup disposition.

## 7. Generation layout and atomicity

Configured backup root must not be inside the live data directory and must not resolve through a symlink.

```text
<backup_root>/
  policy.json
  locks/maintenance.lock
  staging/<generation_id>/
  generations/<generation_id>/
    manifest.json
    focusa.sqlite.zst
    ecs-inventory.json
    cold-export-inventory.json
    receipts.jsonl
  receipts/backup-receipts.jsonl
  receipts/prune-decisions.jsonl
  receipts/restore-receipts.jsonl
  health.json
```

- Create staging with mode 0700.
- Write artifacts to unique temporary files, sync files and directories, then atomically rename staging to generations.
- Manifest is written last within staging and must hash all other artifacts.
- Existing immutable generation IDs are idempotent only when all hashes match; conflict fails closed.
- Partial staging is retained as failed evidence until a newer verified generation exists; it is not counted as a recovery point.

## 8. Full online generation

1. Resolve the exact canonical DB path from `FocusaConfig.data_dir`.
2. Reject symlink, non-regular file, wrong parent, missing WAL mode, unsupported schema, or source identity drift.
3. Acquire the single maintenance lock and disk-headroom gate.
4. Open a separate read connection with bounded busy timeout.
5. Use rusqlite SQLite online-backup support to a staging DB without stopping the daemon.
6. Complete in bounded page steps; after repeated progress regression or 120 seconds, take one bounded final source lock so continuous writers cannot starve generation completion. Emit progress/failure receipts.
7. Run `PRAGMA quick_check` on the staging DB.
8. Read schema/event-chain anchors from the staged DB, not the moving source.
9. Hash ECS handles/objects and cold-export inventories without embedding P4 content.
10. Compress the verified staging DB with bounded zstd settings.
11. Hash compressed artifact and inventory files.
12. Atomically commit the generation manifest and directory.

Full-generation default cadence is hourly. It is base protection, not the 15-minute incremental claim.

## 9. Incremental 15-minute recovery points

Acceptance requires a bounded incremental mechanism that does not write another complete 6.6 GB snapshot every 15 minutes.

The implementation must select and prove one of:

- SQLite WAL/transaction replication with exact frame boundaries, checksums, base-generation binding, and replay; or
- a page-delta sink driven by a consistent SQLite snapshot and content-addressed changed-page packs.

Requirements:

- exact base generation;
- source page/schema identity;
- ordered sequence and replay protection;
- checksum per segment/pack and aggregate manifest;
- atomic cutover;
- recovery replay into isolation;
- WAL reset/checkpoint, concurrent writer, crash, duplicate, missing segment, and wrong-base tests;
- bounded disk/CPU/I/O measurements.

Until this is implemented and restored successfully, health must report `rpo_status=breach`; hourly full snapshots cannot mask the breach.

## 10. Retention

Tier assignment promotes references to the same immutable generation; it must not duplicate artifacts.

Pruning preconditions:

1. candidate is not staging/active;
2. candidate is not the last verified generation;
3. a newer verified generation exists;
4. required off-host settlement exists;
5. at least one retained restore-proven generation remains;
6. dependency closure preserves every incremental's base and segments;
7. manifest/artifact hashes reverify before deletion;
8. prune decision is durably written and synced before mutation;
9. deletion postcondition is verified and settled.

Any unknown field, hash mismatch, missing dependency, insufficient disk headroom, stale restore proof, or off-host outage retains the generation.

## 11. Restore drills

Weekly, restore the newest eligible chain into an isolated directory outside live data:

- verify every manifest/artifact hash;
- reconstruct/decompress DB;
- run SQLite `quick_check`;
- verify schema version and event chain head/anchor;
- verify ECS and cold-export inventory references;
- execute bounded read/replay/projection probes;
- prove no live-path access or mutation;
- record elapsed time and fail if over 2 hours;
- retain receipt before safely removing disposable restored bytes.

Never point a live daemon at an unproven restore.

## 12. Event retention sweep

Wire `runtime::event_retention` and the API route into compilation. The maintenance coordinator runs:

- startup dry-run/eligibility inspection;
- at most one mutating sweep per 24-hour slot;
- cold export before deletion;
- hash-chain anchor persistence;
- durable receipt and health status;
- no prune when backup freshness/verification is unhealthy.

Cold export files need hashes, compression, retention dependency, and off-host settlement before event deletion can be accepted.

## 13. Health and API

Routes:

```text
GET  /v1/backups/health
GET  /v1/backups/generations
POST /v1/backups/run
POST /v1/backups/verify
POST /v1/backups/prune
POST /v1/backups/restore-drill
```

Mutations require existing auth/admin middleware. `run`, `prune`, and `restore-drill` require idempotency keys and return accepted run/generation identities.

Health shape: `focusa.backup_health.v1` with policy digest, last full/incremental/verified/off-host/restore timestamps, ages, retained tier counts, active run, last failure, local/free bytes, and explicit RPO/RTO/off-host/restore status.

## 14. Exact source surfaces

Initial implementation:

- `crates/focusa-core/Cargo.toml` — enable rusqlite backup support.
- `crates/focusa-core/src/runtime/mod.rs` — export backup/event-retention modules.
- `crates/focusa-core/src/runtime/backup_contracts.rs` — versioned policy/manifest/receipt/health contracts.
- `crates/focusa-core/src/runtime/backup.rs` — snapshot, verification, manifests, health, and retention decisions.
- `crates/focusa-core/src/runtime/backup_io.rs` — shared fail-closed backup I/O, locks, hashing, and atomic commit primitives.
- `crates/focusa-core/src/runtime/backup_incremental.rs` — content-addressed page-delta recovery points for the 15-minute RPO.
- `crates/focusa-core/src/runtime/backup_restore.rs` — isolated generation reconstruction and semantic restore proof.
- `crates/focusa-core/src/runtime/backup_offhost.rs` — checksum-verified settlement through an operator-configured rclone remote.
- `crates/focusa-core/src/runtime/backup_retention.rs` — GFS planning, last-generation protection, and receipt-bound pruning.
- `crates/focusa-core/src/runtime/backup_tests.rs` — producer/failure tests.
- `crates/focusa-api/src/routes/mod.rs` — route exports.
- `crates/focusa-api/src/routes/backups.rs` — authenticated thin routes.
- `crates/focusa-api/src/routes/backup_maintenance.rs` — the daemon’s single backup/restore/off-host/retention maintenance coordinator.
- `crates/focusa-api/src/routes/events_retention.rs` — typed receipts and shared maintenance operation.
- `crates/focusa-api/src/server.rs` — one maintenance coordinator and route merge.
- `crates/focusa-cli/src/commands/backup.rs` and command registration — thin client parity.
- `docs/158-event-ledger-retention-and-db-size-architecture.md` — truthful implementation refs/status.
- `docs/current/DATA_RETENTION_BACKUP_DELETION_POLICY.md` — contract references.
- `tests/181-focusa-backup-runtime-contract-test.sh` — consumer/static wiring proof.

Live operations after source proof:

- `/etc/systemd/system/focusa-daemon.service.d/backup.conf` — exact backup root and writable-path hardening.
- `/data/wirebot/bin/focusa-nightly-hygiene.sh` — read-only authenticated health/receipt verification installed; rollback copy: `/root/claude_backups/configs/focusa-nightly-hygiene-20260831T180001Z`.

No other source or infrastructure file is in scope without amendment.

## 15. Environment contract

- `FOCUSA_BACKUP_ENABLED` (default false until root is configured).
- `FOCUSA_BACKUP_ROOT` (required when enabled).
- `FOCUSA_BACKUP_FULL_INTERVAL_SECS` (default 3600).
- `FOCUSA_BACKUP_INCREMENTAL_INTERVAL_SECS` (default 900).
- `FOCUSA_BACKUP_INCREMENTAL_STRATEGY` (default `required_not_implemented`; `experimental_full_snapshot_chunks_v0` never clears the RPO breach).
- `FOCUSA_BACKUP_MIN_FREE_BYTES`.
- `FOCUSA_BACKUP_MIN_FREE_PERCENT` (default 10).
- `FOCUSA_BACKUP_ZSTD_LEVEL`.
- `FOCUSA_BACKUP_OFF_HOST_REQUIRED`.
- `FOCUSA_BACKUP_OFF_HOST_REMOTE` (rclone remote name/object prefix only; never credentials).
- `FOCUSA_BACKUP_RESTORE_INTERVAL_SECS` (default 604800).

Invalid values fail closed and appear in health. Secrets and transport credentials are never accepted through these variables.

## 16. Failure tests

- concurrent live writers throughout online backup;
- source path/symlink/identity drift;
- snapshot interruption and restart adoption;
- disk full/quota/short write/fsync/rename failure;
- corrupt page, failed `quick_check`, compressed hash mismatch;
- manifest tamper and unknown contract version;
- overlap and duplicate slot dispatch;
- wrong project/continuity/schema/base;
- stale/missing ECS or cold export;
- incremental missing/duplicate/out-of-order/corrupt segment;
- off-host partial/wrong-generation receipt;
- prune with one known-good generation;
- prune dependency/base violation;
- restore timeout and replay mismatch;
- retention sweep without healthy backup;
- nightly wrong path/dependency failure/false-success.

## 17. Quality and proof commands

Long commands run through `focusa bg` on OVH where Cargo execution is required.

```text
cargo fmt --all -- --check
cargo test -p focusa-core runtime::backup -- --nocapture
cargo test -p focusa-core runtime::event_retention -- --nocapture
cargo test -p focusa-api backups -- --nocapture
cargo check -p focusa-api
cargo test -p focusa-cli backup -- --nocapture
bash tests/181-focusa-backup-runtime-contract-test.sh
```

Required proof layers: producer tests, API/CLI consumer tests, cross-version legacy-generation rejection, installed Linux/macOS/Windows path semantics, and live restore evidence.

## 18. Rollback

- Source rollback touches only files listed in section 14.
- Runtime config rollback disables new scheduling but preserves every generation/receipt.
- Never delete new generations during rollback.
- Restore the prior nightly script only if it remains disabled; never restore false-green timer behavior.
- Never stop the daemon through `focusa stop` while #486 is open.

## 19. Decomposition

### P1 — truthful full-generation foundation

- versioned contracts and policy parsing;
- online backup, `quick_check`, chain/ECS inventory, compression/hash, atomic manifest;
- health route explicitly showing 15-minute RPO breach;
- overlap/idempotency/headroom guards;
- producer and API consumer tests.

### P2 — incremental recovery points

- **Not accepted:** the source snapshot-chunk prototype deduplicates retained chunks but still materializes a full temporary snapshot every run.
- implement a direct page-delta sink/custom SQLite destination VFS or exact WAL replication that avoids full-copy churn;
- restore the conforming full+incremental chain and adversarially prove checkpoint/reset/crash behavior;
- only then permit health to clear the 15-minute RPO breach.

### P3 — retention/off-host/restore

- deterministic tier retention and dependency closure;
- exact off-host settlement adapter;
- weekly isolated restore drill and RTO evidence.

### P4 — event retention and hygiene

- wire startup/daily sweep behind healthy backup gate;
- bind cold exports to generations/off-host settlement;
- replace false-green nightly script.

### P5 — parity and installed proof

- CLI/Pi/Cockpit surfaces;
- cross-version and all-OS path semantics;
- canonical released/installed live evidence.

## 20. Acceptance boundary

P1 is useful protection but not #487 closure. Closure requires P1–P5, a recent local and off-host generation chain, a successful isolated restore inside two hours, a measured maximum recovery gap of 15 minutes or less, wired retention receipts, and zero false-green hygiene behavior.
