# Focusa Data Retention, Backup, and Deletion Policy

Status: current policy baseline for persisted Focusa state. Spec 181 (`docs/181-focusa-continuous-rolling-backup-restore-and-retention-spec.md`) is the implementation authority; its 15-minute RPO remains breached until a conforming incremental mechanism is released and restore-proven.

## Store inventory

| Store | Default privacy class | Retention baseline | Backup baseline | Deletion baseline |
| --- | --- | --- | --- | --- |
| SQLite `focusa.sqlite` events/snapshots/peers | P2/P3; peer tokens P4 | Governed hot-window retention only after Spec 181 recovery gates | SQLite online full generation plus a conforming incremental chain; private local and off-host copies | Delete only through receipt-bound governed retention/erasure authority; never use broad daemon stop or direct live-file removal. |
| SQLite `event_hash_chain` | P1/P2 integrity metadata | Same lifetime as events | Back up with events to preserve audit continuity | Delete with events; chain alone is not enough to reconstruct payloads. |
| Focus State / Workpoint / Trajectory | P2 | Keep active and recent continuity unless operator clears project data | Included in SQLite/state backups | Deletion must remove both state snapshots and related event history if privacy erasure is required. |
| Metacognition / Predictions | P2/P3 | Keep reusable local learning while project remains active | Include in private backups | Provide project-scoped purge in future tooling before external sharing. |
| ECS/reference artifacts | P2/P3 | Keep referenced artifacts while evidence/handles are active | Back up object store with handle metadata | Garbage collect unreferenced objects; never persist P4 secrets. |
| Telemetry/traces | P1/P2 | Bounded by resource-mode trace retention where implemented | Optional; lower priority than canonical state | Safe to prune when not needed for active investigations. |
| Scratchpad `/tmp/pi-scratch` | P2/P3 transient | Temporary working notes | No required backup | May be purged after session/handoff evidence is captured. |
| Release proof/audit artifacts in `/tmp` | P1/P2 | Short-lived unless copied to docs/evidence | No required backup | Cleanup can move recoverably to trash; preserve canonical docs/evidence. |

## Backup rules

1. Recovery policy: 15-minute RPO, 2-hour RTO, 24 hourly/14 daily/8 weekly/12 monthly generations, weekly restore drill, and local plus off-host copies.
2. Full generations use SQLite’s online backup API without stopping the daemon; a full temporary snapshot every 15 minutes is non-conforming.
3. Never prune the last verified generation; pruning requires a newer verified generation, required off-host settlement, and newer restore proof.
4. Backups of SQLite, ECS objects, Workpoints, metacog, and predictions are private project backups.
5. Backups containing peer `auth_token` values or other credential material are P4 and require approved secret handling or encryption-at-rest.
6. Backup/restore preserves `event_hash_chain` rows with `events` rows and binds ECS/cold-export inventories.
7. Public release artifacts contain summaries/evidence handles, never raw state DBs.

## Deletion rules

1. No direct deletion of live Focusa data directories or SQLite files; daemon-stop workarounds are not deletion authority.
2. Backup generation pruning follows Spec 181 exact allowlists and durable planned/settled receipts; ordinary user-file cleanup remains recoverable where quota semantics permit.
3. Project privacy erasure must include events, snapshots, Workpoints, Trajectory records, metacog, predictions, ECS artifacts, and derived evidence files.
4. P4 secret exposure in persisted state requires immediate token rotation plus data purge/archive decision.
5. Deletion actions should be recorded as bounded evidence handles, not raw deleted payloads.

## Retention rules

1. Local-first active project state is retained by default for continuity.
2. Telemetry/traces should remain bounded and prunable under resource pressure.
3. Stale `/tmp` proof/log files should be moved recoverably by `focusa cleanup --safe`.
4. Long-lived public docs should only retain P0/P1/P2 summaries and redact P3/P4 payloads.

## Restore rules

1. Restore SQLite DB and ECS/reference store as a consistent set.
2. After restore, run health checks and event hash-chain verification once that CLI exists.
3. Treat restored Workpoint/Trajectory packets as canonical only when project_root plus continuity_id match.
4. Re-run security static gates after restore if code and state were restored together.

## Required future tooling

- `focusa doctor security` retention/backup posture report.
- Project-scoped purge/archive command with dry-run and recoverable output.
- Event hash-chain verification/backfill command.
- Encrypted backup guidance for peer tokens and P4-adjacent state.
