# Spec 113 — Focusa Agent Performance Benchmark

**Spec number:** 113
**Status:** Specification
**Purpose:** Standardized benchmark for measuring Focusa-vs-No-Focusa agent performance on metrics that matter, reproducible over time.
**Industry sources:** SWE-bench, AgentBench, METR, τ-bench, τ²-bench, WebArena, TheAgentCompany, xLAM, Gorilla, Anthropic
**Last updated:** 2026-06-26

---

## 1. The Question We Need to Answer

**"Did Specs 110, 111, 112 (and any future changes) actually improve agent performance on metrics that matter, and can we measure that over time?"**

The benchmark must answer:
1. **Before/after comparison** — measure baseline, change, measure again
2. **Metrics that matter** — not vanity metrics, but real outcomes
3. **Reusable over time** — same benchmark runs against v0.9.25, v0.9.26, v0.10.0, etc.
4. **Standardized** — anyone can run it and get the same results
5. **Honest** — no cherry-picking, no agent-only-known tasks, no leaky abstractions

---

## 2. Metrics That Matter (Ranked by Importance)

### Tier 1: Outcome Metrics (MUST-HAVE — Does the agent succeed?)

| Metric | What it measures | How to measure | Why it matters |
|--------|------------------|----------------|----------------|
| **Task completion rate** | % of tasks completed successfully | Binary pass/fail per task | The ultimate question: did the agent finish? |
| **Time to completion** | Wall clock seconds per task | Timestamp delta | Efficiency |
| **Token efficiency** | Tokens used per task | Telemetry from `focusa doctor` | Cost + context bloat |
| **Recovery rate** | % of errors that agent recovered from | Count error events + subsequent success | Resilience |
| **Backtrack count** | How many retries per task | Count task lifecycle events | Confidence in forward path |

### Tier 2: Behavior Metrics (SHOULD-HAVE — How does the agent behave?)

| Metric | What it measures | How to measure | Why it matters |
|--------|------------------|----------------|----------------|
| **Tool selection accuracy** | % of calls using canonical focusa_* tools vs raw shell | `focusa doctor` tool graph analysis | Spec 110 success |
| **Drift incidents** | Count of off-canonical actions per task | Log analysis | Focus Gate integrity |
| **Context overhead** | Size of context per task (bytes/tokens) | Bootstrap packet size | Spec 111 success |
| **License compliance** | % tasks run in proper license mode | License state per task | Commercial viability |
| **Cross-session continuity** | % of tasks that survive compaction | Workpoint resume + work product | Spec 88/96 success |

### Tier 3: Experience Metrics (NICE-TO-HAVE — How does it feel?)

| Metric | What it measures | How to measure | Why it matters |
|--------|------------------|----------------|----------------|
| **AX score** | Agent-rated experience (1-5 scale) | Survey + telemetry | Agent satisfaction |
| **Recovery hint usefulness** | % of recovery hints that led to next action | Click-through | Spec92 quality |
| **Bootstrap freshness** | Age of canonical state at task start | Timestamp compare | Stale data risk |
| **Tool latency** | Time from tool call to response | Telemetry | Performance |

---

## 3. The Task Suite (Standardized Workloads)

The benchmark MUST have a curated task suite that:
- Tests all Focusa features
- Has known-good answers (so we can verify)
- Is reproducible (anyone can run it)
- Is versioned (snapshot of tasks for each Focusa version)
- Is diverse (covers different agent capabilities)

### 3.1 Task Categories

| Category | Count | Difficulty | Tests | Industry analogue |
|----------|-------|------------|-------|-------------------|
| **L1: Setup / activation** | 10 | Trivial | Install, identify project, define trajectory | Product activation / time-to-value |
| **L2: Read / orientation** | 20 | Easy | View trajectory, workpoint, evidence, doctor | AgentBench environment orientation |
| **L3: Write / tool use** | 20 | Medium | Checkpoint workpoint, link evidence, define goal | xLAM/Gorilla function calling |
| **L4: Recover / resilience** | 10 | Hard | Handle scope conflict, license expired, daemon down | τ-bench fault recovery |
| **L5: Multi-step workflow** | 15 | Hard | Define trajectory → checkpoint → work → evidence → resume | τ-bench Pass^N |
| **L6: Cross-session continuity** | 10 | Expert | Compactions, model switches, fork continuations | METR long-horizon retention |
| **L7: Adversarial state** | 10 | Expert | Conflicting scopes, stale packets, malicious inputs, prompt leakage | Benchmark anti-gaming / safety |
| **L8: Real coding tasks** | 20 | Hard | Bugfix/refactor/doc/test tasks with pass/fail tests | SWE-bench / Verified |
| **L9: Dual-control operator tasks** | 10 | Expert | Agent must guide an operator/user who can also mutate state | τ²-bench dual-control |
| **L10: Company workflow tasks** | 10 | Expert | Issue tracker + docs + repo + handoff workflow | TheAgentCompany / enterprise work |
| **L11: Web/computer-use tasks** | 10 | Expert | Browser docs, UI evidence capture, visual diagnostics | WebArena / VisualWebArena |
| **L12: Grounded claims tasks** | 5 | Expert | Evidence-backed answer; penalize unsupported claims | Focusa native + research QA |
| **Total** | **150 tasks** | | | |

### 3.2 Task Snapshot Format

Each task is a JSON file:

```json
{
  "task_id": "L1.001",
  "category": "L1_setup",
  "title": "Install Focusa and verify health",
  "difficulty": "trivial",
  "expected_outcome": {
    "task_completion": "pass",
    "max_time_seconds": 30,
    "max_tokens": 5000,
    "tools_expected": ["focusa_project_identity", "focusa_tool_doctor"],
    "tools_forbidden": ["bash", "curl"]
  },
  "preconditions": {
    "clean_install": true,
    "no_license": true
  },
  "agent_prompt": "Install Focusa on this clean system and verify the daemon is running. Report back the health status.",
  "verification": {
    "method": "curl /v1/health",
    "expected_status": "ok",
    "expected_version": "0.9.25-dev"
  },
  "scoring": {
    "completion": 100,
    "time_under_30s": 50,
    "tokens_under_5k": 30,
    "no_raw_shell": 20
  }
}
```

### 3.3 Task Suite Location

