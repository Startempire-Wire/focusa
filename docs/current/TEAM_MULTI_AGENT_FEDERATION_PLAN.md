# Team / Multi-Agent Federation Plan

Focusa federation means multiple operators/agents can coordinate through explicit scope, Workpoints, evidence, and append-only audit records without merging authority across workstreams.

## Federation model

- Project root identifies the project authority boundary.
- Continuity id identifies the logical workstream authority boundary.
- Session id is temporal metadata only.
- Workpoint is immediate continuation authority.
- Trajectory is north-star route context.
- Context Cognition, Prediction, Metacognition, and Project Card are advisory unless linked to Workpoint/Trajectory/Evidence.

## Roles

| Role | Responsibility |
| --- | --- |
| Operator | approves goal changes, destructive actions, deployment/release boundaries |
| Primary agent | owns current Workpoint slice and captures proof |
| Reviewer agent | reviews evidence, diffs, docs, and release proof without taking write ownership |
| Background agent | works only on assigned beads/workstreams and must checkpoint before handoff |
| Adapter | passes through Focusa authority state and `tool_result_v1` envelopes |

## Handoff protocol

1. Verify project identity.
2. Resume or checkpoint Workpoint.
3. State `project_root`, `continuity_id`, `workpoint_id`, current action, and next action.
4. Link evidence refs, not raw transcript blobs.
5. Mark blockers and do-not-drift boundaries.
6. Transfer only bounded packets or docs; transcript tail is never authority.

## Conflict resolution

- Operator steering supersedes advisory predictions and projections.
- Same project root does not imply same Workpoint.
- Similar mission text does not merge continuity ids.
- Writer conflicts pause/resume through work-loop writer status/preflight.
- Cross-project state requires project identity verification before use.

## Evidence sharing

Evidence refs must name file/test/endpoint/work item plus bounded result. Public sharing uses redacted scope ids and `PUBLIC_STREAM_REDACTION_POLICY.md`.

## Non-goals

- No global memory merge across agents.
- No implicit team workspace from transcript similarity.
- No advisory packet promoted to authority by an adapter.
- No public federation surface without secret scan/redaction review.

## Proof

- Static guard: `tests/team_multi_agent_federation_static_test.sh`
- Related: `MULTI_AGENT_SCOPE_MODEL.md`, `AGENT_ADAPTER_CONTRACT.md`, `PUBLIC_STREAM_REDACTION_POLICY.md`
