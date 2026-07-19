# Spec 135C — UIAI Rich Artifact, Live Refresh, and Research Bridge

**Status:** draft, iterable, NOT FINAL — operator approval required  
**Owner:** Focusa / Verious Smith  
**Created:** 2026-07-17  
**Parent:** [Spec 135](135-focusa-professional-workspaces-and-crist-project-genesis-master-spec.md)  
**Closure relationship:** required companion; Spec 135 cannot close without Spec 135C.  
**Scope:** UIAI Engine screenshots, browser sessions, isolated browser contexts, browser targets, research, diagnostics, data, FPV, stable artifact descriptors, Focusa evidence linkage, semantic evidence candidates, session-origin identity, Mission Canvas renderer dispatch, semantic-delta versus UI-invalidation separation, SSE invalidation, Pi rich rendering, terminal fallbacks, provenance, redaction, freshness, and cross-client parity.

---

## 0. One-line definition

UIAI Engine should act as the browser, research, media, diagnostics, and proof execution plane for Focusa, producing stable rich artifacts that Focusa scopes, attributes to exact sessions/attachments/browser containers, links, evaluates, and projects into the live multiplexed Mission Canvas without storing large browser blobs in hot context or requiring manual UI refresh.

---

## 1. Authority split

```text
Pi / Focusa Mission Canvas clients
  Operator UX, Work Surface presentation, tool selection, artifact viewing,
  explicit steering targets, and bounded session inventory.

UIAI Engine
  Browser/search/session/context/target/media/diagnostics execution
  and stable artifacts.

Focusa
  ProjectIdentity, Workstream and Attachment identity, Workpoint, Trajectory,
  Evidence, Context Authority, artifact linkage, Receipts, history, recovery,
  and next safe action.
```

UIAI may observe Focusa scope metadata. It must not mint Focusa authority.

Focusa must not rebuild UIAI’s browser, search, screenshot, FPV, diagnostics, browser-context, or browser-target systems.

[Spec 135F](135f-domain-general-ontology-core-semantic-graph-domain-packs-and-reactive-context-spec.md) governs how artifact-derived objects, links, claims, evidence candidates, and semantic deltas enter candidate state, satisfy verification policy, and become canonical. [Spec 135G](135g-multiplexed-mission-canvas-work-surfaces-session-attachments-and-browser-context-isolation-spec.md) governs Work Surfaces, session attachments, browser-context isolation, and interaction routing. This spec governs artifact transport, linkage, rendering, and live invalidation; it does not create independent semantic or session authority.

---

## 2. Current implementation reality

UIAI already provides:

- persistent browser sessions;
- search;
- source-to-Markdown;
- page reads;
- accessibility/DOM snapshots;
- screenshots;
- diagnostics;
- browser actions;
- artifact paths and URLs;
- evidence handles;
- research/diagnostics packets;
- FPV share links and live browser streams;
- Focusa scope metadata.

The current UIAI Pi extension primarily returns JSON as text. Several browser tools call a helper that removes screenshot payloads before returning results. Screenshot metadata can include `artifact_path` or `artifact_url`, but the rich image is not presently inserted into a Focusa Mission Canvas sidebar/detail surface.

The current bridge also lacks a complete normalized distinction among:

```text
UIAI browser session
browser context/container
browser target/tab
Focusa Instance/Session/Attachment
Mission Canvas Work Surface
```

Therefore:

```text
UIAI can create the artifact.
Focusa can link evidence.
Pi receives mostly textual metadata.
The rich multiplexed Mission Canvas bridge remains an implementation gap.
```

---

## 3. Design laws

