# SPEC80 B3.1 — Implemented Endpoint Binding Validation

Date: 2026-04-21
Bead: `focusa-yro7.2.3.1`
Label: `implemented-now`

Purpose: validate Appendix B rows marked API-implemented against route registrations in code.

## Validation matrix

| SPEC80 row | Expected API path | Code evidence | Status |
|---|---|---|---|
| `focusa_tree_head` | `GET /v1/lineage/head` | `crates/focusa-api/src/routes/capabilities.rs:504` | ✅ implemented |
| `focusa_tree_path` | `GET /v1/lineage/path/{clt_node_id}` | `crates/focusa-api/src/routes/capabilities.rs:507` | ✅ implemented |
| reflection anchor for metacog alignment | `/v1/reflect/run|history|status` | `crates/focusa-api/src/routes/reflection.rs:1231-1233` | ✅ implemented |

## Notes

- Appendix B correctly marks lineage head/path as existing API routes.
- Appendix B correctly marks metacognition namespace as planned while reflect namespace exists.
- CLI fallback for lineage/metacognition remains planned parity work.

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§15)
- crates/focusa-api/src/routes/capabilities.rs:504-509
- crates/focusa-api/src/routes/reflection.rs:1231-1233
