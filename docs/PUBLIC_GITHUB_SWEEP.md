# Focusa Public GitHub Sweep

Purpose: confirm the public GitHub repo accurately highlights install, post-install, and quickstart processes, and matches current binaries/daemon behavior.

## Sweep findings

| Surface | Status | Evidence |
|---|---|---|
| README install command | accurate | `bash scripts/install-daemon.sh /usr/local` |
| README quickstart | accurate | `focusa start`, `focusa init --quickstart` |
| README Mission Deck reference | accurate | `focusa deck` / `focusa-tui` |
| Release Install Postcard | current | `docs/RELEASE_INSTALL_POSTCARD.md` |
| GTM Five-Minute Proof | current | `docs/GTM_FIVE_MINUTE_PROOF.md` |
| Public Docs Sync | current | `docs/PUBLIC_DOCS_SYNC.md` |
| Newbie Onboarding QA | current | `docs/NEWBIE_ONBOARDING_WALKTHROUGH_QA.md` |
| Spec 117 plan doc | current | `docs/117-mission-deck-onboarding-recall-pwa-spec.md` |

## Inaccuracies to avoid in public copy

- Claiming full PWA is shipped (deferred until apps/deck/ decision).
- Claiming full Recall implementation is shipped (lightweight advisory only; expansion tracked in `focusa-117-arch.29`).
- Saying Recall can directly create canonical Workpoints (forbidden by promotion flow).
- Hiding proof gaps instead of showing them.
- Listing uninstalled/installer-specific behavior without verifying.

## Required corrections before MVP launch

- Ensure README Quickstart stays aligned with `RELEASE_INSTALL_POSTCARD.md`.
- Keep `PUBLIC_DOCS_SYNC.md` allowed-vs-avoid list accurate.
- Reflect bead `.25` startup/loading guarantees in any time claims.

## Verification steps for the next sweep

```bash
git fetch origin --quiet
git status -sb
gh run list --workflow CI --limit 1
bash tests/release_deploy_automation_static_test.sh
```

Acceptance criteria:

- All public docs links resolve.
- All static guards pass.
- Latest CI run is green and matches current `main`.

Sign-off required from operator before claiming public MVP readiness.