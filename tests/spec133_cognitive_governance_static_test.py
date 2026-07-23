#!/usr/bin/env python3
from pathlib import Path
P=Path(__file__).resolve().parents[1]/'crates/focusa-core/src/silent_sessions/cognitive_governance.rs';S=P.read_text()
for x in ['current_ask','project_identity_ref','continuity_id','trajectory_ref','workpoint_ref','waypoints','gap','object_refs','do_not_drift','steering_revision','project mismatch or generic trajectory blocks mutation','context_packet_ref','context_advisory_only','action_authority_ref','authority_fresh','ontology_refs','agent_bootstrap_verified','RuntimeCheckpoint','MeaningfulCheckpointTrigger','BeforeRiskyMutation','BeforeTransfer','BeforeModelSwitch','BeforeCompletion','CompletionEvidenceBundle','git_status_ref','diff_ref','tests_ref','lint_ref','acceptance_verified','adversarial_verified','RiskyMutation','BlockedClaim','Handoff','Bootstrap','Closure','Final','Prepare','Validate','Authorize','Provider','Reconcile','Audit','cannot self-close','ForegroundTakeover','ModelSwitch','RuntimeLoss','prediction_evaluated','evidence-backed lesson required','learning cannot override governance']:
 assert x in S,x
assert len(S.splitlines())<=500
print('Spec133 cognitive governance static contract: PASS')
