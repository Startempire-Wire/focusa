# Spec 120A — Cross-Vertical Architecture-to-Execution Workbench and Full Trajectory Handoff

**Status:** draft companion amendment / operator-reviewable  
**Owner:** Focusa / Verious Smith  
**Parent:** `120-adversarial-spec-workbench-and-operator-approval-gates.md`  
**Extends:** Spec 120 reality grounding, adversarial specification, operator gates, reconciliation, decomposition and implementation-start authorization  
**Reconciles with:** Focusa issue #618 Full/Medium/Short Trajectory requirements and the ADLBOS Algorithm² / Leverage² operating doctrine

---

## 0. One-line definition

Spec 120A turns Spec 120 into a reusable **cross-vertical architecture-to-execution workflow**: recover real operating context, reconcile surrounding systems and authority, resolve the experience and outcome deeply enough to prevent downstream invention, derive one Full Trajectory, reduce the package with Algorithm², and hand an execution agent enough certainty to advance autonomously without turning architecture into a second workload.

Canonical target:

```text
Maximum outcome clarity.
Minimum execution ceremony.
```

Spec 120A is intentionally domain-neutral. Software engineering, legal/compliance, operations, research, sales, physical-world systems and other verticals add domain-specific profiles rather than changing the core lifecycle.

---

## 1. Why this amendment exists

Spec 120 already owns:

```text
rough operator intent
→ current reality grounding
→ research
→ proposer
→ challenger
→ reference audit
→ operator approval
→ whole-spec reconciliation
→ decomposition
→ execution-start authorization
```

That is necessary but not always sufficient for complex work.

A specification can be coherent while still leaving an execution agent to invent:

- boundaries between adjacent systems, organizations or domains;
- canonical ownership and authority seams;
- actors, responsibilities and decision rights;
- information/interaction structure where people use a surface or workflow;
- failure, recovery and handoff behavior;
- complete accepted-outcome coverage;
- dependencies and executable frontier;
- how much discretion the execution agent has below operator-level decisions.

Spec 120A fills that gap without creating another canonical work authority, planner, approval system, Evidence system or execution runtime.

---

## 2. Core versus vertical profile

Spec 120A owns only cross-vertical architecture-to-execution doctrine.

A **vertical profile** specializes the doctrine for one domain.

Examples:

```text
Spec 120A-SWE   software engineering
future profile  business operations
future profile  legal/compliance
future profile  scientific/research
future profile  sales/marketing
future profile  physical-world / field operations
```

A vertical profile may define:

```text
domain-specific reality sources
domain terminology
specialized actors/roles
specialized design/sufficiency lenses
specialized execution artifacts
specialized risk/authority boundaries
specialized verification methods
specialized delivery/activation semantics
```

A profile MUST NOT create a second Workpoint, Trajectory, Evidence, authority, approval or execution system.

The core rule is:

> **Keep the universal reasoning/execution loop in Focusa core; keep domain mechanics in the selected vertical profile.**

---

## 3. Architecting versus execution responsibility

The architecting process owns **outcome and system decision resolution**.

The execution agent owns **ordinary reversible execution choices** inside those rails.

Resolve before handoff:

```text
desired outcome
accepted scope
canonical ownership / authority
system and organizational boundaries
primary actors and critical needs
important workflow / information structure
important state and handoff semantics
accepted completion meaning
major dependencies
Full Trajectory coverage
vertical-specific decisions required by the active profile
```

Delegate after handoff:

```text
ordinary reversible implementation/execution choices
local sequencing among independent ready work
internal organization of the work
small refactors/corrections
focused verification method
performance/efficiency improvements
routine recovery from local tool/path failures
```

If the execution agent still has to decide what outcome or system to create, architecting is incomplete.

If the execution agent must spend material time servicing the architecture process, architecting is overbuilt.

---

## 4. Interwoven Spec 120A workflow

Spec 120A is woven into Spec 120 from the start:

```text
OPERATOR INTENT
  ↓
REALITY RECOVERY
  ↓
SYSTEM / AUTHORITY RECONCILIATION
  ↓
ADVERSARIAL DOMAIN REASONING
  ↓
EXPERIENCE / WORKFLOW / OUTCOME RESOLUTION
  ↓
WHOLE-ARCHITECTURE RECONCILIATION
  ↓
FULL TRAJECTORY DERIVATION
  ↓
ALGORITHM² REDUCTION
  ↓
EXECUTION-READINESS TEST
  ↓
LEAN HANDOFF
  ↓
EXECUTION AGENT RUNS THE TRAJECTORY
```

These are responsibilities and sufficiency checks, not mandatory new documents.

Existing valid artifacts may satisfy multiple stages.

---

# Part I — Reality and architecture reconciliation

## 5. Intent recovery

Recover the actual operator intent rather than interpreting the latest prompt in isolation.

