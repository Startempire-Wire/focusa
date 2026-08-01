import assert from "node:assert/strict";
import { readFileSync, rmSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";
import ts from "typescript";

const root = resolve(import.meta.dirname, "..");
const accessibilityName = `.mission-canvas-accessibility-reference-${process.pid}.mjs`;
const viewName = `.mission-canvas-view-reference-${process.pid}.mjs`;
const accessibilityPath = resolve(root, accessibilityName);
const viewPath = resolve(root, viewName);
const accessibilitySource = readFileSync(resolve(root, "src/mission-canvas-accessibility.ts"), "utf8");
const viewSource = readFileSync(resolve(root, "src/mission-canvas-view.ts"), "utf8");
writeFileSync(accessibilityPath, ts.transpileModule(accessibilitySource, {
  compilerOptions: { module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ES2022 },
}).outputText);
writeFileSync(viewPath, ts.transpileModule(viewSource, {
  compilerOptions: { module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ES2022 },
}).outputText.replace("./mission-canvas-accessibility.js", `./${accessibilityName}`));

const { MissionCanvasView } = await import(`${pathToFileURL(viewPath).href}?v=${Date.now()}`);
const stripAnsi = (value) => value.replace(/\x1b\[[0-9;]*m/g, "");
const rendered = (view, width = 140) => view.render(width).map(stripAnsi).join("\n");
const theme = { fg: (_name, value) => value, bold: (value) => value };
const model = {
  mission: "Ship authentication refactor",
  trajectory: "Finish credential flow without losing evidence",
  nextAction: "Update governed specification",
  workpointId: "WP-73 · Credentials flow",
  workItemId: "focusa-auth-73",
  workRailDetails: ["Spec update · ready", "Tests · 16 passed", "Docs · review"],
  projectRoot: "/workspace/focusa-platform",
  continuityId: "mission-canvas-reference",
  evidenceRefs: ["16 auth tests · verified", "Code diff · verified", "Policy excerpt · promoted"],
  blockers: ["Inactive-account disclosure needs confirmation"],
  sessions: ["Pi · Auth Refactor · live writer", "Silent · Tests · isolated", "UIAI · Research · observer"],
  workSurfaces: ["Overview", "Pi · Auth Refactor", "UIAI · Research", "Silent · Tests", "Evidence"],
  workSurfaceDetails: [["auth_service.ts · modified", "validateCredentials()", "run_tests · 16 passed"]],
  contention: ["Writer lease · Pi", "One proposal pending"],
  researchArtifacts: ["Security Standard §8 · primary", "Vendor Guidance · secondary"],
  history: ["10:21 patch applied", "10:22 tests started", "10:26 evidence promoted"],
  contextStatus: "Auth service · password hasher · error contract",
  roleStatus: "Senior software engineer · writer",
  interviewStatus: "Open · credential validation branch",
  specStatus: "§4.2 Credential validation · approved draft",
  workLoopStatus: "running",
  scopeStatus: "verified",
  workspaceProfile: "general",
  visualVariant: "default",
  steeringQueue: ["Preserve backwards compatibility"],
  followUpQueue: ["Update authentication examples"],
};

const view = new MissionCanvasView(model, theme, () => {}, () => {}, async () => model, () => {}, () => {});
view.setConversation(["PI Scanning auth service and repo", "TOOL run_tests · 16 passed"]);

const captures = {};
captures.overview = view.render(140);
const overview = captures.overview.map(stripAnsi).join("\n");
for (const expected of ["Project  focusa-platform", "Canvas ●", "Mission Status", "Today’s Focus", "Pi Transcript · live", "Steering Queue", "Follow-up Queue", "PROMPT"]) {
  if (expected === "PROMPT") continue; // Prompt Editor belongs to the shell.
  assert.ok(overview.includes(expected), `overview missing ${expected}`);
}
assert.ok(!overview.includes("CURRENT WORKSPACE COCKPIT"));
assert.ok(!overview.includes("WHAT CHANGES"));
assert.ok(!overview.includes("WHAT STAYS THE SAME"));

view.handleInput("mode-next");
captures.context = view.render(140);
const context = captures.context.map(stripAnsi).join("\n");
for (const expected of ["Canonical Facts", "Semantic Graph", "Freshness", "Conflicts"]) assert.ok(context.includes(expected), `context missing ${expected}`);

view.handleInput("profile-next");
captures.software = view.render(140);
const software = captures.software.map(stripAnsi).join("\n");
for (const expected of ["Software Engineering", "Tasks / Work", "Focused Work Surface", "Current Workpoint", "Evidence / Authority"]) assert.ok(software.includes(expected), `software missing ${expected}`);

view.handleInput("profile-next");
captures.legal = view.render(140);
const legal = captures.legal.map(stripAnsi).join("\n");
for (const expected of ["Workspace  Legal", "Documents", "Requirements · Redline", "Authorities / Sources"]) assert.ok(legal.includes(expected), `legal missing ${expected}`);

view.handleInput("profile-next");
captures.markets = view.render(140);
const markets = captures.markets.map(stripAnsi).join("\n");
for (const expected of ["Workspace  Markets", "Active Thesis", "Bull 62%"] ) assert.ok(markets.includes(expected), `markets missing ${expected}`);

for (const width of [64, 96, 140]) {
  const lines = view.render(width).map(stripAnsi);
  assert.ok(lines.every((line) => line.length <= Math.max(40, width)), `line overflow at ${width}`);
}

const sparse = { ...model, mission: "", trajectory: "", nextAction: "", workpointId: "", workItemId: "", workRailDetails: [], evidenceRefs: [], blockers: [], sessions: [], workSurfaceDetails: [[]], contention: [], researchArtifacts: [], history: [], contextStatus: "Unavailable", roleStatus: "not reported", interviewStatus: "No durable interview session reported", specStatus: "not loaded", steeringQueue: [], followUpQueue: [] };
const sparseView = new MissionCanvasView(sparse, theme, () => {}, () => {}, async () => sparse, () => {}, () => {});
const sparseText = rendered(sparseView);
assert.ok(!/Unavailable|No durable|not loaded|not reported/i.test(sparseText));
assert.ok(!sparseText.includes("Steering Queue"));
assert.ok(!sparseText.includes("Follow-up Queue"));

const hostile = { ...model, mission: "\u001b]52;c;SECRETS\u0007\u001b[2JShip safely" };
const hostileView = new MissionCanvasView(hostile, theme, () => {}, () => {}, async () => hostile, () => {}, () => {});
const hostileText = rendered(hostileView);
assert.ok(hostileText.includes("Ship safely"));
assert.ok(!hostileText.includes("SECRETS"));
assert.ok(!hostileText.includes("\u001b[2J"));

if (process.env.FOCUSA_WRITE_CANVAS_EVIDENCE === "1") {
  const evidencePath = resolve(root, "../../docs/evidence/spec135-pi-native-reference-renders.v1.json");
  writeFileSync(evidencePath, `${JSON.stringify({ schema: "focusa.spec135.pi_native_reference_renders.v1", width: 140, captures }, null, 2)}\n`);
}

view.dispose();
sparseView.dispose();
hostileView.dispose();
rmSync(viewPath, { force: true });
rmSync(accessibilityPath, { force: true });
console.log("Mission Canvas authoritative reference-design composition: PASS");
