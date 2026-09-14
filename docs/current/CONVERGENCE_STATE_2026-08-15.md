# Convergence State — 2026-08-15 (#101)

**Program:** automatically converge every enrolled operator-owned Focusa installation and managed surface.

## Converged today

| Surface | Convergence mechanism | Evidence |
| --- | --- | --- |
| Pi package updates | crash-safe OTA commit/rollback transaction | #309, `crates/focusa-cli/src/commands/pi_package.rs` |
| One canonical Pi package | one-canonical rule + retired-extensions + pre-push/CI gates | AGENTS.md, `docs/160` |
| Entitlement across surfaces | developer-origin resolver (agent-kb/tailnet) — uniform gates on trusted machines | #307, `license_developer_origin.rs` |
| Completion signaling | daemon SSE → extension notify → CLI wait — one channel for background-run completion | #311, `docs/159` |
| Event-ledger growth | retention + cold export + anchored hash chain, uniform across installs | `docs/158` |
| Distribution parity visibility | `scripts/audit-distribution-parity.mjs` manifest + drift detection + CI report | #260 |
| Installed vs source drift | detected and typed (0.9.152 installed vs 0.9.121-dev source — expected mid-cycle) | parity script live run |

## Remaining convergence gaps (refreshed 2026-08-16)

| Gap | Owner | State |
| --- | --- | --- |
| Remote hosts without a daemon | RemoteWorkspaceBinding | #89 slices 1-6 landed (docs/162); license-registry receiver + UIAI video remain |
| Workstream-scoped runtime (no singletons) | #125, docs/164 | slices 1-5 + full route-family migration landed; live-daemon acceptance pending restart window |
| Spec 152 unified entitlement service | #119, LICENSING_DIVERGENCE_AUDIT | CLOSED 2026-08-15 (engine in focusa-core, license facade) |
| Agent Card/component/digest parity enforcement at release | #260 | CLOSED — parity script + release gate wired into create-dev-release-tag |
| Menubar/Desktop transition with Pi preservation | #128 | PR #129 CI gate fixed (0531b8b3); review posted |
| CallGraph/Workset execution authority | #267-#274 | #254 slices 1-9 landed (types/validation/eligibility/ledger/routes/leases/liveness/replay/routing) |

## Invariants going forward

- Every surface reports its version/digest through the parity manifest at
  release time; drift blocks the release gate.
- Any new gate must resolve through one of: developer origin (#307), the
  unified entitlement service (#119), or an explicit operator decision —
  no third path.
- Installed surfaces update only through the OTA transaction (#309) or the
  canonical release pipeline.
