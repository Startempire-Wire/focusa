#!/usr/bin/env python3
from pathlib import Path
S=(Path(__file__).resolve().parents[1]/'crates/focusa-core/src/silent_sessions/operator_experience.rs').read_text()
for x in ['SilentDashboardCard','daemon_projection','scoped_authorization','Summary','Text','Tools','Stdout','Stderr','Events','Raw','Evidence','after_cursor','bounded_limit','SendText','SendFollowUp','SendSteering','SendSpecialKey','SoftPause','HardPause','Resume','Interrupt','ControlledStop','ForceCancel','Restart','Adopt','Handoff','OpenWorktree','OpenEvidence','OpenReceipt','WaitingInput','JudgmentBlocker','ModelMismatch','AuthEntitlementFailure','RepeatedProviderFailure','ResourcePressure','CheckpointFailure','ProcessFailure','OrphanedRun','CompletionEvidenceMissing','VerifiedCompletion','dedupe_key','exact_action','all 13 creation steps','provider_visible','model_visible','daemon_api_ref','bounded_rehydrate_ref','cannot mint authority','cannot depend on foreground Pi']:
 assert x in S,x
assert len(S.splitlines())<=500
print('Spec133 operator experience static contract: PASS')
