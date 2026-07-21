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
- normalized total input, cache-read, cache-write, and estimated re-billed tokens plus cache-read ratio; Pi `turn_end.message.usage` is normalized with top-level `turn_end.usage` compatibility;
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

A bounded 2026-07-21 provider run against published commit `492f753da19203891279a903ef11976b078fd81a` completed ten adjacent `openai-codex/gpt-5.6-sol` turns with exact responses and sub-TTL spacing. Turn 1 was cold; turns 2–10 each reported a 2,560-token cache read, producing a 9/9 post-cold hit rate. The immutable system hash remained `126cda8575d8f16bb7e0e9a6bf7500a4353ad2d32fc1d06df7ac7ce7a142b177` while all ten dynamic-slice hashes differed. Normalized observations matched provider usage and reported `cache_safe_tail` throughout.

The proof intentionally used an immediate fast-fail stub for non-telemetry Focusa API routes because independent daemon/lifecycle stalls had repeatedly withheld Pi settlement after completed provider responses. Health and cache telemetry capture remained active. Turn 10 persisted and emitted cache telemetry before that unrelated settlement path stalled. Durable summary: `docs/evidence/spec130a-prompt-cache-provider-proof-2026-07-21.json` (`sha256:7e2e0c8a3e5402dd94ac5dd46aec48e4b84b4b6b78fae75140b7149b1c69ef31`). Raw artifacts remain under `/private/tmp/spec130a-cache-proof-final/`.

## Remaining boundary

This establishes provider-side proof for the cache-prefix stabilization slice, not global installed-extension parity or complete Spec 130A conformance. Compaction eligibility, terminal no-op handling, cooldown/deduplication, cache-safe degradation under qualifying large misses, and the independent Pi lifecycle/daemon settlement stall retain their own acceptance gates. Provider eviction and unsupported cache behavior remain possible even when Focusa preserves the prefix.
