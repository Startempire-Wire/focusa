# Spec 122 — Self-heal as System-Not-Symptom Fixer

**Status:** draft, iterable, NOT FINAL — operator has not yet signed off.
**Owner:** Focusa / Verious Smith
**Created:** 2026-07-08
**Inherits:** Spec 103 (call stack), Spec 107 (spec-first lifecycle), Spec 120 (adversarial spec workbench — this spec was authored *through* the workbench), Spec 121 (menubar rearchitecture — this spec is upstream of any 121 deliverable that touches `release-proof/audit/audit.jsonl`).
**Replaces:** the current `auto-heal-audit.py` behavior, which is a passive mirror.
**Out of scope:** focusa core daemon changes, menubar UI changes.

---

## 0. One-line definition

Self-heal must progressively make it impossible for the same failure class to occur again by changing the system that produced the error. The immediate error may be patched initially, but the same failure class must never be patched twice — every heal must also add a system fix so the class doesn't recur, and output quality improves over time as system fixes accumulate.

## 1. Normative basis

1.1. **Both-initially rule.** A self-heal for an unseen failure class has TWO actions in the SAME commit: (a) patch the immediate error so CI is up, AND (b) add a system fix (new CI gate, new lint, new type, new hook, new test, new doc, or new script) so the failure class doesn't recur. The patch alone is not enough; the system fix alone is not enough. The heal is **both**.
1.2. **Never patch the same error class twice.** A self-heal that has been done correctly adds a system fix. The next time the class appears, the system fix should catch it. A second manual patch for the same class is a system-fix failure: it means the previous self-heal's deliverable was not effective. The system fix is escalated.
1.3. **No manual intervention goal.** Every self-heal exists to reduce the long-run operator intervention rate on CI failures. A self-heal that does not reduce that rate is INVALID.
1.4. **Fail-class-first.** Self-heal does not begin with "this specific error". It begins with "this class of error". If 3 clippy failures of the same class appear in 7 days, the heal is the lint configuration that catches the class — not 3 patches.
1.5. **Heal must produce a deliverable.** Every self-heal row in `release-proof/audit/audit.jsonl` is paired with a `deliverable` reference — a PR number, a script path, a CI gate definition, a hook file, a type file. Passive rows are INVALID.
1.6. **Verifiable close.** A self-heal's `closed: true` requires the failure class to no longer reproduce in the next 3 CI runs. A self-heal whose failure class still reproduces is marked `closed: false` and the next self-heal is escalated.
1.7. **No code in this spec.** Per operator directive ("we never go directly to code. SPECS AREN'T FINAL UNTIL I SAY"), this spec is the contract. Implementation beads are filed under §13 only after operator signoff.

## 2. The current self-heal — diagnosis

The current `auto-heal-audit.py`:

```python
def synthesize(failure: dict, audit_path: Path) -> dict:
    return {
        "ts": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "event": "self_heal",
        "subsystem": failure.get("subsystem", ""),
        "scope": failure.get("scope", ""),
        "category": failure.get("category", ""),
        "derived_from": failure.get("id", ""),
        "symptom": failure.get("symptom", ""),
        "root_cause": failure.get("root_cause", ""),
        "fix": failure.get("fix", ""),       # <-- always generic
        "guard": failure.get("guard", ""),     # <-- just an echo
        "test": failure.get("test", ""),       # <-- just an echo
        "linked_run": failure.get("linked_run", ""),
        "auto_generated": True,                # <-- passive
    }
```

**Why it fails the spec:**
- `fix` is always a generic instruction ("Patch code, then let GitHub CI run again"), never a system change.
- `guard` is just the failure's own `guard` field, echoed back. It does NOT add a new guard.
- `test` is just the failure's own `test` field, echoed back. It does NOT add a new test.
- `auto_generated: True` is a confession of passivity. The system says "I produced this row automatically, I did not do real work."

## 3. New self-heal schema (Spec 122 §3)

Every entry in `release-proof/audit/audit.jsonl` of `event: "self_heal"` MUST conform to:

