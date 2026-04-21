# SPEC80 B1.1 — Head/Path Contract Finalization (Tree/Lineage)

Date: 2026-04-21
Bead: `focusa-yro7.2.1.1`
Label: `documented-authority`

Purpose: finalize machine-stable contracts for `focusa_tree_head` and `focusa_tree_path` from SPEC80 §14.1 and §15.

## 1) Shared typed error envelope

```json
{
  "ok": false,
  "error": {
    "code": "TREE_HEAD_UNAVAILABLE|SESSION_NOT_FOUND|CLT_NODE_NOT_FOUND|INVALID_ARGUMENT",
    "message": "string",
    "retryable": false,
    "details": {}
  },
  "trace": {
    "tool": "focusa_tree_head|focusa_tree_path",
    "api_path": "/v1/...",
    "cli_fallback": "focusa ... --json"
  }
}
```

## 2) `focusa_tree_head` contract

### Input
```json
{
  "session_id": "string (optional, defaults active session)"
}
```

### Success output
```json
{
  "ok": true,
  "data": {
    "session_id": "string",
    "pi_tree_node": "string",
    "clt_head": "string",
    "branch_id": "string",
    "observed_at": "ISO-8601"
  }
}
```

### Error codes
- `TREE_HEAD_UNAVAILABLE`
- `SESSION_NOT_FOUND`
- `INVALID_ARGUMENT`

### Binding + permission
- API: `GET /v1/lineage/head`
- CLI fallback: `focusa lineage head --json` (planned parity surface)
- Permission: `lineage:read`

### Ontology layers
- 8 (lineage/branch semantics)
- 12 (outcome/impact semantics)

## 3) `focusa_tree_path` contract

### Input
```json
{
  "clt_node_id": "string (optional; defaults clt_head)",
  "to_root": true
}
```

### Success output
```json
{
  "ok": true,
  "data": {
    "head": "string",
    "path": [
      {
        "clt_node_id": "string",
        "parent_clt_node_id": "string|null",
        "depth": 0,
        "branch_id": "string"
      }
    ],
    "branch_point": {
      "clt_node_id": "string|null",
      "depth": 0
    },
    "depth": 0,
    "divergence_summary": {
      "branch_distance": 0,
      "common_ancestor": "string|null"
    },
    "observed_at": "ISO-8601"
  }
}
```

### Error codes
- `CLT_NODE_NOT_FOUND`
- `INVALID_ARGUMENT`

### Binding + permission
- API: `GET /v1/lineage/path/{clt_node_id}`
- CLI fallback: `focusa lineage path <id> --json` (planned parity surface)
- Permission: `lineage:read`

### Ontology layers
- 8 (lineage/branch semantics)
- 7 (temporal freshness/decay semantics)
- 12 (outcome/impact semantics)

## 4) Contract invariants

1. `branch_id` and `clt_head` must be non-empty on successful `focusa_tree_head`.
2. `path[0].clt_node_id` must equal requested/defaulted head node on successful `focusa_tree_path`.
3. `depth` must equal `len(path)-1` when `to_root=true`.
4. No hidden mutation is allowed in either tool path (read-only behavior).

## 5) Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§6.1, §14.1, §15)
- docs/24-capabilities-cli.md (API/CLI parity + machine-usable output)
- crates/focusa-api/src/routes/capabilities.rs (lineage route authority anchor)
