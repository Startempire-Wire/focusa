# Spec 135C — UIAI Rich Artifact, Live Refresh, and Research Bridge

**Status:** draft, iterable, NOT FINAL — operator approval required  
**Owner:** Focusa / Verious Smith  
**Created:** 2026-07-17  
**Parent:** [Spec 135](135-focusa-professional-workspaces-and-crist-project-genesis-master-spec.md)  
**Closure relationship:** required companion; Spec 135 cannot close without Spec 135C.  
**Scope:** UIAI Engine screenshots, browser sessions, research, diagnostics, data, FPV, stable artifact descriptors, Focusa evidence linkage, workspace renderer dispatch, SSE invalidation, Pi rich rendering, terminal fallbacks, provenance, redaction, freshness, and cross-client parity.

---

## 0. One-line definition

UIAI Engine should act as the browser, research, media, diagnostics, and proof execution plane for Focusa, producing stable rich artifacts that Focusa scopes, links, evaluates, and projects into the live professional workspace without storing large browser blobs in hot context or requiring manual UI refresh.

---

## 1. Authority split

```text
Pi / Focusa clients
  Operator UX, tool selection, artifact viewing, steering.

UIAI Engine
  Browser/search/session/media/diagnostics execution and stable artifacts.

Focusa
  ProjectIdentity, Workpoint, Trajectory, Evidence, Context Authority,
  artifact linkage, Receipts, history, recovery, and next safe action.
```

UIAI may observe Focusa scope metadata. It must not mint Focusa authority.

Focusa must not rebuild UIAI’s browser, search, screenshot, FPV, or diagnostics systems.

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

The current UIAI Pi extension primarily returns JSON as text. Several browser tools call a helper that removes screenshot payloads before returning results. Screenshot metadata can include `artifact_path` or `artifact_url`, but the rich image is not presently inserted into a Focusa sidebar/detail surface.

Therefore:

```text
UIAI can create the artifact.
Focusa can link evidence.
Pi receives mostly textual metadata.
The rich workspace bridge remains an implementation gap.
```

---

## 3. Design laws

1. Stable handles over transcript blobs.
2. Large artifacts remain outside hot model context by default.
3. Every artifact preserves provenance, project scope, Workpoint, freshness, and redaction posture.
4. Events contain refs and invalidation hints, not full image/document payloads.
5. Focusa links and evaluates meaning; UIAI executes browser/research work.
6. Rich display degrades honestly by client capability.
7. A terminal without image support must remain fully operable.
8. Research remains proposal-only until captured/linked through Focusa Evidence.
9. Browser sessions and artifacts must expose cleanup/retention posture.
10. Cross-project artifact leakage is forbidden.

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
  browser_session_id:
  source_url:
  captured_at:

scope:
  project_root:
  continuity_id:
  workpoint_id:
  work_item_ref:

trust:
  evidence_status: proposal_only | capture_pending | captured | linked | verified | stale | blocked | scope_mismatch
  redaction_status:
  freshness_status:
  provenance_status:

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
| `fpv_session` | live UIAI FPV pane/share | session status + share/open action |

No client may silently discard an artifact because it cannot render the preferred format.

---

## 6. Required UIAI tool-output changes

UIAI Pi and agent outputs should return:

```text
compact textual summary
+ Workspace Artifact descriptor
+ Focusa evidence candidate
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
UIAI action
→ UIAI creates or identifies stable artifact
→ UIAI returns artifact descriptor and evidence candidate
→ Focusa validates project/workstream scope
→ Focusa captures or links Evidence
→ Focusa records artifact linkage event
→ Focusa emits workspace invalidation event
→ client refetches bounded artifact/read model
→ sidebar, detail pane, Work Rail, and history rerender
```

