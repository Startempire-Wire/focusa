# Spec 120A — Architecture-to-Execution Workbench and Full Trajectory Handoff

**Status:** draft companion amendment / operator-reviewable  
**Owner:** Focusa / Verious Smith  
**Parent:** `120-adversarial-spec-workbench-and-operator-approval-gates.md`  
**Extends:** Spec 120 reality grounding, adversarial specification, operator gates, reconciliation, decomposition and implementation-start authorization  
**Reconciles with:** Focusa issue #618 Full/Medium/Short Trajectory requirements and the ADLBOS Algorithm² / Leverage² operating doctrine

---

## 0. One-line definition

Spec 120A turns Spec 120 from a strong spec-authoring workflow into a complete **architecture-to-execution workflow**: recover reality, reconcile the surrounding system, resolve the product through Strategy/Scope/Structure/Skeleton/Surface, derive one Full Trajectory, reduce the package with Algorithm², and hand a build agent enough product certainty to execute autonomously without burdening it with architecture ceremony.

Canonical target:

```text
Maximum product clarity.
Minimum execution ceremony.
```

---

## 1. Why this amendment exists

Spec 120 already correctly owns:

```text
rough operator idea
→ current docs/code reality
→ research
→ proposer
→ challenger
→ reference audit
→ operator approval
→ whole-spec reconciliation
→ task decomposition
→ implementation-start authorization
```

That is necessary but not always sufficient for a large product feature.

A spec can be internally coherent while still leaving a build agent to invent:

- product boundaries across adjacent systems;
- canonical ownership and authority seams;
- information architecture;
- interaction/state behavior;
- screen/component anatomy;
- visual language;
- complete accepted-outcome coverage;
- implementation sequencing and dependencies;
- how much freedom the build agent has below product-level decisions.

Spec 120A fills that gap without creating another canonical work authority, planning database, approval system, Evidence system or execution runtime.

---

## 2. Relationship to Spec 120

Spec 120A is an **architecting mode and amendment to the existing Spec 120 workflow**.

It does not replace:

```text
Reality Scanner
UIAI research
Spec Proposer
Adversarial Challenger
Reference Auditor
Synthesis Arbiter
operator authority
whole-spec reconciliation
provider-neutral decomposition
Workpoint / CallGraph / Evidence / Receipt ownership
```

It changes what “implementation ready” means for features whose product/UX/system architecture must be resolved before construction.

For those features:

```text
final prose spec approved
```

is not by itself the implementation-start condition.

The architecting work must also reach the Architecture-to-Execution exit condition in this amendment.

---

## 3. Core invariant

The architecting agent owns **product decision resolution**.

The build agent owns **ordinary engineering execution**.

The architecting agent must remove foundational ambiguity without prescribing every reversible implementation detail.

The target boundary is:

```text
RESOLVED BEFORE HANDOFF

product purpose
accepted scope
canonical owner / authority boundaries
system/product boundaries
user classes and critical needs
information architecture
interaction/state semantics
screen/component hierarchy where user-facing
visual language where user-facing
accepted outcome and completion meaning
major dependencies / upstream seams
Full Trajectory coverage

DELEGATED TO BUILD AGENT

internal function names
ordinary component/file boundaries
store organization
adapter internals
small refactors
CSS/layout mechanics within visual intent
focused test structure
implementation order among ready independent nodes
batching adjacent Workpoints
performance fixes
ordinary bug fixes
other reversible engineering choices
```

If the build agent still has to decide what product to build, architecting is incomplete.

If the build agent must service the architecture process more than build the product, architecting is overbuilt.

---

## 4. The interwoven Spec 120A workflow

Spec 120A does not run after Spec 120. The following concerns are woven through the Workbench from the beginning.

```text
OPERATOR INTENT
  ↓
REALITY RECOVERY
  ↓
ECOSYSTEM / ARCHITECTURE RECONCILIATION
  ↓
ADVERSARIAL PRODUCT REASONING
  ↓
FIVE-PLANE UX / PRODUCT RESOLUTION
  ↓
WHOLE-ARCHITECTURE RECONCILIATION
  ↓
FULL TRAJECTORY DERIVATION
  ↓
ALGORITHM² REDUCTION
  ↓
BUILD-AGENT READINESS TEST
  ↓
IMPLEMENTATION-START HANDOFF
  ↓
BUILD AGENT RUNS THE TRAJECTORY
```

