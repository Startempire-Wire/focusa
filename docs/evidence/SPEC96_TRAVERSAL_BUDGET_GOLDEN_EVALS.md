# Spec96 Traversal Budget Golden Evals

`tests/spec96_traversal_budget_golden_eval_test.sh` verifies that surgical traversal covers partial lineage, ontology, evidence/ECS, metacognition, snapshots, and trajectory retrieval under low-memory style constraints.

The eval compares current bounded surfaces against an old full-dump baseline and checks:

- default/requested limits and hard caps;
- cursor/next_cursor windows;
- field projection;
- cold full-payload opt-in guards;
- timeout taxonomy;
- safe-audit failures for missing surgical surfaces or budget controls;
- example model tool calls that request only the relevant slice.
