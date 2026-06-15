# Real Browser/Product QA — 2026-06-14

Operator directive: decompose all issues into workorder beads immediately, then run a deeper meticulous real browser/product QA pass using the same UIAI methodology. No `.sh` or Python test scripts are used for this QA track.

## Scope

- Workorder parent: `focusa-qasy`
- Browser sessions:
  - First pass: `y6mwDWN7` on `http://127.0.0.1:5173`
  - Second pass: `fShHY5B-` on `http://127.0.0.1:1420`
- Focusa scope: `/home/wirebot/focusa`, continuity `focusa-cont-root-20b6704c-5a49-4d9d-a4b6-a30bf45bfc61`
- Product surfaces covered:
  - Menubar first-run and connection fallback
  - Menubar tabs: Focus, Now, Path, WP, Proof, Loop, Gate, Sync, Pair, Settings
  - Canvas route: `/canvas`
  - Daemon/browser endpoints: `/v1/health`, `/v1/doctor`, `/v1/awareness/card`, `/v1/workpoint/current`, scoped `/v1/workpoint/resume`, `/v1/project/verify`, `/v1/ontology/tool-contracts`, `/v1/work-loop/health`
  - Focusa tools: tool doctor, Workpoint resume, Trajectory view, Context Cognition, Project Card, evidence capture, diagnostics intake, prediction/evaluation, project verify
  - Awareness/OpenClaw plugin package surfaces
  - Pi extension prompt/skill surfaces
  - Generated/current docs product surfaces

## Environment changes during QA

- Temporarily set UIAI `vision.allow_private_urls: true` to permit local product browser testing.
- Restarted UIAI engine and verified browser health.
- After first pass, restored `allow_private_urls: false` and verified standby.
- Re-enabled for second pass; restore is required at end of active QA loop.

## Workorders created

Parent: `focusa-qasy` — Real browser/product QA hardening from Spec106 completion.

Initial children: `focusa-qasy.1` through `focusa-qasy.19`.

Additional deeper-pass children:

- `focusa-qasy.20` — Fix awareness card cross-project stale Workpoint contamination.
- `focusa-qasy.21` — Fix workpoint current versus scoped resume mismatch.
- `focusa-qasy.22` — Fix transient post-connect no-daemon-signal state.
- `focusa-qasy.23` — Modernize Pi extension focusa-context prompt route.
- `focusa-qasy.24` — Add scope/tool-result contract to awareness OpenClaw plugin docs.

## Menubar findings

### First-run / connection

Bead: `focusa-qasy.1`

- First-run screen renders `Connect to Focusa`.
- Browser/Vite mode shows raw implementation error: `Cannot read properties of undefined (reading 'invoke')`.
- Message also says automatic callback unavailable and suggests Advanced paste fallback.
- A11y snapshot initially exposes only `Copy errors` and `Advanced`.
- Advanced fallback opens connection settings and `Connect / Test` succeeds against `http://127.0.0.1:8787`.
- After `Connect / Test`, localStorage contains:
  - `focusa_api_url=http://127.0.0.1:8787`
  - `focusa_has_connected_successfully=true`
  - one saved `Local Focusa` connection.

Bead: `focusa-qasy.22`

- After successful connect, second pass briefly showed `Focusa is out of view` and `No daemon signal yet` before main app loaded.
- This is contradictory after a successful Connect/Test and should be a deterministic loading state.

### Focus tab

Bead: `focusa-qasy.12`

- Second pass body text measured approximately 575,126 characters.
- Shows current bubble but also `BACKGROUND CLOUDS (4537)` and many stale historical frames.
- Browser reads are huge/truncated and product status is hard to inspect.

### Now / Path / WP tabs

Bead: `focusa-qasy.2`

- Now tab shows:
  - `No mission loaded`
  - `ProjectIdentity unknown`
  - `Continuity ID unbound`
  - `Workpoint not_found`
  - `Workpoint not canonical`
  - `Context Authority status unknown`
- Path tab shows:
  - `No trajectory yet`
  - `unknown project`
  - HLT/MLG/STG not defined or derived placeholders
- WP tab shows:
  - `No canonical continuation`
  - `not_found`
  - `no project root`
  - no mission/current action/next action/evidence/blockers/do-not-drift.

