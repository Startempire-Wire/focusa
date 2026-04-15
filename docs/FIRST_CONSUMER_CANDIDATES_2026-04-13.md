# First Consumer Candidates — 2026-04-13

Purpose:
- ground sparse/post-substrate docs in their first real runtime consumers
- identify where no honest consumer exists yet

## Provisional first consumers

### Doc 61 — domain-general cognition primitives
Likely first consumer:
- Branch A routing substrate (`apps/pi-extension/src/turns.ts`) for current-ask / scope / relevance selection

Reason:
- downstream cognition primitives only matter if they shape routing, projection, or autonomy behavior

### Doc 71 — governing priors
Likely first consumer:
- none strong yet
- candidate future consumer: relevance ranking inside Branch A or bounded autonomy weighting in doc 78

Judgment:
- still mostly blocked until a concrete ranking/weighting consumer is chosen

### Doc 72 — identity / role fields
Likely first consumer:
- trace/routing permission semantics
- possible existing code hints in attachment role types and route validation surfaces

Judgment:
- must stay narrow and consumer-led

### Doc 73 — commitment lifecycle
Likely first consumer:
- persistent autonomy / bounded secondary cognition branch (doc 78)
- possibly task continuity / resume semantics if a truthful commitment object is defined

Judgment:
- mostly blocked until doc-78 consumer path is clearer

### Doc 74 — reference resolution
Likely first consumer:
- projection/slice/traces needing stable object references
- existing reference/meta route surfaces are a likely anchor

Judgment:
- can begin as a support layer for projections and trace review

### Doc 75 — projection / view semantics
Likely first consumer:
- bounded world / ontology projection surfaces
- later Pi bounded-view emission once Branch A matures

Existing anchor:
- `tests/ontology_world_contract_test.sh`

### Doc 76 — retention / decay
Likely first consumer:
- slice selection in Branch A / shared projection selection
- command-path `memory.decay_tick` as substrate hook

Judgment:
- can start once a slice-selection consumer is named explicitly

### Doc 66 — affordances / execution environment
Likely first consumer:
- tool/action selection and route/tool contract planning

Judgment:
- should not start as a free-floating ontology; start when one action-selection consumer is chosen

### Docs 58-65 — visual/UI track
Likely first consumer:
- none honest yet beyond sparse UI/TUI rendering and generic view code

Judgment:
- still blocked on object/evidence consumer identification
- generic TUI rendering is not enough

## Decomposition rule from this pass

If a doc has no honest first consumer yet:
- it is decomposed but still blocked
- do not inflate it into pseudo-implementation