```
crates/focusa-bench/
  tasks/
    L1_setup/
      L1.001_install.json
      L1.002_identify.json
      ...
    L2_read/
      L2.001_trajectory.json
      ...
    L3_write/
    L4_recover/
    L5_multi/
    L6_cross_session/
    L7_adversarial/
    L8_real_coding/
    L9_dual_control/
    L10_company_workflow/
    L11_web_computer_use/
    L12_grounded_claims/
  models/
    model_matrix.json   # pinned model/provider/version/class/pricing plan for each run
  splits/
    public.json          # 70% public regression set
    private_holdout.json # 30% release-evidence set; never exposed to agents
  runners/
    bench.sh           # POSIX runner
    bench.ps1          # Windows runner
    bench.py           # Cross-platform Python runner (recommended)
    ablate.py          # Runs no-Focusa / passive / tool-only / full Focusa arms
  scoring/
    score.py           # Aggregates results
    market_score.py    # Agent Power Index + Focusa Uplift Score
  reports/
    2026-06-25-v0.9.25-dev.json
    2026-06-30-v0.9.26-dev.json
    ...
```

---

## 4. Measurement Methodology

### 4.1 Per-Task Measurement

For each task, measure:
```json
{
  "task_id": "L1.001",
  "run_id": "run-2026-06-25-001",
  "agent_id": "focusa-mvp-eval-agent-v1",
  "focusa_version": "0.9.25-dev",
  "started_at": "2026-06-25T14:00:00Z",
  "completed_at": "2026-06-25T14:00:30Z",
  "duration_seconds": 30,
  "task_completed": true,
  "tokens_used": 4500,
  "tool_calls": [
    {"tool": "focusa_project_identity", "timestamp": "14:00:05", "result": "verified"},
    {"tool": "focusa_tool_doctor", "timestamp": "14:00:15", "result": "healthy"}
  ],
  "errors_encountered": 0,
  "recoveries": 0,
  "backtracks": 0,
  "raw_shell_used": false,
  "score": 200,
  "max_score": 200
}
```

### 4.2 Per-Suite Aggregation

Example schema only; numeric values below are placeholders until produced by completed `/v1/evals/*` runs.

```json
{
  "suite_id": "v0.9.25-dev-2026-06-25",
  "focusa_version": "0.9.25-dev",
  "spec_versions": {
    "spec_110": "draft",
    "spec_111": "proposed",
    "spec_112": "spec_no_impl"
  },
  "tasks_total": 150,
  "tasks_passed": 112,
  "tasks_failed": 38,
  "pass_rate": 0.747,
  "mean_time_seconds": 45,
  "p95_time_seconds": 180,
  "mean_tokens": 8000,
  "recovery_rate": 0.78,
  "tool_selection_accuracy": 0.65,
  "drift_incidents_per_task": 0.4,
  "agent_power_index": 0.71,
  "focusa_uplift_score_vs_no_focusa": 1.42,
  "by_category": {
    "L1_setup": {"tasks": 10, "pass_rate": 1.0, "mean_time": 28, "mean_tokens": 3000},
    "L2_read": {"tasks": 20, "pass_rate": 0.95, "mean_time": 35, "mean_tokens": 5000},
    "L3_write": {"tasks": 20, "pass_rate": 0.85, "mean_time": 50, "mean_tokens": 8000},
    "L4_recover": {"tasks": 10, "pass_rate": 0.55, "mean_time": 90, "mean_tokens": 12000},
    "L5_multi": {"tasks": 15, "pass_rate": 0.65, "mean_time": 120, "mean_tokens": 15000},
    "L6_cross_session": {"tasks": 10, "pass_rate": 0.40, "mean_time": 180, "mean_tokens": 20000},
    "L7_adversarial": {"tasks": 10, "pass_rate": 0.20, "mean_time": 300, "mean_tokens": 30000},
    "L8_real_coding": {"tasks": 20, "pass_rate": 0.50, "mean_time": 600, "mean_tokens": 40000},
    "L9_dual_control": {"tasks": 10, "pass_rate": 0.45, "mean_time": 420, "mean_tokens": 28000},
    "L10_company_workflow": {"tasks": 10, "pass_rate": 0.40, "mean_time": 900, "mean_tokens": 60000},
    "L11_web_computer_use": {"tasks": 10, "pass_rate": 0.35, "mean_time": 480, "mean_tokens": 30000},
    "L12_grounded_claims": {"tasks": 5, "pass_rate": 0.80, "mean_time": 240, "mean_tokens": 12000}
  }
}
```

### 4.3 Cross-Version Comparison

```json
{
  "comparison_id": "v0.9.24-vs-v0.9.25",
  "baseline": {
    "version": "v0.9.24-dev",
    "pass_rate": 0.65,
    "mean_tokens": 9500
  },
  "candidate": {
    "version": "v0.9.25-dev",
    "pass_rate": 0.744,
    "mean_tokens": 8000
  },
  "delta": {
    "pass_rate_change": +0.094,
    "token_reduction": 0.158,
    "significant": true,
    "p_value": 0.03
  }
}
```

---

## 5. Data Collection Architecture (Codebase-Aligned)

**Adversarial correction:** the first draft proposed `POST /v1/telemetry/agent/*` endpoints. That conflicts with `docs/31-telemetry-api.md`, which declares telemetry queryable and never mutable, and with `docs/29-telemetry-spec.md`, which describes CTL as passive observability.

**Approved exception:** evals need first-class durable run capture, but the exception belongs to a separate Eval Ledger, not to general telemetry. Telemetry remains read-only for agents and normal clients. Eval harnesses write append-only eval events through `/v1/evals/*`; CTL observes and aggregates those events.

### 5.1 Authority Boundary

| Surface | Role | Mutation? |
|---------|------|-----------|
| `crates/focusa-bench/` | Runner, task suite, scoring logic, local replay artifacts | YES, benchmark-local files |
| `/v1/evals/runs` | Creates a scoped eval run record | YES, append-only eval ledger |
| `/v1/evals/runs/{run_id}/events` | Appends task/tool/drift/judge events | YES, append-only eval ledger |
| `/v1/evals/runs/{run_id}/complete` | Closes a run with immutable summary | YES, terminal append event only |
| `/v1/evals/runs/{run_id}` | Reads one run and its artifacts | NO |
| `/v1/evals/compare` | Compares baseline/candidate/arm results | NO |
| `/v1/telemetry/events` | Reads CTL-observed events with bounded cursor | NO |
| `/v1/telemetry/tokens` | Reads token aggregates | NO |
| `/v1/telemetry/productivity` | Reads completion/correction/rework/time-to-resolution | NO |
| `/v1/telemetry/autonomy` | Reads autonomy score/timeline/reversions | NO |
| `/v1/telemetry/export` | Starts read/export job per docs | YES, export job only; no event mutation |
| `focusa_evidence_capture` | Captures benchmark report/evidence refs | YES, evidence surface, not CTL |

