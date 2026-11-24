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

## Focusa in Pi: terminal experience and conformance

### Outcome

An operator can open or resume Pi, know which Focusa project and workstream they are in, see what matters now, and switch, steer, verify, or recover without losing their place in the conversation. Focusa makes Pi more useful without becoming the subject of every turn. Pi remains the reference agent integration, useful on its own over SSH; Focusa remains the source of truth, and Focusa Desktop remains the intended home of the rich workspace.

### Requirements

1. **Earn the connection label.** Pi displays a workstream as connected only when the Focusa service confirms an attachment for this session to that exact project and workstream. Project recognition, a directory, a cached selection, a successful refresh, or service health alone is not attachment. Distinguish no project, no workstream, switching, connected with no active task, and stale/unavailable. On switch, resume, fork, reload, or disconnect, clear invalid work before rendering replacement state; late replies cannot repaint or authorize the old session. Use the existing typed scope and lifecycle contracts, not a new store. [52; 40 as qualified by 158/164; 135G; 157]
2. **One quiet default; useful depth on demand.** Consolidate the competing readiness and work widgets into one theme-aware, width-aware view above the editor. Default: one line for identity/state, up to one for actionable blocker or current/next work; avoid repeating built-in Pi status. Use plain labels, keyboard navigation, predictable focus, and states that make sense without color. The existing detailed view exposes current work, running/waiting jobs, evidence, and explicit workstream switching as supported. Diagnostics retain exact details on demand. Show session-local work by default; label wider views and never infer an action target from a highlighted row. [75; 135A/G; 135K usability principles; Desktop transition; Pi TUI docs]
3. **Keep the operator in control.** The newest request leads; stale work stays paused until it is relevant and valid. Launching Pi without a project is a normal state. Generic help and conversation remain available without setup; guidance or onboarding is optional unless a specific operation requires it. A real blocker names the affected operation and a useful next step, without presenting advisory gaps as blanket permission denials. Preserve scoped context and safe continuation through compaction and session replacement; do not make the human UI a second copy of the model's context. [52; 101/105/111/125/130/140; 135B/I for existing guided setup]
4. **Keep capability discovery comprehensive and current.** Audit in both directions: every tool actually loaded by Pi must match its current CLI and API semantics, and every supported agent-facing operation newly available through CLI or API must be callable or explicitly accounted for in Pi. Use the existing [Spec 90 parity contracts](90-ontology-backed-tool-contracts-parity-spec.md) and generated registry, not a second capability catalog. Compare method, route, arguments, schema, scope, authority, confirmation, side effects, results and recovery. Bind missing capabilities through existing typed adapters or add a focused tool where no equivalent exists. Classify aliases, composite tools, Pi-local, internal/admin-only, unavailable, and deprecated operations explicitly rather than duplicating tools or exposing raw endpoints. Make common actions easy to find and keep the complete catalog searchable without loading every schema into each turn. Registration and metadata agreement are not execution proof. [111; 135A/J; 171]
5. **Keep autonomous work moving without false alarms.** Distinguish an advisory, a failed read, a pending job, an actual refusal and verified completion. An advisory or optional reporting failure cannot stop unrelated authorized work; a failed queue read is not an empty queue, and dispatch is not completion. Reconcile uncertain effects before retrying. Continue ready work without routine operator updates or renewed approval, and surface only a real blocker, consequential decision, or material outcome. Do not widen scope, weaken admission, or claim progress without an observed result. Reuse the [cohesive project workflow](agent/02-focusa-cohesive-project-flow.md) and Spec 187.
6. **Ground claims: read the docs, test the behavior, then report remaining discrepancies.** Start with the original intent and the current owning specification or runbook; check what superseded older guidance. Cite the controlling section for claims about intended behavior. For claims about what actually works, test the current implementation through the relevant consumer, including the normal path and meaningful failure, recovery, and scope cases. Distinguish documented expectations, observed results, inferences, and unverified behavior. If a test disagrees with the docs, verify the version, scope, and test route; reproduce and try supported in-scope recovery before reporting a confirmed discrepancy. State expected versus observed behavior, evidence, and limits without generalizing from one failure. If no authoritative doc or sufficient proof exists, say what is unknown; do not invent intent or present a hypothesis as fact. This sequencing does not delay reporting a time-sensitive safety issue.

### Acceptance and delivery boundaries

- **Identity proof:** two projects and two workstreams in at least one project; exercise start, switch, resume, reload and compaction. Cover unbound launch, valid attachment without a task, denied action, service loss/reconnect, and out-of-order refresh. No old project/task flashes, wrong-workstream action, or stale connection label. Include headless Pi: presentation never supplies authority.
- **Terminal proof:** capture before/after from the real Pi terminal at narrow and wide widths (proposed 48/80/120 columns), including resize, monochrome, and keyboard-only use. Default is at most two Focusa lines and no repeated widget, with relevant work and recovery still reachable. A person can tell where they are, what matters, and where to go next without reading diagnostics.
- **Capability proof:** cover the whole inventory with a disposition; exercise representative reads, actions, refusals and recovery through the actual consumer. Reuse existing isolation, refresh, reload, width, and menu/parity tests. Require contract, producer, consumer, version-compatibility, and real installed Pi evidence before claiming delivery. Use the supported release/install path; retain the last verified package and supported rollback path, with no state migration assumed. [157; 171; production consistency and release rules]
- **Agent claim proof:** exercise documented behavior that passes, a wrong-route failure that recovers, a reproducible contract mismatch, and a case with missing documentation or untested behavior. Verify that the agent reads the owning docs first, tests through the actual consumer, keeps independent work moving, and reports only the remaining discrepancy or material uncertainty without claiming more than the evidence supports.
- **Not in this addendum:** no Pi fork, desktop-style canvas rebuild, new project/credential store, prompt or authority redesign, automatic project setup, or expansion into every adjacent Focusa roadmap feature. Preserve references and supported functionality; change the affected owning docs rather than adding a new Spec 135 companion.
