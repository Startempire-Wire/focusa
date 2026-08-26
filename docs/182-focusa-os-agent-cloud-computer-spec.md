# 182 — Focusa OS & Agent Cloud Computer Spec

**Status:** DRAFT (vision locked 2026-08-26). No code yet. Phases 0–1 are the first implementable slices.
**Owner:** Focusa product direction; extends 174 (extension MVP), 180 (widgets/wall), 181 (browser runtime).
**Companions:** `WPUIAI/uiai-engine/docs/010-uiai-engine-web-runtime-leap-spec.md` (C-010-*); Omarchy (basecamp/omarchy, MIT).
**Product frame:** the client setup offer includes an **Agent Cloud Computer** — a streamable, human/agent collaborative workspace built from **webtop + Focusa OS + UIAI Engine**.

---

## 1. Vision

Focusa today ships as a **browser extension** talking to a daemon. The primitives that power it
(daemon authority, worksets/workstreams, silent sessions, contextual authorization, approvals,
credentials broker, device pairing, work loop, roles, widgets/wall, SSE, audit) are **platform
primitives**, not extension features.

**Focusa OS** lifts those primitives from *extension ↔ daemon* to *OS services ↔ OS GUI*, so Focusa
stops being "a browser extension" and becomes an **agent/human operating environment**. The browser
extension, the desktop, mobile, and cloud runtimes become interchangeable **surfaces** over the same
daemon-owned truth.

The **Agent Cloud Computer** is the client-facing product: a real Linux desktop (streamable via
webtop/Selkies) running Focusa OS, with the UIAI Engine as the agent's browser/eyes — a workspace
where a human and one or more governed agents work side by side.

> **Surfaces are interchangeable; primitives are the platform.**

---

## 2. Composition of Focusa OS

```
┌──────────────────────────────────────────────────────────────┐
│  Custom Focusa GUI (QML shell, waybar modules, branded theme) │
├──────────────────────────────────────────────────────────────┤
│  System-wide Focusa layer (daemon svc, IPC bus, approvals,    │
│  credentials, pairing, work-loop scheduler, audit, notify)    │
├──────────────────────────────────────────────────────────────┤
│  UIAI Engine (agent browser: verbs, budgets, fleet, events)   │
├──────────────────────────────────────────────────────────────┤
│  Omarchy base (Arch + Hyprland, themes, dotfiles) — upstream  │
└──────────────────────────────────────────────────────────────┘
```

- **Base:** Omarchy (MIT) — beautiful, opinionated, Hyprland/Wayland, QML GUI stack. Kept **upstream-untouched**.
- **System-wide Focusa layer:** an **overlay** (separate repo/package) that installs *on top of* Omarchy. This is the maintainability key — no deep fork, so upstream updates merge cleanly.
- **Custom Focusa GUI:** branded Hyprland theme + waybar Focusa modules + QML shell surfaces (start page / wall / command panel as native desktop surfaces).
- **UIAI Engine:** the agent's browser and perception layer (intent verbs, budgets, warm fleet, artifact-ref screenshots, event stream) — already live on OVH.

---

## 3. System-wide Focusa layer — primitive → OS mapping

| Focusa primitive | Extension today | Focusa OS |
|---|---|---|
| Daemon (runtime/authority) | background service | OS init service (the agent "kernel") |
| Worksets / Workstreams | tab views | desktop work organization |
| Silent sessions | governed bg tasks | OS-level autonomous agent processes |
| `can(principal, capability, context)` | route gating | OS permission model for agents (sudo/UAC for agents) |
| Approvals | in-panel prompts | OS-wide approval prompts (human-in-the-loop) |
| Credentials broker | extension secrets | OS keyring / secret service for agents |
| Device pairing | pair a browser | pair any device as a trusted OS surface |
| Work loop | polling widget | OS agent scheduler (agent-aware cron) |
| Roles / composer | dropdown | OS-level agent identities/personas |
| Widgets / wall | browser panels | native desktop widgets + the "wall" |
| SSE event stream | notification center | OS notification bus (real-time agent events) |
| Audit | in-panel log | system-wide durable audit |

**Accept (layer):** each primitive is exposed as a system service with a stable IPC contract; a
reference GUI module consumes it read-only before any mutation surface exists.

---

## 4. Custom Focusa GUI

- **Branded Hyprland theme** in Focusa identity (colors, logo, cursor, splash).
- **waybar modules** surfacing live Focusa state: active mission, agent activity, pending approvals, budget posture.
- **QML Focusa shell** (Omarchy is ~31% QML — native stack): start page, wall, and command panel as desktop surfaces, not browser tabs.
- **Notification daemon** integration for agent events (from the SSE bus).

**Accept (GUI):** a Focusa-branded session boots under Hyprland; waybar shows live daemon state from a fixture; the shell renders the start page natively.

---

## 5. UIAI Engine as the agent's browser

