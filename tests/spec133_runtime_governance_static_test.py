#!/usr/bin/env python3
from pathlib import Path
R=Path(__file__).resolve().parents[1]/'crates/focusa-core/src/silent_sessions'
C=(R/'runtime_control.rs').read_text(); P=(R/'recovery_policy.rs').read_text(); A=(R/'resource_admission.rs').read_text(); F=(R/'failure_envelope.rs').read_text()
for x in ['PauseMode','Soft','Hard','FollowUp','Steering','SpecialKey','unsupported; no control was applied','ExactRuntimeTarget']: assert x in C,x
for x in ['Provider','Transport','Harness','Tool','Model','Runner','WorkItem','base_backoff_ms','manifest_hash_matches','heartbeat_authenticated','prior_run_classified_orphaned','workpoint_checkpoint_ref','new run generation']: assert x in P,x
for x in ['global','user','project','provider','cpu_millis','memory_bytes','pids','file_descriptors','io_bytes','disk_bytes','output_bytes','tokens','cost_usd','CheckpointAndPause','capture_cursor_persisted']: assert x in A,x
classes='''scope_mismatch project_identity_unverified continuity_missing workpoint_unavailable writer_conflict workspace_conflict authorization_required permission_denied approval_expired context_authority_blocked config_invalid config_locked model_not_found model_entitlement_unverified model_mismatch fallback_disallowed harness_unsupported backend_unsupported capability_missing runner_unavailable runner_lost process_spawn_failed process_control_failed process_exited child_leak_detected transport_degraded transport_lost waiting_input provider_failure retry_exhausted resource_admission_denied resource_limit_exceeded output_storage_pressure stream_corruption checkpoint_failed evidence_missing verification_failed completion_evidence_missing receipt_commit_failed orphan_adoption_rejected protocol_incompatible retention_blocked_by_hold'''.split()
def pascal(s): return ''.join(p.title() for p in s.split('_'))
for x in classes: assert pascal(x) in F,x
for x in ['why','current_lifecycle','canonical_runtime_posture','safe_retry_posture','side_effects_performed','exact_recovery_tools','operator_action_required']: assert x in F,x
for f in ['runtime_control.rs','recovery_policy.rs','resource_admission.rs','failure_envelope.rs']: assert len((R/f).read_text().splitlines())<=500
print('Spec133 runtime governance batch static contract: PASS')
