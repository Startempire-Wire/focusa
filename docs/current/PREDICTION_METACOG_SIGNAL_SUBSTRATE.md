# Prediction + Metacognition Signal Substrate

Purpose: define the minimum useful substrate for Focusa predictive power and learning compounding using **existing Focusa technology**.

This is not a global data-platform spec. It does not require atomic clocks, satellite feeds, market data, FastAPI, Redis, BigQuery, or a separate ingestion stack. Those may be future external integrations, but they are unnecessary for general Focusa predictive power.

## Core idea

Focusa predictive power should come from a tight flywheel:

```text
Focusa signals → prediction → action → evidence/outcome → metacognition → promoted learning → next prediction
```

The model should use:

- `focusa_metacog_retrieve` before similar/risky/high-leverage work.
- `focusa_predict_record` before choosing a next action or tool route.
- `focusa_predict_evaluate` or `focusa predict capture-outcome` after proof.
- `focusa_metacog_capture` / `evaluate_outcome` when the outcome teaches a reusable lesson.
- Ontology context on prediction and learning records so lessons attach to objects/actions/tools/evidence, not vague prose.

## Existing Focusa layers used

| Existing layer | Role in flywheel |
| --- | --- |
| Project Identity | keeps predictions and lessons project-scoped |
| Trajectory | groups prediction accuracy by HLT/MLG/STG/Waypoint |
| Workpoint | anchors the immediate action and next continuation |
| Evidence refs | provide outcome/proof handles for evaluation |
| Ontology context | binds learning to `object_refs`, `action_refs`, `tool_refs`, `evidence_refs`, `relation_refs` |
| Tool result envelopes | provide failure class, retry posture, next tools, degraded/canonical status |
| Reflex primitives | identify repeatable recovery moves |
| ResourceMode | tells the model when to choose bounded/low-memory paths |
| Metacognition | stores reusable lessons and promoted adjustments |
| Predictions | records expected outcomes and evaluates calibration |

## Canonical bounded signal

A signal should be a compact Focusa-local observation, not raw logs.

```json
{
  "signal_type": "tool_outcome",
  "summary": "focusa_workpoint_resume returned canonical=true",
  "observed_at": "2026-05-26T00:00:00Z",
  "confidence": 1.0,
  "evidence_refs": ["focusa_workpoint_resume:019e..."],
  "ontology_context": {
    "object_refs": ["Workpoint:019e..."],
    "action_refs": ["resume_workpoint"],
    "tool_refs": ["focusa_workpoint_resume"],
    "evidence_refs": ["focusa_workpoint_resume:019e..."],
    "relation_refs": ["tool_result_supports_next_action"]
  }
}
```

For coding-agent workflows, useful signals are already available from current Focusa and repo context:

| Signal | Source today | Predictive use |
| --- | --- | --- |
| canonical Workpoint available | `focusa_workpoint_resume` | next action likely safe to continue |
| stale/missing Workpoint | Workpoint result envelope | predict drift risk / checkpoint need |
| trajectory gap | `focusa_trajectory_view` / assess | predict useful next Workpoint |
| evidence present/missing | evidence refs / Workpoint verifications | predict completion confidence |
| tool failure class | tool-result envelope | predict best recovery route |
| resource pressure | `focusa_resource_mode`, telemetry | predict timeout/context risk |
| repeated failure | failure records / metacog / tool envelopes | predict stuck loop |
| prior lesson match | `focusa_metacog_retrieve` | predict safer strategy |
| prior prediction score | `focusa_predict_stats` / recent | calibrate confidence |

## Flywheel contract

### 1. Before action: retrieve + predict

The model should ask:

- What similar lessons exist?
- What is likely to happen if I take this next action?
- What tool route is most useful now?
- What evidence would prove success/failure?

Tool route:

```text
focusa_metacog_retrieve → focusa_predict_record → action/tool route
```

### 2. During action: preserve ontology/evidence refs

Prediction records should include bounded ontology context:

```json
{
  "object_refs": ["Workpoint:active", "File:crates/focusa-api/src/routes/predictions.rs"],
  "action_refs": ["patch_prediction_flywheel"],
  "tool_refs": ["focusa_predict_record", "focusa_metacog_retrieve"],
  "evidence_refs": ["cargo check -p focusa-api -p focusa-cli"],
  "relation_refs": ["prediction_guides_action", "evidence_evaluates_prediction"]
}
```

### 3. After action: evaluate outcome

Tool route:

```text
focusa_predict_evaluate OR focusa predict capture-outcome
```

Successful prediction outcomes are automatically captured into metacognition memory with `strategy_class=prediction_metacog_flywheel`.

### 4. After meaningful learning: promote + follow up

Tool route:

```text
focusa_metacog_reflect → focusa_metacog_plan_adjust → focusa_metacog_evaluate_outcome
```

Successful metacog evaluations promote a learning capture and create a follow-up prediction of type `metacog_learning_transfer`.

### 5. Next turn: re-enter context

The next model turn should see or retrieve:

- relevant promoted lessons
- recent prediction accuracy
- active ontology refs
- the next suggested prediction/evaluation action

## What not to include by default

Exclude from the general Focusa predictive-power layer:

- external global sensors
- atomic-clock/GPS timing infrastructure
- market feeds
- GDELT/Google Trends/ACLED/AIS/ADS-B pipelines
- new FastAPI/Redis/MySQL/S3 architecture
- large analytics warehouses
- broad sentiment platform design

Those are external application domains. Focusa only needs a connector boundary later: external systems can submit bounded observations with ontology context.

## Minimum implementation target

To make prediction/metacognition feel like a rating-9 system for agents:

1. Predictions always accept trajectory + ontology context.
2. Prediction outcomes can be captured automatically after proof.
3. Successful prediction evaluations feed metacognition captures.
4. Successful metacog evaluations create follow-up predictions.
5. Agent quickstart tells models to use retrieve → predict → act → evaluate → learn regularly.
6. Tool choreography routes evidence and tool outcomes into this loop.
7. Focus Slice eventually exposes a compact `PREDICTIVE_CONTEXT` card.

## Compact `PREDICTIVE_CONTEXT` target

```text
PREDICTIVE_CONTEXT
- relevant_lessons: 2 promoted metacog captures
- recent_accuracy: next_action_success 0.82, tool_choice 0.76
- active_ontology_refs: Workpoint:active, Tool:focusa_workpoint_resume
- likely_next_risk: stale_state if project_root/continuity not verified
- recommended_prediction: record next_action_success before patching
- outcome_capture: evaluate after cargo check / tests / evidence link
```

Design rule: predictive power is useful only when it improves action selection, confidence calibration, recovery, or reusable learning. Everything else is context bloat.
