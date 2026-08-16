# UIAI Browser Diagnostics and Eval → Focusa Integration Spec

**Status:** current local integration guide  
**Scope:** convert UIAI browser diagnostics, browser evaluations, screenshots, responsive/visual proof, and browser artifacts into bounded Focusa Evidence, predictions, Workpoint continuity, generated-UI updates, and Receipts.  
**Authority:** UIAI Engine owns browser execution and browser proof. Focusa owns project scope, cognition, Workpoints, Evidence linkage, authority, and recovery.

## 1. Purpose

UIAI Engine provides browser interaction, contexts, visual QA, diagnostics, and evaluation through local Rod/Chrome sessions. Focusa consumes structured browser results as scoped Evidence so agents diagnose and verify real web behavior instead of guessing from screenshots or transcript descriptions.

UIAI Engine and UIAI Engine Eval are the exclusive browser runtime and browser-proof plane for Focusa.

## 2. Current verified baseline

The local UIAI implementation includes:

- service target `localhost:7456`;
- browser open, screenshot, scroll, click, hover, type, eval, bounded async eval, snapshot, DOM, navigation, resize, CSS, wait, fill, select, press, history, text, cookies, close, diagnostics, and diagnostics-clear operations;
- bounded console, exception, request, failed-request, and summary diagnostics;
- Focusa scope on session creation and echoed session/diagnostic results;
- `focusa_browser_diagnostics_intake` Pi wrapper;
- browser health and pressure metrics;
- browser reliability gates and scoped stress/soak Evidence packets;
- stable screenshot/share Evidence handles;
- persistent browser sessions and artifact references.

Full HAR, trace export, source-map mapping, and raw body/header capture remain explicit capability gaps until UIAI implements them. Their absence does not authorize a second browser runtime in Focusa.

## 3. Ownership law

```text
UIAI Engine
  browser processes
  browser sessions and contexts
  browser targets
  browser actions
  diagnostics
  screenshots
  responsive and visual proof
  browser accessibility snapshots
  browser evaluation
  browser recovery

Focusa
  ProjectRootKey / WorkstreamKey / AttachmentKey
  Workpoint
  action authority
  Context and Trajectory
  Evidence and Receipt linkage
  predictions and metacognition
  generated Mission Canvas projections
```

Focusa MUST NOT implement a competing browser automation, browser test, browser diagnostics, visual-comparison, or browser evidence subsystem.

## 4. Browser Evidence flow

1. Start or resume the scoped Focusa Workpoint.
2. Open or reuse a UIAI session with explicit Focusa scope.
3. Reproduce the issue using UIAI browser actions.
4. Read UIAI diagnostics and browser state.
5. Run `focusa_browser_diagnostics_intake` or the typed Workspace Artifact/Evidence adapter.
6. Resolve candidate active objects from URL, stack, component, endpoint, and failed-request evidence.
7. Record a prediction for the likely cause and next action when useful.
8. Patch and test through normal project work.
9. Re-run UIAI diagnostics or UIAI Engine Eval.
10. Capture verification Evidence, evaluate the prediction, update the Workpoint, and emit a Receipt when required.

Agents MUST call UIAI diagnostics before guessing when work involves a blank/broken page, console failure, exception, failed interaction, navigation mismatch, API/network suspicion, CORS, authentication, responsive behavior, or visual mismatch.

## 5. Evidence shape

```json
{
  "target_ref": "browser:https://example.test/app",
  "evidence_ref": "uiai-diagnostics:session=abc12345:seq=42",
  "result": "Console TypeError in app.js:88 and GET /api/items returned 500."
}
```

Retain bounded:

- URL and title;
- console error/warning counts;
- top console errors;
- top exceptions with source positions;
- failed-request count and top failures;
- screenshot or visual-comparison handles;
- UIAI session/context/target IDs;
- diagnostics sequence;
- Focusa project, continuity, Workpoint, Attachment, and Evidence scope.

Do not store raw cookies, authorization headers, full bodies by default, inline image blobs, or unbounded logs.

