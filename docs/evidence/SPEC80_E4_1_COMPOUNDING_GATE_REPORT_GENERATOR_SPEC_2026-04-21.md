# SPEC80 E4.1 — Compounding Gate Report Generator Spec

Date: 2026-04-21
Bead: `focusa-yro7.5.4.1`
Label: `documented-authority`

Purpose: define report generator contract for Gate D decisions.

## Inputs

1. Metric scorecards (E1 pipeline)
2. Baseline medians (E2)
3. Threshold evaluator output (E1.2)
4. Form-quality stats (E3)

## Report requirements

- Contract-by-contract deltas and pass/fail state
- Sample-size gate status
- Regression override status (`failed_turn_ratio > +5%`)
- Gate D final decision (`pass|fail`)
- Evidence references used for decision

## Output schema

```json
{
  "report_id": "string",
  "window": {"start":"ISO-8601","end":"ISO-8601"},
  "contracts": {},
  "sample_gates": {"turns_ok": true, "novel_context_ok": true, "failure_loops_ok": true},
  "critical_regression": false,
  "passed_contracts": 0,
  "gate_d_decision": "pass|fail",
  "evidence_refs": [],
  "generated_at": "ISO-8601"
}
```

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§8, §10 Gate D, §16, §18.5)
- docs/evidence/SPEC80_E1_2_THRESHOLD_EVALUATOR_DESIGN_2026-04-21.md
