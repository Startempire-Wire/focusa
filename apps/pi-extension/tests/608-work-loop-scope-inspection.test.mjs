import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

const tools = readFileSync(new URL("../src/tools.ts", import.meta.url), "utf8");
const route = readFileSync(
  new URL("../../../crates/focusa-api/src/routes/work_loop.rs", import.meta.url),
  "utf8",
);

assert.match(route, /fn read_only_scope_inspection_allowed/);
assert.match(route, /read_only_scope_inspection_allowed\(&parts\.method, parts\.uri\.path\(\)\)/);
assert.match(route, /"\/v1\/work-loop\/status"/);
assert.match(route, /assert!\(!read_only_scope_inspection_allowed\(\s*&Method::GET,\s*"\/v1\/work-loop\/enable"/s);

assert.match(tools, /active_execution_scope/);
assert.match(tools, /requested_scope/);
assert.match(tools, /active=\{\$\{active\}\} requested=\{\$\{requested\}\}/);
assert.doesNotMatch(
  tools,
  /expected=\$\{expected\} packet=\$\{actual\}; \$\{hint\}/,
  "scope mismatch output must not collapse typed scopes to unknown/unknown",
);

console.log("Issue #608 read-only scope inspection and diagnostics passed");
