import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { homedir } from "node:os";
import test from "node:test";
import {
  canReuseFreshVerifiedBindingOffline,
  projectBindingAllowsDurableWrites,
  reconcileProjectBindingDecision,
} from "../src/project-binding.ts";

const sessionSource = readFileSync(new URL("../src/session.ts", import.meta.url), "utf8");
const macHome = homedir().replace(/\/+$/, "");

const candidate = {
  project_root: `${macHome}/Projects/focusa`,
  active_worktree_root: `${macHome}/Projects/focusa-worktree`,
  canonical_parent_root: `${macHome}/Projects/focusa`,
  score: 950,
  sources: ["git", "focusa_marker"],
  markers: ["git", "focusa_marker"],
  repo_fingerprint: "repo:focusa",
  project_fingerprint: "project:focusa",
};

test("direct macOS user home is quarantined and never promoted", () => {
  assert.match(macHome, /\/Users\/[^/]+$/);
  const decision = reconcileProjectBindingDecision({
    selectedProjectRoot: macHome,
    candidates: [{ ...candidate, project_root: macHome }],
    selectedRootSafe: false,
    verificationCanonical: false,
    verificationStatus: "unsafe",
    daemonAvailable: true,
    effectiveAt: "2026-07-31T00:00:00.000Z",
  });
  assert.equal(decision.state, "QUARANTINED");
  assert.equal(projectBindingAllowsDurableWrites(decision), false);
  assert.match(decision.rejection_reasons.join(" "), /unsafe_selected_root/);
});

test("verified marked macOS project binds without a verification modal", () => {
  const decision = reconcileProjectBindingDecision({
    selectedProjectRoot: candidate.project_root,
    selectedWorktreeRoot: candidate.active_worktree_root,
    canonicalParentRoot: candidate.canonical_parent_root,
    candidates: [candidate],
    selectedRootSafe: true,
    verificationCanonical: true,
    verificationStatus: "verified",
    daemonAvailable: true,
    evidenceFreshness: "current",
    repoFingerprint: candidate.repo_fingerprint,
    projectFingerprint: candidate.project_fingerprint,
    effectiveAt: "2026-07-31T00:00:00.000Z",
  });
  const verifyStart = sessionSource.indexOf("async function promptForProjectVerifyIfNeeded");
  const verifyEnd = sessionSource.indexOf("async function promptForWorkpointIfNeeded", verifyStart);
  assert.equal(decision.state, "BOUND");
  assert.equal(projectBindingAllowsDurableWrites(decision), true);
  assert.doesNotMatch(sessionSource.slice(verifyStart, verifyEnd), /ctx\.ui\.confirm/);
});

test("same-project worktree fingerprint permits bounded fresh offline reuse", () => {
  const decision = reconcileProjectBindingDecision({
    selectedProjectRoot: candidate.project_root,
    selectedWorktreeRoot: candidate.active_worktree_root,
    canonicalParentRoot: candidate.canonical_parent_root,
    candidates: [candidate],
    selectedRootSafe: true,
    verificationCanonical: true,
    verificationStatus: "verified",
    daemonAvailable: true,
    evidenceFreshness: "current",
    repoFingerprint: candidate.repo_fingerprint,
    projectFingerprint: candidate.project_fingerprint,
    effectiveAt: "2026-07-31T00:00:00.000Z",
  });
  assert.equal(
    canReuseFreshVerifiedBindingOffline(decision, {
      selectedProjectRoot: candidate.project_root,
      repoFingerprint: candidate.repo_fingerprint,
      nowMs: Date.parse("2026-07-31T00:05:00.000Z"),
    }),
    true
  );
});

test("stale different-repo evidence cannot reuse authority", () => {
  const decision = reconcileProjectBindingDecision({
    selectedProjectRoot: candidate.project_root,
    candidates: [candidate],
    selectedRootSafe: true,
    verificationCanonical: true,
    verificationStatus: "verified",
    daemonAvailable: true,
    evidenceFreshness: "current",
    repoFingerprint: candidate.repo_fingerprint,
    projectFingerprint: candidate.project_fingerprint,
    effectiveAt: "2026-07-31T00:00:00.000Z",
  });
  assert.equal(
    canReuseFreshVerifiedBindingOffline(decision, {
      selectedProjectRoot: `${macHome}/Projects/other`,
      repoFingerprint: "repo:other",
      nowMs: Date.parse("2026-07-31T00:05:00.000Z"),
    }),
    false
  );
});
