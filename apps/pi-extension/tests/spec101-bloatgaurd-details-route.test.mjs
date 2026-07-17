import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdtempSync, rmSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const projectDir = fileURLToPath(new URL("..", import.meta.url));
const outDir = mkdtempSync(join(projectDir, ".bloatgaurd-tools-"));

try {
  execFileSync(
    "./node_modules/.bin/tsc",
    ["-p", "tsconfig.json", "--outDir", outDir, "--noEmit", "false", "--module", "ES2022"],
    { cwd: projectDir, stdio: "pipe" }
  );

  const tools = await import(pathToFileURL(join(outDir, "tools.js")).href);
  const state = await import(pathToFileURL(join(outDir, "state.js")).href);

  state.attachmentRuntimeRegistry.reset();
  const registeredTools = new Map();

  const pi = {
    on() {},
    registerTool(tool) {
      if (typeof tool?.name === "string" && tool.name.startsWith("focusa_bloatgaurd_")) {
        registeredTools.set(tool.name, tool);
      }
    },
  };

  tools.registerTools(pi);

  const attachmentKey = state.makeAttachmentKey({
    projectRoot: "/tmp/focusa-bloatgaurd",
    continuityId: "cont-bloatgaurd",
    sessionId: "session-bloatgaurd",
    attachmentId: "attach-bloatgaurd",
  });

  state.attachmentRuntimeRegistry.bindSessionAttachment(attachmentKey);

  await state.runWithAttachmentRuntime(attachmentKey, async () => {
    state.setLastTrajectoryClarity({
      refreshed_at: Date.now(),
      project_root: "/tmp/focusa-bloatgaurd",
      continuity_id: "cont-bloatgaurd",
      trajectory_id: "traj-bloatgaurd",
      hlt: "bloatgaurd detail check",
      mlg: "tool route probe",
      stg: "focusa route assertion",
      waypoints: ["domain", "tokenbloat", "gate", "profile", "routine"],
    });
  });

  const cases = [
    {
      tool: "focusa_bloatgaurd_domain",
      name: "output-firewall",
      expectedPath: "/v1/bloatgaurd/domain/output-firewall",
      response: { status: "completed", domain: { name: "output-firewall" } },
    },
    {
      tool: "focusa_bloatgaurd_tokenbloat_domain",
      name: "tokenbloat-control",
      expectedPath: "/v1/bloatgaurd/tokenbloat/domain/tokenbloat-control",
      response: { status: "completed", domain: { name: "tokenbloat-control" } },
    },
    {
      tool: "focusa_bloatgaurd_gate_mode",
      name: "A",
      expectedPath: "/v1/bloatgaurd/gate-modes/mode/A",
      response: { status: "completed", mode: { code: "A", name: "advisory" } },
    },
    {
      tool: "focusa_bloatgaurd_profile",
      name: "daily_driver",
      expectedPath: "/v1/bloatgaurd/profiles/profile/daily_driver",
      response: { status: "completed", profile: { name: "daily_driver" } },
    },
    {
      tool: "focusa_bloatgaurd_routine",
      name: "patrol",
      expectedPath: "/v1/bloatgaurd/routines/routine/patrol",
      response: { status: "completed", routine: { name: "patrol" } },
    },
  ];

  const expectedPathsByTool = new Map(cases.map((entry) => [entry.tool, entry.expectedPath]));
  const responsesByPath = new Map(cases.map((entry) => [entry.expectedPath, entry.response]));
  const calls = [];

  const originalFetch = globalThis.fetch;
  globalThis.fetch = async (input) => {
    const path = new URL(String(input)).pathname;
    calls.push(path);
    const response = responsesByPath.get(path);
    if (!response) {
      return {
        ok: false,
        status: 400,
        async json() {
          return { status: "schema_invalid", failure_class: "schema_invalid" };
        },
      };
    }
    return {
      ok: true,
      status: 200,
      async json() {
        return response;
      },
    };
  };

  try {
    for (const entry of cases) {
      const tool = registeredTools.get(entry.tool);
      assert.ok(tool, `${entry.tool} must be registered`);
      const invocationId = `${entry.tool}-invocation`;
      const result = await state.runWithAttachmentRuntime(attachmentKey, async () =>
        tool.execute(invocationId, { name: entry.name })
      );
      const failureClass = result?.details?.tool_result_v1?.failure_class ?? null;
      assert.notEqual(
        failureClass,
        "schema_invalid",
        `${entry.tool} should not return failure_class: schema_invalid for valid {name}`
      );
      assert.equal(
        calls.includes(expectedPathsByTool.get(entry.tool)),
        true,
        `${entry.tool} must reach route ${expectedPathsByTool.get(entry.tool)}`
      );
    }

    for (const entry of cases) {
      const expected = expectedPathsByTool.get(entry.tool);
      const count = calls.filter((path) => path === expected).length;
      assert.equal(count, 1, `${entry.tool} should call exactly one expected route`);
    }

    const uniqueExpected = Array.from(new Set(expectedPathsByTool.values()));
    assert.equal(
      calls.filter((path) => uniqueExpected.includes(path)).length,
      cases.length,
      "all expected Bloatgaurd detail routes should be hit"
    );
  } finally {
    globalThis.fetch = originalFetch;
  }

  console.log("spec101 bloatgaurd detail route regression test passed");
} finally {
  rmSync(outDir, { recursive: true, force: true });
}
