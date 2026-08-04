# Spec 135 Series — Spec 158 and Focusa Desktop Transition Overlay

**Status:** proposed normative overlay pending integration into `docs/135-series-current-manifest.md`  
**Date:** 2026-08-04  
**Applies to:** Spec 135 and companions 135A–135K  
**Upstream authority:** Spec 158  
**Transition authority:** FOCUSA-TRANSITION-001

---

## 0. Decision

This overlay resolves two newly identified conflicts in the current Spec 135 series:

1. the existing Work Surface/Attachment model often relies on Project + Continuity without a stable durable WorkstreamId;
2. existing 135A topology language can be read as making UIAI Engine Cockpit the single rich desktop owner for both products.

The corrected architecture is:

```text
Spec 158
  owns Workstream-rooted canonical cognition, reducer routing and persistence

Spec 135
  owns Mission Canvas, Work Surface, C.R.I.S.T. and workspace behavior

Focusa Desktop
  primary rich Focusa Mission Canvas and Pi environment

UIAI Engine Cockpit
  primary rich UIAI browser execution, FPV and Test Lab environment

Pi
  standalone and embedded coding/conversation Work Surface
  plus bounded terminal compatibility Canvas

Focusa menubar
  compact lifecycle/status/handoff surface

Focusa.work
  hosted/web projection of portable Focusa workspaces
```

---

## 1. Workstream identity amendment

Every Spec 135 primitive that can own, address, restore, mutate or present canonical cognition SHALL carry or resolve:

```text
ScopeRef / ProjectRootKey
WorkstreamId
optional ContinuityId
optional AttachmentKey
```

This applies to:

- Mission Canvas;
- Mission Deck;
- Work Surface;
- Work Rail;
- C.R.I.S.T. state;
- Context and Role;
- Workpoints;
- tactical Trajectory;
- Sessions and Silent Sessions;
- layout restoration;
- deep links;
- generated UI operations;
- Evidence and Receipts;
- contention and approvals;
- browser/UIAI attachments;
- agent-control and Desktop presentation commands.

ContinuityId SHALL NOT remain the durable identity of a Workstream.

Thread is legacy terminology and SHALL NOT be introduced into new Spec 135 contracts.

---

## 2. Visual focus and authority

Visual focus, selected card, active tab, split pane, detached window, drag-and-drop order and restored layout are presentation state only.

A UI selection may request an explicit attach or navigate operation. It cannot silently alter canonical Workstream authority.

Cross-Project and cross-Workstream dashboards are aggregate read models. Mutations require an exact target Workstream.

---

## 3. Primary renderer amendment

The complete rich Mission Canvas SHALL NOT be constrained to or primarily implemented inside Pi TUI.

Pi retains:

- Focusa tools and commands;
- exact Workstream Attachment binding;
- Work Rail and compact status;
- steering, rehydration and compaction;
- standalone terminal/SSH operation;
- an embedded PTY-backed Desktop Work Surface;
- a bounded compatibility/recovery Mission Canvas.

Focusa Desktop becomes the primary rich renderer for Focusa Mission Canvas, C.R.I.S.T., Work Surfaces, Evidence, Documents and Research.

The existing Pi implementation is preserved and decomposed under FOCUSA-TRANSITION-001 rather than discarded.

---

## 4. Desktop product topology amendment

Where 135A describes UIAI Engine Cockpit as the single primary rich desktop operator environment, replace that interpretation with product-qualified ownership:

```text
Focusa Desktop
  rich Focusa cognition and Mission Canvas authority projection

UIAI Engine Cockpit
  rich UIAI browser execution and proof authority projection
```

The products may share versioned product-neutral shell primitives. They retain separate licensing, app identities, canonical runtime ownership and release channels.

---

## 5. Agent-control amendment

Spec 135 Desktop surfaces SHALL be fully addressable through one semantic command graph shared by:

- Desktop navigation;
- Desktop command palette;
- Focusa CLI;
- Pi tools/commands;
- Focusa agent tools;
- Focusa.work.

Stable identifiers include:

```text
workspace_id
subsection_id
view_id
object_ref
work_surface_id
command_id
layout_id
operation_id
receipt_ref
```

The agent SHALL NOT require coordinate clicks, selectors, OCR or button-label matching to control Focusa Desktop.

---

## 6. Generated UI and framework decisions remain

This overlay does not reopen the frozen generated UI stack:

```text
A2UI 0.9.1
@a2ui/web_core/v0_9
@a2ui/lit/v0_9
Focusa Svelte Custom Elements
```

SvelteKit owns authored shell/composition. A2UI Lit owns generated surface processing. Generated OpenAPI clients own transport DTOs. Clients must not introduce handwritten duplicate DTO layers or a competing Svelte A2UI renderer.

---

## 7. Preview and release amendment

Focusa Desktop development follows:

- continuous shared SvelteKit browser preview;
- UIAI Engine browser proof;
- one pinned MacBook Rust toolchain;
- full Tauri shell gates at 5%, 25%, 50%, 75% and 100%;
- no direct MacBook push to main/shared legacy branches;
- canonical release initiation from the approved KnownHost release host at 75% after operator approval.

See `docs/transitions/FOCUSA-TRANSITION-001-preview-build-and-release-milestones.md`.

---

## 8. Required integration work

Before this overlay is marked integrated:

- update the 135 current manifest;
- add WorkstreamId to Spec 135 schemas and ledgers;
- audit Work Surface/Attachment/restore/deep-link contracts;
- correct Pi Mission Canvas models before extraction;
- update 135A desktop topology wording;
- update 135G Work Surface identity;
- update 135I generated UI operation envelopes;
- update 135J operation/event envelopes;
- update tests and compatibility matrices;
- link the Spec 158 and transition task graphs.

Until those edits land, this overlay governs conflicting implementation choices.
