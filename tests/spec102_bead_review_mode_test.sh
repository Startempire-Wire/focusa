#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="${ROOT_DIR:-/home/wirebot/focusa}"
cd "$ROOT_DIR"
SCRIPT="scripts/spec102-bead-review"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

for term in bead_review changed_files linked_evidence tests_run clean_repair_checklist residual_ui_risk residual_authority_risk next_follow_up close_recommendation keep_open split_followup; do
  rg -F "$term" "$SCRIPT" >/dev/null || fail "bead review generator missing $term"
done
pass "bead review generator declares Section 15.10 review fields"

# Missing-proof fixture must not recommend close; stable even after all real Section 15 beads close.
"$SCRIPT" __missing_proof_fixture /tmp/spec102-review-open.json >/tmp/spec102-review-open.out
jq -e '.bead_review.bead_id == "__missing_proof_fixture" and .bead_review.close_recommendation == "keep_open" and .bead_review.clean_repair_checklist.implementation_proof == false and .bead_review.residual_ui_risk != "none"' /tmp/spec102-review-open.json >/dev/null || fail "open bead review did not keep open without proof"
pass "missing-proof review keeps bead open without proof"

# A previously closed, fully proven bead should recommend close.
"$SCRIPT" focusa-pm2b.35 /tmp/spec102-review-closed.json >/tmp/spec102-review-closed.out
jq -e '
  .bead_review.bead_id == "focusa-pm2b.35"
  and .bead_review.close_recommendation == "close"
  and .bead_review.clean_repair_checklist.implementation_proof == true
  and .bead_review.clean_repair_checklist.refs_present == true
  and .bead_review.clean_repair_checklist.residual_risks_none == true
  and .bead_review.residual_ui_risk == "none"
  and .bead_review.residual_authority_risk == "none"
  and (.bead_review.tests_run | length) >= 1
' /tmp/spec102-review-closed.json >/dev/null || fail "proven bead review missing close recommendation/proof fields"
pass "proven bead review recommends close compactly"

echo "SPEC102 bead review mode test: PASS"