### 5.2 Eval Ledger API

```http
POST /v1/evals/runs
POST /v1/evals/runs/{run_id}/events
POST /v1/evals/runs/{run_id}/complete
GET  /v1/evals/runs/{run_id}
GET  /v1/evals/compare?baseline=<run_id>&candidate=<run_id>
```

Required write-scope fields:

```json
{
  "suite_id": "focusa-agent-bench-v1",
  "run_id": "run-2026-06-25-001",
  "task_id": "L1.001",
  "scenario_id": "L1_setup",
  "arm": "no_focusa | passive_focusa | tool_only_focusa | full_focusa",
  "agent_id": "claude-sonnet-4.5",
  "model_provider": "anthropic",
  "model_id": "claude-sonnet-4.5",
  "model_version": "2026-06-25",
  "model_class": "frontier_generalist",
  "environment_id": "clean-linux-x86_64-glibc",
  "prompt_hash": "sha256:...",
  "task_seed": 12345,
  "pricing_snapshot": "2026-06-25",
  "eval_mode": true,
  "schema_version": "focusa.eval_event.v1"
}
```

### 5.3 Eval Event Types

```json
{"event":"task_started","task_id":"L1.001","arm":"full_focusa","agent_id":"claude-sonnet-4.5","started_at":"2026-06-25T14:00:00Z"}
{"event":"tool_call","task_id":"L1.001","tool":"focusa_project_identity","canonical":true,"latency_ms":180,"result_status":"ok"}
{"event":"drift","task_id":"L1.001","drift_type":"raw_shell_for_focusa_api","expected_tool":"focusa_project_identity","actual_tool":"bash"}
{"event":"judge_result","task_id":"L1.001","resolved":true,"judge":"deterministic_test","evidence_refs":["pytest::tests/test_health.py"]}
{"event":"task_completed","task_id":"L1.001","resolved":true,"duration_seconds":30,"tokens_used":4500,"cost_usd":0.09}
```

### 5.4 Guardrails for Eval Writes

1. Eval writes are namespaced under `/v1/evals/*`, never `/v1/telemetry/*`.
2. Eval events are append-only, idempotent by `event_id`, and immutable after write.
3. Eval events cannot mutate Focus State, Workpoints, Trajectory, ontology, prompts, gates, or agent behavior.
4. Every write requires `eval_mode=true`, `suite_id`, `run_id`, `task_id`, `scenario_id`, `arm`, `model_provider`, `model_id`, `model_version`, `model_class`, `environment_id`, `prompt_hash`, `pricing_snapshot`, and `schema_version`.
5. Secrets, API keys, PII, and raw private operator text are redacted before persistence.
6. CTL may read/index eval events for reports, but CTL remains passive and non-authoritative.
7. Production telemetry and eval runs have separate retention, export, and privacy policy.
8. Public evidence reports must include raw JSONL, scoring code version, environment digest, prompt hashes, and confidence intervals.

### 5.5 CTL Join Step

After each task, the runner queries Focusa CTL read surfaces and joins results into the eval ledger/report:

```bash
curl -fsS 'http://127.0.0.1:8787/v1/telemetry/events?session_id=<sid>&limit=200'
curl -fsS 'http://127.0.0.1:8787/v1/telemetry/tokens?group_by=session&window=7d'
curl -fsS 'http://127.0.0.1:8787/v1/telemetry/productivity'
curl -fsS 'http://127.0.0.1:8787/v1/telemetry/autonomy'
```

### 5.6 Evidence Capture

Only final benchmark artifacts are linked into Focusa evidence:

```json
{
  "target_ref": "benchmark:spec-113:run-2026-06-25-001",
  "evidence_ref": "crates/focusa-bench/runs/run-2026-06-25-001/report.json",
  "result": "Full Focusa arm beat no-Focusa arm on Agent Power Index with 95% CI"
}
```

This preserves CTL's read-only contract while giving evals first-class durable evidence.

---

## 6. Baseline Measurements (What We Need First)

Before we can measure before/after, we need a **Focusa-vs-No-Focusa baseline** across the full 150-task suite.

### 6.1 Required Baseline Run

**Task:** Run `focusa-agent-bench-v1` against the current release candidate with all four arms:

```text
no_focusa
passive_focusa
tool_only_focusa
full_focusa
```

The baseline report must include:
- `full_focusa` vs `no_focusa` primary comparison
- ablation comparisons for `passive_focusa` and `tool_only_focusa`
- public/private split results
- per-category results (L1-L12)
- confidence intervals and run counts
- evidence refs for every deterministic judge result
- raw Eval Ledger JSONL and scoring code commit

### 6.2 Baseline Is Measurement, Not Prediction

Predicted pass rates may be used for planning and power analysis, but they are not benchmark evidence.

Invalid public wording:
> "Focusa is 2x better" based on expected results.

Valid public wording:
> "On run `<run_id>`, Focusa `full_focusa` improved resolved rate from `<no_focusa>` to `<full_focusa>` with 95% CI `<ci>` and raw artifacts `<evidence_ref>`."

### 6.3 Minimum Baseline Slices

| Slice | Purpose | Minimum Evidence |
|-------|---------|------------------|
| L1 setup | Prove Focusa activation and installer value | clean-machine run on each supported OS family |
| L2/L3 tool use | Prove Focusa tool layer improves orientation/actions | tool-call accuracy + drift events |
| L4 recovery | Prove Focusa recovery hints reduce dead ends | error attribution + recovery success |
| L5/L6 long horizon | Prove Focusa continuity across work and compaction | METR-style time horizon + Workpoint resume evidence |
| L8 coding | Prove Focusa helps real software work | tests, patch size, fail-to-pass/pass-to-pass |
| L9 dual-control | Prove Focusa helps agents guide operators | reasoning-vs-communication attribution |
| L12 grounded claims | Prove Focusa reduces unsupported claims | claim/evidence coverage ratio |

### 6.4 Release Regression Baseline

