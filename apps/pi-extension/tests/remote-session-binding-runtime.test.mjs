import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const state = readFileSync(fileURLToPath(new URL("../src/state.ts", import.meta.url)), "utf8");
const session = readFileSync(fileURLToPath(new URL("../src/session.ts", import.meta.url)), "utf8");
const index = readFileSync(fileURLToPath(new URL("../src/index.ts", import.meta.url)), "utf8");
const project = readFileSync(
  fileURLToPath(new URL("../../../crates/focusa-api/src/routes/project.rs", import.meta.url)),
  "utf8"
);

assert.match(session, /sessionManager\.getSessionId\(\)/);
assert.ok(
  index.indexOf("ctx?.sessionManager?.getSessionId?.()") <
    index.indexOf("ctx?.sessionManager?.getSessionFile?.()"),
  "attachment routing must prefer native Pi UUID over session file path"
);
assert.doesNotMatch(index, /sessionId\s*[:=][^\n]*getSessionFile\(/);
assert.doesNotMatch(session, /sessionId\s*[:=][^\n]*getSessionFile\(/);
assert.match(state, /query\.set\("pi_session_id", sessionId\)/);
for (const field of [
  "remote_host",
  "remote_user",
  "remote_port",
  "remote_repo_remote",
  "remote_workspace_kind",
  "remote_deploy_root",
]) {
  assert.match(state, new RegExp(`query\\.set\\("${field}"`), `${field} must survive resume lookup`);
}
assert.match(project, /pub pi_session_id: Option<String>/);
assert.match(project, /identity\.insert\("pi_session_id"/);
assert.match(project, /remote_host_plus_project_root_plus_fingerprint/);
console.log("remote native-session binding propagation passed");
