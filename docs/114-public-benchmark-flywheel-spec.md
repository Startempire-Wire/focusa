# Spec 114 — Focusa Public Benchmark Flywheel and Evidence Observatory

**Spec number:** 114  
**Status:** Draft / implementation-ready specification  
**Owner:** Verious Smith  
**Created:** 2026-06-25  
**Extends:** `docs/113-agent-benchmark-spec.md`  
**Primary public domain:** `bench.focusa.dev`  
**Related technical domains:** `evals.focusa.dev`, `proof.focusa.dev`  
**Purpose:** Mesh Focusa's public benchmark marketing surface with the actual core improvement loop so the world can observe Focusa-vs-No-Focusa performance, agent failures, fixes, reruns, promotion decisions, and public-safe proof bundles.

---

## 0. One-Line Product Definition

Focusa Public Benchmark Flywheel is the public evidence system that shows how Focusa improves AI coding agents and how failed evals become the next Focusa improvements.

Short form:

```text
Same agent. Same task. Focusa ON vs Focusa OFF.
Every failure becomes a fixture.
Every fixture becomes a product improvement.
Every improvement ships with proof.
```

---

## 1. Domain Naming and Product Boundary

The public product MUST use Focusa's subdomain pattern.

### 1.1 Domain authority

| Domain | Role | Audience | Meaning |
| --- | --- | --- | --- |
| `bench.focusa.dev` | Public benchmark, scoreboard, evidence observatory | Buyers, developers, investors, cohort members | Measured Focusa-vs-No-Focusa performance and improvement over time |
| `evals.focusa.dev` | Technical eval system, task suite, run ledger, failure lab | Operator, contributors, advanced technical users | Test cases, eval runs, failures, candidates, promotions, schemas |
| `proof.focusa.dev` | Public-safe proof bundle viewer | Public + technical buyers | Immutable redacted evidence behind a claim |

### 1.2 Vocabulary rule

```text
Evals are the engine.
Bench is the public scoreboard.
Proof is the receipt.
```

Do not use `evals.focusa.dev` as the flagship public marketing site. `evals` sounds internal and technical. The main public claim surface is `bench.focusa.dev`.

### 1.3 Site responsibilities

```text
bench.focusa.dev
  /                  public benchmark homepage
  /latest            latest public-safe benchmark snapshot
  /releases          Focusa release-over-release benchmark history
  /models            model × scenario Focusa Uplift matrix
  /scenarios         L1-L12 scenario breakdown
  /runs/:run_id      public-safe run summary
  /failures          public failure taxonomy and resolved/regressed failures
  /improvements      failure-to-fix board
  /methodology       benchmark methodology, limitations, claim policy
  /proof/:snapshot_id redirects or embeds proof.focusa.dev snapshot

evals.focusa.dev
  /tasks             task suite explorer
  /suites            suite versions and splits
  /runs              technical run ledger browser
  /ledger            append-only event explorer
  /failures          failure classifier output
  /candidates        improvement candidate queue
  /promotions        promotion/rollback decisions
  /holdout           private holdout manifest hashes only
  /schemas           JSON schemas and runner contracts

proof.focusa.dev
  /snapshots/:snapshot_id
  /runs/:run_id
  /release/:version
```

---

## 2. Relationship to Spec 113

Spec 113 defines the benchmark.

Spec 114 defines the productized public improvement flywheel and observatory built around that benchmark.

Spec 113 asks:

```text
How much better is an agent with Focusa than without Focusa?
```

Spec 114 asks:

```text
How does Focusa use those measurements to improve itself, and how do we show that publicly without hype, leakage, or false claims?
```

Spec 114 MUST NOT weaken Spec 113. It adds:

1. Public observability through `bench.focusa.dev`.
2. Technical eval-system separation through `evals.focusa.dev`.
3. Immutable public-safe receipts through `proof.focusa.dev`.
4. Failure-to-roadmap conversion.
5. Eval-backed promotion gates.
6. Release-over-release improvement narratives.
7. Public marketing claim generation from measured artifacts only.

---

## 3. Current Repo Grounding

This spec is grounded in the current Focusa repo state as of this writing.

### 3.1 Existing benchmark spec

`docs/113-agent-benchmark-spec.md` already defines:

- the core Focusa-vs-No-Focusa question,
- ranked benchmark metrics,
- a 150-task suite across L1-L12,
- four arms: `no_focusa`, `passive_focusa`, `tool_only_focusa`, `full_focusa`,
- Eval Ledger API shape under `/v1/evals/*`,
- reporting and visualization requirements,
- public evidence bundle requirements,
- model × scenario matrix,
- release-over-release comparison,
- valid public claim template.

### 3.2 Existing static eval layer

The current repo already has a static advisory eval layer:

```text
docs/current/FOCUSA_AGENT_INTELLIGENCE_EVALS.md
tests/evals/agent_intelligence_cases.json
scripts/run-agent-intelligence-evals.sh
tests/agent_intelligence_benchmark_static_test.sh
```

This layer validates category coverage and score thresholds for:

