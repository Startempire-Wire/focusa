#!/usr/bin/env python3
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]
core=(ROOT/'crates/focusa-core/src/temporal.rs').read_text()
tests=(ROOT/'crates/focusa-core/src/temporal_tests.rs').read_text()
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
assert 'pub mod temporal;' in lib
assert len(core.splitlines()) < 500
assert len(tests.splitlines()) < 500
print('Spec137 temporal authority release gate: PASS')
