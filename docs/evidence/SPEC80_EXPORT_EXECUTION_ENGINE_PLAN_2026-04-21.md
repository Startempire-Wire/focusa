# SPEC80 C1.1 — Export Execution Engine Plan

Date: 2026-04-21
Bead: `focusa-yro7.3.1.1`
Purpose: record the export execution-engine plan and current implemented baseline for `sft|preference|contrastive|long-horizon` in Spec80 Epic C.

## Authority
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§7, §20.1)
- crates/focusa-cli/src/commands/export.rs
- docs/21-data-export-cli.md

## Current code-reality checkpoint

- Export command surface exists for all dataset families.
- `/v1/export/status` and `/v1/export/run` are endpoint-backed and implemented.
- CLI export writes JSONL/Parquet datasets plus manifest files for non-dry-run mode.
- Export envelopes/manifests include baseline eligibility, provenance, redaction, and quality metadata.
- Remaining maturity work is deeper training-data scoring/provenance semantics, not stub removal.

## Execution engine architecture (implemented baseline)

Core components:
1. **Selector layer**
   - Resolves dataset family and validates flags/filters.
2. **Session replay loader**
   - Streams eligible events/turns from canonical sources.
3. **Dataset builder modules**
   - `build_sft`
   - `build_preference`
   - `build_contrastive`
   - `build_long_horizon`
4. **Writer layer**
   - JSONL writer and Parquet `record_json` writer are implemented in the CLI.
5. **Manifest + summary emitter**
   - emits counts, exclusions, filters, dataset flags, redaction summary, quality summary, provenance completeness, and run metadata.

## Dataset-family execution requirements

| Dataset family | Minimum execution requirements | Initial done condition |
|---|---|---|
| `sft` | endpoint-backed turn extraction plus dataset flags, quality metadata, provenance, redaction | writes records + manifest; empty set is valid when no eligible turns exist |
| `preference` | adjacent-turn pair generation plus source-pair provenance | writes pairwise examples + manifest |
| `contrastive` | adjacent-turn contrastive generation plus source-pair provenance | writes contrastive pair records + manifest |
| `long-horizon` | ordered 3-turn trajectory chunks plus provenance | writes multi-step trajectory records + manifest |

## CLI behavior contract during rollout

- `--json --dry-run --explain` remains stable and backward compatible.
- First execution-enabled release for each dataset family must return deterministic success envelope:
  - `{ status:"ok", dataset_type, records_written, output, format, manifest }`.
- Failures must return typed error envelope (no plain-string bails):
  - `{ status:"error", code, reason, dataset_type }`.

## Delivery phases

1. **Phase E1: runtime scaffolding** — complete.
   - Shared execution context, selectors, manifest skeleton, and endpoint-backed run route exist.
2. **Phase E2: SFT execution path** — complete at baseline.
   - End-to-end dry-run/write path enabled through `/v1/export/run` and CLI writers.
3. **Phase E3: Preference + contrastive** — complete at baseline.
   - Pair-generation paths emit records with provenance/eligibility metadata.
4. **Phase E4: Long-horizon execution** — complete at baseline.
   - Ordered trajectory chunks emit records with provenance/eligibility metadata.

## Remaining maturity dependencies

- Deeper quality scoring should use richer outcome, correction, and verification signals beyond the current baseline heuristic.
- Provenance can be strengthened with stable source handles and schema fingerprints.
- JSON schema registry + compatibility policy (C4.1/C4.2) must gate envelope changes.

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md
- crates/focusa-cli/src/commands/export.rs
- docs/21-data-export-cli.md
- docs/evidence/SPEC80_CLI_JSON_SCHEMA_REGISTRY_2026-04-21.md
- docs/evidence/SPEC80_CLI_JSON_COMPATIBILITY_POLICY_2026-04-21.md
