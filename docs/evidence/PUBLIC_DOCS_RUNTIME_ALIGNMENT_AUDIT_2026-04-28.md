# Public Docs Runtime Alignment Audit — 2026-04-28

## Scope

Operator asked for a thorough evaluation and comparison of the upgraded Focusa core, API, CLI, and public-facing docs, with GitHub-facing README updates that explain architecture, benefits, goals, why Focusa exists, how it is used, and what a user can expect from Focusa-enhanced agents.

Constraint applied: do not describe Focusa as finalized. Use current snapshot / logical version language because Focusa remains under development.

## Current logical version language

Public docs now use:

- `v0.9.0-dev` for the current public/runtime snapshot.
- “current snapshot”, “released runtime proof”, or “active development” instead of “finalized” or “pre-implementation”.

## Core evaluation

Code anchors:

- `crates/focusa-core/src/types.rs`
- `crates/focusa-core/src/reducer.rs`

Current core state includes:

- `FocusaState` with focus stack, Focus State, gate, memory, telemetry, ontology, Workpoint, and continuous work state.
- `FocusState` bounded slots for intent, current focus, decisions, constraints, failures, next steps, open questions, recent results, notes, and artifacts.
- `OntologyState` with proposal, verification, working-set refresh, and delta records.
- `WorkpointState` with active Workpoint ID, Workpoint records, resume events, drift events, and degraded fallbacks.
- Reducer-owned Workpoint events: checkpoint proposed/promoted/rejected, superseded, resume rendered, drift detected, evidence linked, degraded fallback recorded, action intent bound, verification linked.

Public README now reflects that Focusa is not just a design: the core reducer/state model is implemented and is the state authority.

## API evaluation

Code anchors:

- `crates/focusa-api/src/server.rs`
- `crates/focusa-api/src/routes/workpoint.rs`
- route modules under `crates/focusa-api/src/routes/`

Current API state includes live namespaces for health/status, Focus State, Workpoint continuity, work-loop, lineage, ontology, metacognition, threads, instances, capabilities, telemetry, memory/ECS/reference, gate, proposals, autonomy, cache, and tokens.

Real release proof found two live timing gaps and repaired them:

1. Workpoint checkpoint returned accepted before active state was visible to `/current` and `/resume`.
2. Workpoint evidence link returned accepted before evidence was visible in `resume_packet.verification_records`.

The API now waits for reducer-visible state before returning `accepted`, otherwise returns `pending` with retry guidance.

## CLI evaluation

Code anchors:

- `crates/focusa-cli/src/main.rs`
- `crates/focusa-cli/src/commands/`

Current CLI domains observed via `focusa --help`:

```text
start, stop, status, focus, stack, gate, memory, ecs, env, events,
turns, state, clt, lineage, autonomy, constitution, telemetry, rfm,
proposals, reflect, metacognition, ontology, skills, thread, export,
contribute, cache, workpoint, tokens, wrap
```

README now describes the CLI as an implemented operator/debug surface rather than future-only design.

## Pi/tooling evaluation

Code anchors:

- `apps/pi-extension/src/tools.ts`
- `apps/pi-extension/skills/focusa/SKILL.md`
- `.pi/skills/focusa/SKILL.md`

Current Pi extension exposes 43 `focusa_*` tools across Focus State, Workpoint, work-loop, tree/lineage/snapshots, metacognition, state hygiene, and tool doctor families. Public README now describes this tool surface and common `tool_result_v1` envelope behavior.

The Focusa skill conflict was repaired separately and the public README now includes the required frontmatter pattern and validation command.

## Public docs updated

- `README.md`
  - Rewritten as a current GitHub-facing overview.
  - Explains why Focusa exists in plain terms.
  - Describes architecture, benefits, user expectations, Workpoints, metacognition, state hygiene, CLI/API/Pi usage, current release proof, and development status.
  - Removes stale “pre-implementation”, “architecture locked”, GUI release, and finalized-style claims.

- `docs/README.md`
  - Updated opening status and core concept list for current runtime.
  - Replaced pre-implementation/MVP language with `v0.9.0-dev` active-development language.
  - Clarified GUI/TUI state.

- `docs/INDEX.md`
  - Renamed from finalized documentation set to current documentation snapshot.
  - Added `v0.9.0-dev` snapshot language.
  - Marked some older GUI/design docs as design direction rather than current runtime guarantee.

- `docs/PRD.md`
  - Changed status/version language from architecture-locked MVP to active-development `v0.9.0-dev` snapshot.
  - Updated augmentation bullets to include Workpoint continuity, evidence linking, metacognition, and work-loop controls.

- `docs/G1-detail-PRD-gen2-intermediate.md`
  - Reframed as an older design snapshot retained for architecture context.

- `docs/22-data-contribution.md`
  - Replaced finalized artifact language with reducer-owned/evidence-linked current artifact language.

- `docs/SPEC88_IMPLEMENTATION_DECOMPOSITION_2026-04-28.md`
  - Replaced finalized phrasing with maturation language.

- `docs/89-focusa-tool-suite-improvement-hardening-spec.md`
  - Renamed final success criteria to current success criteria.
  - Added real release proof evidence row.

## Stale public claims removed from key public docs

Verified no matches in the updated public set for:

- `Finalized`
- `finalized`
- `Pre-Implementation`
- `Specifications Complete`
- `Architecture Locked`
- `GitHub Releases`

Checked files:

- `README.md`
- `docs/README.md`
- `docs/INDEX.md`
- `docs/PRD.md`
- `docs/G1-detail-PRD-gen2-intermediate.md`
- `docs/22-data-contribution.md`
- `docs/SPEC88_IMPLEMENTATION_DECOMPOSITION_2026-04-28.md`
- `docs/89-focusa-tool-suite-improvement-hardening-spec.md`

## Validation commands

```bash
rg -n "Finalized|finalized|Pre-Implementation|Specifications Complete|Architecture Locked|GitHub Releases" README.md docs/README.md docs/INDEX.md docs/PRD.md docs/G1-detail-PRD-gen2-intermediate.md docs/22-data-contribution.md docs/SPEC88_IMPLEMENTATION_DECOMPOSITION_2026-04-28.md docs/89-focusa-tool-suite-improvement-hardening-spec.md
cargo run --quiet --bin focusa -- --help
```

## Result

The GitHub-facing README and the highest-risk public docs now describe the implemented Focusa snapshot more accurately: current architecture, active runtime surfaces, user benefits, operational expectations, and live proof are visible without claiming Focusa is finished.
