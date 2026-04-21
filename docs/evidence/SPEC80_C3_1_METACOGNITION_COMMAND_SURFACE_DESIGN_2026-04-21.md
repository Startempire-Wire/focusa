# SPEC80 C3.1 — Metacognition Command Surface Design

Date: 2026-04-21
Bead: `focusa-yro7.3.3.1`
Labels: `implemented-now` (CLI surface exists as stubs), `planned-extension` (endpoint-backed execution)

Purpose: lock the metacognition CLI command surface and transition contract from stub payloads to live API parity.

## Implemented command surface (stub mode)

- `focusa metacognition capture --kind --content [--rationale --confidence --strategy-class]`
- `focusa metacognition retrieve --current-ask --scope-tag ... [--k]`
- `focusa metacognition reflect --turn-range --failure-class ...`
- `focusa metacognition adjust --reflection-id --selected-update ...`
- `focusa metacognition evaluate --adjustment-id --observed-metric ...`

Code anchor:
- `crates/focusa-cli/src/commands/metacognition.rs`
- `crates/focusa-cli/src/main.rs`

## Stub contract (current)

Current behavior emits `status: not_implemented` payload with:
- `command`
- `planned_api_path`
- `reason`
- `label: planned-extension`

This is machine-honest and should remain until endpoints exist.

## Target parity bindings

- capture → `POST /v1/metacognition/capture`
- retrieve → `POST /v1/metacognition/retrieve`
- reflect → `POST /v1/metacognition/reflect`
- adjust → `POST /v1/metacognition/adjust`
- evaluate → `POST /v1/metacognition/evaluate`

## Transition policy

1. Keep command names/flags stable.
2. Replace stub emitter with API call path once endpoint is implemented.
3. Preserve typed error envelope from B4 mapping.
4. Preserve `--json` as machine contract surface.

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§6.2, §15)
- crates/focusa-cli/src/commands/metacognition.rs
- crates/focusa-cli/src/main.rs
