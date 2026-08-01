import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import test from "node:test";
import ts from "typescript";

const source = readFileSync(fileURLToPath(new URL("../src/project-binding.ts", import.meta.url)), "utf8");
const sessionSource = readFileSync(fileURLToPath(new URL("../src/session.ts", import.meta.url)), "utf8");
const stateSource = readFileSync(fileURLToPath(new URL("../src/state.ts", import.meta.url)), "utf8");
const toolsSource = readFileSync(fileURLToPath(new URL("../src/tools.ts", import.meta.url)), "utf8");
const compiled = ts.transpileModule(source, {
  compilerOptions: {
    module: ts.ModuleKind.ESNext,
    target: ts.ScriptTarget.ES2022,
  },
}).outputText;
const binding = await import(`data:text/javascript;base64,${Buffer.from(compiled).toString("base64")}`);

const candidate = {
  project_root: "/repo/focusa",
  active_worktree_root: "/repo/focusa-wt",
  canonical_parent_root: "/repo/focusa",
  score: 950,
  sources: ["git", "focusa_marker"],
  markers: ["git", "focusa_marker"],
  repo_fingerprint: "repo:focusa",
  project_fingerprint: "project:focusa",
};

function decide(overrides = {}) {
  return binding.reconcileProjectBindingDecision({
    selectedProjectRoot: "/repo/focusa",
    selectedWorktreeRoot: "/repo/focusa-wt",
    canonicalParentRoot: "/repo/focusa",
    continuityId: "focusa-locked",
    candidates: [candidate],
    selectedRootSafe: true,
    verificationCanonical: true,
    verificationStatus: "verified",
    daemonAvailable: true,
    evidenceFreshness: "current",
    repoFingerprint: "repo:focusa",
    projectFingerprint: "project:focusa",
    effectiveAt: "2026-07-31T00:00:00.000Z",
    ...overrides,
  });
}

test("canonical verified project becomes BOUND with one deterministic receipt", () => {
  const first = decide();
  const second = decide({ effectiveAt: "2026-07-31T00:01:00.000Z" });
  assert.equal(first.state, "BOUND");
  assert.equal(first.permitted_capability_tier, "scoped");
  assert.equal(binding.projectBindingAllowsDurableWrites(first), true);
  assert.equal(first.decision_id, second.decision_id);
  assert.equal(first.binding_receipt_id, second.binding_receipt_id);
  assert.equal(binding.shouldEmitProjectScopeRecoveryPacket(first, second), false);
});

test("fresh matching verified binding can be reused offline without guessing", () => {
  const decision = decide();
  assert.equal(
    binding.canReuseFreshVerifiedBindingOffline(decision, {
      selectedProjectRoot: "/repo/focusa",
      repoFingerprint: "repo:focusa",
      nowMs: Date.parse("2026-07-31T00:05:00.000Z"),
    }),
    true
  );
  assert.equal(
    binding.canReuseFreshVerifiedBindingOffline(decision, {
      selectedProjectRoot: "/repo/other",
      repoFingerprint: "repo:focusa",
      nowMs: Date.parse("2026-07-31T00:05:00.000Z"),
    }),
    false
  );
  assert.equal(
    binding.canReuseFreshVerifiedBindingOffline(decision, {
      selectedProjectRoot: "/repo/focusa",
      repoFingerprint: "repo:other",
      nowMs: Date.parse("2026-07-31T00:30:00.000Z"),
    }),
    false
  );
});

test("daemon outage preserves conversation but fences project writes", () => {
  const decision = decide({
    verificationCanonical: false,
    verificationStatus: "unavailable",
    daemonAvailable: false,
    evidenceFreshness: "stale",
  });
  assert.equal(decision.state, "RECOVERING");
  assert.equal(decision.permitted_capability_tier, "recovery_read_plan");
  assert.equal(binding.projectBindingAllowsDurableWrites(decision), false);
  assert.equal(binding.shouldEmitProjectScopeRecoveryPacket(null, decision), true);
});

test("conflicting candidates quarantine without promoting either root", () => {
  const decision = decide({
    ambiguous: true,
    verificationCanonical: false,
    candidates: [candidate, { ...candidate, project_root: "/repo/other" }],
  });
  assert.equal(decision.state, "QUARANTINED");
  assert.equal(decision.permitted_capability_tier, "unbound_read_only");
  assert.match(decision.rejection_reasons.join(" "), /conflicting_strong_candidates/);
});

