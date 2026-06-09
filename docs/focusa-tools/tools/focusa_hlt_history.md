# focusa_hlt_history

## Purpose

Read the append-only HLT (High-Level Trajectory) ledger for a project. Returns the exact history of HLT changes with timestamps, source, old/new values, and evidence refs.

Per Spec98/99: scope-bounded by `(project_root, continuity_id)`, no singleton, CRDT-grade events.

## When to use

- When agent needs to verify the exact HLT wording from previous sessions.
- After compaction/resume when HLT precision is critical.
- When reconstructing trajectory history for handover.
- Before changing HLT to see what was previously set.

## Example usage

```json
{
  "project_root": "/home/wirebot/focusa",
  "continuity_id": "focusa-cont-focusa-99e8217b-31fc-4cba-95fa-21e0783f1079",
  "limit": 20
}
```

## Expected result

```json
{
  "status": "completed",
  "project_root": "/home/wirebot/focusa",
  "continuity_id": "focusa-cont-...",
  "count": 5,
  "entries": [
    {
      "timestamp": "2026-06-08T12:00:00Z",
      "event_id": "019ea9f2-...",
      "project_root": "/home/wirebot/focusa",
      "continuity_id": "focusa-cont-...",
      "session_id": "pi-session-...",
      "old_hlt": "Maintain and improve Focusa within verified project scope",
      "new_hlt": "What WPUIAI desires to be: a self-driving WordPress...",
      "source": "trajectory_define_goal",
      "reason": "trajectory_goal_defined",
      "evidence_refs": []
    }
  ],
  "ledger_file": "/home/wirebot/.focusa/hlt-ledger/.../hlt.jsonl"
}
```

## Ledger file format

Each line in `hlt.jsonl` is a JSON object:

```json
{
  "timestamp": "2026-06-08T12:00:00Z",
  "event_id": "019ea9f2-...",
  "lamport_ts": 12345,
  "project_root": "/home/wirebot/focusa",
  "continuity_id": "focusa-cont-...",
  "session_id": "pi-session-...",
  "old_hlt": "previous HLT value or null",
  "new_hlt": "current HLT value",
  "source": "operator|focus_state|trajectory_define_goal|...",
  "reason": "optional reason",
  "evidence_refs": ["ref1", "ref2"]
}
```

## Scope rules

- `project_root` is **required** — HLT ledger is scoped to project.
- `continuity_id` is **optional** — filters entries to specific workstream.
- `limit` defaults to 50, max 500.
- File path is deterministic: `{data_dir}/hlt-ledger/{project_root_hash}/hlt.jsonl`

## Notes

- Ledger is append-only — old entries are never modified or deleted.
- Entries are ordered by timestamp (oldest first, most recent last).
- Per Spec98/99: no singleton, scope-bounded, CRDT-grade.