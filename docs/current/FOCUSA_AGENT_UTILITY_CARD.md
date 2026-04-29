# Focusa Agent Utility Card

This card is injected by the Pi extension at startup/reload and included in the system prompt so agents are aware of Focusa as a utility without reading the repository first.

## Runtime card content requirements

The card must mention:

- Focusa availability/degraded status.
- Current mission or latest operator/current repo fallback.
- Current next anchor or `focusa_workpoint_resume` fallback.
- Current project scope or project-root binding rule.
- `focusa_tool_doctor` as the first recovery tool when uncertain.
- `focusa_workpoint_checkpoint` before compaction/model switch/fork/risky continuation.
- `focusa_workpoint_resume` after compaction/reload/resume.
- Workpoint scope and continuity rules.
- evidence capture/linking after proof.
- prediction record/evaluate around risky or uncertain actions.
- Metacognition/work-loop tool families for learning/continuous work.
- Compaction fallback rule: related canonical fallback, not blank `none` fields.
- Operator steering wins.

## Source

Runtime implementation:

```text
apps/pi-extension/src/awareness.ts
apps/pi-extension/src/turns.ts
```

Validation:

```bash
node scripts/validate-agent-awareness.mjs
```
