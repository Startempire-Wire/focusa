# SPEC80 C4.1 — CLI JSON Schema Registry

Date: 2026-04-21
Updated: 2026-05-23
Bead: `focusa-yro7.3.4.1`
Purpose: define machine-stable JSON schema registry for CLI surfaces consumed by tool wrappers under Spec80 Epic C.

## Authority
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§7, §15, §20.1)
- crates/focusa-cli/src/commands/lineage.rs
- crates/focusa-cli/src/commands/metacognition.rs
- crates/focusa-cli/src/commands/export.rs
- crates/focusa-api/src/routes/training.rs

## Registry conventions

Each registered schema includes:
- `schema_id`
- `command`
- `status_class` (`implemented-now|planned-extension`)
- `required_fields`
- `compatibility_policy` (major/minor/patch expectations)

## Schema registry entries

### 1) `cli.lineage.tree.v1`
- `command`: `focusa --json lineage tree [--session-id <id>]`
- `status_class`: `implemented-now`
- `required_fields`:
  - `session_id` (nullable)
  - `root`
  - `head`
  - `nodes` (array)
  - `total` (number)
- `compatibility_policy`:
  - minor/patch may add optional fields
  - listed required fields are non-removable in v1

### 2) `cli.lineage.head.v1`
- `command`: `focusa --json lineage head [--session-id <id>]`
- `status_class`: `implemented-now`
- `required_fields`:
  - `session_id` (nullable)
  - `head`

### 3) `cli.metacognition.not_implemented.v1`
- `command`: `focusa --json metacognition <capture|retrieve|reflect|adjust|evaluate> ...`
- `status_class`: `planned-extension`
- `required_fields`:
  - `status` (must be `"not_implemented"`)
  - `command` (capture|retrieve|reflect|adjust|evaluate)
  - `planned_api_path`
  - `reason`
  - `label` (must be `"planned-extension"`)
- `compatibility_policy`:
  - envelope shape is stable until first implemented API release
  - transition to implemented payload requires new schema id

### 4) `cli.export.status.v1`
- `command`: `focusa --json export status`
- `status_class`: `implemented-now`
- `required_fields`:
  - `status` (`"ready"` when export API is available)
  - `implemented` (boolean)
  - `dataset_types` (array; includes `sft`, `preference`, `contrastive`, `long-horizon`)
  - `supported_formats` (array; includes `jsonl`, `parquet`)
  - `history_count` (number)
  - `last_export_at` (nullable)
  - `reason` (nullable)
- `compatibility_policy`:
  - minor/patch may add optional readiness/provenance fields
  - status payload must remain export-pipeline status, not contribution queue status

### 5) `cli.export.dataset.v1`
- `command`: `focusa --json export <sft|preference|contrastive|long-horizon> --dry-run --explain ...`
- `status_class`: `implemented-now`
- `required_fields`:
  - `status` (`"ok"` on successful export planning/execution)
  - `dataset_type`
  - `dry_run`
  - `explain`
  - `output`
  - `format`
  - `eligible_records`
  - `excluded_records`
  - `estimated_dataset_size_bytes`
  - `sample_schema_preview`
  - `exclusion_reasons` (array)
  - `manifest` (object)
  - `records` (array)
- `compatibility_policy`:
  - dry-run never writes output files
  - non-dry-run writes the requested dataset plus `<output>.manifest.json`
  - records may be empty when no eligible events exist, but the envelope remains machine-readable

## Compatibility note

This registry is scoped to currently tool-consumed/parity-critical CLI JSON surfaces in Epic C and does not yet cover all CLI domains.

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md
- tests/spec80_epic_c_lineage_cli_parity_test.sh
- tests/spec80_cli_metacognition_contract_test.sh
- tests/cli_contract_regression_test.sh
- tests/spec80_impl_export_execution_contract_test.sh
- tests/spec80_impl_parquet_export_support_test.sh