1. Stable handles over transcript blobs.
2. Large artifacts remain outside hot model context by default.
3. Every artifact preserves provenance, project scope, Workpoint, freshness, redaction posture, and session origin.
4. Events contain refs and invalidation hints, not full image/document payloads.
5. Focusa links and evaluates meaning; UIAI executes browser/research work.
6. Rich display degrades honestly by client capability.
7. A terminal without image support must remain fully operable.
8. Research remains proposal-only until captured/linked through Focusa Evidence.
9. Browser sessions, contexts, targets, and artifacts must expose cleanup/retention posture.
10. Cross-project and unintended cross-context artifact leakage is forbidden.
11. Workspace invalidation events and semantic ontology deltas are distinct contracts: one refreshes projections; the other may influence governed cognition only through registered subscriptions and reducer policy.
12. Browser target, browser context, UIAI session, Focusa Attachment, and Work Surface identities must remain distinct.
13. Closing a Work Surface must not implicitly close its UIAI session, browser context, or target.
14. Shared browser contexts require explicit visible selection; isolation may not be inferred from separate tabs alone.

---

## 4. Workspace Artifact contract

```yaml
schema: focusa.workspace_artifact.v1

artifact_id:
artifact_kind: image | markdown | dataset | diff | browser_snapshot | diagnostics | chart | document | media | fpv_session
mime_type:
title:
summary:

content:
  handle_ref:
  artifact_url:
  artifact_path:
  inline_preview:
  sha256:
  size_bytes:

source:
  system: uiai | focusa | local_file | connector | provider | operator
  source_ref:
  source_url:
  captured_at:

scope:
  project_root:
  project_identity_ref:
  continuity_id:
  workpoint_id:
  work_item_ref:

origin:
  instance_id:
  focusa_session_id:
  attachment_id:
  work_surface_id:
  harness_session_ref:
  silent_session_id:
  silent_run_id:
  uiai_session_id:
  browser_context_id:
  browser_target_id:

trust:
  evidence_status: proposal_only | capture_pending | captured | linked | verified | stale | blocked | scope_mismatch
  redaction_status:
  freshness_status:
  provenance_status:

semantic:
  domain_pack_refs: []
  candidate_object_refs: []
  candidate_link_refs: []
  candidate_claim_refs: []
  verification_policy_refs: []
  semantic_delta_refs: []

retention:
  policy:
  expires_at:
  cleanup_action:

render:
  preferred_renderer:
  fallback_renderer:
  width:
  height:
```

`artifact_id` must be stable and rehydratable. Path/URL fields are projections and may change.

Scope identifies which project/workstream the artifact belongs to. Origin identifies which runtime/session/context/target produced it. Neither may substitute for the other.

---

## 5. Artifact kinds and required renderers

| Artifact kind | Primary rich renderer | Required fallback |
|---|---|---|
| `image` | image viewer with zoom, metadata, source, and evidence | artifact card + Open action |
| `markdown` | cited research/document reader | bounded text + handle |
| `dataset` | sortable/filterable table | schema/row summary + download/open |
| `diff` | workspace-specific change viewer | unified text diff |
| `browser_snapshot` | structured accessibility tree and refs | bounded JSON/text tree |
| `diagnostics` | console/network/error inspector | summarized findings + refs |
| `chart` | interactive chart where supported | table and static summary |
| `document` | document/PDF reader | extracted text + source page refs |
| `media` | bounded media viewer | metadata + external/open action |
| `fpv_session` | live UIAI FPV Work Surface/share | session status + share/open action |

No client may silently discard an artifact because it cannot render the preferred format.

---

## 6. Required UIAI tool-output changes

UIAI Pi and agent outputs should return:

```text
compact textual summary
+ Workspace Artifact descriptor
+ Focusa evidence candidate
+ optional bounded semantic proposal refs
+ project/workstream scope
+ Instance/Session/Attachment origin refs
+ UIAI session/browser-context/browser-target refs
+ target_ref
+ preferred Focusa tool
+ next tools
+ cleanup posture
```

When the Pi tool-result content model supports images, screenshot tools may include a bounded image content item in addition to text and artifact metadata.

The implementation must stop treating `withoutScreenshot()` as the only safe presentation path. Instead it should choose among:

```text
inline bounded image
artifact descriptor
thumbnail
external rich viewer
text fallback
```

based on output mode and client capability.

---

## 7. Artifact capture flow

