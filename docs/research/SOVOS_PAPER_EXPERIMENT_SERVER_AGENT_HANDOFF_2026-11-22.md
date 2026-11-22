# Server-Agent Experiment Handoff — SOVOS / Focusa / UIAI / Wirebot Research

**Handoff status:** READY FOR IMPLEMENTATION AND EMPIRICAL EXECUTION, NOT YET EXECUTED  
**Date:** 2026-11-22  
**Owning repository:** Startempire-Wire/focusa  
**Source inspected:** Focusa main at `2b7ab94c609bbb627e711bac233a67612bdf61b8`  
**Research consumer:** Startempire-Wire/agent-driven-life-business-os PR #49, v0.3 scientific manuscript  
**Book consumer:** Startempire-Wire/the-entrepreneurs-path PR #104 (review only)  
**Existing owning specs:** docs/113-agent-benchmark-spec.md, docs/114-public-benchmark-flywheel-spec.md, docs/current/FOCUSA_PUBLIC_BENCHMARK_OBSERVATORY.md, docs/current/FOCUSA_AGENT_INTELLIGENCE_EVALS.md, and Focusa issue #353.

## 0. Actual request to the server build agent

**Deliver measured, reproducible, third-party-reviewable evidence** for the scientific paper's central question:

> When the agent, underlying model and task are held comparable, what difference do Focusa's governed cognition/continuity and UIAI's versioned, verified computer interaction make to externally accepted outcomes, failures, recovery, attention cost and resource use?

Implement the **existing Spec 114 Phase 2 / Spec 113 executable benchmark obligations** and run the first legitimate experiment on an authorized isolated server/harness. Return a **separate results PR** with real task receipts, independent oracle outcomes, statistics and figures; post the results-PR URL as a comment on this handoff PR. Do not return a new planning document as the experimental result. Reuse the existing `crates/focusa-bench`, Eval Ledger contract, established CLI/tool adapters, and existing public snapshot redaction gate. Do not invent a parallel benchmark product, scheduler, memory system or mission authority.

This is an implementation/evaluation handoff, not an instruction to publish, reboot live services, buy inference capacity, change licensing, modify customer data or alter a canonical architectural decision. Use existing approved execution authority and read-only installed/runtime discovery. Apply Context Authority preflight for consequential changes. If the target host lacks authorization, provider budget or working capabilities, record exact blocked evidence and continue the independent fixture/runner implementation.

## 1. Verified foundation and defects: do not confuse source with measurements

| Source | Inspected state | Required consequence |
| --- | --- | --- |
| `docs/113-agent-benchmark-spec.md` | Defines the comparative study and scoring | Use as canonical measurement contract |
| `docs/114-public-benchmark-flywheel-spec.md` §29 | Defines **18 live tasks across L1, L2, L3, L4, L6, L12**, plus seven L0 static checks | Start here, without removing the full-scope future task suite |
| `crates/focusa-bench/src/{arms,ledger,metrics,reports,task_suite}.rs` | Code-backed data types; no complete live harness proved | Implement experiment execution and grade actual effects |
| `crates/focusa-bench/src/task_suite.rs` | Generates 150 placeholder prompts with no executable fixture bodies (78 public, 72 private) | Do not present 150 placeholder entries as 150 executed tasks; preserve existing future expansion |
| `crates/focusa-bench/src/lib.rs` | Constants advertise 75/75, differing from actual task pool 78/72 | Reconcile count labels with the existing source owner; no count-only success claims |
| `scripts/run-agent-intelligence-evals.sh` | Definition check, **exit 78 / UNMEASURED** for valid inputs, not a model execution benchmark | Do not treat exit 78 as passing; preserve issue #353's safety gate until real runner is proven |
| `tests/353_agent_benchmark_truthfulness_test.py` | Refuses self-scored fixtures and unsupported arms | Keep and extend negative tests |
| `public/bench/latest.json` | `publish_allowed:false`, no public measured run | Do not modify to publish until real receipts, redaction and independent grading |
| `crates/focusa-bench/src/model_matrix.rs` | Example historical model/version/cost entries | Pin actual installed model IDs, versions, sampling, pricing and provenance at run time |
| `crates/focusa-bench/src/ledger.rs` | In-memory append-only chain using FNV-1a | Do not describe this alone as cryptographic immutable provenance; use owning durable ledger and SHA-256 evidence digests |
| `crates/focusa-api/src/routes/evals.rs` | Not present in inspected main tree | Discover actual installed API; implement required Spec 114 routes in the owning API rather than claiming this file exists |
| Focusa issues #353 and #364 | Identify historically misleading benchmark PASS paths; present source now fails closed for absent intelligence measurements | Retain fail-closed behavior and add real execution-backed proof |

