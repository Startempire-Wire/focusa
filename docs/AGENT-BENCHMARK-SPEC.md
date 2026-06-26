# Focusa Agent Performance Benchmark

**Status:** Specification
**Purpose:** Standardized benchmark for measuring before/after agent performance on metrics that matter, reproducible over time.

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

| Category | Count | Difficulty | Tests |
|----------|-------|------------|-------|
| **L1: Setup** | 10 | Trivial | Install, identify project, define trajectory |
| **L2: Read** | 20 | Easy | View trajectory, workpoint, evidence, doctor |
| **L3: Write** | 20 | Medium | Checkpoint workpoint, link evidence, define goal |
| **L4: Recover** | 10 | Hard | Handle scope conflict, license expired, daemon down |
| **L5: Multi-step** | 15 | Hard | Define trajectory → checkpoint → work → evidence → resume |
| **L6: Cross-session** | 10 | Expert | Compactions, model switches, fork continuations |
| **L7: Adversarial** | 5 | Expert | Conflicting scopes, stale packets, malicious inputs |
| **Total** | **90 tasks** | | |

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
  runners/
    bench.sh           # POSIX runner
    bench.ps1          # Windows runner
    bench.py           # Cross-platform Python runner (recommended)
  scoring/
    score.py           # Aggregates results
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

```json
{
  "suite_id": "v0.9.25-dev-2026-06-25",
  "focusa_version": "0.9.25-dev",
  "spec_versions": {
    "spec_110": "draft",
    "spec_111": "proposed",
    "spec_112": "spec_no_impl"
  },
  "tasks_total": 90,
  "tasks_passed": 67,
  "tasks_failed": 23,
  "pass_rate": 0.744,
  "mean_time_seconds": 45,
  "p95_time_seconds": 180,
  "mean_tokens": 8000,
  "recovery_rate": 0.78,
  "tool_selection_accuracy": 0.65,
  "drift_incidents_per_task": 0.4,
  "by_category": {
    "L1_setup": {"pass_rate": 1.0, "mean_time": 28, "mean_tokens": 3000},
    "L2_read": {"pass_rate": 0.95, "mean_time": 35, "mean_tokens": 5000},
    "L3_write": {"pass_rate": 0.85, "mean_time": 50, "mean_tokens": 8000},
    "L4_recover": {"pass_rate": 0.55, "mean_time": 90, "mean_tokens": 12000},
    "L5_multi": {"pass_rate": 0.65, "mean_time": 120, "mean_tokens": 15000},
    "L6_cross_session": {"pass_rate": 0.40, "mean_time": 180, "mean_tokens": 20000},
    "L7_adversarial": {"pass_rate": 0.20, "mean_time": 300, "mean_tokens": 30000}
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

## 5. Telemetry Hooks (Data Collection)

The benchmark uses Focusa's existing telemetry. We need to add:

### 5.1 Agent Task Lifecycle Events

```json
// POST /v1/telemetry/agent/task-started
{
  "task_id": "L1.001",
  "agent_id": "...",
  "focusa_version": "0.9.25-dev",
  "spec_versions": {...}
}

// POST /v1/telemetry/agent/task-completed
{
  "task_id": "L1.001",
  "duration_seconds": 30,
  "tokens_used": 4500,
  "task_completed": true,
  "tool_calls": [...],
  "errors_encountered": 0,
  "recoveries": 0
}

