import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { pathToFileURL } from "node:url";

const outDir = mkdtempSync(join(tmpdir(), "focusa-pi-runtime-"));
try {
  execFileSync(
    "./node_modules/.bin/tsc",
    ["-p", "tsconfig.json", "--outDir", outDir, "--noEmit", "false", "--module", "ES2022"],
    { cwd: new URL("..", import.meta.url), stdio: "pipe" }
  );
  writeFileSync(join(outDir, "package.json"), '{"type":"module"}\n');
  const state = await import(pathToFileURL(join(outDir, "state.js")).href);
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
  console.log("spec104 attachment runtime isolation and scoped request headers passed");
} finally {
  rmSync(outDir, { recursive: true, force: true });
}