Use, where relevant:

```text
operator direction
current domain records/documents
current systems and actual behavior
recent accepted decisions
existing specs/policies
current Trajectory/work state
customer/stakeholder experience
connected ecosystem contracts
observed real-world outcomes
```

Separate:

```text
accepted intent
current observed truth
older design/policy intent
superseded assumptions
unresolved operator decisions
```

Do not ask the operator for discoverable facts.

Ask only where operator judgment, preference, tradeoff, scope or authority is genuinely required.

---

## 6. Architecture reality pack

The Spec 120 Reality Scanner is extended for architecture work.

A sufficient reality pack identifies only what materially affects the outcome:

```yaml
architecture_reality:
  target_domain:
  target_system_or_outcome:
  current_observed_state:
  current_user_or_operator_surfaces: []
  canonical_owner_map: []
  adjacent_systems_and_contracts: []
  relevant_specs_policies_records: []
  current_operations_capabilities: []
  preserved_primitives: []
  stale_or_conflicting_doctrine: []
  duplicated_or_overloaded_concepts: []
  unresolved_cross-system_seams: []
  domain_constraints: []
```

This is a working synthesis, not a required standalone artifact when equivalent truth already exists.

---

## 7. Reconcile before extending

Do not stack a new design on top of contradictory authority or stale doctrine.

For each material concern ask:

```text
Who owns this truth today?
Is that still the intended owner?
Would the proposed work duplicate or bypass it?
Is a stale document/policy/process likely to misdirect execution?
Is the apparent gap a missing capability, missing projection, or merely missing coordination?
```

Typical cross-vertical concerns:

```text
identity
owner/operator authority
roles and delegation
work authority
memory/context
entitlement/access
credentials/secrets
approvals/attention
Evidence / receipts / settlement
execution control
resource/body/runtime placement
network/federation boundaries
outcome/standing/reputation
handoffs between systems or people
```

Correct the owning architecture when necessary. Do not create downstream workarounds to avoid fixing the source of truth.

---

## 8. Distinction table

Where concepts are easy to collapse, explicitly separate them before execution.

Examples:

```text
person/organization identity != runtime/session identity
operating partner           != domain supervisor
operating role              != architecture authority
entitlement                 != authorization
authorization               != consent for this effect
resource fleet              != cross-owner federation
Evidence                     != accepted outcome
presentation acknowledgement != source resolution
trajectory coverage         != presentation detail
work lifecycle              != autonomy level
```

Only add distinctions that prevent a real ambiguity or wrong effect.

---

# Part II — Adversarial architecting

## 9. Challenger duties

Under architecture mode, the Spec 120 Challenger attacks more than prose quality:

```text
wrong canonical owner
hidden duplicate state or process
system-boundary collision
stale assumptions
missing handoff or return path
identity/authority collapse
unresolved failure/recovery path
experience/workflow ambiguity that forces downstream invention
scope hidden behind “future” wording
unnecessary abstraction, approval or process
```

A valid objection must identify a concrete consequence, contradiction, missing accepted outcome or material execution ambiguity.

Complexity by itself is not quality.

---

## 10. Two reconciliation passes

Large or cross-system work receives two materially different passes before architecture freeze.

### Pass A — outward consistency

```text
Does this fit the surrounding ecosystem/domain?
Does it conflict with adjacent systems/specs/policies?
Are names/roles/authority overloaded?
Are external promises consistent with operational truth?
```

### Pass B — inward semantic integrity

```text
Could two objects/processes claim the same authority?
Could stale information look current?
Could a handoff carry unintended authority?
Could retry/replay duplicate a consequential effect?
Could different agents interpret the same term differently?
Could completion be claimed while accepted scope remains unsettled?
```

More passes are justified only when material new contradictions appear or the architecture materially changes.

---

# Part III — Experience and workflow sufficiency

## 11. Universal sufficiency questions

Every vertical resolves these questions where applicable:

### Intent / Strategy

```text
Who needs this?
What outcome matters?
What business/mission objective does it serve?
What is explicitly not the goal?
How is success recognized?
```

### Scope

```text
What capability/work is accepted?
What information/artifacts are required?
What states/permissions/boundaries matter?
What nonfunctional/domain constraints apply?
What is explicitly excluded?
```

### Structure

```text
How does work/information flow?
What are the decision and interaction paths?
How is context/scope selected or inherited?
How do state transitions and handoffs work?
How does degraded/recovery behavior work?
```

### Concrete operating form

Resolve enough concrete anatomy that the execution agent does not invent the operating model.

Depending on domain this may mean:

```text
screens/components
forms/reports
queues/checklists
work cells
approval packets
field procedures
research protocols
service workflows
physical stations/devices
other domain artifacts
```

### Presentation / sensory form

Only where materially relevant, resolve the visual/physical/presentation language enough to prevent a materially different experience from being invented downstream.