Every future Focusa release compares against:
1. previous Focusa release (`full_focusa` vs prior `full_focusa`)
2. current No-Focusa baseline (`full_focusa` vs `no_focusa`)
3. current ablations (`full_focusa` vs `passive_focusa` and `tool_only_focusa`)

This preserves the market story while also showing where Focusa improvements come from.

---

## 7. Reporting & Visualization

### 7.1 Focusa-vs-No-Focusa Release Report

```text
================================================================================
FOCUSA AGENT PERFORMANCE BENCHMARK — v0.9.X
Run date: <date>
Suite: focusa-agent-bench-vX
Primary comparison: full_focusa vs no_focusa
Spec versions: 110=<status>, 111=<status>, 112=<status>, 113=<status>
================================================================================

FOCUSA VS NO-FOCUSA HEADLINE
  Resolved %:                 <full_focusa> vs <no_focusa>   Δ=<delta>, 95% CI=<ci>
  Agent Power Index:          <full_focusa> vs <no_focusa>   FUS=<ratio>, 95% CI=<ci>
  Cost per resolved task:     <full_focusa> vs <no_focusa>   Δ=<delta>
  Time horizon @ 50%:         <full_focusa> vs <no_focusa>   Δ=<delta>
  Pass^N:                     <full_focusa> vs <no_focusa>   Δ=<delta>
  Groundedness:               <full_focusa> vs <no_focusa>   Δ=<delta>
  Operator burden:            <full_focusa> vs <no_focusa>   OBR=<ratio>

ABLATIONS (DIAGNOSTIC ONLY)
  passive_focusa vs no_focusa:       <delta summary>
  tool_only_focusa vs no_focusa:     <delta summary>
  full_focusa vs tool_only_focusa:   <bootstrap/reminder/workpoint uplift>

BY CATEGORY (L1-L12)
  L1 setup/activation:        <full_focusa> vs <no_focusa>
  L2 read/orientation:        <full_focusa> vs <no_focusa>
  L3 write/tool-use:          <full_focusa> vs <no_focusa>
  L4 recovery/resilience:     <full_focusa> vs <no_focusa>
  L5 multi-step workflow:     <full_focusa> vs <no_focusa>
  L6 cross-session:           <full_focusa> vs <no_focusa>
  L7 adversarial state:       <full_focusa> vs <no_focusa>
  L8 real coding:             <full_focusa> vs <no_focusa>
  L9 dual-control:            <full_focusa> vs <no_focusa>
  L10 company workflow:       <full_focusa> vs <no_focusa>
  L11 web/computer-use:       <full_focusa> vs <no_focusa>
  L12 grounded claims:        <full_focusa> vs <no_focusa>

FAILURE ATTRIBUTION
  reasoning:                  <count>
  communication/coordination: <count>
  tool selection:             <count>
  tool arguments:             <count>
  environment:                <count>
  Focusa runtime:             <count>
  user/operator:              <count>

EVIDENCE
  Eval run:                   <run_id>
  Raw ledger:                 <events.jsonl>
  Report JSON:                <report.json>
  Scoring commit:             <git_sha>
  Environment digest:         <digest>

VERDICT: <MEASURED_IMPROVED | MIXED | REGRESSED | INCONCLUSIVE>
```

### 7.2 Trend Chart (Over Time)

Trend charts must plot **Focusa Uplift Score** and raw metrics, not just pass rate:

```text
Focusa Uplift Score (full_focusa / no_focusa)
v0.9.25 ───── <measured>
v0.9.26 ───── <measured>
v0.10.0 ───── <measured>
```

---

## 8. Acceptance Criteria for the Benchmark Itself

The benchmark is "real" when:

1. **Reproducible:** Two runs on same version produce same pass rate ±2%
2. **Sensitive:** A 10% improvement in any metric is detectable
3. **Fast:** Public smoke slice runs in <30 minutes; full 150-task suite runs in <4 hours or publishes cost/time budget
4. **Documented:** Each task has clear expected outcomes
5. **Versioned:** Task suite has versions matching Focusa versions
6. **Honest:** No cherry-picking, no agent-specific tuning
7. **Public:** Anyone can clone, install Focusa, run benchmark, compare results
8. **Automated:** CI runs benchmark on every release

---

## 9. Implementation Roadmap

### Phase 1 (Before MVP Cohort)
- [ ] Create `crates/focusa-bench/` with the 150-task suite and public/private split
- [ ] Implement Python runner (`bench.py`) plus `ablate.py` for matched benchmark arms
- [ ] Implement append-only Eval Ledger endpoints (`/v1/evals/*`)
- [ ] Join existing read-only CTL surfaces (`/v1/telemetry/events|tokens|productivity|autonomy`) into reports
- [ ] Run baseline against v0.9.25-dev using `no_focusa`, `passive_focusa`, `tool_only_focusa`, and `full_focusa` arms
- [ ] Publish report only with measured values and confidence intervals

### Phase 2 (During MVP Cohort)
- [ ] Add regression alerts
- [ ] Run benchmark nightly in CI
- [ ] Track per-agent performance
- [ ] Compare focusa versions

### Phase 3 (Post-MVP)
- [ ] Add adversarial task generation
- [ ] Public leaderboard
- [ ] Community-contributed tasks
- [ ] Cross-agent comparison (Claude vs Pi vs other harnesses)

---

## 10. Open Questions

1. **Where to run the agent?** Pi required? Other harnesses?
2. **Who defines tasks?** Internal team? Community? Spec authors?
3. **How to handle tasks that depend on each other?** Sequential suites?
4. **What if Focusa breaks a task?** Skip or fail?
5. **How to measure "agent rated AX" consistently?** Standardize rating?
6. **How to prevent benchmark gaming?** Tasks that look easy but aren't?
7. **Should benchmark include "doing the wrong thing fast" tests?** Adversarial?
8. **What about non-English languages?** Multilingual tasks?

---

## 11. Bead References

- `focusa-cme3` — Tauri release artifacts (related: benchmark must work in released version)
- `focusa-iqqi` — Install binary architecture (benchmark depends on real installer)
- `focusa-fm0f` — EPIC: Spec 112 (foundation for benchmark)
- `focusa-o0o6` — EPIC: Spec 111 (bootstrap for benchmark)

---

## 12. Example Benchmark Run (Illustrative — Not Evidence)

