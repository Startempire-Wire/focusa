# UIAI Browser Diagnostics → Focusa Integration Spec

**Status:** current local integration guide; UIAI diagnostics baseline implemented in `uiai-engine` commit `1221d80`.  
**UIAI companion spec:** `/home/wpuiai/uiai-engine/docs/BROWSER_DIAGNOSTICS_SPEC.md`.  
**Scope:** turn local browser console/network/runtime failures into bounded Focusa evidence, predictions, and Workpoint continuity.

## 1. Purpose

UIAI Engine provides browser interaction, visual QA, and implemented browser diagnostics through local Rod/Chrome sessions. Console errors, JS exceptions, and network failures are exposed as structured session data. Focusa should consume those diagnostics as evidence so models troubleshoot real web issues with proof instead of guessing from screenshots.

## 2. Current verified baseline

Local implementation in `/home/wpuiai/uiai-engine` now includes:

- UIAI service target: `localhost:7456`.
- Browser tools include open, screenshot, scroll, click, hover, type, eval, snapshot, DOM, navigate, resize, CSS, wait, fill, select, press, back, forward, text, cookies, close, `browser_diagnostics`, and `browser_diagnostics_clear`.
- Diagnostics expose bounded console logs/errors, JS exceptions, network requests, failed requests, and summary counts.
- Full HAR, trace export, source-map stack mapping, and raw body/header capture are not implemented in the baseline.
- Session routes live in `/home/wpuiai/uiai-engine/internal/routes/session.go`.
- Session diagnostics recorder lives in `/home/wpuiai/uiai-engine/internal/vision/diagnostics.go` and is attached from `/home/wpuiai/uiai-engine/internal/vision/session.go`.

## 3. Integration goals

- Convert browser diagnostics into stable Focusa evidence refs.
- Preserve reproduction context across compaction/model switch with Workpoints.
- Use active object resolution to map browser evidence to likely source files, API routes, components, or docs.
- Record bounded predictions before fixes and evaluate them after verification.
- Keep UIAI as the local lightweight browser backend; do not require Playwright/Puppeteer for this path.

## 4. Non-goals

- Focusa does not become a browser automation runtime.
- Focusa does not store raw HAR dumps, screenshots, or unbounded logs.
- Focusa does not treat diagnostics as canonical project truth without source/test verification.
- Focusa does not bypass UIAI redaction or local security boundaries.

## 5. Browser evidence flow

1. Start or resume a Focusa Workpoint for the web issue.
2. Open/reuse a UIAI browser session for the failing URL.
3. Reproduce the issue with existing UIAI browser actions.
4. Read UIAI diagnostics from `browser_diagnostics` / `GET /api/session/{id}/diagnostics`.
5. Capture a bounded Focusa evidence ref.
6. Resolve active objects from URL, stack, component names, endpoint paths, and failed network URLs.
7. Record a prediction for likely cause and next fix path.
8. Patch/test outside Focusa as normal project work.
9. Re-run UIAI diagnostics and capture verification evidence.
10. Evaluate the prediction and close/update Workpoint.

## 6. Evidence shape

Recommended Focusa evidence summary:

```json
{
  "target_ref": "browser:https://example.test/app",
  "evidence_ref": "uiai-diagnostics:session=abc12345:seq=42",
  "result": "Console TypeError in app.js:88 and GET /api/items returned 500; screenshot ref optional."
}
```

Recommended diagnostic fields to retain in Focusa evidence summaries:

- URL and title.
- Count of console errors/warnings.
- Top 1-3 console errors.
- Top 1-3 JS exceptions with source location when available.
- Failed request count.
- Top 1-5 failed requests with method, URL path, status, failure reason.
- Optional screenshot artifact path/ref.
- UIAI session ID and diagnostics sequence.

Focusa should avoid storing:

- Raw cookies.
- Authorization headers.
- Full request/response bodies by default.
- Large screenshots inline.
- Long console logs without summarization.

## 7. Tool choreography

