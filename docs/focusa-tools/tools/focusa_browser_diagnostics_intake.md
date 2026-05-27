# focusa_browser_diagnostics_intake

Composite Pi tool for UIAI/browser debugging evidence.

Use after a browser diagnostics JSON artifact or typed UIAI action failure envelope is available.

## Inputs

- `diagnostics` — diagnostics JSON object or failure envelope.
- `diagnostics_ref` — stable artifact/file path/URL for the diagnostics JSON.
- `target_ref` — optional page, endpoint, selector, or component under diagnosis.
- `project_root` — explicit project root for canonical Workpoint evidence linkage.
- `attach_to_workpoint` — defaults true; false performs dry intake.
- `create_prediction` — defaults true.
- `create_metacog` — defaults false; enable only for reusable lessons.

## Effects

When attached, the tool links a bounded evidence ref to the active Workpoint, emits active-object hints, records a bounded prediction candidate, and can optionally capture a metacog lesson.

## Next tools

- `focusa_active_object_resolve`
- `focusa_evidence_capture`
- `focusa_predict_record`
- `focusa_metacog_capture`
