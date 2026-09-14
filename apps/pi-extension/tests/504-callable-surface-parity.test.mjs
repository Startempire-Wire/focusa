import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

const root = new URL("../../../", import.meta.url);
const read = (path) => readFileSync(new URL(path, root), "utf8");

test("silent completion routes are compiled and mounted", () => {
  const modules = read("crates/focusa-api/src/routes/mod.rs");
  const server = read("crates/focusa-api/src/server.rs");
  assert.match(modules, /pub mod silent_sessions_wait;/);
  assert.match(server, /\.merge\(routes::silent_sessions_wait::router\(\)\)/);
});

test("accepted orphan CLI modules are callable", () => {
  const modules = read("crates/focusa-cli/src/commands/mod.rs");
  const main = read("crates/focusa-cli/src/main.rs");
  for (const moduleName of [
    "workset",
    "workstream",
    "remote",
    "infra",
    "rebuild_state",
    "callgraph",
  ]) {
    assert.match(modules, new RegExp(`pub mod ${moduleName};`));
  }
  for (const variant of [
    "Workset",
    "Workstream",
    "Remote",
    "Infra",
    "RebuildState",
    "Callgraph",
  ]) {
    assert.match(main, new RegExp(`Commands::${variant}\\(args\\) =>`));
  }
  const rebuildState = read("crates/focusa-cli/src/commands/rebuild_state.rs");
  assert.match(rebuildState, /if !args\.dry_run && !args\.confirm/);
  assert.match(rebuildState, /pass --confirm or use --dry-run/);
});

test("direct Pi callers normalize to exactly one v1 path segment", () => {
  const tools = read("apps/pi-extension/src/tools.ts");
  assert.match(tools, /function focusaApiV1Base\(\): string/);
  assert.match(tools, /configured\.endsWith\("\/v1"\) \? configured : `\$\{configured\}\/v1`/);
  assert.doesNotMatch(tools, /fetch\(`\$\{base\}\/v1\/(?:worksets|callgraph-runs|silent-sessions\/fanout)/);
  assert.match(tools, /fetch\(`\$\{base\}\/worksets\/\$\{encodeURIComponent\(params\.workset_id\)\}\/projection`\)/);
  assert.match(tools, /fetch\(`\$\{base\}\/silent-sessions\/fanout`,/);
});
