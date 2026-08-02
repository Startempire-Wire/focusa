#!/usr/bin/env python3
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]
core=(ROOT/'crates/focusa-core/src/temporal.rs').read_text()
tests=(ROOT/'crates/focusa-core/src/temporal_tests.rs').read_text()
authority=(ROOT/'crates/focusa-core/src/temporal_authority.rs').read_text()
claims=(ROOT/'crates/focusa-core/src/temporal_claims.rs').read_text()
forecast=(ROOT/'crates/focusa-core/src/temporal_forecast.rs').read_text()
forecast_evaluation=(ROOT/'crates/focusa-core/src/temporal_forecast_evaluation.rs').read_text()
ledger=(ROOT/'crates/focusa-core/src/temporal_ledger.rs').read_text()
deadline=(ROOT/'crates/focusa-core/src/temporal_deadline.rs').read_text()
operations=(ROOT/'crates/focusa-core/src/temporal_operations.rs').read_text()
progress=(ROOT/'crates/focusa-core/src/temporal_progress.rs').read_text()
platform=(ROOT/'crates/focusa-core/src/temporal_platform.rs').read_text()
lib=(ROOT/'crates/focusa-core/src/lib.rs').read_text()
api=(ROOT/'crates/focusa-api/src/routes/temporal.rs').read_text()
api_advanced=(ROOT/'crates/focusa-api/src/routes/temporal_advanced.rs').read_text()
api_closure=(ROOT/'crates/focusa-api/src/routes/temporal_closure.rs').read_text()
api_all=api+api_advanced+api_closure
cli=(ROOT/'crates/focusa-cli/src/commands/temporal.rs').read_text()
pi=(ROOT/'apps/pi-extension/src/tools.ts').read_text()
turns=(ROOT/'apps/pi-extension/src/turns.ts').read_text()
canvas=(ROOT/'apps/menubar/src/lib/components/TemporalAuthorityPeek.svelte').read_text()
mission_canvas=(ROOT/'apps/menubar/src/lib/components/MissionCanvasView.svelte').read_text()
tui=(ROOT/'crates/focusa-tui/src/mission_control.rs').read_text()
registry=(ROOT/'docs/contracts/spec135/generated-contract-v1/operation-registry.json').read_text()
bindings=(ROOT/'docs/contracts/spec135/generated-contract-v1/ui-action-bindings.fixture.json').read_text()
for symbol in (
 'TemporalClockDomain','TemporalClaimKind','TemporalClaimStatus','TemporalConfidence',
 'TemporalUncertainty','TemporalScope','TemporalClockSample','TemporalClaim','TemporalEvent',
 'DeadlineStatus','TemporalProjection','TemporalLedger','TemporalLedgerError',
): assert symbol in core, symbol
for plane in ('ClockFact','ExternalCommitment','InternalReadinessTarget','Estimate','Forecast','UrgencySignal','PresentationHint','NoDeadline','ObservedDuration','MissedTarget'):
 assert plane in core, plane
for invariant in ('CommitmentRequiresConfirmation','CommitmentRequiresTarget','EstimateCannotBecomeCommitment','InvalidUncertaintyRange','InvalidCoverageProbability','RevisionMustAdvance','SupersessionRequired'):
 assert invariant in core, invariant
for durability in ('events.jsonl','OpenOptions::new()','sync_data()','sync_all()','verify_event_chain','predecessor_digest','idempotency_key'):
 assert durability in core+ledger, durability
for symbol in ('HumanCalendarContext','TemporalPriorityFrame','TemporalExecutionGuard','authorize_temporal_action','DeadlineConflictState','PressureNarrowingChecklist'):
 assert symbol in operations, symbol
for symbol in ('DeadlineContract','DeadlineContractKind','DeadlineBoundaryPolicy','DeadlineComparison','CivilTimeIntent','resolve_civil_time','TemporalBreach','OpportunityRisk','DeadlineDispatchPolicy'):
 assert symbol in deadline, symbol
for symbol in ('PlatformClockCapture','capture_platform_clocks','capture_temporal_clock_sample','unsupported'):
 assert symbol in platform, symbol
for symbol in ('ForecastEvaluation','ForecastValidityFingerprint','evaluate_forecast','forecast_remains_valid','policy_quantiles'):
 assert symbol in forecast_evaluation, symbol
