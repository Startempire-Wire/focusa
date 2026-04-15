import { registerSession } from "../apps/pi-extension/src/session.ts";
import { registerCompaction } from "../apps/pi-extension/src/compaction.ts";
import { registerTurns } from "../apps/pi-extension/src/turns.ts";
import { S, getFocusState } from "../apps/pi-extension/src/state.ts";

type Handler = (event: any, ctx: any) => any | Promise<any>;

function assert(cond: any, msg: string) {
  if (!cond) throw new Error(msg);
}

async function waitFor(check: () => Promise<boolean>, label: string, timeoutMs = 4000) {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    if (await check()) return;
    await new Promise((r) => setTimeout(r, 100));
  }
  throw new Error(`timeout waiting for ${label}`);
}

class MockPi {
  handlers = new Map<string, Handler[]>();
  entries: any[] = [];
  sessionName = "";
  statuses: Record<string, string> = {};
  notifications: string[] = [];

  on(name: string, handler: Handler) {
    const list = this.handlers.get(name) || [];
    list.push(handler);
    this.handlers.set(name, list);
  }

  async emit(name: string, event: any, ctx: any) {
    const list = this.handlers.get(name) || [];
    let last: any;
    for (const handler of list) last = await handler(event, ctx);
    return last;
  }

  getFlag(_flag: string) { return false; }
  setSessionName(name: string) { this.sessionName = name; }
  appendEntry(customType: string, data: any) { this.entries.push({ type: "custom", customType, data }); }
  exec(_cmd: string, _args: string[], _opts?: any) { return Promise.resolve({ code: 0, stdout: "", stderr: "" }); }
  getActiveTools() { return []; }
  setActiveTools(_tools: any[]) {}
  registerCommand() {}
  registerShortcut() {}
  registerFlag() {}
  registerMessageRenderer() {}
  registerProvider() {}
  sendMessage(_message: any, _options?: any) { return Promise.resolve(); }
  ui = {
    notify: (msg: string) => { this.notifications.push(msg); },
    setStatus: (key: string, value: string) => { this.statuses[key] = value; },
    setWidget: (_key: string, _value: any) => {},
  };
}