```json
{
  "ts": "<ISO8601>",
  "event": "self_heal",
  "failure_class": "<class from classify-ci-failure.py>",
  "scope": "<which workflow / subsystem>",
  "derived_from": "<failure-id>",
  "fail_count_30d": <integer>,            // how many times this class failed in last 30 days
  "deliverable": {
    "type": "ci_gate" | "lint" | "retry" | "type" | "hook" | "test" | "doc" | "script",
    "ref": "<file path or PR URL>",
    "change_summary": "<one-line description>"
  },
  "before": {
    "manual_intervention_rate_pct": <number, baseline>,
    "failure_class_repro_count": <integer>
  },
  "after": {
    "manual_intervention_rate_pct": <number, measured at next 3 runs>,
    "failure_class_repro_count": <integer, 0 if closed>
  },
  "closed": <bool>,                       // true if after.failure_class_repro_count == 0 across 3 runs
  "linked_run": "<failure run-id>",
  "escalation_count": <integer>,          // 0 if first attempt, 1 if 2nd, etc.
  "operator_reviewed": <bool>            // true if human signed off on the deliverable
}
```

**Required fields per `deliverable.type`:**

| type | what the ref must be | what the change_summary must say |
|---|---|---|
| `ci_gate` | path to a `.github/workflows/*.yml` file that adds a new failing step BEFORE the step that produced the failure | "Added a failing gate at <step> that catches <failure_class>." |
| `lint` | path to a `clippy.toml` or `.cargo/config.toml` or a custom linter | "Added a lint for <failure_class>." |
| `retry` | path to a shared retry helper script in `scripts/` | "Replaced ad-hoc retry with `scripts/retry.sh`." |
| `type` | path to a Rust source file under `crates/focusa-core/src/types/` that adds a new variant or guard | "Added `<TypeName>` that prevents <failure_class>." |
| `hook` | path to a `.git/hooks/*` script or a pre-commit config | "Added a hook that runs <check>." |
| `test` | path to a new test file under `tests/` | "Added a test that fails when <failure_class> reproduces." |
| `doc` | path to a `docs/*.md` spec | "Added Spec <n> that defines <failure_class>." |
| `script` | path to a `scripts/*.sh` or `scripts/*.py` | "Added a script that <does the work>." |

## 4. Hard design laws

4.1. **Forbidden actions in a self-heal deliverable:**
   - ❌ Patching a test to make it pass
   - ❌ Skipping a CI step
   - ❌ Commenting out a failing assertion
   - ❌ Lowering a coverage threshold
   - ❌ Widening an `if` to swallow an error class
   - ❌ Adding `continue` to a loop to skip bad data
   - ❌ Lowering a clippy lint from `deny` to `warn`
   - ❌ Pinning a dependency to dodge an upstream bug
   - ❌ Patching the same failure class a second time (the system fix from the first heal should have caught it)
   - ❌ Touching the failing file without also adding a system fix from §4.2 in the same PR

4.2. **Required actions in a self-heal deliverable (at least one):**
   - ✅ **Patch the immediate error** (only when the failure class is first seen; never again)
   - ✅ Add a failing CI gate BEFORE the step that produced the failure
   - ✅ Add a clippy lint or rustc linter that catches the class
   - ✅ Add a retry helper that is reusable by other workflows
   - ✅ Add a runtime type in `crates/focusa-core/src/types/` that the system uses to guard against the failure class
   - ✅ Add a git hook that runs a check before commit/push
   - ✅ Add a test file under `tests/` that fails when the class reproduces
   - ✅ Add a doc spec that defines the failure class and the prevention strategy
   - ✅ Add a script under `scripts/` that performs the work and is referenced from the workflow
   - ✅ When the failure class is first seen, the same PR must contain BOTH an immediate patch AND one of the system fixes above.

4.3. **The deliverable must be in the same PR as the self-heal audit row.** A self-heal commit that says "I added a script" but does not include the script is INVALID.

## 5. Workflow changes

5.1. The cron `17 * * * *` (hourly) is reduced to `17 0 * * *` (daily at 00:17).
5.2. A new self-heal row is generated ONLY IF the failure class has appeared at least 3 times in the last 7 days (de-dup window). Below the threshold, the failure is logged but no self-heal is generated — the assumption is that one-off failures are caught by retries or operator attention.
5.3. Above the de-dup threshold, the self-heal workflow runs `scripts/propose-system-fix.py` instead of `auto-heal-audit.py`. The new script:
   - looks at the failure_class and the 7-day history
   - selects the most appropriate deliverable.type from §3
   - generates a concrete deliverable (CI gate, lint, hook, test, type, or script)
   - opens a PR with the deliverable
   - writes the self-heal row referencing the PR
