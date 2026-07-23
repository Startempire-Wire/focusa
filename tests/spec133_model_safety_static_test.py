#!/usr/bin/env python3
from pathlib import Path
S=(Path(__file__).resolve().parents[1]/'crates/focusa-core/src/silent_sessions/model_safety.rs').read_text()
for marker in ['ExactModelBinding','auth_profile_ref','entitlement_verified','catalog_verified','context_window_verified','rate_limit_verified','budget_verified','authorize_project_mutation','mismatch_abort_required','abort exact run before project mutation','ModelSwitchCheckpoint','workpoint_checkpoint_ref','bootstrap_packet_ref','fallback model is not explicitly allowlisted','fallback trigger class is not explicitly allowlisted','model fallback is disabled']:
    assert marker in S, marker
assert len(S.splitlines()) <= 500
print('Spec133 exact model safety static contract: PASS')