Recommended Pi/Focusa sequence:

```text
focusa_workpoint_checkpoint
  mission="Troubleshoot browser failure on <url>"
  target_objects=["browser:<url>"]

UIAI browser_open / browser actions / browser_diagnostics

focusa_evidence_capture
  target_ref="browser:<url>"
  result="<bounded console/network summary>"
  evidence_ref="uiai-diagnostics:session=<id>:seq=<seq>"

focusa_active_object_resolve
  hint="<url + stack frame + failed endpoint>"

focusa_predict_record
  prediction_type="browser_failure_cause"
  predicted_outcome="<likely cause>"
  recommended_action="<next fix/check>"
```

After fix:

```text
UIAI browser_diagnostics
focusa_evidence_capture result="No console errors; failed requests 0 after fix"
focusa_predict_evaluate actual_outcome="<verified outcome>"
```

## 8. Proposed Focusa-facing wrapper

A future Pi tool or helper can wrap UIAI diagnostics and Focusa evidence capture:

### `focusa_browser_diagnostics_intake` candidate

Inputs:

```json
{
  "url": "https://example.test/app",
  "session_id": "abc12345",
  "diagnostics": {},
  "screenshot_ref": "/tmp/uiai/app.jpg",
  "project_root": "/path/to/project",
  "workpoint_id": "optional"
}
```

Outputs:

```json
{
  "evidence_ref": "uiai-diagnostics:session=abc12345:seq=42",
  "summary": "1 JS exception, 1 failed GET /api/items 500",
  "active_object_hints": ["/api/items", "app.js:88", "browser:https://example.test/app"],
  "recommended_next_tools": [
    "focusa_active_object_resolve",
    "focusa_predict_record",
    "focusa_workpoint_link_evidence"
  ]
}
```

This wrapper is optional. The current integration can already be done with existing Focusa tools plus UIAI CLI/API calls.

## 9. Workpoint contract

For browser debugging Workpoints, checkpoint fields should use:

- `mission`: user-visible browser issue and target URL.
- `target_objects`: URL, endpoint paths, component/file hints, UIAI session ID if stable enough.
- `current_action`: `browser_diagnostics_reproduce`, `patch_browser_failure`, or `verify_browser_fix`.
- `verified_evidence`: UIAI diagnostic evidence refs and test refs.
- `next_action`: exact next browser/API/source step.
- `do_not_drift`: unrelated UI polish, unrelated backend refactors, unverified assumptions.

## 10. Prediction and metacognition use

Before patching:

- Record a prediction for cause class:
  - `frontend_runtime_exception`
  - `failed_api_request`
  - `auth/session_state`
  - `asset_loading`
  - `cors/mixed_content`
  - `selector_or_render_timing`

After verification:

- Evaluate whether the prediction matched the actual cause.
- Capture a reusable metacog lesson only if it changes future debugging behavior.

## 11. Acceptance checks

Focusa integration is acceptable when:

- A UIAI diagnostics snapshot can be represented as one bounded `focusa_evidence_capture` result.
- A Workpoint resume packet can tell the next agent the URL, current evidence, and next browser/source action without transcript tail reliance.
- Active object resolution can use URL + stack + failed endpoint hints.
- Prediction record/evaluate closes the loop after fix verification.
- UIAI companion docs link back to this Focusa integration spec.

## 12. Cross-reference

- UIAI browser diagnostics spec: `/home/wpuiai/uiai-engine/docs/BROWSER_DIAGNOSTICS_SPEC.md`
- UIAI session docs: `/home/wpuiai/uiai-engine/docs/SESSION_API.md`
- Focusa evidence tool: `docs/focusa-tools/tools/focusa_evidence_capture.md`
- Focusa Workpoint tools: `docs/focusa-tools/workpoint.md`
- Focusa prediction guide: `docs/current/PREDICTIVE_POWER_GUIDE.md`
- Focusa active object tool: `docs/focusa-tools/tools/focusa_active_object_resolve.md`
