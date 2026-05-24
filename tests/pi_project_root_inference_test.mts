import { mkdirSync, rmSync, writeFileSync } from "fs";
import { join } from "path";
import { S, adoptPiProjectRoot, resolvePiProjectRoot, resolvePiProjectRootCandidate, isProjectRootAuthoritySafe, ensurePiFrame } from "../apps/pi-extension/src/state.ts";

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

const portableRoot = `/tmp/focusa-portable-root-${process.pid}`;
const projectRootCache = join(portableRoot, "focusa-project-root-cache.json");
process.env.FOCUSA_PI_PROJECT_ROOT_CACHE = projectRootCache;
rmSync(portableRoot, { recursive: true, force: true });
mkdirSync(join(portableRoot, "project", "packages", "agent", "src"), { recursive: true });
mkdirSync(join(portableRoot, "project", ".git"), { recursive: true });
mkdirSync(join(portableRoot, "project", "packages", "agent", ".beads"), { recursive: true });
writeFileSync(join(portableRoot, "project", "packages", "agent", "package.json"), "{}");
const portableInferred = resolvePiProjectRoot(join(portableRoot, "project", "packages", "agent", "src"));
const portableResolution = resolvePiProjectRootCandidate(join(portableRoot, "project", "packages", "agent", "src"));
assert(portableInferred === join(portableRoot, "project"), `portable marker scoring should prefer git project root over nested agent markers, got ${portableInferred}`);
assert(portableResolution.confidenceScore >= 0.90 && portableResolution.requiresOperatorConfirmation === false, "git project root should be >=90% confidence without operator confirmation");

const packageOnlyRoot = join(portableRoot, "package-only");
mkdirSync(join(packageOnlyRoot, "src"), { recursive: true });
writeFileSync(join(packageOnlyRoot, "package.json"), "{}");
const packageOnlyResolution = resolvePiProjectRootCandidate(join(packageOnlyRoot, "src"));
assert(packageOnlyResolution.projectRoot === packageOnlyRoot, `package marker should produce a candidate root, got ${packageOnlyResolution.projectRoot}`);
assert(packageOnlyResolution.confidenceScore < 0.90 && packageOnlyResolution.requiresOperatorConfirmation === true, "package-only inference should require operator menu confirmation");
assert((packageOnlyResolution.candidates || []).length > 0, "low-confidence inference should expose candidate roots for menu selection");
Object.assign(S, { focusaAvailable: true, activeFrameId: null, activeFramePromise: null, sessionFrameKey: "low-confidence-session" });
adoptPiProjectRoot(join(packageOnlyRoot, "src"));
const lowConfidenceFrame = await ensurePiFrame(join(packageOnlyRoot, "src"), "low-confidence-session", "test-low-confidence");
assert(lowConfidenceFrame === null, "low-confidence project root must block automatic frame creation before operator confirmation");

const markerRoot = join(portableRoot, "marker-project");
mkdirSync(join(markerRoot, "nested", "tool"), { recursive: true });
writeFileSync(join(markerRoot, ".focusa-project.json"), "{}");
const markerInferred = resolvePiProjectRoot(join(markerRoot, "nested", "tool"));
assert(markerInferred === markerRoot, `portable focusa marker should define root on any directory layout, got ${markerInferred}`);
adoptPiProjectRoot(join(markerRoot, "nested", "tool"));
Object.assign(S, { sessionCwd: "" });
const rememberedRoot = resolvePiProjectRoot("/root");
assert(rememberedRoot === "/root", `unsafe /root must not reuse durable remembered project root from another tree, got ${rememberedRoot}`);
assert(!isProjectRootAuthoritySafe(rememberedRoot), "/root should remain unsafe even when a remembered project exists elsewhere");
const sameTreeRememberedRoot = resolvePiProjectRoot(join(markerRoot, "nested", "tool"));
assert(sameTreeRememberedRoot === markerRoot, `same-tree cwd should reuse durable remembered project root, got ${sameTreeRememberedRoot}`);

rmSync(projectRootCache, { force: true });
Object.assign(S, { sessionCwd: "" });
const unsafeRoot = resolvePiProjectRoot("/root");
assert(unsafeRoot === "/root", `no remembered project from /root should remain fail-closed, got ${unsafeRoot}`);
assert(!isProjectRootAuthoritySafe(unsafeRoot), "/root should remain unsafe without project evidence");

S.activeWorkpointPacket = {
  canonical: true,
  status: "active",
  project_root: "/home/wirebot/focusa",
  continuity_id: "focusa-cont-test",
};
const unsafeAdopted = adoptPiProjectRoot("/root");
assert(unsafeAdopted === "/root", `unsafe cwd must not default to unrelated active Workpoint root, got ${unsafeAdopted}`);

S.sessionFrameKey = "same-session";
const sameSessionResolution = resolvePiProjectRootCandidate("/root", {
  canonical: true,
  status: "active",
  project_root: "/home/wirebot/focusa",
  continuity_id: "focusa-cont-test",
  pi_session_frame_key: "same-session",
});
assert(sameSessionResolution.projectRoot === "/home/wirebot/focusa", "same-session Workpoint packet may offer a project root candidate");
assert(sameSessionResolution.requiresOperatorConfirmation === true, "same-session Workpoint root still requires confirmation below 90% confidence");

rmSync(portableRoot, { recursive: true, force: true });

console.log("Pi project root inference proof passed");
