import assert from "node:assert/strict";
import { mkdtemp, readFile, rm, unlink, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { fileURLToPath, pathToFileURL } from "node:url";
import path from "node:path";
import ts from "typescript";

const root = fileURLToPath(new URL("..", import.meta.url));
const sourcePath = path.join(root, "src", "operator-status-widgets.ts");
const compiledPath = path.join(root, `.operator-status-widgets-test-${process.pid}.mjs`);
const configCompiledPath = path.join(root, "src", `.operator-status-config-test-${process.pid}.mjs`);
let temporaryProject;
const source = await readFile(sourcePath, "utf8");
const compiled = ts.transpileModule(source, {
  compilerOptions: { module: ts.ModuleKind.ES2022, target: ts.ScriptTarget.ES2022 },
  fileName: sourcePath,
});
await writeFile(compiledPath, compiled.outputText);

try {
  const widgets = await import(`${pathToFileURL(compiledPath).href}?v=${Date.now()}`);
  const registry = widgets.createOperatorWidgetRegistry();
  assert.deepEqual(registry.map((item) => item.id), ["time", "prediction", "version", "ota", "provider-usage"]);

  const migrated = widgets.migrateOperatorStatusSettings(undefined, {
    time: false,
    prediction: true,
    version: false,
    ota: true,
    "provider-usage": false,
  }, registry);
  assert.deepEqual(migrated, {
    version: 1,
    enabled: { time: false, prediction: true, version: false, ota: true, "provider-usage": false },
  });

  // A settings JSON restart round-trip retains independent visibility.
  const restarted = widgets.migrateOperatorStatusSettings(JSON.parse(JSON.stringify(migrated)), {}, registry);
  assert.deepEqual(restarted, migrated);
  const upgraded = widgets.migrateOperatorStatusSettings({ version: 0, enabled: { time: false } }, {}, registry);
  assert.equal(upgraded.enabled.time, false);
  assert.equal(upgraded.enabled.version, true);
  const rollback = widgets.operatorStatusRollbackPatch(restarted);
  assert.deepEqual(rollback, {
    operatorStatusTimeEnabled: false,
    operatorStatusDeadlineEnabled: false,
    operatorStatusPredictionEnabled: true,
    operatorStatusVersionEnabled: false,
    operatorStatusOtaEnabled: true,
    operatorStatusModelUsageEnabled: false,
  });

  // Exercise the real durable config writer/loader across a simulated restart.
  const configSource = await readFile(path.join(root, "src", "config.ts"), "utf8");
  await writeFile(configCompiledPath, ts.transpileModule(configSource, {
    compilerOptions: { module: ts.ModuleKind.ES2022, target: ts.ScriptTarget.ES2022 },
  }).outputText);
  const config = await import(`${pathToFileURL(configCompiledPath).href}?v=${Date.now()}`);
  temporaryProject = await mkdtemp(path.join(tmpdir(), "focusa-widget-settings-"));
  config.saveConfigOverrides(temporaryProject, { operatorStatusWidgets: restarted, ...rollback });
  const persisted = config.loadConfig(temporaryProject).config;
  assert.deepEqual(persisted.operatorStatusWidgets, restarted);
  assert.equal(persisted.operatorStatusPredictionEnabled, true);
  assert.equal(persisted.operatorStatusVersionEnabled, false);

  const now = Date.parse("2026-08-01T12:00:00Z");
  const fullData = {
    now,
    timezone: "UTC",
    deadline: "2026-08-01T18:00:00Z",
    prediction: "verify the widget proof",
    predictionObservedAt: now,
    version: "focusa-pi-bridge@0.9.150-dev",
    ota: "current",
    otaState: "ready",
    otaObservedAt: now,
    provider: "openai",
    model: "gpt-test",
    usagePercent: 42,
    renewalAt: "2026-08-02T00:00:00Z",
    providerObservedAt: now,
  };
  const allEnabled = widgets.migrateOperatorStatusSettings({}, {}, registry);
  const rendered = widgets.renderOperatorStatusBar(fullData, allEnabled, 1_000, registry);
  assert.equal(rendered.hidden, 0);
  assert.deepEqual(rendered.widgets.map((item) => item.state), ["ready", "ready", "ready", "ready", "ready"]);
  assert.match(rendered.text, /Next verify the widget proof/);
  assert.match(rendered.text, /42% used/);
  assert.match(rendered.text, /active Workpoint/);

  const mixed = widgets.renderOperatorStatusBar({ now, version: "", ota: "unknown", otaState: "degraded", predictionLoading: true }, allEnabled, 1_000, registry);
  assert.deepEqual(mixed.widgets.map((item) => item.state), ["degraded", "loading", "unavailable", "degraded", "unavailable"]);
  assert.doesNotMatch(mixed.text, /context \d+%/);
  assert.match(mixed.text, /deadline unavailable/);
  assert.match(mixed.text, /renewal unavailable/);

  const stale = widgets.renderOperatorStatusBar({ ...fullData, providerObservedAt: now - 3_600_001, predictionObservedAt: now - 3_600_001 }, allEnabled, 1_000, registry);
  assert.equal(stale.widgets.find((item) => item.id === "provider-usage").state, "stale");
  assert.equal(stale.widgets.find((item) => item.id === "prediction").state, "stale");

  for (let width = 1; width <= 180; width += 1) {
    const narrow = widgets.renderOperatorStatusBar(fullData, allEnabled, width, registry);
    assert.ok(narrow.text.length <= width, `overflow at width ${width}: ${narrow.text.length}`);
  }

  const onlyVersion = widgets.migrateOperatorStatusSettings({ enabled: {
    time: false,
    prediction: false,
    version: true,
    ota: false,
    "provider-usage": false,
  } }, {}, registry);
  assert.deepEqual(widgets.renderOperatorStatusBar(fullData, onlyVersion, 200, registry).widgets.map((item) => item.id), ["version"]);

  // Extensibility proof: registration alone adds a deterministic fixture widget.
  const fixture = {
    id: "fixture",
    label: "Fixture",
    order: 35,
    defaultEnabled: true,
    render: () => ({ id: "fixture", label: "Fixture", order: 35, text: "Fixture canonical", state: "ready", source: "test fixture" }),
  };
  const extended = widgets.createOperatorWidgetRegistry([...registry, fixture]);
  assert.deepEqual(extended.map((item) => item.id), ["time", "prediction", "version", "fixture", "ota", "provider-usage"]);
  const extendedSettings = widgets.migrateOperatorStatusSettings({}, {}, extended);
  assert.match(widgets.renderOperatorStatusBar(fullData, extendedSettings, 1_000, extended).text, /Fixture canonical/);
  assert.throws(() => widgets.createOperatorWidgetRegistry([...registry, fixture, fixture]), /Duplicate/);

  const integrationSource = await readFile(path.join(root, "src", "polish.ts"), "utf8");
  assert.match(integrationSource, /registerCommand\("focusa-bar"/);
  assert.match(integrationSource, /saveConfigOverrides/);
  assert.match(integrationSource, /renderOperatorStatusBar/);
  assert.doesNotMatch(integrationSource, /context \$\{Math\.round\(runtime\.currentContextPct\)\}%/);

  console.log("operator status widgets tests passed");
} finally {
  await unlink(compiledPath).catch(() => {});
  await unlink(configCompiledPath).catch(() => {});
  if (temporaryProject) await rm(temporaryProject, { recursive: true, force: true });
}
