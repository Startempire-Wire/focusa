# Pi lifecycle prompt re-entrancy fallback — 2026-07-20

Issue: <https://github.com/Startempire-Wire/focusa/issues/14>

## Reproduced cause

Pi documents and types `ExtensionAPI.sendUserMessage()` as always triggering an agent turn. `deliverAs: "followUp"` controls delivery only after processing state is established; it does not serialize two callers that concurrently observe idle and enter `AgentSession.prompt()`. The second call can therefore reach `pi-agent-core` after `activeRun` is set and throw:

```text
Agent is already processing a prompt. Use steer() or followUp() to queue messages, or wait for completion.
```

Focusa had two lifecycle auto-prompt paths in `apps/pi-extension/src/session.ts`:

- the unbound-project nag;
- the project-identity bootstrap turn.

Both wrapped `sendUserMessage()` in `Promise.resolve(...).catch(...)`, but the normal extension API returns `void`, so that pattern cannot observe later processing failures and does not prevent the idle-to-active race.

## Patch

`apps/pi-extension/src/lifecycle-advisory.ts` now provides one coalesced lifecycle-advisory boundary:

- uses `sendMessage(..., { triggerTurn: false })`, never `sendUserMessage()`;
- shows the advisory immediately and supplies it to the next operator-triggered turn without starting an agent turn;
- skips headless UI delivery without issuing a prompt;
- appends `focusa.pi_lifecycle_advisory_outcome.v1` (`queued`, `skipped_headless`, or `failed`) to the Pi session;
- reports only the failure class, avoiding accidental secret-bearing exception text;
- preserves daemon telemetry for advisory outcome and `trigger_turn=false`;
- keeps existing per-session/project idempotency keys, preventing duplicate advisories.

`apps/pi-extension/src/session.ts` routes both lifecycle paths through this boundary. It contains no remaining `sendUserMessage()` call.

## Verification

```text
npm run check
  passed

npm run lint
  passed

npm run test:lifecycle-advisory
  PASS: queued delivery uses triggerTurn=false; headless skips; send failures persist a bounded outcome

node --preserve-symlinks-main <no-space-symlink>/apps/pi-extension/tests/*.test.mjs
  11/11 test files passed

npx prettier --check src/lifecycle-advisory.ts src/session.ts tests/session-lifecycle-advisory.test.mjs package.json
  passed

git diff --check -- apps/pi-extension
  passed
```

The direct aggregate test command initially exposed an unrelated existing test-harness defect: `spec104-pi-runtime-isolation.test.mjs` uses `URL.pathname` without decoding `%20`, so a worktree path containing spaces becomes nonexistent. Running the unchanged suite through a no-space main-module path proved all 11 tests; no product file was altered to hide the defect.

## Boundary

This patch removes Focusa's known lifecycle re-entrant `prompt()` callers. It does not claim that third-party extensions or future explicit prompt producers are serialized; any genuinely turn-triggering producer still requires an agent-state-aware arbiter with typed `steer`, `followUp`, or wait semantics.