test("unsafe broad roots fail closed", () => {
  const decision = decide({
    selectedProjectRoot: "/Volumes/Macintosh HD/Users/vsmith",
    selectedRootSafe: false,
    verificationCanonical: false,
  });
  assert.equal(decision.state, "QUARANTINED");
  assert.equal(binding.projectBindingAllowsDurableWrites(decision), false);
  assert.match(decision.rejection_reasons.join(" "), /unsafe_selected_root/);
});

test("unchanged recovery evidence does not duplicate recovery packets", () => {
  const first = decide({ verificationCanonical: false, verificationStatus: "pending" });
  const same = decide({
    verificationCanonical: false,
    verificationStatus: "pending",
    previousDecision: first,
  });
  const changed = decide({
    verificationCanonical: false,
    verificationStatus: "pending",
    candidates: [{ ...candidate, score: 900 }],
    previousDecision: first,
  });
  assert.equal(first.state, "VERIFY");
  assert.equal(binding.shouldEmitProjectScopeRecoveryPacket(first, same), false);
  assert.equal(binding.shouldEmitProjectScopeRecoveryPacket(first, changed), true);
  assert.equal(changed.supersedes_decision_id, first.decision_id);
});

test("Pi startup persists one typed decision and never opens a verification modal", () => {
  const start = sessionSource.indexOf("async function promptForProjectVerifyIfNeeded");
  const end = sessionSource.indexOf("async function promptForWorkpointIfNeeded", start);
  const verifyBlock = sessionSource.slice(start, end);
  assert.match(verifyBlock, /reconcileProjectBindingDecision/);
  assert.match(verifyBlock, /setCurrentProjectBindingDecision/);
  assert.match(verifyBlock, /shouldEmitProjectScopeRecoveryPacket/);
  assert.doesNotMatch(verifyBlock, /ctx\.ui\.confirm/);
  assert.match(sessionSource, /persistedBindingDecision/);
  assert.match(sessionSource, /projectBindingDecisionV1\.state === "BOUND"/);
  assert.match(stateSource, /projectBindingDecisions:/);
  assert.match(stateSource, /projectBindingTelemetry:/);
  assert.match(stateSource, /projectBindingAllowsDurableWrites\(sessionBindingDecision\)/);
  const genesisStart = sessionSource.indexOf("async function ensureProjectGenesis");
  const genesisEnd = sessionSource.indexOf("export function registerSession", genesisStart);
  const genesisBlock = sessionSource.slice(genesisStart, genesisEnd);
  assert.match(genesisBlock, /operatorConfirmed = false/);
  assert.ok(
    genesisBlock.indexOf("if (!operatorConfirmed)") <
      genesisBlock.indexOf('focusaFetch("/project/genesis/start"')
  );
  assert.match(genesisBlock, /pi_project_genesis_next_action_preserved/);
});

test("RECOVERING and QUARANTINED fence mutation while identity and verify remain available", () => {
  const fetchStart = toolsSource.indexOf("async function focusaFetchDetailed");
  const fetchEnd = toolsSource.indexOf("function formatWorkLoopBudgetRemaining", fetchStart);
  const fetchBlock = toolsSource.slice(fetchStart, fetchEnd);
  assert.match(fetchBlock, /projectBindingAllowsDurableWrites\(bindingDecision\)/);
  assert.match(fetchBlock, /failure_class: "scope_recovery_required"/);
  assert.match(fetchBlock, /path\.startsWith\("\/project\/identity"\)/);
  assert.match(fetchBlock, /path\.startsWith\("\/project\/verify"\)/);
  assert.match(fetchBlock, /path\.startsWith\("\/workpoint\/resume"\)/);
  assert.match(fetchBlock, /operator_selection_required: firstMutationSelection/);
  assert.match(fetchBlock, /duplicate_selection_suppressed/);
  assert.match(fetchBlock, /projectBindingTelemetry\.operator_interruption_count/);

  const verifyStart = toolsSource.indexOf('name: "focusa_project_verify"');
  const verifyEnd = toolsSource.indexOf('name: "focusa_project_bootstrap"', verifyStart);
  const verifyBlock = toolsSource.slice(verifyStart, verifyEnd);
  assert.match(verifyBlock, /reconcileProjectBindingDecision/);
  assert.match(verifyBlock, /setCurrentProjectBindingDecision/);
  assert.match(verifyBlock, /binding_decision_v1: bindingDecisionV1/);
});