## 6. Focusa-facing wrapper

`focusa_browser_diagnostics_intake` accepts diagnostics data or a diagnostics reference, target reference, explicit scope, Workpoint attachment posture, prediction posture, and metacognition posture.

It returns:

- bounded Evidence linkage;
- active-object hints;
- prediction candidate when requested;
- metacognition candidate when requested;
- exact recovery and next tools.

Embedded `focusa_scope` is reused only when it satisfies typed scope validation. Authority-bearing writes MUST NOT adopt ambiguous ambient scope.

## 7. UIAI Engine Eval contract

Browser-facing Spec 135 proof uses:

```yaml
schema: uiai.focusa_ui_eval_scenario.v1
scenario_id:
requirement_refs: []
project_scope:
work_surface_ref:
browser_context:
  isolation_class:
  authentication_fixture_ref:
viewport_matrix: []
steps: []
functional_assertions: []
accessibility_assertions: []
diagnostic_assertions: []
visual_assertions: []
reconnect_assertions: []
expected_focusa_events: []
expected_evidence: []
expected_receipts: []
```

```yaml
schema: uiai.focusa_ui_eval_result.v1
scenario_id:
status:
browser_session_refs: []
browser_context_refs: []
step_results: []
screenshots: []
diagnostics: []
accessibility_report_ref:
visual_comparison_refs: []
focusa_evidence_refs: []
receipt_refs: []
failure_class:
recovery_action:
```

UIAI Engine Eval owns browser end-to-end, responsive, visual, reconnect, authentication, browser-context isolation, diagnostic, and browser-accessibility proof.

## 8. Workpoint contract

Browser Workpoints use:

- `mission`: visible issue and target;
- `target_objects`: URL, endpoint, component/file hints, UIAI session/context IDs;
- `current_action`: reproduce, diagnose, patch, or verify;
- `verified_evidence`: UIAI diagnostic and Eval references;
- `next_action`: exact next browser/API/source step;
- `do_not_drift`: unrelated polish, refactors, and unverified assumptions.

## 9. Trajectory, ontology, prediction, and metacognition

- Use Trajectory assessment to determine whether the failure is a current gap, blocker, or drift.
- Resolve active objects from URL, stack, endpoint, component, and Evidence references; keep results candidate until verified.
- Record predictions before fixes and evaluate them after UIAI verification.
- Capture metacognition only when Evidence changes future debugging behavior.

## 10. Acceptance criteria

The integration is acceptable when:

1. UIAI diagnostics become bounded Focusa Evidence.
2. Workpoint resume identifies target, evidence, current action, and next action without transcript authority.
3. Trajectory and active-object tools cite UIAI Evidence.
4. Prediction record/evaluation closes after verification.
5. UIAI health and pressure appear through Focusa doctor surfaces.
6. Stress/soak and Eval results contain stable Focusa Evidence packets.
7. Screenshots and visual comparisons use stable artifact handles.
8. Browser-facing Spec 135 proof uses UIAI Engine Eval exclusively.
9. No competing Focusa browser runtime or browser test framework is introduced.
10. Browser results update generated Work Surfaces through typed Focusa Artifact, Evidence, event, and Receipt paths.

## 11. Cross-references

- UIAI browser diagnostics spec: `/home/wpuiai/uiai-engine/docs/BROWSER_DIAGNOSTICS_SPEC.md`
- UIAI browser reliability runbook: `/home/wpuiai/uiai-engine/docs/BROWSER_RELIABILITY_RUNBOOK.md`
- UIAI session docs: `/home/wpuiai/uiai-engine/docs/SESSION_API.md`
- Focusa diagnostics intake: `docs/focusa-tools/tools/focusa_browser_diagnostics_intake.md`
- Focusa Evidence: `docs/focusa-tools/tools/focusa_evidence_capture.md`
- Focusa Workpoints: `docs/focusa-tools/workpoint.md`
- Focusa active objects: `docs/focusa-tools/tools/focusa_active_object_resolve.md`
