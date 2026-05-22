# Beads DB event-payload compaction — 2026-05-22

## Purpose

Reduce `.beads/beads.db` quota bloat without destroying historical Beads information.

## Preservation archive

- Exact pre-compaction database archive: `/root/focusa-beads-archives/20260522-150308/beads.db.precompact.zst`
- Archive SHA-256: `dc1ed8e40f376df1184feab6ff3226cc647260418e58e82a2003455dd80cc46e`
- Archive verification: `zstd -t` passed before the DB swap.
- Restore command: `zstd -dc /root/focusa-beads-archives/20260522-150308/beads.db.precompact.zst > beads.db.restore`

## Compaction method

- Preserved all operational rows and row counts.
- Preserved `events` metadata: `id`, `issue_id`, `event_type`, `actor`, `created_at`, and comments.
- Replaced `events.old_value` and `events.new_value` payload blobs with archive pointers containing event id, field, original byte count, archive path, and archive SHA.
- Current issue/comment/dependency state remains in the compact operational database.

## Verification

- `PRAGMA integrity_check` on compact DB: `ok`.
- `events` rows preserved: `25,317`.
- `issues` rows preserved: `1,324`.
- Archived event fields: `42,110`.
- Removed from operational event payloads: `3,188,366,093` old-value bytes and `3,170,530,577` new-value bytes.
- Compact operational event payload bytes: `10,361,499`.
- Final `.beads/beads.db` size: `19M`.
- Beads daemon restarted in local mode and `bd doctor` reported database integrity OK.

## Notes

The exact historical event payloads were not destroyed; they moved from hot SQLite rows to the compressed pre-compaction DB archive. Operational Beads commands now use a compact DB with archive pointers for old/new event snapshots.