A Focusa link failure must not destroy the UIAI artifact. It returns `capture_pending`, `scope_mismatch`, or `blocked` with recovery guidance.

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
  "artifact_id": "uiai-screenshot:sha256:abc",
  "artifact_kind": "image",
  "invalidate": [
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
workspace_artifact_capture_pending
workspace_artifact_linked
workspace_artifact_verified
workspace_artifact_stale
workspace_artifact_redacted
workspace_artifact_removed
workspace_artifact_render_failed
```

Events do not carry base64 screenshots, full Markdown, full datasets, raw diagnostics, cookies, tokens, or private page dumps.

---

## 9. Live refresh behavior

Primary mechanism:

```text
Focusa SSE
→ reconnectable event cursor
→ map event to query keys
→ invalidate affected bounded read models
→ refetch active views
→ rerender
```

Required properties:

- automatic reconnect;
- duplicate-event tolerance;
- missed-event recovery through version/read-model refetch;
- project/workstream filtering;
- event ordering metadata;
- stale indicator during disconnect;
- polling fallback only when SSE is unavailable;
- no full workspace refetch for unrelated events.

UIAI live browser data may use its own stream/FPV transport, but Focusa workspace state changes still flow through Focusa linkage and invalidation events.

---

## 10. Image rendering tiers

Terminal image support is not universal.

```text
Tier A — native terminal graphics
Kitty/iTerm/Sixel or supported Pi image rendering.

Tier B — Focusa PWA/Tauri/UIAI FPV
Full rich image with zoom, side-by-side metadata, and evidence actions.

Tier C — terminal-safe thumbnail
Unicode/block or bounded preview where useful.

Tier D — artifact card
Title, source, dimensions, capture time, evidence status, and Open action.
```

The client capability profile chooses the best available tier. Tier fallback is not feature omission.

---

## 11. Visual provenance card

Every rich capture displays:

```text
Captured by
Source URL or source ref
Capture time
UIAI session
Project/workstream
Workpoint
Evidence handle
Verification status
Freshness
Redaction status
Retention/cleanup posture
```

A screenshot or research card without provenance is invalid.

---

## 12. Browser/research packet integration

Required packet flow:

```text
current Focusa scope
→ UIAI search/open/read/snapshot/diagnostics
→ ResearchDiagnosticsPacket
→ Focusa Evidence capture or browser diagnostics intake
→ active-object hints
→ optional prediction/metacognition
→ Workpoint checkpoint
→ artifact/history projection
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
- evidence handles.

The workspace bridge should project these into:

```text
Research card
Source reader
Claim/evidence extraction candidate
Context artifact candidate
Citation/provenance panel
```

Ingestion into Project Context remains governed by Spec 135B. Displaying research is not equivalent to accepting its claims as project truth.

---

## 14. FPV integration

The Focusa workspace should show an active UIAI browser session as:

```text
Browser active
URL/title
session status
observed FPS/latency where available
diagnostics status
Focusa scope status
operator steering/audit status
[Open FPV] [Inspect artifacts] [Capture evidence] [Close]
```

The rich PWA/Tauri cockpit may embed or dock FPV. Terminal clients open the share/view externally and display status locally.

Operator actions through FPV remain audited and must preserve the Pi steering boundary.

---

## 15. Vertical artifact projection

### Software

- screenshot comparison;
- responsive breakpoint gallery;
- DOM/accessibility snapshot;
- console/network diagnostics;
- visual regression;
- code diff linkage.

### Legal

- source document page image;
- citation source capture;
- exhibit card;
- redline and source comparison;
- deadline/source metadata;
- confidentiality indicators.

### Markets

- chart/data capture;
- source freshness;
- catalyst evidence;
- thesis revision;
- contrary-source set;
- explicit research-only status.

### Research

- source reader;
- claim extraction;
- supporting/contrary grouping;
- source graph;
- research synthesis artifact.

All projections use the same canonical artifact contract.

---

## 16. Security and privacy

Required:

- URL and source redaction;
- secret query-parameter stripping;
- cookie/header/token exclusion;
- private-target policy;
- artifact access control;
- short-lived share links;
- cross-project isolation;
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
- long artifact lists are virtualized;
- diagnostics are bounded;
- datasets use paged reads;
- images use responsive sizes;
- FPV does not block Focusa state updates;
- artifact capture runs outside canonical state locks;
- cleanup/retention runs asynchronously and produces observable state.

---

## 18. Client parity

Required surfaces:

- Pi compact and expanded tool renderers;
- Pi compatibility artifact cards;
- enhanced Pi detail pane;
- PWA/Tauri rich artifact gallery;
- menubar latest-proof peek;
- native TUI artifact metadata/fallback;
- API/CLI rehydration;
- MCP/JSON/RPC descriptors.

Rich clients may render more, but no client receives a semantically different artifact.

---

## 19. Acceptance criteria

Spec 135C is accepted when:

1. UIAI screenshot, read, snapshot, diagnostics, Source-to-Markdown, dataset/chart, and FPV results produce Workspace Artifact descriptors.
2. Focusa captures/links artifacts to correct project, continuity, Workpoint, and work item.
3. Scope mismatch and capture-pending states recover cleanly.
4. Focusa emits targeted invalidation events.
5. Active clients refresh automatically without manual reload.
6. SSE reconnect and missed-event recovery work.
7. Rich clients display images and documents.
8. unsupported terminals display useful fallbacks.
9. Artifact provenance, freshness, redaction, and evidence posture are visible.
10. Vertical renderer dispatch works.
11. UIAI research can become a Project Context candidate without becoming silent authority.
12. FPV status and launch controls work.
13. Security, redaction, access-control, and cross-project tests pass.
14. Large artifacts remain outside hot context and event payloads.
15. Pi, PWA, Tauri, menubar, TUI, API, CLI, MCP, and JSON/RPC parity is proven.
16. Actual screenshot and live-refresh proof artifacts are captured.

---

## 20. Closure blockers

This spec cannot close while:

- UIAI results remain text-only in the professional workspace;
- screenshot artifacts are discarded from the UX;
- manual refresh is required for normal linked artifacts;
- artifact events carry large blobs;
- provenance or scope is missing;
- rich display works only in one client with no fallback;
- FPV is disconnected from Focusa scope/evidence state;
- research display silently promotes project truth;
- security or cross-project isolation is unproven.
