# Spec 120A-SWE — Software Engineering Architecture-to-Execution Vertical Profile

**Status:** draft vertical profile / operator-reviewable  
**Owner:** Focusa / Verious Smith  
**Core:** `120a-architecture-to-execution-workbench-and-full-trajectory-handoff-spec.md`  
**Parent workflow:** `120-adversarial-spec-workbench-and-operator-approval-gates.md`  
**Applies to:** software products, applications, browser extensions, APIs, CLIs, services, libraries, infrastructure, developer tools and software-heavy integrations

---

## 0. One-line definition

Spec 120A-SWE specializes the cross-vertical Spec 120A architecting process for software engineering so a coding/build agent receives a reconciled product architecture, resolved user experience where applicable, one complete Full Trajectory, real source/runtime constraints, and broad engineering autonomy without being burdened by proof or process ceremony.

Target:

```text
Resolved software product decisions.
Free-flowing engineering execution.
```

---

## 1. What belongs here instead of Focusa core

Software-specific concepts belong in this profile:

```text
repository and branch reality
source code and runtime behavior
API / CLI / tool contracts
data schemas and migrations
frontend information architecture
screen/component hierarchy
visual design system
browser/desktop/mobile constraints
unit/contract/integration/browser tests
build and package systems
CI
release/deployment
backward compatibility
observability/runtime diagnostics
software security boundaries
```

Spec 120A core should not assume these concepts exist in every vertical.

---

## 2. Activation

Use this profile when the accepted outcome materially requires creating or changing software.

Examples:

```text
new product or feature
software redesign
browser extension
web/mobile/desktop application
API or service
CLI/tooling
agent/runtime capability
infrastructure control surface
software integration
large refactor with user/product consequences
```

For a mixed-domain project, apply this profile only to the software branch of the Full Trajectory.

---

# Part I — Software reality recovery

## 3. Software reality pack

Extend the Spec 120A reality pack with the smallest useful current software truth:

```yaml
software_reality:
  repositories: []
  default_and_active_branches: []
  current_source_surfaces: []
  current_runtime_surfaces: []
  generated_contracts: []
  api_routes: []
  cli_commands: []
  tool_operations: []
  data_schemas: []
  current_user_surfaces: []
  tests_and_acceptance: []
  build_package_system:
  ci_release_path:
  deployed_consumers: []
  compatibility_constraints: []
  preserved_primitives: []
  known_stale_docs_or_specs: []
  known_runtime_doc_mismatches: []
```

Do not scan the entire codebase indiscriminately when a bounded set of files/contracts answers the question.

Reality beats architectural memory.

---

## 4. Software-specific reconciliation questions

Before adding architecture, resolve:

```text
Where does canonical state live?
Which reducer/store/service owns each mutation?
Which API/CLI/tool operation already exists?
Is a missing UI capability actually a missing backend contract?
Are two clients creating parallel state?
Are docs describing behavior that is not mounted/running?
Are current generated schemas authoritative?
Are authentication, authorization, entitlement and consent being collapsed?
Can retries duplicate effects?
Can stale client state authorize a mutation?
What survives process/browser/device restart?
What must remain backward compatible?
What is the real build/deployment path?
```

Do not solve an upstream contract gap by inventing client-side canonical state.

---

## 5. Preserve behavior, not accidental file shape

Identify proven primitives that should normally be reused.

Examples:

```text
auth/pairing
API client
schema validation
state reconciliation
retry/idempotency
storage
SSE/event handling
execution adapters
permission enforcement
existing generated clients
release/deployment mechanics
```

Refactor them when needed. Do not preserve accidental structure merely because it exists.

---

# Part II — Software product and UX closure

## 6. Garrett five planes for user-facing software

For interactive software, use Jesse James Garrett's five planes as the software-profile sufficiency model.

They are sufficiency checks, not five required files.

### Strategy
Resolve:

```text
primary users/operators
real user need
product/business objective
success condition
non-goals
```

### Scope
Resolve:

```text
accepted features
information/content requirements
important states
permissions and boundaries
performance/accessibility/platform constraints
explicit exclusions
```

### Structure
Resolve:

```text
information architecture
navigation
interaction flows
scope/context inheritance
state transitions
cross-surface handoffs
loading/degraded/recovery behavior
```