**Registered tool capability inventory:** `docs/current/focusa-tool-contracts.json` currently declares **157 tools across 20 families**. These are declarations, not evidence that the current installed Pi, OpenClaw or daemon exposes every tool or that it is authorized. Query the actual installed consumer and generated registry; do not hardcode the observed count as a ceiling.

## 2. Minimal capable tool route (discover installed contracts before use)

| Test intention | Existing declared Focusa operations / owning surfaces |
| --- | --- |
| Installed readiness & actual authority | `focusa_tool_doctor`, installed `focusa --version`, `focusa --help`, `focusa doctor --help`, current authorization and scope status |
| Exact project binding | `focusa_project_identity`, `focusa_project_verify`, `focusa_trajectory_view` |
| Continuity, checkpoint, proof | `focusa_workpoint_checkpoint`, `focusa_workpoint_resume`, `focusa_evidence_capture`, `focusa_workpoint_link_evidence` |
| Session rollover, lineage and recovery | `focusa_session_transfer`, `focusa_silent_sessions`, `focusa_tree_head`, `focusa_lineage_tree` |
| Long-running isolated jobs | `focusa_bg_run`, `focusa_bg_status`, `focusa_work_loop_status`, `focusa_work_loop_writer_status` |
| Cognitive comparison | `focusa_context_cognition`, `focusa_context_cognition_curate_eval`, `focusa_predict_record`, `focusa_predict_evaluate`, `focusa_metacog_evaluate_outcome` |
| Independent browser observation/action | Installed UIAI Engine session/snapshot/action/diagnostics/evidence tools; source-owned UIAI versioned observation and artifact interfaces |
| Consequential mutation | Actual Focusa Context Authority `focusa action preflight` / installed typed action-preflight interface; independent source/credential/entitlement checks |

Use the Focusa Agent Adapter Contract and installed capability discovery. **No tool invocation or `focusa_*` name supplies authority merely because it appears in this table.** Direct API/CLI is preferred when a source owner supports it; UIAI remains first-party browser/computer execution for relevant UI tasks. UIAI's unscoped screenshot delivery remains under issue #299; report its actual effect, no fabricated success.

## 3. Executable experiment matrix — follow the existing 18-task Spec 114 starter

Use `docs/research/SOVOS_PAPER_BENCHMARK_MVP_CASES_2026-11-22.json` as a *task-design inventory*, not task outcomes. Each of the 18 named scenarios needs its own resettable synthetic fixture, exact agent input, expected operation and authority posture, deterministic/independent external oracle, adverse-path cases and documented execution budget. The case IDs map to the six Spec 114 categories.

**Primary arms (mandatory):** `no_focusa` and `full_focusa`, using the same permitted tools and source domain access except for the Focusa-specific cognitive intervention. Run matched tasks and repeat trials with pinned model/harness/source/environment configuration. Do not starve the baseline of necessary task inputs or protections, and do not leak full-Focusa state into the baseline. No-Focusa must be a genuine independent arm, not a relabel.

**Diagnostic arms (required by Spec 114 after primary proof, still tracked):** `passive_focusa` and `tool_only_focusa`. Distinguish prompt-only vs tools-only vs full Workpoint recovery; do not silently treat a documentation-only intervention as running Focusa.

**First inexpensive execution:** fixture-only zero-provider tests, then one authorized matched smoke task per category. After that run the full 18-task paired experiment with independently verified outcomes, as provider budgets allow. Keep remaining accepted Spec 113/114 obligations visible; the 18-task research slice is a sequencing decision, not scope deletion.

