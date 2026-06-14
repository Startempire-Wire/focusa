# Migration and Backup Policy

Focusa is local-first. Backups and migrations preserve operator-owned local state while protecting secrets, private paths, and append-only audit continuity.

## State classes

- daemon data directory
- SQLite/event store if configured
- Focus State, Workpoints, Trajectory, HLT ledger
- Evidence refs and proof handles
- Prediction and metacognition ledgers
- device pairing ledger and revoked-device records
- generated current docs and release proof artifacts
- adapter configuration and local environment contracts

## Backup checklist

1. Stop or quiesce the daemon when taking a consistent filesystem copy.
2. Snapshot daemon data directory and config.
3. Include append-only ledgers with their hash/audit metadata.
4. Include generated docs/proof artifacts when backing release evidence.
5. Store API tokens, pairing tokens, and OS keychain secrets with approved secret tooling, not plain docs.
6. Record backup timestamp, source host, Focusa version, and project identity.
7. Run restore smoke test on a non-production path when possible.

## Restore checklist

1. Verify target host/project identity before restoring.
2. Restore data/config to a private local path.
3. Restore secrets through secret storage, not raw transcript/chat.
4. Start daemon and run health/version checks.
5. Run Workpoint resume and Trajectory view to verify scope continuity.
6. Link restore evidence to a Workpoint.

## Migration checklist

- Export source project identity and continuity ids.
- Copy local state and append-only ledgers intact.
- Preserve `project_root + continuity_id` authority boundary.
- Re-pair devices instead of copying tokens across trust domains when practical.
- Regenerate current docs on the target version.
- Run release/version proof and focused smoke checks.

## Deletion / archive policy

Deletion is deliberate and operator-approved. Archive before deletion when state may be needed for audit, evidence, or rollback. Never delete `.beads/`, release proof bundles, or append-only ledgers during active project work without explicit operator approval.

## Related docs

- `DATA_RETENTION_BACKUP_DELETION_POLICY.md`
- `LOCAL_FIRST_DATA_MODEL.md`
- `TOKEN_AND_SECRET_HANDLING.md`
- `INSTALLER_UPDATE_POLICY.md`

## Proof

- Static guard: `tests/migration_backup_policy_static_test.sh`
