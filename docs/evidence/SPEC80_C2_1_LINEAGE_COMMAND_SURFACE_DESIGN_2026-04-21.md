# SPEC80 C2.1 — Lineage Command Surface Design

Date: 2026-04-21
Bead: `focusa-yro7.3.2.1`
Label: `implemented-now`

Purpose: lock CLI lineage command surface and machine-usable JSON expectations.

## Implemented command surface (code)

- `focusa lineage head [--session-id <id>]`
- `focusa lineage tree [--session-id <id>]`
- `focusa lineage node <clt_node_id>`
- `focusa lineage path <clt_node_id>`
- `focusa lineage children <clt_node_id>`
- `focusa lineage summaries [--session-id <id>]`

Code anchor:
- `crates/focusa-cli/src/commands/lineage.rs`
- `crates/focusa-cli/src/main.rs` command registration

## API parity bindings

- head → `GET /v1/lineage/head`
- tree → `GET /v1/lineage/tree`
- node → `GET /v1/lineage/node/{clt_node_id}`
- path → `GET /v1/lineage/path/{clt_node_id}`
- children → `GET /v1/lineage/children/{clt_node_id}`
- summaries → `GET /v1/lineage/summaries`

## JSON output contract policy

1. `--json` output must be API-response passthrough (no field renaming).
2. Non-json mode is human summary only; not machine contract surface.
3. CLI failures must return typed error envelope (to be harmonized with B4 mapping).
4. `session_id` query support must remain optional and stable for head/tree/summaries.

## Spec alignment update

Appendix B row corrections applied in SPEC80:
- `focusa_tree_head` CLI in current code: ✅ exists
- `focusa_tree_path` CLI in current code: ✅ exists

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§15, §19.2)
- crates/focusa-cli/src/commands/lineage.rs
- crates/focusa-cli/src/main.rs
- crates/focusa-api/src/routes/capabilities.rs
