# Focusa Battery Test Report — 2026-05-22

Operator request: run a broad battery across Focusa features/tools and identify broken, unresponsive, or incorrect behavior.

## Environment

- Project root: `/home/wirebot/focusa`
- Live daemon: `127.0.0.1:8787`, `FOCUSA_DATA_DIR=/home/wirebot/focusa/data/.focusa`
- Isolated daemon probes: `target/release/focusa-daemon` with temporary `FOCUSA_DATA_DIR` and high ports.
- Rust toolchain: available to root at `/root/.cargo/bin/cargo`; unavailable to `wirebot` user.

## Passing coverage

- `apps/pi-extension npx tsc --noEmit` — PASS.
- `cargo test --workspace --locked` with `CARGO_TARGET_DIR=/tmp/focusa-cargo-test-target` — PASS: 403 tests.
- `node scripts/validate-focusa-tool-contracts.mjs` — PASS: 58/58 documented contracts.
- `node scripts/validate-docs-runtime-parity.mjs` — PASS.
- `node scripts/validate-spec92-surface.mjs` — PASS.
- Agent/OpenClaw awareness validators — PASS.
- `node scripts/prove-focusa-tool-contracts-live.mjs --safe-fixtures --json` — PASS: live registry 58/58, payload equal.
- `node scripts/audit-focusa-tool-suite-safe.mjs --json` — PASS with no failures/warnings; hot GET probes 1–54ms.
- Spec96 static suite — PASS: 42/42 static gates run in this battery.
- Representative direct tools passed: project identity, project verify, work-loop read/preflight, state hygiene doctor/plan, resource status, tree head/recent snapshots/lineage, metacog capture/retrieve/reflect/loop, prediction record/evaluate, silent-session list.

## Broken / suspect findings

### F1 — `focusa_traverse` Pi tool wrapper is broken (HTTP 422)

- All direct `focusa_traverse` tool calls returned `blocked: request failed (422)`.
- Direct API works:
  - `POST /v1/traverse {"surface":"lineage","selector":"head","limit":10}` returns `status=completed`.
- Direct API fails when the body includes both aliases:
  - `include_payload:false` and `include_full_payload:false` together return 422.
- Root cause: Pi tool wrapper sends both alias fields unconditionally; Rust `TraverseRequest` defines `include_full_payload` with alias `include_payload`.
- Impact: official traversal tool unusable from Pi despite API route working.
- Tracking bead: child of `focusa-qi3t` titled `Bug: Pi focusa_traverse tool sends duplicate include payload aliases`.

### F2 — live daemon session is stuck closed

- Live `/v1/status` reports a closed `/root` session.
- Live `POST /v1/session/start` returns `{ "status": "accepted" }`, but `/v1/status` still shows the old closed session.
- Live `POST /v1/focus/push` then rejects with `{ "status":"rejected", "reason":"session_inactive", "session_status":"closed" }`.
- Isolated daemon with temp data accepts session start and focus push correctly.
- Impact: Focus State writes from Pi tools fall back with `No active/scoped Pi frame`; canonical evidence/workpoint linking also blocked by scope/trajectory gates.
- Tracking bead: child of `focusa-qi3t` titled `Bug: live daemon session_start accepted but session remains closed`.

### F3 — Focus State write tools are unavailable in current live session

- `focusa_intent`, `focusa_current_focus`, `focusa_next_step`, `focusa_open_question`, `focusa_recent_result`, `focusa_note`, `focusa_decide`, `focusa_constraint`, and `focusa_failure` all returned no active/scoped Pi frame and saved scratch fallback.
- Correlates with F2 live daemon closed-session state and unsafe `/root` Utility Card scope.
- Impact: Focus State is not canonical for this Pi logical session until session/frame recovery.

### F4 — clippy CI gate fails