---

## 12. Garrett five planes are a UI/product vertical lens, not universal core law

For interactive digital product work, a vertical profile may use Jesse James Garrett's:

```text
Strategy
Scope
Structure
Skeleton
Surface
```

as the concrete sufficiency model.

The core runtime MUST NOT require every vertical to manufacture screen hierarchy, typography or visual-system artifacts.

For non-UI work, the active vertical profile supplies the equivalent domain lenses.

---

# Part IV — Full Trajectory

## 13. Full Trajectory is the cross-vertical coverage model

Once the outcome architecture is sufficiently resolved, derive one issue-#618-aligned Full Trajectory:

```text
HLT / accepted outcome
→ required MLG branches
→ STGs
→ Waypoints / Worksets / CallGraphs where applicable
→ nearest executable Workpoints
→ accepted outcome settlement
```

Full / Medium / Short are projections of the same truth.

The trajectory exposes:

```text
accepted scope
active scope
ready scope
blocked scope
unresolved dependencies
explicitly removed scope
current executable frontier
ultimate accepted outcome
```

Unknown distant details are not fabricated. A blocking unknown gets an owned bounded resolution step.

---

## 14. Coverage without bureaucracy

Invariant:

```text
accepted requirement/outcome
→ execution/disposition path or explicit unresolved dependency

execution node
→ justified accepted outcome/requirement
```

Stable IDs may aid coverage and debugging. They are not ticket bureaucracy.

Do not require every action, artifact or report to carry IDs.

Do not create one proof packet per Workpoint.

---

## 15. Thin connected outcome path

Before broad horizontal expansion, identify the earliest real connected path that exercises the essential owners and produces a meaningful domain outcome.

It must not substitute:

```text
mock-only state
placeholder success
pure presentation
local duplicate authority
```

for the real outcome.

Once the shared path works, expand independent lanes aggressively.

---

# Part V — Algorithm² and Leverage²

## 16. Apply Algorithm² twice

Apply:

```text
Question
→ Delete
→ Simplify
→ Accelerate
→ Automate last
```

first to the **solution/outcome architecture**, then to the **machinery used to design and execute it**.

### Question
For every retained requirement, artifact, abstraction, gate and verification obligation:

> What wrong outcome, wrong effect, lost requirement or material failure does this prevent?

### Delete
Remove:

```text
duplicate documents/processes
synonymous requirements
parallel authorities/stores
unnecessary approvals
per-node status ceremony
proof artifacts that prove nothing new
repeated full replanning when dependencies did not change
speculative abstractions
serial sequencing without real dependency
```

### Simplify
Prefer:

```text
one owner per concern
one accepted scope
one Full Trajectory
one handoff
existing primitives
thin adapters/projections
connected end-to-end behavior
```

### Accelerate
Enable:

```text
parallel independent work
batching adjacent Workpoints
execution while context is hot
reuse of valid Evidence
reconciliation only where reality changed
continuation around local blockers
```

### Automate last
Automate only repetition that survived the first four steps and proved useful.

Do not automate ceremony into permanence.

---

## 17. Leverage²

When an improvement proves reusable:

```text
prove locally
→ identify lowest correct shared owner
→ generalize only the reusable primitive
→ let later work inherit it
→ remove duplicated local workaround
```

Do not generalize speculation.

---

# Part VI — Execution-agent autonomy

## 18. Architecting exit test

Before execution-start authorization ask:

> Could a strong execution agent take this package, make ordinary domain-execution decisions autonomously, and drive to the accepted outcome without redesigning the outcome/system?

If no, architecting is incomplete.

Then ask:

> Would the execution agent spend material time servicing the architecture process instead of producing the outcome?

If yes, the package is overbuilt and must be reduced.

---

## 19. Lean handoff

The handoff normally contains only:

```text
accepted outcome
current architecture / authority boundaries
current Full Trajectory / Short frontier
where deeper vertical-specific detail lives
current operating truth
known genuine blockers
autonomy boundary
activation/delivery boundary
```

Do not preload every supporting document.

Load detail on demand from the active frontier.

---

## 20. Execution autonomy law

After architecting closes:

```text
OUTCOME / SYSTEM DECISIONS
resolved through architecting/operator process

ORDINARY EXECUTION DECISIONS
freely delegated inside those rails
```

The execution agent may without operator permission:

```text
choose reversible local execution details
batch adjacent Workpoints
reorder independent ready work
refactor/reorganize local implementation/process
fix discovered defects
choose focused verification
continue around local blockers
```

Operator input is required only when the unresolved decision materially changes:

```text
accepted scope/outcome
owner authority
privacy/security/legal boundary
meaningful spend/resource commitment
irreversible/high-consequence external effect
major ecosystem/system boundary
```

Minor ambiguity is not a blocker.

---

