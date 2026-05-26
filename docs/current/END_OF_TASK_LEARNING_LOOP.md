# End-of-task Learning Loop

Focusa treats prediction and metacognition as core session surfaces, not optional side tools.

## Required surfaces

- **Compaction cards:** include task summary, predictive context, metacog context, and possibilities.
- **Trajectory reviews:** pair goal/gap review with `focusa_predict_*` and `focusa_metacog_*` context.
- **Work reports:** before claiming task completion, include evidence, prediction evaluation, reusable lesson, and next bounded prediction.
- **Project cards / bootstrap:** project identity, ontology, trajectory, prediction, evidence, and metacog signals should propose or refresh trajectory hierarchy at bootstrap/re-bootstrap.
- **Tool cross-references:** Workpoint, Trajectory, Evidence, Prediction, Metacog, and Ontology traversal tools should route to each other where the user is closing or reviewing a task.

## End-of-task report contract

Every final task report should answer:

1. What changed?
2. What evidence proves it?
3. Which prediction was evaluated or should be recorded now?
4. Which reusable lesson was retrieved/captured?
5. What possibility remains, framed as a bounded next prediction plus trajectory gap?

## Default tool route

```text
Orient: focusa_project_identity → focusa_traverse(surface=ontology) → focusa_workpoint_resume → focusa_trajectory_view
Bootstrap/re-bootstrap: project card + prediction/metacog signals → focusa_trajectory_define_goal / focusa_trajectory_assess
Learn before acting: focusa_metacog_doctor/retrieve → focusa_predict_record
Prove: focusa_evidence_capture / focusa_workpoint_link_evidence → focusa_trajectory_assess
Close: focusa_predict_recent/stats → focusa_predict_evaluate → focusa_metacog_capture/retrieve → final work report
Continue: record next bounded prediction and next trajectory gap
```

## Guardrail

Prediction/metacog calls should be **mandatory at task boundaries** and **lightweight at compaction/trajectory review**, not noisy every turn.