```text
continuity
scope
evidence
context
execution
learning
safety
```

This is valuable, but it is not the full Spec 113 live benchmark. Spec 114 classifies it as:

```text
L0_internal_static_quality
```

### 3.3 Existing public proof and redaction surfaces

The repo already defines public-safe proof and redaction boundaries:

```text
docs/current/PUBLIC_PROOF_BUNDLE_VIEWER.md
docs/current/PUBLIC_STREAM_REDACTION_POLICY.md
crates/focusa-api/src/routes/awareness.rs
```

Spec 114 MUST reuse those boundaries. It must not invent a separate public publishing policy.

### 3.4 Existing optimizer pattern

The repo already has a smaller version of the promotion pattern in Context Cognition:

```text
crates/focusa-api/src/routes/context_cognition.rs
```

The existing `curate_eval` / `curate_optimize` path records eval results, compares baseline/eval scores, and promotes only when score improves and meets threshold. Spec 114 generalizes this pattern from Context Cognition to Focusa-wide benchmark-backed product improvement.

### 3.5 Current implementation gap

At the time of this spec, `/v1/evals/*` is specified in docs but not yet visible as an implemented API route module in:

```text
crates/focusa-api/src/routes/mod.rs
crates/focusa-api/src/server.rs
```

Spec 114 therefore treats Eval Ledger implementation as a P0 deliverable.

---

## 4. Core Product Loop

The complete flywheel is:

```text
1. Run matched benchmark arms.
2. Capture append-only Eval Ledger events.
3. Score Focusa-vs-No-Focusa.
4. Classify failures.
5. Convert repeatable failures into Improvement Candidates.
6. Create or update specs/beads/Workpoints.
7. Patch Focusa.
8. Rerun focused evals and release suite.
9. Promote only if measured improvement beats baseline.
10. Generate redacted public proof snapshot.
11. Publish to bench.focusa.dev and proof.focusa.dev.
```

Engineering view:

```text
eval failure → classified defect → spec/workpoint → implementation → rerun → promotion
```

Marketing view:

```text
visible failure → visible fix → visible measured improvement → trust
```

---

## 5. Non-Negotiable Principles

### 5.1 Focusa-vs-No-Focusa is the primary public story

Every public benchmark page MUST default to:

```text
full_focusa vs no_focusa
```

Ablations are secondary:

```text
passive_focusa
tool_only_focusa
```

They explain why Focusa helped, but they do not replace the market claim.

### 5.2 Measured claims only

No public page, card, launch post, release note, pricing page, or sales page may claim improvement unless the claim is backed by a completed Eval Ledger run and public-safe proof bundle.

Valid:

```text
On focusa-agent-bench-v1 run <run_id>, Focusa improved L6 cross-session completion from 42% to 61% versus No-Focusa using <model/version>, with raw artifacts and scoring commit <sha>.
```

Invalid:

```text
Focusa makes all agents smarter.
Focusa doubles developer productivity.
Focusa makes cheap models as good as frontier models.
```

### 5.3 Failures are product assets

A failed eval is not embarrassment. It is a captured market-relevant defect.

Each repeatable failure should become one of:

```text
improvement_candidate
regression_fixture
spec_amendment
new_spec
bead/task
workpoint
release_blocker
public_known_limit
```

### 5.4 The system must not self-mutate silently

Eval results may recommend changes, open improvement candidates, or block promotion.

Eval results must not directly mutate:

```text
Focus State
Trajectory
Workpoint authority
Context Authority gates
Prompts
Ontology
Agent routing
Release status
Public publication status
```

Human/operator or explicit release automation remains the promotion authority.

### 5.5 Public display is deny-by-default

The observatory must publish summaries, evidence refs, hashes, scorecards, and redacted timelines.

It must not publish:

```text
raw logs
raw private prompts
raw diffs unless explicitly public-safe
tokens
secrets
private file contents
sensitive browser diagnostics
unredacted paths
host environment secrets
private holdout task bodies
```

---

## 6. Architecture Overview

```text
crates/focusa-bench/
  ↓
/v1/evals/*
  ↓
Eval Ledger persistence
  ↓
CTL read-only joins
  ↓
Scoring + comparison engine
  ↓
Failure classifier
  ↓
Improvement candidate generator
  ↓
Spec / bead / Workpoint bridge
  ↓
Release promotion gate
  ↓
Public proof snapshot generator
  ↓
bench.focusa.dev
  ↓
proof.focusa.dev
```

The Eval Ledger is the central source of benchmark evidence.

The public site is a read model generated from completed, redacted, public-safe snapshots.

---

## 7. New Crate: `crates/focusa-bench`

Create:

```text
crates/focusa-bench/
  Cargo.toml
  src/
    lib.rs
    main.rs
    task.rs
    model_matrix.rs
    runner.rs
    arms.rs
    scoring.rs
    compare.rs
    failure.rs
    public_snapshot.rs
    redaction.rs
  tasks/
    L1_setup/
    L2_read/
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
    model_matrix.json
  splits/
    public.json
    private_holdout.manifest.json
  runners/
    bench.py
    ablate.py
    replay.py
  scoring/
    score.py
    market_score.py
    confidence.py
  reports/
  runs/
```

