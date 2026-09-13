import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import ts from "typescript";
import { Type } from "@sinclair/typebox";
import { Value } from "@sinclair/typebox/value";

const read = (path) => readFileSync(new URL(path, import.meta.url), "utf8");
const source = read("../src/tools.ts");
const anchor = source.indexOf('name: "focusa_credentials_verify"');
assert.notEqual(anchor, -1);
const start = source.lastIndexOf("pi.registerTool({", anchor);
const end = source.indexOf("pi.registerTool({", anchor);
assert.ok(start >= 0 && end > start);
const compiled = ts.transpileModule(source.slice(start, end), {
  compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.None },
}).outputText;

// Reuse the request exercised by the real Rust HTTP regression, not a second fixture.
const route = read("../../../crates/focusa-api/src/routes/credentials.rs");
const fixtureMatch = route.match(/let mut fixture = json!\(([\s\S]*?)\);/);
assert.ok(fixtureMatch, "credential HTTP fixture missing");
const fixture = JSON.parse(fixtureMatch[1]);
const input = { requirement: fixture.requirement, grants: fixture.grants };
const core = read("../../../crates/focusa-core/src/credential_authority.rs");
const fields = core.match(/pub struct CredentialRequirement \{([\s\S]*?)\n\}/);
assert.ok(fields, "requirement DTO missing");
const required = [];
let defaulted = false;
for (const line of fields[1].split("\n")) {
  if (line.includes("#[serde(default)]")) defaulted = true;
  const field = line.match(/pub (\w+):/);
  if (field) {
    if (!defaulted) required.push(field[1]);
    defaulted = false;
  }
}

function load(fetch) {
  let tool;
  new Function("pi", "Type", "focusaFetchDetailed", "toolResult", compiled)(
    { registerTool: (value) => { tool = value; } }, Type, fetch,
    (ok, status, message, data) => ({ ok, status, message, data }),
  );
  return tool;
}

test("credential tool declares the existing Rust requirement contract", () => {
  const tool = load(() => { throw new Error("schema inspection must not fetch"); });
  const schema = tool.parameters.properties.requirement;
  for (const key of required) {
    assert.ok(schema.required.includes(key), `missing required field: ${key}`);
  }
  for (const key of Object.keys(fixture.requirement)) {
    assert.ok(schema.properties[key], `undeclared fixture field: ${key}`);
  }
  assert.ok(Value.Check(tool.parameters, input));
  assert.match(tool.description, /advisory/i);
});

test("credential tool forwards identity and keeps both model verdicts advisory", async () => {
  for (const satisfied of [false, true]) {
    const tool = load(async (path, options) => {
      assert.equal(path, "/credentials/verify-requirement");
      assert.deepEqual(JSON.parse(options.body), input);
      return { ok: true, body: {
        satisfied, advisory: true, canonical: false,
        reasons: satisfied ? [] : ["no grant matches the requirement"],
      } };
    });
    const result = await tool.execute("fixture-call", structuredClone(input));
    assert.equal(result.ok, satisfied);
    assert.equal(result.status, satisfied ? "satisfied" : "denied");
    assert.equal(result.data.canonical, false);
    assert.equal(result.data.advisory, true);
    assert.match(result.message, /advisory/i);
    if (satisfied) assert.match(result.message, /authorization is not established/);
  }
});
