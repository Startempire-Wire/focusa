# SPEC80 B4.1 — Typed Error Envelope Mapping

Date: 2026-04-21
Bead: `focusa-yro7.2.4.1`
Labels: `documented-authority` + `planned-extension`

Purpose: define canonical error envelope and map tool-level error codes for API/CLI parity.

## Canonical envelope

```json
{
  "ok": false,
  "error": {
    "code": "UPPER_SNAKE_CASE",
    "category": "input|auth|availability|conflict|budget|internal",
    "message": "string",
    "retryable": false,
    "details": {}
  },
  "meta": {
    "tool": "string",
    "api_path": "string",
    "cli_cmd": "string|null",
    "timestamp": "ISO-8601"
  }
}
```

## Mapping rules

1. API and CLI must emit same `error.code` for equivalent failure classes.
2. `category` is mandatory and drives fallback behavior.
3. `retryable=true` only for availability/transient internal states.
4. Unknown errors normalize to `INTERNAL_UNCLASSIFIED` with preserved raw detail.

## Primary code groups

- Lineage read: `TREE_HEAD_UNAVAILABLE`, `SESSION_NOT_FOUND`, `CLT_NODE_NOT_FOUND`, `DIFF_INPUT_INVALID`
- Snapshot state-write: `SNAPSHOT_WRITE_DENIED`, `SNAPSHOT_CONFLICT`, `SNAPSHOT_NOT_FOUND`, `RESTORE_CONFLICT`
- Metacognition: `CAPTURE_SCHEMA_INVALID`, `RETRIEVE_BUDGET_EXCEEDED`, `REFLECT_INPUT_INVALID`, `ADJUST_POLICY_CONFLICT`, `EVAL_INPUT_INVALID`

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§14, §15)
- docs/24-capabilities-cli.md
