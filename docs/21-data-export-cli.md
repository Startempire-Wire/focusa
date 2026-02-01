# docs/21-data-export-cli.md — Focusa Data Export CLI Specification (AUTHORITATIVE)

This document specifies the `focusa export` CLI used to generate training datasets
from local Focusa data stores.

The CLI is **read-only**, **deterministic**, and **auditable**.

---

## 1. Command Overview

```bash
focusa export <dataset_type> [options]
```

Where `<dataset_type>` is one of:
- `sft`
- `preference`
- `contrastive`
- `long-horizon`

---

## 2. Global Options

```bash
--output <path>              # required
--format jsonl|parquet       # default: jsonl
--min-uxp <float>            # default: 0.7
--max-ufi <float>            # default: 0.3
--min-autonomy <int>         # default: 0
--agent <agent_id|all>
--task <task_id|all>
--since <iso8601>
--until <iso8601>
--dry-run
--explain
```

---

## 3. Dataset-Specific Flags

### 3.1 SFT
```bash
--require-success
--min-turns 3
```

### 3.2 Preference
```bash
--min-delta 0.15
--require-user-correction
```

### 3.3 Contrastive
```bash
--require-abandoned-branch
--max-path-length 20
```

### 3.4 Long Horizon
```bash
--min-session-length 30m
--min-state-transitions 5
```

---

## 4. Execution Phases

### Phase 1 — Discovery
- Enumerate sessions
- Filter by eligibility
- Validate license/consent

### Phase 2 — Extraction
- Load Focus State snapshots
- Resolve CLT paths
- Resolve Reference Store artifacts

### Phase 3 — Normalization
- Canonicalize text
- Normalize formats
- Strip provider fingerprints

### Phase 4 — Validation
- Schema validation
- Provenance completeness
- Outcome signals present

### Phase 5 — Export
- Write dataset
- Emit manifest
- Emit statistics

---

## 5. Dry Run Mode

```bash
focusa export sft --dry-run --explain
```

Outputs:
- number of eligible records
- exclusion reasons
- sample schema preview
- estimated dataset size

No files written.

---

## 6. Manifest File

Each export produces a manifest:

```json
{
  "dataset_type": "focusa_sft",
  "record_count": 1243,
  "filters": { },
  "uxp_threshold": 0.7,
  "ufi_threshold": 0.3,
  "generated_at": "iso8601",
  "focusa_version": "semver"
}
```

---

## 7. Safety & Privacy Guarantees

- No network calls
- No mutation of Focusa state
- Explicit opt-in required
- Redaction hooks available
- Per-record exclusion logging

---

## 8. Integration Targets

Verified compatibility with:
- Unsloth
- HuggingFace `datasets`
- Axolotl
- TRL (DPO/IPO)

---

## 9. Canonical Rule

> **Exporting data is an act of training — not logging.  
> Treat it with the same rigor as model evaluation.**
