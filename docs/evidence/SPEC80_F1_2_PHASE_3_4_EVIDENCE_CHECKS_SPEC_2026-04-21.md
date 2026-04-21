# SPEC80 F1.2 — Phase 3-4 Evidence Checks Spec

Date: 2026-04-21
Bead: `focusa-yro7.6.1.2`
Label: `documented-authority`

Purpose: define evidence-gated readiness criteria for Phase 3 (metacognition compounding) and Phase 4 (evaluation/hardening) before program closure.

## Authority
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§10 Gate D/E, §11 phases 3-4, §16 Appendix C, §18 Appendix E, §20.3-§20.4)
- docs/evidence/SPEC80_COMPOUNDING_GATE_REPORT_GENERATOR_2026-04-21.md
- docs/evidence/SPEC80_SILENT_MUTATION_SENTINEL_CHECKS_2026-04-21.md

## Phase 3 — Metacognition compounding evidence checks

Required:
1. Tool-chain coverage proof for capture/retrieve/reflect/plan_adjust/evaluate_outcome flow contracts.
2. Outcome contract instrumentation + threshold evaluator artifacts are present and schema-stable.
3. Practice+observation form contract + quality validation rules are present (Appendix E).
4. Gate D precondition data is available:
   - baseline/evaluation windows,
   - sample floors,
   - form-volume floors.

## Phase 4 — Evaluation/hardening evidence checks

Required:
1. Gate D report generator produces deterministic decision envelope (`pass|fail|insufficient_data`).
2. Critical regression guard is enforced (`failed_turn_ratio >5%` forces fail).
3. Governance integrity checks present:
   - zero silent mutation sentinel,
   - auditable mutation path constraints.
4. Full-utilization closure proof packet includes criteria mapping for §20.3 items.

## Readiness decision rules

Phase 3 ready when:
- all Phase 3 required checks pass with evidence refs.

Phase 4 ready when:
- all Phase 4 required checks pass,
- Gate D decision available,
- Gate E governance integrity checks satisfied.

Program closure eligibility:
- Phase 3 and Phase 4 both `ready=true`, with no blocking items.

## Readiness report schema

```json
{
  "phase": "3|4",
  "ready": true,
  "required_checks": [{"id":"string","pass":true,"evidence_refs":[]}],
  "gate_bindings": ["Gate D", "Gate E"],
  "blocking_items": []
}
```

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md
- docs/evidence/SPEC80_COMPOUNDING_GATE_REPORT_GENERATOR_2026-04-21.md
- docs/evidence/SPEC80_OUTCOME_THRESHOLD_EVALUATOR_DESIGN_2026-04-21.md
- docs/evidence/SPEC80_EVALUATION_CADENCE_AUTOMATION_2026-04-21.md
- docs/evidence/SPEC80_SILENT_MUTATION_SENTINEL_CHECKS_2026-04-21.md
