# Focusa Metacognition Runbook

## Workflow

1. Retrieve before planning with `focusa_metacog_retrieve` or diagnose with `focusa_metacog_doctor`.
2. Capture only concise, reusable, evidence-backed lessons—not transcript blobs.
3. Reflect across a bounded turn range after an outcome exists.
4. Turn selected updates into an adjustment and evaluate observed metrics before promotion.
5. Link resulting evidence to the active Workpoint.

## Tool discovery

Use `focusa_tool_search` for `metacognition`, then `focusa_tool_describe` for exact schemas. All metacognition Pi tools are projected in `docs/contracts/spec141/generated-capability-v2/pi-tools.json` and documented under `docs/focusa-tools/tools/`.

## Recovery

- Validation rejection: inspect the live contract with `focusa_tool_doctor`; do not retry unchanged.
- Scope mismatch: verify `project_root + continuity_id` before recording learning.
- No outcome evidence: defer evaluation rather than inventing success.

## Done condition

A learning record has scoped evidence, a measurable adjustment, and an evaluated outcome; otherwise it remains a candidate.