### Skeleton
Resolve:

```text
screen hierarchy
primary information/action hierarchy
component anatomy
empty/loading/error/stale states
responsive interaction
keyboard/accessibility behavior
```

### Surface
Resolve enough to prevent the build agent from inventing a materially different product:

```text
visual character
typography
spacing
semantic colors
component treatment
iconography where relevant
motion
responsive visual behavior
reference screens where useful
```

For backend/library/infrastructure work, mark non-applicable UX planes explicitly rather than manufacturing UI work.

---

## 7. Software architecture closure

Beyond the five planes, resolve material software boundaries:

```text
canonical state ownership
operation/effect boundaries
API/CLI/tool parity expectations
schema/version compatibility
data migration responsibility
auth and permission semantics
browser/process/service lifecycle
failure/retry/reconciliation
external integrations
security/privacy/secrets
observability needed for actual operation
release/deployment destination
rollback/recovery where material
```

The architect does not need to choose every class, function or folder.

---

# Part III — Software Full Trajectory

## 8. Software coverage model

Derive one Full Trajectory for the accepted software outcome.

A typical path may connect:

```text
accepted product requirement
→ owning architecture/contract
→ executable software change
→ consumer-visible behavior
→ necessary verification
→ delivery/deployment
→ real consumer acceptance
```

Do not equate:

```text
source changed = feature complete
unit test passed = product accepted
build succeeded = deployed
deployed = consumer outcome verified
one working branch = whole HLT complete
```

Full / Medium / Short remain projections of the same trajectory truth.

---

## 9. Software unknowns

Do not fabricate routes, schemas, APIs, grants, framework behavior or deployment assumptions.

When a required software contract is unknown:

```text
inspect current generated/source/runtime truth
→ use existing operation if real
→ use thin adapter if semantics already exist
→ implement smallest missing operation in its owning layer if genuinely absent
→ return to the product path
```

The resolution node blocks only its dependents.

---

# Part IV — Production-shaped thin slice

## 10. First connected slice

Before broad horizontal scaffolding, establish the earliest useful end-to-end software path through real owners.

A good slice typically crosses only the layers actually required, for example:

```text
real source contract
→ real state owner
→ real API/tool operation
→ real UI/client behavior
→ real persisted/reconciled result
→ real consumer-visible acceptance
```

It is not satisfied by:

```text
mock-only backend
static UI
placeholder success
unpersisted local state
fake integration
manual state patching
client-owned substitute for missing server truth
```

Once the integration spine is real, open independent lanes aggressively.

---

## 11. Vertical slicing over horizontal ceremony

Prefer complete connected behavior over large batches of disconnected foundations.

Use horizontal foundation work only where several immediate slices genuinely share it.

Do not build speculative frameworks in anticipation of distant features.

---

# Part V — Algorithm² for software engineering

## 12. Question

For architecture, code, dependencies, tests, abstractions, docs and tooling ask:

> What real product requirement, material failure risk or reusable constraint requires this?

---

## 13. Delete

Delete or avoid:

```text
duplicate clients/stores
parallel authority
unused abstraction layers
speculative framework wrappers
generic components that erase domain meaning
per-ticket proof packets
routine approval pauses
unchanged expensive test reruns
status/report documents with no execution value
serial Workpoint order without dependency
```

---

## 14. Simplify

Prefer:

```text
existing framework conventions
existing generated contracts
one owner per state/effect
thin adapters
small local state
one connected build path
one trajectory
one build-agent handoff
```

---

## 15. Accelerate

The coding/build agent may:

```text
batch adjacent Workpoints
complete multiple STGs in one implementation pass
parallelize independent branches
reorder ready work for faster feedback
refactor opportunistically when it reduces total complexity
reuse one test/journey across many requirements
continue independent work around local blockers
```

---

## 16. Automate last

Automate only proven repetition:

```text
generated clients/contracts
repeatable release steps
reliable regression checks
repeated migration mechanics
proven environment setup
```

Do not automate process that should have been deleted.

---

## 17. Leverage² for software

When a local implementation improvement proves generally useful:

```text
prove in the real slice
→ move to lowest correct shared library/service/runtime owner
→ preserve the simple consuming interface
→ migrate current consumers when worthwhile
→ delete local duplicate workaround
```

