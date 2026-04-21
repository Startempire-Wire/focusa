# SPEC80 C1.1 — Export Execution Engine Plan

Date: 2026-04-21
Bead: `focusa-yro7.3.1.1`
Purpose: define the execution-engine plan to close the export runtime gap for `sft|preference|contrastive|long-horizon` in Spec80 Epic C.

## Authority
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§7, §20.1)
- crates/focusa-cli/src/commands/export.rs
- docs/21-data-export-cli.md

## Current code-reality checkpoint

- Export command surface exists for all dataset families.
- Non-dry-run execution paths currently return `not_implemented` / bail on pipeline not implemented.
- Gap is explicitly tracked in Spec80 §20.1 (`Export execution pipeline`).

## Execution engine architecture (planned)

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
   - JSONL writer first; parquet optional/feature-gated.
5. **Manifest + summary emitter**
   - emits counts, exclusions, schema fingerprint, and run metadata.

## Dataset-family execution requirements

| Dataset family | Minimum execution requirements | Initial done condition |
|---|---|---|
| `sft` | enforce `min_turns`, `require_success`, quality filters | writes records + manifest; non-empty when eligible |
| `preference` | pair ranking using `min_delta` + correction constraints | writes pairwise examples + preference labels |
| `contrastive` | branch divergence/abandonment-aware pair extraction | writes contrastive pair records with branch refs |
| `long-horizon` | session-length and transition thresholds | writes multi-step trajectory records |

## CLI behavior contract during rollout

- `--json --dry-run --explain` remains stable and backward compatible.
- First execution-enabled release for each dataset family must return deterministic success envelope:
  - `{ status:"ok", dataset_type, records_written, output, format, manifest }`.
- Failures must return typed error envelope (no plain-string bails):
  - `{ status:"error", code, reason, dataset_type }`.

## Delivery phases

1. **Phase E1: runtime scaffolding**
   - Shared execution context + selectors + manifest skeleton.
2. **Phase E2: SFT execution path**
   - first end-to-end write path enabled.
3. **Phase E3: Preference + contrastive**
   - pair-generation and branch-aware extraction.
4. **Phase E4: Long-horizon execution**
   - trajectory pipeline and final parity check.

## Blocking dependencies

- Canonical replay/source reader must provide stable turn/session traversal.
- JSON schema registry + compatibility policy (C4.1/C4.2) must gate envelope changes.

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md
- crates/focusa-cli/src/commands/export.rs
- docs/21-data-export-cli.md
- docs/evidence/SPEC80_CLI_JSON_SCHEMA_REGISTRY_2026-04-21.md
- docs/evidence/SPEC80_CLI_JSON_COMPATIBILITY_POLICY_2026-04-21.md
