import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import vm from "node:vm";
import { createRequire } from "node:module";
import ts from "typescript";

const require = createRequire(import.meta.url);
const root = path.resolve(import.meta.dirname, "..");
const modulePath = path.join(root, "src", "cache-safe-context.ts");
const turnsPath = path.join(root, "src", "turns.ts");
const configPath = path.join(root, "src", "config.ts");
const source = fs.readFileSync(modulePath, "utf8");
const compiled = ts.transpileModule(source, {
  compilerOptions: {
    module: ts.ModuleKind.CommonJS,
    target: ts.ScriptTarget.ES2022,
  },
  fileName: modulePath,
}).outputText;
const module = { exports: {} };
vm.runInNewContext(compiled, {
  module,
  exports: module.exports,
  require,
  Date,
  Map,
  Math,
  String,
  JSON,
});
const { attachFocusSliceToNewestUser, buildCachePrefixSnapshot, CacheSafetyMonitor, normalizeCacheUsage } =
  module.exports;

const marker = "[Focusa Focus Slice — minimal applicable context]";

{
  assert.deepEqual(
    JSON.parse(JSON.stringify(normalizeCacheUsage({ input: 2_955, cacheRead: 2_560, cacheWrite: 0 }))),
    {
      inputTokens: 5_515,
      uncachedInputTokens: 2_955,
      cacheReadTokens: 2_560,
      cacheWriteTokens: 0,
    }
  );
  assert.deepEqual(
    JSON.parse(
      JSON.stringify(
        normalizeCacheUsage({
          inputTokens: 4_000,
          cacheReadInputTokens: 1_000,
          cacheCreationInputTokens: 500,
        })
      )
    ),
    {
      inputTokens: 5_500,
      uncachedInputTokens: 4_000,
      cacheReadTokens: 1_000,
      cacheWriteTokens: 500,
    }
  );
}

{
  const historicalUser = { role: "user", content: "old ask" };
  const historicalAssistant = { role: "assistant", content: "old answer" };
  const newestUser = { role: "user", content: [{ type: "text", text: "current ask" }] };
  const messages = [historicalUser, historicalAssistant, newestUser];
  const result = attachFocusSliceToNewestUser(messages, `${marker}\nCURRENT_ASK: current ask`);
  assert.equal(result.length, messages.length + 1);
  assert.equal(result[0], historicalUser);
  assert.equal(result[1], historicalAssistant);
  assert.equal(result[2], newestUser);
  assert.equal(result[2].content[0], newestUser.content[0]);
  assert.equal(result[2].content.length, 1, "operator-authored message must stay pristine");
  assert.equal(result[3].role, "user");
  assert.equal(result[3].content[0].text.startsWith(marker), true);
  assert.equal(messages[2].content.length, 1, "source messages must not be mutated");

  const repeated = attachFocusSliceToNewestUser(result, `${marker}\nchanged`);
  assert.equal(repeated.length, result.length, "the same request must not receive duplicate slices");
}

{
  const result = attachFocusSliceToNewestUser(
    [{ role: "assistant", content: "orphan response" }],
    `${marker}\nCURRENT_ASK: none`
  );
  assert.equal(result.length, 2);
  assert.equal(result[1].role, "user");
  assert.equal(result[1].content[0].text.startsWith(marker), true);
}

{
  const history = [
    { role: "user", content: "first" },
    { role: "assistant", content: "answer" },
    { role: "user", content: "second" },
  ];
  const a = buildCachePrefixSnapshot("stable-system", history, `${marker}\nA`, 1);
  const b = buildCachePrefixSnapshot("stable-system", history, `${marker}\nB`, 2);
  assert.equal(a.stableSystemPrefixHash, b.stableSystemPrefixHash);
  assert.equal(a.historyPrefixHash, b.historyPrefixHash);
  assert.notEqual(a.dynamicSliceHash, b.dynamicSliceHash);
}

{
  const monitor = new CacheSafetyMonitor();
  const sessionKey = "cache-session";
  const systemHashA = monitor.captureSystemPrompt(sessionKey, "stable-system");
  const base = [
    { role: "user", content: "old ask" },
    { role: "assistant", content: "old answer" },
  ];
  const misses = [];
  for (let turn = 0; turn < 10; turn += 1) {
    monitor.captureRequest(
      sessionKey,
      [
        ...base,
        ...Array.from({ length: turn }, (_, index) => ({ role: "assistant", content: `extra-${index}` })),
        { role: "user", content: `ask-${turn}` },
      ],
      `${marker}\nturn=${turn}`
    );
    misses.push(
      monitor.observeUsage({
        sessionKey,
        provider: "openai-codex",
        model: "gpt-5.2",
        inputTokens: 45_000,
        cacheReadTokens: 18_944,
        cacheWriteTokens: 0,
        layoutMode: "cache_safe_tail",
        observedAt: 1_000 + turn * 1_000,
      })
    );
  }
  assert.equal(misses[0].reason, "unknown_provider_miss");
  assert.equal(misses[0].idleDurationMs, null);
  assert.equal(misses[1].transitionedToDegraded, true);
  assert.equal(misses[1].cacheSafeDegraded, true);
  assert.equal(misses[1].consecutivePrefixMisses, 2);
  assert.equal(misses[1].idleDurationMs, 1_000);
  assert.equal(misses[1].layoutMode, "cache_safe_tail");
  assert.equal(misses[1].estimatedRebilledTokens, 45_000);
  assert.equal(misses[1].cacheWriteTokens, 0);
  assert.equal(typeof misses[1].sessionCacheKeyHash, "string");
  assert.equal(misses[9].reason, "unknown_provider_miss");
  assert.equal(misses[9].consecutivePrefixMisses, 10);
  assert.equal(monitor.isDegraded(sessionKey), true);

  monitor.resetForDiscontinuity(sessionKey);
  assert.equal(monitor.isDegraded(sessionKey), false);
  assert.equal(monitor.captureSystemPrompt(sessionKey, "stable-system"), systemHashA);
}