```text
UIAI action in an explicit session/context/target
→ UIAI creates or identifies stable artifact
→ UIAI returns artifact descriptor and evidence candidate
→ Focusa validates project/workstream scope and Attachment origin
→ Focusa captures or links Evidence
→ Focusa records bounded candidate semantic deltas where applicable
→ Spec 135F verification/promotion policy evaluates those candidates
→ Focusa records artifact linkage event
→ Focusa emits targeted Mission Canvas invalidation event
→ client refetches bounded artifact/read model
→ related Work Surfaces, sidebar, Work Rail, and history rerender
```

A Focusa link failure must not destroy the UIAI artifact. It returns `capture_pending`, `scope_mismatch`, `origin_mismatch`, or `blocked` with recovery guidance.

---

## 8. Event and invalidation contract

Example:

```json
{
  "schema": "focusa.workspace_event.v1",
  "event": "workspace_artifact_added",
  "project_root": "/project",
  "continuity_id": "main",
  "workpoint_id": "019...",
  "instance_id": "instance-1",
  "session_id": "session-1",
  "attachment_id": "attachment-1",
  "work_surface_id": "surface-1",
  "uiai_session_id": "uiai-1",
  "browser_context_id": "context-1",
  "browser_target_id": "target-2",
  "artifact_id": "uiai-screenshot:sha256:abc",
  "artifact_kind": "image",
  "invalidate": [
    "mission_canvas.surface_detail:surface-1",
    "workspace.artifacts",
    "workspace.sidebar.proof",
    "workspace.sidebar.research",
    "workspace.history",
    "workpoint.current"
  ]
}
```

Other required events:

```text
uiai_session_opened
uiai_session_status_changed
uiai_fpv_share_created
browser_context_created
browser_context_status_changed
browser_context_closed
browser_target_opened
browser_target_navigated
browser_target_moved
browser_target_closed
workspace_artifact_capture_pending
workspace_artifact_linked
workspace_artifact_verified
workspace_artifact_stale
workspace_artifact_redacted
workspace_artifact_removed
workspace_artifact_render_failed
```

Events do not carry base64 screenshots, full Markdown, full datasets, raw diagnostics, cookies, tokens, browser storage, or private page dumps. `focusa.workspace_event.v1` is a projection-invalidation contract and must not be treated as an ontology promotion event. Semantic deltas use the versioned Spec 135F envelope, stable refs, scope, cursor, and authority metadata.

---

## 9. Live refresh behavior

Primary mechanism:

```text
Focusa SSE
→ reconnectable event cursor
→ validate project/workstream and origin identity
→ map event to Mission Canvas/Work Surface query keys
→ invalidate affected bounded read models
→ refetch visible or subscribed views
→ rerender
```

Required properties:

- automatic reconnect;
- duplicate-event tolerance;
- missed-event recovery through version/read-model refetch;
- project/workstream/session/attachment filtering;
- Work Surface-targeted invalidation;
- event ordering metadata;
- stale indicator during disconnect;
- polling fallback only when SSE is unavailable;
- no full Mission Canvas or workspace refetch for unrelated events;
- no high-frequency hidden-pane rerender unless subscribed.

UIAI live browser data may use its own stream/FPV transport, but Focusa workspace state changes still flow through Focusa linkage and invalidation events.

---

## 10. Image rendering tiers

Terminal image support is not universal.

```text
Tier A — native terminal graphics
Kitty/iTerm/Sixel or supported Pi image rendering.

Tier B — UIAI Engine Cockpit / Focusa Mission Canvas rich client
Full image with zoom, side-by-side metadata, evidence actions, and origin identity.

Tier C — terminal-safe thumbnail
Unicode/block or bounded preview where useful.

Tier D — artifact card
Title, source, dimensions, capture time, session origin, evidence status,
and Open action.
```

The client capability profile chooses the best available tier. Tier fallback is not feature omission.

---

## 11. Visual provenance card

Every rich capture displays:

```text
Captured by
Source URL or source ref
Capture time
Project/workstream
Workpoint
Focusa Instance/Session/Attachment
Mission Canvas Work Surface
UIAI session
Browser context/container
Browser target/tab
Isolation/shared-context posture
Evidence handle
Verification status
Freshness
Redaction status
Retention/cleanup posture
```

A screenshot or research card without provenance and origin is invalid.

---

