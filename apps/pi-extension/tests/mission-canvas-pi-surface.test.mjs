import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const read = (name) => readFileSync(resolve(root, `src/${name}`), "utf8");
const commands = read("commands.ts");
const index = read("index.ts");
const widget = read("mission-canvas-widget.ts");
const config = read("config.ts");
const view = read("mission-canvas-view.ts");
const model = read("mission-canvas-model.ts");
const tool = read("mission-canvas-tool.ts");
const shell = read("mission-canvas-shell.ts");
const turns = read("turns.ts");
const session = read("session.ts");
const inventory = read("mission-canvas-session-inventory.ts");

// One control path switches the current Pi TypeScript TUI on/off.
assert.match(commands, /registerCommand\("mission-canvas"/);
assert.match(commands, /\["on", "off", "toggle", "status"\]/);
assert.match(commands, /Mission Canvas switched/);
assert.match(commands, /registerCommand\("mission-canvas-mode"/);
assert.match(tool, /name: "focusa_mission_canvas"/);
assert.match(tool, /"open", "on", "off", "toggle", "status", "set_profile"/);
assert.match(tool, /executeMissionCanvasAction/);
assert.match(tool, /effectiveInteraction/);
assert.doesNotMatch(tool, /canvas_enabled: interaction\.mode/);
assert.match(tool, /gui: "pi_tui"/);
assert.match(tool, /host_scope: "current_pi_session"/);
assert.doesNotMatch(tool, /RichHostLifecycleManager/);
assert.doesNotMatch(tool, /richHostLifecycle\.on/);
assert.match(tool, /mission-canvas\/pi-session\/events/);
assert.match(tool, /mission_canvas_session_restored/);
assert.match(tool, /mission_canvas_lifecycle_receipt/);
assert.match(index, /registerMissionCanvasTool\(pi\)/);

// The mounted component is the authoritative Canvas, not a compatibility dashboard.
assert.match(shell, /Authoritative Pi-native Mission Canvas/);
assert.doesNotMatch(shell, /Terminal compatibility projection/);
assert.doesNotMatch(shell, /AGENT STREAM · SAME SESSION/);
assert.match(shell, /setConversation\(recentConversation\(this\.ctx\)\)/);
assert.match(shell, /sendUserMessage\(prompt\)/);
assert.match(shell, /setTitle\("Focusa Mission Canvas"\)/);
assert.match(shell, /FOCUSA MISSION CANVAS · CURRENT PI SESSION/);
assert.match(shell, /PROMPT EDITOR · To: Pi · current session · New Workpoint: \/focus-work/);
assert.match(shell, /setFooter\(undefined\)/);
assert.match(shell, /this\.dispose\(\);\n    this\.done\(\);/);
// Redraws must be event-driven; an unconditional shell interval causes terminal scroll storms.
assert.doesNotMatch(shell, /setInterval\(/);
assert.doesNotMatch(shell, /clearInterval\(this\.refreshTimer\)/);
assert.match(shell, /this\.input\.getValue\(\)/);
assert.match(shell, /setEditorText\(draft\)/);
assert.match(shell, /\/mission-canvas off restores stock Pi/);

// Reference-design composition exists in the Pi-native renderer.
assert.match(view, /class MissionCanvasView implements Component/);
assert.match(view, /Pi-native authoritative Mission Canvas/);
assert.match(view, /MissionCanvasActivity/);
for (const label of [
  "Overview", "Context", "Role", "Interview", "Spec", "Tasks / Work",
  "Sessions", "Documents", "Research", "Evidence", "History", "Controls",
]) assert.ok(view.includes(`"${label}"`), `missing activity ${label}`);
assert.match(view, /resolveContributions/);
assert.match(view, /meaningful/);
assert.match(view, /layoutMemory/);
assert.match(view, /Steering Queue/);
assert.match(view, /Follow-up Queue/);
assert.match(view, /Pi Transcript · live/);
assert.match(view, /Semantic Graph/);
assert.match(view, /Evidence Matrix/);
for (const action of ["/focusa-context", "/focusa-role", "/focusa-interview", "/focusa-crist", "/focusa-rail"]) assert.ok(view.includes(action), `missing generated action ${action}`);
assert.match(view, /Canvas ●/);
assert.match(view, /setInterval\(\(\) => void this\.refresh\(\), 5_000\)/);
assert.match(view, /clearInterval\(this\.refreshTimer\)/);
assert.doesNotMatch(view, /WHAT CHANGES/);
assert.doesNotMatch(view, /WHAT STAYS THE SAME/);
assert.doesNotMatch(view, /CURRENT WORKSPACE COCKPIT/);

// The current command surface owns the same-session Pi custom component and mode.
assert.match(commands, /new MissionCanvasShell/);
assert.match(commands, /canvas-guided/);
assert.match(commands, /hasActiveMissionCanvasShell/);
assert.match(commands, /refreshMissionCanvasWidget/);
assert.match(inventory, /projectSessionInventory/);
assert.match(model, /workpointId/);
assert.match(config, /missionCanvasWorkspaceProfile/);
assert.match(widget, /Mission Canvas/);

console.log("Mission Canvas Pi-native authoritative surface registration passed");
