# 188 — Invocation-bound background dispatch repair

Status: operator-authorized bounded source repair; installed acceptance pending.
Owner: Verious Smith III. Operator instruction: “Proceed to create repair spec, decompose spec to callgraph then fix / implement”.
Parent: [Spec187](187-focusa-autonomous-agentic-flow-state-release-workset-spec.md), FLOW-02 / `focusa-ak5sv.3` and FLOW-11 / `focusa-ak5sv.12`; dependent issue #682. This is a dependency refinement, not a replacement workset, extra scheduler or new authority store.
Graph: [typed decomposition](contracts/spec188/invocation-dispatch-repair.callgraph.v1.json).

## 1. Outcome and evidence

An already verified native agent invocation must pass its own exact attachment to the existing background CLI without depending on a prior interactive startup or a process-global attachment. The CLI and daemon retain authority, scope, lifecycle, frontier, budget and provider checks. Missing or foreign invocation context must produce a bounded failure before spawning, never inherit another session's authority.

Observed: installed dispatch returned HTTP409 with missing project/continuity; no job or verification receipt exists for the staged #682 proof. Pinned main6cf15b2 and authoring worktreea64ce816 both route callbacks through attachment-local runtime context while background-job-tools.ts inherits process environment. Startup promotion alone publishes FOCUSA_ATTACHMENT_KEY_V1. This establishes a source seam; only real consumer proof can establish the installed fault and its resolution.

## 2. Governing boundaries

- Specs40/104 and applicable158/164 identity contracts: runtime session/attachment is not cwd or a selected-project preference. Reuse the existing verified scope registry and current invocation accessor. No new binding, discovery store, private helper invocation, manufactured attachment or global last-active fallback.
- Specs89/90/91/109/141 and Production Consistency Policy: canonical contract owners and supported schemas; read failures remain unknown, genuine permission/policy differences remain visible. This repair does not strip operation_policy or change #682 acceptance.
- Spec187 R01/R02/R03/R04/R06/R07/R13/R14/R17/R19: preserve exact accepted purpose, requests, grants and budgets; no avoidable selection/reload prompts; reconcile uncertain effects before retry. Existing publication for other compatibility consumers is not silently removed.
- KH owns authority/control only; bounded tests execute on OVH through the supported governed executor. No provider-auth changes, new credentials, marker/goal rewrites, service changes, paid CI or installed-package edits.

## 3. Call stack and one-owner design

Native routing resolves the current attachment → existing tools.ts registrations capture it via currentAttachmentKey() → scoped-state.ts validates the project key against its existing verified registry and creates a new child environment → background-job-tools.ts passes that environment explicitly to execFile → existing CLI parser serializes the same identity into create → daemon validates/adjudicates → existing job monitor and terminal receipt.

scoped-state.ts owns the pure child-environment projection because it already owns ScopeRef, WorkstreamKey, AttachmentKey, registered scope verification and sameRootScope. The dispatcher remains a thin process/receipt adapter. Both single and multi-dispatch registrations use the same projection. There is no public environment/attachment parameter that a model can supply, no process.env mutation and no binding inferred from command cwd.

## 4. Repair contracts