## 12. Browser/research packet integration

Required packet flow:

```text
current Focusa scope + explicit Attachment/UIAI context
→ UIAI search/open/read/snapshot/diagnostics
→ ResearchDiagnosticsPacket
→ Focusa Evidence capture or browser diagnostics intake
→ bounded candidate semantic objects/links/claims
→ active-object hints
→ optional prediction/metacognition
→ Workpoint checkpoint
→ artifact/history/Work Surface projection
```

Packet capture types include:

- search;
- source Markdown;
- browser read;
- snapshot;
- screenshot;
- diagnostics;
- error;
- share/FPV.

Research packet fields must remain bounded and secret-safe.

---

## 13. Source-to-Markdown and document presentation

UIAI Source-to-Markdown can include:

- Markdown;
- metadata;
- links;
- optional image references;
- JSONL/chunks;
- diagnostics;
- evidence handles;
- session/context/target origin refs.

The Mission Canvas bridge should project these into:

```text
Research Work Surface
Source reader
Claim/evidence extraction candidate
Context artifact candidate
Citation/provenance panel
```

Ingestion into Project Context remains governed by Spec 135B. Displaying research is not equivalent to accepting its claims as project truth.

---

## 14. UIAI session, browser-context, target, and FPV integration

### 14.1 Required hierarchy

```text
UIAI Browser Session
└── Browser Context / Container
    ├── Browser Target / Tab A
    ├── Browser Target / Tab B
    └── Worker/Popup targets where supported
```

The Focusa Mission Canvas must show every active UIAI session and context as a resolvable work object rather than collapsing them into one active browser.

### 14.2 Required browser Work Surface summary

```text
Browser session active
UIAI session ID
Browser context/container ID
Isolation class
Target/tab count
Current target URL/title
Session and context status
Observed FPS/latency where available
Diagnostics status
Focusa project/workstream/Attachment status
Operator steering/audit status
[Open FPV] [Targets] [Inspect artifacts] [Capture evidence] [Close view]
```

### 14.3 Isolation classes

```text
shared_authenticated
isolated_authenticated
ephemeral_isolated
read_only_observer
capture_worker
```

Two Work Surfaces must not share a browser context accidentally. Shared context requires an explicit action and visible badge. Separate targets inside one context do not constitute container isolation.

### 14.4 Target controls

Supported governed actions include:

- open target in same context;
- duplicate target in same context;
- duplicate/open target in new isolated context;
- move target to another context where the backend supports it;
- close target;
- close context with preview of all affected targets;
- close Work Surface without terminating session/context.

### 14.5 FPV

The UIAI Engine Cockpit may embed or dock multiple FPV Work Surfaces. Terminal clients open the selected share/view externally and display all active session/context states locally.

Operator actions through FPV remain audited and must preserve the Pi steering boundary and explicit Attachment target.

---

## 15. Vertical artifact projection

### Software

- screenshot comparison;
- responsive breakpoint gallery;
- DOM/accessibility snapshot;
- console/network diagnostics;
- visual regression;
- code diff linkage;
- test browser context and authenticated production-like context kept distinct.

### Legal

- source document page image;
- citation source capture;
- exhibit card;
- redline and source comparison;
- deadline/source metadata;
- confidentiality indicators;
- research browser context and client-authenticated context visibly separated.

### Markets

- chart/data capture;
- source freshness;
- catalyst evidence;
- thesis revision;
- contrary-source set;
- explicit research-only status;
- context/session timestamp and source isolation.

### Research

- source reader;
- claim extraction;
- supporting/contrary grouping;
- source graph;
- research synthesis artifact;
- multiple research contexts preserved as separate origin streams.

All projections use the same canonical artifact contract.

---

## 16. Security and privacy

Required:

- URL and source redaction;
- secret query-parameter stripping;
- cookie/header/token exclusion;
- browser storage exclusion from Focusa payloads;
- private-target policy;
- artifact access control;
- short-lived share links;
- cross-project isolation;
- browser-context cookie/storage/permission isolation;
- explicit shared-context disclosure;
- bounded artifact sizes;
- content-type verification;
- malicious file/content handling;
- HTML/Markdown sanitization;
- no arbitrary `file://` access;
- audit trail for operator browser actions;
- export redaction preview;
- local-first retention defaults.

