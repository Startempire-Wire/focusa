#!/usr/bin/env python3
"""Generate Spec 135K-3 governed adaptive UI proposal contract."""
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/"docs/contracts/spec135-adaptive-ui-governance.v1.json"
C={"schema":"focusa.spec135.adaptive_ui_governance.v1","acceptance_criteria":"No UI adaptation self-promotes; approved changes improve measured outcomes and rollback exactly.","candidate":{"required_fields":["candidate_id","scope","evidence_refs","baseline_metrics","predicted_outcome","preview_ref","version","rollback_snapshot_ref"],"advisory_only":True},"promotion":{"operator_approval_required":True,"self_promotion":False,"rule":"eval_score > baseline_score AND eval_score >= score_threshold","minimum_evaluation_window":30,"cohort_isolation_required":True,"authority_panels_hideable":False},"versioning":{"immutable_artifact":True,"supersedes_ref_required":True,"history_append_only":True},"rollback":{"exact_snapshot_required":True,"idempotent":True,"restores_layout_and_preferences":True,"mutates_canonical_project_state":False,"receipt_required":True},"evaluation":{"metrics":["task_success","time_to_recovery","repeat_prompt_count","accessibility_failure","abandonment"],"harm_gate":"any authority/proof/safety regression blocks promotion"},"laws":["No UI adaptation self-promotes","Model preference is never approval","Adaptation cannot hide authority, evidence, safety, or recovery surfaces","Rejected candidates remain inspectable but inactive"]}
OUT.write_text(json.dumps(C,indent=2)+"\n")
print("Spec 135K-3 adaptive UI governance generated")
