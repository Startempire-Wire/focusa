#!/usr/bin/env python3
"""Generate Spec 135G-3 Work Surface lifecycle/close authority contract."""
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/"docs/contracts/spec135-surface-lifecycle-close-semantics.v1.json"
C={
 "schema":"focusa.spec135.surface_lifecycle_close_semantics.v1",
 "acceptance_criteria":"Lifecycle tests prove close never terminates work and explicit termination uses separate governed authority.",
 "surface_actions":["create","arrange","pin","group","suspend","resume","rehydrate","close_view"],
 "close_view":{"effect":"remove presentation attachment from open canvas","session_terminated":False,"provider_work_closed":False,"workpoint_completed":False,"reopenable":True},
 "suspend":{"effect":"pause presentation/live subscription while preserving durable state","session_terminated":False,"rehydratable":True},
 "terminate_session":{"surface_action":False,"authority":"separate governed session operation","confirmation_required":True,"preview_required":True,"receipt_required":True,"cannot_be_inferred_from":"close_view"},
 "transition_laws":["close_view never terminates session or provider work","terminate_session is not a SurfaceAction","suspend preserves exact scope and attachment identity","resume and rehydrate preserve state revision lineage","arrange/pin/group mutate presentation state only"],
 "implementation_ref":"crates/focusa-api/src/routes/mission_canvas_surfaces.rs::SurfaceAction",
}
OUT.write_text(json.dumps(C,indent=2)+"\n")
print("Spec 135G-3 surface lifecycle contract generated")
