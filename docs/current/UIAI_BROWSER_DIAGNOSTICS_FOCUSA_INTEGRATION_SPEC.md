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


---

## 12. P0 UIAI Agent-Owned Continuous Visual Perception — Focusa feedback contract (2026-11-21)

**Status:** owner-requested P0 integration outcome; this addendum is an implementation proposal and cross-repository compatibility contract, NOT proof that the new UIAI observer or all Focusa target-state projections are installed. It augments the **existing** Focusa Browser Diagnostics and Eval integration, Visual/UI Ontology/Verification/Workflow docs 58–65, Spec135C/F/G and Specs 138/182/183. No new Focusa 135-letter, canonical evidence store, browser runtime, new Radar, new Foreman, new model identity, or new authority class is created.

**UIAI upstream implementation-owner PR:** [UIAI #321](https://github.com/WPUIAI/uiai-engine/pull/321). The two implementation-bound source documents are `docs/UIAI_AGENT_CONTINUOUS_VISUAL_PERCEPTION_P0_SPEC_2026-11-21.md` and `docs/contracts/UIAI_AGENT_CONTINUOUS_VISUAL_PERCEPTION_P0_C01.yaml` on that PR. UIAI requirement IDs `AVP-001`–`AVP-020` remain open until independently verified. Focusa-side work must not assert P0 shipped because a documentation PR merged.

### 12.1 User outcome and source-of-truth rule

An authorized worker visually observes the **rendered page itself** while coding or navigating, notices material visual defects, reasons about their significance, prepares a permitted fix, and visually verifies the actual changed site before claiming completion. The visual observer lives in UIAI; it is not a human-facing FPV feed or a new Focusa vision service. Focusa must own the entire project-level feedback chain that makes the observation actionable and recoverable.

```text
Human / Wirebot Chief of Staff
    -> exact Focusa Workstream / currently authorized Workpoint
    -> Workstream Foreman role (Spec 182 when actually installed)
    -> Focusa Work Loop / Silent Session worker (Spec 133)
    -> UIAI owned session / CDP screencast / changed-region sampling / real pixels
    -> agent visual reasoning -> bounded evidence-linked visual finding
    -> Focusa scope check + candidate/Workpoint intake
    -> authorized fix and UIAI visual + functional Eval
    -> existing Focusa Evidence, outcome / Receipt / Workpoint advancement
    -> Focusa Prediction and Metacognition (Spec 138), only if meaningful
    -> Focusa Radar (Spec 183 when installed), only distinct material conditions
    -> exact Foreman/Workstream attention -> next safe action
```

Each arrow is one governed boundary. No screenshot, browser page, untrusted CSS content, model caption, or Radar Signal acquires Focusa authority merely by being received.

### 12.2 Placement and dual-rate loops

Keep **Focusa daemon authority on KH** per `AGENTS.md`. UIAI browser processes, screencast, changed-region detector, image crops, vision inference and UIAI Eval operate on the authorized OVH execution nodes. The normal guarded KH/OVH relay transports bounded observation/evidence **refs and material summaries**, not continuous raw pixels or inference load to KH. An authorized remote agent may retrieve actual UIAI image content through the existing permission-checked private image adapter; this never implies public FPV sharing.

The fast **foreground worker perception loop** consumes frames and images at UIAI-native speed. Only material defect candidates, browser diagnostic changes, interactions that affect the current Workpoint, image artifact handles, and verification outcomes cross into Focusa. **Radar is a separate, lower-frequency attention loop**: it consumes those structured observations as approved `focusa.radar_observation.v1`, fingerprints/deduplicates into existing `RadarEpisode` and `RadarSignal` types, and routes them to the exact Foreman. It may not start its own browser screenshot watcher. Do not turn thousands of frames into thousands of Focusa writes.

### 12.3 Exact identity and authority binding

A project-bound capture must resolve and validate from the installed Focusa owner:

- ProjectRootKey / project_identity_ref, WorkstreamKey and ContinuityId, including verified `scope_source=focusa_verified` where the adapter exposes it.
- Exact Workpoint/current operation, required approval/grant, and if applicable Source/Work Item/Trajectory identities; **scope is not inferred from browser URL or selected Mission Canvas tab.**
- AttachmentKey (Workstream + instance/session/attachment) and Work Surface binding where an attachment exists; session/worker/actor/harness identity including SilentSession/Run when applicable.
- Corresponding UIAI session ID, browser context, browser target/tab, document/navigation/runtime generation, viewport/DPR, capture time, image observation ref and redaction/evidence status.

Do not claim that a literal payload includes all these keys before installed schema discovery. Consume exact generated Focusa Spec141 contracts, verify version and compatibility, and preserve the currently implemented Scope Guard: `docs/current/FOCUSA_UIAI_CROSS_PROJECT_SCOPE_GUARD.md`. A project-root string and continuity string alone do not authorize a project write; a verified attachment/actor and current permission posture still matter.

Every visual observation may be **source-authoritative for browser pixels** while being **advisory for Focusa semantic truth and work completion**. Wrong Workstream, stale document, mismatched session Attachment or missing required evidence grade are distinct typed errors. A private ad hoc visual capture is not retrospectively sealed as project Evidence.

### 12.4 Reuse current Focusa ingestion and visual domain models

Follow the exact installed capability-discovery path (`focusa_agent_card` → `focusa_tool_search` / `focusa_tool_describe`). When available under verified scope, use `focusa_browser_diagnostics_intake`, `focusa_evidence_capture` and `focusa_workpoint_link_evidence` for bounded links, and existing Spec135C Workspace Artifact linkage/events for authorized visual cards. `crates/focusa-api/src/routes/browser_interop.rs` currently validates WebMCP tool claims and workflow plans; do **not** treat `/v1/browser/capabilities/intake` as a newly installed visual-observation endpoint. If an image-finding intake shape is not yet exposed, record an explicit gap and compatibility task; do not fabricate success or make another store.

Map visual defects to existing `docs/60-visual-ui-verification-and-critique.md` conceptual output objects:
- `ComparisonResult`, `CritiqueResult`, `VerificationFinding`, `VerificationDimension`.
- A finding carries `severity`, `target_ref`, `expected`, `observed`, `remediation_hint`, `evidence_refs`, confidence and provenance.
- Separate **fidelity to approved reference** from **human-facing quality**. Avoid a universal aesthetic rule that penalizes appropriate whitespace, white backgrounds or premium editorial restraint.
- Categories include `responsive_issue`, `fidelity_gap`, `cohesion_issue`, `alignment_issue`, `token_mismatch`, `interaction_missing`, `state_missing`, `validation_missing`.

Map the lifecycle to the **existing** `docs/62-visual-ui-evidence-and-workflow.md` candidate concepts: reference intake → VisualRun → VisualIteration → comparison and critique → VisualFix → UIAI rerender/re-evaluation → CompletionReview. These are conceptual unless capability discovery verifies implemented operations; do not invent endpoints or canonical VisualRun storage. Spec135F candidate graph and verification policy must approve any candidate-to-canonical promotion. Spec135G Work Surface and Spec135C events only project/refresh **existing** state; UI selection and invalidation are not authority.

### 12.5 Workpoint and Foreman loop

The project worker reads current Workpoint and approved spec/artwork, receives a UIAI pixel-grounded finding with location/freshness and optional high-resolution crop, and fixes it under the existing Workpoint, writer lease and granted action. The agent must re-open the **installed/staging rendered** page and verify real pixels plus behavior before checkpoint/advancement. Never declare complete from source compilation or a pleasing screenshot alone.

The Spec182 Foreman projection (when implemented) hydrates from its existing source-bearing `focusa.foreman_hydration_packet.v1`, including `trajectory_ref`, `current_workpoint_ref`, `decision_refs`, `recent_evidence_refs`, `recent_receipt_refs` and `radar_episode_refs`. It may prioritize visual defects within approved scope, delegate distinct repairs to bounded Spec133 workers and report the result to Wirebot. It cannot alter visual design authority, approved scope or issue new grants. If Foreman runtime is not installed, the **existing Workpoint and governed Worker path remains functional** and projection status is explicitly pending; no phantom Foreman state.

### 12.6 Radar attention economics and operator routing

When a **material** condition persists or recurs, convert only the compact UIAI source observation into existing Spec183 RadarObservation with `scope_ref`, `workstream_root_ref`, `source_kind`, `source_ref`, `source_event_ref`, `observed_at`, `freshness_ref`, `trust_class`, `privacy_class`, `summary`, `structured_payload_ref`, `evidence_refs`, `confidence`, and `fingerprint`, subject to installed acceptance/version. Deduplication key: verified Workstream + affected browser surface/element + normalized condition + meaningful revision; timestamps alone are not new Episodes.

Examples: desktop header collision, crucial CTA clipped, wrong hero artwork, repeated failed responsive breakpoint, checkout state that appears ready but cannot fulfill. Radar's existing attention scoring considers impact, urgency, information gain and recurrence **minus** compute, privacy, interruption and duplicate costs. Low-signal pixel changes, animations and ordinary scroll are not operator notifications. Route useful signals to the owning Workstream Foreman; never bypass project interpretation/authority to accelerate execution. If Radar is not installed, record a source-bearing Workpoint/diagnostic candidate and target-state dependency; don't pretend an Episode was stored.

### 12.7 Prediction, learning, memory and continuity

Use **existing** `docs/current/PREDICTION_METACOG_SIGNAL_SUBSTRATE.md` and Spec138 rather than a new perception memory loop:
1. Retrieve relevant Workstream-scoped prior lessons (`focusa_metacog_retrieve`) before high-leverage visual repair.
2. Where available and appropriate, record the expected result (`focusa_predict_record`) *before* modifying CSS/code; preserve the original information set and uncertainty.
3. Run browser change, actual visual and functional Eval, and resolve outcome with current Evidence and receipts; then evaluate prediction (`focusa_predict_evaluate`).
4. Only after meaningful discrepancy, surprise or reusable success, submit bounded `focusa_metacog_capture` / reflection, adjustment proposal and outcome evaluation **as capabilities support**. Spec138's review, applicability, conflict, cross-project isolation, expiry, rollback and promotion gates remain in force. Reflection is not canon.
5. A later worker/Foreman rehydrates from canonical Workpoint/Trajectory/Evidence and retrieves relevant *promoted* lessons; no transcript, frame ring or model summary becomes durable instruction authority.

Master Loop, Portable Memory, Master Observer, Perpetua and Wirebot/SOVOS compositions, where independently approved and actually installed, consume these **already authorized Focusa outcome, learning and evidence projections** through their own interfaces. The presence of this doc does not launch them, create a second memory store or relocate Focusa's cognitive authority. Workstream continuity survives agent model/provider changes, resource pressure, compaction, takeover and daemon restart by the existing Focusa lifecycle.

### 12.8 Source provenance, release, privacy and truthful closure

The proposal/observation, browser execution, screenshot/crop, UIAI Eval result, Focusa Evidence link, Workpoint change, Receipt and approved reusable learning must be traceable via exact source/version refs. Do not inline massive frame streams, model private reasoning, credentials, sensitive page fields, OAuth params, cookies or user secrets into Focusa history. Image retrieval/persisted handles obey UIAI private capture/EPWA and the existing Focusa privacy/scope/retention policies. For ad hoc no-project observation use the distinct authorized ad hoc path; never use that mode to evade a formal Workpoint's Evidence obligation.

P0 integration cannot close because `github PR merged` or `website looks good`. Require real installed UIAI and Focusa versions; verified KH-to-OVH scope/authorization, raw-pixel-to-agent perception, bounded summary relay, a complete Workpoint correction/recheck with Evidence/Receipt; negative tests for wrong Workstream/Attachment, stale frames, capture/EPWA failure, absent model provider and secret leakage; native Pi/MCP/CLI truthfulness; independent human-facing aesthetic and functional review.

For the first end-to-end proof, use **The Entrepreneur's Path**, requiring 365/768/1024/1440 CSS widths plus 320/390 and fluid width sweeps, scroll/navigation, artwork, typography, desktop layout, hover/focus, interaction/checkout states. The code and actual rendered deployed revision must be reconciled before treating a screenshot as the currently shipped site. Measure source frames, selected model frames, actual model cost, Focusa writes, latency, missed critical visual defects, verified repairs and cost per accepted outcome.

**Dependency status:** Focusa 182 Foreman, 183 Radar, 135F semantic promotion and 135C artifact projections include target-state contracts; their documentation is not equivalent to installed support. Each unimplemented interface is an **OPEN** exact owner task/acceptance gap, not permission to drop the requirement. The current diagnostics/Workpoint/Evidence adapters provide the compatibility path until installed closure proof exists. Updates to normative architecture, scope or operational authority remain subject to the existing owner approval and generated-contract process.

## 13. Related P0 implementation cross-reference

- [WPUIAI/uiai-engine PR #321 — P0 agent-owned continuous visual perception](https://github.com/WPUIAI/uiai-engine/pull/321) — source architecture/acceptance and `AVP-001`–`AVP-020`.
- [Focusa Spec 135 series current manifest](../135-series-current-manifest.md) — frozen compatibility and generated UI/Eval ownership.
- [Focusa Visual/UI integration docs 58–65](../65-visual-ui-focusa-integration.md) — conceptual verification/iteration domain.
- [Focusa Spec 182 Foreman](../182-focusa-project-foreman-workstream-intelligence-projection-spec.md), [183 Radar](../183-focusa-radar-proactive-observation-episodes-signal-economics-and-attention-routing-spec.md), [138 learning](../138-focusa-prediction-outcome-calibration-metacognitive-learning-transfer-and-epistemic-governance-spec.md), [133 Silent Sessions](../133-daemon-native-durable-silent-sessions-and-governed-autonomous-execution-spec.md).
- Current `docs/current/FOCUSA_UIAI_CROSS_PROJECT_SCOPE_GUARD.md`, `docs/current/FOCUSA_UIAI_PACKET_CAPTURE_STATUS_RENDERING.md` and `docs/current/PREDICTION_METACOG_SIGNAL_SUBSTRATE.md` remain operative.
