# SPEC80 C2.2 — Lineage CLI Contract Tests Plan

Date: 2026-04-21
Bead: `focusa-yro7.3.2.2`
Label: `documented-authority`

Purpose: define tests that guarantee lineage CLI parity remains machine-stable.

## Coverage set

1. Command/API binding tests
- each lineage subcommand hits expected `/v1/lineage/*` route.

2. JSON passthrough tests
- `--json` preserves API field names/types for head/path/tree/node/children/summaries.

3. Query behavior tests
- `--session-id` present: query appended.
- `--session-id` absent: canonical route path unchanged.

4. Error handling tests
- missing node id path returns typed API error and non-zero exit.
- unknown node id returns `CLT_NODE_NOT_FOUND` contract family.

5. Human output sanity tests
- non-json output remains concise and never used as machine contract.

## Suggested test locations

- CLI integration: `crates/focusa-cli/tests/lineage_cli_contract.rs`
- API parity fixture reuse: route fixtures from `crates/focusa-api/src/routes/capabilities.rs` test module

## Exit criteria

1. All lineage subcommands covered by command-binding tests.
2. JSON schema snapshots approved for six commands.
3. Failure-mode tests pass for invalid ids and route errors.

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§7, §15)
- crates/focusa-cli/src/commands/lineage.rs
- crates/focusa-api/src/routes/capabilities.rs