These are responsibilities and sufficiency checks, not a requirement to create a new document for every arrow.

Existing valid artifacts may satisfy multiple stages.

---

# Part I — Architecture recovery and reconciliation

## 5. Intent recovery

Before drafting architecture, recover the actual product intent rather than interpreting the latest prompt in isolation.

Use, where relevant:

```text
operator direction
current product docs
current source
current runtime behavior
recent accepted decisions
existing specs
current issues / Trajectory
actual customer/deployment experience
connected ecosystem contracts
```

Separate:

```text
accepted intent
current implementation truth
older design intent
superseded assumptions
unresolved questions
```

Do not ask the operator for discoverable facts.

Ask only where operator judgment, preference, tradeoff, scope or authority is genuinely required.

---

## 6. Architecture reality pack

The Spec 120 Reality Scanner is extended for architecture work.

A sufficient architecture reality pack identifies only what materially affects the feature:

```yaml
architecture_reality:
  target_product:
  current_source_state:
  current_user_surfaces: []
  canonical_owner_map: []
  adjacent_products_and_contracts: []
  relevant_specs: []
  current_runtime_operations: []
  preserved_primitives: []
  known_stale_doctrine: []
  duplicated_or_overloaded_concepts: []
  unresolved_cross_product_seams: []
  implementation_constraints: []
```

This is a working synthesis, not a requirement for a standalone artifact if the same truth already exists elsewhere.

---

## 7. Reconcile before extending

Do not stack a new architecture on top of contradictory current doctrine.

Before inventing new primitives, compare the proposed feature against all material current owners.

For each concern, answer:

```text
Who owns this truth today?
Does that owner still match current product direction?
Would this feature duplicate it?
Is there a stale document that would misdirect implementation?
Is the apparent gap a missing contract or only a missing projection?
```

Typical concerns:

```text
identity
owner/operator authority
work/task authority
memory/context
entitlement
credentials
approvals/attention
Evidence / receipts / settlement
execution control
runtime/body placement
federation/networking
outcome/reputation
surface handoff
```

Update the owning architecture when necessary. Do not create a downstream workaround merely to avoid reconciling the source of truth.

---

## 8. Architecture distinction table

Where concepts are easy to collapse, the architecting agent must explicitly separate them before implementation.

Examples:

```text
product identity        != runtime identity
Operating Partner       != Foreman
Operating Partner       != architecture authority
entitlement             != authorization
authorization           != consent for this effect
fleet                    != sovereign federation
Evidence                 != accepted outcome
presenter acknowledgement != source resolution
trajectory coverage      != presentation detail
work lifecycle           != autonomy level
```

Only add distinctions that prevent a real ambiguity or wrong effect.

---

# Part II — Adversarial architecting inside Spec 120

## 9. Challenger duties expand beyond prose quality

For architecture-mode sections, the Spec 120 Challenger also attacks:

```text
wrong canonical owner
hidden duplicate state
product-boundary collision
stale assumptions
missing cross-product handoff
identity collapse
authority/entitlement collapse
unresolved failure/recovery path
UX ambiguity that would force build-time invention
scope hidden behind “future” wording
unnecessary abstraction or process
```

The Challenger should not reward complexity.

A valid objection must identify a concrete consequence, contradiction, missing accepted outcome or material implementation ambiguity.

---

## 10. Multi-pass reconciliation

Large features should receive at least two materially different reconciliation passes before architecture freeze:

### Pass A — cross-system consistency

Look outward:

```text
Does this fit current ecosystem/product ownership?
Does it conflict with adjacent products/specs?
Are names/identities/authority models overloaded?
Are public/product promises consistent with runtime truth?
```

### Pass B — internal semantic integrity

Look inward:

```text
Could two objects claim the same authority?
Could stale state look current?
Could a handoff transfer more authority than intended?
Could failure/replay duplicate effects?
Could a downstream agent interpret the same term differently?
Could completion be claimed before accepted scope is settled?
```

