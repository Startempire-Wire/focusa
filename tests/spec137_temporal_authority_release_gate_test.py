#!/usr/bin/env python3
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]
core=(ROOT/'crates/focusa-core/src/temporal.rs').read_text()
tests=(ROOT/'crates/focusa-core/src/temporal_tests.rs').read_text()
authority=(ROOT/'crates/focusa-core/src/temporal_authority.rs').read_text()
claims=(ROOT/'crates/focusa-core/src/temporal_claims.rs').read_text()
lib=(ROOT/'crates/focusa-core/src/lib.rs').read_text()
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
assert 'pub mod temporal;' in lib and 'pub mod temporal_authority;' in lib and 'pub mod temporal_claims;' in lib
assert len(core.splitlines()) < 500
assert len(claims.splitlines()) < 500
assert len(authority.splitlines()) < 500
assert len(tests.splitlines()) < 500
print('Spec137 temporal authority release gate: PASS')
