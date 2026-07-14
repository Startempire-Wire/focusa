import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import {
  proactiveCompactionDecision,
  registerAutoCompaction,
} from "../apps/pi-extension/src/auto-compaction.ts";
import {
  loadConfig,
  saveConfigOverrides,
} from "../apps/pi-extension/src/config.ts";

function assert(condition: any, message: string): asserts condition {
  if (!condition) throw new Error(message);
}

const below = proactiveCompactionDecision({
  tokens: 250_000,
  contextWindow: 372_000,
  percent: 67.2,
});
assert(!below.trigger, "below-threshold context triggered compaction");
assert(below.reserveTokens === 37_200, "adaptive reserve mismatch");
assert(below.triggerAtTokens === 256_000, "balanced/absolute trigger mismatch");

const pressured = proactiveCompactionDecision({
  tokens: 371_566,
  contextWindow: 372_000,
  percent: 99.88,
});
assert(pressured.trigger, "pressure context did not trigger compaction");
assert(pressured.reason === "context_pressure", "pressure reason mismatch");

const smallWindow = proactiveCompactionDecision({
  tokens: 112_000,
  contextWindow: 128_000,
  percent: 87.5,
});
assert(smallWindow.reserveTokens === 16_384, "minimum reserve mismatch");
assert(
  smallWindow.triggerAtTokens === 89_600,
  "small-window balanced trigger mismatch",
);
assert(smallWindow.trigger, "small-window pressure did not trigger");

const unknown = proactiveCompactionDecision({
  tokens: null,
  contextWindow: 372_000,
  percent: null,
});
assert(
  !unknown.trigger && unknown.reason === "unknown_usage",
  "unknown usage was not fail-open",
);

const disabled = proactiveCompactionDecision(
  { tokens: 371_566, contextWindow: 372_000, percent: 99.88 },
  {
    enabled: false,
    triggerPercent: 70,
    tokenCap: 256_000,
    reserveTokens: 16_384,
    reservePercent: 10,
    cooldownMs: 60_000,
  },
);
assert(
  !disabled.trigger && disabled.reason === "disabled",
  "disabled policy triggered compaction",
);

const customized = proactiveCompactionDecision(
  { tokens: 121_000, contextWindow: 200_000, percent: 60.5 },
  {
    enabled: true,
    triggerPercent: 60,
    tokenCap: 0,
    reserveTokens: 8_192,
    reservePercent: 5,
    cooldownMs: 30_000,
  },
);
assert(
  customized.triggerAtTokens === 120_000,
  "custom trigger percentage was ignored",
);
assert(customized.trigger, "custom compaction policy did not trigger");

const configRoot = mkdtempSync(
  join(tmpdir(), "focusa-auto-compaction-config-"),
);
try {
  const saved = saveConfigOverrides(configRoot, {
    autoCompactionEnabled: false,
    compactPct: 60,
    autoCompactionTokenCap: 192_000,
    autoCompactionReserveTokens: 32_768,
    autoCompactionReservePct: 15,
    autoCompactionCooldownMs: 120_000,
  });
  assert(
    saved.errors.length === 0,
    `saved compaction config was invalid: ${saved.errors.join(", ")}`,
  );
  const reloaded = loadConfig(configRoot).config;
  assert(!reloaded.autoCompactionEnabled, "enabled option did not persist");
  assert(reloaded.compactPct === 60, "trigger percentage did not persist");
  assert(
    reloaded.autoCompactionTokenCap === 192_000,
    "token cap did not persist",
  );
  assert(
    reloaded.autoCompactionReserveTokens === 32_768,
    "reserve tokens did not persist",
  );
  assert(
    reloaded.autoCompactionReservePct === 15,
    "reserve percent did not persist",
  );
  assert(
    reloaded.autoCompactionCooldownMs === 120_000,
    "cooldown did not persist",
  );
} finally {
  rmSync(configRoot, { recursive: true, force: true });
}

const handlers = new Map<string, Function[]>();
const pi = {
  on(name: string, handler: Function) {
    const current = handlers.get(name) || [];
    current.push(handler);
    handlers.set(name, current);
  },
};
registerAutoCompaction(pi as any);
assert(handlers.has("agent_end"), "agent_end fallback not registered");
assert(
  handlers.has("agent_settled"),
  "agent_settled idle-boundary fallback not registered",
);
assert(handlers.has("session_compact"), "session_compact reset not registered");

let usage: any = { tokens: 371_566, contextWindow: 372_000, percent: 99.88 };
let compactCalls = 0;
let compactOptions: any;
let idle = true;
const statuses: Array<[string, string | undefined]> = [];
const ctx = {
  isIdle: () => idle,
  getContextUsage: () => usage,
  compact(options: any) {
    compactCalls += 1;
    compactOptions = options;
  },
  ui: {
    setStatus(key: string, value: string | undefined) {
      statuses.push([key, value]);
    },
    notify() {},
  },
};
const invoke = async (name: string, ...args: any[]) => {
  for (const handler of handlers.get(name) || []) await handler(...args);
};
const waitForTimer = () => new Promise((resolve) => setTimeout(resolve, 10));

await invoke("session_start", {}, ctx);
await invoke("agent_end", {}, ctx);
await waitForTimer();
assert(compactCalls === 1, "pressure fallback did not invoke ctx.compact");
assert(
  typeof compactOptions.onComplete === "function",
  "completion callback missing",
);
assert(typeof compactOptions.onError === "function", "error callback missing");
assert(
  statuses.some(([key]) => key === "focusa-auto-compaction"),
  "status not exposed",
);

await invoke("agent_end", {}, ctx);
await waitForTimer();
assert(compactCalls === 1, "pending compaction was duplicated");
compactOptions.onComplete({});

// Regression: agent_end can fire while Pi still owns an automatic retry or
// queued continuation. The authoritative agent_settled boundary must retry the
// pressure check rather than silently dropping automatic compaction.
await invoke("session_start", {}, ctx);
usage = { tokens: 371_566, contextWindow: 372_000, percent: 99.88 };
idle = false;
await invoke("agent_end", {}, ctx);
await waitForTimer();
assert(compactCalls === 1, "busy agent_end should not compact");
idle = true;
await invoke("agent_settled", {}, ctx);
assert(
  compactCalls === 2,
  "settled idle boundary did not recover skipped compaction",
);
compactOptions.onComplete({});

// Native Pi compaction gets first chance: usage becomes unknown before the
// zero-delay fallback recheck, so Focusa must not issue a duplicate compact.
await invoke("session_start", {}, ctx);
usage = { tokens: 371_566, contextWindow: 372_000, percent: 99.88 };
await invoke("agent_end", {}, ctx);
usage = { tokens: null, contextWindow: 372_000, percent: null };
await waitForTimer();
assert(compactCalls === 2, "fallback duplicated native compaction");

console.log("PASS: Spec 130 automatic compaction fallback");
