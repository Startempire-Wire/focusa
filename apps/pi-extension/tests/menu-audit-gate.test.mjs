import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdirSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { loadConfig, resolveInteractionMode, saveConfigOverrides, setSessionInteractionMode } from "../src/config.ts";
import { fileURLToPath } from "node:url";
import { loadSourceSection } from "./helpers/source-section-harness.mjs";
import { effectiveOperatorStatusSettings, operatorStatusRollbackPatch } from "../src/operator-status-widgets.ts";

const root = fileURLToPath(new URL("../../..", import.meta.url));
const commands = readFileSync(`${root}/apps/pi-extension/src/commands.ts`, "utf8");
const commandSources = [commands, ...['auto-compaction.ts', 'polish.ts', 'wbm.ts'].map(
  name => readFileSync(`${root}/apps/pi-extension/src/${name}`, 'utf8')
)];
const session = readFileSync(`${root}/apps/pi-extension/src/session.ts`, "utf8");
const manifest = JSON.parse(
  readFileSync(`${root}/docs/contracts/48-49-focusa-pi-menu-inventory.json`, "utf8")
);
const digest = createHash("sha256")
  .update(commandSources.join('') + session)
  .digest("hex");
assert.equal(manifest.source_sha256, digest, "menu source changed without regenerated audit");
const registered = [...commandSources.join('\n').matchAll(/pi\.registerCommand\("([^"]+)"/g)].map((m) => m[1]);
const inventoried = manifest.items.filter((x) => x.kind === "command").map((x) => x.id);
assert.deepEqual(inventoried.sort(), registered.sort());
const settings = manifest.items.filter((x) => x.kind === "setting");
assert.ok(settings.length > 0);
const settingsSection = commands.slice(commands.indexOf('const buildSimpleItems'), commands.indexOf('const displayedItems'));
const declared = [
  ...[...settingsSection.matchAll(/id:\s*"([^"]+)"/g)].map(match => match[1]),
  ...[...settingsSection.matchAll(/\["(operatorStatus[^"]+)",\s*"[^"]+"\]/g)].map(match => match[1]),
];
assert.deepEqual(settings.map(item => item.id).sort(), declared.sort(), 'missing or extra settings placements');
for (const item of settings) {
  assert.ok(
    item.owner &&
      item.user_job &&
      item.placement &&
      item.rationale &&
      item.help &&
      item.scope &&
      item.risk &&
      item.interaction_test
  );
  assert.ok(item.values.length > 0, `${item.id} lacks allowed-value coverage`);
  assert.match(commands, new RegExp(`id === "${item.id}"`), `${item.id} lacks callback coverage`);
}
assert.equal(
  manifest.items.filter((x) => ["select", "confirm", "input", "custom"].includes(x.kind)).length,
  3
);
assert.ok(manifest.after.simple_settings < manifest.baseline.simple_settings);
assert.equal(manifest.after.simple_settings, 9);
assert.deepEqual(
  Object.keys(manifest.migration).sort(),
  ["otaProfile", "vitalInfoPromptSurfaces", "workLoopStatusHeartbeatMs"].sort()
);
assert.match(commands, /try \{/);
assert.match(commands, /prior configuration remains active/);
assert.match(commands, /prior value restored/);
// Execute the production settings handler; capture writes rather than changing host policy.
const constants = Object.fromEntries([...commands.matchAll(/^const ([A-Z_]+) = (\[.*\]);$/gm)].map(
  match => [match[1], JSON.parse(match[2])]
));
let handler, panel, otaRead, writes, saves, panelComponent;
const settingsAttachment = { workstream: { root_scope: { scope_kind: 'project', root_path: '/fixture-attached' } } };
const runtime = { cfg: {} };
class Container { addChild() {} invalidate() {} }
class Text {}
class SettingsList { constructor(items, _size, _theme, change) { panel = { items, change }; } }
loadSourceSection('commands.ts', 'pi.registerCommand("focusa-settings"', '// /focusa-status (§10.3)', {
  ...constants, Container, Text, SettingsList, getSettingsListTheme: () => ({}),
  effectiveOperatorStatusSettings, operatorStatusRollbackPatch,
  pi: { registerCommand(_name, definition) { handler = definition.handler; } },
  currentAttachmentKey: () => settingsAttachment, getAttachmentRuntime: () => runtime,
  isProjectRootAuthoritySafe: root => root === '/fixture-attached', priorInteractionModes: new WeakMap(),
  focusaFetch: async (path, init) => { if (!init) return otaRead; writes.push({ path, data: JSON.parse(init.body) }); return { status: 'completed', policy: { auto_apply_allowed: true } }; },
  normalizeTierConfig() {}, saveConfigOverrides(cwd, draft) { assert.equal(cwd, '/fixture-attached'); const config = { ...draft }; saves.push(config); return { config, errors: [], path: '/fixture/settings.json' }; },
}, 'undefined');
const ctx = { mode: 'tui', hasUI: true, cwd: '/fixture', ui: {
  notify() {}, custom(factory) { panelComponent = factory({ requestRender() {} }, { fg: (_c, s) => s, bold: s => s }, null, () => {}); },
} };
let exercised = 0;
for (const [args, placement] of [['', 'simple_default'], ['advanced', 'advanced_searchable']]) {
  for (const item of settings.filter(setting => setting.placement === placement)) {
    for (const value of item.values) {
      runtime.cfg = {}; writes = []; saves = []; otaRead = { status: 'completed', policy: { enabled: false } };
      await handler(args, ctx);
      assert.deepEqual(panel.items.find(row => row.id === item.id)?.values, item.values);
      panel.change(item.id, value);
      await new Promise(resolve => setImmediate(resolve));
      if (item.id === 'otaEnabled' || item.id === 'otaProfile') {
        assert.deepEqual(writes.map(write => write.path), ['/update/policy', '/update/scheduler']);
        const expected = item.id === 'otaEnabled' ? value === 'true' : value !== 'notify';
        assert.equal(writes[0].data.enabled, expected);
      } else if (item.id === 'simpleProfile') {
        assert.equal(saves.length, value === 'custom' ? 0 : 1);
      } else {
        assert.equal(saves.length, 1, item.id);
        assert.equal(String(saves[0][item.id]), value, item.id);
      }
      exercised++;
    }
  }
}
runtime.cfg = { operatorStatusVersionEnabled: true, operatorStatusWidgets: { version: 1, enabled: { version: false, time: true } } };
await handler('advanced', ctx);
assert.equal(panel.items.find(row => row.id === 'operatorStatusVersionEnabled').currentValue, 'false');
panel.change('operatorStatusVersionEnabled', 'true');
assert.equal(runtime.cfg.operatorStatusWidgets.enabled.version, true);
assert.equal(effectiveOperatorStatusSettings(runtime.cfg).enabled.version, true);
panel.change('operatorStatusTimeEnabled', 'false');
assert.equal(effectiveOperatorStatusSettings(runtime.cfg).enabled.time, false);
assert.equal(runtime.cfg.operatorStatusDeadlineEnabled, false);
const savedBeforeDispose = saves.length;
panelComponent.dispose();
panel.change('operatorStatusTimeEnabled', 'true');
assert.equal(saves.length, savedBeforeDispose);
runtime.cfg = { workLoopMaxRetries: 0, workLoopCooldownMs: 0 };
await handler('advanced', ctx);
for (const id of ['workLoopMaxRetries', 'workLoopCooldownMs']) assert.equal(panel.items.find(row => row.id === id)?.currentValue, '0');
otaRead = null;
await handler('advanced', ctx);
assert.ok(!panel.items.some(row => row.id === 'otaEnabled' || row.id === 'otaProfile'));
panel = null;
await handler('advanced', { ...ctx, mode: 'rpc' });
assert.equal(panel, null);
let workLoopHandler;
const workLoopCalls = [];
const notifications = [];
loadSourceSection('commands.ts', 'pi.registerCommand("focus-work"', 'pi.registerCommand("focusa-on"', {
  pi: { registerCommand(_name, definition) { workLoopHandler = definition.handler; } },
  currentAttachmentKey: () => 'fixture-key',
  actions: {
    async workLoopControl(_id, params) { workLoopCalls.push(params); return { content: [{ text: 'writer not verified' }], details: { ok: false } }; },
    async workLoopCheckpoint(_id, params) { workLoopCalls.push(params); return { content: [{ text: 'checkpoint saved' }], details: { ok: true } }; },
  },
}, 'undefined');
const commandContext = { ui: { notify: (...args) => notifications.push(args) } };
await workLoopHandler('resume', commandContext);
assert.equal(workLoopCalls[0].action, 'resume');
assert.deepEqual(notifications[0], ['writer not verified', 'warning']);
await workLoopHandler('off', commandContext);
assert.equal(workLoopCalls[1].action, 'stop');
await workLoopHandler('checkpoint example', commandContext);
assert.deepEqual(workLoopCalls[2], { summary: 'example' });
assert.deepEqual(notifications[2], ['checkpoint saved', 'info']);
await workLoopHandler('unknown-option', commandContext);
assert.equal(workLoopCalls.length, 3);
assert.match(notifications[3][0], /Usage:/);

const projectReplay = loadSourceSection('commands.ts', 'function replayConsumerSurface(', 'const WARN_OPTIONS', {}, 'replayConsumerSurface');
for (const unreadable of [null, {}, { status: 'ok', degraded: true }, { status: 'blocked', secondary_loop_continuity_gate: { state: 'open' } }]) {
  const model = projectReplay(unreadable);
  assert.equal(model.pairLabel, 'unknown');
  assert.equal(model.continuityGate, 'fail-closed');
}
assert.equal(projectReplay({ status: 'ok' }).continuityGate, 'fail-closed', 'missing gate must not default open');
assert.equal(projectReplay({ status: 'ok', secondary_loop_continuity_gate: { state: 'open' } }).continuityGate, 'open');
const resetDefinition = commands.slice(commands.indexOf('pi.registerCommand("focusa-reset"'));
assert.doesNotMatch(resetDefinition, /\/focus\/update/);
assert.match(resetDefinition, /saved history retained/i);

let workLoopResponse;
const readWorkLoop = loadSourceSection('commands.ts', 'async function readVerifiedWorkLoop()', 'export function registerCommands', {
  focusaFetch: async route => { assert.equal(route, '/work-loop/status'); return workLoopResponse; },
}, 'readVerifiedWorkLoop');
for (const response of [null, { status: 'blocked' }, { enabled: false, degraded: true }, { enabled: false, ok: false }]) {
  workLoopResponse = response;
  assert.equal(await readWorkLoop(), null);
}
workLoopResponse = { enabled: false, status: 'disabled' };
assert.deepEqual(await readWorkLoop(), workLoopResponse, 'explicit disabled state must remain distinguishable from unreadable');

const workspace = mkdtempSync(join(tmpdir(), 'focusa-menu-persistence-'));
const settingsPath = join(workspace, '.pi', 'settings.json');
mkdirSync(join(workspace, '.pi'));
writeFileSync(settingsPath, '{"theme":"other-extension-theme"}\n');
const valid = saveConfigOverrides(workspace, { interactionMode: 'terminal-guided' });
assert.deepEqual(valid.errors, []);
assert.equal(loadConfig(workspace).config.interactionMode, 'terminal-guided');
const attachmentA = {}, attachmentB = {};
const inherited = resolveInteractionMode(workspace, attachmentB);
setSessionInteractionMode(attachmentA, 'headless');
assert.deepEqual(resolveInteractionMode(workspace, attachmentA), { mode: 'headless', source: 'session' });
assert.deepEqual(resolveInteractionMode(workspace, attachmentB), inherited);
setSessionInteractionMode(attachmentA);
assert.deepEqual(resolveInteractionMode(workspace, attachmentA), inherited);
assert.equal(JSON.parse(readFileSync(settingsPath, 'utf8')).theme, 'other-extension-theme');
const priorBytes = readFileSync(settingsPath, 'utf8');
const rejected = saveConfigOverrides(workspace, { warnPct: 95, compactPct: 50 });
assert.ok(rejected.errors.length);
assert.equal(readFileSync(settingsPath, 'utf8'), priorBytes);
for (const content of ['[1,2]', '{bad JSON']) {
  writeFileSync(settingsPath, content);
  assert.throws(() => saveConfigOverrides(workspace, { warnPct: 40 }), /Refusing to overwrite/);
  assert.equal(readFileSync(settingsPath, 'utf8'), content);
}
console.log(`menu audit gate: PASS (${manifest.items.length} entries; ${settings.length} placements; ${exercised} setting values exercised)`);
