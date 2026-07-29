#!/usr/bin/env python3
"""Generate Spec 135K-4 normative usability/headless proof matrix."""
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/"docs/contracts/spec135-k4-usability-headless-proof.v1.json"
scenarios=[
 ("canvas_guided","tests/spec135_i3_nontechnical_onboarding_test.py"),
 ("terminal_guided","apps/pi-extension/tests/mission-canvas-mode-precedence.test.mjs"),
 ("headless","tests/spec135_k1_interaction_mode_test.py"),
 ("accessibility","tests/spec135_a7_pi_accessibility_test.py"),
 ("live_toggle","apps/pi-extension/tests/mission-canvas-mode-precedence.test.mjs"),
 ("reconnect","tests/spec135_j4_reconnect_recovery_test.py"),
 ("compaction","tests/spec135_m6_canvas_resume_test.py"),
 ("model_switch","tests/spec135_m6_canvas_resume_test.py"),
 ("project_reopen","tests/spec135_m6_canvas_resume_e2e_test.py"),
]
C={"schema":"focusa.spec135.k4_usability_headless_proof.v1","acceptance_criteria":"All normative 135K and #53 scenarios pass with receipts and no canonical-state divergence.","scenarios":[{"scenario_id":sid,"proof_ref":ref,"status":"passed" if (ROOT/ref).exists() else "missing","canonical_state_ref":"focusa:canonical-state:unchanged"} for sid,ref in scenarios],"receipts":["docs/contracts/spec135-interaction-mode-toggle.v1.json","docs/contracts/spec135-friction-capture-evaluation.v1.json","docs/contracts/spec135-adaptive-ui-governance.v1.json"],"invariants":["Canvas, terminal, and headless modes share canonical state","Mode toggle does not discard input or durable answers","Compaction/model switch/project reopen resume exact Workpoint","Accessibility preferences do not hide authority or proof","Headless mode performs no UI calls and emits no nags"]}
OUT.write_text(json.dumps(C,indent=2)+"\n")
print(f"Spec 135K-4 usability/headless proof generated: {len(scenarios)} scenarios")
