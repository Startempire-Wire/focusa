# Implementation Status Matrix — Docs 51-78

Status legend:
- **Implemented** — runtime largely matches doc intent
- **Partial** — meaningful implementation exists but material gaps/drift remain
- **Drifted** — code exists but materially contradicts the newer doc intent
- **Docs-only** — mostly normative/spec-level, little faithful implementation
- **Blocked** — downstream doc should not be fully implemented before prerequisites land

This matrix is a decomposition instrument, not a closure claim.

| Doc | Theme | Status | Why | Prereqs / Blocking Relationship |
|---|---|---|---|---|
| 51 | Ontology expression and proxy integration | Drifted | Pi hot path historically used broad focus-state injection instead of operator-first minimal slice assembly. | Prereq for 52, 53, 54b, 57, 67-69 |
| 52 | Pi extension contract | Implemented | Pi-Focusa bridge now persists resumable WBM state, restores session metadata, and passes strict contract gating for bounded input/output surfaces. | Depends on 51; blocks downstream Pi alignment work |
| 53 | Pi behavioral alignment contract | Implemented | Operator-first minimal-slice routing, steering traces, subject-hijack prevention markers, and prompt-shaping checks are now systematically gated. | Depends on 51/52/54a/54b/57 |
| 54a | Operator priority and subject preservation | Implemented | Mission carryover is now gated by explicit continuation/relevance and the Pi hot path emits subject-hijack prevention traces. | Depends on 51/52; blocks 54b/67-69/78 |
| 54b | Context injection and attention routing | Implemented | Pi hot path builds a minimal applicable slice, suppresses irrelevant context, and records explicit exclusion reasons and routing traces. | Depends on 51/54a/67/68 |
| 54 | Pi visible output boundary | Partial | Some visible-output discipline exists, but not comprehensively verified against newer subject-preservation rules. | Depends on 52/53/54a |
| 55 | Tool and action contracts | Implemented | PushDelta/write-path semantics, failure taxonomy, status-envelope handling, and operator-critical fallback mirroring are now strictly gated. | Depends on 52/70 |
| 55 impl | Tool/action contracts impl | Implemented | The implementation mapping now matches the bridged write-path behavior closely enough to act as a closure-proof surface. | Depends on 55 |
| 56 | Trace, checkpoints, recovery | Partial | Checkpoints and trace surfaces exist; many required dimensions only recently began to be emitted. | Depends on 51/52; blocks 57/69/78 |
| 57 | Golden tasks and evals | Implemented | Golden/eval scripts are part of the spec-gates path and now serve as live proof surfaces for behavioral, routing, and contract regressions. | Depends on 51-56 |
| 58 | Visual/UI ontology core | Docs-only | Limited code correspondence in current runtime. | Blocks 59-65 |
| 59 | Visual/UI reverse engineering | Docs-only | Little faithful runtime layer visible. | Depends on 58 |
| 60 | Visual/UI verification and critique | Partial / Docs-only | Some verification language and worker/eval concepts exist, but no strong end-to-end visual critique substrate. | Depends on 58/59 |
| 62 | Visual/UI evidence and workflow | Docs-only / Partial | Evidence-first language influences later docs, but implementation remains sparse. | Depends on 58-60 |
| 63 | Visual/UI invention and variation | Docs-only | Minimal corresponding implementation. | Depends on 58-62 |
| 64 | Visual/UI to implementation | Docs-only | Little evidence of systematic handoff machinery. | Depends on 58-63 |
| 65 | Visual/UI Focusa integration | Docs-only | Minimal integration surface visible. | Depends on 58-64 |
| 61 | Domain-general cognition core | Partial / Normative substrate | Heavily influences newer docs, but not fully realized as a cohesive runtime ontology layer. | Supports 67-78 |
| 66 | Affordance and execution environment ontology | Docs-only | Very limited code-level substrate identified so far. | Supports 55/64/67 |
| 67 | Query scope and relevance control | Implemented | Current-ask classification, continuation/relevance gating, exclusion reasons, and routing regression evals now provide explicit scope control. | Depends on 51/54a/54b; blocks 68/69/78 |
| 68 | Current ask and scope integration | Implemented | CurrentAsk and QueryScope now govern Pi hot-path slice construction, mission reuse, telemetry, and exclusion tracking. | Depends on 67; blocks 69 |
| 69 | Scope failure and relevance tracing | Early Partial | Initial objective traces exist; semantic scope-failure events remain largely unimplemented. | Depends on 67/68/56 |
| 70 | Shared interfaces, statuses, lifecycle | Docs-only / Partial vocabulary | Terms like Verifiable/Scoped exist in docs and some naming, but not a comprehensive enforced substrate. | Prereq for 71-77 |
| 71 | Governing priors and scalar weights | Docs-only | Minimal concrete implementation surfaced so far. | Depends on 70 |
| 72 | Agent identity, role, self-model ontology | Docs-only / Partial vocabulary | Some identity/role language exists; not a strong implemented ontology layer. | Depends on 70 |
| 73 | Intention, commitment, self-regulation | Partial / Docs-only | Reflection/autonomy loops exist, but not full newer-doc discipline. | Depends on 70/72 |
| 74 | Identity and reference resolution | Docs-only / Partial | Some handle/id systems exist; not a clearly implemented ontology domain matching the doc. | Depends on 70 |
| 75 | Projection and view semantics | Docs-only / Partial influence | Projection vocabulary influences newer specs, but runtime still often uses direct broad state views. | Depends on 70/74 |
| 76 | Retention, forgetting, decay policy | Partial / Docs-only | Some memory confidence/decay logic exists, but not full cross-domain retention discipline. | Depends on 70/75 |
| 77 | Ontology governance, versioning, migration | Partial / Docs-only | Proposal governance/tests exist, but shared-layer change governance remains incomplete. | Depends on 70-76 |
| 78 | Bounded secondary cognition and persistent autonomy | Partial spec / implementation incomplete | Spec hardened; implementation spread across open beads and still blocked on earlier substrate docs. | Depends on 51,54a,54b,56,57,67-77 |

---

## First implementation frontier (provisional)

These docs appear to be the earliest post-cutoff docs that should drive actual implementation next:

1. **51** — ontology expression and proxy integration
2. **54a** — operator priority and subject preservation
3. **54b** — context injection and attention routing
4. **67** — query scope and relevance control
5. **68** — current ask and scope integration
6. **69** — scope failure and relevance tracing
7. **56** — trace/checkpoints/recovery (as required observability substrate)
8. **70** — shared interfaces/statuses/lifecycle (as shared substrate for truthful object semantics)

These are the likely first frontier because they define:
- what should be injected
- how operator input outranks carryover
- how scope is determined
- how context is selected/excluded
- how failures are traced
- how objects/fields are named and verified

Without these, many downstream docs remain blocked or too speculative to implement honestly.

---

## Docs likely blocked until first frontier advances

### Mostly blocked by first frontier
- 53
- 55 / 55 impl
- 57
- 71-77
- 78 remainder

### Visual/UI track mostly separate and currently lower-confidence for immediate implementation
- 58-65

These likely need decomposition too, but should not distract from the core current-ask / routing / trace / substrate frontier.
