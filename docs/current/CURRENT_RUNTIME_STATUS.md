# Current Runtime Status

**Snapshot:** `v0.9.12-dev`
**Repo head when written:** `e8275d9`
**State:** current development build, not a finished product.

## Implemented in the present build

- Rust workspace with `focusa-core`, `focusa-api`, `focusa-cli`, and `focusa-tui` crates.
- Local daemon binary: `focusa-daemon` from `focusa-api`.
- CLI binary: `focusa` from `focusa-cli`.
- Pi extension under `apps/pi-extension` exposing 58 current `focusa_*` tools.
- Focusa skills under `.pi/skills/`, `apps/pi-extension/skills/`, and installed runtime copies under `${PI_SKILLS_DIR:-$HOME/.pi/skills}/`.
- Workpoint continuity APIs and Pi tools for checkpoint, current, resume, drift-check, active-object resolve, and evidence link.
- Metacognition APIs and Pi tools for capture, retrieve, reflect, adjust, evaluate, recent lists, loop-run, and doctor; evaluations persist as first-class records, successful evaluations promote learning back into retrieval memory, and API/CLI readback includes `evaluations/recent`.
- Work-loop APIs and Pi tools for status, writer-status, control, context, checkpoint, and select-next; `/v1/work-loop/health` exposes dispatch readiness, boundary reason, pause flags, and transport degradation while deep diagnostics remain opt-in.
- Tree/lineage/snapshot tools and lineage API surfaces.
- Focus State bounded write tools and scratchpad separation.
- State hygiene doctor/plan/apply surfaces; apply is approval-gated, non-destructive, and records an auditable Focus State note through `/v1/focus/update`.
- Agent-first polish surfaces: `focusa doctor`, `focusa status --agent`, `focusa continue`, `focusa release prove`, `focusa cleanup --safe`, token/cache doctors, hook telemetry, and error-empty recovery envelopes.
- Prediction loop API/CLI/Pi tools for bounded record/recent/evaluate/stats workflows; ontology memory-pipeline promotions now persist durable artifacts and create prediction follow-up records.
- Project/session isolation: frames and Workpoints carry `project_root + continuity_id`; cross-project and same-root/different-continuity packets reject, while temporal `session_id` changes preserve continuity only after hard gates match.
- Pi project-root resolution persists the last verified safe project folder across Pi sessions and reuses it when the next session starts from a broad cwd such as `/root`.
- Pi replacement compaction uses intelligent related fallbacks instead of bare `none` fields.
- Source-available licensing boundary is explicit: root `LICENSE.md`, `COMMERCIAL.md`, `TRADEMARKS.md`, `CONTRIBUTING.md`, support terms, and commercial/CLA templates are present; Cargo metadata points to `LICENSE.md` instead of MIT.

## Current proof files

- `docs/evidence/SPEC89_REAL_RELEASE_LIVE_PROOF_2026-04-28.md`
- `docs/evidence/FOCUSA_FOCUSED_SKILLS_AND_TOOL_DOCS_RELEASE_2026-04-28.md`
- `docs/evidence/FOCUSA_ONE_TOOL_PER_DOC_CORRECTION_2026-04-28.md`
- `docs/evidence/SPEC90_INITIAL_IMPLEMENTATION_2026-04-28.md`
- `docs/evidence/SPEC91_LIVE_TOOL_CONTRACT_PROOF_2026-04-28.md`
- `docs/evidence/PRODUCTION_RELEASE_MAC_APP_GITHUB_FIX_2026-04-28.md`
- `docs/current/WORKPOINT_SESSION_SCOPE_GUARD.md`
- `docs/current/PREDICTIVE_POWER_GUIDE.md`
- `docs/current/COMPACTION_FALLBACKS.md`
- `docs/evidence/SPEC92_FULL_ROLLOUT_PROOF_2026-04-28.md`

## Current verification commands

```bash
cd ${FOCUSA_PROJECT_ROOT:-<focusa-repo>}
node scripts/validate-focusa-tool-contracts.mjs
node scripts/prove-focusa-tool-contracts-live.mjs --safe-fixtures
cargo clippy --workspace -- -D warnings
./scripts/ci/run-spec-gates.sh
curl -sS --max-time 5 http://127.0.0.1:8787/v1/health | jq .
```

See [`PRODUCTION_RELEASE_COMMANDS.md`](PRODUCTION_RELEASE_COMMANDS.md) for full release, restart, GitHub, and cleanup commands.

## Current limits

- Focusa remains under active development.
- Some older docs contain design-direction details beyond current runtime behavior.
- State hygiene apply does not perform destructive cleanup in this build; approved applies append audit notes only.
- Work-loop write endpoints require writer ownership semantics; writer conflicts are expected blocked states.
- Public docs should use snapshot/version language, not finished/frozen language.