{
  const monitor = new CacheSafetyMonitor();
  const sessionKey = "ten-turn-cache-hit-proof";
  monitor.captureSystemPrompt(sessionKey, "stable-system");
  const ratios = [];
  for (let turn = 0; turn < 10; turn += 1) {
    monitor.captureRequest(
      sessionKey,
      [
        { role: "user", content: "stable historical ask" },
        { role: "assistant", content: "stable historical answer" },
        { role: "user", content: `new ask ${turn}` },
      ],
      `${marker}\nturn=${turn}`
    );
    const inputTokens = 15_000 + turn * 150;
    const cacheReadTokens = 90_000 + turn * 1_000;
    ratios.push(cacheReadTokens / (inputTokens + cacheReadTokens));
    const observation = monitor.observeUsage({
      sessionKey,
      provider: "openai-codex",
      model: "gpt-5.6-sol",
      inputTokens,
      cacheReadTokens,
      cacheWriteTokens: 0,
      layoutMode: "cache_safe_tail",
      observedAt: 10_000 + turn * 1_000,
    });
    assert.equal(observation.cacheSafeDegraded, false);
  }
  assert.equal(ratios.length, 10);
  assert.equal(
    ratios.every((ratio) => ratio >= 0.82),
    true
  );
}

{
  const monitor = new CacheSafetyMonitor();
  const sessionKey = "changed-prefix";
  monitor.captureSystemPrompt(sessionKey, "system-a");
  monitor.captureRequest(sessionKey, [{ role: "user", content: "one" }], `${marker}\none`);
  monitor.observeUsage({
    sessionKey,
    provider: "provider",
    model: "model",
    inputTokens: 1_000,
    cacheReadTokens: 9_000,
    cacheWriteTokens: 100,
    layoutMode: "cache_safe_tail",
    observedAt: 1_000,
  });
  monitor.captureSystemPrompt(sessionKey, "system-b");
  monitor.captureRequest(
    sessionKey,
    [
      { role: "user", content: "one" },
      { role: "assistant", content: "answer" },
      { role: "user", content: "two" },
    ],
    `${marker}\ntwo`
  );
  const observation = monitor.observeUsage({
    sessionKey,
    provider: "provider",
    model: "model",
    inputTokens: 45_000,
    cacheReadTokens: 18_944,
    cacheWriteTokens: 0,
    layoutMode: "cache_safe_tail",
    observedAt: 2_000,
  });
  assert.equal(observation.reason, "stable_system_prefix_changed");
}

const turnsSource = fs.readFileSync(turnsPath, "utf8");
assert.match(turnsSource, /\? "newest_user_turn_tail"\s*:\s*"legacy_history_prepend"/);
assert.match(turnsSource, /cache_safe_degraded/);
assert.match(turnsSource, /session_cache_key_hash/);
assert.match(turnsSource, /cache_write_tokens/);
assert.match(turnsSource, /estimated_rebilled_tokens/);
assert.match(turnsSource, /idle_duration_ms/);
assert.match(turnsSource, /layout_mode/);
assert.match(turnsSource, /normalizeCacheUsage\(ev\.usage \|\| ev\.message\?\.usage\)/);
assert.match(turnsSource, /CACHE_SAFE_DEGRADED_RETAINED_SECTIONS/);
const retainedSectionSet = turnsSource.match(
  /const CACHE_SAFE_DEGRADED_RETAINED_SECTIONS = new Set\(\[([\s\S]*?)\]\);/
)?.[1];
assert.ok(retainedSectionSet);
assert.match(retainedSectionSet, /current_ask/);
assert.match(retainedSectionSet, /current_ask_scope_verdict/);
assert.match(retainedSectionSet, /trajectory/);
assert.match(retainedSectionSet, /workpoint/);
assert.match(retainedSectionSet, /constraints/);
assert.match(retainedSectionSet, /ontology_evidence_handles/);
assert.match(retainedSectionSet, /tool_affordances/);
assert.doesNotMatch(
  retainedSectionSet,
  /artifacts|recent_results|verified_deltas|failures|decisions|historical_context|decayed_context/
);
assert.doesNotMatch(turnsSource, /injectRecentTurnsSlice/);
assert.equal((turnsSource.match(/systemPrompt\s*\+=/g) || []).length, 1);
assert.match(turnsSource, /cacheSafePromptLayoutEnabled === false/);
assert.equal((turnsSource.match(/attachCacheSafeFocusSlice\(event, contextMessages/g) || []).length, 3);

const configSource = fs.readFileSync(configPath, "utf8");
assert.match(configSource, /cacheSafePromptLayoutEnabled: true/);
assert.match(configSource, /FOCUSA_PI_CACHE_SAFE_PROMPT_LAYOUT: "cacheSafePromptLayoutEnabled"/);

console.log("PASS: stable prefix, newest-user-tail injection, miss classification, and degraded fallback");
