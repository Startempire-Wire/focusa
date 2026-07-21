# Spec 130A Pi prompt-cache prefix stabilization — 2026-07-20

Issue: <https://github.com/Startempire-Wire/focusa/issues/13>

Normative owner: `docs/130a-zero-waste-compaction-performance-addendum.md`. This evidence covers the prompt-prefix/cache-observability slice only; it does not establish complete Spec 130A implementation or closure.

## Prior evidence

A bounded same-provider/same-model segment contained 19 assistant turns, 17 cache misses, and 829,940 re-billed prompt tokens. Fifteen misses reused exactly 18,944 tokens despite no idle interval exceeding five minutes. Pi already supplied a session-derived `prompt_cache_key`; the repeatable partial reuse instead implicated prefix layout.

The Pi extension previously changed cache-critical bytes in two places on each model call:

1. `before_agent_start` appended project identity, Workpoint content, visible-recap state, recent turns, a dynamic utility card, and WBM context to `systemPrompt`.
2. `context` prepended the current `[Focusa Focus Slice]` before all historical messages.

A cache key or longer retention window cannot make changing prefix bytes reusable.

## Guarded implementation

The implementation is guarded by `cacheSafePromptLayoutEnabled` (default `true`) and the environment rollback switch:

```text
FOCUSA_PI_CACHE_SAFE_PROMPT_LAYOUT=false
```

When enabled:

- `before_agent_start` appends only byte-stable Focusa behavioral and Workpoint authority laws;
- project identity, Workpoint values, recap state, recent turns, WBM data, Focus State, trajectory, and tool affordances remain dynamic;
- dynamic context is cloned onto the newest user message as its final text part rather than prepended before history;
- historical message objects and their ordering are preserved;
- duplicate Focus Slices are not appended to the same request;
- a missing user message receives one bounded user message at the end rather than at the front.

When disabled, the extension retains the prior dynamic-system/history-prepend layout as an explicit rollback path. Both layouts preserve project scope, Workpoint authority, constraints, evidence, blockers, and fail-closed behavior.

## Diagnostics and fallback

`CacheSafetyMonitor` emits bounded request/usage evidence:

- `stable_system_prefix_hash`;
- `history_prefix_hash` and historical message count;
- dynamic-slice hash and estimated tokens;
- selected injection position;
- provider/model, hashed session/cache key, layout mode, and idle duration;
- input, cache-read, cache-write, and estimated re-billed tokens plus cache-read ratio;
- typed miss reason: model change, TTL expiry, stable-system change, historical-prefix shift, compaction/branch reset, provider cache unavailable, or unknown provider miss.

After two consecutive large, same-provider/same-model, sub-five-minute qualifying misses with a stable cache-read plateau or structural prefix change, `cache_safe_degraded` retains only the current ask, verified scope/conflict posture, HLT/Trajectory posture, canonical Workpoint next action and blockers, critical constraints, evidence handles, and exact tool affordances. It suppresses optional Utility Card, recent-turn/WBM prose, ontology detail, historical/decayed context, general decision/result/failure prose, and noncritical receipts. Model selection resets comparison and degraded state.

## Verification

```text
npm run check
  passed

npm run lint
  passed

npm run test:cache-safe-context
  passed: immutable history, newest-user-tail injection, idempotency,
  prefix hashes, typed miss classification, ten adjacent same-model/sub-TTL
  fixture turns, two-miss plateau fallback transition, model discontinuity reset,
  guarded rollback configuration

all current-main apps/pi-extension/tests/*.test.mjs
  9/9 test files passed

git diff --check -- apps/pi-extension
  passed
```

The current-main integration was validated in the isolated no-space worktree `/private/tmp/focusa-spec137-publish` using the existing dependency tree; no dependency installation occurred.

## Remaining live proof

This is source and deterministic runtime-fixture proof, not installed-provider proof. Source integration, commit, or publication cannot substitute for installed behavior. Before issue closure, complete Spec 130A conformance, or release promotion, run at least ten adjacent real provider turns with one provider/model and sub-TTL spacing, then compare provider usage against the emitted prefix diagnostics. Provider eviction and unsupported cache behavior remain possible even when Focusa preserves the prefix.
