import assert from "node:assert/strict";
import { readFileSync, rmSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";
import ts from "typescript";

const root = resolve(import.meta.dirname, "..");
const source = readFileSync(resolve(root, "src/mission-canvas-layout.ts"), "utf8");
const token = `${process.pid}-${Date.now()}`;
const stateName = `.mission-canvas-layout-state-${token}.mjs`;
const layoutPath = resolve(root, `.mission-canvas-layout-runtime-${token}.mjs`);
const statePath = resolve(root, stateName);
const compiled = ts
  .transpileModule(source, {
    compilerOptions: { module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ES2022 },
  })
  .outputText.replace("./state.js", `./${stateName}`);

writeFileSync(layoutPath, compiled);
writeFileSync(
  statePath,
  `export const calls = [];
let stateVersion = 4;
let surface = {
  work_surface_id: "surface:one",
  state_revision: 2,
  project_root: "/workspace/focusa",
  continuity_id: "canvas-work",
  attachment_id: "attachment:canvas-work",
  instance_id: "instance:one",
  session_id: "session:one",
  workpoint_id: "focusa-123",
  mission_ref: "mission:canvas",
  title: "Primary Workpoint",
  surface_kind: "workpoint",
  status: "active",
  pane_id: "primary",
  tab_index: 0,
  pinned: false,
  unread: false,
  canonical_state_refs: ["focusa-123"]
};
export async function focusaFetch(path, options = {}) {
  calls.push({ path, options });
  if (options.method !== "POST") return { state_version: stateVersion, surfaces: [surface] };
  const body = JSON.parse(options.body);
  stateVersion += 1;
  if (body.action === "create") {
    surface = {
      ...surface,
      ...body,
      work_surface_id: "surface:created",
      state_revision: 1,
      status: "active"
    };
  } else {
    surface = {
      ...surface,
      ...body,
      state_revision: surface.state_revision + 1,
      status: body.action === "suspend" ? "suspended" : body.action === "resume" ? "active" : body.action === "close_view" ? "view_closed" : surface.status
    };
  }
  return { state_version: stateVersion, surface };
}
export function getActiveWorkpointPacket() {
  return { workpoint_id: "focusa-123", mission_ref: "mission:canvas", attachment_id: "attachment:canvas-work" };
}
export function getContinuityId() { return "canvas-work"; }
export function getSessionCwd() { return "/workspace/focusa"; }
`
);

try {
  const [{ openMissionCanvasSurfaceManager }, state] = await Promise.all([
    import(`${pathToFileURL(layoutPath).href}?v=${Date.now()}`),
    import(pathToFileURL(statePath).href),
  ]);
  const messages = [];
  const notifications = [];
  const pi = { sendMessage(message) { messages.push(message); } };

  async function run(action, secondary, { input = [], confirm = true } = {}) {
    const selections = [action, secondary].filter(Boolean);
    const inputs = [...input];
    const ctx = {
      hasUI: true,
      ui: {
        async select(_title, options) {
          const next = selections.shift();
          if (next === "$first") return options[0];
          return next;
        },
        async input() { return inputs.shift(); },
        async confirm() { return confirm; },
        notify(message, level) { notifications.push({ message, level }); },
      },
    };
    await openMissionCanvasSurfaceManager(pi, ctx);
  }

  await run("Pin or unpin surface", "$first");
  let mutation = state.calls.filter((call) => call.options.method === "POST").at(-1);
  let body = JSON.parse(mutation.options.body);
  assert.equal(mutation.path, "/mission-canvas/surfaces/mutate");
  assert.equal(body.action, "arrange");
  assert.equal(body.pinned, true);
  assert.equal(body.expected_state_version, 4);
  assert.equal(body.expected_surface_revision, 2);

  await run("Suspend surface", "$first");
  body = JSON.parse(state.calls.filter((call) => call.options.method === "POST").at(-1).options.body);
  assert.equal(body.action, "suspend");
  assert.equal(body.expected_state_version, 5);

  await run("Close view (work continues)", "$first");
  body = JSON.parse(state.calls.filter((call) => call.options.method === "POST").at(-1).options.body);
  assert.equal(body.action, "close_view");
  assert(messages.at(-1).content.includes("Closing this view never terminates"));

  await run("Create surface", "Document", { input: ["Release notes"] });
  body = JSON.parse(state.calls.filter((call) => call.options.method === "POST").at(-1).options.body);
  assert.equal(body.action, "create");
  assert.equal(body.title, "Release notes");
  assert.equal(body.surface_kind, "document");
  assert.deepEqual(body.canonical_state_refs, ["focusa-123", "mission:canvas", "attachment:canvas-work"]);
  assert.equal(notifications.length, 0);

  console.log("Mission Canvas durable Work Surface manager: PASS");
} finally {
  rmSync(layoutPath, { force: true });
  rmSync(statePath, { force: true });
}
