# SPEC80 B1.2 — Snapshot/Restore/Diff Contract Finalization

Date: 2026-04-21
Bead: `focusa-yro7.2.1.2`
Label: `planned-extension`

Purpose: finalize contracts for `focusa_tree_snapshot_state`, `focusa_tree_restore_state`, and `focusa_tree_diff_context` from SPEC80 §14.1/§15/§17.

## 1) `focusa_tree_snapshot_state`

### Input
```json
{
  "clt_node_id": "string",
  "snapshot_reason": "branch-switch|checkpoint|pre-compaction|manual",
  "include_domains": ["decisions","constraints","failures","open_questions","work_loop"],
  "idempotency_key": "string (optional)"
}
```

### Success output
```json
{
  "ok": true,
  "data": {
    "snapshot_id": "string",
    "clt_node_id": "string",
    "created_at": "ISO-8601",
    "checksum": "sha256:string",
    "object_counts": {"decisions": 0, "constraints": 0, "failures": 0, "open_questions": 0}
  }
}
```

### Errors
- `SNAPSHOT_WRITE_DENIED`
- `SNAPSHOT_CONFLICT`
- `CLT_NODE_NOT_FOUND`

### Binding
- API: `POST /v1/focus/snapshots` (planned)
- CLI: `focusa state snapshot create --json` (planned)
- Permission: `state:write`

## 2) `focusa_tree_restore_state`

### Input
```json
{
  "clt_node_id": "string",
  "restore_mode": "exact|merge",
  "snapshot_id": "string (optional; defaults newest for clt_node_id)",
  "dry_run": false
}
```

### Success output
```json
{
  "ok": true,
  "data": {
    "restored": true,
    "snapshot_id": "string",
    "clt_node_id": "string",
    "restore_mode": "exact|merge",
    "conflicts": [
      {
        "field": "string",
        "base_value": "any",
        "incoming_value": "any",
        "resolution": "kept_base|applied_incoming|manual_required"
      }
    ],
    "post_restore_checksum": "sha256:string"
  }
}
```

### Errors
- `SNAPSHOT_NOT_FOUND`
- `RESTORE_CONFLICT`
- `AUTHORITY_DENIED`
- `CLT_NODE_NOT_FOUND`

### Binding
- API: `POST /v1/focus/snapshots/restore` (planned)
- CLI: `focusa state snapshot restore --json` (planned)
- Permission: `state:write`

## 3) `focusa_tree_diff_context`

### Input
```json
{
  "from_clt_node_id": "string",
  "to_clt_node_id": "string",
  "domains": ["decisions","constraints","failures","open_questions"],
  "include_unchanged": false
}
```

### Success output
```json
{
  "ok": true,
  "data": {
    "from_clt_node_id": "string",
    "to_clt_node_id": "string",
    "decisions_delta": {"added": [], "removed": [], "changed": []},
    "constraints_delta": {"added": [], "removed": [], "changed": []},
    "failures_delta": {"added": [], "removed": [], "changed": []},
    "open_questions_delta": {"added": [], "removed": [], "changed": []}
  }
}
```

### Errors
- `DIFF_INPUT_INVALID`
- `CLT_NODE_NOT_FOUND`

### Binding
- API: `POST /v1/focus/snapshots/diff` (planned)
- CLI: `focusa state snapshot diff --json` (planned)
- Permission: `lineage:read`

## 4) Shared invariants (from §9 + §17)

1. No cross-branch state leakage on restore.
2. `exact` restore must reproduce snapshot checksum.
3. `merge` restore must emit explicit `conflicts[]`; no silent overwrite.
4. Diff output must be deterministic for identical input pair ordering.
5. Compaction must preserve snapshot restore equivalence.

## 5) Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§14.1, §15, §17)
- docs/17-context-lineage-tree.md
- docs/24-capabilities-cli.md
