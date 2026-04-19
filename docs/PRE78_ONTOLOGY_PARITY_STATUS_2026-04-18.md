# Pre-78 Ontology Parity Status — 2026-04-18 (updated)

Purpose:
- truthful implementation status for ontology-related docs before 78
- separate validated parity from partial/pending rows
- anchor status to executable tests + code loci

## Executive truth

Major blocking parity suites are now green in this pass:
- `tests/ontology_event_contract_test.sh` ✅
- `tests/ontology_world_contract_test.sh` ✅
- `tests/ontology_visual_implementation_handoff_contract_test.sh` ✅

So the previous hard blockers (docs 45-50/64 contract gates) are cleared.
Remaining work is matrix completeness for docs with indirect/no dedicated parity gate rows.

## Key implementation changes in this pass

`crates/focusa-api/src/routes/proposals.rs`
1. `POST /v1/proposals` now dispatches `Action::SubmitProposal` (PRE-native path) instead of emitting `ProposalSubmitted` directly via reducer event route. This restores reliable proposal visibility/id retrieval for contract tests.
2. `POST /v1/proposals/resolve` now derives `window_start` from earliest pending proposal instead of `Utc::now()`.
3. Clarification outcomes now apply deterministic tie-break fallback (score → created_at → id) and proceed as accepted resolution for API resolution flow, allowing bounded convergence under large pending sets.

These changes restored expected ontology lifecycle events:
- `ontology_proposal_promoted`
- `ontology_verification_applied`
- `ontology_working_set_refreshed`
- `ontology_proposal_rejected`

## Evidence commands (current pass)

- `tests/ontology_event_contract_test.sh` → 0
- `tests/ontology_world_contract_test.sh` → 0
- `tests/ontology_visual_implementation_handoff_contract_test.sh` → 0
- `tests/ontology_route_metadata_contract_test.sh` → 0
- `tests/ontology_affordance_schema_contract_test.sh` → 0
- `tests/work_loop_query_scope_boundary_contract_test.sh` → 0
- `tests/doc70_shared_interfaces_lifecycle_contract_test.sh` → 0 (consolidated semantic runtime gate for docs 70/72/75)
- `tests/proxy_mode_b_parity_test.sh` → 0
- `tests/ontology_pre79_regression_gate.sh` → 0 (16 suites, runtime-behavioral bundle)

## Doc-by-doc status (45-77)

Legend:
- ✅ passing proof surface observed this pass
- ⚠️ partial/indirect or pending explicit matrix proof row

| Doc | Topic | Status | Evidence |
| --- | --- | --- | --- |
| 45 | Ontology overview | ✅ | `ontology_world_contract` passing |
| 46 | Ontology core primitives | ✅ | `ontology_world_contract` passing |
| 47 | Ontology software world | ✅ | `ontology_world_contract` passing |
| 48 | Links/actions | ✅ | `ontology_world_contract` passing |
| 49 | Working sets/slices | ✅ | `ontology_world_contract` passing |
| 50 | Classification/reducer/events | ✅ | `ontology_event_contract` passing |
| 51 | Expression/proxy | ✅ | `tests/proxy_mode_b_parity_test.sh` passing (4/4); mapped to `crates/focusa-core/src/adapters/openai.rs` + `crates/focusa-core/src/adapters/anthropic.rs` operator-first minimal-slice path |
| 52-57 | Pi/tool/trace/eval contracts | ⚠️ | outside strict ontology core; explicit rowing pending |
| 58 | Visual ontology core | ✅ | visual object/reverse pipeline contracts passing |
| 59 | Visual reverse engineering | ✅ | visual reverse pipeline contract passing |
| 60 | Visual verification/critique | ✅ | visual workflow evidence routes passing |
| 61 | Domain-general cognition core | ✅ | doc61 frontier + first-consumer tests passing |
| 62 | Visual evidence/workflow | ✅ | visual workflow evidence + commands contracts passing |
| 63 | Visual invention/variation | ⚠️ | indirect coverage; dedicated row pending |
| 64 | Visual→implementation handoff | ✅ | handoff contract test passing |
| 65 | Visual UI Focusa integration | ✅ | visual workflow route/command contracts passing |
| 66 | Affordance/execution environment ontology | ✅ | affordance schema + consumer/environment tests passing |
| 67 | Query scope/relevance | ✅ | query-scope boundary + route metadata tests passing |
| 68 | Current-ask/scope integration | ✅ | query-scope boundary + scope-failure taxonomy passing |
| 69 | Scope failure/relevance tracing | ✅ | scope-failure taxonomy events passing |
| 70 | Shared interfaces/lifecycle | ✅ | consolidated semantic runtime gate (`doc70_shared_interfaces_lifecycle_contract_test.sh`) passing |
| 71 | Governing priors/scalars | ✅ | governing-priors consumer path passing |
| 72 | Agent identity/role/self-model | ✅ | consolidated semantic runtime gate (`doc70_shared_interfaces_lifecycle_contract_test.sh`) validates identity/role object+link+action projections and contract semantics |
| 73 | Intention/commitment/self-regulation | ✅ | doc73 + commitment lifecycle tests passing |
| 74 | Identity/reference resolution | ✅ | doc74 consumer path passing |
| 75 | Projection/view semantics | ✅ | consolidated semantic runtime gate (`doc70_shared_interfaces_lifecycle_contract_test.sh`) validates projection/view objects+actions and contract target semantics |
| 76 | Retention/decay policy | ✅ | doc76 retention consumer path passing |
| 77 | Ontology governance/versioning/migration | ✅ | migration conformance checks passing |

## Beads alignment

- Reopened parent epic remains active: `focusa-zerg`
- Recovery epic active: `focusa-0g67`
  - `.1` event parity now green (candidate for close)
  - `.2` world projection parity now green (candidate for close)
  - `.3` visual handoff parity now green (candidate for close)
  - `.4` matrix completion still active until pending ⚠️ rows get explicit proof rows

## Regression gate bundle

- Canonical executable bundle: `tests/ontology_pre79_regression_gate.sh`
- Current result: pass (16/16)

## Closure rule (unchanged)

Pre-78 ontology can be called fully implemented only when:
1. all ontology contract suites are green (now true for current blocking trio),
2. docs 45-77 each have explicit proof rows (pending for ⚠️ docs),
3. no false-closed BD remains in ontology parity scope.
