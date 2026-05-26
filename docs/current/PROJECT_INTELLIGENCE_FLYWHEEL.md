# Project Intelligence Flywheel

Focusa should compound project intelligence over time, not just remember session notes.

## Core loop

```text
ontology objects → trajectory hierarchy → prediction → action/evidence → evaluation → metacog condensation → project card refresh → better trajectory/next step
```

## Ontology role

The ontology is the typed backbone. Prediction and metacognition should attach to real project objects instead of floating prose:

- `ProjectIdentity` / project card
- `TrajectoryProjection` / hierarchy nodes
- `WorkpointRecord`
- `EvidenceRef`
- `PredictionRecord`
- `MetacognitionSignal`
- `Risk` / `AcceptanceCriterion` / `NotDoneIf`

This lets lessons answer: **which project object did this improve, falsify, risk, or complete?**

## Bootstrap and re-bootstrap

At session start, project resume, low-confidence trajectory, or stale goal detection, Focusa should build a project card from `GET /v1/project/card` / `focusa project card`, which fuses:

1. `focusa_project_identity`
2. `focusa_traverse(surface=ontology)` bounded object context
3. `focusa_trajectory_view`
4. recent prediction stats/outcomes
5. relevant metacog lessons
6. evidence/workpoint state

If no current trajectory exists, the bootstrap prompt should offer a **learning-informed trajectory** as the first draft. If a trajectory exists but conflicts with evidence/predictions/lessons, re-bootstrap should propose a refreshed hierarchy rather than silently continue stale goals. The card is advisory-only; `focusa_trajectory_define_goal` remains the explicit write path.

## Project card fields

A project card should summarize:

- identity: name, root, repo, environment, deployment URLs
- trajectory: high-level goal, desired end state, current state, active gap, waypoints
- ontology: key objects and relationships
- evidence: latest proof handles and missing acceptance checks
- prediction: open predictions, evaluated accuracy, likely next risk/success
- metacognition: top reusable lessons and anti-patterns
- possibilities: bounded next options ranked by evidence and expected value

## Prior-informed bootstrap

`GET /v1/project/card` includes `prior_session_context`, so bootstrap candidates can be informed by previous sessions instead of starting blank. The packet carries the trajectory ladder (HLG/MLG/STG/waypoints), recent Focus decisions, recent frame goals/results, prediction summary, recent algorithm-run outcomes, and a metacog retrieval prompt.

## Trajectory Success Sequence

`GET /v1/project/card` includes `success_sequence`, an advisory ordered event plan derived from the high-level trajectory goal, active gap, ontology context, prediction probabilities, metacog prompts, and expected utility.

The sequence stages are:

1. orient project card,
2. refresh/confirm trajectory,
3. retrieve lessons,
4. forecast next action,
5. execute highest expected-value slice,
6. prove outcome,
7. evaluate and compound.

This lets a model ask: **what event should happen next for the most productive/profitable/successful path?** The sequence is outcome-aware: recent `project_card_algorithm_outcomes.jsonl` scores bias readiness, refresh, learning, and expected utility. It also exposes `shortest_path_to_success`, a weighted path-elimination view over execute/refresh/learn routes with selected path, rejected paths, and elimination reasons. Hot-path reads use bounded JSONL tails and project weights are projected read-only on card GET; persisted learning happens on explicit outcomes. The answer stays advisory; operator steering and explicit trajectory writes remain authoritative.

Bootstrap quality rule: when prior data exists, define or refresh HLG/MLG/STG from `prior_session_context` plus explicit operator steering; use predictions for risk/expected-value, metacog for reusable lessons, and decisions for durable architectural boundaries.

## TrajectoryReporting elapsed + token card

`GET /v1/project/card` includes `trajectory_report_card`, a reporting card for operator-visible progress. It summarizes HLT/LTG/MTG/STG hierarchy, waypoint accomplishment status from recent outcomes, and `time_and_tokens` from project-card outcomes. End-of-task reports should include elapsed wall-clock time (`HH:MM:SS`), total tokens, trajectory hierarchy, and waypoint accomplishments so similar task completion time/token cost can improve.

## Next-step quality rule

A next step is strong when it is:

- tied to a trajectory gap,
- grounded in ontology object refs,
- predicted with confidence and rationale,
- testable with evidence,
- able to produce or refine a reusable lesson.

## Compounding rule

Only promote learning when an outcome changes future behavior. Condense repeated captures into stronger, shorter lessons; evaluate predictions; retire weak or stale signals; feed promoted lessons back into bootstrap/re-bootstrap trajectory drafts.
