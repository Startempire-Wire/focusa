# Agent Awareness Quickstart

Focusa is an agent utility layer: working memory, continuation contracts, evidence links, prediction records, recovery guidance, and governance for long-running AI sessions.

## What agents must know first

1. **Focusa is not chat memory.** It stores bounded state, Workpoints, evidence refs, predictions, lineage, and recovery hints.
2. **Workpoint beats transcript tail.** After compaction/reload/model switch/fork, call `focusa_workpoint_resume` and follow the canonical packet unless the operator steers otherwise.
3. **Checkpoint before risky boundaries.** Before compaction, model switch, fork, context overflow, or risky continuation, call `focusa_workpoint_checkpoint`.
4. **Doctor first when uncertain.** If Focusa seems stale/offline/blocked/degraded, call `focusa_tool_doctor` before guessing.
5. **Evidence is first-class.** After tests, release proof, API proof, or file proof, call `focusa_evidence_capture` or `focusa_workpoint_link_evidence`.
6. **Predictions are measurable.** Before risky or uncertain next action, call `focusa_predict_record`; after outcome, call `focusa_predict_evaluate`.
7. **Compaction must be useful.** Sparse Focusa slots should use related Workpoint/current-ask/frame/local-shadow/session fallbacks, never random filler or bare `none`.

## Minimal runtime loop

```text
Start/reload:      focusa_tool_doctor if uncertain; focusa_workpoint_resume if resuming.
Before boundary:  focusa_workpoint_checkpoint.
During work:      link evidence; maintain Focus State only for durable decisions/constraints/failures/results.
Before risk:      focusa_predict_record.
After outcome:    focusa_predict_evaluate and evidence link.
After compaction: focusa_workpoint_resume; continue from canonical packet.
```

## Operator steering

Operator steering always wins. Focusa guides, preserves, and audits; it does not overrule a fresh operator instruction.