| ID | Acceptance |
|---|---|
| INV-01 | Valid registered project key, nonbootstrap continuity and nonempty instance/session/attachment identifiers produce an explicit child environment containing the exact input attachment. No actor or continuity is regenerated. |
| INV-02 | Missing, malformed, host/bootstrap, unknown project or mismatched scope-id/fingerprint/path is rejected before spawning; a valid-looking ambient process key cannot rescue it. |
| INV-03 | Parent environment is not mutated. Ordinary inherited variables remain available to the existing CLI; only the attachment routing value is replaced in the private child snapshot. Neither environment nor attachment appears in public results/errors. |
| INV-04 | Registered focusa_bg_run and focusa_bg_run_many use the actual current accessor, not model parameters, cwd, cached published key or a fabricated key; multi-dispatch catches per-job preparation errors through existing result handling. |
| INV-05 | Concurrent project/workstream/session A/B calls receive independent snapshots even if the process-global attachment is missing, malformed, stale or from B. Snapshot is captured before child execution and cannot be changed by later input mutation. |
| INV-06 | CLI dispatch argv, platform quoting, timeout/output limits, current/legacy receipt compatibility, failure details, SSE completion and daemon admission remain unchanged. A dispatch acknowledgement is not terminal success. |
| INV-07 | Source producer and real registered-tool consumer tests cover valid, absent/foreign, concurrency, both registrations and no leakage. Existing background, scoped-routing, contract-projection and relevant lifecycle regressions remain applicable. |
| INV-08 | Source success does not close #682 or Spec187. Exact candidate/installed consumer proof must still show admitted bounded OVH execution, terminal receipt, policy-drift/unknown-read compatibility and required lifecycle transitions. Existing unresolved dependencies and all original acceptance remain open. |

## 5. Exact source scope

Production: apps/pi-extension/src/scoped-state.ts, src/tools.ts, src/background-job-tools.ts.
Tests: existing tests/background-job-tools.test.mjs; additive tests/188-invocation-dispatch.test.mjs using the existing registered-tool harness, canonical pure loader and real production dispatcher. The existing test:bg-tools package command includes both. The existing spec104-attachment-runtime-isolation.test.mjs recovery case owns a private temporary project/task fixture instead of borrowing the checkout's markers or real task queue; its original recovery and request assertions remain unchanged.
Documentation: this document, its graph and a bounded reference in the existing extension workflow guide. No generated registry/schema edits unless generator verification demonstrates a public contract change; no new public tool parameters are intended.

Source authoring worktree: /tmp/focusa-cohesion-release-20261014 (shared Git common directory /home/wirebot/focusa/.git); preserve the primary checkout's unrelated marker/Beads changes. Tests run from an isolated OVH source slice with existing installed dependencies. No dependency installation or branch creation.

## 6. Dependency graph, evidence and failure paths

SPEC → RED → IMPLEMENT → SOURCE-PROOF → CONSUMER-ACCEPTANCE.

SPEC verifies existing source/owners and validates the typed graph. RED exercises the actual registered callbacks and dispatcher; expected failure must show missing invocation projection, not missing dependencies. IMPLEMENT applies only this spec's source changes. SOURCE-PROOF runs the targeted regressions and applicable static/type checks with a terminal receipt. CONSUMER-ACCEPTANCE is separately gated by existing delivery authority and supported installation, and cannot be marked done by source proof.

Each frame returns evidence handles and a bounded disposition. Missing executor admission stops the affected test/dispatch, not read-only investigation or authorized source preparation. Unexpected RED success requires diagnosis before implementation. Failed tests return to the owning source frame without invented retries or budget renewal. At most one explicit run per command per proof attempt; any retry requires effect reconciliation and a new recorded cause/remedy.

Failure/rollback: do not alter or restart the live package/daemon. Preserve preimage hashes and the scoped patch. Revert only these exact source hunks if independently required/authorized, using compare-and-swap against the current bytes; never reset shared work. If the source baseline changes, stop patch application and reconcile it first. Fail-closed cases produce no child process to cancel. Already-admitted jobs continue using their existing terminal-receipt path.

## 7. Proof plan and reporting

1. Versioned graph validation through the canonical deterministic validator (validation is not execution admission).
2. Producer/consumer Node regressions on OVH, existing TypeScript and scoped/tool-contract tests as applicable. Existing lifecycle proof is retained but not relabeled as proof of this patch.
3. Current/legacy child receipt contracts and installed CLI parser remain compatible; preserve provider/daemon rejections unchanged.
4. Separately verify actual native agent/session continuation and exact installed consumer before claiming delivery or closing an issue.

Record command, source digest, start/end, exit status and bounded output; do not include credentials, environment payloads or fabricated timing. Required public status distinctions: spec/graph, implemented source, source proof, release/installation and consumer acceptance.
