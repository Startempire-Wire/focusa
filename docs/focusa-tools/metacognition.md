# Focusa Metacognition Tools

Use metacognition tools to turn observed work into reusable, evidence-backed learning. These tools are for future behavior improvement, not raw journaling.

Each tool below includes what it does, when to use it, and one concrete example. Examples use Pi tool-call style; adapt parameter syntax to the active harness.

## `focusa_metacog_capture`

Stores a reusable learning signal with rationale/confidence/evidence refs. Quality gate improves with concrete content, rationale, confidence, and evidence.

Example:

```text
focusa_metacog_capture kind="docs_release" content="Skill docs should be family-specific and linked from README." rationale="Progressive disclosure keeps the main skill concise." confidence=0.9 strategy_class="docs_maintenance"
```

## `focusa_metacog_retrieve`

Retrieves prior learning relevant to a current ask before planning or reflection.

Example:

```text
focusa_metacog_retrieve current_ask="update public docs accurately" scope_tags=["docs_release"] k=5
```

## `focusa_metacog_reflect`

Generates hypotheses/strategy updates from recent turns.

Example:

```text
focusa_metacog_reflect turn_range="last_20" failure_classes=["docs_staleness"]
```

## `focusa_metacog_plan_adjust`

Turns a reflection into a tracked adjustment artifact.

Example:

```text
focusa_metacog_plan_adjust reflection_id="refl-123" selected_updates=["Validate public docs against runtime before release"]
```

## `focusa_metacog_evaluate_outcome`

Evaluates whether an adjustment improved results and should be promoted.

Example:

```text
focusa_metacog_evaluate_outcome adjustment_id="adj-123" observed_metrics=["docs_accuracy","release_proof"]
```

## `focusa_metacog_recent_reflections`

Lists recent reflection IDs before adjustment planning.

Example:

```text
focusa_metacog_recent_reflections limit=5
```

## `focusa_metacog_recent_adjustments`

Lists recent adjustment IDs before evaluation.

Example:

```text
focusa_metacog_recent_adjustments limit=5
```

## `focusa_metacog_loop_run`

Runs capture/retrieve/reflect/adjust/evaluate in one compressed workflow.

Example:

```text
focusa_metacog_loop_run current_ask="improve release docs workflow" turn_range="last_30" kind="workflow_signal" confidence=0.8
```

## `focusa_metacog_doctor`

Diagnoses metacog signal quality and retrieval usefulness.

Example:

```text
focusa_metacog_doctor current_ask="Spec89 release documentation" scope_tags=["release_validation"] k=3
```
