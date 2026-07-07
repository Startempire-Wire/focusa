# Phase 2 — Focusa Operator Preview (controlled cohort)

**Date:** 2026-07-07
**Owner:** Verious Smith
**Status:** planned, not yet launched
**Tag:** `v0.9.74-dev`

## Goal

Run a tight, **5–10 person Operator Preview cohort** before any public Product
Hunt surface. The cohort is for serious AI coding users who can install Focusa
from a one-line installer, complete a five-minute demo, and report whether the
value is obvious without an operator in the room.

## What we are validating

| Question | How we measure |
|---|---|
| Install path works on a real user machine | successful install rate (target ≥ 9/10) |
| Daemon comes up healthy | time to first `/v1/health ok` (target < 60s on Linux + macOS) |
| First Workpoint is obvious | time to first Workpoint (target < 5 min) |
| First Evidence ref is obvious | time to first Evidence ref (target < 10 min) |
| Resume after handoff/compaction works | successful resume rate (target ≥ 8/10) |
| Value is understood without an operator | score 1–5 on "did you need Verious to explain" (target ≤ 2) |

## Cohort profile (5–10)

Target audience (mix of):

- 2× indie hackers using Cursor / Claude Code / Codex / Pi power
- 2× solo SaaS builder who runs long agent sessions overnight
- 2× dev agency engineer carrying context across handoff
- 1× macOS Tauri / Swift pair-programming user (probes the menubar preview)
- 1× backend infra engineer who runs daemon on a VPS (proves the non-loopback auth path)

Selection criteria:

- Has shipped an AI-assisted project in the last 90 days
- Comfortable running `curl install.focusa.dev/focusa | bash` and reading logs
- Willing to give honest feedback in a 30-min debrief call

## Install path the cohort runs

```bash
curl -fsS install.focusa.dev/focusa | bash
focusa start
focusa init --quickstart
focusa doctor
```

Expected outcome of the five-minute proof:

1. `focusa doctor` returns `status=ok`.
2. `focusa workpoint checkpoint` returns a Workpoint id.
3. `focusa evidence link <workpoint-id> <proof-ref>` returns ok.
4. `focusa resume <workpoint-id>` returns the resumed state.

## Menubar preview cohort (optional 1× macOS user)

If a cohort member uses macOS, also exercise:

- `apps/menubar` build / install / first-run
- Device pairing via QR + VPS browser handoff (focusa-ui0y Mode C)
- Real `.app` lifecycle + screenshot/log capture (NOT required to ship preview verdict)

## What is OUT of scope for Phase 2

- Product Hunt listing
- Pricing / commercial license messaging
- Public web pages outside the canonical docs site
- Native menubar lifecycle claims (still tracked testing work)
- Marketing copy beyond the README + GTM five-minute proof

## Success criteria to advance to Phase 3 (Product Hunt)

ALL must hold for the cohort:

- ≥ 8/10 successful install rate
- ≥ 8/10 time to first Workpoint under 5 min
- ≥ 8/10 successful resume after handoff/compaction
- ≥ 7/10 score on "value without operator" (≤ 2 on the 1–5 scale)
- Zero P0/P1 install or runtime bugs reported in the cohort

If any criterion misses, we harden for another cohort, NOT advance.

## Risk register

| Risk | Mitigation |
|---|---|
| Cohort member can't install | one-line installer, fallback to release binary tarball |
| Cohort runs on AlmaLinux 8 (glibc 2.28) | musl artifact is default; documented in installer |
| Cohort runs into license gate on `release prove` | the install + daemon + Workpoint flow is fully open-source; `release prove` is Operator-tier |
| Cohort reports menubar issues | menubar is **preview**, not flagship; issues go into testing beads, do not block preview verdict |
| Other agent in flight introduces regressions | hold the v0.9.74-dev re-cut until the cohort install path is validated end-to-end |

## Status tracking

Update after each cohort member:

- `bd` issue per cohort member, priority 1, label `phase2-cohort`
- Close only after the debrief call is recorded
- Open `phase2-followup` bead for any bug or gap surfaced