The Engine (spec 010) is the agent's eyes/hands inside Focusa OS:
- **Intent verbs / budgets / fleet** give agents governed browsing.
- **Artifact-ref screenshots** give the human a view of what the agent sees.
- **Event stream** feeds the OS notification bus (agent activity is visible, never silent).
- The Engine runs as a system service; the GUI can embed an agent-view (live screenshot / stream).

**Accept:** an agent browsing task produces an artifact-ref visible in the Focusa GUI; a budget pause surfaces as an OS notification.

---

## 6. Architecture: overlay, not deep fork

- Keep Omarchy **upstream-untouched**; consume it as a base image / install.
- Ship Focusa as `focusa-os-layer` (its own repo): theme + services + GUI + tooling, installed on top.
- **Fork only** what must be rebranded (logo/name/boot splash). Everything else is overlay.
- Benefit: upstream Omarchy updates merge for free; the Focusa layer stays small, reviewable, brandable.

**Accept:** a clean `focusa-os-layer` applies onto a stock Omarchy install with no patches to upstream files.

---

## 7. Agent Cloud Computer (the client offering)

The deliverable bundled into client setup:
- **webtop** (LinuxServer/Selkies) for streaming the desktop to any browser (human view), no-login ephemeral share like `gui.focusa.dev`.
- **Focusa OS** (Omarchy + overlay) as the desktop environment.
- **UIAI Engine** as the agent browser.
- **Focusa daemon** as the authority; the extension remains as one optional surface.

Result: a human opens a URL and shares a desktop with governed agents; both act in one workspace.

**Accept:** a provisioned Agent Cloud Computer streams to a URL; a human and an agent both see and act on the same desktop state; teardown on idle.

---

## 8. Licensing & entitlement

- Premium OS surfaces (agent concurrency, cloud-computer hours, persona/mesh features) are **fail-closed** without an entitled tier.
- Authority is the existing **wpuiai.com WP-REST license validation** → `Identity{Tier}` (per spec 172 / engine licensing matrix).
- Unknown/internal tiers get zero premium OS features. The base desktop remains usable.

**Accept:** an unentitled Agent Cloud Computer boots to base desktop only; entitled tier unlocks agent surfaces; validation failure never grants premium.

---

## 9. Phased plan

- **Phase 0 — Prove the concept (days):** Focusa theme + preinstalled extension + daemon, on the *current* Ubuntu webtop, visible at a share URL. Validates streaming + branding cheaply.
  **Accept:** branded desktop streams; extension loads by default.
- **Phase 1 — Omarchy base + overlay (week):** `focusa-os-layer` (theme + daemon systemd service + preinstall) applied to an Omarchy image; runs in webtop.
  **Accept:** Omarchy+overlay boots in webtop; daemon service healthy; share URL shows Focusa OS.
- **Phase 2 — System layer (weeks):** OS IPC bus, waybar Focusa modules, approvals/notify integration, credentials broker as secret service.
  **Accept:** waybar shows live daemon state; an agent action triggers an OS approval prompt.
- **Phase 3 — Custom GUI (weeks):** QML Focusa shell (start/wall/command as native surfaces); agent-view embed (Engine artifact-ref).
  **Accept:** shell renders start page natively; agent screenshot visible in GUI.
- **Phase 4 — Ship (weeks):** installable image (real agent computers) + streamable webtop (human view) + licensing gate.
  **Accept:** provisioned cloud computer passes §7 + §8 acceptance.

Sequencing: 0 → 1 → 2 → 3 → 4. Each phase is independently valuable and releasable.

---

## 10. Non-goals

- A new kernel / from-scratch distro (Omarchy is the base).
- Deep-forking Omarchy (overlay only).
- Making the extension the only surface (it becomes one of many).
- Real-time GUI streaming from Cloudflare containers (CF DO/containers are for *headless* scale; interactive streaming stays on webtop/VM).

## 11. Risks & caveats

- **Containerizing Arch+Hyprland** (for streaming) is harder than Ubuntu-KDE; Phase 0 stays Ubuntu, Phase 1 invests in an Omarchy container or ships Omarchy as installable-only.
- **Upstream velocity:** Omarchy moves fast (6k+ commits); overlay design is the mitigation.
- **Scope:** this is a real product effort; phases keep it incremental. No phase is wasted.
- **GPU/encoding** for streaming lives in the webtop/Selkies stack, not in Focusa.

## 12. Open questions

- Omarchy-in-container vs installable-image-first for Phase 1?
- Which premium OS surfaces map to which license tiers (extend spec 172 matrix)?
- Branding: fork-and-rebrand vs overlay-only theming?
- How do device-paired mobile surfaces render the shared desktop (read-only stream vs interactive)?

---

## 13. Relation to existing work

- **Done primitives** (daemon, worksets, silent sessions, `can()`, approvals, credentials, pairing, work loop, roles, widgets/wall, SSE, audit) are **re-homed**, not rebuilt. The OS is packaging + surfacing.
- **GUI lab** (`gui.focusa.dev`, `uiai-lab-live`) is the live proving ground for Phases 0–1.
- **Extension** (174/180) remains a first-class surface and the fastest demo path.
