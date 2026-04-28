# Focusa Tool Stress Evidence — 2026-04-28

## Scope

Full live stress pass across Focusa API, CLI, and Pi bridge tool surfaces after operator request to stress all Focusa tools, observe feedback, and harden failures.

## Failures Found

1. **Pi Focus State write bridge false-offline**
   - Symptom: `focusa_recent_result`, `focusa_note`, `focusa_constraint`, and `focusa_failure` reported `Focusa offline` while `/v1/health` and Workpoint tools were reachable.
   - Cause: stale `S.focusaAvailable=false` and health probe result could veto `/v1/focus/update` before the authoritative write attempt.
   - Hardening: `pushDelta()` now treats health recovery as probe-only and lets `/v1/focus/update` determine write availability; successful writes set `S.focusaAvailable=true`; catch path reports offline only when follow-up health check is also offline.

2. **Healthcheck restart storm under load**
   - Symptom: stress suite saw connection resets/refused responses and daemon PIDs changed during sequential route probes.
   - Cause: `/usr/local/bin/focusa-daemon-healthcheck.sh` ran every 30s with one 3s `/v1/health` probe and restarted the daemon on transient route pressure.
   - Hardening: healthcheck now uses 3 attempts, 5s default timeout, `/v1/status` fallback, and only restarts after all attempts fail.

3. **Lineage path route pathological traversal**
   - Symptom: `/v1/lineage/path/<head>` timed out at 30s and daemon CPU stayed high.
   - Cause: every parent hop used `s.clt.nodes.iter().find(...)`, producing effectively quadratic traversal over a large CLT.
   - Hardening: route now indexes CLT nodes by id, tracks visited ids, caps depth at 512, and returns `truncated` metadata.

4. **Stress harness assumptions too narrow**
   - Symptom: harness rejected valid `/v1/lineage/head` string shape and work-loop `ok:true` responses.
   - Hardening: harness accepts string/object head shapes, uses active work-loop writer identity, and accepts current work-loop response envelopes.

## Validation

- `cd apps/pi-extension && ./node_modules/.bin/tsc --noEmit` ✅
- `cargo check -p focusa-api --target-dir /tmp/focusa-cargo-target` ✅
- `cargo build --release -p focusa-api -p focusa-cli --target-dir /home/wirebot/focusa/target` ✅
- `systemctl restart focusa-daemon.service` ✅
- Targeted perf probes after fix:
  - `/v1/lineage/path/<head>`: HTTP 200 in ~0.08s ✅
  - `/v1/focus/snapshots`: HTTP 200 in ~0.002s ✅
  - `/v1/ontology/world`: HTTP 200 in ~0.08s ✅
  - `/v1/ontology/slices`: HTTP 200 in ~0.012s ✅
  - `/v1/work-loop/status`: HTTP 200 in ~0.19s ✅
- `./tests/focusa_tool_stress_test.sh`: `passed=36 failed=0` ✅

## Covered Surfaces

- Health/status/focus stack/focus update
- Workpoint checkpoint/current/resume/drift-check/idempotency
- Lineage head/tree/path
- Snapshot create/recent/diff
- Metacognition capture/retrieve/reflect/recent/adjust/evaluate
- Work-loop status/context/checkpoint/pause/resume/stop
- Ontology primitives/world/slices
- CLI workpoint current/resume/drift-check
- Pi tools sampled live: tree head, recent snapshots, metacog doctor, work-loop status, lineage tree, Focus State write tools

## Operational Note

Current Pi process successfully recorded Focus State writes after bridge recovery. A future Pi reload is still recommended so the running extension definitely picks up the source-level `pushDelta()` and `focusaFetch()` hardening.
