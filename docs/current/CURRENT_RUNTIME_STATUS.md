# Current Runtime Status

**Snapshot:** `v0.9.11-dev`  
**Repo head when written:** `5e4f284`  
**State:** current development build, not a finished product.

## Implemented in the present build

- Rust workspace with `focusa-core`, `focusa-api`, `focusa-cli`, and `focusa-tui` crates.
- Local daemon binary: `focusa-daemon` from `focusa-api`.
- CLI binary: `focusa` from `focusa-cli`.
- Pi extension under `apps/pi-extension` exposing 47 current `focusa_*` tools.
- Focusa skills under `.pi/skills/`, `apps/pi-extension/skills/`, and installed runtime copies under `/root/.pi/skills/`.
- Workpoint continuity APIs and Pi tools for checkpoint, current, resume, drift-check, active-object resolve, and evidence link.
- Metacognition APIs and Pi tools for capture, retrieve, reflect, adjust, evaluate, recent lists, loop-run, and doctor.
- Work-loop APIs and Pi tools for status, writer-status, control, context, checkpoint, and select-next.
- Tree/lineage/snapshot tools and lineage API surfaces.
- Focus State bounded write tools and scratchpad separation.
- State hygiene doctor/plan/apply surfaces; apply is approval-gated and non-destructive in the current build.
- Agent-first polish surfaces: `focusa doctor`, `focusa status --agent`, `focusa continue`, `focusa release prove`, `focusa cleanup --safe`, token/cache doctors, hook telemetry, and error-empty recovery envelopes.
- Prediction loop API/CLI/Pi tools for bounded record/recent/evaluate/stats workflows.
- Workpoint session scope guard: checkpoint/resume packets carry `project_root`/`session_id` and reject cross-project resume packets with `rejected_scope_mismatch`.
- Pi replacement compaction uses intelligent related fallbacks instead of bare `none` fields.

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
cd /home/wirebot/focusa
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
- State hygiene apply does not perform destructive cleanup in this build.
- Work-loop write endpoints require writer ownership semantics; writer conflicts are expected blocked states.
- Public docs should use snapshot/version language, not finished/frozen language.