async function main() {
  const base = process.env.FOCUSA_BASE_URL;
  if (!base) throw new Error("FOCUSA_BASE_URL required");

  Object.assign(S, {
    pi: null,
    cfg: {
      enabled: true,
      warnPct: 50,
      compactPct: 70,
      hardPct: 85,
      contextStatusMode: "actionable",
      cooldownMs: 180000,
      maxCompactionsPerHour: 8,
      minTurnsBetweenCompactions: 3,
      compactInstructions: "Preserve intent, decisions, constraints, next_steps, failures.",
      externalizeThresholdBytes: 8192,
      externalizeThresholdLines: 200,
      focusaApiBaseUrl: `${base}/v1`,
      workLoopPreset: "balanced",
    },
    focusaAvailable: false,
    activeFrameId: null,
    activeFramePromise: null,
    activeFrameTitle: "",
    activeFrameGoal: "",
    sessionFrameKey: "",
    sessionCwd: "",
    wbmEnabled: false,
    wbmDeep: false,
    wbmNoCatalogue: false,
    localDecisions: [],
    localConstraints: [],
    localFailures: [],
    lastFocusSnapshot: { decisions: [], constraints: [], failures: [], intent: "", currentFocus: "" },
    cataloguedDecisions: [],
    cataloguedFacts: [],
    totalCompactions: 0,
    turnCount: 0,
    healthInterval: null,
    healthBackoffMs: 30000,
    healthFailCount: 0,
    outageStart: null,
    currentAsk: null,
    queryScope: null,
    excludedContext: null,
  });

  const pi = new MockPi();
  registerSession(pi as any);
  registerCompaction(pi as any);
  registerTurns(pi as any);

  const mkCtx = () => ({
    cwd: "/home/wirebot/focusa",
    ui: pi.ui,
    sessionManager: { getEntries: () => pi.entries },
  });

  await pi.emit("session_start", { sessionId: "pi-runtime-1", entries: pi.entries }, mkCtx());
  assert(S.activeFrameId, "session_start did not create Pi frame");

  const startupStack = await fetch(`${base}/v1/focus/stack`).then((r) => r.json());
  const startupFrame = startupStack?.stack?.frames?.find((f: any) => f.id === S.activeFrameId);
  assert(startupFrame?.title === "Pi: focusa", `expected startup fallback frame, got ${startupFrame?.title}`);

  await pi.emit("input", { text: "Focusa: ✅ Connected Frame: 019ddead Title: Pi: root Goal: Work on root WBM: off Turns: 3838 Config: warn=60%" }, mkCtx());
  await new Promise((r) => setTimeout(r, 200));
  const stillFallbackStack = await fetch(`${base}/v1/focus/stack`).then((r) => r.json());
  const stillFallbackActive = stillFallbackStack?.stack?.frames?.find((f: any) => f.status === "active");
  assert(stillFallbackActive?.title === "Pi: focusa", `status blob should not rescope frame: ${stillFallbackActive?.title}`);

  await pi.emit("input", { text: "restarted again, still wrong: [focusa-context] Focusa Context Rendered live from focusa-pi-bridge current state. Current Focus Frame: Pi: root Goal: Work on root" }, mkCtx());
  await new Promise((r) => setTimeout(r, 200));
  const stillFallbackStack2 = await fetch(`${base}/v1/focus/stack`).then((r) => r.json());
  const stillFallbackActive2 = stillFallbackStack2?.stack?.frames?.find((f: any) => f.status === "active");
  assert(stillFallbackActive2?.title === "Pi: focusa", `embedded focusa-context blob should not rescope frame: ${stillFallbackActive2?.title}`);
  assert(S.currentAsk?.kind === "meta", `embedded focusa-context blob should classify as meta, got ${S.currentAsk?.kind}`);
  assert(S.currentAsk?.text === "", `currentAsk should be cleared for stripped focusa payload, got ${S.currentAsk?.text}`);

  await pi.emit("input", { text: "Implement Pi runtime authority proof" }, mkCtx());
  await waitFor(async () => {
    const stack = await fetch(`${base}/v1/focus/stack`).then((r) => r.json());
    const active = stack?.stack?.frames?.find((f: any) => f.status === "active");
    return Boolean(active?.title?.includes("Implement Pi runtime authority proof"));
  }, "post-input frame rescope");

  const rescopedStack = await fetch(`${base}/v1/focus/stack`).then((r) => r.json());
  const rescopedActive = rescopedStack?.stack?.frames?.find((f: any) => f.status === "active");
  assert(rescopedActive?.title?.includes("Implement Pi runtime authority proof"), `active frame was not rescoped: ${rescopedActive?.title}`);
  assert(rescopedActive?.goal === "Implement Pi runtime authority proof", `active frame goal was not rescoped: ${rescopedActive?.goal}`);

  await waitFor(async () => {
    const status = await fetch(`${base}/v1/status`).then((r) => r.json());
    return status?.session?.status === "active";
  }, "active session after session_start");
  const status1 = await fetch(`${base}/v1/status`).then((r) => r.json());

  const frameId = S.activeFrameId;
  const update = await fetch(`${base}/v1/focus/update`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      frame_id: frameId,
      turn_id: "pi-runtime-turn-1",
      delta: {
        decisions: ["Authoritative runtime decision"],
        constraints: ["Authoritative runtime constraint"],
        failures: ["Authoritative runtime failure"],
        intent: "Runtime proof mission",
        current_focus: "Verify compact/resume/switch authority",
      },
    }),
  }).then((r) => r.json());
  assert(update?.status === "accepted", `focus/update failed: ${JSON.stringify(update)}`);

  const hydrated = await getFocusState();
  assert(hydrated?.fs?.current_focus === "Verify compact/resume/switch authority", `getFocusState did not hydrate live ASCC current_focus: ${hydrated?.fs?.current_focus}`);
  assert(Array.isArray(hydrated?.fs?.decisions) && hydrated.fs.decisions.includes("Authoritative runtime decision"), "getFocusState did not hydrate live ASCC decisions");

  await pi.emit("turn_start", { sessionId: "pi-runtime-1" }, mkCtx());
  await pi.emit("turn_end", {
    sessionId: "pi-runtime-1",
    usage: { inputTokens: 123, outputTokens: 45, cacheReadInputTokens: 6, cacheCreationInputTokens: 7 },
    message: { role: "assistant", content: [{ type: "text", text: "Runtime authority turn output" }] },
  }, mkCtx());
  await waitFor(async () => {
    const events = await fetch(`${base}/v1/events/recent?limit=20`).then((r) => r.json());
    const match = (events?.events || []).find((e: any) => e?.type === "TurnCompleted" && e?.turn_id === "pi-turn-1");
    return !!match && match.prompt_tokens === 123 && match.completion_tokens === 45 && match.assistant_output === "Runtime authority turn output";
  }, "turn completion event with tokens and output");

  await pi.emit("session_before_compact", { sessionId: "pi-runtime-1" }, mkCtx());
  const stateEntry = [...pi.entries].reverse().find((e) => e.customType === "focusa-state");
  assert(stateEntry, "session_before_compact did not persist focusa-state entry");
  assert(stateEntry.data.authoritativeDecisions?.includes("Authoritative runtime decision"), `persisted entry missing authoritative decision: ${JSON.stringify(stateEntry.data)}`);
  assert(stateEntry.data.intent === "Runtime proof mission", `persisted entry missing intent: ${JSON.stringify(stateEntry.data)}`);
  assert(stateEntry.data.currentFocus === "Verify compact/resume/switch authority", `persisted entry missing currentFocus: ${JSON.stringify(stateEntry.data)}`);

  await pi.emit("session_compact", {
    sessionId: "pi-runtime-1",
    compactionEntry: {
      details: {
        readFiles: ["apps/pi-extension/src/session.ts"],
        modifiedFiles: ["apps/pi-extension/src/compaction.ts"],
      },
    },
  }, mkCtx());
  await waitFor(async () => {
    const stack = await fetch(`${base}/v1/focus/stack`).then((r) => r.json());
    const frame = stack?.stack?.frames?.find((f: any) => f.id === frameId);
    const artifacts = frame?.focus_state?.artifacts || [];
    return artifacts.some((a: any) => a?.path_or_id === "apps/pi-extension/src/compaction.ts" && a?.kind === "file");
  }, "compaction artifacts persisted");

  await pi.emit("session_before_switch", { sessionId: "pi-runtime-1" }, mkCtx());
  await waitFor(async () => {
    const status = await fetch(`${base}/v1/status`).then((r) => r.json());
    return status?.session?.status === "closed";
  }, "closed session after session_before_switch");

  await pi.emit("session_switch", { sessionId: "pi-runtime-2", entries: pi.entries }, mkCtx());
  assert(S.activeFrameId, "session_switch did not recreate Pi frame");
  await waitFor(async () => {
    const status = await fetch(`${base}/v1/status`).then((r) => r.json());
    return status?.session?.status === "active";
  }, "active session after session_switch");
  const status2 = await fetch(`${base}/v1/status`).then((r) => r.json());

  const stack = await fetch(`${base}/v1/focus/stack`).then((r) => r.json());
  assert(stack?.stack?.frames?.length > 0, `focus stack empty after session_switch: ${JSON.stringify(stack)}`);

  await pi.emit("session_shutdown", { sessionId: "pi-runtime-2" }, mkCtx());
  await waitFor(async () => {
    const status = await fetch(`${base}/v1/status`).then((r) => r.json());
    return status?.session?.status === "closed";
  }, "closed session after session_shutdown");

  console.log("runtime authority proof passed");
}

main().catch((err) => {
  console.error(err.stack || String(err));
  process.exit(1);
});
