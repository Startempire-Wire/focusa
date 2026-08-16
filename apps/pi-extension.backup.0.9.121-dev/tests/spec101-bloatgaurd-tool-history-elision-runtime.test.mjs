import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const projectDir = fileURLToPath(new URL("..", import.meta.url));
const outDir = mkdtempSync(join(tmpdir(), "focusa-bloatgaurd-tool-history-"));
const turnsSource = readFileSync(new URL("../src/turns.ts", import.meta.url), "utf8");

const handledText = [
  '[HANDLE:text:019f-test-old "bash output" (9000 bytes, ~2200 tokens)]',
  "TRAJECTORY_CONTEXT: repeated prompt-visible context",
  "Use /focusa-rehydrate 019f-test-old to retrieve full content.",
].join("\n");
const oldHandled = {
  role: "toolResult",
  toolCallId: "call-old",
  toolName: "bash",
  isError: false,
  content: [{ type: "text", text: handledText }],
};
const oldRaw = {
  role: "toolResult",
  toolCallId: "call-raw",
  toolName: "read",
  isError: false,
  content: [{ type: "text", text: "raw output without a rehydration handle" }],
};
const oldError = {
  role: "toolResult",
  toolCallId: "call-error",
  toolName: "bash",
  isError: true,
  content: [{ type: "text", text: handledText }],
};
const recentHandled = {
  role: "toolResult",
  toolCallId: "call-recent",
  toolName: "bash",
  isError: false,
  content: [{ type: "text", text: handledText.replaceAll("old", "recent") }],
};
const messages = [
  oldHandled,
  oldRaw,
  oldError,
  { role: "assistant", content: [{ type: "text", text: "keep tool-call pairing" }] },
  recentHandled,
];

try {
  symlinkSync(join(projectDir, "node_modules"), join(outDir, "node_modules"), "dir");
  execFileSync(
    "./node_modules/.bin/tsc",
    ["-p", "tsconfig.json", "--outDir", outDir, "--noEmit", "false", "--module", "ES2022"],
    { cwd: projectDir, stdio: "pipe" }
  );
  writeFileSync(join(outDir, "package.json"), '{"type":"module"}\n');

  const turns = await import(pathToFileURL(join(outDir, "turns.js")).href);
  const result = turns.elideOldRehydratableToolHistory(messages, 2);

  assert.notEqual(result, messages, "elision should return a new message array");
  assert.notEqual(result[0], oldHandled, "old handled result should be copied and collapsed");
  assert.equal(result[0].toolCallId, oldHandled.toolCallId, "tool-call identity must be preserved");
  assert.equal(result[0].toolName, oldHandled.toolName, "tool name must be preserved");
  assert.deepEqual(result[0].content, [
    {
      type: "text",
      text: '[HANDLE:text:019f-test-old "bash output" (9000 bytes, ~2200 tokens)]\nUse /focusa-rehydrate 019f-test-old to retrieve full content.',
    },
  ]);
  assert.equal(result[1], oldRaw, "raw output without a stable handle must not be elided");
  assert.equal(result[2], oldError, "error evidence must not be elided");
  assert.equal(result[4], recentHandled, "bounded recent tool history must remain verbatim");
  assert.equal(oldHandled.content[0].text, handledText, "input messages must not be mutated");
  assert.deepEqual(turns.elideOldRehydratableToolHistory(result, 2), result, "elision must be idempotent");

  assert.equal(
    (turnsSource.match(/pi\.on\("context"/g) || []).length,
    1,
    "tool-history elision must reuse the single context hook"
  );
  assert.match(
    turnsSource,
    /const contextMessages = elideOldRehydratableToolHistory\(event\.messages \|\| \[\]\)/
  );

  console.log("spec101 bloatgaurd tool-history elision runtime test passed");
} finally {
  rmSync(outDir, { recursive: true, force: true });
}
