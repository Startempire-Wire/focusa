import { S, adoptPiProjectRoot, resolvePiProjectRoot, isProjectRootAuthoritySafe } from "../apps/pi-extension/src/state.ts";

function assert(cond: any, msg: string) {
  if (!cond) throw new Error(msg);
}

Object.assign(S, {
  sessionCwd: "",
  activeWorkpointPacket: null,
  continuityId: "",
});

const repoRoot = resolvePiProjectRoot("/home/wirebot/focusa/apps/pi-extension/src");
assert(repoRoot === "/home/wirebot/focusa", `expected repo root, got ${repoRoot}`);
assert(isProjectRootAuthoritySafe(repoRoot), "repo root should be safe");

const unsafeRoot = resolvePiProjectRoot("/root");
assert(unsafeRoot === "/root", `no explicit project from /root should remain fail-closed, got ${unsafeRoot}`);
assert(!isProjectRootAuthoritySafe(unsafeRoot), "/root should remain unsafe without project evidence");

S.activeWorkpointPacket = {
  canonical: true,
  status: "active",
  project_root: "/home/wirebot/focusa",
  continuity_id: "focusa-cont-test",
};
const adopted = adoptPiProjectRoot("/root");
assert(adopted === "/home/wirebot/focusa", `unsafe cwd should adopt active Workpoint project root, got ${adopted}`);
assert(S.sessionCwd === "/home/wirebot/focusa", "adoptPiProjectRoot should update sessionCwd");

console.log("Pi project root inference proof passed");
