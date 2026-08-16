import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const srcDir = fileURLToPath(new URL("../src/", import.meta.url));
const files = readdirSync(srcDir).filter((name) => name.endsWith(".ts"));
const sourceByFile = new Map(files.map((name) => [name, readFileSync(join(srcDir, name), "utf8")]));

const forbiddenRuntimeName = "runtime" + "State";
for (const [name, source] of sourceByFile) {
  assert(!/\bexport\s+const\s+S\b/.test(source), `${name}: forbidden Pi S singleton export`);
  assert(!/\bS\s*\./.test(source), `${name}: forbidden direct S authority access`);
  assert(!/\bimport\s*\{[^}]*\bS\b/.test(source), `${name}: forbidden S import`);
  assert(
    !new RegExp(`\\bexport\\s+const\\s+${forbiddenRuntimeName}\\b`).test(source),
    `${name}: forbidden exported mutable runtime singleton`
  );
  assert(
    !new RegExp(`\\b${forbiddenRuntimeName}\\s*\\.`).test(source),
    `${name}: forbidden direct mutable runtime authority access`
  );
}

const state = sourceByFile.get("state.ts");
assert(state.includes("class TypedScopeStore"), "state.ts must retain typed scope stores");
assert(
  state.includes("function getAttachmentRuntime"),
  "state.ts must expose explicit attachment runtime resolver"
);
assert(
  sourceByFile.get("compaction.ts").includes("buildProjectWorkstreamKey"),
  "compaction must build explicit workstream keys"
);
const runtimeObject = state.slice(
  state.indexOf("function createAttachmentRuntime()"),
  state.indexOf("export type AttachmentRuntimeState")
);
assert(runtimeObject.length > 0, "attachment runtime object must be created per typed key");
assert(
  !state.includes("const attachmentRuntime = {"),
  "module-global attachment runtime object is forbidden"
);
assert(state.includes("class AttachmentRuntimeRegistry"), "typed attachment runtime registry is required");
assert(
  state.includes("AsyncLocalStorage<AttachmentKey>"),
  "AsyncLocalStorage-bound attachment key is required"
);
assert(
  !/lastProjectRootResolution\s*:/.test(runtimeObject),
  "project root resolution authority must not live in runtime singleton object"
);
assert(
  !/activeWorkpointPacket\s*:/.test(runtimeObject),
  "active Workpoint authority must not live in runtime singleton object"
);
assert(
  !/lastTrajectoryClarity\s*:/.test(runtimeObject),
  "trajectory shadow authority must not live in runtime singleton object"
);
assert(
  !/lastProjectIdentity\s*:/.test(runtimeObject),
  "identity shadow authority must not live in runtime singleton object"
);

const scoped = sourceByFile.get("scoped-state.ts");
assert(scoped.includes("export interface ScopeRef"), "ScopeRef contract is required");
assert(scoped.includes("export interface WorkstreamKey"), "WorkstreamKey contract is required");
assert(scoped.includes("export interface AttachmentKey"), "AttachmentKey contract is required");
assert(scoped.includes("reconcileScopedRecord"), "scoped CRDT reconcile contract is required");

const compaction = sourceByFile.get("compaction.ts");
assert(/scope:\s*WorkstreamKey/.test(compaction), "compaction must pass typed workstream scope");
assert(/currentCompactionScope\(\)/.test(compaction), "compaction must derive an explicit scope object");
assert(
  !/process\.cwd\(\)\)\s*\|\|/.test(compaction),
  "compaction must not fallback from cwd into prior scope"
);

const tools = sourceByFile.get("tools.ts");
const transferStart = tools.indexOf('name: "focusa_session_transfer"');
const transferEnd = tools.indexOf('name: "focusa_project_verify"', transferStart);
assert(transferStart > 0 && transferEnd > transferStart, "focusa_session_transfer tool block must exist");
const transfer = tools.slice(transferStart, transferEnd);
for (const required of [
  "source_scope",
  "target_scope",
  "target_continuity_id",
  "source_session_id",
  "target_session_id",
  "checkpoint_ref",
  "workpoint_packet_ref",
  "compaction_packet_ref",
  "rollover_action",
]) {
  assert(transfer.includes(required), `focusa_session_transfer missing ${required}`);
}
assert(
  !/ensureContinuityId\s*\(/.test(transfer),
  "focusa_session_transfer must not derive continuity from project identity/fingerprint"
);
assert(
  /sourceScope\s*=\s*buildProjectWorkstreamKey/.test(transfer) &&
    /targetScope\s*=\s*buildProjectWorkstreamKey/.test(transfer),
  "focusa_session_transfer must build explicit source and target workstream scopes"
);

console.log("spec104 Pi runtime isolation static/runtime model checks passed");