More passes are justified only when they find material new contradictions or the architecture materially changes.

Do not perform repeated review passes as ritual.

---

# Part III — Five-plane product/UX closure

## 11. Garrett five-plane sufficiency model

For user-facing features, Spec 120A uses Jesse James Garrett’s five planes as a sufficiency check.

They are not five mandatory documents.

### Strategy

Resolve:

```text
user/operator needs
product/business objective
primary actors
success condition
non-goals
```

Exit question:

> Do we know why this product exists, for whom, and what successful use means?

### Scope

Resolve:

```text
accepted functionality
content/information requirements
important states
permissions/boundaries
nonfunctional requirements
explicit exclusions
```

Exit question:

> Could implementation silently omit a required capability because scope remains implicit?

### Structure

Resolve:

```text
information architecture
navigation
interaction flows
scope/context behavior
state transitions
cross-surface handoffs
recovery/degraded behavior
```

Exit question:

> Could two competent builders produce materially different product behavior from the same spec?

### Skeleton

Resolve:

```text
screen hierarchy
primary information
primary and secondary actions
component anatomy
loading/empty/stale/error behavior
responsive interaction
```

Exit question:

> Would the builder need to invent major screen anatomy or action hierarchy?

### Surface

Resolve, where the feature has visual UI:

```text
visual character
typography
spacing
semantic colors
component treatment
motion
responsive visual behavior
reference surfaces where useful
```

Exit question:

> Would the builder need to choose a materially different visual language or interaction emphasis?

For non-visual/backend features, explicitly mark Skeleton/Surface as not applicable rather than manufacturing UI artifacts.

---

## 12. Five-plane closure law

A plane is resolved when the build agent can execute it without making a foundational product/design decision.

A plane is not resolved merely because prose exists.

Do not over-specify reversible implementation mechanics in order to claim a plane is complete.

---

# Part IV — Full Trajectory derivation

## 13. Full Trajectory is the implementation coverage model

Once accepted product architecture is sufficiently resolved, derive one issue-#618-aligned Full Trajectory.

```text
HLT
→ required MLG branches
→ STGs
→ Waypoints / Worksets / CallGraphs as owned by Focusa
→ nearest executable Workpoints
→ accepted outcome
```

Full / Medium / Short are projections of the same truth.

They are not separate plans and not synonyms for HLT/MLG/STG.

The trajectory must expose:

```text
accepted scope
active scope
ready scope
blocked scope
unresolved upstream dependencies
explicitly removed scope
current executable frontier
ultimate accepted outcome
```

Unknown distant implementation details are not fabricated.

An unknown that blocks progress gets an owned bounded resolution step.

---

## 14. Requirements connect upward and downward without bureaucracy

Coverage invariant:

```text
accepted requirement
→ implementation/disposition path or explicit unresolved dependency

implementation node
→ justified accepted outcome/requirement
```

Stable IDs are useful for coverage and debugging, not mandatory ticket ceremony.

Do not require every commit, edit, test or report to carry requirement IDs.

Do not create a separate proof packet per Workpoint.

---

## 15. Cross-functional thin path

Before broad horizontal build-out, identify the earliest production-shaped connected path that proves the architecture.

A good first path crosses the real owners necessary to establish the product center.

It should not be:

```text
mock-only
static UI
placeholder success
new local authority substituting for missing upstream truth
```

After the shared contracts are proven, open independent expansion lanes aggressively.

This reuses the Cross-Functional Alpha principle from the Spec 135 series without making Spec 120A dependent on that product family.

---

# Part V — Algorithm² applied to architecting

## 16. Apply Algorithm² twice

Before implementation handoff, apply:

```text
Question
→ Delete
→ Simplify
→ Accelerate
→ Automate last
```

First to the **feature/product architecture**.

Then apply it again to the **architecture/build machinery itself**.

### Question

For every retained major requirement, artifact, abstraction, gate and proof obligation:

> What wrong product, wrong effect, lost requirement or material failure does this prevent?

### Delete

Remove:

```text
duplicate docs
synonymous requirements
parallel stores/authorities
unnecessary approval layers
per-node status ceremony
proof artifacts that prove nothing new
repeated whole-system planning when dependencies did not change
speculative abstractions
serial sequencing that has no real dependency
```