### 7.1 MVP runner rule

The first production runner may be Python for implementation speed, but the schemas should live in Rust types or generated JSON schema so API, CLI, and runner stay aligned.

### 7.2 Task file schema

```json
{
  "schema": "focusa.bench.task.v1",
  "suite_id": "focusa-agent-bench-v1",
  "task_id": "L6.001",
  "category": "L6_cross_session",
  "title": "Resume exact Workpoint after compaction",
  "difficulty": "expert",
  "public_split": true,
  "holdout": false,
  "seed": 12345,
  "agent_prompt": "Resume the previous task after context compaction and identify the exact next action.",
  "preconditions": {
    "project_root_fixture": "fixtures/svelte-app",
    "focusa_state_fixture": "fixtures/workpoint-with-evidence.json",
    "requires_daemon": true,
    "requires_browser": false,
    "requires_license_mode": "eval"
  },
  "expected_outcome": {
    "resolved": true,
    "max_duration_seconds": 300,
    "max_tokens": 20000,
    "required_tools": ["focusa_workpoint_resume"],
    "forbidden_behaviors": [
      "transcript_tail_as_authority",
      "wrong_project_root",
      "missing_evidence_claim"
    ]
  },
  "verification": {
    "judge": "deterministic",
    "commands": [
      "focusa workpoint resume --project-root <fixture> --continuity-id <id> --json"
    ],
    "expected_json_paths": {
      "$.canonical": true,
      "$.workpoint.next_slice": "exact_next_action"
    }
  },
  "scoring": {
    "resolved": 100,
    "exact_next_action": 40,
    "used_canonical_tool": 30,
    "no_scope_drift": 30,
    "evidence_linked": 20
  }
}
```

---

## 8. Eval Ledger API

Add new API module:

```text
crates/focusa-api/src/routes/evals.rs
```

Register it in:

```text
crates/focusa-api/src/routes/mod.rs
crates/focusa-api/src/server.rs
```

### 8.1 Routes

```http
POST /v1/evals/runs
POST /v1/evals/runs/{run_id}/events
POST /v1/evals/runs/{run_id}/complete
GET  /v1/evals/runs/{run_id}
GET  /v1/evals/runs
GET  /v1/evals/compare
GET  /v1/evals/failures
POST /v1/evals/failures/{failure_id}/candidate
GET  /v1/evals/improvement-candidates
POST /v1/evals/promotions
GET  /v1/evals/public/snapshots
GET  /v1/evals/public/snapshots/{snapshot_id}
```

### 8.2 Required write contract

All write routes require:

```json
{
  "eval_mode": true,
  "schema_version": "focusa.eval_event.v1",
  "suite_id": "focusa-agent-bench-v1",
  "run_id": "run-2026-06-26-001",
  "task_id": "L6.001",
  "scenario_id": "L6_cross_session",
  "arm": "no_focusa | passive_focusa | tool_only_focusa | full_focusa",
  "agent_id": "claude-code",
  "model_provider": "anthropic",
  "model_id": "claude-sonnet-4.5",
  "model_version": "2026-06-26",
  "model_class": "frontier_generalist",
  "environment_id": "linux-x86_64-glibc",
  "prompt_hash": "sha256:...",
  "task_seed": 12345,
  "pricing_snapshot": "2026-06-26",
  "provider_routing_locked": true
}
```

### 8.3 Append-only invariant

Eval events are append-only and idempotent by `event_id`.

No endpoint may edit a prior event in place.

Corrections are represented as new events:

```json
{
  "event": "correction",
  "corrects_event_id": "evt_123",
  "reason": "judge artifact path normalized",
  "created_at": "..."
}
```

### 8.4 Disallowed side effects

`/v1/evals/*` must not mutate:

```text
Focus State
Workpoint state
Trajectory
Context Authority
Ontology
agent prompts
Context Cognition promoted artifacts
release status
public stream status
```

Allowed side effects:

```text
append eval event
append eval run summary
append failure classification
append improvement candidate
append promotion decision record
generate local/public-safe snapshot candidate
```

---

## 9. Eval Event Types

Minimum event types:

```text
run_started
task_started
agent_prompt_delivered
tool_call
tool_result
drift
recovery_hint
operator_intervention
judge_result
task_completed
task_failed
failure_classified
candidate_created
candidate_linked_to_spec
candidate_linked_to_bead
candidate_linked_to_workpoint
rerun_started
promotion_decision
public_snapshot_candidate
public_snapshot_published
run_completed
```

### 9.1 Example event

```json
{
  "schema_version": "focusa.eval_event.v1",
  "event_id": "evt_01",
  "event": "drift",
  "run_id": "run-2026-06-26-001",
  "task_id": "L3.004",
  "scenario_id": "L3_write",
  "arm": "tool_only_focusa",
  "agent_id": "pi",
  "model_id": "claude-sonnet-4.5",
  "timestamp": "2026-06-26T14:10:00Z",
  "drift_type": "raw_shell_for_focusa_api",
  "expected_tool": "focusa_workpoint_checkpoint",
  "actual_tool": "bash",
  "severity": "medium",
  "public_safe": true,
  "redaction_status": "redacted_scope_only"
}
```

