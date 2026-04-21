# SPEC80 C3.2 — Metacognition CLI Contract Tests Plan

Date: 2026-04-21
Bead: `focusa-yro7.3.3.2`
Label: `documented-authority`

Purpose: define test coverage for metacognition CLI surface across current stub mode and future endpoint-backed mode.

## Phase A — stub-mode contract tests (current)

1. Each subcommand returns `status: not_implemented` in `--json` mode.
2. `planned_api_path` matches Appendix B route mapping.
3. `label` remains `planned-extension`.
4. Non-json mode prints consistent human summary.

## Phase B — endpoint parity tests (future)

1. Command/API binding tests
- each command hits correct `/v1/metacognition/*` endpoint.

2. Request shape tests
- CLI flags map to expected JSON request body fields.

3. Response passthrough tests
- `--json` output preserves API field names/types.

4. Error parity tests
- API and CLI return same typed error code family.

## Suggested test locations

- `crates/focusa-cli/tests/metacognition_cli_contract.rs`
- API route tests once `/v1/metacognition/*` exists

## Exit criteria

1. Stub-mode tests pass now (truthful planned behavior).
2. Endpoint parity suite added before switching stubs to live calls.
3. No command flag breakage across transition.

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§15, §20.1)
- crates/focusa-cli/src/commands/metacognition.rs
- docs/evidence/SPEC80_B2_1_CAPTURE_RETRIEVE_REFLECT_SCHEMAS_2026-04-21.md
