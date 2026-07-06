# Focusa Public Docs Sync

Purpose: keep public-facing docs aligned with what is proven in the current MVP/Operator Preview branch.

## Proven public entry points

| User intent | Public doc | Proof boundary |
|---|---|---|
| Understand Focusa | [`README.md`](../README.md) | Current snapshot, not final product claims |
| Install and quickstart | [`RELEASE_INSTALL_POSTCARD.md`](RELEASE_INSTALL_POSTCARD.md) | Install → start → health → quickstart → Mission Deck |
| Five-minute buyer proof | [`GTM_FIVE_MINUTE_PROOF.md`](GTM_FIVE_MINUTE_PROOF.md) | Clone/install/start/init/deck/walkthrough/API/CI proof path |
| Deep Mission Deck plan | [`117-mission-deck-onboarding-recall-pwa-spec.md`](117-mission-deck-onboarding-recall-pwa-spec.md) | Spec roadmap; PWA/full Recall are not all complete |

## Public claims allowed now

- Focusa has a Rust daemon, CLI, TUI/Mission Deck, API, Pi extension, and menubar build proof.
- Mission Deck has Deck Home, Beginner Mode, Help Overlay, Next Safe Action, Mission Ladder, Proof Meter, Scope Badge, lightweight advisory Recall tab, and read-only Deck API routes.
- Walkthroughs exist for First Mission, Agent Handoff, and No Proof, No Done.
- Recall is advisory in the current lightweight surface; full Recall expansion is tracked separately.
- PWA work remains roadmap/deferred until the workspace path is finalized.

## Public claims to avoid until separately proven

- Full PWA is shipped.
- Full Recall implementation is shipped.
- Recall can directly create canonical Workpoints.
- Proof gaps count as done.
- Installer/platform behavior is universal unless verified on that platform.

## Current launch-blocking polish beads

- `focusa-117-arch.24` — Final TUI beautification pass before launch.
- `focusa-117-arch.25` — Ensure blazing-fast TUI startup and progressive loading.
- `focusa-117-arch.26` — Full newbie onboarding and walkthrough experience QA.
- `focusa-117-arch.27` — Full public GitHub Focusa sweep and onboarding docs polish.
- `focusa-117-arch.28` — Final pre-MVP polish across every layer.
- `focusa-117-arch.29` — Expand Mission Recall into a full dedicated specification.

## Sync checklist before public MVP claim

- [ ] README Quickstart matches current installer behavior.
- [ ] Release Install Postcard commands verified.
- [ ] GTM Five-Minute Proof run captured.
- [ ] Mission Deck screenshots/headless proof current.
- [ ] PWA/full Recall language accurately marked as roadmap/deferred/partial.
- [ ] CI green link captured for final launch commit.
