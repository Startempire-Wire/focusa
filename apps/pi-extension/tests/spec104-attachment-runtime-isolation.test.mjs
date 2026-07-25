import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const outDir = mkdtempSync(join(tmpdir(), "focusa-pi-runtime-"));
try {
  execFileSync(
    "./node_modules/.bin/tsc",
    ["-p", "tsconfig.json", "--outDir", outDir, "--noEmit", "false", "--module", "ES2022"],
    { cwd: new URL("..", import.meta.url), stdio: "pipe" }
  );
  writeFileSync(join(outDir, "package.json"), '{"type":"module"}\n');
  const state = await import(pathToFileURL(join(outDir, "state.js")).href);
  const scopedState = await import(pathToFileURL(join(outDir, "scoped-state.js")).href);
  state.attachmentRuntimeRegistry.reset();
  const keyA = state.makeAttachmentKey({
    projectRoot: "/tmp/project-a",
    continuityId: "cont-a",
    sessionId: "session-a",
    attachmentId: "attach-a",
  });
  const keyB = state.makeAttachmentKey({
    projectRoot: "/tmp/project-b",
    continuityId: "cont-b",
    sessionId: "session-b",
    attachmentId: "attach-b",
  });
  state.attachmentRuntimeRegistry.bindSessionAttachment(keyA);
  assert.equal(
    state.attachmentRuntimeRegistry.boundSessionAttachment("session-a")?.workstream.continuity_id,
    "cont-a",
    "typed project attachment must bind to its Pi session"
  );
  const unsafeKey = state.makeAttachmentKey({
    projectRoot: "/root",
    continuityId: "cont-unsafe",
    sessionId: "session-a",
    attachmentId: "attach-unsafe",
  });
  state.attachmentRuntimeRegistry.bindSessionAttachment(unsafeKey);
  assert.equal(
    state.attachmentRuntimeRegistry.boundSessionAttachment("session-a")?.workstream.continuity_id,
    "cont-a",
    "unsafe broad-root attachment must not replace verified session binding"
  );
  const extensionSessionBinding = new scopedState.PiExtensionSessionBinding();
  extensionSessionBinding.bind(keyA);
  assert.equal(
    extensionSessionBinding.resolve()?.workstream.continuity_id,
    "cont-a",
    "scope-less tools must reuse the latest verified attachment in one Pi extension instance"
  );
  extensionSessionBinding.bind(keyB);
  assert.equal(
    extensionSessionBinding.resolve()?.workstream.root_scope.root_path,
    "/tmp/project-b",
    "an explicit typed project switch must replace the extension-instance attachment"
  );
  extensionSessionBinding.clear();
  assert.equal(extensionSessionBinding.resolve(), undefined);

  assert.deepEqual(
    scopedState.attachmentRoutingHints({
      toolName: "focusa_workpoint_resume",
      input: {
        project_root: "/tmp/project-a",
        continuity_id: "cont-a",
      },
    }),
    {
      sessionId: undefined,
      projectRoot: "/tmp/project-a",
      continuityId: "cont-a",
    },
    "tool_call event.input must supply typed attachment routing hints before execution"
  );

  await state.runWithAttachmentRuntime(keyA, async () => {
    const runtime = state.getAttachmentRuntime();
    runtime.currentAsk = { text: "ask-a" };
    runtime.localDecisions = ["decision-a"];
    runtime.localConstraints = ["constraint-a"];
    runtime.localFailures = ["failure-a"];
    runtime.activeFrameId = "frame-a";
    runtime.continuityId = "cont-a";
    state.setActiveWorkpointPacket({
      project_root: "/tmp/project-a",
      continuity_id: "cont-a",
      mission: "wp-a",
    });
    state.setLastTrajectoryClarity({
      project_root: "/tmp/project-a",
      continuity_id: "cont-a",
      long_term_goal: "traj-a",
    });
    state.setLastProjectIdentity({ project_root: "/tmp/project-a", canonical_name: "identity-a" });
    state.setTurnCount(7);
  });
  await state.runWithAttachmentRuntime(keyB, async () => {
    const runtime = state.getAttachmentRuntime();
    runtime.currentAsk = { text: "ask-b" };
    runtime.localDecisions = ["decision-b"];
    runtime.localConstraints = ["constraint-b"];
    runtime.localFailures = ["failure-b"];
    runtime.activeFrameId = "frame-b";
    runtime.continuityId = "cont-b";
    state.setActiveWorkpointPacket({
      project_root: "/tmp/project-b",
      continuity_id: "cont-b",
      mission: "wp-b",
    });
    state.setLastTrajectoryClarity({
      project_root: "/tmp/project-b",
      continuity_id: "cont-b",
      long_term_goal: "traj-b",
    });
    state.setLastProjectIdentity({ project_root: "/tmp/project-b", canonical_name: "identity-b" });
    state.setTurnCount(3);
  });
  await state.runWithAttachmentRuntime(keyA, async () => {
    const runtime = state.getAttachmentRuntime();
    assert.equal(runtime.currentAsk.text, "ask-a");
    assert.deepEqual(runtime.localDecisions, ["decision-a"]);
    assert.deepEqual(runtime.localConstraints, ["constraint-a"]);
    assert.deepEqual(runtime.localFailures, ["failure-a"]);
    assert.equal(runtime.activeFrameId, "frame-a");
    assert.equal(state.getContinuityId(), "cont-a");
    assert.equal(state.getActiveWorkpointPacket().mission, "wp-a");
    assert.equal(state.getLastTrajectoryClarity().long_term_goal, "traj-a");
    assert.equal(state.getLastProjectIdentity().canonical_name, "identity-a");
    assert.equal(state.getTurnCount(), 7);
  });
  await state.runWithAttachmentRuntime(keyB, async () => {
    const runtime = state.getAttachmentRuntime();
    assert.equal(runtime.currentAsk.text, "ask-b");
    assert.deepEqual(runtime.localDecisions, ["decision-b"]);
    assert.deepEqual(runtime.localConstraints, ["constraint-b"]);
    assert.deepEqual(runtime.localFailures, ["failure-b"]);
    assert.equal(runtime.activeFrameId, "frame-b");
    assert.equal(state.getContinuityId(), "cont-b");
    assert.equal(state.getActiveWorkpointPacket().mission, "wp-b");
    assert.equal(state.getLastTrajectoryClarity().long_term_goal, "traj-b");
    assert.equal(state.getLastProjectIdentity().canonical_name, "identity-b");
    assert.equal(state.getTurnCount(), 3);
  });
  assert.throws(() => state.getAttachmentRuntime(), /attachment_runtime_key_required/);

  const index = readFileSync(new URL("../src/index.ts", import.meta.url), "utf8");
  assert(
    index.includes("attachmentRuntimeRegistry.boundSessionAttachment(sessionId)"),
    "unscoped tool events must reuse the verified session attachment"
  );
  assert(
    index.includes("tool.execute(id, params, signal, onUpdate, ctx)"),
    "tool wrapper must preserve Pi execution context for session binding"
  );
  const tools = readFileSync(new URL("../src/tools.ts", import.meta.url), "utf8");
  const detailedStart = tools.indexOf("async function focusaFetchDetailed");
  const detailedEnd = tools.indexOf("async function", detailedStart + 1);
  const detailed = tools.slice(detailedStart, detailedEnd > detailedStart ? detailedEnd : undefined);
  for (const header of [
    "x-scope-project-root",
    "x-scope-continuity-id",
    "x-scope-session-id",
    "x-scope-id",
    "x-scope-kind",
    "x-scope-attachment-id",
  ]) {
    assert(detailed.includes(header), `focusaFetchDetailed missing typed scope header ${header}`);
  }
  assert(detailed.includes("currentAttachmentKey()"), "focusaFetchDetailed must read bound attachment key");
  assert(!/process\.cwd\(\).*x-focusa-project-root/s.test(detailed), "headers must not use cwd fallback");
  for (const route of ["metacognition", "turn", "snapshots"]) {
    assert(detailed.includes("scopeHeaders"), `${route} mocked requests use shared scoped headers`);
  }
  const repoRoot = fileURLToPath(new URL("../../..", import.meta.url)).replace(/\/$/, "");
  const recoveryKey = state.makeAttachmentKey({
    projectRoot: repoRoot,
    continuityId: "cont-frame-recovery",
    sessionId: "session-frame-recovery",
  });
  const originalFetch = globalThis.fetch;
  let recoveredFrameRequestHeaders;
  let recoveredSessionStartBody;
  globalThis.fetch = async (input, init) => {
    const url = String(input);
    if (url.includes("/focus/push")) recoveredFrameRequestHeaders = init?.headers;
    if (url.includes("/session/start")) recoveredSessionStartBody = JSON.parse(init?.body || "{}");
    const body = url.includes("/focus/push")
      ? { frame_id: "frame-recovered-after-stale-health" }
      : url.includes("/session/start")
        ? { status: "accepted" }
        : {};
    return {
      ok: true,
      status: 200,
      async json() {
        return body;
      },
    };
  };
  try {
    const recoveredFrame = await state.runWithAttachmentRuntime(recoveryKey, async () => {
      const runtime = state.getAttachmentRuntime();
      runtime.focusaAvailable = false;
      runtime.sessionCwd = repoRoot;
      runtime.sessionFrameKey = "session-frame-recovery";
      return state.ensurePiFrame(repoRoot, "session-frame-recovery", "spec104-stale-health-proof");
    });
    assert.equal(
      recoveredFrame,
      "frame-recovered-after-stale-health",
      "stale health cache must not veto authoritative scoped frame recovery"
    );
    assert.deepEqual(recoveredSessionStartBody, {
      adapter_id: "pi",
      workspace_id: repoRoot,
      project_root: repoRoot,
      continuity_id: "cont-frame-recovery",
    });
    assert.equal(recoveredFrameRequestHeaders?.["X-Scope-Project-Root"], repoRoot);
    assert.equal(recoveredFrameRequestHeaders?.["X-Scope-Continuity-Id"], "cont-frame-recovery");
    assert.equal(recoveredFrameRequestHeaders?.["X-Scope-Session-Id"], "session-frame-recovery");
  } finally {
    globalThis.fetch = originalFetch;
  }

  state.attachmentRuntimeRegistry.reset();
  const attachmentlessFetch = globalThis.fetch;
  const attachmentlessRequests = [];
  globalThis.fetch = async (url, init) => {
    attachmentlessRequests.push({ url: String(url), headers: new Headers(init.headers) });
    return new Response(JSON.stringify({ status: "completed" }), { status: 200 });
  };
  try {
    assert.deepEqual(await state.focusaFetch("/update/policy"), { status: "completed" });
    assert.equal(attachmentlessRequests.length, 1);
    assert.equal(attachmentlessRequests[0].url, "http://127.0.0.1:8787/v1/update/policy");
    assert.equal(attachmentlessRequests[0].headers.has("X-Scope-Project-Root"), false);
  } finally {
    globalThis.fetch = attachmentlessFetch;
  }

  const legacyPredictionText = scopedState.renderScopedResultHuman({
    status: "ok",
    predictions: [{ prediction_id: "p1" }],
  });
  assert(legacyPredictionText.includes("Predictions: 1"), "legacy prediction envelope must render safely");
  assert(
    legacyPredictionText.includes("canonical authority not inferred"),
    "legacy prediction rendering must not invent authority"
  );
  console.log("spec104 attachment runtime isolation and scoped request headers passed");
} finally {
  rmSync(outDir, { recursive: true, force: true });
}
