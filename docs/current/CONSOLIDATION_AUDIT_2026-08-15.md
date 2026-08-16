# Onboarding Seam Consolidation Audit — 2026-08-15 (#52)

**Gate:** P0 consolidation gate — onboarding seam audit, spec/runtime closure, dead-road removal.
**State:** audit evidence; the consolidation refactor compiles clean across the workspace (gates in flight).

## Dead roads removed (and why)

| Removed surface | Reason |
| --- | --- |
| `focusa-core/src/temporal*.rs` (temporal, temporal_authority, temporal_claims, temporal_forecast, temporal_tests) | Abandoned temporal-authority road; superseded by Spec 137/138 canonical prediction + metacognition authority. The epoch-0 placeholder event flood in the production DB came from this retired writer — removal also eliminates the junk source permanently. |
| `focusa-cli/src/commands/temporal.rs` + `crates/focusa-api/src/routes/temporal.rs` | Same road, CLI/API surfaces. |
| `project_genesis.rs`, `project_bootstrap.rs` + support/tests + CLI + e2e | Superseded by Spec 135B ProjectGenesisRecord + Spec 140 runtime constitution; duplicate bootstrap roads folded into `project.rs`/`focusa_project_verify` + the canonical marker service (#243). |
| `release_cycle.rs`, `release_intelligence.rs` + tests | Superseded by the canonical release pipeline (create-dev-release-tag → CI → deploy). |
| `docs/137a/138a/144` matrix family + closure scripts/tests + workflows | Documentation-architecture closure road retired; spec truth now lives in the canonical specs + generated capability contracts. |
| `config/focusa-release-topology.json` | Release topology moved into the canonical pipeline docs. |
| `apps/pi-extension.backup.0.9.121-dev/` | Age-bounded version backup; content superseded by 0.9.152 line. |

## Seam closure (onboarding journey)

```text
install → host health → project marker (#243 one canonical service) →
walkthrough (setup walkthrough alias fixed, #303) → first mission →
Workpoint checkpoint/resume (typed accepted/pending flow, #266) →
silent-session completion notification (#311)
```

- Marker production now routes through `focusa-core/src/project_marker.rs`
  (init + onboard wired; API parity next in IR2).
- `focusa setup walkthrough` registered — the deprecation alias now points at
  a real subcommand (#303).
- Developer-origin entitlement (#307) removes the evaluation/commercial gate
  friction on trusted machines; feature gates resolve through
  `license_developer_origin`.
- Agent guidance: AGENTS.md (repo + root) carries TBQ, disk-headroom, and
  one-canonical-Pi-package rules; the walkthrough/first-mission checklists
  point at the formal marker command, not manual JSON writes.

## Evidence

- `cargo check --workspace` clean on the consolidated tree (build-host run).
- Focused gates (retention, completion-events, pi_package, update, clippy
  `-D warnings`) green — serial test threads (parallel env-mutation races in
  the E6 fixture family are a known test-infra limitation, not product).
- Deployed extension full suite green with the day's fixes.

## Remaining (IR2+)

- API-side marker parity through `project_marker` (#243 IR2).
- RemoteWorkspaceBinding implementation (#89, design in docs/162).
- Spec 152 unified entitlement service (#119) on top of the #307 resolver.
- CallGraph/Workset programs (explicitly out of the 0.9.x scope).