Do not create a framework before the repeated need is real.

---

# Part VI — Build-agent autonomy

## 18. Product decisions versus engineering decisions

Architecting resolves:

```text
what product behavior is accepted
canonical ownership
user-facing information/interaction hierarchy
important state semantics
security/authority boundaries
major platform/compatibility constraints
accepted completion and deployment destination
```

The build agent freely owns ordinary reversible choices:

```text
file/component boundaries
function/class names
internal APIs
store organization
adapter implementation
local refactors
CSS/layout mechanics within UX intent
test implementation
implementation order among ready work
performance fixes
bug fixes
```

Do not ask the operator for ordinary engineering choices.

---

## 19. Default build loop

```text
current Short frontier
→ inspect real source/runtime
→ implement largest safe useful increment
→ run cheapest meaningful check
→ fix/reconcile
→ continue
```

Do not stop because:

```text
one Workpoint ended
one commit landed
one test passed
one tool failed
a report could be written
docs could be polished
```

---

# Part VII — Minimum sufficient software verification

## 20. Verification selection

Use the cheapest check that can expose a material defect in what changed.

Possible checks:

```text
focused unit test
contract/schema test
build/package validation
integration test
real browser/app journey
relevant stale/reconnect case
relevant auth/authority negative case
migration compatibility check
real deployed consumer check
```

Use only the relevant subset.

One connected browser or consumer journey can satisfy many UI requirements when it genuinely exercises them.

Run broad regression at meaningful integration/release boundaries, not reflexively after every edit.

---

## 21. Release and deployment truth

The active project profile must identify the real release path.

Generic software pattern:

```text
working source
→ changed-scope verification
→ integration/CI boundary
→ dogfood/staging where applicable
→ explicit deployment/promotion
→ verify changed consumer journey
→ rollback/recover on material regression
```

Do not silently equate merge/push/build with production delivery.

---

# Part VIII — Software handoff

## 22. Minimum build-agent handoff

A software handoff should normally provide:

```text
accepted product outcome
canonical state/effect owners
current Full Trajectory and Short frontier
relevant repo/source/runtime starting points
active UX plane artifacts where applicable
known upstream contract gaps
preserved primitives
technology/platform constraints that are truly decided
release/deployment boundary
engineering autonomy law
```

Do not require the build agent to preload the full architecture corpus.

---

## 23. Software architecting exit test

The software architecture is ready when:

```text
1. Current source and runtime reality were inspected.
2. Relevant existing specs/contracts were reconciled.
3. Canonical state/effect ownership is unambiguous.
4. Applicable Garrett planes are sufficiently resolved.
5. Material API/data/lifecycle/security/release boundaries are resolved.
6. One Full Trajectory covers the accepted product outcome.
7. Current Short frontier is executable or has a bounded resolution step.
8. A real production-shaped first slice is identifiable.
9. Algorithm² removed unnecessary product/build machinery.
10. Build agent can make ordinary engineering decisions without operator involvement.
11. Verification expectations are risk-based rather than quota-based.
12. Handoff lets implementation begin immediately from real source truth.
```

---

## 24. Reference case — Focusa Workforce Extension

The Focusa Workforce redesign is the reference proving case for this profile.

Its architecting process included:

```text
recover original Workforce vision
→ inspect current extension implementation
→ reconcile Focusa / UIAI / Veragensia / ADLBOS / Wirebot boundaries
→ correct stale upstream architecture before extending downstream design
→ run outward ecosystem-consistency pass
→ run inward semantic-integrity pass
→ resolve Strategy / Scope / Structure / Skeleton / Surface
→ derive issue-#618-style Full Trajectory
→ reduce requirements/proof/build machinery with Algorithm²
→ separate product decisions from engineering discretion
→ produce one lean build-agent handoff
```

The reusable lesson is not the browser-extension-specific document set.

It is:

> **Resolve the software product deeply enough that the coding agent does not redesign it, then delete enough process that the coding agent can actually build it.**

---

## 25. Final principle

Spec 120A supplies Focusa's cross-vertical architecture-to-execution doctrine.

Spec 120A-SWE supplies the software engineering lens.

Software engineering is Focusa's strongest first vertical, not the definition of Focusa itself.