### Proof tab

Bead: `focusa-qasy.3`

- Shows `scope:unknown`.
- Workpoint evidence count is `0` despite QA evidence captures.
- Shows predictions/metacog, but includes unrelated Arena predictions.
- Needs scoped evidence and prediction filtering.

### Loop tab

Bead: `focusa-qasy.13`

- Shows `Readiness unknown`, writer `unknown`, status `transport_degraded`.
- Active task text is stale/unrelated: `Run real deep Focusa tests beyond harness`.
- Matches `focusa_tool_doctor` work_loop transport degradation.

### Gate tab

Bead: `focusa-qasy.4`

- Shows `SOFT CANDIDATES (200)`.
- Contains unrelated/stale candidates from TEP Book, JARVIS, Flow Mesh, CONTRIBUTING.
- Many entries have `0%` confidence.
- Needs project/continuity/recency filtering and default collapse/hide for stale low-confidence entries.

### Sync tab

- Baseline OK in second pass.
- Shows no peers configured and Add peer action.

### Pair tab

Bead: `focusa-qasy.5`

- Pair UI renders instructions and QR path.
- QR appeared after about 1.4 seconds in second pass.
- Polling label shows timestamp-like value: `attempt 1781484134053`.
- Paired device list contains duplicate/stale `Mac ScriptalertMac` entries.
- Active and revoked devices are mixed together.
- Second pass counted 7 paired entries and 18 revoked mentions in same visible list.

### Settings tab

- Shows connected local Focusa state.
- Displays active URL `http://127.0.0.1:8787`, saved connections `1`, events `6`.
- Connection help is visible and useful.

### Canvas route

Bead: `focusa-qasy.6`

- `/canvas` visually loads Focus Canvas with ASCC/Timeline controls.
- Browser read returns only `90%`.
- Accessibility snapshot exposes ASCC, Timeline, and one unnamed button.
- DOM text contains actual timeline entries, so content exists but is not exposed well to browser/read/a11y.
- Diagnostics include repeated `[focusa-menubar-diagnostic] Object` console errors.

### Favicon

Bead: `focusa-qasy.7`

- First pass diagnostics found `/favicon.ico` 404.

## Daemon/browser endpoint findings

Bead: `focusa-qasy.15`

Browser fetch matrix from session `fShHY5B-`:

- `GET /v1/health`: 200, ok true.
- `GET /v1/doctor`: 200, status ok, but `contracts_expected=58`.
- `GET /v1/awareness/card`: 200, canonical true, top-level `project_root`, `continuity_id`, `session_id` null.
- `GET /v1/workpoint/current`: 200, canonical false, status `not_found`.
- Scoped `POST /v1/workpoint/resume`: 200, canonical true, status completed, Workpoint `019ec8bb-b2ea-7252-a5a6-45e3f4e689f0`.
- `POST /v1/project/verify`: 200, canonical true, project root `/home/wirebot/focusa`.
- `GET /v1/ontology/tool-contracts`: 200, tool_count 80.
- `GET /v1/work-loop/health`: 200, status ok.

Bead: `focusa-qasy.8` and `focusa-qasy.20`

- `/v1/awareness/card` top-level scope is null but rendered card says `project-bound Workpoint found`.
- Rendered card points to unrelated Arena mission and `/home/focusadev/philoveracity-launch-pages` continuity.
- This is cross-project stale Workpoint contamination and authority-risky.

Bead: `focusa-qasy.9`

- `/v1/doctor` expects 58 contracts while current registry reports 80.
- `focusa_tool_doctor` also reports static contracts 79 vs live contracts 80.
- Tool/contract count drift exists in more than one product surface.

Bead: `focusa-qasy.21`

- `/v1/workpoint/current` not_found conflicts with scoped `/v1/workpoint/resume` canonical completed.
- Menubar appears to consume current/unscoped state and therefore shows no Workpoint.

## Focusa tool workflow findings

Bead: `focusa-qasy.16`

Actual Focusa tools used during this QA:

