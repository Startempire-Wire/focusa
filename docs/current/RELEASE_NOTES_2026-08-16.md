# Release notes — 2026-08-16 canonical squash release

## What ships
The canonical big-squash (PR #314, `release/canon-final` from `origin/main`) lands the full bug-closure + IR2 + ecosystem session:

| Surface | Highlights |
|---|---|
| SSE/notifications | Live-tail yield fix — broadcast events reach consumers (root cause: live-tail consumed and dropped messages) |
| Background jobs | `focusa bg` + completion notifications with bounded output tails, EMA-based ETA, spinner widget |
| Workstream | Scoped write-through partitions; restart resumes the exact partition root (zero-bleed) |
| Workset program (#267–#274) | Append-only ledger, deterministic replay, evidence-gated DAG transitions, providers, freshness, context packets |
| CallGraph (#254) | Full spec surface: definitions, runs, dispatches, leases, 12-step disposition, FlowMesh, exports, envelopes, 30s sweeper |
| Ecosystem flywheel (#252) | Gaps A–F: UIAI research-packet intake, workset→completion + bg receipt→atom bridges, steer→workloop re-rank, cockpit projection, deslop-as-bg-job |
| Credentials (#299) | Secret-free grant verification + lifecycle routes; provider adapter seam |
| Letta line (#107) | The 4 stateful-runtime crates wired into the workspace (were invisible to check/test/clippy) |
| Guards | json_guard accepts the typed ScopeKind enum alongside query kinds (was rejecting every typed-scope body) |
| Docs | AGENTS.md/INDEX aligned with the current directives; Spec 152 authority-issued command rewrites; route-health sweep |

## Verification
- Workspace gates: cargo check + test (all targets, thousands of tests) + clippy `-D warnings` — green on the canonical OVH build host
- E2E live matrix: 21/21
- Route-health sweep: 26/26 post-deploy
- CI strict gates: resolution-policy corrections applied (main-wins typed types.rs + COMPACTION_FALLBACKS; shrinkwrap patch step; retired interview tool ref removed)
- Five-proof production consistency policy enforced per surface (docs/current/PRODUCTION_CONSISTENCY_POLICY.md)

## Deploy
- Maintenance window: healthcheck timer + daemon service down → binary swap (musl static-pie) → up
- Post-deploy checklist: health, version, sweep 26/26, predictions route, cockpit projection, e2e matrix

## Follow-through queue
- #299 provider adapter seam consumption
- 102 `as any` convergence (15 stripped this session, tsc green)
- #107 live Letta acceptance
- #288 operator-Mac TODO.txt exporter parity
- Workset operator approval surfaces (Spec 149 superseded by operator authorization)
