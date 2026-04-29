# Current Runtime Status

**Snapshot:** `v0.9.2-dev`  
**Repo head when written:** `093b7f8`  
**State:** current development build, not a finished product.

## Implemented in the present build

- Rust workspace with `focusa-core`, `focusa-api`, `focusa-cli`, and `focusa-tui` crates.
- Local daemon binary: `focusa-daemon` from `focusa-api`.
- CLI binary: `focusa` from `focusa-cli`.
- Pi extension under `apps/pi-extension` exposing 43 current `focusa_*` tools.
- Focusa skills under `.pi/skills/`, `apps/pi-extension/skills/`, and installed runtime copies under `/root/.pi/skills/`.
- Workpoint continuity APIs and Pi tools for checkpoint, current, resume, drift-check, active-object resolve, and evidence link.
- Metacognition APIs and Pi tools for capture, retrieve, reflect, adjust, evaluate, recent lists, loop-run, and doctor.
- Work-loop APIs and Pi tools for status, writer-status, control, context, checkpoint, and select-next.
- Tree/lineage/snapshot tools and lineage API surfaces.
- Focus State bounded write tools and scratchpad separation.
- State hygiene doctor/plan/apply surfaces; apply is approval-gated and non-destructive in the current build.

## Current proof files

- `docs/evidence/SPEC89_REAL_RELEASE_LIVE_PROOF_2026-04-28.md`
- `docs/evidence/FOCUSA_FOCUSED_SKILLS_AND_TOOL_DOCS_RELEASE_2026-04-28.md`
- `docs/evidence/FOCUSA_ONE_TOOL_PER_DOC_CORRECTION_2026-04-28.md`

## Current limits

- Focusa remains under active development.
- Some older docs contain design-direction details beyond current runtime behavior.
- State hygiene apply does not perform destructive cleanup in this build.
- Work-loop write endpoints require writer ownership semantics; writer conflicts are expected blocked states.
- Public docs should use snapshot/version language, not finished/frozen language.
