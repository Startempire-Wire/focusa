import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import test from "node:test";
import ts from "typescript";

function moduleUrl(source) {
  return `data:text/javascript;base64,${Buffer.from(source).toString("base64")}`;
}

function deferred() {
  let resolve;
  const promise = new Promise((done) => {
    resolve = done;
  });
  return { promise, resolve };
}

async function flushMicrotasks() {
  await Promise.resolve();
  await new Promise((resolve) => queueMicrotask(resolve));
  await Promise.resolve();
}

const source = readFileSync(
  fileURLToPath(new URL("../src/mission-canvas-widget.ts", import.meta.url)),
  "utf8"
);
const lifecycleGuardSource = readFileSync(
  fileURLToPath(new URL("../src/lifecycle-guard.ts", import.meta.url)),
  "utf8"
);
const lifecycleGuardUrl = moduleUrl(
  ts.transpileModule(lifecycleGuardSource, {
    compilerOptions: {
      module: ts.ModuleKind.ESNext,
      target: ts.ScriptTarget.ES2022,
    },
  }).outputText
);

const stateModule = `
const state = globalThis.__focusaMissionCanvasLifecycleState;
export const focusaFetch = (...args) => state.focusaFetch(...args);
export const getAttachmentRuntime = () => state.runtime;
export const currentProjectBindingDecision = () => ({ state: "BOUND" });
export const getActiveWorkpointPacket = () => null;
export const getContinuityId = () => state.continuityId;
export const getEffectiveFocusSnapshot = () => null;
export const getSessionCwd = () => state.projectRoot;
export const normalizeProjectRoot = (value) => String(value || "").replace(/\\/$/, "");
export const normalizeWorkpointResumePacketEnvelope = (value) => value?.packet || null;
export const refreshTrajectoryClarityLifecycle = (...args) => state.refreshTrajectory(...args);
export const setActiveWorkpointPacket = (packet) => state.activePackets.push(packet);
export const stampWorkpointPacketForCurrentPiSession = (packet) => packet;
`;
const configModule = `
export const resolveInteractionMode = () => ({ mode: "canvas-guided" });
`;
const railModule = `
export const workRailSnapshotFromPacket = () => ({});
export const renderWorkRailWidget = () => ["rail"];
`;
const scopedRefreshModule = `
const state = globalThis.__focusaMissionCanvasLifecycleState;
export const buildTruthfulScopedSurfaceSnapshot = (startupCwd) => ({
  selected_scope: state.projectRoot,
  startup_cwd: startupCwd,
  project: "bound",
  trajectory: "persisted",
  workpoint: "present",
  bead: "present",
  proof: "verified",
  proof_count: 1,
  stale_age_ms: 0,
  last_refresh_status: "observed",
});
export const currentScopedProjectRoot = () => state.projectRoot;
export const latestScopedStateChange = () => state.latestReceipt;
export const publishScopedStateChange = (input) => {
  const receipt = { ...input };
  state.latestReceipt = receipt;
  state.published.push(receipt);
  queueMicrotask(() => {
    for (const listener of state.listeners) listener(receipt);
  });
  return receipt;
};
export const scopedReceiptMatchesCurrentScope = () => true;
export const subscribeScopedStateChanges = (listener) => {
  state.listeners.add(listener);
  return () => state.listeners.delete(listener);
};
`;

const compiled = ts.transpileModule(
  source
    .replace('from "./state.js"', `from "${moduleUrl(stateModule)}"`)
    .replace('from "./config.js"', `from "${moduleUrl(configModule)}"`)
    .replace('from "./lifecycle-guard.js"', `from "${lifecycleGuardUrl}"`)
    .replace('from "./work-rail-widget.js"', `from "${moduleUrl(railModule)}"`)
    .replace('from "./scoped-surface-refresh.js"', `from "${moduleUrl(scopedRefreshModule)}"`),
  {
    compilerOptions: {
      module: ts.ModuleKind.ESNext,
      target: ts.ScriptTarget.ES2022,
    },
  }
).outputText;

const state = {
  projectRoot: "/home/wirebot/focusa",
  continuityId: "focusa-main",
  runtime: { startupReceptionistActive: false, currentAsk: { text: "fix issue 301" } },
  listeners: new Set(),
  latestReceipt: null,
  published: [],
  activePackets: [],
  fetchCalls: [],
  pendingWorkpoint: null,
  focusaFetch(path, options) {
    this.fetchCalls.push({ path, options });
    if (path === "/workpoint/resume" && this.pendingWorkpoint) return this.pendingWorkpoint;
    return Promise.resolve(null);
  },
  refreshTrajectory() {
    return Promise.resolve(null);
  },
};
globalThis.__focusaMissionCanvasLifecycleState = state;

const widget = await import(moduleUrl(compiled));

function context(name) {
  let stale = false;
  let staleReads = 0;
  const widgetCalls = [];
  return {
    ctx: {
      cwd: `/workspace/${name}`,
      get hasUI() {
        if (stale) {
          staleReads += 1;
          throw new Error("This extension ctx is stale after session replacement or reload.");
        }
        return true;
      },
      ui: {
        setWidget(...args) {
          widgetCalls.push(args);
        },
      },
    },
    markStale() {
      stale = true;
    },
    get staleReads() {
      return staleReads;
    },
    widgetCalls,
  };
}

