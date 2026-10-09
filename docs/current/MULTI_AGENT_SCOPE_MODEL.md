# Multi-Agent Scope Model

Focusa supports multiple agents by making scope explicit and refusing to merge authority across workstreams.

## Authority keys

- ScopeRef/ProjectRootKey — verified governed domain/project identity.
- WorkstreamId — durable cognitive workspace within that scope.
- ContinuityId — continuation lineage within that Workstream.
- AttachmentKey — verified binding of the current runtime to the workspace.
- `project_root` and `continuity_id` — public lookup/lineage inputs; not complete authority by themselves.
- `session_id` — temporal metadata only.
- `workpoint_id` — current continuation packet identity within a continuity.

## Rules

- Same project root does not imply same Workpoint.
- Similar trajectory/mission does not merge workstreams.
- Transcript tail is not authority after compaction or tool-output flood.
- Context Cognition, Project Card, Prediction, Metacognition, and Call Stack Design are advisory unless linked through Workpoint/Trajectory/Evidence.
- Risky mutation requires Context Authority preflight.

## Adapter behavior

Follow `AGENT_ADAPTER_CONTRACT.md` and [the shared project journey](../agent/02-focusa-cohesive-project-flow.md). Resolve installed bindings and select only capabilities needed by current verified work or recovery; initialize missing baseline/intent conditionally. Source interface mapping is not connected-adapter or behavior proof. Resume, preflight, evidence and progression retain their distinct authority. Approved active-development testing does not imply a universal Git push or a production release.