- Command: `cargo clippy --workspace --locked -- -D warnings`.
- Failure: `clippy::large_enum_variant` in `crates/focusa-core/src/types.rs:2006`, `Action::EmitEvent { event: FocusaEvent }` makes enum ~1152 bytes.
- Cargo unit/integration tests pass, but CI clippy gate is red.
- Tracking bead: child of `focusa-qi3t` titled `Bug: clippy gate fails large Action enum variant`.

### F5 — validation scripts have stale path/expectation drift

- `node scripts/validate-compaction-fallbacks.mjs` fails on forbidden fallback markers in `apps/pi-extension/src/compaction.ts:223` (`|| "none"`).
- `node scripts/validate-skill-hygiene.mjs` fails importing `/home/wirebot/.nvm/versions/node/v22.22.0/.../pi-coding-agent/dist/core/skills.js`; actual Pi docs/package are under `/opt/node-v22.22.3-linux-x64/...` in this environment.
- `tests/spec89_tool_envelope_contract_test.sh` expects 43 `focusa_*` tools but current registry/docs expose 58.
- Tracking bead: child of `focusa-qi3t` titled `Bug: Focusa validation scripts rely on stale local tool paths or stale expectations`.

### F6 — LowMem workpoint stress mismatch

- `tests/spec96_lowmem_surgical_agent_stress_test.sh` mostly passes: LowMem forced, 58 tools callable, hot routes callable, cold route degradation explicit.
- It fails because Workpoint checkpoint returns `status=pending`, `failure_class=resource_exhausted`, no `workpoint_id`.
- Impact: checkpoint contract/test disagree under saturated LowMem.
- Tracking bead: child of `focusa-qi3t` titled `Bug: lowmem workpoint stress returns pending without workpoint_id`.

### F7 — ontology/status contract drift under bounded defaults

- `tests/ontology_world_contract_test.sh` fails 14 checks: bounded default `/v1/ontology/world` omits `working_sets` and `action_catalog`, and architecture slice lacks code-world members.
- `tests/behavioral_alignment_test.sh` fails `Ontology slice prompt shaping missing`.
- `tests/channel_separation_test.sh` fails because `/v1/status` has no active frame after isolated daemon seed pattern.
- Could be stale tests against newer summary-first defaults, or projection regression requiring explicit include flags.
- Tracking bead: child of `focusa-qi3t` titled `Bug: ontology/status contract gates drifted from bounded summary defaults`.

### F8 — Pi RPC driver contract currently fails

- `tests/pi_rpc_driver_contract_test.sh` route registration passes.
- Driver start returns HTTP 400 bad request; daemon-supervised Pi session is not visible; stop reports driver inactive.
- Needs request schema / route contract reconciliation.

### F9 — `scripts/api_contract_probe.py` is timeout-prone on reflection

- Probe timed out at `POST /v1/reflect/run` with default timeout.
- Direct isolated call later completed in ~4.22s, close to the script timeout window.
- Impact: API probe can be flaky/red even when route eventually responds.

### F10 — runtime scripts fail for `wirebot` due missing cargo

- Many runtime gates exit 127 under `as-user wirebot`: `cargo: command not found`.
- Root has cargo and can run with `CARGO_TARGET_DIR=/tmp/...` without writing build artifacts under `/home/wirebot/focusa`.
- Impact: local non-root CI parity is broken unless cargo is made available to `wirebot` or scripts use existing binaries.

## Resource note

Heavy battery temporarily pushed live Focusa ResourceMode to emergency/RSS-hard-exceeded. It later returned to normal, but peak RSS recorded ~835 MB and live root daemon remained CPU-active during observation. Hot routes still returned quickly.

## Evidence artifacts

- `/tmp/focusa-battery-live-audit.json`
- `/tmp/focusa-battery-live-tool-proof.json`
- `/tmp/focusa-battery-static-suite.log`
- `/tmp/focusa-battery-isolated-contracts.log`
- `/tmp/focusa-battery-isolated-fails-rerun.log`
- `/tmp/focusa-battery-spec96-runtime.log`
- `/tmp/focusa-battery-cargo-test.log`
- `/tmp/focusa-battery-cargo-clippy.log`