```bash
$ cd /home/wirebot/focusa
$ bench/run.sh --suite focusa-agent-bench-v1 --version v0.9.25-dev --agent claude-sonnet-4.5 --arms no_focusa,passive_focusa,tool_only_focusa,full_focusa

[INFO] Loading task suite: 150 tasks across 12 categories (public=105, private_holdout=45)
[INFO] Creating eval run: POST /v1/evals/runs
[INFO] Starting daemon health check... ok
[INFO] Running L1 setup (10 tasks)...
[INFO] Running L2 read (20 tasks)...
[INFO] Running L3 write (20 tasks)...
[INFO] Running L4 recover (10 tasks)...
[INFO] Running L5 multi (15 tasks)...
[INFO] Running L6 cross-session (10 tasks)...
[INFO] Running L7 adversarial (10 tasks)...
[INFO] Running L8 real-coding (20 tasks)...
[INFO] Running L9 dual-control (10 tasks)...
[INFO] Running L10 company-workflow (10 tasks)...
[INFO] Running L11 web-computer-use (10 tasks)...
[INFO] Running L12 grounded-claims (5 tasks)...
[INFO] Completing eval run: POST /v1/evals/runs/{run_id}/complete

[REPORT] pass_rate, time_horizon, cost, Pass^N, recovery_rate, and Agent Power Index emitted with 95% CI
[REPORT] Saved: crates/focusa-bench/runs/run-2026-06-26-001/report.json
```

All values in public reports must come from completed Eval Ledger runs, not from predicted examples.

---

## 13. Pre-Registered Hypotheses for Specs 110+111+112

These are hypotheses, not evidence. They become claims only after a completed `/v1/evals/*` run with raw artifacts, confidence intervals, and public scoring code.

| Hypothesis | Expected Direction | Evidence Required |
|------------|--------------------|-------------------|
| Spec 112 improves L1 setup/activation | full_focusa > no_focusa on install/health/license tasks | Clean-machine eval run across Linux/macOS/Windows |
| Spec 111 improves orientation and context efficiency | fewer tokens, lower time-to-first-correct-action | Bootstrap packet ablation: passive_focusa vs full_focusa |
| Spec 110 improves tool selection | fewer raw-shell-for-Focusa-API drifts | tool_only_focusa vs full_focusa arm comparison |
| Combined specs improve long-horizon completion | higher METR-style 50% time horizon | L5/L6/L10 completed runs with matched models |
| Focusa improves market-relevant agent power | higher Agent Power Index at lower cost | 150-task suite, no_focusa baseline, 95% CI |

Rule: publish the measured deltas, not the predicted deltas.

---

## 14. The Big Insight

**Without this benchmark, we cannot honestly claim "Spec X improved agent performance."**

**With this benchmark, every spec change has measurable before/after.**

This is the difference between "trust me bro" engineering and "show me the data" engineering. Focusa needs this. The MVP Cohort needs this. Every future spec needs this.

The benchmark itself is a P0 deliverable. Without it, the MVP Cohort evaluation is just vibes.

---

## 15. Missing Metrics — Industry Gap Analysis

Researched via UIAI browser (2026-06-26): SWE-bench, AgentBench, METR, τ-bench, τ²-bench, WebArena, TheAgentCompany, Anthropic's "Building Effective Agents", xLAM, Gorilla.

### 15.1 What Industry Standards Add (That Our Spec §2 Missed)

| Source | Metric | Why It Matters | Add to Benchmark? |
|--------|--------|----------------|-------------------|
| **SWE-bench** | `Resolved %` (binary pass per real GitHub issue) | Real-world software engineering, not synthetic | YES — add L8: real-world coding tasks |
| **SWE-bench** | `Patch size` (lines changed) | Measures surgical vs sprawling fixes | YES — add to all write tasks |
| **SWE-bench** | `Resolve time` (mean per task) | Already have this — refine metric | already in §2 |
| **SWE-bench** | `Fail-to-Pass / Pass-to-Pass tests` | Quality measure beyond binary success | YES — add verification depth metric |
| **AgentBench** | `8 distinct environments` (OS, DB, web, game, etc.) | Diverse environments = robust benchmark | YES — add cross-domain stress |
| **AgentBench** | `Success Rate by Environment` | Per-domain scores identify weak spots | YES — already have by_category |
| **AgentBench** | `Instruction Following Failure Rate` | Separate metric for instruction following | YES — add Tier 2 metric |
| **METR** | **Task duration (time horizon)** | **Most cited industry metric** — 50% threshold | **YES — ADD AS TIER 1 METRIC** |
| **METR** | `Doubling time` (months for 50% improvement) | Industry framing | YES — report trend |
| **METR** | `Cost per task` | Time × compute × tokens | YES — Tier 1 |
| **τ-bench** | **Pass^N** (success rate per turn count) | Multi-turn agent capability | YES — add multi-turn tracking |
| **τ-bench** | **Simulated user** (LLM acts as user) | Tests conversational competence | YES — add conversational tasks |
| **τ-bench** | `Fault Assignment` (user/agent/env error) | Root cause analysis | YES — error attribution |
| **τ-bench** | `Fault Type` (wrong_tool, wrong_arg, etc.) | Granular failure mode | YES — failure classification |
| **τ²-bench** | **Dual-control** (user + agent both have tools) | Real-world customer support | YES — add L9: dual-control |
| **τ²-bench** | **Reasoning vs Communication split** | Separates failure modes | YES — add behavior attribution |
| **τ²-bench** | `Compositional task generator` | Programmatic task creation | YES — automate task generation |
| **WebArena** | `Visual + textual tasks` | Multimodal capability | NICE — add vision tasks |
| **WebArena-Infinity** | `Continuous environments` | Avoid benchmark saturation | YES — version-controlled tasks |
| **TheAgentCompany** | **Real software company workflow** (GitLab, OwnCloud, Plane, Rocket.Chat, GitHub) | Consequential work, not toy tasks | YES — add L10: company tasks |
| **TheAgentCompany** | `Self-collaboration` (multi-agent) | Tests agent-to-agent cooperation | YES — add multi-agent tasks |
| **TheAgentCompany** | `Task dependencies` (some tasks block others) | Real workflow dependency | YES — add dependency graph |
| **Anthropic** | **Augmented LLM building block** | Frames what agents augment | YES — call out what Focusa augments |
| **Anthropic** | **Workflows vs Agents distinction** | Workflow = deterministic, Agent = dynamic | YES — split test suite into both |
| **Anthropic** | `Evaluator-optimizer loops` | Self-critique pattern | YES — add self-critique tasks |
| **Anthropic** | **"Start simple, add complexity when needed"** | Industry philosophy — validates Focusa's modular design | yes — note in philosophy |
| **xLAM / Gorilla** | `Function-calling accuracy` (ast match) | Tests tool use | YES — add tool accuracy tests |
| **xLAM / Gorilla** | `Multi-step function calling` | Compound tool calls | YES — add L3 chained calls |
| **xLAM / Gorilla** | `Hallucination rate` (calls nonexistent tool) | Catches drift | YES — Tier 2 |

