# Focusa Glossary-Linked Docs UI

The docs UI should teach Focusa's vocabulary without flattening it. Terms remain canonical, disciplined, and linked to operational docs.

## Canonical term index

| Term | Meaning | Primary doc |
| --- | --- | --- |
| Focus State | operator-facing current cognitive state slots | `AUTHORITY_MODEL.md` |
| Trajectory | HLT/MLG/STG/waypoint north-star route context | `TRAJECTORY_GTM_AND_GAPS.md` |
| Workpoint | immediate continuation authority packet | `WORKPOINT_LIFECYCLE_GUIDE.md` |
| Evidence | bounded proof handle linked to Workpoints | `GOLDEN_WORKFLOW_PUBLIC_DEMO.md` |
| Context Authority | risky mutation/classification gate | `CONTEXT_AUTHORITY_CURRENT.md` |
| Context Cognition | advisory structured context packet | `GOLDEN_WORKFLOW.md` |
| Project Identity | project scope verification boundary | `AUTHORITY_MODEL.md` |
| Continuity ID | logical workstream authority key | `MULTI_AGENT_SCOPE_MODEL.md` |
| Session ID | temporal metadata only | `MULTI_AGENT_SCOPE_MODEL.md` |
| Call Stack Design | typed entry-to-output implementation plan | `CALL_STACK_DESIGN_CURRENT.md` |
| Public Card | redacted public stream summary | `PUBLIC_STREAM_REDACTION_POLICY.md` |
| Tool Result Envelope | `tool_result_v1` status/failure/retry/evidence wrapper | `TOOL_RESULT_ENVELOPE_V1.md` |

## UI expectations

- Show glossary hover/card for canonical terms.
- Link each term to one primary doc and optional related docs.
- Preserve exact term spelling in headings and cards.
- Mark advisory vs authority terms visually.
- Include search aliases but never replace canonical names.
- Keep public docs redaction-safe; no raw private paths/tokens/logs.

## Suggested navigation groups

- Start: `FIRST_RUN_FLOW.md`, `GOLDEN_WORKFLOW_PUBLIC_DEMO.md`
- Authority: `AUTHORITY_MODEL.md`, `CONTEXT_AUTHORITY_CURRENT.md`, `MULTI_AGENT_SCOPE_MODEL.md`
- Continuation: `WORKPOINT_LIFECYCLE_GUIDE.md`, `TRAJECTORY_GTM_AND_GAPS.md`
- Proof: `PUBLIC_PROOF_BUNDLE_VIEWER.md`, `VALIDATION_AND_RELEASE_PROOF.md`
- Adapters: `AGENT_ADAPTER_CONTRACT.md`, `NON_PI_AGENT_ADAPTER_EXAMPLES.md`

## Proof

- Static guard: `tests/glossary_linked_docs_ui_static_test.sh`
- Spec source: `docs/106-focusa-vision-tightening-spec.md`
