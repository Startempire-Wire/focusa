# Spec 133 Phase 0 requirement-to-bead-test-evidence traceability

Date: 2026-07-13

## Phase map

| Spec 133 sections / acceptance family | Beads | Primary proof class |
|---|---|---|
| §30 Phase 0 freeze/baseline | `focusa-a6yq6.1.1`–`.1.5` | static evidence, legacy wrapper labels, dependency audit |
| §§8–12 domain, lifecycle, persistence, streams | `focusa-a6yq6.2.1`–`.2.7` | unit, reducer, persistence, replay, corruption tests |
| §§15–17, 23–24 config, API, authorization, CLI | `focusa-a6yq6.3.1`–`.3.6` | route/scope/CLI tests |
| §§13–16, 25 runner, adapters, model safety, Pi migration | `focusa-a6yq6.4.1`–`.4.7` | runner/adapter/model proof |
| §§13, 21, 28 supervision/recovery/resources/failures | `focusa-a6yq6.5.1`–`.5.7` | process/fault/resource tests |
| §18 concurrency/worktrees/scheduler/integration | `focusa-a6yq6.6.1`–`.6.6` | worktree and lease E2E tests |
| §§19–20 governance/evidence/completion/receipts | `focusa-a6yq6.7.1`–`.7.7` | Workpoint/Evidence/Spec119 proof |
| §22 operator/dashboard/menubar/Pi surfaces | `focusa-a6yq6.8.1`–`.8.6` | UI/API rehydration proof |
| §§14, 29.3 optional/cross-platform backends | `focusa-a6yq6.9.1`–`.9.6` | capability matrix proof |
| §§26–29, 32–33 retention/evolution/final matrix | `focusa-a6yq6.10.1`–`.10.9` | exhaustive runtime, security, real Pi, final audit |

## Gap closure mapping (§33)

| Gap | Closure bead range | Evidence gate |
|---|---|---|
| 1 Durability | Phase 1 | `focusa-a6yq6.2.7` |
| 2 Output | Phase 1 | `focusa-a6yq6.2.7` |
| 3 Lifecycle | Phase 1 | `focusa-a6yq6.2.7` |
| 4 Process supervision | Phase 4 | `focusa-a6yq6.5.7` |
| 5 Launcher defects | Phase 3 | `focusa-a6yq6.4.7` |
| 6 Authorization/security | Phase 2 | `focusa-a6yq6.3.6` |
| 7 Configuration | Phase 2 | `focusa-a6yq6.3.6` |
| 8 Provider/model safety | Phase 3 | `focusa-a6yq6.4.7` |
| 9 Operator experience | Phase 7 | `focusa-a6yq6.8.6` |
| 10 Foreground independence | Phases 2–4 | `focusa-a6yq6.4.7` |
| 11 Concurrency safety | Phase 5 | `focusa-a6yq6.6.6` |
| 12 Governance/evidence | Phase 6 | `focusa-a6yq6.7.7` |
| 13 Resources | Phase 4 | `focusa-a6yq6.5.7` |
| 14 Evolution/retention | Phase 9 | `focusa-a6yq6.10.8` |
| 15 Testing | Phase 9 | `focusa-a6yq6.10.9` |

## Provisional Spec132/history mapping

Spec 132 remains historically relevant for the current Pi/tmux compatibility surface and installer/TUI proof, but Spec 133 supersedes Pi-local tmux as canonical Silent Session architecture. Open Spec132 proof work (`focusa-slxpz.5.6`) is a release dependency blocker for Spec133 Phase0 gate closure and is recorded in `docs/evidence/spec133-phase0-release-gate.md`.

## Phase 0 evidence handles

- 0.1: `docs/evidence/spec133-phase0-baseline.md`
- 0.2: `docs/focusa-tools/tools/focusa_silent_sessions.md`, `apps/pi-extension/src/tools.ts`, `apps/pi-extension/src/tool-contracts.ts`
- 0.3: this file
- 0.4: `docs/evidence/spec133-phase0-release-gate.md`
- static proof: `tests/spec133_phase0_static_test.sh`
