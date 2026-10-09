import assert from "node:assert/strict";
import { fileURLToPath } from "node:url";
import plugin from "../apps/focusa-awareness/index.ts";

// Controlled consumer fixtures, not live OpenClaw/model/authority acceptance.
const originalFetch = globalThis.fetch;
const oldRoot = process.env.FOCUSA_PROJECT_ROOT;
const oldContinuity = process.env.FOCUSA_CONTINUITY_ID;
let calls = 0;
function register(config: Record<string, unknown>) {
  let callback: any;
  plugin.register({
    pluginConfig: config,
    logger: { info() {}, warn() {} },
    on(event: string, handler: any) {
      assert.equal(event, "before_agent_start");
      callback = handler;
    },
  } as any);
  assert.ok(callback, "enabled plugin must register the consumer hook");
  return callback;
}
try {
  delete process.env.FOCUSA_PROJECT_ROOT;
  delete process.env.FOCUSA_CONTINUITY_ID;
  globalThis.fetch = (async () => {
    calls++;
    return new Response("", { status: 503 });
  }) as typeof fetch;
  const unbound = await register({ projectRoot: "", continuityId: "" })({}, { sessionKey: "fixture-unbound" });
  assert.equal(calls, 0, "missing lookup inputs must not fetch or infer scope");
  assert.match(unbound.prependContext, /Fallback context is advisory/);
  assert.match(unbound.prependContext, /not verified binding/);
  const configured = { projectRoot: fileURLToPath(new URL("../", import.meta.url)), continuityId: "fixture-continuity" };
  const unavailable = await register(configured)({}, { sessionKey: "fixture-unavailable" });
  assert.equal(calls, 1);
  assert.match(unavailable.prependContext, /current execution admission unverified/);
  assert.match(unavailable.prependContext, /independently admitted work continues/);
  globalThis.fetch = (async () => new Response(JSON.stringify({ rendered_card: "fixture advisory card" }), { status: 200 })) as typeof fetch;
  const available = await register(configured)({}, { sessionKey: "fixture-available" });
  assert.equal(available.prependContext, "fixture advisory card");
  console.log("PASS: 3 controlled awareness consumer fixtures; live harness/model behavior remains unverified");
} finally {
  globalThis.fetch = originalFetch;
  if (oldRoot === undefined) delete process.env.FOCUSA_PROJECT_ROOT;
  else process.env.FOCUSA_PROJECT_ROOT = oldRoot;
  if (oldContinuity === undefined) delete process.env.FOCUSA_CONTINUITY_ID;
  else process.env.FOCUSA_CONTINUITY_ID = oldContinuity;
}
