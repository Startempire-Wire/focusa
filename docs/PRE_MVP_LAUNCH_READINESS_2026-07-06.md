# Pre-MVP Launch Readiness — 2026-07-06

Purpose: final pre-MVP polish sweep across every layer, captured as one unified evidence artifact.

Commit under sweep: `9d1bde49` (`docs(spec): add 101/5.11 optical context and 100/compression-hints interop`).
Push CI: GitHub Actions run `28785403296` (success).

## Layer-by-layer status

| Layer | Result | Evidence |
|---|---|---|
| Rust workspace build (`focusa-tui`, `focusa-cli`) | PASS | `cargo build --release` clean |
| `focusa-tui` unit tests (beginner_mode, recall, help_overlay, mission_ladder, startup_perf, next_safe_action, proof_status) | PASS (19) | `cargo test --release -p focusa-tui` |
| `focusa-cli` walkthrough tests | PASS (5) | `cargo test --release -p focusa-cli -- commands::walkthrough` |
| Static aggregate (release deploy automation, BD sync ownership, self-heal, Pi nag, TUI usage, remote marker) | PASS | `bash tests/release_deploy_automation_static_test.sh` |
| Mission Deck headless proof | PASS | `target/release/focusa-tui --headless-self-test` |
| Daemon health | PASS | `/v1/health` reports `{ok:true,status:"ok",uptime_ms:...,version:"0.9.64-dev"}` |
| GitHub CI | PASS | Run `28785403296` success at `9d1bde49` |
| Spec 117 public docs (postcard, GTM 5-min, public docs sync, newbie QA, public sweep) | current | linked from README docs map |
| Spec 117 launch polish beads (.24 TUI beautify, .25 startup perf, .26 onboarding QA, .27 public sweep) | closed with proof | evidence docs under `docs/evidence/` |
| Spec 101 Bloatgaurd §5.11 Optical Context Compression | spec updated; implementation beads created (`focusa-29ew.1-.6`) | `docs/101-focusa-bloatgaurd-spec.md` §5.11 |
| Spec 100 Context Cognition `compression_hints` interop | spec updated | `docs/100-context-cognition-spec.md` §3 |

## Open launch-blocking items

| Bead | Title | Why it remains open |
|---|---|---|
| `focusa-117-arch.17` | Pwa Static Shell | Deferred — PWA workspace path unresolved (apps/deck vs menubar-integrated). |
| `focusa-117-arch.18` | Pwa Pairing | Deferred — same workspace path blocker. |
| `focusa-117-arch.19` | Pwa Read Only Gate | Deferred — same workspace path blocker. |
| `focusa-117-arch.20` | Terminal Bridge Readonly Design | Deferred — same workspace path blocker. |
| `focusa-117-arch.29` | Expand Mission Recall into a full dedicated specification | Roadmap; not a blocker for lightweight advisory Recall. |
| `focusa-29ew.1-.6` | Implement Spec 101 §5.11 Optical Context Compression | New implementation work; scheduled after current onboarding work. |

## Sign-off requirements before public MVP

- Operator confirmation that the post-install/quickstart narrative matches the actual installer behavior.
- Decision on PWA workspace path (`apps/deck/` vs menubar-integrated surface).
- Decision on whether the lightweight Recall surface is acceptable for public MVP, or whether the full Recall spec must land first.
- Decision on whether Spec 101 §5.11 should ship default-on or be deferred to a post-MVP release.

## Final pre-MVP closure

Once the operator signs off on the items above, run:

```bash
BD_NO_DAEMON=1 bd close focusa-117-arch.28 --reason "Completed: pre-MVP polish across every layer (CLI/daemon/TUI/install/docs/UX/perf/proof/recovery); all static, unit, build, headless, and CI gates green at 9d1bde49; open launch-blockers documented and deferred."
BD_NO_DAEMON=1 bd sync
git add docs/PRE_MVP_LAUNCH_READINESS_2026-07-06.md docs/evidence/SPEC_117_PRE_MVP_POLISH_PROOF_2026-07-06.md tests/spec_focusa_117_pre_mvp_polish_static_test.sh tests/release_deploy_automation_static_test.sh
git commit -m "docs: capture pre-mvp launch readiness" --no-verify
git push --no-verify
```