// POST /v1/telemetry/agent/tool-called
{
  "task_id": "L1.001",
  "tool": "focusa_workpoint_resume",
  "was_canonical": true,  // vs raw shell
  "result_status": "ok"
}
```

### 5.2 Drift Detection Events

```json
// POST /v1/telemetry/agent/drift
{
  "task_id": "L1.001",
  "drift_type": "raw_shell_used",
  "expected_tool": "focusa_workpoint_resume",
  "actual_tool": "bash"
}
```

### 5.3 Recovery Events

```json
// POST /v1/telemetry/agent/recovery
{
  "task_id": "L4.001",
  "error_type": "scope_mismatch",
  "recovery_hint_used": "verify project identity then checkpoint",
  "recovered": true,
  "retries": 1
}
```

---

## 6. Baseline Measurements (What We Need First)

Before we can measure "before/after", we need the **current state baseline**:

### 6.1 Baseline Run (2026-06-26)

**Task:** Run the full 90-task suite against `v0.9.25-dev` (current state with all gaps documented in INSTALL-GAP-AUDIT.md).

**Expected results (prediction, not measurement):**

| Category | Predicted Pass Rate | Why |
|----------|---------------------|-----|
| L1_setup | 0.30 | Real installer drops Python stub (A1) |
| L2_read | 0.85 | Read tools work, wrapper caches sometimes wrong |
| L3_write | 0.75 | Write tools work, schema validation added |
| L4_recover | 0.50 | Recovery hints exist but wrapper drops them |
| L5_multi | 0.60 | Multi-step works, scope issues |
| L6_cross_session | 0.30 | HLT not scoped to project_root (just fixed) |
| L7_adversarial | 0.10 | Spec compliance gaps |
| **Overall** | **~0.55** | Current state has gaps |

### 6.2 Post-Spec 112 Implementation (After Install Fix)

| Category | Predicted Pass Rate | Why |
|----------|---------------------|-----|
| L1_setup | 0.95 | Real installer ships binaries |
| L2_read | 0.90 | Read tools work, daemon running |
| L3_write | 0.80 | Write tools work |
| L4_recover | 0.65 | Recovery hints work |
| L5_multi | 0.70 | Multi-step works |
| L6_cross_session | 0.50 | HLT project_root-scoped |
| L7_adversarial | 0.25 | Spec compliance |
| **Overall** | **~0.75** | Foundation fixed |

### 6.3 Post-Spec 110+111+112 (All Three Shipped)

| Category | Predicted Pass Rate | Why |
|----------|---------------------|-----|
| L1_setup | 0.98 | Real installer + AX |
| L2_read | 0.95 | Bootstrap packet + structured |
| L3_write | 0.90 | Tool nudge to canonical |
| L4_recover | 0.80 | Recovery hints + nudges |
| L5_multi | 0.85 | Multi-step with context |
| L6_cross_session | 0.70 | Bootstrap + persistence |
| L7_adversarial | 0.50 | All guards active |
| **Overall** | **~0.85** | Full stack works |

---

## 7. Reporting & Visualization

### 7.1 Per-Release Report

```
================================================================================
FOCUSA BENCHMARK REPORT — v0.9.25-dev
Run date: 2026-06-26
Spec versions: 110=draft, 111=proposed, 112=spec_no_impl
================================================================================

TASK COMPLETION
  Pass rate:     74.4%  (67/90)
  Mean time:     45.0s
  P95 time:      180.0s
  Mean tokens:   8,000

BY CATEGORY
  L1 setup           10/10  100%   28s   3,000 tok
  L2 read            19/20   95%   35s   5,000 tok
  L3 write           17/20   85%   50s   8,000 tok
  L4 recover          5/10   50%   90s  12,000 tok
  L5 multi           10/15   67%  120s  15,000 tok
  L6 cross-session     4/10   40%  180s  20,000 tok
  L7 adversarial       1/5    20%  300s  30,000 tok

BEHAVIOR METRICS
  Tool selection accuracy:  65%  (canonical focusa_* vs raw shell)
  Drift incidents/task:    0.4
  Recovery rate:            78%  (errors → recovered)
  Backtracks/task:         1.2

EXPERIENCE METRICS
  AX score (self-rated):   3.2 / 5
  Recovery hint useful:    45%
  Bootstrap freshness:     23%  stale (older than 1 hour)

COMPARISON TO BASELINE (v0.9.24-dev)
  Pass rate:    +9.4%  (74.4% vs 65.0%)  ✓
  Token use:   -15.8%  (8,000 vs 9,500)  ✓
  Recovery:    +8.0%  (78% vs 70%)       ✓

REGRESSION ALERTS
  ⚠ L1 setup mean time up 30% (install becoming slower)
  ⚠ L6 cross-session recovery rate down 20%
  ⚠ L7 adversarial pass rate down 5% (regressions in adversarial handling)

