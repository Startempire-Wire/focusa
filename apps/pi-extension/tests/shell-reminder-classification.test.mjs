import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import test from "node:test";
import ts from "typescript";

const classifierSource = readFileSync(
  fileURLToPath(new URL("../src/shell-reminder-classification.ts", import.meta.url)),
  "utf8"
);
const compiled = ts.transpileModule(classifierSource, {
  compilerOptions: { module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ES2022 },
}).outputText;
const { classifyShellReminderInteraction } = await import(
  `data:text/javascript;base64,${Buffer.from(compiled).toString("base64")}`
);

const unrelated = [
  "gh issue list --state open",
  "git status --short",
  "rg -n reminder apps/pi-extension/src",
  "cargo test -p focusa-core",
  "npm run typecheck",
];

test("ordinary development shell commands never trigger a Focusa reminder", () => {
  for (const command of unrelated) {
    const result = classifyShellReminderInteraction("bash", { command });
    assert.equal(result.classification, "shell_used", command);
    assert.equal(result.equivalent_tool, null, command);
  }
});

test("raw Focusa API bypass maps to a concrete governed equivalent", () => {
  const cases = [
    ["curl -fsS http://127.0.0.1:8787/v1/project/identity", "focusa_project_identity"],
    ["curl -X POST http://localhost:8787/v1/workpoint/resume", "focusa_workpoint_resume"],
    ["wget -qO- http://127.0.0.1:8787/v1/trajectory/view", "focusa_trajectory_view"],
    ["curl http://127.0.0.1:8787/v1/lineage/head", "focusa_tree_head"],
  ];
  for (const [command, equivalent] of cases) {
    const result = classifyShellReminderInteraction("bash", { command });
    assert.equal(result.classification, "actual_focusa_bypass");
    assert.equal(result.confidence, "high");
    assert.equal(result.equivalent_tool, equivalent);
  }
});

test("uncertain or no-equivalent Focusa-like commands remain silent", () => {
  for (const command of [
    "echo http://127.0.0.1:8787/v1/project/identity",
    "curl http://127.0.0.1:8787/v1/health",
    "systemctl status focusa-daemon",
  ]) {
    const result = classifyShellReminderInteraction("bash", { command });
    assert.equal(result.classification, "shell_used");
    assert.equal(result.equivalent_tool, null);
  }
});

test("reminder runtime preserves cooldown/frequency and typed telemetry", () => {
  const polish = readFileSync(fileURLToPath(new URL("../src/polish.ts", import.meta.url)), "utf8");
  const config = readFileSync(fileURLToPath(new URL("../src/config.ts", import.meta.url)), "utf8");
  assert.match(polish, /classification === "actual_focusa_bypass"/);
  assert.match(polish, /confidence === "high"/);
  assert.match(polish, /equivalent_tool/);
  assert.match(polish, /agent_shell_classification/);
  assert.match(polish, /now - lastReminder > cooldownMs/);
  assert.match(polish, /Math\.max\(2, reminderCfg\.agentReminderShellFrequency \|\| 3\)/);
  assert.match(config, /agentReminderShellFrequency: 3/);
});
