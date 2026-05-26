# AI AgentOps Session SOP

1. Start with project identity.
2. Define or resume the Workpoint.
3. Confirm Trajectory orientation; treat it as advisory.
4. Execute one bounded step.
5. Attach evidence.
6. Run drift check.
7. Close with handoff/resume packet.

## Minimal command flow

```bash
focusa onboard --agent manual
focusa workpoint resume --mode compact_prompt
focusa workpoint evidence-link --target-ref "<ref>" --result "<summary>" --evidence-ref "<proof>"
focusa workpoint drift-check --latest-action "<what changed>"
focusa awareness card --adapter-id manual --workspace-id local --agent-id cli
```
