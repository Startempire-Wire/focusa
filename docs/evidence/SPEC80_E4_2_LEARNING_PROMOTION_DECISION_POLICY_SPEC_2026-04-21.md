# SPEC80 E4.2 — Learning Promotion Decision Policy Spec

Date: 2026-04-21
Bead: `focusa-yro7.5.4.2`
Label: `documented-authority`

Purpose: define deterministic promote/inhibit policy for `compound_candidate` outputs.

## Promotion eligibility

`promote=true` only when all conditions hold:
1. Gate D pass achieved for evaluation window.
2. No critical regression override triggered.
3. Form quality pass rate >= 95% for evaluated forms.
4. Candidate includes `learning_statement`, `applicability`, `expiry_policy`.
5. Evidence refs include at least one code/doc/test trace.

## Inhibit conditions

- insufficient sample size gates
- regression override active
- missing required candidate fields
- low evidence quality / unverifiable claim

## Output policy schema

```json
{
  "candidate_id": "string",
  "decision": "promote|inhibit",
  "reasons": ["..."],
  "expiry_policy": "string",
  "review_after": "ISO-8601"
}
```

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§8, §16, §18.3, §18.5)
- docs/evidence/SPEC80_E4_1_COMPOUNDING_GATE_REPORT_GENERATOR_SPEC_2026-04-21.md
