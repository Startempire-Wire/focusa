#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="${ROOT_DIR:-/home/wirebot/focusa}"
cd "$ROOT_DIR"
SPEC="docs/102-focusa-agent-ux-composition-and-real-life-test-spec.md"
EVIDENCE="docs/evidence/SPEC102_REAL_LIFE_SURFACE_BATTERY_2026-06-06.md"
AUDIT="docs/evidence/SPEC102_NO_DEFERRAL_CLOSURE_GATE_AUDIT_2026-06-06.md"
FULL_AUDIT="docs/evidence/SPEC102_FULL_IMPLEMENTATION_AUDIT_CURRENT_2026-06-06.md"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }
[[ -s "$SPEC" ]] || fail "Spec102 doc missing"
[[ -s "$EVIDENCE" ]] || fail "Spec102 evidence battery missing"
[[ -s "$AUDIT" ]] || fail "Spec102 no-deferral audit missing"
[[ -s "$FULL_AUDIT" ]] || fail "Spec102 current full audit missing"
rg -n '## 13\. Spec98/99 singleton-remnant mapping' "$SPEC" >/dev/null || fail "Section 13 missing"
rg -n '### 13\.6 Clean-repair UX acceptance bar' "$SPEC" >/dev/null || fail "clean-repair acceptance bar missing"
rg -n '## 14\. Repair backlog and invisible-UX acceptance matrix' "$SPEC" >/dev/null || fail "Section 14 repair matrix missing"
rg -n '## 15\. Other-agent UX backlog' "$SPEC" >/dev/null || fail "Section 15 other-agent backlog missing"
rg -n '## 16\. Full implementation assurance and no-deferral closure gate' "$SPEC" >/dev/null || fail "Section 16 assurance gate missing"
for t in \
  tests/spec102_workpoint_requested_id_fallback_runtime_test.sh \
  tests/spec102_trajectory_workpoint_reconciliation_runtime_test.sh \
  tests/spec102_focusstate_workpoint_bridge_runtime_test.sh \
  tests/spec102_golden_happy_path_runtime_test.sh \
  tests/spec102_no_deferral_closure_gate.sh \
  tests/spec102_prep_packet_enforcement_test.sh \
  tests/spec102_proof_matrix_enforcement_test.sh \
  tests/spec102_supersession_policy_test.sh; do
  [[ -x "$t" ]] || fail "required Spec102 executable test missing: $t"
done
rg -n 'residual_ui_risk: none|Clean-repair criteria|happy path passes only if a fresh tester' "$SPEC" >/dev/null || fail "clean-repair no-residual language missing"
pass "Spec102 docs/evidence/acceptance gate preserved"
