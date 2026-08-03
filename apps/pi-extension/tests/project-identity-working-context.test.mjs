import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import ts from "typescript";

const source = readFileSync(
  fileURLToPath(new URL("../src/project-identity-working-context.ts", import.meta.url)),
  "utf8"
);
const compiled = ts.transpileModule(source, {
  compilerOptions: { module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ES2022 },
}).outputText;
const { resolveCanonicalMarkerProjectRoot, resolveProjectIdentityLookupCwd } = await import(
  `data:text/javascript;base64,${Buffer.from(compiled).toString("base64")}`
);

assert.equal(
  resolveCanonicalMarkerProjectRoot(fileURLToPath(new URL("..", import.meta.url))),
  "/home/wirebot/focusa",
  "a linked worktree marker declares the canonical parent independently of ambient cwd"
);
assert.equal(
  resolveCanonicalMarkerProjectRoot("/root"),
  "",
  "broad unmarked launchers must not invent canonical project authority"
);

const richIdentity = {
  status: "verified",
  canonical_parent_root: "/home/wirebot/focusa",
  project_root: "/home/wirebot/focusa",
  active_worktree_root: "/tmp/focusa-next-locked-release",
  working_context: {
    active_worktree_root: "/tmp/focusa-next-locked-release",
    working_subpath: { working_subpath_id: "working-subpath:06c27313" },
  },
};

assert.equal(
  resolveProjectIdentityLookupCwd({
    projectRoot: "/home/wirebot/focusa",
    ambientCwd: "/root",
    persistedIdentity: richIdentity,
  }),
  "/tmp/focusa-next-locked-release",
  "a broad launcher cwd must not collapse a verified resumed worktree to primary"
);
assert.equal(
  resolveProjectIdentityLookupCwd({
    projectRoot: "/home/wirebot/focusa",
    ambientCwd: "/home/wirebot/focusa/apps/pi-extension",
    persistedIdentity: richIdentity,
  }),
  "/home/wirebot/focusa/apps/pi-extension",
  "an ambient cwd inside the canonical project is authoritative for the lookup"
);
assert.equal(
  resolveProjectIdentityLookupCwd({
    projectRoot: "/home/wirebot/focusa",
    ambientCwd: "/root",
    persistedIdentity: { ...richIdentity, canonical_parent_root: "/tmp/foreign-project" },
  }),
  "/home/wirebot/focusa",
  "foreign persisted working context must fail closed"
);
assert.equal(
  resolveProjectIdentityLookupCwd({
    projectRoot: "/home/wirebot/focusa",
    ambientCwd: "/root",
    persistedIdentity: {
      ...richIdentity,
      working_context: {
        ...richIdentity.working_context,
        working_subpath: { working_subpath_id: "primary" },
      },
    },
  }),
  "/home/wirebot/focusa",
  "primary context must not invent a detached worktree"
);
assert.equal(
  resolveProjectIdentityLookupCwd({
    projectRoot: "/home/wirebot/focusa",
    ambientCwd: "/root",
    persistedIdentity: { project_identity: richIdentity },
  }),
  "/tmp/focusa-next-locked-release",
  "wrapped project identity envelopes preserve the verified worktree"
);
const stateSource = readFileSync(
  fileURLToPath(new URL("../src/state.ts", import.meta.url)),
  "utf8"
);
assert.match(stateSource, /resolveProjectIdentityLookupCwd\(\{ projectRoot, ambientCwd, persistedIdentity: persisted \}\)/);
assert.doesNotMatch(
  stateSource,
  /const cwdForIdentity = safe && !ambientInsideProject \? projectRoot : ambientCwd/,
  "session identity must not collapse a verified resumed worktree to canonical primary"
);
const identityBuilderSource = stateSource.slice(
  stateSource.indexOf("export async function buildFocusaSessionIdentity"),
  stateSource.indexOf("export async function refreshTrajectoryClarity")
);
assert.doesNotMatch(
  identityBuilderSource,
  /query\.set\("project_root", projectRoot\)/,
  "ambient Pi cwd must not masquerade as operator-confirmed project_root authority"
);
assert.match(
  identityBuilderSource,
  /normalizeProjectRoot\(persistedBody\.project_root\) === authorityProjectRoot/,
  "stale ambient persistence must not conflict with marker-derived authority"
);
assert.match(
  stateSource,
  /decision\.selected_project_root \|\|\s*resolveCanonicalMarkerProjectRoot\(process\.cwd\(\)\)/,
  "verified continuity adoption must survive transition from bootstrap to typed attachment"
);
assert.match(stateSource, /verifiedContinuityBySessionRoot = new Map<string, string>\(\)/);
assert.match(
  stateSource,
  /verified \|\| store\?\.continuityId \|\| getAttachmentRuntime\(\)\.continuityId/,
  "exact verified continuity must outrank stale attachment-local continuity"
);
const toolsSource = readFileSync(
  fileURLToPath(new URL("../src/tools.ts", import.meta.url)),
  "utf8"
);
assert.match(
  toolsSource,
  /cachedVerified &&\s*cachedCanonical/,
  "unverified ambient identity cache must not bypass canonical marker resolution"
);
assert.match(
  toolsSource,
  /!explicit && markerCanonical && isProjectRootAuthoritySafe\(markerCanonical\)/,
  "scope-free Pi tools must promote a safe local canonical marker before ambient cwd"
);
assert.match(
  toolsSource,
  /const sessionCwd = ambientMarkerCanonical\s*\? ambientCwd/,
  "fresh process cwd with a marker must outrank stale broad session cwd"
);
assert.match(
  toolsSource,
  /const authorityProjectRoot = normalizeProjectRoot\(p\.project_root \|\| markerProjectRoot\)/,
  "project identity reads must bind a scope-free request to its local canonical marker"
);
assert.match(
  toolsSource,
  /query\.set\("pi_session_id", getSessionFrameKey\(\)\)/,
  "project identity reads must carry the native Pi session UUID"
);
assert.match(
  toolsSource,
  /fallback_source_continuity_id[\s\S]*adoptVerifiedContinuityForCurrentSession\(verifiedRoot, priorContinuity\)/,
  "verified project identity must recover the prior exact project continuity before later surface reads"
);
console.log("project identity working-context retention passed");
