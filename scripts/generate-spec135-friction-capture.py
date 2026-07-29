#!/usr/bin/env python3
"""Generate Spec 135K-2 consent-aware UXP/UFI friction contract."""
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/"docs/contracts/spec135-friction-capture-evaluation.v1.json"
C={"schema":"focusa.spec135.friction_capture_evaluation.v1","acceptance_criteria":"Friction data is scoped, privacy-safe, evidence-linked, and cannot become canonical authority.","record":{"required_fields":["friction_id","project_root","continuity_id","attachment_id","cohort","interaction_kind","task_success","recovery_outcome","accessibility_posture","evidence_refs","consent","captured_at"],"cohorts":["canvas-guided","terminal-guided","headless"],"bounded_summary_max_chars":500,"raw_input_stored":False,"secret_fields_stored":False},"consent":{"default":False,"explicit_required":True,"revocable":True,"deletion_receipt_required":True},"authority":{"plane":"telemetry_history","canonical_state_owner":False,"cannot_change_workpoint":True,"cannot_change_permission":True,"cannot_promote_ontology":True},"evaluation":{"minimum_window":30,"learning_rate_max":0.1,"separate_cohorts":True,"metrics":["task_success","time_to_recovery","repeat_prompt_count","accessibility_failure","abandonment"]},"implementation_refs":["crates/focusa-core/src/types.rs::UxpProfile","crates/focusa-core/src/types.rs::UfiState"],"laws":["Friction observations remain advisory telemetry","Evidence links use stable refs, never raw transcript blobs","Canvas and terminal cohorts are evaluated separately","No adaptive change occurs from a single friction event"]}
OUT.write_text(json.dumps(C,indent=2)+"\n")
print("Spec 135K-2 friction capture contract generated")