### 15.2 Metrics Now Defined (After Industry Research)

#### Tier 0: Foundational (Industry-Standard, MUST HAVE)

| Metric | Source | What It Measures |
|--------|--------|------------------|
| **Task completion rate (Resolved %)** | SWE-bench | Binary pass per task |
| **Task duration (Time Horizon @ 50%)** | METR | Max task length agent can complete at 50% success |
| **Cost per task** | METR | $ tokens + compute + wall-clock |
| **Pass^N (multi-turn success)** | τ-bench | Success rate by turn count |
| **Pass@k (k independent attempts)** | SWE-bench | Sampling-based success rate |

#### Tier 1: Outcome (already in §2)

| Metric | Source | Status |
|--------|--------|--------|
| Task completion rate | SWE-bench | ✓ already have |
| Time to completion | METR | ✓ already have |
| Token efficiency | METR | ✓ already have |
| Recovery rate | τ-bench | ✓ already have |
| Backtrack count | τ-bench | ✓ already have |

#### Tier 2: Behavior (now expanded)

| Metric | Source | New? |
|--------|--------|------|
| Tool selection accuracy | xLAM/Gorilla | ✓ already |
| Drift incidents | Gorilla | ✓ already |
| Context overhead | Focusa native | ✓ already |
| License compliance | Focusa native | ✓ already |
| Cross-session continuity | Focusa native | ✓ already |
| **Instruction following rate** | AgentBench | **NEW** |
| **Hallucination rate (calling nonexistent tools)** | xLAM/Gorilla | **NEW** |
| **Function-call AST match %** | xLAM/Gorilla | **NEW** |
| **Per-tool success rate** | xLAM | **NEW** |
| **Self-critique quality** | Anthropic eval-opt | **NEW** |
| **Dual-control success** | τ²-bench | **NEW** |
| **Reasoning vs Communication split** | τ²-bench | **NEW** |

#### Tier 3: Experience (now expanded)

| Metric | Source | New? |
|--------|--------|------|
| AX score | Focusa native | ✓ already |
| Recovery hint usefulness | Focusa native | ✓ already |
| Bootstrap freshness | Focusa native | ✓ already |
| Tool latency | Focusa native | ✓ already |
| **Patch quality (lines changed per fix)** | SWE-bench | **NEW** |
| **Verify depth (F2P/P2P test count)** | SWE-bench | **NEW** |
| **Continuity across compactions** | Focusa native | **NEW** |
| **Cross-domain transfer (OS/DB/Web/Game)** | AgentBench | **NEW** |

### 15.3 What This Means

The industry-standard additions transform the benchmark from an internal Focusa tool check into a 150-task market evidence suite across 12 categories.

The central question becomes: **How much does Focusa improve the same agent versus No-Focusa, under industry-recognized metrics?**

Now we can credibly report measured Focusa uplift on SWE-bench-like real coding tasks, METR-style time horizons, τ-bench-style multi-turn success, τ²-style dual-control coordination, and xLAM/Gorilla-style tool accuracy — but only when those values come from completed Eval Ledger runs.

---

## 16. The Comparison That Matters Most: Focusa vs No-Focusa

**Primary market comparison: `full_focusa` vs `no_focusa`.** This is not optional and must appear in every public report.

The additional arms (`passive_focusa`, `tool_only_focusa`) are diagnostic ablations only. They explain *why* Focusa helps; they do not replace the core Focusa-vs-No-Focusa claim.

### 16.1 Benchmark Arms

| Arm | What the agent has | Why it exists |
|-----|---------------------|---------------|
| `no_focusa` | Raw harness only: shell, files, browser, model memory | Measures market baseline without Focusa |
| `passive_focusa` | Docs/prompts only; no Focusa tools | Separates documentation benefit from runtime benefit |
| `tool_only_focusa` | `focusa_*` tools available; no bootstrap/reminder automation | Measures tool-layer value |
| `full_focusa` | Installer + daemon + bootstrap + reminders + Workpoints + eval ledger | Measures complete product value |

### 16.2 What "No Focusa" Means

An agent running `no_focusa` has:
- No `focusa_*` tools
- No structured Workpoint checkpointing
- No bootstrap packet
- No trajectory intelligence
- No recovery hints
- No canonical state enforcement
- **Still has:** raw shell, raw HTTP, raw JSON, model memory, retries, and any non-Focusa harness tools

### 16.3 Matched-Arm Methodology

For each of the 150 tasks, run matched trials:

```text
Task: L1.001 install
  ├── no_focusa          same model, same prompt, raw harness
  ├── passive_focusa     same model, same prompt, Focusa docs/prompt only
  ├── tool_only_focusa   same model, same prompt, focusa_* tools only
  └── full_focusa        same model, same prompt, complete Focusa runtime
```

Required controls:
1. Same model, model version, temperature, max tokens, and provider routing.
2. Same machine class, OS, network policy, repo snapshot, and seed.
3. Same task prompt except capability disclosure for available tools.
4. Same timeout and cost budget.
5. Same deterministic judge or blind external judge.
6. At least 5 runs per task for variance-sensitive claims, or documented statistical power analysis.
7. Private holdout tasks excluded from agent-visible docs/prompts.

### 16.4 Metrics to Compare

| Metric | Required Report |
|--------|-----------------|
| Pass rate / Resolved % | arm mean, 95% CI, p-value vs `no_focusa` |
| METR-style time horizon @ 50% | estimated task-duration threshold and CI |
| Cost per resolved task | tokens + wall-clock + infrastructure cost |
| Pass^N | success by turn count / interaction count |
| Pass@k | independent attempt success rate |
| Recovery rate | error → corrected path rate |
| Tool-call accuracy | expected vs actual tool call / argument match |
| Hallucination rate | nonexistent tool/API/file/reference calls |
| Groundedness | claims with evidence refs / total substantive claims |
| Operator burden | number of clarifications, manual interventions, and steering turns |