5.4. If no deliverable can be generated automatically (e.g., the failure class is novel), the self-heal row is generated with `deliverable: null` and `closed: false`. This is escalated to operator review.
5.5. The "Commit audit updates" step in `audit-recorder.yml` only commits if a self-heal row with `deliverable != null` is generated. Passive self-heal rows do NOT commit.

## 6. Operator intervention rate — the metric

The whole point of self-heal is to reduce manual intervention. The metric:

```
operator_intervention_rate = (manual_interventions_required / total_CI_runs) × 100
```

**Baseline:** the current rate is roughly 100% (every clippy failure or rust compile error requires manual fix; the passive self-heal just records it). The baseline must be measured and recorded as the `before.manual_intervention_rate_pct` in the first self-heal of each class.

**Goal:** the long-run rate must trend toward 0% as the system-not-symptom gates accumulate. After 90 days of self-heal operating, the rate should be <5%.

**Cadence:** measured daily. Tracked as a new line in the audit ledger (`event: "intervention_rate"`).

## 7. The 4 current top failure classes — what to do in each heal

| Failure class | Count (30d) | Current behavior | Immediate fix (allowed ONCE per class) | System fix (Spec 122 deliverable) |
|---|---|---|---|---|
| `ci_clippy_failure` | 35 | manual fix | Run `cargo clippy --fix` on the failing files (one time). | `lint` deliverable: extend `clippy.toml` to deny the warning class as a hard error. The same warning is never patched in source again. |
| `unknown_process_failure` | 15 | passive row | Add a one-time logging statement to identify which process died. | `ci_gate` deliverable: add a step before the failing step that runs `process-health-check.py`. |
| `rust_compile_failure` | 11 | passive row | Fix the immediate compiler error (one time). | `type` deliverable: add a runtime type in `crates/focusa-core/src/types/` that the failing module would have used. **Never** add `#[allow(...)]`. |
| `transient_github_or_network_failure` | 6 | manual retry | Retry the failed step (one time). | `retry` deliverable: replace the ad-hoc retry block in the failing workflow with `scripts/retry.sh` that is shared across all workflows. |
| `ci_test_failure` | 6 | manual fix | Make the test pass for the current input (one time). | `test` deliverable: add a NEW test under `tests/` that fails when the failure class reproduces. The original failing test stays as-is. |
| `deploy_health_failure` | 6 | manual fix | Manually verify the deploy succeeded and health endpoint is up (one time). | `ci_gate` deliverable: add a pre-deploy gate that pings the health endpoint with a circuit breaker. |

## 8. The de-dup / escalation rules

- 1st occurrence: log only (no self-heal commit)
- 2nd occurrence within 7 days: log only
- 3rd occurrence within 7 days: self-heal fires; `escalation_count: 0`; `operator_reviewed: false`
- 4th occurrence within 7 days: self-heal fires again; `escalation_count: 1`; `operator_reviewed: false`; flagged in the next daily rate report
- 5th+ occurrence: `operator_reviewed` becomes REQUIRED. The self-heal cannot commit until a human signs off on the deliverable.

## 9. What the new self-heal PR looks like

```
[spec-122] System-not-symptom self-heal: ci_clippy_failure

What the failure was:
  35 occurrences of `ci_clippy_failure` in the last 7 days.
  Each one was a manual patch of a source file.

Immediate fix (allowed once for this class):
  Ran `cargo clippy --fix` on the files reported by CI.
  This unblocks the current run only.

What the system fix is:
  Extended `clippy.toml` to deny the warning class as a hard error.
  The same clippy warning will fail CI before it can reach the test suite.
  No source file is manually patched again for this warning class.

Closed: true (3 consecutive CI runs, no clippy failures of this class)
Deliverable: type=lint, ref=clippy.toml, change_summary="deny <warning-name>"

Operator review: NOT required (3rd occurrence, no escalation)
```

## 10. Migration path

