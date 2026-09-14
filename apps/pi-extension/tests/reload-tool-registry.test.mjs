import assert from "node:assert/strict";
import { mkdtempSync, writeFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { pathToFileURL } from "node:url";

const sdkEntry = process.env.PI_SDK_ENTRY
  ? pathToFileURL(process.env.PI_SDK_ENTRY).href
  : import.meta.resolve("@earendil-works/pi-coding-agent");
const { DefaultResourceLoader, SettingsManager, SessionManager, ModelRuntime, createAgentSession } =
  await import(sdkEntry);
const root = mkdtempSync(join(tmpdir(), "focusa-reload-registry-"));
const source = process.env.FOCUSA_RELOAD_SOURCE_ROOT || resolve(import.meta.dirname, "../src");
let session;
try {
  // Load the real coordinator and the complete real tool registry through Pi's
  // native loader. Omit unrelated service/communications hooks from this fixture.
  const fixture = join(root, "extension.ts");
  writeFileSync(
    fixture,
    `import {registerAutoCompaction} from ${JSON.stringify(join(source, "auto-compaction.ts"))};\nimport {registerTools} from ${JSON.stringify(join(source, "tools.ts"))};\nexport default function(pi) { if (registerAutoCompaction(pi)) registerTools(pi); }\n`
  );
  const settingsManager = SettingsManager.inMemory({ packages: [], extensions: [] });
  const loaderOptions = { cwd: root, agentDir: root, settingsManager, additionalExtensionPaths: [fixture] };
  const resourceLoader = new DefaultResourceLoader(loaderOptions);
  await resourceLoader.reload();
  assert.deepEqual(resourceLoader.getExtensions().errors, []);
  const modelRuntime = await ModelRuntime.create({
    authPath: join(root, "auth.json"),
    allowModelNetwork: false,
    modelsPath: join(root, "models.json"),
    modelsStorePath: join(root, "models-store.json"),
  });
  ({ session } = await createAgentSession({
    cwd: root,
    agentDir: root,
    resourceLoader,
    settingsManager,
    modelRuntime,
    model: modelRuntime.getModel("anthropic", "claude-sonnet-4-5"),
    sessionManager: SessionManager.inMemory(root),
  }));
  const names = () =>
    session
      .getAllTools()
      .map((t) => t.name)
      .filter((n) => n.startsWith("focusa_"))
      .sort();
  const initial = names();
  assert.ok(initial.includes("focusa_tool_doctor"));
  assert.ok(initial.includes("focusa_project_identity"));
  const duplicate = new DefaultResourceLoader(loaderOptions);
  await duplicate.reload();
  assert.equal(
    duplicate.getExtensions().extensions.reduce((n, e) => n + e.tools.size, 0),
    0,
    "duplicate installation must still be suppressed"
  );
  for (let i = 0; i < 5; i++) {
    await session.reload();
    assert.deepEqual(resourceLoader.getExtensions().errors, []);
    assert.deepEqual(names(), initial, `full tool registry lost after native reload ${i + 1}`);
    assert.deepEqual(
      session
        .getActiveToolNames()
        .filter((n) => n.startsWith("focusa_"))
        .sort(),
      initial
    );
  }
  const lease = globalThis[Symbol.for("focusa.compaction.coordinator.v1")];
  assert.ok(lease.owner);
  lease.owner.nativeSession = undefined;
  lease.owner.attachmentId = "retired-session-fixture";
  const recovered = new DefaultResourceLoader(loaderOptions);
  await recovered.reload();
  assert.deepEqual(recovered.getExtensions().errors, []);
  assert.deepEqual(
    recovered
      .getExtensions()
      .extensions.flatMap((e) => [...e.tools.keys()])
      .sort(),
    initial,
    "legacy stranded registration must recover without restarting the process"
  );
  console.log(
    JSON.stringify({
      status: "passed",
      native_reloads: 5,
      focusa_tools: initial.length,
      duplicate_guard: "preserved",
    })
  );
} finally {
  session?.dispose();
  rmSync(root, { recursive: true, force: true });
}
