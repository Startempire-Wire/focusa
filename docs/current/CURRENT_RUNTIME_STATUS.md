# Current Runtime Status

<!-- GENERATED: scripts/generate-current-runtime-status. Do not edit by hand. -->

Generated: 2026-07-16T21:52:57Z
Version: 0.9.118-dev
Tool contracts: 105
Tool surface summary: [docs/current/generated/tool-surface-summary.md](docs/current/generated/tool-surface-summary.md)

## Current shipped functionality

- evaluations persist as first-class records for prediction/metacog readback and promotion.
- /v1/work-loop/health exposes dispatch readiness for work-loop diagnostics.
- ontology memory-pipeline promotions are documented and auditable.

## Release invariant inputs

- release stamp: docs/current/.release-version-stamp
- version consistency: scripts/verify-doc-version-consistency
- tool contracts: docs/current/focusa-tool-contracts.json
- proof command: focusa release prove --tag <tag>