---

## 10. Storage Layout

Default local storage:

```text
data/evals/
  runs/
    <run_id>/
      run.json
      events.jsonl
      summary.json
      score.json
      compare.json
      failures.jsonl
      artifacts/
        report.json
        report.md
        raw-ledger.sha256
        scoring-commit.txt
        environment-digest.json
  failures/
    failures.jsonl
  improvement-candidates/
    candidates.jsonl
  promotions/
    promotion-decisions.jsonl
  public/
    snapshots/
      <snapshot_id>.json
      <snapshot_id>.md
      <snapshot_id>.sha256
```

### 10.1 Hash chain

Each `events.jsonl` row should include:

```json
{
  "event_hash": "sha256:<current_event_hash>",
  "previous_event_hash": "sha256:<prior_event_hash>"
}
```

This makes benchmark evidence tamper-evident.

---

## 11. Scoring Model

### 11.1 Primary scores

```text
Resolved %
Agent Power Index
Focusa Uplift Score
Cost per resolved task
Time to resolution
METR-style time horizon @ 50%
Pass^N
Pass@k
Grounded claims %
Operator burden ratio
Tool selection accuracy
Drift incidents per task
Recovery rate
Backtrack count
Cross-session continuity
```

### 11.2 Focusa Uplift Score

```text
Focusa Uplift Score = AgentPowerIndex(full_focusa) / AgentPowerIndex(no_focusa)
```

### 11.3 Agent Power Index

Initial formula:

```text
APIx =
  0.35 * resolved_score
+ 0.15 * time_horizon_score
+ 0.15 * cost_efficiency_score
+ 0.10 * groundedness_score
+ 0.10 * continuity_score
+ 0.10 * recovery_score
+ 0.05 * operator_burden_score
```

All formula weights must be versioned in:

```text
crates/focusa-bench/scoring/market_score.py
docs/current/FOCUSA_AGENT_POWER_INDEX.md
```

### 11.4 Honesty requirement

The public UI must always show raw metric deltas beside composite scores.

Never show only the composite score.

---

## 12. Failure Taxonomy

Each failed task receives one primary failure class and optional secondary tags.

```text
agent_reasoning_failure
agent_instruction_following_failure
tool_selection_failure
tool_argument_failure
focusa_runtime_failure
focusa_scope_authority_failure
focusa_workpoint_failure
focusa_evidence_failure
focusa_context_failure
focusa_bootstrap_failure
focusa_installer_failure
focusa_license_failure
focusa_redaction_failure
environment_failure
model_refusal
operator_intervention_failure
judge_failure
benchmark_harness_failure
unknown
```

### 12.1 Focusa subsystem mapping

```text
tool_selection_failure           → Spec 110 / agent reminder / Pi tool layer
focusa_bootstrap_failure         → Spec 111 / AgentBootstrapPacket
focusa_installer_failure         → Spec 112 / installer
focusa_scope_authority_failure   → ProjectIdentity / Context Authority
focusa_workpoint_failure         → Workpoint resume/checkpoint
focusa_evidence_failure          → evidence capture/link / claim discipline
focusa_context_failure           → Context Cognition / curator / optimizer
focusa_redaction_failure         → Public Stream / Proof Bundle Viewer
focusa_runtime_failure           → daemon/API/CLI/TUI/menubar
```

---

## 13. Improvement Candidate Schema

When a failure is repeatable or severe, create an improvement candidate.

```json
{
  "schema": "focusa.improvement_candidate.v1",
  "candidate_id": "ic_2026_06_26_001",
  "source": "eval_failure",
  "run_id": "run-2026-06-26-001",
  "task_id": "L6.001",
  "scenario_id": "L6_cross_session",
  "failure_class": "focusa_workpoint_failure",
  "affected_subsystems": ["workpoint", "awareness", "session_transfer"],
  "public_safe_summary": "Agent resumed a stale next action after compaction.",
  "private_details_ref": "data/evals/runs/<run_id>/failures.jsonl#...",
  "expected_behavior": "Use canonical Workpoint resume packet as immediate authority.",
  "actual_behavior": "Used transcript tail as authority.",
  "recommended_fix": "Tighten Workpoint resume utility card and add regression fixture.",
  "recommended_spec_action": "amend_existing_spec",
  "spec_refs": ["Spec 88", "Spec 111", "Spec 113"],
  "bead_refs": [],
  "workpoint_ref": null,
  "status": "candidate",
  "severity": "high",
  "repeatability": "confirmed",
  "created_at": "2026-06-26T14:30:00Z"
}
```

### 13.1 Candidate statuses

```text
candidate
accepted
rejected
spec_required
bead_created
workpoint_created
implementation_started
fix_available
rerun_pending
promoted
rolled_back
published
```

---

## 14. Spec / Bead / Workpoint Bridge

The Eval Ledger must not directly create durable product authority by default.

