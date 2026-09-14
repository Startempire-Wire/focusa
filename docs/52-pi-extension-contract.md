# Pi Extension Contract

## Purpose

This document defines the formal integration contract between Pi and Focusa.

**Pi is Focusa's default/reference agent harness integration.** The Focusa Pi extension is fundamental to the reference Focusa agent experience and is the leading integration surface for typed Focusa tools, skills/runbooks, authority hooks, Workpoint/Trajectory continuity, evidence, metacognition and recovery.

That reference role does not make Pi a parallel cognitive authority or make Focusa state Pi-private. Focusa daemon/core remains the canonical cognitive/operational authority in its domain, and non-Pi harnesses remain first-class through thin adapters and the same generated capability contracts.

Pi must be a disciplined consumer and producer at the harness edge.
It must not become a parallel cognitive system.

## Contract Goals

Pi should:
- consume bounded ontology slices
- act within the current mission and active focus
- emit typed proposals and action intents
- stay reducer-compatible
- operate safely in degraded mode
- exercise the canonical generated Focusa capability contracts rather than inventing Pi-only truth
- provide the reference harness behavior that non-Pi adapters should match where their host capabilities permit

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

## Cross-harness portability

Focusa must keep canonical state and capability semantics outside Pi-specific hidden state.

Non-Pi harnesses may differ in transport, presentation, tool-loading mechanism, session tree or user experience, but adapters should preserve the same essential authority order and outcome semantics:

```text
ProjectIdentity
→ Trajectory / Workpoint
→ scoped capability
→ action / observation
→ Evidence / receipt
→ recovery / continuation
```

Pi is the reference implementation of that harness-edge relationship, not the only permitted host.

## Success Condition

This document is satisfied when Pi behaves as the default/reference thin, disciplined harness-side integration to Focusa rather than a second cognitive authority, and when the same canonical Focusa state remains available to compatible non-Pi harnesses without being reconstructed from Pi-private memory.
