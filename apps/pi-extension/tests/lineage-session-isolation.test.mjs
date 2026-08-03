import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import test from "node:test";

const tools = readFileSync(fileURLToPath(new URL("../src/tools.ts", import.meta.url)), "utf8");
const capabilities = readFileSync(
  fileURLToPath(new URL("../../../crates/focusa-api/src/routes/capabilities.rs", import.meta.url)),
  "utf8"
);
const clt = readFileSync(
  fileURLToPath(new URL("../../../crates/focusa-api/src/routes/clt.rs", import.meta.url)),
  "utf8"
);

function block(source, startToken, endToken) {
  const start = source.indexOf(startToken);
  const end = source.indexOf(endToken, start + startToken.length);
  assert.notEqual(start, -1, `missing ${startToken}`);
  assert.notEqual(end, -1, `missing ${endToken}`);
  return source.slice(start, end);
}

test("Pi lineage tools default to the active native session and never global lineage", () => {
  const spec80Caller = block(tools, "async function callSpec80Tool", "const SPEC81_ID_PATTERN");
  assert.match(spec80Caller, /"X-Scope-Session-Id": requestSessionId/);
  assert.match(spec80Caller, /"X-Scope-Project-Root": requestProjectRoot/);
  assert.match(spec80Caller, /"X-Scope-Continuity-Id": requestContinuityId/);

  const treeHead = block(tools, 'name: "focusa_tree_head"', 'name: "focusa_tree_path"');
  assert.match(treeHead, /getAttachmentRuntime\(\)\.sessionFrameKey/);
  assert.match(treeHead, /global lineage fallback is prohibited/);
  assert.match(treeHead, /lineage response session scope mismatch/);
  assert.doesNotMatch(treeHead, /\|\| "global"/);

  const lineageTree = block(tools, 'name: "focusa_lineage_tree"', 'name: "focusa_li_tree_extract"');
  assert.match(lineageTree, /getAttachmentRuntime\(\)\.sessionFrameKey/);
  assert.match(lineageTree, /session_scope_required/);
  assert.match(lineageTree, /response session scope mismatch/);
  assert.match(lineageTree, /scope_provenance/);

  const extract = block(tools, 'name: "focusa_li_tree_extract"', 'name: "focusa_predict_record"');
  assert.match(extract, /getAttachmentRuntime\(\)\.sessionFrameKey/);
  assert.match(extract, /session_scope_required/);
  assert.match(extract, /response session scope mismatch/);
});

test("server lineage reads require exact project continuity and Pi session scope", () => {
  assert.match(clt, /!session_id\.is_empty\(\)/);
  assert.match(clt, /node\.session_id[\s\S]*Some\(session_id\)/);
  assert.match(clt, /missing_session_scope/);
  assert.match(clt, /scoped_clt_state\(&clt, &missing_session_scope\)/);
  assert.match(clt, /\.nodes\s*\.is_empty\(\)/);

  assert.match(capabilities, /fn exact_lineage_session_id/);
  assert.match(capabilities, /session_scope_required/);
  assert.match(capabilities, /Global or most-recent lineage fallback is prohibited/);
  assert.match(capabilities, /"global_fallback": false/);
  assert.match(capabilities, /lineage session scope mismatch/);
  assert.match(capabilities, /Never use foreign lineage as mutation context/);

  const scopedCalls = capabilities.match(/scoped_clt_state\(&s\.clt, &scope\)/g) || [];
  assert.ok(scopedCalls.length >= 7, `expected all lineage handlers scoped, saw ${scopedCalls.length}`);
});