Instead, it creates candidates.

Operator-approved candidate promotion may create:

```text
new spec
spec amendment
bead/task
Workpoint
regression fixture
release blocker
known limitation
```

### 14.1 CLI bridge commands

```bash
focusa eval failures --run <run_id> --json
focusa eval candidate create --failure <failure_id>
focusa eval candidate accept <candidate_id>
focusa eval candidate link-spec <candidate_id> --spec docs/114-public-benchmark-flywheel-spec.md
focusa eval candidate link-bead <candidate_id> --bead focusa-xxxx
focusa eval candidate checkpoint-workpoint <candidate_id>
focusa eval promote --candidate <candidate_id> --baseline <run_id> --candidate-run <run_id>
```

### 14.2 Workpoint bridge

When an improvement candidate becomes a Workpoint, the Workpoint should include:

```text
mission = fix the failure class
next_slice = smallest safe implementation action
verification_records = eval failure refs + rerun proof refs
active_object_refs = related spec/code/test paths
do_not_drift = original failure boundaries
```

---

## 15. Promotion Policy

A Focusa change may be promoted as eval-backed only when:

```text
candidate run score > baseline score
no critical regression
public smoke split passes
private holdout does not regress beyond threshold
redaction scan passes
evidence bundle generated
scoring code commit recorded
environment digest recorded
operator/release gate accepts
```

### 15.1 Promotion decision schema

```json
{
  "schema": "focusa.eval_promotion_decision.v1",
  "promotion_id": "prom_2026_06_26_001",
  "candidate_id": "ic_2026_06_26_001",
  "baseline_run_id": "run_before",
  "candidate_run_id": "run_after",
  "decision": "promote | rollback | inconclusive | blocked",
  "metric_deltas": {
    "resolved_delta": 0.12,
    "focusa_uplift_delta": 0.18,
    "cost_per_resolved_delta": -0.09,
    "groundedness_delta": 0.21
  },
  "regressions": [],
  "confidence": {
    "method": "bootstrap_ci",
    "ci_95": [0.04, 0.20]
  },
  "required_evidence": {
    "raw_ledger": "data/evals/runs/run_after/events.jsonl",
    "score_report": "data/evals/runs/run_after/score.json",
    "environment_digest": "data/evals/runs/run_after/environment-digest.json",
    "scoring_commit": "abc123"
  },
  "public_snapshot_id": "snap_2026_06_26_001",
  "created_at": "2026-06-26T16:00:00Z"
}
```

---

## 16. Public Snapshot System

Public snapshots are immutable redacted artifacts generated from completed eval runs.

### 16.1 Snapshot states

```text
draft_private
redaction_pending
publish_blocked
publish_ready
published_snapshot
revoked
```

### 16.2 Snapshot schema

```json
{
  "schema": "focusa.public_benchmark_snapshot.v1",
  "snapshot_id": "snap_2026_06_26_001",
  "state": "published_snapshot",
  "title": "Focusa Agent Performance Benchmark — v0.9.26",
  "focusa_version": "0.9.26-dev",
  "suite_id": "focusa-agent-bench-v1",
  "public_domain": "bench.focusa.dev",
  "proof_domain": "proof.focusa.dev",
  "technical_eval_domain": "evals.focusa.dev",
  "run_ids": {
    "no_focusa": "run_no_focusa",
    "passive_focusa": "run_passive",
    "tool_only_focusa": "run_tool_only",
    "full_focusa": "run_full"
  },
  "primary_comparison": "full_focusa_vs_no_focusa",
  "headline_metrics": {
    "resolved_no_focusa": 0.42,
    "resolved_full_focusa": 0.61,
    "resolved_delta": 0.19,
    "focusa_uplift_score": 1.37,
    "cost_per_resolved_delta": -0.12,
    "groundedness_delta": 0.22
  },
  "model_matrix_summary": {
    "models_tested": 3,
    "model_classes": ["frontier", "budget", "open_weight"]
  },
  "scenario_matrix": {},
  "release_over_release": {},
  "weak_model_close_rate": {},
  "failure_summary": {},
  "known_limitations": [],
  "redaction": {
    "redaction_status": "passed",
    "secret_scan_status": "passed",
    "raw_logs_included": false,
    "raw_diffs_included": false,
    "private_file_contents_included": false,
    "publish_allowed": true
  },
  "evidence": {
    "raw_ledger_hash": "sha256:...",
    "public_task_files_ref": "crates/focusa-bench/tasks/public",
    "private_holdout_manifest_hash": "sha256:...",
    "scoring_commit": "abc123",
    "environment_digest_hash": "sha256:..."
  },
  "claim_text": "On focusa-agent-bench-v1, Focusa improved resolved rate from 42% to 61% versus No-Focusa using pinned model runs. Raw artifacts are linked in this snapshot."
}
```

### 16.3 Public claim generation

Public pages must use generated claim text from snapshot JSON.

Marketing copy should not manually invent numbers.

---

## 17. Public Website: `bench.focusa.dev`

### 17.1 Primary page sections

