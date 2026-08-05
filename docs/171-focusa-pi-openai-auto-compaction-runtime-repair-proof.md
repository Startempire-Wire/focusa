# Focusa Pi/OpenAI Auto-Compaction Runtime Repair Proof

Date: 2026-08-05

Candidate base: `e5f7d4e72e1c`; repair: the commit containing this proof

Installed runtime: `/root/.pi/agent/extensions/focusa-runtime`

Reported provider/model: `openai-codex` / `gpt-5.6-sol`

## Verdict

**Fresh-process execution: PASS.** A fresh Pi extension loader discovers exactly one Focusa runtime. Its real installed `agent_settled` handler records pressure, requests native compaction, and calls `ctx.compact()` exactly once for `openai-codex` / `gpt-5.6-sol`.

**Already-running conversation activation: RELOAD REQUIRED.** Pi loads extension modules into a process. Editing files cannot replace handlers already loaded by this conversation. Run `/reload` once; the new `runtime_registration_verified` session entry is the activation receipt. Do not treat file hashes alone as live activation proof.

## Root cause

Pi discovers every direct one-level subdirectory under `~/.pi/agent/extensions` when it contains `index.ts`, `index.js`, or a Pi package manifest. Dot-prefixed directories are not excluded (`dist/core/extensions/loader.js`, `discoverExtensionsInDir`).

The installed extension directory contained 40 loadable `.focusa-*` rollback/backup directories. They were deployment backups, but Pi treated them as active extensions. This caused stale and duplicate Focusa modules—including `focusa-pi-bridge@0.9.137-dev`—to load alongside `/root/.pi/agent/extensions/focusa-runtime`. Previous source edits therefore did not prove which runtime owned the active event path.

## Repair

1. Preserved and relocated all 40 `.focusa-*` directories from:
   - `/root/.pi/agent/extensions/`
   - to existing disabled storage: `/root/.pi/agent/disabled-resources/20260801-013120/extensions/`
2. Kept one discoverable runtime:
   - `/root/.pi/agent/extensions/focusa-runtime/src/index.ts`
3. Added `runtime_registration_verified` on `session_start`, including:
   - exact extension build;
   - registration source;
   - native session ID.
4. Moved successful compaction outcome settlement into the authoritative `ctx.compact.onComplete` callback, before active-epoch teardown. `session_compact` no longer attempts duplicate/lost outcome settlement.
5. Added regression assertions to `apps/pi-extension/tests/auto-compaction-resilience.test.mjs`.

## Alignment

- Pi emits and awaits `agent_settled` after marking the run inactive (`dist/core/agent-session.js:309-323`).
- Pi's native auto-compaction reasons are `threshold` and `overflow`; manual compaction uses `manual` (`dist/core/agent-session.js:1490-1698`).
- Pi's extension API delegates `ctx.compact(options)` to the bound core compactor (`dist/core/extensions/runner.js:525-528`).
- Focusa Spec 130A requires one native invocation per epoch and one coordinator (`docs/130a-zero-waste-compaction-performance-addendum.md:205-215`, `:301-311`, `:387-397`).
- Focusa must preserve Pi's native threshold/overflow recovery and queued operator input (`docs/130a-zero-waste-compaction-performance-addendum.md:874-885`).

## Evidence

### Discovery repair

Before:

```text
loadable .focusa-* directories: 40
oldest stale build: focusa-pi-bridge@0.9.137-dev
```

After:

```text
focusa_count: 1
path: /root/.pi/agent/extensions/focusa-runtime/src/index.ts
loader errors: []
required handlers:
  agent_settled: true
  session_before_compact: true
  session_compact: true
  session_start: true
remaining .focusa-* directories in active extensions: 0
preserved .focusa-* directories in disabled storage: 40
```

### Real installed OpenAI handler execution

A fresh Node process used Pi's own `discoverAndLoadExtensions`, selected the real installed Focusa extension, supplied high pressure for `openai-codex` / `gpt-5.6-sol`, and invoked its registered `agent_settled` handler.

```json
{
  "schema": "focusa.openai_agent_settled_runtime_proof.v1",
  "extension": "/root/.pi/agent/extensions/focusa-runtime/src/index.ts",
  "provider": "openai-codex",
  "model": "gpt-5.6-sol",
  "compact_calls": 1,
  "event_kinds": [
    "pressure_observed",
    "native_compaction_requested",
    "attempt_started"
  ],
  "has_on_complete": true,
  "has_on_error": true
}
```

Result: **PASS**.

### Candidate regression gates

```text
node --test \
  apps/pi-extension/tests/auto-compaction-resilience.test.mjs \
  apps/pi-extension/tests/compaction-native-lifecycle.test.mjs

28 passed, 0 failed
```

The detached candidate has no `node_modules`; dependency-based TypeScript/long-run gates were not installed or claimed. No Cargo build ran.

### Installed hashes after repair

```text
1169c615aa456eaac76d04d11910dd08e7ec3c0e32468196902ddfd07705ab03  index.ts
7b0f07bd3417efce2f9c83e8a1db7952230d4ad38b2f52321af9e205bae16e5b  src/auto-compaction.ts
b13537ce30ffa7faafca9cdcab3e2a0c92778882bda00d6a448479e22f722c9a  src/session.ts
```

## Live acceptance after `/reload`

The first post-reload session activation must append a custom entry whose payload contains:

```json
{
  "schema": "focusa.auto_compaction_event.v1",
  "kind": "runtime_registration_verified",
  "extension_build": "focusa-pi-bridge@0.9.143",
  "registration_source": "apps/pi-extension/src/index.ts"
}
```

At the next qualifying pressure crossing, acceptance requires this ordered evidence:

1. `pressure_observed`
2. `native_compaction_requested`
3. `attempt_started`
4. one Pi native compaction lifecycle
5. `attempt_completed` or a persisted primary failure

Absence of `runtime_registration_verified` after `/reload` means activation failed and must not be reported as fixed.

## Operational prevention

Never keep loadable rollback copies directly under `~/.pi/agent/extensions`, even when dot-prefixed. Store backups under `disabled-resources` or another directory outside Pi's extension discovery root. Every runtime update must prove fresh discovery count = 1 before claiming activation.
