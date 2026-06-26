# Focusa New Specs Implementation Order

Status: active execution plan  
Scope: Specs 109–114 plus Spec 112 installer blockers  
Rule: no compatible-platform support is deferred; no spec is considered done until its beads, tests, docs, and release evidence are complete.

## Gate Beads

| Wave | Gate Bead | Purpose |
|------|-----------|---------|
| Wave 0 | `focusa-9im1` | Spec 112 P0 installer/platform blockers and live-session P0 truth gaps |
| Wave 1 | `focusa-1ha1` | Spec 109 API authority plus Spec 114 Eval Ledger |
| Wave 2 | `focusa-zwkt` | Spec 111 bootstrap plus Spec 110 reminder behavior |
| Wave 3 | `focusa-3zay` | Spec 113 benchmark runner plus Spec 114 Phase 2 measured evidence |
| Wave 4 | `focusa-mlbp` | Spec 114 failure flywheel, snapshots, API, tests, CI |
| Wave 5 | `focusa-pskm` | Spec 114 bench.focusa.dev observatory UI |

## Primary Order

### Wave 0 — P0 Foundation and Live-Session Truth

Purpose: remove blockers that make all downstream agent/product evidence unreliable.

1. Critical installer/platform blockers from Spec 112:
   - `focusa-4c2t` real installer downloads Rust binaries, not Python stub.
   - `focusa-soer` SHA256SUMS + GPG/cosign signing.
   - `focusa-xvhd` Linux musl assets.
   - `focusa-pydn` Windows ARM64 asset.
   - `focusa-covz` macOS code signing / notarization / Gatekeeper handling.
   - `focusa-3cok` macOS LaunchAgent.
   - `focusa-foyr` Linux systemd daemon install.
   - `focusa-iwft` Windows PowerShell installer.
2. Existing P0 AX wrapper/session truth gaps must remain ahead of new P1 work.

### Wave 1 — Agent-First API Authority and Eval Ledger

Purpose: stabilize the API contract before bootstrap/reminder/benchmark depend on it.

1. Spec 109 child beads:
   - capabilities endpoint, schema index, OpenAPI, llms.txt.
   - request/response/error envelope.
   - materialization, preview/commit, idempotency, version checks.
   - resource controls, canonical routes, metadata registry, auth matrix, hardening.
2. Spec 114 Phase 1:
   - `focusa-n41d` append-only `/v1/evals/*` Eval Ledger API.

### Wave 2 — Agent Bootstrap and Tool-Layer Behavior

Purpose: make agents start with correct context and stay on Focusa-native tools.

1. Spec 111 in dependency order:
   - vocabulary → core module/types → routes → CLI → Pi integration → tool contracts → rendering/security/integrations.
2. Spec 110 in dependency order:
   - reminder text + shell classification → config/state/frequency gate → route/API → emission/hooks → telemetry/eval contract → docs/tests.

### Wave 3 — Benchmark Runner and Measured Evidence

Purpose: produce Focusa-vs-No-Focusa evidence from runnable tasks.

1. Spec 113 implementation:
   - `crates/focusa-bench` 150-task suite.
   - matched arms: `no_focusa`, `passive_focusa`, `tool_only_focusa`, `full_focusa`.
   - model matrix and multi-model scenario reporting.
   - Agent Power Index, Focusa Uplift Score, Operator Burden Reduction.
   - groundedness/tool/time-horizon/Pass^N metrics.
   - measured-claim report artifacts and static/CI smoke tests.
2. Spec 114 Phase 2:
   - MVP runner integration with Eval Ledger.

### Wave 4 — Failure Flywheel, Public Snapshots, and Release Gates

Purpose: turn failures into product improvements and publish public-safe proof.

1. Spec 114 Phase 3 failure-to-improvement candidate loop.
2. Spec 114 Phase 4 public-safe snapshots with redaction/hash chain/claim generation.
3. Spec 114 public data API.
4. Spec 114 static/live-safe tests.
5. Spec 114 CI gates for PR/nightly/release.

### Wave 5 — Public Observatory

Purpose: publish the public evidence surface after generated artifacts and gates exist.

1. Spec 114 Phase 5 `bench.focusa.dev` observatory UI.
2. Use cPanel/WordPress/static snapshot pattern for MVP.
3. Use Perpetua hybrid pattern only if dynamic replay/search/proof API is needed.

## Completion Rule

A wave is complete only when:

- all wave beads are closed or explicitly superseded by a linked bead;
- tests and static checks pass;
- docs/current references are updated;
- evidence is captured with a stable ref;
- git is committed and pushed.

## No-Omission Rule

Before closing an EPIC:

1. `bd list --status open --limit 0 | rg "Spec <number>|EPIC: Implement Spec <number>"` must show no remaining child tasks except intentionally superseded tasks.
2. The spec’s acceptance criteria must have corresponding tests or proof artifacts.
3. Public claims must be measured, not predicted.