```text
Hero
Focusa-vs-No-Focusa headline card
Live / latest benchmark runs
Focusa Uplift Score trend
Model × scenario matrix
Failure-to-fix improvement board
Task replay theater
Evidence bundle explorer
Methodology and limitations
Raw artifact download / hash verification
```

### 17.2 Hero

```text
Same agent.
Same task.
One run loses the mission.
One run keeps it.

Focusa measures the difference.
```

### 17.3 Headline card

```text
FOCUSA AGENT PERFORMANCE BENCHMARK — <version>

Suite: focusa-agent-bench-v1
Primary comparison: full_focusa vs no_focusa
Models tested: <n>
Tasks: <n>

Resolved:              <full_focusa> vs <no_focusa>
Focusa Uplift Score:   <ratio>
Cost per resolved:     <delta>
Grounded claims:       <delta>
Time horizon @50%:     <delta>
Operator burden:       <delta>

Evidence:
Run IDs
Raw ledger hash
Scoring commit
Environment digest
Proof snapshot
```

### 17.4 Failure-to-fix board

This is the unique Focusa marketing layer.

Columns:

```text
Observed failure
Failure class
Affected Focusa subsystem
Improvement candidate
Spec/bead/workpoint
Rerun result
Promotion decision
Public proof
```

Example public card:

```text
Failure:
Agent resumed stale next action after compaction.

Focusa subsystem:
Workpoint + Awareness + Bootstrap

Fix:
Tightened Workpoint resume card and added regression fixture.

Before:
L6 continuity pass rate: 40%

After:
L6 continuity pass rate: 58%

Status:
Promoted with evidence
```

### 17.5 Task replay theater

Each task replay shows:

```text
task prompt hash
arm
timeline
tool calls
drift events
judge result
evidence refs
redaction status
failure classification
score
```

It must not show private raw transcript unless explicitly public-safe.

### 17.6 Honesty rail

Every page should include a persistent honesty rail:

```text
Measured vs hypothesis
Public split vs private holdout
Known limitations
Model/version/date
Pricing snapshot
Scoring commit
Environment digest
Redaction status
```

---

## 18. Public Data API

The public site should consume static or read-only public snapshot JSON.

Recommended public artifact path:

```text
public/bench/snapshots/<snapshot_id>.json
public/bench/latest.json
public/bench/releases/<focusa_version>.json
public/bench/models/<model_id>.json
```

Do not expose local daemon `/v1/evals/*` directly to the public internet.

The daemon generates snapshots. The website serves immutable public-safe artifacts.

---

## 19. Redaction and Publication Gate

A snapshot cannot be published unless:

```text
publish_allowed = true
redaction_status = passed
secret_scan_status = passed | not_required_no_raw_payload
evidence_refs_public_safe only
no raw logs
no raw token payloads
no raw private prompts
no private file contents
no sensitive browser diagnostics
no unredacted project paths
no raw diffs unless explicitly public-safe
private holdout bodies excluded
```

### 19.1 Secret scan

Add script:

```text
scripts/scan-benchmark-public-snapshot.mjs
```

It should scan for:

```text
API keys
tokens
absolute private paths
home directories
SSH material
.env values
raw diffs
raw logs
browser URLs with query secrets
private file body markers
```

---

## 20. Release Integration

Every Focusa release should include:

```bash
focusa eval run --suite smoke --arms no_focusa,full_focusa
focusa eval compare --baseline previous_release --candidate current_release
focusa eval public-snapshot create --run <run_id>
focusa eval public-snapshot verify <snapshot_id>
focusa release prove --tag <tag> --include-benchmark <snapshot_id>
```

### 20.1 Release verdicts

```text
MEASURED_IMPROVED
MIXED
REGRESSED
INCONCLUSIVE
BLOCKED_NO_EVIDENCE
```

### 20.2 Release note rule

Release notes may say:

```text
This release improved Focusa's L6 continuity score by X on benchmark run Y.
```

Release notes may not say:

```text
This release improves agent performance.
```

unless the statement names the exact metric, run, model, suite, and confidence interval.

---

## 21. CI Plan

### 21.1 PR CI

Run:

```bash
scripts/run-agent-intelligence-evals.sh
tests/agent_intelligence_benchmark_static_test.sh
tests/eval_metrics_dashboard_static_test.sh
tests/public_proof_bundle_viewer_static_test.sh
tests/public_stream_redaction_policy_static_test.sh
tests/spec113_benchmark_contract_static_test.sh
tests/spec114_public_benchmark_flywheel_static_test.sh
```

### 21.2 Nightly CI

Run:

```bash
crates/focusa-bench/runners/bench.py --suite smoke --arms no_focusa,full_focusa
crates/focusa-bench/scoring/score.py --run <run_id>
crates/focusa-bench/scoring/market_score.py --run <run_id>
scripts/generate-eval-dashboard-readmodel.mjs
```

### 21.3 Release CI

Run:

