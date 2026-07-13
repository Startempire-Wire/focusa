import {
  proactiveCompactionDecision,
  registerAutoCompaction,
} from "../apps/pi-extension/src/auto-compaction.ts";

function assert(condition: any, message: string): asserts condition {
  if (!condition) throw new Error(message);
}

const below = proactiveCompactionDecision({
  tokens: 300_000,
  contextWindow: 372_000,
  percent: 80.65,
});
assert(!below.trigger, "below-threshold context triggered compaction");
assert(below.reserveTokens === 37_200, "adaptive reserve mismatch");
assert(below.triggerAtTokens === 334_800, "adaptive trigger mismatch");

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
assert(smallWindow.trigger, "small-window pressure did not trigger");

const unknown = proactiveCompactionDecision({
  tokens: null,
  contextWindow: 372_000,
  percent: null,
});
assert(!unknown.trigger && unknown.reason === "unknown_usage", "unknown usage was not fail-open");

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
assert(handlers.has("session_compact"), "session_compact reset not registered");

let usage: any = { tokens: 371_566, contextWindow: 372_000, percent: 99.88 };
let compactCalls = 0;
let compactOptions: any;
const statuses: Array<[string, string | undefined]> = [];
const ctx = {
  isIdle: () => true,
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
assert(typeof compactOptions.onComplete === "function", "completion callback missing");
assert(typeof compactOptions.onError === "function", "error callback missing");
assert(statuses.some(([key]) => key === "focusa-auto-compaction"), "status not exposed");

await invoke("agent_end", {}, ctx);
await waitForTimer();
assert(compactCalls === 1, "pending compaction was duplicated");
compactOptions.onComplete({});

// Native Pi compaction gets first chance: usage becomes unknown before the
// zero-delay fallback recheck, so Focusa must not issue a duplicate compact.
await invoke("session_start", {}, ctx);
usage = { tokens: 371_566, contextWindow: 372_000, percent: 99.88 };
await invoke("agent_end", {}, ctx);
usage = { tokens: null, contextWindow: 372_000, percent: null };
await waitForTimer();
assert(compactCalls === 1, "fallback duplicated native compaction");

console.log("PASS: Spec 130 automatic compaction fallback");
