import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import ts from "typescript";

const source = readFileSync(
  fileURLToPath(new URL("../src/rehydrate.ts", import.meta.url)),
  "utf8"
);
const compiled = ts.transpileModule(source, {
  compilerOptions: { module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ES2022 },
}).outputText;
const { parseRehydrateArgs, rehydrateHandle } = await import(
  `data:text/javascript;base64,${Buffer.from(compiled).toString("base64")}`
);

assert.deepEqual(parseRehydrateArgs("local-123 50"), { handleId: "local-123", maxTokens: 50 });
assert.deepEqual(parseRehydrateArgs("019f-test"), { handleId: "019f-test", maxTokens: 300 });
assert.equal(parseRehydrateArgs("019f-test 99999").maxTokens, 4000);
assert.throws(() => parseRehydrateArgs("../secret"), /Usage/);
assert.throws(() => parseRehydrateArgs("019f-test zero"), /positive integer/);

const local = await rehydrateHandle("local-123 2", {
  getLocal: (_kind, id) => (id === "local-123" ? "abcdefghijk" : null),
  fetchRemote: async () => { throw new Error("remote must not run"); },
});
assert.equal(local.source, "local");
assert.equal(local.content, "abcdefgh…");
assert.equal(local.truncated, true);

let requested = "";
const remote = await rehydrateHandle("019f-remote 42", {
  getLocal: () => null,
  fetchRemote: async (path, init) => {
    requested = `${init.method} ${path}`;
    return { content: "remote body", truncated: false, original_size: 11 };
  },
});
assert.equal(requested, "POST /ecs/rehydrate/019f-remote?max_tokens=42");
assert.deepEqual(remote, {
  handleId: "019f-remote",
  content: "remote body",
  source: "ecs",
  truncated: false,
  originalSize: 11,
});

const commands = readFileSync(
  fileURLToPath(new URL("../src/commands.ts", import.meta.url)),
  "utf8"
);
assert.match(commands, /registerCommand\("focusa-rehydrate"/);
assert.match(commands, /getEcsArtifact/);
assert.match(commands, /deliverAs: "nextTurn"/);
console.log("focusa rehydrate command runtime contract passed");