### 16.5 Market Uplift Metrics

The public evidence should report three aggregate scores:

```text
Agent Power Index (APIx) = weighted score over resolved%, time_horizon, Pass^N, groundedness, recovery, and cost.
Focusa Uplift Score (FUS) = APIx(full_focusa) / APIx(no_focusa).
Operator Burden Reduction (OBR) = 1 - interventions(full_focusa) / interventions(no_focusa).
```

Weights must be pre-registered in the suite manifest before runs. Publish raw per-metric values so readers can recompute scores with different weights.

### 16.6 Publishable Claim Rule

Predicted values are internal hypotheses only. Public claims must use the template:

> "On `focusa-agent-bench-vX`, using `<model/version>` across `<n>` matched trials, `full_focusa` improved `<metric>` from `<baseline>` to `<candidate>` versus `no_focusa` (`Δ=<delta>`, 95% CI `<ci>`, p=`<p>`), with raw artifacts at `<evidence_ref>`."

No headline such as "2.1x better" is valid until backed by completed Eval Ledger runs.

---

## 17. Cross-Market Standard Mapping

How Focusa's metrics map to industry standards (so we can publish comparably):

| Focusa Metric | Industry Equivalent | Source |
|---------------|---------------------|--------|
| Pass rate | Resolved % (SWE-bench), Success Rate (AgentBench) | SWE-bench, AgentBench |
| Time horizon | Time horizon @ 50% | METR |
| Cost per task | Cost per resolved issue | METR |
| Pass^4 | Multi-turn success | τ-bench |
| Tool accuracy | Function-call AST match | xLAM/Gorilla |
| Hallucination rate | Hallucination rate | xLAM |
| Per-tool success | Per-tool success rate | xLAM |
| Recovery rate | Recovery rate | τ-bench |
| Drift incidents | Fault assignment rate | τ-bench |
| Bootstrap freshness | State staleness | Focusa native |
| Cross-session continuity | Cross-session success | Focusa native |
| Dual-control success | Dual-control success | τ²-bench |
| Instruction following | Instruction following rate | AgentBench |
| Self-critique quality | Evaluator-optimizer success | Anthropic |

**Mapping claim:** Focusa benchmark results can be compared directly to:
- SWE-bench results (binary task completion)
- METR results (time horizon)
- τ-bench results (multi-turn)
- xLAM results (tool use)
- AgentBench results (cross-domain)

**This is the credibility claim:** "Focusa is benchmarked against the same standards as frontier agent evaluation."

---


---

## 18. Multi-Model Scenario Uplift Report

Focusa's market evidence must show not only whether Focusa helps one model, but **which LLM models benefit, in which scenarios, by how much, and at what cost**.

### 18.1 Model Coverage

Every benchmark release SHOULD include at least one model from each class where access is available:

| Model Class | Purpose | Examples (pin exact version at run time) |
|-------------|---------|-------------------------------------------|
| Frontier generalist | Measures best-case agent performance | Claude Sonnet/Opus, GPT-5-class, Gemini-class |
| Budget/fast generalist | Measures cost-sensitive production value | mini/haiku/flash class models |
| Coding-specialized | Measures SWE-bench-like tasks | coding-tuned models |
| Open-weight hosted | Measures portability and buyer independence | Llama/Qwen/DeepSeek-class hosted models |
| Local/constrained | Measures Focusa value under weak-model or edge conditions | local quantized or smaller OSS models |

Required metadata per run:

```json
{
  "model_provider": "anthropic",
  "model_id": "claude-sonnet-4.5",
  "model_version": "2026-06-25",
  "model_class": "frontier_generalist",
  "temperature": 0.2,
  "max_tokens": 16000,
  "context_window": 200000,
  "pricing_snapshot": "2026-06-25",
  "provider_routing_locked": true
}
```

No result may be generalized to "LLMs" unless at least three model classes are represented.

### 18.2 Scenario Matrix

Reports must include a model × scenario matrix for the primary comparison `full_focusa` vs `no_focusa`.

| Scenario | Metric Focus | Why It Matters |
|----------|--------------|----------------|
| L1 setup/activation | time-to-value, install success | product adoption |
| L2 orientation | time-to-first-correct-action, token use | agent onboarding |
| L3 tool use | tool accuracy, wrong-argument rate | Focusa tool-layer value |
| L4 recovery | recovery rate, dead-end rate | resilience |
| L5/L6 long horizon | METR-style time horizon, continuity | compaction/session value |
| L8 coding | resolved %, patch quality, tests | software market proof |
| L9 dual-control | coordination success | operator/customer support value |
| L10 company workflow | dependency handling | enterprise realism |
| L11 web/computer-use | grounded browser actions | UIAI/browser value |
| L12 grounded claims | evidence coverage, unsupported-claim rate | trustworthiness |

### 18.3 Focusa Model Uplift Matrix

For each model and scenario, report:

```text
Focusa Uplift Score = AgentPowerIndex(full_focusa) / AgentPowerIndex(no_focusa)
Resolved Delta      = Resolved%(full_focusa) - Resolved%(no_focusa)
Cost Delta          = CostPerResolved(full_focusa) - CostPerResolved(no_focusa)
Time Delta          = TimeToResolution(full_focusa) - TimeToResolution(no_focusa)
Grounding Delta     = GroundedClaims%(full_focusa) - GroundedClaims%(no_focusa)
```

Example report table format:

| Model | Class | L1 Setup | L4 Recovery | L6 Continuity | L8 Coding | L12 Grounded Claims | Overall FUS |
|-------|-------|----------|-------------|---------------|-----------|---------------------|-------------|
| `<model A>` | frontier | `<FUS>` | `<FUS>` | `<FUS>` | `<FUS>` | `<FUS>` | `<FUS>` |
| `<model B>` | budget | `<FUS>` | `<FUS>` | `<FUS>` | `<FUS>` | `<FUS>` | `<FUS>` |
| `<model C>` | open-weight | `<FUS>` | `<FUS>` | `<FUS>` | `<FUS>` | `<FUS>` | `<FUS>` |

Heatmap colors must be based on measured confidence intervals, not raw point estimates alone.

