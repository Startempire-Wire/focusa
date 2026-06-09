# focusa_browser_diagnostics_intake

**Family:** `workpoint`  
**Label:** Browser Diagnostics Intake

## Purpose

Composite Pi tool for UIAI/browser debugging evidence.

## Expected result

`tool_result_v1` with `ok`, status, `target_ref`, `failure_class` (when present), bounded `evidence_ref`, Workpoint linkage flag, inferred `focusa_scope`, scoped `workpoint_id` / `continuity_id`, active-object hints, optional prediction candidate, and optional metacog lesson. The tool links the diagnostics to the active Workpoint when scope is verified, never overrides Workpoint identity, and preserves the original diagnostics artifact handle.

Use after a UIAI `browser_diagnostics` JSON artifact or typed browser action failure envelope is available. The tool summarizes diagnostics, links bounded evidence to the active Workpoint when safe, emits active-object hints, records a prediction candidate by default, and can optionally capture a reusable metacog lesson. If the UIAI diagnostics include `focusa_scope`, the tool uses it as the default Workpoint/project scope.

## When to use

- Browser page is blank, broken, visually wrong, or unexpectedly navigated.
- UIAI action failed: click, wait, eval/eval_async, selector lookup, navigation, screenshot, or snapshot.
- Console errors, JS exceptions, failed requests, CORS/API failures, or flake clues are present.
- A diagnostics JSON file exists and should become Workpoint evidence.
- A UIAI session was opened with `focusa_scope` and diagnostics should link without manually retyping Workpoint IDs.

## Inputs

- `diagnostics` — diagnostics JSON object or UIAI/browser action failure envelope.
- `diagnostics_ref` — stable artifact/file path/URL for the diagnostics JSON.
- `target_ref` — optional page, endpoint, selector, or component under diagnosis.
- `workpoint_id` — optional explicit Workpoint; omit to use active Workpoint or `diagnostics.focusa_scope.workpoint_id`.
- `project_root` — explicit project root for canonical evidence linkage; defaults to `diagnostics.focusa_scope.project_root` when present.
- `session_id` / `continuity_id` — optional identity axes; continuity is part of the authority boundary and defaults to `diagnostics.focusa_scope.continuity_id` when present.
- `attach_to_workpoint` — defaults true; false performs dry intake.
- `create_prediction` — defaults true.
- `create_metacog` — defaults false; enable only for evidence-backed reusable lessons.

## Examples

### Intake from diagnostics artifact

```json
{
  "diagnostics_ref": "/tmp/uiai-browser-diagnostics.json",
  "target_ref": "browser:https://example.test/app",
  "project_root": "/path/to/project"
}
```

### Intake from failure envelope without linking

```json
{
  "diagnostics": {"error_class":"selector_not_found","diagnostics_summary":{"console_errors":0,"failed_requests":1}},
  "target_ref": "selector:@e7",
  "project_root": "/path/to/project",
  "attach_to_workpoint": false
}
```

### Capture a reusable lesson

```json
{
  "diagnostics_ref": "/tmp/uiai-soak-flake.json",
  "target_ref": "browser-soak:checkout-flow",
  "project_root": "/path/to/project",
  "create_prediction": true,
  "create_metacog": true
}
```

## Effects

When attached, the tool links a bounded evidence ref to the active Workpoint. Embedded `focusa_scope` fields are used unless explicit parameters override them. It may also:

- Return active-object hints from URL, selector, endpoint, exception source, and failed request paths.
- Record a bounded prediction candidate for likely cause or next verification.
- Capture a metacog candidate only when requested.
- Preserve the diagnostics artifact handle instead of raw browser logs.
- Echo inferred `focusa_scope`, scoped Workpoint ID, and scoped continuity ID in details for auditability.

## Recovery notes

- `attach_to_workpoint=false` is useful when scope is uncertain or the artifact is only exploratory.
- If evidence linkage is blocked by Workpoint/project scope, run `focusa_workpoint_resume` and retry with explicit `project_root`/`workpoint_id`.
- If trajectory clarity gates time out, treat cached advisory trajectory as orientation only; run `focusa_resource_mode`/`focusa_tool_doctor` and retry summary trajectory later.
- Diagnostics are evidence, not canonical project truth; verify with code/tests or a second browser proof.

## Failure recovery

`tool_result_v1.failure_class` is part of the recovery contract. Common values: `scope_unverified` (recover with `focusa_project_verify` + `focusa_workpoint_resume`), `workpoint_unavailable` (recover with `focusa_workpoint_resume`), `daemon_unavailable` (recover with `focusa_tool_doctor`), and `evidence_attach_blocked` (drop `attach_to_workpoint=false` and re-run). When `failure_class` is missing, treat the call as advisory evidence; verify with `focusa_workpoint_resume` or `focusa_evidence_capture` before relying on the result.

## Contract summary

- Family: `workpoint`
- Side effects: `composite_evidence_prediction_optional_metacog`
- Result envelope: `tool_result_v1`
- API routes: `POST /v1/workpoint/evidence/link`, `POST /v1/predictions`, `POST /v1/metacognition/capture`
- CLI commands: `focusa workpoint evidence-link`, `focusa predict record`, `focusa metacognition capture`
- Core surface: `Pi composite Workpoint evidence intake`

## Next tools

- `focusa_active_object_resolve`
- `focusa_evidence_capture`
- `focusa_workpoint_link_evidence`
- `focusa_predict_record`
- `focusa_predict_evaluate`
- `focusa_metacog_capture`