---

## 17. Performance requirements

- artifact events contain refs, not blobs;
- thumbnails and previews are generated once and cached by content hash;
- full artifacts load lazily;
- long artifact and session lists are virtualized;
- hidden Work Surfaces do not rerender high-frequency content unless subscribed;
- diagnostics are bounded;
- datasets use paged reads;
- images use responsive sizes;
- FPV streams are per session/context and do not block Focusa state updates;
- artifact capture runs outside canonical state locks;
- cleanup/retention runs asynchronously and produces observable state;
- browser target updates invalidate only related context/session/Work Surface projections.

---

## 18. Client parity

Required surfaces:

- Pi compact and expanded tool renderers;
- Pi compatibility artifact cards and session switcher;
- enhanced Pi Mission Canvas detail pane;
- UIAI Engine Cockpit rich artifact/session/context gallery;
- Mission Deck PWA project/session overview;
- menubar latest-proof and active-session peek;
- native TUI artifact metadata/fallback and session inventory;
- API/CLI rehydration;
- MCP/JSON/RPC descriptors.

Rich clients may render more, but no client receives a semantically different artifact or session identity.

---

## 19. Acceptance criteria

Spec 135C is accepted when:

1. UIAI screenshot, read, snapshot, diagnostics, Source-to-Markdown, dataset/chart, and FPV results produce Workspace Artifact descriptors.
2. Focusa captures/links artifacts to the correct project, continuity, Workpoint, work item, Instance, Session, Attachment, UIAI session, browser context, and target.
3. Scope mismatch, origin mismatch, and capture-pending states recover cleanly.
4. Focusa emits targeted invalidation events.
5. Active Mission Canvas Work Surfaces refresh automatically without manual reload.
6. SSE reconnect and missed-event recovery work.
7. Rich clients display images and documents.
8. Unsupported terminals display useful fallbacks.
9. Artifact provenance, freshness, redaction, evidence posture, and session origin are visible.
10. Vertical renderer dispatch works.
11. UIAI research can become a Project Context candidate without becoming silent authority.
12. Multiple UIAI session/context FPV states and launch controls work.
13. Security, redaction, access-control, cross-project, and cross-context tests pass.
14. Large artifacts remain outside hot context and event payloads.
15. Pi, Mission Deck PWA, UIAI Engine Cockpit, menubar, TUI, API, CLI, MCP, and JSON/RPC parity is proven.
16. Actual screenshot and live-refresh proof artifacts are captured.
17. Artifact-derived semantic proposals remain candidate state until their registered verification and promotion policies pass.
18. UI invalidation and semantic reaction streams are separately versioned, filtered, replayable, and tested against accidental authority escalation.
19. Multiple browser targets operate inside one context with distinct target IDs.
20. Multiple isolated contexts prove separate cookies, local/session storage, permissions, and context identity.
21. Shared-context use requires explicit visible action.
22. Closing a Work Surface does not implicitly terminate the UIAI session/context/target.
23. Context/target close, move, duplicate, restoration, and retention behaviors pass actual runtime tests.
24. The word Cockpit is used only for UIAI Engine Cockpit.

---

## 20. Closure blockers

This spec cannot close while:

- UIAI results remain text-only in the professional workspace;
- screenshot artifacts are discarded from the UX;
- manual refresh is required for normal linked artifacts;
- artifact events carry large blobs;
- provenance, scope, or session origin is missing;
- rich display works only in one client with no fallback;
- FPV is disconnected from Focusa scope/evidence/Attachment state;
- research display silently promotes project truth;
- security, cross-project isolation, or browser-context isolation is unproven;
- workspace invalidation is treated as semantic promotion or autonomous-action authority;
- an artifact renderer or UIAI adapter silently invents canonical domain relations;
- browser context and target are conflated;
- multiple UIAI sessions/contexts cannot be represented simultaneously;
- closing a view implicitly terminates underlying runtime state;
- a generic Focusa/Pi surface is named Cockpit.