### Simplify

Prefer:

```text
one owner per concern
one accepted scope register
one Full Trajectory
one build handoff
existing primitives
thin adapters
connected vertical behavior
```

### Accelerate

Enable:

```text
parallel independent work
batching adjacent Workpoints
implementation while context is hot
reuse of valid evidence
reconciliation only where reality changed
continuation around local blockers
```

### Automate last

Automate only repetition that survived the first four steps and has proven value.

Do not institutionalize ceremony by automating it.

---

## 17. Leverage²

When an implementation or architectural improvement is proven reusable:

```text
prove locally
→ identify the lowest correct shared owner
→ move/generalize only the reusable primitive
→ let later work inherit it
→ remove duplicated local workaround
```

Do not generalize speculative improvements merely because they sound reusable.

---

# Part VI — Build-agent autonomy contract

## 18. Architecting-agent exit test

Before implementation-start authorization, ask:

> Could a strong build agent take this package, make ordinary engineering decisions autonomously, and drive to the accepted outcome without redesigning the product?

If **no**, architecting is incomplete.

Then ask:

> Would the build agent spend material time servicing the architecture process instead of building?

If **yes**, the package is overbuilt and must be reduced.

The correct exit lies between those failures.

---

## 19. Build handoff minimum

The handoff should usually contain only:

```text
accepted product outcome
current architectural/product boundaries
current Full Trajectory / Short frontier
where to find deeper plane-specific detail
current source/runtime truth
known genuine upstream blockers
implementation autonomy boundary
promotion/deployment boundary
```

Do not force the build agent to preload every supporting architecture document.

Load detail on demand from the active frontier.

---

## 20. Build-agent autonomy law

After architecting closes:

```text
PRODUCT DECISIONS
resolved by architecting/operator process

ENGINEERING DECISIONS
freely delegated to build agent inside those rails
```

The build agent may, without operator permission:

```text
choose reversible internal implementation details
batch adjacent Workpoints
reorder independent ready work
refactor opportunistically
fix discovered implementation defects
choose focused verification
continue around local blockers
```

Owner/operator input is required only when the unresolved decision materially changes:

```text
accepted scope / product behavior
owner authority
privacy/security boundary
meaningful spend
irreversible or high-consequence external effect
major ecosystem/product boundary
```

Minor ambiguity is not a blocker.

---

## 21. Outcomes-over-process build law

The default build loop is:

```text
current frontier
→ inspect real source
→ implement the largest safe useful increment
→ verify material behavior/risk
→ fix/reconcile what changed
→ continue
```

Do not stop because:

```text
a Workpoint ended
an STG/MLG ended
a commit landed
a test passed
a preferred tool failed
a report could be written
documentation could be polished
```

Stop only at accepted completion, an actual affected dependency/authority boundary, or a genuine owner decision.

---

## 22. Minimum sufficient verification

Keep a verification step only when removing it would leave an applicable acceptance condition or concrete material failure risk unverified.

Prefer proof naturally produced by doing the work:

```text
source revision/diff
focused test
build/manifest result
real browser/consumer behavior
owning-system state/receipt
```

One check may satisfy several accepted requirements when it genuinely establishes them.

Do not create proof-management work around valid proof.

Run broad integration/regression at meaningful integration/release boundaries, not reflexively after each small change.

---

# Part VII — Spec 120 gate amendments

## 23. Section approval under architecture mode

Spec 120 section approval remains operator-controlled, but the architecting workflow should group routine detail around actual product decisions.

Operator attention should concentrate on:

```text
scope
tradeoffs
product behavior
architecture boundaries
important UX decisions
risk/authority/privacy decisions
meaningful exclusions
final architecture acceptance
```

Do not turn every derived implementation detail into a new operator gate.

Where multiple sections merely express one already-resolved operator decision, they may be reviewed/approved together through one exact scoped gate if the underlying Spec 120 implementation supports grouped revisions.

Until grouped gates exist, the Workbench may preserve section records while presenting them as one operator decision cluster.

---

## 24. Whole-spec reconciliation is expanded

Spec 120 whole-spec reconciliation under architecture mode also checks:

```text
canonical owner consistency
product-boundary consistency
five-plane consistency
cross-surface behavior consistency
identity/authority terminology
accepted-scope coverage
Full Trajectory completeness
unresolved upstream dependencies
build-agent ambiguity
process/ceremony bloat
```

Reconciliation should propose deletion/simplification as readily as additions.

---

## 25. Decomposition amendment

Provider-neutral task decomposition remains supported, but **Full Trajectory is the governing coverage/dependency model** for architecture-mode features.

Task-provider items are projections/adapters over admitted work. They are not a second project plan.

Do not require one provider ticket per Workpoint.

A provider item may cover several adjacent Workpoints when ownership, dependencies and acceptance remain clear.

---

## 26. Implementation-start gate amendment

For architecture-mode features, implementation start is authorized when:

```text
operator intent and accepted scope are clear
current reality and canonical ownership are reconciled
material contradictions are resolved or explicitly bounded
applicable Garrett planes are sufficiently resolved
one Full Trajectory covers the accepted outcome
current Short frontier is executable or has an owned resolution node
Algorithm² has removed unnecessary architecture/process
build-agent autonomy boundary is explicit
required consequential operator decisions are closed
```

The following are **not** required merely for ceremony:

```text
a unique document for every stage
a ticket for every Workpoint
proof packets for every node
repeated full review with no new material finding
implementation-detail operator approval
full-corpus preload by the build agent
```

---

## 27. Architecture-mode completion artifact

The architecture package may be one document or several owner-specific artifacts.

What matters is that the following truth exists and is discoverable:

```text
accepted scope
canonical ownership and seams
five-plane product/UX decisions where applicable
Full Trajectory
current frontier
known blockers
build-agent autonomy law
minimum promotion boundary
```

Do not create another architecture database solely to package these references.

---

## 28. Definition of architecting done

Architecting is done when:

```text
1. The current product/system reality has been inspected rather than assumed.
2. Relevant existing specs/products have been reconciled and stale contradictions are not silently inherited.
3. Canonical ownership and authority boundaries are unambiguous.
4. Material adversarial objections are resolved, bounded or explicitly accepted.
5. Applicable Strategy, Scope, Structure, Skeleton and Surface decisions are sufficiently resolved.
6. Accepted scope is represented by one complete Full Trajectory with an exact current frontier.
7. Unknowns are explicit and owned rather than fabricated.
8. Algorithm² has removed unnecessary product and process complexity.
9. The build agent has broad engineering discretion beneath the resolved product boundary.
10. The build agent can begin immediately from real source truth without reopening foundational product design.
11. The handoff does not impose proof/report/documentation ceremony unrelated to true completion.
12. Operator involvement after handoff is limited to genuine owner-level decisions or newly discovered material scope/authority changes.
```

---

## 29. Reference workflow — the pattern proven by the Focusa Workforce redesign

The reusable pattern is:

```text
recover product intent
→ inspect current implementation
→ inspect relevant upstream/current architecture
→ reconcile boundaries and canonical owners
→ find contradictions/stale doctrine/missing seams
→ update owning architecture before stacking more design
→ run outward consistency pass
→ run inward semantic-integrity pass
→ resolve Strategy
→ resolve Scope
→ resolve Structure
→ resolve Skeleton
→ resolve Surface
→ derive one Full Trajectory
→ test whether a build agent would still need to invent product decisions
→ close remaining foundational ambiguity
→ apply Algorithm² to the feature
→ apply Algorithm² again to the architecture/build process
→ remove proof/process/documentation ceremony
→ explicitly separate product decisions from engineering discretion
→ apply Leverage² to proven reusable primitives
→ produce one lean build-agent handoff
→ implementation begins from the Short frontier
```

This is a pattern, not a requirement to repeat the exact number of documents produced by that feature.

---

## 30. Final principle

Spec 120 protects the transition from idea to grounded, adversarially tested, operator-approved specification.

Spec 120A ensures that for architecture-heavy features the result is not merely a good document, but a **resolved product architecture that can be executed autonomously**.

The governing standard is:

> **Give the build agent enough rails that it cannot accidentally build the wrong product, and enough freedom that the rails do not become the work.**
