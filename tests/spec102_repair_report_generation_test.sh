#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="${ROOT_DIR:-/home/wirebot/focusa}"
cd "$ROOT_DIR"
SCRIPT="scripts/spec102-repair-report"
REPORT="docs/evidence/SPEC102_REPAIR_REPORT_CURRENT.md"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

for term in repair_report_index repair_id target_files_or_routes original_failure_proof happy_path_proof clean_repair_checklist_result residual_ui_risk residual_authority_risk evidence_refs; do
  rg -F "$term" "$SCRIPT" >/dev/null || fail "report generator missing template term $term"
done
pass "repair report generator carries Section 14.8 template terms"

"$SCRIPT" focusa-pm2b "$REPORT" >/tmp/spec102-repair-report.out
rg -F 'missing_required_fields=0' /tmp/spec102-repair-report.out >/dev/null || fail "repair report generator found missing fields"
for term in repair_report_index repair_id original_failure_proof happy_path_proof clean_repair_checklist_result 'residual_ui_risk: none' 'residual_authority_risk: none'; do
  rg -F "$term" "$REPORT" >/dev/null || fail "generated report missing $term"
done
pass "generated repair report has required proof fields and residual risk none"

echo "SPEC102 repair report generation test: PASS"
