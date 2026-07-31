import assert from "node:assert/strict";
import test from "node:test";
import { readFileSync } from "node:fs";
import ts from "typescript";

const source = readFileSync(new URL("../src/session-classification.ts", import.meta.url), "utf8");
const compiled = ts.transpileModule(source, {
  compilerOptions: {
    module: ts.ModuleKind.ESNext,
    target: ts.ScriptTarget.ES2022,
  },
}).outputText;
const classification = await import(
  `data:text/javascript;base64,${Buffer.from(compiled).toString("base64")}`
);
const { classifyPiSessionProject } = classification;

const base = {
  currentProjectRoot: "/projects/focusa",
  markerExists: true,
  persistedStateFound: true,
  persistedProjectRoot: "/projects/focusa",
};

test("known resumed session and verified project rehydrate without onboarding", () => {
  assert.equal(classifyPiSessionProject({ ...base, reason: "resume" }), "resumed_session_resumed_project");
});

test("missing marker with same-root durable state is recoverable before prompting", () => {
  assert.equal(
    classifyPiSessionProject({ ...base, reason: "resume", markerExists: false }),
    "resumed_session_recoverable_project"
  );
});

test("new session in a known verified project does not repeat project onboarding", () => {
  assert.equal(
    classifyPiSessionProject({
      ...base,
      reason: "new",
      persistedStateFound: false,
      persistedProjectRoot: undefined,
    }),
    "new_session_existing_project"
  );
});

test("resumed session rebinds safely across verified worktrees in one canonical project", () => {
  assert.equal(
    classifyPiSessionProject({
      ...base,
      reason: "resume",
      currentProjectRoot: "/projects/focusa-worktree",
      sameCanonicalProject: true,
    }),
    "resumed_session_worktree_rebound"
  );
  assert.equal(
    classifyPiSessionProject({ ...base, reason: "resume", bindingAmbiguous: true }),
    "session_project_mismatch"
  );
});

test("project mismatch fails closed and fork metadata preserves continuation", () => {
  assert.equal(
    classifyPiSessionProject({ ...base, reason: "resume", currentProjectRoot: "/projects/other" }),
    "session_project_mismatch"
  );
  assert.equal(classifyPiSessionProject({ ...base, reason: "fork" }), "forked_compacted_continuation");
});

test("Pi 0.81 lifecycle consumes classification instead of obsolete post-switch events", () => {
  const sessionSource = readFileSync(new URL("../src/session.ts", import.meta.url), "utf8");
  assert.match(sessionSource, /ctx\.sessionManager\.getSessionId\(\)/);
  assert.match(sessionSource, /pi_session_project_classified/);
  assert.match(sessionSource, /session_project_mismatch_blocked/);
  assert.match(sessionSource, /binding_decision/);
  assert.match(sessionSource, /persisted_project_root/);
  assert.match(sessionSource, /sameCanonicalProject/);
  assert.match(sessionSource, /sameRepoFingerprint/);
  assert.match(sessionSource, /sameProjectFingerprint/);
  assert.match(sessionSource, /persisted_binding_conflicts_with_current_repo/);
  const mismatchStart = sessionSource.indexOf('sessionProjectClassification === "session_project_mismatch"');
  const mismatchEnd = sessionSource.indexOf(
    "const projectRoot = await promptForConfirmedProjectRoot",
    mismatchStart
  );
  const mismatchBlock = sessionSource.slice(mismatchStart, mismatchEnd);
  assert.match(mismatchBlock, /Candidate selection is deferred until a project-aware mutation is requested/);
  assert.doesNotMatch(mismatchBlock, /queueProjectIdentityBootstrapTurn/);
  assert.ok(
    sessionSource.indexOf('sessionProjectClassification === "new_session_new_project"') <
      sessionSource.indexOf('queueUnboundProjectNag(pi, ctx, "new_session_new_project")'),
    "onboarding advisory must route only after session/project classification"
  );
  assert.doesNotMatch(sessionSource, /pi\.on\("session_switch"/);
  assert.doesNotMatch(sessionSource, /pi\.on\("session_fork"/);
});
