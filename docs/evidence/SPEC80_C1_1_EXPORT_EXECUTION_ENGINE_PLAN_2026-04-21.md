# SPEC80 C1.1 — Export Execution Engine Plan

Date: 2026-04-21
Bead: `focusa-yro7.3.1.1`
Labels: `documented-authority` + `planned-extension`

Purpose: replace current `not_implemented` export behavior with a concrete execution architecture aligned to docs/21.

## Current code-reality anchor

- CLI dataset commands return `status: not_implemented` and bail in non-dry-run mode.
  - `crates/focusa-cli/src/commands/export.rs`
- API exposes only `GET /v1/export/status` with `implemented: false`.
  - `crates/focusa-api/src/routes/training.rs`

## Proposed execution architecture

1. **Planner layer (CLI)**
- Parse command flags into canonical `ExportRequest`.
- Validate dataset-specific constraints.
- Dispatch to API run endpoint.

2. **Execution layer (API/core)**
- New endpoint: `POST /v1/export/run`.
- Orchestrates docs/21 phases: discovery → extraction → normalization → validation → export.
- Returns structured result for dry-run or write mode.

3. **Dataset builder layer (core)**
- `build_sft(request)`
- `build_preference(request)`
- `build_contrastive(request)`
- `build_long_horizon(request)`
- Shared filters + provenance enforcement.

4. **Writers + manifest layer**
- Deterministic JSONL writer first; parquet behind capability flag.
- Emit manifest and stats for non-dry-run.

## API contract draft

### Request
```json
{
  "dataset_type": "sft|preference|contrastive|long-horizon",
  "output": "path",
  "format": "jsonl|parquet",
  "filters": {"min_uxp":0.7,"max_ufi":0.3,"min_autonomy":0,"agent":"all","task":"all","since":null,"until":null},
  "dataset_flags": {},
  "dry_run": true,
  "explain": false
}
```

### Response
```json
{
  "status": "ok|partial|invalid_request",
  "dataset_type": "sft",
  "dry_run": true,
  "eligible_records": 0,
  "excluded_records": 0,
  "exclusion_reasons": [],
  "estimated_dataset_size_bytes": 0,
  "sample_schema_preview": {},
  "manifest": null,
  "stats": {"phase_ms": {"discovery":0,"extraction":0,"normalization":0,"validation":0,"export":0}}
}
```

## Implementation order

1. Introduce `ExportRequest/ExportResult` types in `focusa_core`.
2. Add API `POST /v1/export/run` and wire to engine.
3. Replace CLI `emit_not_implemented` paths with API call + output handling.
4. Keep `GET /v1/export/status` but switch `implemented=true` after run path ships.

## Evidence citations
- docs/21-data-export-cli.md (§1-§6)
- crates/focusa-cli/src/commands/export.rs
- crates/focusa-api/src/routes/training.rs
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§7, §20.1)