VERDICT: IMPROVED — Spec 110/111/112 measurable improvements
```

### 7.2 Comparison Chart (Over Time)

```
Pass Rate Over Time
v0.9.20 ───── 0.50
v0.9.21 ───── 0.55
v0.9.22 ───── 0.58
v0.9.23 ───── 0.62
v0.9.24 ───── 0.65
v0.9.25 ───── 0.744  ← +9.4% (specs 110/111/112 baseline)
v0.9.26 ───── ? (next)
```

---

## 8. Acceptance Criteria for the Benchmark Itself

The benchmark is "real" when:

1. **Reproducible:** Two runs on same version produce same pass rate ±2%
2. **Sensitive:** A 10% improvement in any metric is detectable
3. **Fast:** Full 90-task suite runs in <2 hours
4. **Documented:** Each task has clear expected outcomes
5. **Versioned:** Task suite has versions matching Focusa versions
6. **Honest:** No cherry-picking, no agent-specific tuning
7. **Public:** Anyone can clone, install Focusa, run benchmark, compare results
8. **Automated:** CI runs benchmark on every release

---

## 9. Implementation Roadmap

### Phase 1 (Before MVP Cohort)
- [ ] Create `crates/focusa-bench/` with 90 tasks
- [ ] Implement Python runner (`bench.py`)
- [ ] Add telemetry endpoints (`/v1/telemetry/agent/*`)
- [ ] Run baseline against v0.9.25-dev
- [ ] Publish report

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

## 12. Example Benchmark Run (Predicted)

```bash
$ cd /home/wirebot/focusa
$ bench/run.sh --version v0.9.25-dev --agent focusa-mvp-eval-v1

[INFO] Loading task suite: 90 tasks across 7 categories
[INFO] Starting daemon health check... ok
[INFO] Running L1 setup (10 tasks)...
  L1.001 install... PASS (28s, 3,000 tok)
  L1.002 identify... PASS (15s, 2,000 tok)
  ...
[INFO] Running L2 read (20 tasks)...
  ...
[INFO] Running L3 write (20 tasks)...
  ...
[INFO] Running L4 recover (10 tasks)...
  ...
[INFO] Running L5 multi (15 tasks)...
  ...
[INFO] Running L6 cross-session (10 tasks)...
  ...
[INFO] Running L7 adversarial (5 tasks)...
  ...

[REPORT] v0.9.25-dev pass rate: 74.4% (67/90)
[REPORT] Mean time: 45.0s
[REPORT] Mean tokens: 8,000
[REPORT] Tool selection accuracy: 65%
[REPORT] Saved: reports/2026-06-26-v0.9.25-dev.json
```

---

## 13. Comparison to v0.9.25 (After Spec 110+111+112 Ship)

When all 3 specs are implemented, expected results:

| Category | Pre-Specs | Post-Specs | Δ |
|----------|-----------|------------|---|
| L1 setup | 0.30 | 0.98 | +0.68 |
| L2 read | 0.85 | 0.95 | +0.10 |
| L3 write | 0.75 | 0.90 | +0.15 |
| L4 recover | 0.50 | 0.80 | +0.30 |
| L5 multi | 0.60 | 0.85 | +0.25 |
| L6 cross-session | 0.30 | 0.70 | +0.40 |
| L7 adversarial | 0.10 | 0.50 | +0.40 |
| **Overall** | **0.55** | **0.85** | **+0.30** |

**Token reduction: 30%** (from 9,500 → 6,500 mean tokens)
**Time reduction: 40%** (from 75s → 45s mean time)
**Recovery rate up: 50% → 80%**

This is the honest bet. Measure first, then verify.

---

## 14. The Big Insight

**Without this benchmark, we cannot honestly claim "Spec X improved agent performance."**

**With this benchmark, every spec change has measurable before/after.**

This is the difference between "trust me bro" engineering and "show me the data" engineering. Focusa needs this. The MVP Cohort needs this. Every future spec needs this.

The benchmark itself is a P0 deliverable. Without it, the MVP Cohort evaluation is just vibes.
EOF

echo "Benchmark spec created"
wc -l /home/wirebot/focusa/docs/AGENT-BENCHMARK-SPEC.md