**Adversarial supplements** (separately labeled, not counted among the 18 until they have executable grade fixtures):
- A no-project **goal-directed ad hoc** session performs allowed UIAI research/screenshot without creating a Workstream; when explicitly promoted to a project, prior unsealed images are not silently recast as verified Evidence (Focusa #672, UIAI #299).
- Two formal Workstream IDs share the same project root and continuity lookup candidate; the agent must not merge or select the wrong one based on cwd/session recency.
- The webpage changes after the agent observed it; stale document/fingerprint must trigger re-observation, not an outdated click.
- The external effect succeeds but the response is lost; source reconciliation prevents duplicate side effect and an unsupported success claim.

## 4. Strict evidence capture and independent grading

Every task attempt records:
- task ID/revision, trial/seed, arm, configured feature flags and **proof of arm isolation**;
- benchmark/scorer source commit, fixture hash and independent oracle version;
- runtime/harness/model/provider/temperature/endpoint identity, tokens, elapsed time, resource and approved spend;
- authenticated synthetic principal + tenant/workstream/ad hoc mode, explicit scope/effect/grant posture, and safe proof refs (no secrets);
- tool action trace/effect ref, observed source-domain postcondition, independent oracle verdict with exact oracle artifact hash;
- `passed | failed | inconclusive | unavailable | unauthorized`, with reasons, never undocumented omission;
- Focusa Evidence/Receipt linkage only if actually accepted by the appropriate canonical owner; browser preview is not canonical proof;
- human intervention seconds, retries, recoveries, stale action attempts, wrong-scope attempts, predicted vs actual outcome.

The **independent oracle** must inspect source state or execute an external evaluator. An agent saying “done,” tool HTTP 200, a passing build without source-state reconciliation, or a stored fixture `score` does not count as a successful outcome. Freeze the test spec and independent scoring before result collection. Negative and failed cases stay in denominators.

Never put production credentials, private client records, unredacted prompts, audio, screenshots, URLs with secrets, customer contact data or internal operational details in a public repository. Use synthetic fixtures and source-qualified redacted refs.

## 5. Acceptance metrics and research charts

**Primary:** rate of correctly authorized, independently verified accepted outcomes, reported by category and arm with denominators and appropriate 95% confidence intervals.

**Secondary:** scope leak or unauthorized mutation count; stale action admissibility; evidence/provenance completeness; source-verification false-success rate; recovery after compaction/restart; median and tail latency; token usage; measured cost per accepted outcome (when denominator >0); human intervention burden; failure/retry distribution; unknown/inconclusive coverage.

Produce genuine figures from recorded run rows, never mock numbers:
1. Paired success with CI by arm, plus per-category differences.
2. Error/failure-class matrix and security-boundary outcomes.
3. Recovery timeline and distribution after interruption or session rollover.
4. Cost/latency versus accepted-outcome frontier, with CIs and explicit missing-data labels.
5. A trace graph showing source observation → Focusa work/authority → UIAI action → source outcome → proof/settlement for at least one full replay.

Pre-register a statistical plan before collecting outcomes (paired analysis where task pairing is valid, bootstrap interval with disclosed samples and missingness). No p-value/statistical claim without actual trial data and appropriate model assumptions. Do not call the operational interaction effect IIT Phi.

## 6. Explicit server execution and lifecycle

1. **Discovery-only:** identify current real target host, account and isolated checkout, writer/Workstream occupancy, installed Focusa / Pi / UIAI versions, model/provider entitlement and existing Beads/Focusa task ownership. Source presence and CLI `--help` are not installation, grant or working consumer proof. Never assume example hostname or root/sudo. Do not restart anything to obtain command discovery.
2. **Review boundary:** confirm real approved provider budget (no implied purchase), time/memory caps, existing access grants, synthetic test domain and one current authorized writer. Run exact-scope preflight before code, release or service mutation. Use the existing Workpoint/Beads graph rather than creating a new cross-product authority.
3. **Implement/fix existing owner:** repair/complete `crates/focusa-bench` executable task suite, real ON/OFF isolation, oracle grader, append-only durable run receipts, scorer, publication gate and tests per Specs 113/114 and issue #353. Prefer smallest qualified dependency-ordered implementation slices; do not replace production daemon or change release infrastructure without separate authority.
4. **Run on isolated test harness:** synthetic fixtures and snapshots first. Provider-backed runs only under the already-approved constrained profile. For UIAI interaction use the already-owned UIAI server, not independently launched ad hoc external browser services. Record exact observed errors and remaining failures.
5. **Independent review:** a separate reviewer/worker rechecks hashes, reproduces at least one success *and* one failure case, confirms the grader was blind to the desired hypothesis and inspects isolation. If unavailable, mark results `review_pending` and refrain from paper claims.
6. **Return evidence PR:** create a new linked GitHub PR with code patch, task manifest, redacted run rows, failure evidence, scorer code/commit, generated plots, independent review status and exact run commands. Do not merge source-only or fabricate a pass to unblock publicity. Link back to this handoff and to SOVOS research PR #49.

## 7. Response required from the server agent

Post a concise comment on the **handoff PR** reporting:
- discovered target scope and installed consumer versions (public-safe identifiers only);
- actual Focusa tool families reachable to the installed agent, and exact unavailable/advisory ones;
- source or implementation mismatch to Spec 113/114 and #353;
- the **results PR URL** and exact commit;
- observed task/arm/attempt counts and proof hashes, or exact reason the run remains `UNMEASURED`;
- independent grader/reviewer status, real failures and next ready closure;
- whether the experiment has any paper-publishable measurements.

The response must be direct source-backed results, not merely a statement that another agent was asked to run the test. A PR comment does not dispatch an agent by itself; the owning runtime must actually assign this handoff to its approved server builder.

## 8. Scientific publication seam and uncompleted scope

SOVOS PR #49 may cite the benchmark only after a real measured-result PR is reviewable with re-executable scoring and redaction checks. Book PR #104 may point to that paper revision after author review; do not insert statistics into the book, marketing or arXiv source before they exist.

This effort does not independently verify machine consciousness. IIT 4.0 is a separately version-pinned finite-model research track under SOVOS Master Loop §12, not a substitute for measured Focusa performance or source outcome acceptance.

**Deliverable acceptance:** a reviewer can clone the pinned source, run the supplied synthetic fixtures and published scoring, independently reproduce reported results (or see exact environmental constraints), inspect every negative outcome, and connect each numerical chart to immutable underlying recorded observations. Anything short of that remains engineering evidence or planned research, not a peer-review-ready efficacy claim.

## 9. Embedded cross-vertical substrate experiment (additional study)

The author has identified a second research claim: **the same Focusa runtime and UIAI Engine primitives can be embedded across Wirebot, WorkRouter, Wirebase and Focusa Market Lab** with each domain retaining independent business/source truth. This is an additional controlled study, not replacement of the original Spec 114 18-task starter.

- Read `docs/research/SOVOS_PAPER_CROSS_VERTICAL_RESEARCH_HANDOFF_2026-11-22.md` and its paired `docs/research/SOVOS_PAPER_CROSS_VERTICAL_PORTABILITY_STUDY_2026-11-22.json` (12 design-only test cases, zero measurements).
- Include one or more **implemented** cross-vertical synthetic test cases and their actual source-domain oracles in the eventual results PR. Preserve `unmeasured` or `incomparable` on any not executed or non-equivalent arm. Test both ad hoc and formal scoped uses where appropriate.
- The core scientific controls compare agent/harness-only, Focusa addition, UIAI verified interaction where applicable, and composed Focusa/UIAI—**without denying the baseline equivalent source access or pretending a browser is needed for deterministic finance operations**.
- Market Lab is paper/synthetic-fixture only, not an order/trading authority. Separate a simulated fast clock from real uninterrupted 24-hour operational monitoring. Autonomous duty cycle means verified goal maintenance and selective event-driven work; it does not mean constant LLM calls.
- Return cross-domain results, failure/effect traces, 95% uncertainty where justified, and genuinely observed scientific figures through the **same source-owned benchmark/result PR**. The reusable underlying primitive contracts are the scientific subject; each application supplies its own source records and evaluation oracle.