test("Mission Canvas invalidates delayed and in-flight work across every replacement lifecycle", async () => {
  const nativeSetTimeout = globalThis.setTimeout;
  const nativeClearTimeout = globalThis.clearTimeout;
  const nativeSetInterval = globalThis.setInterval;
  const nativeClearInterval = globalThis.clearInterval;
  const timeouts = [];
  const intervals = new Set();

  globalThis.setTimeout = (callback) => {
    const handle = { callback, unref() {} };
    timeouts.push(handle);
    return handle;
  };
  globalThis.clearTimeout = (handle) => {
    handle.cancelled = true;
  };
  globalThis.setInterval = (callback) => {
    const handle = { callback, unref() {} };
    intervals.add(handle);
    return handle;
  };
  globalThis.clearInterval = (handle) => {
    intervals.delete(handle);
  };

  const handlers = new Map();
  widget.registerMissionCanvasWidget({
    on(event, handler) {
      handlers.set(event, handler);
    },
  });

  try {
    let active = context("startup");
    handlers.get("session_start")({ reason: "startup" }, active.ctx);
    await flushMicrotasks();
    assert.equal(state.listeners.size, 1);
    assert.equal(intervals.size, 1);

    for (const reason of ["reload", "new", "resume", "fork"]) {
      const old = active;
      const delayed = timeouts.at(-1);
      const oldInterval = [...intervals][0];
      handlers.get("session_shutdown")({ reason }, old.ctx);
      assert.equal(delayed.cancelled, true, `${reason} must cancel the startup poll timer`);
      assert.equal(state.listeners.size, 0, `${reason} must unsubscribe old listeners`);
      assert.equal(intervals.size, 0, `${reason} must clear old polling intervals`);
      old.markStale();

      active = context(reason);
      handlers.get("session_start")({ reason }, active.ctx);
      await flushMicrotasks();
      delayed.callback();
      oldInterval.callback();
      await flushMicrotasks();

      assert.equal(old.staleReads, 0, `${reason} delayed work must not read the old context`);
      assert.equal(state.listeners.size, 1);
      assert.equal(intervals.size, 1);
    }

    for (let index = 0; index < 128; index += 1) {
      const old = active;
      const delayed = timeouts.at(-1);
      handlers.get("session_shutdown")({ reason: "reload" }, old.ctx);
      assert.equal(delayed.cancelled, true);
      old.markStale();
      active = context(`stress-${index}`);
      handlers.get("session_start")({ reason: "reload" }, active.ctx);
      delayed.callback();
      await flushMicrotasks();
      assert.equal(old.staleReads, 0);
      assert.equal(state.listeners.size, 1, "replacement cycles must not leak subscriptions");
      assert.equal(intervals.size, 1, "replacement cycles must not leak polling intervals");
    }

    const old = active;
    const delayed = timeouts.at(-1);
    const workpoint = deferred();
    state.pendingWorkpoint = workpoint.promise;
    delayed.callback();
    await flushMicrotasks();
    assert.equal(state.fetchCalls.filter((call) => call.path === "/workpoint/resume").length, 1);

    handlers.get("session_shutdown")({ reason: "reload" }, old.ctx);
    old.markStale();
    active = context("after-in-flight-reload");
    handlers.get("session_start")({ reason: "reload" }, active.ctx);
    await flushMicrotasks();

    state.pendingWorkpoint = null;
    timeouts.at(-1).callback();
    await flushMicrotasks();
    await flushMicrotasks();
    assert.equal(
      state.fetchCalls.filter((call) => call.path === "/workpoint/resume").length,
      2,
      "the replacement session must not be blocked by the old in-flight poll"
    );

    workpoint.resolve({
      status: "completed",
      packet: {
        workpoint_id: "stale-workpoint",
        project_root: state.projectRoot,
        continuity_id: state.continuityId,
      },
    });
    await flushMicrotasks();
    await flushMicrotasks();

    assert.equal(old.staleReads, 0, "in-flight completion must not read the replaced context");
    assert.deepEqual(state.activePackets, [], "replaced in-flight results must not mutate active state");
    assert.equal(
      state.published.filter((receipt) => receipt.source === "poll").length,
      1,
      "only the replacement session may publish a poll result"
    );

    handlers.get("session_shutdown")({ reason: "quit" }, active.ctx);
    assert.equal(state.listeners.size, 0);
    assert.equal(intervals.size, 0);

    const defensive = context("defensive-containment");
    defensive.markStale();
    assert.doesNotThrow(() => widget.refreshMissionCanvasWidget(defensive.ctx));
    assert.equal(defensive.staleReads, 1, "unexpected stale calls remain contained instead of killing Pi");
  } finally {
    globalThis.setTimeout = nativeSetTimeout;
    globalThis.clearTimeout = nativeClearTimeout;
    globalThis.setInterval = nativeSetInterval;
    globalThis.clearInterval = nativeClearInterval;
    delete globalThis.__focusaMissionCanvasLifecycleState;
  }
});
