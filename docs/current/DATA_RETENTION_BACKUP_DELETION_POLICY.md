# Focusa Data Retention, Backup, and Deletion Policy

Status: current local-first policy baseline for persisted Focusa state. This policy implements the CIS Controls v8 data protection/recovery follow-up and complements `PERSISTED_STATE_PRIVACY_CLASSES.md`.

## Store inventory

| Store | Default privacy class | Retention baseline | Backup baseline | Deletion baseline |
| --- | --- | --- | --- | --- |
| SQLite `focusa.sqlite` events/snapshots/peers | P2/P3; peer tokens P4 | Keep until operator archive/delete; prune via future retention tooling | Back up with data directory; protect as private local data | Delete by stopping daemon and removing/archiving data dir; peer tokens require secure handling. |
| SQLite `event_hash_chain` | P1/P2 integrity metadata | Same lifetime as events | Back up with events to preserve audit continuity | Delete with events; chain alone is not enough to reconstruct payloads. |
| Focus State / Workpoint / Trajectory | P2 | Keep active and recent continuity unless operator clears project data | Included in SQLite/state backups | Deletion must remove both state snapshots and related event history if privacy erasure is required. |
| Metacognition / Predictions | P2/P3 | Keep reusable local learning while project remains active | Include in private backups | Provide project-scoped purge in future tooling before external sharing. |
| ECS/reference artifacts | P2/P3 | Keep referenced artifacts while evidence/handles are active | Back up object store with handle metadata | Garbage collect unreferenced objects; never persist P4 secrets. |
| Telemetry/traces | P1/P2 | Bounded by resource-mode trace retention where implemented | Optional; lower priority than canonical state | Safe to prune when not needed for active investigations. |
| Scratchpad `/tmp/pi-scratch` | P2/P3 transient | Temporary working notes | No required backup | May be purged after session/handoff evidence is captured. |
| Release proof/audit artifacts in `/tmp` | P1/P2 | Short-lived unless copied to docs/evidence | No required backup | Cleanup can move recoverably to trash; preserve canonical docs/evidence. |

## Backup rules

1. Backups of `data/`, SQLite files, ECS objects, Workpoints, metacog, and predictions are private project backups.
2. Backups containing peer `auth_token` values or other credential material are P4 and must use approved secret handling or encryption-at-rest.
3. Backup/restore must preserve `event_hash_chain` rows with `events` rows to keep audit continuity.
4. Public release artifacts should contain summaries/evidence handles, not raw state DBs.

## Deletion rules

1. No deletion of Focusa data directories while the daemon is running.
2. Prefer recoverable archive/trash moves before permanent deletion.
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