```bash
crates/focusa-bench/runners/ablate.py \
  --suite focusa-agent-bench-v1 \
  --arms no_focusa,passive_focusa,tool_only_focusa,full_focusa

crates/focusa-bench/scoring/score.py
crates/focusa-bench/scoring/confidence.py
scripts/generate-benchmark-public-snapshot.mjs
scripts/scan-benchmark-public-snapshot.mjs
scripts/prove-benchmark-public-snapshot.mjs
```

---

## 22. Static Tests to Add

```text
tests/spec113_eval_ledger_api_contract_static_test.sh
tests/spec114_public_benchmark_flywheel_static_test.sh
tests/spec114_public_snapshot_redaction_static_test.sh
tests/spec114_claim_policy_static_test.sh
tests/spec114_failure_candidate_schema_static_test.sh
tests/spec114_no_telemetry_mutation_for_evals_static_test.sh
tests/spec114_router_evals_registration_static_test.sh
tests/spec114_public_site_artifact_contract_static_test.sh
```

### 22.1 Critical static assertions

The tests must assert:

```text
crates/focusa-api/src/routes/evals.rs exists
routes/mod.rs contains pub mod evals
server.rs merges routes::evals::router()
/v1/evals strings exist only in eval route/docs/tests
benchmark write paths do not target /v1/telemetry/*
public snapshots include redaction_status
public snapshots include secret_scan_status
public snapshots include publish_allowed
public snapshots include raw_ledger_hash, not raw private logs
claim templates include run_id, model_version, metric, confidence/evidence
bench.focusa.dev is the public benchmark domain
evals.focusa.dev is the technical eval-system domain
proof.focusa.dev is the public-safe receipt domain
```

---

## 23. API Live-Safe Tests to Add

```text
tests/spec114_eval_ledger_live_safe_test.sh
tests/spec114_eval_idempotency_live_safe_test.sh
tests/spec114_eval_compare_live_safe_test.sh
tests/spec114_public_snapshot_live_safe_test.sh
tests/spec114_failure_candidate_live_safe_test.sh
```

### 23.1 Live-safe test cases

1. Create eval run.
2. Append `task_started`.
3. Append duplicate event with same `event_id`; assert idempotent result.
4. Append `tool_call`.
5. Append `judge_result`.
6. Complete run.
7. Read run.
8. Compare two runs.
9. Generate public snapshot candidate.
10. Verify snapshot blocks when `publish_allowed=false`.
11. Verify snapshot publishes only after redaction/secret scan pass.
12. Verify no Workpoint/Trajectory/FocusState mutation occurred.

---

## 24. Integration with Current Static Agent Intelligence Evals

The current `FOCUSA_AGENT_INTELLIGENCE_EVALS.md` and `tests/evals/agent_intelligence_cases.json` should become:

```text
L0_internal_static_quality
```

Purpose:

```text
Fast repo-level smoke check for categories and promotion-boundary discipline.
```

It should not pretend to be the full Spec 113 benchmark.

Map current categories into Spec 113:

```text
continuity → L6_cross_session
scope      → L7_adversarial + L4_recover
evidence   → L12_grounded_claims
context    → L2_read + L5_multi + Context Cognition evals
execution  → L8_real_coding + Call Stack Verify
learning   → L10_company_workflow + metacog/prediction loops
safety     → L4_recover + L7_adversarial
```

---

## 25. Integration with Context Cognition Optimizer

Context Cognition already has the right mini-pattern:

```text
baseline score
eval score
threshold
promote or rollback
append artifact
```

Spec 114 should generalize that pattern to benchmark-wide product changes.

Do not replace the Context Cognition optimizer.

Use it as the reference design for:

```text
EvalPromotionDecision
ImprovementCandidate
Promotion gate
Rollback record
Optimizer artifact read model
```

---

## 26. Integration with Workpoints and Evidence

Final benchmark artifacts should be linked as Workpoint evidence only after a completed run.

Example:

```json
{
  "target_ref": "benchmark:spec-113:run-2026-06-26-001",
  "evidence_ref": "data/evals/runs/run-2026-06-26-001/report.json",
  "result": "full_focusa beat no_focusa on L6 continuity with public-safe proof bundle"
}
```

Failure candidates should not automatically become Workpoints.

When accepted by operator/release process, create Workpoint:

```json
{
  "mission": "Fix L6 continuity failure: stale next action after compaction",
  "next_slice": "Add regression fixture and tighten Workpoint resume card",
  "active_object_refs": [
    "docs/113-agent-benchmark-spec.md",
    "docs/114-public-benchmark-flywheel-spec.md",
    "crates/focusa-api/src/routes/workpoint.rs",
    "tests/evals/L6_cross_session/..."
  ],
  "verification_records": [
    {
      "target_ref": "eval_failure:<failure_id>",
      "evidence_ref": "data/evals/runs/<run_id>/failures.jsonl"
    }
  ]
}
```

---

## 27. UX: What the World Should See

The public site should intentionally show three things side by side:

```text
1. Market proof
2. Engineering honesty
3. Improvement velocity
```

### 27.1 Market proof

```text
Focusa beats No-Focusa on measured agent performance metrics.
```

### 27.2 Engineering honesty