assert 'DeadlineStatus::None' in core
assert 'projection.urgency.is_none()' in tests
assert 'ledger_fsyncs_causal_batch_and_replays_idempotently' in tests
for symbol in ('TemporalDomainClockPolicy','ClockTrustProfile','TemporalAuthority','ClockSynchronizationStatus','evaluate_clock_sample'):
 assert symbol in authority, symbol
for boundary in ('suspend_consumes_interval','reboot_consumes_interval','required_independent_source_count','sources_authenticated','HoldoverExpired','Disagreement'):
 assert boundary in authority, boundary
for symbol in ('TemporalClaimAuthority','TemporalClaimEnvelope','TemporalPreflight','authorize_claim','revise_claim','temporal_preflight','derive_urgency'):
 assert symbol in claims, symbol
for invariant in ('ForecastCannotBecomeCommitment','OperatorConfirmationRequired','EvidenceRequired','authority_escalated: false','no deadline is set; no urgency was inferred'):
 assert invariant in claims, invariant
for symbol in ('ObservedDuration','ForecastRange','ReleaseTimingPlan','ForecastCalibration','MissedTargetReceipt','DoraTemporalMetrics','forecast_phase','build_release_timing_plan','calibrate','dora_metrics'):
 assert symbol in forecast, symbol
for phase in ('Freeze','Build','Sign','Publish','Deploy','ArtifactPropagation','UpdateRollout','CanaryObservation','RollbackDecision','RollbackRecovery','Approval'):
 assert phase in forecast, phase
assert 'NoObservedHistory' in forecast and 'critical_path_ms' in forecast and 'slack_ms' in forecast
assert 'pub mod temporal;' in lib and 'pub mod temporal_authority;' in lib and 'pub mod temporal_claims;' in lib and 'pub mod temporal_forecast;' in lib
assert len(core.splitlines()) < 500
assert len(claims.splitlines()) < 500
assert len(forecast.splitlines()) < 500
for action in ('status','commit','revise','observe','forecast','preflight'):
 endpoint=f'/v1/temporal/{action}'
 assert endpoint in api, endpoint
 assert f'focusa.temporal.{action}' in registry
 assert f'focusa.temporal.{action}' in bindings
for endpoint in ('/v1/temporal/priority/commit','/v1/temporal/civil/resolve','/v1/temporal/clock/capture','/v1/temporal/high-consequence/preflight','/v1/temporal/migrate-signatures'):
 assert endpoint in api, endpoint
assert 'TemporalCmd' in cli and 'name: "focusa_temporal_authority"' in pi
for action in ('commit-priority','resolve-civil-time','capture-clock','high-consequence-preflight','migrate-signatures','settle-closure'):
 assert action in pi, action
for symbol in ('LostTimeClassification','IncidentVerificationStatus','LostTimeIncident','validate_lost_time_incident'):
 assert symbol in progress, symbol
for field in ('subject_ref','interval_start','interval_end','wall_clock_lost_ms','classification','verification_status','settlement_ref'):
 assert field in progress, field
for symbol in ('TemporalClosureSettlementRequest','LostTimeIncidentRecorded','ClosurePostureRecorded','ReceiptLinked','idempotent_replay'):
 assert symbol in api_closure, symbol
assert '/v1/temporal/settle-closure' in api
assert 'SettleClosure' in cli and '/v1/temporal/settle-closure' in cli
assert 'closure_packet' in pi
for marker in ('TEMPORAL_PRIORITY','durable_or_consequential_action','temporal_priority_frame','temporal_execution_guard'):
 assert marker in turns, marker
for marker in ('aria-live="polite"','<dt>Deadline</dt>','<dt>Forecast</dt>','<dt>Urgency</dt>','<dt>Calendar</dt>'):
 assert marker in canvas, marker
assert 'TemporalAuthorityPeek' in mission_canvas
assert 'deadline=' in tui and 'urgency=' in tui
assert 'confirmation_required' in api_all and 'forecast_history_insufficient' in api_all
assert len(api.splitlines()) < 500
assert len(api_advanced.splitlines()) < 500
assert len(ledger.splitlines()) < 500
assert len(forecast_evaluation.splitlines()) < 500
assert len(deadline.splitlines()) < 500
assert len(operations.splitlines()) < 500
assert len(platform.splitlines()) < 500
assert len(cli.splitlines()) < 500
assert len(authority.splitlines()) < 500
assert len(tests.splitlines()) < 500
print('Spec137 temporal authority release gate: PASS')
