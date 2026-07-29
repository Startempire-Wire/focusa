#!/usr/bin/env python3
"""Generate Spec 135G-4 steering/contention/writer governance contract."""
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/"docs/contracts/spec135-writer-governance.v1.json"
C={
 "schema":"focusa.spec135.writer_governance.v1",
 "acceptance_criteria":"Concurrent work routes correctly and conflicting writers fail closed with visible recovery.",
 "recipient":{"required_identity":["project_root","continuity_id","attachment_id","work_surface_id"],"broadcast_preview_required":True,"implicit_broadcast":False},
 "queues":{"steering":{"durable":True,"ordered":True},"follow_up":{"durable":True,"ordered":True},"observations":{"advisory_only":True},"proposals":{"promotion_required":True}},
 "writer_lease":{"scope":["project_root","continuity_id","target_ref"],"single_active_writer":True,"optimistic_version_required":True,"conflict_outcome":"fail_closed","recovery_visible":True,"recovery_actions":["inspect_active_writer","defer_action","request_takeover","select_non_conflicting_worktree"]},
 "worktree":{"exact_root_required":True,"canonical_parent_preserved":True,"cross_worktree_mutation":"blocked_without_explicit_scope"},
 "approval":{"required_for":["broadcast","writer_takeover","conflict_resolution","side_effect_mutation"],"preview_receipt_required":True},
 "laws":["Visual focus does not select a mutation recipient implicitly","Observations cannot mint canonical state","Proposals require governed promotion","Conflicting writers never silently last-write-wins","Recovery names the active writer, conflict scope, and safe next actions"],
 "implementation_refs":["crates/focusa-api/src/routes/work_loop.rs","crates/focusa-api/src/scope.rs","apps/pi-extension/src/work-rail-interactions.ts"],
}
OUT.write_text(json.dumps(C,indent=2)+"\n")
print("Spec 135G-4 writer governance generated")