```text
Here are the categories where Focusa regressed, failed, or was inconclusive.
```

### 27.3 Improvement velocity

```text
Here is how quickly failures became fixtures, fixes, reruns, and promoted improvements.
```

This is the differentiator.

Most benchmark sites show static scores.

Focusa should show the living loop.

---

## 28. Visual Language

The site should look like a mission-control evidence wall.

Primary components:

```text
Split-run comparison
Focusa Delta chart
Run timeline
Failure heatmap
Improvement kanban
Release trend line
Model × scenario matrix
Evidence receipt card
Redaction status badge
Known limitation card
```

### 28.1 Trust badges

```text
measured
hypothesis
public split
private holdout hash
redacted
secret-scan passed
evidence linked
regression
inconclusive
promoted
rolled back
```

---

## 29. Suggested Initial MVP

Do not start with the full 150-task suite.

Start with an MVP benchmark slice:

```text
L0 internal static quality: existing 7 cases
L1 setup: 3 tasks
L2 orientation: 3 tasks
L3 tool use: 3 tasks
L4 recovery: 3 tasks
L6 continuity: 3 tasks
L12 grounded claims: 3 tasks
```

Total MVP:

```text
18 live tasks + 7 static fixture checks
```

Arms:

```text
no_focusa
full_focusa
```

Add ablations after the first clean public smoke run:

```text
passive_focusa
tool_only_focusa
```

---

## 30. Phase Plan

### Phase 0 — Spec reconciliation

Deliverables:

```text
docs/114-public-benchmark-flywheel-spec.md
docs/current/FOCUSA_PUBLIC_BENCHMARK_OBSERVATORY.md
docs/current/FOCUSA_EVAL_PROMOTION_POLICY.md
tests/spec114_public_benchmark_flywheel_static_test.sh
```

### Phase 1 — Eval Ledger

Deliverables:

```text
crates/focusa-api/src/routes/evals.rs
focusa_core::types::EvalRun
focusa_core::types::EvalEvent
focusa_core::types::EvalFailure
focusa_core::types::EvalPromotionDecision
SQLite/file-backed append-only persistence
```

### Phase 2 — Bench runner MVP

Deliverables:

```text
crates/focusa-bench/
18 live task files
model_matrix.json
public split manifest
bench.py
score.py
market_score.py
```

### Phase 3 — Failure candidate loop

Deliverables:

```text
failure classifier
improvement candidate generator
candidate CLI commands
candidate-to-Workpoint bridge
candidate-to-spec/bead docs
```

### Phase 4 — Public snapshot generator

Deliverables:

```text
scripts/generate-benchmark-public-snapshot.mjs
scripts/scan-benchmark-public-snapshot.mjs
scripts/prove-benchmark-public-snapshot.mjs
public/bench/latest.json
public/bench/snapshots/<snapshot_id>.json
```

### Phase 5 — Observatory UI

Deliverables:

```text
bench.focusa.dev
Focusa-vs-No-Focusa headline
trend chart
failure-to-fix board
task replay theater
evidence bundle explorer
methodology page
```

---

## 31. Acceptance Criteria

Spec 114 is accepted when:

1. The repo has this public benchmark flywheel spec linked from Spec 113 or adjacent benchmark docs.
2. `bench.focusa.dev` is defined as the public benchmark domain.
3. `evals.focusa.dev` is defined as the technical eval-system domain.
4. `proof.focusa.dev` is defined as the public-safe receipt domain.
5. `/v1/evals/*` exists as a separate append-only Eval Ledger route.
6. `/v1/evals/*` is registered in the API router.
7. Eval writes do not target `/v1/telemetry/*`.
8. Eval writes do not mutate Focus State, Workpoints, Trajectory, Context Authority, or ontology.
9. Existing static agent intelligence evals are preserved as L0 smoke checks.
10. At least one live benchmark run can be created, appended, completed, read, and compared.
11. At least one failed task can become an Improvement Candidate.
12. At least one Improvement Candidate can be linked to a spec/bead/Workpoint manually or through explicit command.
13. At least one before/after rerun can produce a promotion decision.
14. Public snapshot generation is deny-by-default.
15. Public snapshot publication requires redaction and secret scan pass.
16. The public snapshot includes run IDs, scoring commit, environment digest, raw ledger hash, model/version, and measured claim text.
17. `bench.focusa.dev` or a static preview can render the latest public-safe snapshot.
18. CI includes static and live-safe guards for all critical boundaries.
19. Public copy clearly distinguishes measured values, hypotheses, regressions, and limitations.

---

## 32. Final Vision Statement

Focusa should not market itself as another AI leaderboard.

Focusa should market itself as the first public, eval-backed mission runtime where the world can watch agent failures become product improvements.

Final public framing:

```text
AI benchmarks tell you which model scored higher.

Focusa Bench shows what happens when the same agent gets durable mission state, scoped context, Workpoints, evidence, recovery, and authority — and then shows how every failure becomes the next Focusa improvement.

Same agent. Same task. Focusa ON vs Focusa OFF.
Measured. Redacted. Replayable. Improving.
```
