#!/usr/bin/env python3
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]
core=(ROOT/'crates/focusa-core/src/temporal.rs').read_text()
tests=(ROOT/'crates/focusa-core/src/temporal_tests.rs').read_text()
authority=(ROOT/'crates/focusa-core/src/temporal_authority.rs').read_text()
claims=(ROOT/'crates/focusa-core/src/temporal_claims.rs').read_text()
forecast=(ROOT/'crates/focusa-core/src/temporal_forecast.rs').read_text()
lib=(ROOT/'crates/focusa-core/src/lib.rs').read_text()
api=(ROOT/'crates/focusa-api/src/routes/temporal.rs').read_text()
cli=(ROOT/'crates/focusa-cli/src/commands/temporal.rs').read_text()
pi=(ROOT/'apps/pi-extension/src/tools.ts').read_text()
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
 assert durability in core, durability
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
assert 'TemporalCmd' in cli and 'name: "focusa_temporal_authority"' in pi
assert 'confirmation_required' in api and 'forecast_history_insufficient' in api
assert len(api.splitlines()) < 500
assert len(cli.splitlines()) < 500
assert len(authority.splitlines()) < 500
assert len(tests.splitlines()) < 500
print('Spec137 temporal authority release gate: PASS')
