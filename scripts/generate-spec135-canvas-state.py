#!/usr/bin/env python3
"""Generate Spec 135G-2 durable Mission Canvas presentation-state schema."""
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
SD=ROOT/"docs/contracts/spec135/generated-contract-v1/json-schema"
SP=SD/"focusa.mission_canvas_layout_state.v1.json"
CP=ROOT/"docs/contracts/spec135-canvas-layout-restoration.v1.json"
fields={
 "schema":{"const":"focusa.mission_canvas_layout_state.v1"}, "canvas_id":{"type":"string","minLength":1}, "state_revision":{"type":"integer","minimum":1},
 "project_root":{"type":"string","minLength":1}, "continuity_id":{"type":"string","minLength":1}, "client_instance_id":{"type":"string","minLength":1},
 "user_id":{"type":"string","minLength":1}, "device_id":{"type":"string","minLength":1}, "open_work_surface_ids":{"type":"array","items":{"type":"string"}},
 "focused_work_surface_id":{"type":["string","null"]}, "secondary_focused_surface_id":{"type":["string","null"]}, "split_layout_ref":{"type":["string","null"]},
 "group_order":{"type":"array","items":{"type":"string"}}, "aggregate_project_roots":{"type":"array","items":{"type":"string"}},
 "aggregate_continuity_ids":{"type":"array","items":{"type":"string"}}, "aggregate_surface_kinds":{"type":"array","items":{"type":"string"}},
 "aggregate_surface_states":{"type":"array","items":{"type":"string"}}, "selected_context_refs":{"type":"array","items":{"type":"string"}},
 "unread_event_cursor":{"type":["integer","null"],"minimum":0}, "session_projection_revision":{"type":"integer","minimum":0},
 "idempotency_key":{"type":"string","minLength":1}, "created_at":{"type":"string","minLength":1}, "updated_at":{"type":"string","minLength":1},
}
required=[k for k in fields if k not in {"focused_work_surface_id","secondary_focused_surface_id","split_layout_ref","unread_event_cursor"}]
schema={"$schema":"https://json-schema.org/draft/2020-12/schema","$id":"https://docs.startempire.ai/focusa/spec135/focusa.mission_canvas_layout_state.v1.json","title":"Focusa Mission Canvas Layout State v1","description":"Per-user/device presentation state; never canonical project authority.","type":"object","required":required,"properties":fields,"additionalProperties":False}
contract={"schema":"focusa.spec135.canvas_layout_restoration.v1","acceptance_criteria":"Layout restores after restart without mutating canonical project state.","rust_model_ref":"crates/focusa-core/src/types.rs::MissionCanvasStateRecord","restoration_groups":{"open_focus":["open_work_surface_ids","focused_work_surface_id","secondary_focused_surface_id"],"split_group":["split_layout_ref","group_order"],"filters":["aggregate_project_roots","aggregate_continuity_ids","aggregate_surface_kinds","aggregate_surface_states","selected_context_refs"],"revision":["state_revision","session_projection_revision","unread_event_cursor"]},"ownership":{"project_defaults":"project-owned","canvas_preferences":"user/device-owned","canonical_project_state_mutation":False},"laws":["Visual focus never becomes singleton canonical authority","Presentation restoration cannot hide authority, proof, or safety panels","Idempotency key prevents duplicate restore writes","Scope remains exact project_root plus continuity_id"]}
SD.mkdir(parents=True,exist_ok=True); SP.write_text(json.dumps(schema,indent=2)+"\n"); CP.write_text(json.dumps(contract,indent=2)+"\n")
print("Spec 135G-2 canvas layout restoration generated")