10.1. **Today (before signoff):** the current self-heal behavior remains. No code change.
10.2. **Spec signoff:** operator signs off this spec. Implementation beads are filed under §13.
10.3. **Phase 1 (foundation):** rewrite `scripts/auto-heal-audit.py` in place so behavior is proactive: de-dup, deliverable-first synthesis, and failure_class-based action selection. Also write the de-dup logic.
10.4. **Phase 2 (top-class coverage):** produce the 6 deliverables from §7 (one per top failure class). Each is a separate PR that closes its class.
10.5. **Phase 3 (workflow change):** update `audit-recorder.yml` to use the new logic. The cron reduces to daily.
10.6. **Phase 4 (rate tracking):** add the `intervention_rate` event to the audit ledger. The metric is now measurable.
10.7. **Phase 5 (close the loop):** after 90 days, the rate should be <5%. If it isn't, the spec is re-opened.

## 11. What this spec is NOT

- **NOT a way to silence CI failures.** Self-heal never suppresses, never bypasses, never retries-into-success. It closes the failure class.
- **NOT a way to reduce the number of audit rows.** A healthy self-heal may write more rows than the current implementation (because each row has more fields), but the COMMIT count drops because passive rows don't commit.
- **NOT a replacement for the human.** Operator review is still required for novel classes. The goal is to reduce, not eliminate, human attention.
- **NOT a license to break things.** Every deliverable is verified by the next 3 CI runs before `closed: true`.

## 12. Open operator questions

1. Should the `escalation_count` threshold for `operator_reviewed: required` be 5 (as proposed) or 3 or 10?
2. Should the de-dup window be 7 days (as proposed) or 3 days or 14 days?
3. Should the cron reduction (hourly → daily) be paired with a "rebuild from git events" trigger that re-emits heals when a workflow_run fails more than 3 times in an hour? (This is the safety net for missing-daily-run scenarios.)
4. Should the rate-tracking event (`intervention_rate`) be visible in the menubar's RuntimeView (Spec 121 §3)?

## 13. Implementation beads (filed; IDs below)

These beads are tracked in `.beads/` and linked to the spec. Phase 1 is `in_progress`; Phase 2 deliverables are `open`; Phases 3-5 are `open`.

| Phase | Bead ID | Title | Status | Depends on |
|---|---|---|---|---|
| Phase 1 | `focusa-focusa-ssh-phase1-hjarh` | Rewrite `scripts/auto-heal-audit.py` proactive foundation (no rename) | `in_progress` | — |
| Phase 2 | `focusa-focusa-ssh-clippy-7e8e9` | Clippy lint deliverable | `open` | Phase 1 |
| Phase 2 | `focusa-focusa-ssh-process-9kb2o` | Unknown process CI gate deliverable | `open` | Phase 1 |
| Phase 2 | `focusa-focusa-ssh-rustcompile-0ky0t` | Rust compile type guard deliverable | `open` | Phase 1 |
| Phase 2 | `focusa-focusa-ssh-retry-aqi33` | Network retry helper deliverable | `open` | Phase 1 |
| Phase 2 | `focusa-focusa-ssh-test-of4jn` | Test failure regression test deliverable | `open` | Phase 1 |
| Phase 2 | `focusa-focusa-ssh-deploy-hxp3o` | Deploy health pre-deploy gate deliverable | `open` | Phase 1 |
| Phase 3 | `focusa-focusa-ssh-workflow-90hke` | Workflow changes (cron + commit conditions) | `open` | Phase 1 |
| Phase 4 | `focusa-focusa-ssh-rate-xxb9m` | Intervention rate tracking | `open` | Phase 3 |
| Phase 5 | `focusa-focusa-ssh-close-ld0yp` | 90-day close loop | `open` | Phase 4 |

**Decomposition rule:** Each bead is a system fix, not a symptom fix. Each deliverable is verified by the next 3 CI runs before the bead is closed.


## 14. Diff against prior drafts

- **v1 → v2:** added the "both initially, never twice" rule; split §4 and §7 into immediate fix + system fix framing; updated PR example.
- **v2 → v3:** added full implementation phase decomposition (Phases 1–5), real bead IDs in §13, and clarified that file rename is unnecessary (proactive behavior remains in `scripts/auto-heal-audit.py`).

The document is iterable — operator-revised versions appear as `122-...-v2.md` etc., with explicit diff sections at the bottom of each.

---

**Reminder:** Specs are NOT final until operator says so. This spec is the first of potentially many iterations. The 2026-07-07 commit-history lesson applies: rebase, don't overwrite.
