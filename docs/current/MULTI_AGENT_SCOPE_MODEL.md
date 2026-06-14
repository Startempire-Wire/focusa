# Multi-Agent Scope Model

Focusa supports multiple agents by making scope explicit and refusing to merge authority across workstreams.

## Authority keys

- `project_root` — project folder authority boundary.
- `continuity_id` — logical workstream authority boundary.
- `session_id` — temporal metadata only.
- `workpoint_id` — current continuation packet identity within a continuity.

## Rules

- Same project root does not imply same Workpoint.
- Similar trajectory/mission does not merge workstreams.
- Transcript tail is not authority after compaction or tool-output flood.
- Context Cognition, Project Card, Prediction, Metacognition, and Call Stack Design are advisory unless linked through Workpoint/Trajectory/Evidence.
- Risky mutation requires Context Authority preflight.

## Adapter behavior

All adapters follow `AGENT_ADAPTER_CONTRACT.md`: read awareness card, verify project identity, resume/checkpoint Workpoint, capture/link evidence, run Context Authority preflight, render Context Cognition, surface `tool_result_v1`, and respect canonical/advisory/degraded state.