- `focusa_workpoint_checkpoint`
- `focusa_active_object_resolve`
- `focusa_predict_record`
- `focusa_predict_evaluate`
- `focusa_evidence_capture`
- `focusa_browser_diagnostics_intake`
- `focusa_failure`
- `focusa_note`
- `focusa_project_identity`
- `focusa_project_verify`
- `focusa_context_cognition`
- `focusa_context_cognition_render`
- `focusa_trajectory_view`
- `focusa_trajectory_assess`
- `focusa_project_card`
- `focusa_tool_doctor`

Findings:

- `focusa_tool_doctor` readiness ready, but static/live contract drift exists: 79 vs 80.
- `focusa_tool_doctor` says work_loop transport_degraded.
- `focusa_workpoint_resume` returns canonical QA Workpoint.
- `focusa_context_cognition` returns matched scope but advisory/canonical false.
- `focusa_project_verify` verifies `/home/wirebot/focusa` high confidence when canonical name is `Focusa`, but lowercase `focusa` gave mismatch earlier.

Bead: `focusa-qasy.10`

- Canonical-name case/alias sensitivity can produce avoidable mismatch.

## Awareness/OpenClaw and Pi extension artifact findings

Bead: `focusa-qasy.17`

Browser fetched:

- `apps/focusa-awareness/README.md`
- `apps/focusa-awareness/openclaw.plugin.json`
- `apps/focusa-awareness/index.ts`
- `apps/focusa-awareness/dist/index.js`

Findings:

- README and plugin JSON do not mention `project_root`, `continuity_id`, Workpoint, or `tool_result_v1` expectations.
- Index/dist include project_root and Workpoint logic but not clear continuity/tool_result contract in surfaced docs.

Bead: `focusa-qasy.23`

- `apps/pi-extension/prompts/focusa-context.md` is stale.
- It still says run curl `/v1/focus/stack` and `/v1/ascc/state` with jq.
- It does not reflect Spec106 route: ProjectIdentity → Trajectory → Workpoint → Context Authority/Context Cognition → Evidence.

Bead: `focusa-qasy.24`

- Awareness/OpenClaw plugin docs need explicit scope/tool-result contract.

## Docs/current product surface browser pass

Bead: `focusa-qasy.18`

Browser-fetched 14 docs/current surfaces through temporary local static server:

- `PI_EXTENSION_FINAL_TOOLSET_AUDIT.md`
- `FIRST_RUN_FLOW.md`
- `GOLDEN_WORKFLOW_PUBLIC_DEMO.md`
- `NON_PI_AGENT_ADAPTER_EXAMPLES.md`
- `COMMERCIAL_PACKAGING.md`
- `INSTALLER_UPDATE_POLICY.md`
- `MIGRATION_BACKUP_POLICY.md`
- `TEAM_MULTI_AGENT_FEDERATION_PLAN.md`
- `PUBLIC_PROOF_BUNDLE_VIEWER.md`
- `FOCUSA_GLOSSARY_LINKED_DOCS_UI.md`
- `EVAL_METRICS_DASHBOARD.md`
- `SECURITY_MODEL.md`
- `DEVICE_PAIRING_THREAT_MODEL.md`
- `TOKEN_AND_SECRET_HANDLING.md`

Results:

- All returned HTTP 200.
- No stale `partial`, `TODO`, `deferred`, `omitted`, `not implemented` markers in sampled launch docs.
- Only false-positive `gap` appears in valid phrase `active gap`.

## UIAI/tooling findings

Bead: `focusa-qasy.11`

- Normal UIAI click on Now tab timed out once in first pass.
- Full tab matrix eval timed out after 12 seconds in second pass.
- Splitting into smaller groups succeeds.
- Product panels are heavy enough to affect browser automation and likely user responsiveness.

Bead: `focusa-qasy.19`

- UIAI private local URLs must be enabled for local product browser QA.
- Procedure must include restore to `allow_private_urls:false` and active page cleanup.

## Cleanup reminders

- Close UIAI session `fShHY5B-` when done.
- Restore `/home/wpuiai/uiai-engine/config.yaml` `vision.allow_private_urls:false`.
- Stop temporary static docs server from `/tmp/focusa-static-docs.pid`.
- Existing long-running menubar Vite server on port 1420 predates this pass and was reused.
