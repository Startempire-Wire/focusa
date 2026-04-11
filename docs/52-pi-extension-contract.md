# Pi Extension Contract

## Purpose

This document defines the formal integration contract between Pi and Focusa.

Pi must be a disciplined consumer and producer at the harness edge.
It must not become a parallel cognitive system.

## Contract Goals

Pi should:
- consume bounded ontology slices
- act within the current mission and active focus
- emit typed proposals and action intents
- stay reducer-compatible
- operate safely in degraded mode

## Pi Input Contract

Pi must be able to receive:
- active mission
- active frame / thesis
- active working set
- applicable constraints
- recent relevant decisions
- recent verified deltas
- unresolved blockers/open loops
- allowed actions
- degraded-mode flag if applicable

## Pi Output Contract

Pi may emit:
- `OntologyProposal`
- `OntologyActionIntent`
- `VerificationRequest`
- `EvidenceLinkedObservation`
- `FailureSignal`
- `BlockerSignal`
- `ScratchReasoningRecord`
- `DecisionCandidate`

Pi may not emit:
- direct canonical ontology writes
- direct reducer bypass writes
- parallel long-lived local world state

## Operator steering precedence

Pi must treat the operator’s newest explicit input as the primary conversation/action driver.

Pi may consult Focusa state after determining:
- whether the active mission still applies
- which constraints are relevant
- which decisions are relevant

Pi must not:
- let injected focus state become the default subject
- continue stale mission context when operator steering has changed
- answer daemon/metacognitive context instead of the operator’s actual request

## Success Condition

This document is satisfied when Pi behaves as a thin, disciplined harness-side adapter to Focusa rather than a second cognitive authority.
