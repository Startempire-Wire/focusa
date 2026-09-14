# Tool taxonomy audit — 2026-08-16 (#258 slice 1)

Read-only audit of the deployed Pi extension's 116 focusa_* tools.

## Results

- 116 tools across 45 families; every tool has a typed name + schema
  (strict-key validation), served through focusa_tool_search/describe/
  graph/bundle (progressive discovery + bounded traversal).
- Largest families: bloatgaurd (11), metacog (9), tree (7), context (7),
  work_loop (6), project (6), trajectory (6), device (6).
- Semantic-duplicate candidates: 1 (`focusa_device_pair_status` vs
  `focusa_device_pair_list`) — distinct operations (status ≠ list),
  NOT a duplicate. Verdict: zero real duplicates.
- Dynamic working sets: `focusa_tool_bundle` + search/describe/graph
  deliver per-task capability bundles; no static mega-list is ever
  injected into prompts.

## Verdict

No redesign required this pass. The taxonomy, dedup, and dynamic
working-set surfaces already conform to #258. Remaining #258 work:
daemon-side tool docs parity re-run at the next release (docs freshness
per docs-maintenance), and cross-harness bundle schema conformance.