### 18.4 Before/After Across Focusa Releases

For each model, report both:

1. **Focusa-vs-No-Focusa uplift** within a release.
2. **Focusa release-over-release improvement** for the same model.

```text
model=<model_id>
scenario=L6_cross_session
no_focusa_v0.9.25=<score>
full_focusa_v0.9.25=<score>
full_focusa_v0.9.26=<score>
focusa_uplift_v0.9.25=<ratio>
focusa_release_delta=<full_focusa_v0.9.26 - full_focusa_v0.9.25>
```

This separates "Focusa beats no Focusa" from "Focusa itself improved since the last release."

### 18.5 Weak-Model Amplification

One of Focusa's strongest possible market claims is that it makes weaker/cheaper models more capable.

Report:

```text
Weak Model Close Rate = APIx(cheap_model + full_focusa) / APIx(frontier_model + no_focusa)
```

This answers:
- Can Focusa make cheaper models viable?
- Can Focusa reduce model spend while preserving success?
- Which scenarios benefit most from structure, memory, Workpoints, and recovery hints?

### 18.6 Model Interaction Warnings

Model comparisons are invalid unless:

1. model versions are pinned;
2. provider routing is locked or recorded;
3. prices are snapshotted at run time;
4. context windows are recorded;
5. temperature/sampling settings match;
6. retry policies match;
7. safety refusals are classified separately from task failures;
8. public claims name exact model/version and run date.

### 18.7 Multi-Model Public Claim Template

Valid claim:

> "Across `<n>` pinned LLM models and `<m>` scenarios, Focusa improved Agent Power Index versus No-Focusa in `<k>/<n*m>` model-scenario cells, with median Focusa Uplift Score `<median>` and cost-normalized uplift `<cost_uplift>`. Raw artifacts: `<evidence_ref>`."

Invalid claim:

> "Focusa makes all LLMs better."

## 19. What to Publish as Evidence

Every public artifact should make the product name and claim unmistakable: **Focusa improves agent performance compared with No-Focusa baselines.**

### 19.1 Focusa-vs-No-Focusa Headline Card

Use measured values only:

```text
FOCUSA AGENT PERFORMANCE BENCHMARK — v0.9.X

Suite:                         focusa-agent-bench-vX
Total tasks:                   150 (public + private holdout)
Models / scaffolds:            <model_matrix_summary>, <agent harness>, <versions>
Primary comparison:            full_focusa vs no_focusa
Multi-model report:            model × scenario Focusa uplift matrix

Resolved %:                    <full_focusa> vs <no_focusa>   Δ=<delta>, 95% CI=<ci>
Agent Power Index:             <full_focusa> vs <no_focusa>   FUS=<ratio>, 95% CI=<ci>
METR-style time horizon @50%:  <full_focusa> vs <no_focusa>   Δ=<delta>
Cost per resolved task:        <full_focusa> vs <no_focusa>   Δ=<delta>
Pass^N / multi-turn success:   <full_focusa> vs <no_focusa>   Δ=<delta>
Groundedness:                  <full_focusa> vs <no_focusa>   Δ=<delta>
Operator burden:               <full_focusa> vs <no_focusa>   OBR=<ratio>

LLM MODEL UPLIFT
  Models tested:               <n_models> across <model_classes>
  Median Focusa Uplift Score:  <median_fus>
  Best uplift scenario:        <scenario_id> / <model_class>
  Weak-model close rate:       <cheap_full_focusa>/<frontier_no_focusa>

Ablations: passive_focusa, tool_only_focusa
Evidence: <run_id>, <raw_jsonl>, <scoring_commit>, <environment_digest>
Comparable framing: SWE-bench, METR, τ-bench/τ²-bench, xLAM/Gorilla, AgentBench/WebArena
```

### 19.2 Detailed Focusa Benchmark Report

- Focusa-vs-No-Focusa primary comparison on every metric
- Multi-model scenario uplift matrix (model × scenario × arm)
- Weak-model amplification and cost-normalized model comparisons
- Ablation analysis (`passive_focusa`, `tool_only_focusa`) after the primary comparison
- Per-category breakdown (L1-L12)
- Per-task success rate and confidence interval
- Time/cost/token distributions
- Groundedness and unsupported-claim analysis
- Failure mode attribution: reasoning, communication, tool selection, tool arguments, environment, user/operator, Focusa runtime
- Regression comparison to previous Focusa releases
- Private-holdout vs public-regression split

### 19.3 Methodology Doc

- Task generation and holdout policy
- Agent harness and model configuration
- Focusa and No-Focusa environment setup
- Judge implementation and blind-scoring policy
- Statistical methods and minimum run counts
- Cost accounting formula
- Reproducibility instructions
- Known limitations and invalid claim examples

### 19.4 Raw Evidence Bundle

- 150 task JSON files for public split
- Private holdout manifest hash
- Per-task Eval Ledger JSONL
- Aggregated report JSON
- CTL export refs used for token/cost/time joins
- Scoring code version and commit
- Model matrix: provider, model id, exact version/date, model class, pricing snapshot, context window, sampling settings
- Environment digest: OS, arch, Focusa version, daemon version, installer channel, license mode

### 19.5 Focusa Benchmark Leaderboard

- Public URL: `focusa.dev/bench`
- Default view: **Focusa vs No-Focusa**
- Secondary views: by model, by Focusa version, by category, by cost, by time horizon
- Drill-down: task, event timeline, evidence refs, judge output, failure attribution
- Clear labels for measured values vs hypotheses

---

## 20. Conclusion

Spec 113 now defines a market-grade **Focusa Agent Performance Benchmark**.

The benchmark is not just a Focusa internal eval. It is a public evidence system built around the question buyers and builders care about:

> **How much more powerful, grounded, cost-effective, and operator-friendly is an agent with Focusa than the same agent without Focusa?**

The required headline is always **Focusa vs No-Focusa**. Ablations explain the uplift, but they never replace the primary market comparison.

Valid public claim template:

> **"On `focusa-agent-bench-vX`, Focusa improved `<metric>` from `<no_focusa>` to `<full_focusa>` versus the No-Focusa baseline (`Δ=<delta>`, 95% CI `<ci>`), using `<model/version>` across `<n>` matched trials. Raw artifacts: `<evidence_ref>`."**

This is what makes Focusa grounded: measured claims, raw artifacts, replayable evals, and explicit No-Focusa baselines.