## 21. Outcomes-over-process execution law

Default loop:

```text
current frontier
→ inspect current reality
→ execute largest safe useful increment
→ verify material outcome/risk
→ fix/reconcile what changed
→ continue
```

Do not stop because:

```text
a Workpoint ended
an STG/MLG ended
an artifact was produced
a check passed
a preferred tool/path failed
a report could be written
documentation could be polished
```

Stop only at accepted completion, an actual affected dependency/authority boundary, or a genuine operator decision.

---

## 22. Minimum sufficient verification

Keep a verification step only when removing it would leave an applicable acceptance condition or concrete material failure risk unverified.

Prefer Evidence naturally produced by doing the work:

```text
observed source/state change
focused test/check
consumer/user-visible behavior
owning-system record/receipt
measured real-world result
```

One check may satisfy several obligations when it genuinely establishes them.

Do not build a proof-management workload around valid Evidence.

---

# Part VII — Spec 120 gate amendments

## 23. Operator attention under architecture mode

Operator attention should concentrate on genuine operator decisions:

```text
scope
tradeoffs
outcome behavior
architecture/system boundaries
important experience/workflow decisions
risk/authority/privacy/legal decisions
meaningful exclusions
final architecture acceptance
```

Do not turn derived execution detail into an operator gate.

---

## 24. Whole-spec reconciliation expands

Under architecture mode, Spec 120 whole-spec reconciliation also checks:

```text
canonical owner consistency
system-boundary consistency
active vertical-profile sufficiency
cross-surface/workflow consistency
identity/authority terminology
accepted-scope coverage
Full Trajectory completeness
unresolved dependencies
execution-agent ambiguity
process/ceremony bloat
```

Reconciliation should propose deletion/simplification as readily as additions.

---

## 25. Decomposition amendment

Provider/work-item decomposition remains supported, but **Full Trajectory is the governing coverage/dependency model**.

Provider/task/work-order items are projections over admitted work, not a second project plan.

Do not require one provider item per Workpoint.

---

## 26. Execution-start gate

Architecture-mode execution may begin when:

```text
operator intent and accepted scope are clear
current reality and canonical ownership are reconciled
material contradictions are resolved or explicitly bounded
active vertical-profile sufficiency checks are resolved
one Full Trajectory covers the accepted outcome
current Short frontier is executable or has an owned resolution node
Algorithm² removed unnecessary architecture/process
execution-agent autonomy boundary is explicit
required consequential operator decisions are closed
```

Not required merely for ceremony:

```text
a unique document for every stage
a task/ticket for every Workpoint
proof packets for every node
repeated full review with no new finding
operator approval of ordinary execution details
full-corpus preload by the execution agent
```

---

## 27. Definition of architecting done

Architecting is done when:

```text
1. Current domain/system reality was inspected rather than assumed.
2. Relevant systems/specs/policies were reconciled; stale contradictions are not silently inherited.
3. Canonical ownership and authority are unambiguous.
4. Material adversarial objections are resolved, bounded or explicitly accepted.
5. Active vertical-profile sufficiency checks are resolved.
6. Accepted scope is represented by one complete Full Trajectory with exact current frontier.
7. Unknowns are explicit and owned rather than fabricated.
8. Algorithm² removed unnecessary solution and process complexity.
9. Execution agent has broad discretion beneath resolved outcome boundaries.
10. Execution can begin from real current truth without reopening foundational decisions.
11. Handoff imposes no proof/report/documentation ceremony unrelated to true completion.
12. Operator involvement after handoff is limited to genuine operator-level decisions or newly discovered material changes.
```

---

## 28. Reference pattern proven by the Focusa Workforce redesign

The reusable cross-vertical pattern is:

```text
recover intent
→ inspect current reality
→ inspect relevant upstream/current architecture
→ reconcile boundaries and owners
→ find contradictions/stale doctrine/missing seams
→ update owning architecture before stacking more design
→ outward consistency pass
→ inward semantic-integrity pass
→ resolve active vertical-profile sufficiency lenses
→ derive one Full Trajectory
→ test whether execution agent would still need to invent foundational decisions
→ close remaining ambiguity
→ Algorithm² on the solution
→ Algorithm² again on the architecture/execution machinery
→ remove proof/process/documentation ceremony
→ separate operator/architecture decisions from execution discretion
→ Leverage² proven reusable primitives
→ produce one lean handoff
→ execution begins from Short frontier
```

The pattern does not require the same artifacts in every vertical.

---

## 29. Final principle

Spec 120 protects the transition from rough intent to grounded, adversarially tested, operator-approved specification.

Spec 120A ensures that complex work leaves the Workbench as a **resolved outcome architecture that can be executed autonomously across verticals**.

> **Give the execution agent enough rails that it cannot accidentally pursue the wrong outcome, and enough freedom that the rails do not become the work.**
