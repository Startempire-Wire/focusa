# Focusa Public Docs Sync

Purpose: keep public-facing docs aligned with the v0.9.142 stable-release surface and its verified proof boundaries.

## v0.9.142 public surface

The current release surface includes the Rust daemon/CLI/TUI, Pi extension, menubar preview, typed project/workstream scope, Trajectory and Workpoints, evidence/prediction/metacognition/work-loop routes, Context Cognition, Mission Canvas, browser/UIAI interoperability, device pairing, preload, Silent Sessions, Temporal Authority, and progressive Tool Discovery. Claims remain bounded by the linked runtime and release evidence.

## Proven public entry points

| User intent | Public doc | Proof boundary |
|---|---|---|
| Understand Focusa | [`README.md`](../README.md) | Current snapshot, not final product claims |
| Install and quickstart | [`RELEASE_INSTALL_POSTCARD.md`](RELEASE_INSTALL_POSTCARD.md) | Install → start → health → quickstart → Mission Deck |
| Five-minute buyer proof | [`GTM_FIVE_MINUTE_PROOF.md`](GTM_FIVE_MINUTE_PROOF.md) | Clone/install/start/init/deck/walkthrough/API/CI proof path |
| Deep Mission Deck plan | [`117-mission-deck-onboarding-recall-pwa-spec.md`](117-mission-deck-onboarding-recall-pwa-spec.md) | Spec roadmap; PWA/full Recall are not all complete |

## Public claims allowed now

- Focusa has a Rust daemon, CLI, TUI/Mission Deck, typed API, Pi extension, and menubar preview build proof.
- Mission Deck has Deck Home, Beginner Mode, Help Overlay, Next Safe Action, Mission Ladder, Proof Meter, Scope Badge, advisory Recall, and read-only Deck API routes.
- Agents can use project identity, Trajectory, Workpoint checkpoint/resume, evidence capture/linking, predictions, metacognition, work-loop health, and no-deadend recovery routes.
- Context Cognition, Project Card/Genesis, Temporal Authority, preload, Silent Sessions, browser/UIAI capability workflows, device pairing, and Tool Discovery are documented and parity-gated.
- Full PWA and full Recall expansion remain roadmap/deferred unless separately proven; the menubar remains a preview surface.

## Public claims to avoid until separately proven

- Full PWA is shipped.
- Full Recall implementation is shipped.
- Recall can directly create canonical Workpoints.
- Proof gaps count as done.
- Installer/platform behavior is universal unless verified on that platform.

## Remaining bounded polish boundaries

These historical roadmap beads are not used as release acceptance claims; each remains independently tracked and must not be presented as missing v0.9.142 core functionality.

- `focusa-117-arch.24` — Final TUI beautification pass before launch.
- `focusa-117-arch.25` — Ensure blazing-fast TUI startup and progressive loading.
- `focusa-117-arch.26` — Full newbie onboarding and walkthrough experience QA.
- `focusa-117-arch.27` — Full public GitHub Focusa sweep and onboarding docs polish.
- `focusa-117-arch.28` — Final pre-MVP polish across every layer.
- `focusa-117-arch.29` — Expand Mission Recall into a full dedicated specification.

## Sync checklist before public v0.9.142 claim

- [ ] README Quickstart matches current installer behavior.
- [ ] Release Install Postcard commands verified.
- [ ] GTM Five-Minute Proof run captured.
- [ ] Mission Deck screenshots/headless proof current.
- [ ] PWA/full Recall language accurately marked as roadmap/deferred/partial.
- [ ] CI green link captured for final launch commit.
