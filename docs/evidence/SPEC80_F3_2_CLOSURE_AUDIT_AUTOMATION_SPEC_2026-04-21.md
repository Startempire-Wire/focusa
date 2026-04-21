# SPEC80 F3.2 — Closure Audit Automation Spec

Date: 2026-04-21
Bead: `focusa-yro7.6.3.2`
Label: `documented-authority`

Purpose: define automated audit process to verify closure-proof completeness across SPEC80 beads.

## Audit checks

1. Required closure fields present (`code`, `spec section`, `Evidence citations`).
2. Evidence files referenced exist on disk.
3. Label taxonomy compliance (from F3.1).
4. No open child beads for closed parent beads.

## Audit run modes

- pre-close check for single bead
- nightly full-tree audit for `focusa-yro7.*`

## Audit output schema

```json
{
  "run_id": "string",
  "scope": "single|tree",
  "total_checked": 0,
  "passed": 0,
  "failed": 0,
  "failures": [{"bead_id":"string","reason":"..."}]
}
```

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§19.3, §20.4)
- docs/evidence/SPEC80_F3_1_BEAD_METADATA_LINT_POLICY_SPEC_2026-04-21.md
