# Spec146 — Focusa Canonical Release Cycle Operations, OTA, Benchmark, and Proof Runbook

**Status:** normative companion to Spec145  
**Authority:** GitHub #56 / Bead `focusa-vbcqu.6.1`

## 10. Automatic OTA architecture

### 10.1 Managed surfaces

- CLI
- daemon
- TUI
- Pi extension
- installer
- menubar when installed
- agent-context bundle through installer integration

### 10.2 Trust chain

```text
GitHub release
-> complete expected asset set
-> SHA256SUMS
-> detached signatures
-> signed release manifest
-> provenance
-> CI proof
-> deploy proof where required
-> local policy authority
-> stage/download/hash verify
-> rollback snapshot
-> atomic promote
-> restart/activation proof
-> history receipt
```

### 10.3 Pi extension ownership

Being inside any parent Git repository does not make a package externally
managed. `path_is_git_managed` must prove the target package is tracked by
`git ls-files`. An untracked installed package remains OTA-managed. A tracked
source checkout is notify-only.

### 10.4 Systemd scheduler

The scheduler unit:

- runs the canonical CLI with `--automatic`;
- captures the installing environment PATH needed for package activation;
- checks every 120 seconds with 20% jitter on trusted developer hosts;
- applies exponential backoff after network/provider failures;
- remains enabled and persistent across reboot;
- reports failed apply as nonzero even after successful rollback.

### 10.5 Data safety

OTA never overwrites license, environment, projects, state, evidence, logs,
permissions, ownership, xattrs, or capabilities. Daemon promotion occurs last.
A failed later package promotion restores all previously promoted parts from
the same transaction journal.

## 11. Fast-release controls

### 11.1 Eliminate duplicate work

- Reuse exact-SHA PR/main evidence.
- Cache target/toolchain dependency builds.
- Trigger Release only for tags.
- Cancel stale PR CI/Spec132 runs while preserving locked-candidate `main` gates.
- Run independent target builds in parallel.
- Revalidate only stages invalidated by a bounded fix.

### 11.2 Preserve truth while optimizing

No optimization may:

- skip a required surface;
- reduce the supported target matrix;
- convert a failure to warning;
- publish mutable tags;
- reuse evidence across SHAs/input digests;
- conceal rollback or stale installed/running versions.

### 11.3 No-progress escalation

Escalate when:

- a stage exceeds calibrated p95;
- no evidence changes during two temporal pulses;
- the same gate fails twice for the same cause;
- queue time dominates useful work;
- retry/coordination cost exceeds expected parallel benefit;
- release remains draft while installed/running truth is stale.

## 12. Baseline benchmark: v0.9.127-dev

Measured 2026-07-26/27 from GitHub receipts:

| Run | Elapsed | Dominant stage |
|---|---:|---|
| CI `30229564480` | 613 s | strict spec gates 608 s |
| Spec132 `30229565493` | 888 s | Windows release target 700 s |
| Release `30229565463` | 1021 s | Apple arm64 binary 523 s |
| Deploy `30230221089` | 42 s | live daemon job 37 s |

Approximate tag-to-live critical path: 17–18 minutes. First-pass release gates
were green after the #72 repair. Manual interventions before this candidate
exposed missing path ownership, duplicated build work, and stale updater
failure semantics.

Initial improvement hypothesis:

- tag-only Release removes duplicate no-op workflow runs;
- stale PR-run cancellation reduces queue waste without cancelling locked candidate gates;
- Rust cache reuse reduces repeated target compilation;
- complete Spec132 paths move target failures before tags;
- exact-SHA evidence reuse can remove duplicate full workspace tests after a
  dedicated evaluation proves equivalent or stronger coverage.

The final release must record actual deltas; no speed claim is accepted without
measured improvement.

### 12.1 First canonical-cycle outcome: v0.9.128/v0.9.129

`v0.9.128-dev` proved the pre-tag source gates and immutable candidate lock but
failed the pull-request inclusion gate because unrelated PR #73 was treated as
a release blocker. The bounded fix lane preserved the failed tag, changed only
PR-queue and duplicate-trigger authority, reran invalidated gates, and required
a new candidate.

`v0.9.129-dev` completed the cycle:

| Run | Elapsed | Delta from v0.9.127 |
|---|---:|---:|
| CI `30239380676` | 596 s | -17 s (-2.8%) |
| Spec132 `30239380662` | 742 s | -146 s (-16.4%) |
| Release `30239381932` | 1044 s | +23 s (+2.3%) |
| Deploy `30240189485` | 38 s | -4 s (-9.5%) |

Tag-workflow start to live deployment was 1082 seconds. End-to-end human-clock
speed was effectively unchanged/slightly worse than the 17–18 minute baseline,
so the first architecture slice does **not** claim overall speed improvement.
It did improve cross-target preflight and deployment duration, eliminate the
duplicate Spec132 tag run, move CI/Spec132 before immutable tagging, and produce
a durable exact-SHA candidate-lock artifact.

Next critical-path optimization:

- Release remains the bottleneck at 1044 seconds.
- Release surfaces are stamped into an untagged candidate commit, pushed to
  `main`, and exact candidate-SHA CI/conditional Spec132 must pass before the
  immutable tag is created.
- Release revalidates those exact candidate receipts; full Rust tests/clippy are
  removed from the serial Release gate.
- The independent tag-CI publication gate queries exact-SHA workflow receipts;
  checksum publication waits for its successful result.
- Cache hit/miss telemetry must be captured per target to distinguish cold-cache
  candidate cost from steady-state speed.
- The next candidate must measure whether concurrent artifact builds plus the
  publication gate reduce overall time; rollback/failure truth remains unchanged.

Proof refs:

- release `v0.9.129-dev` at `b97ad16c652e497c5190ebaceea55d875d695ec9`;
- candidate artifact `release-candidate-v0.9.129-dev` from run `30239381932`;
- candidate topology digest `83f1c40b9dbd24304752033ef428ad00b07f5cbb61774ae0aabfd7537151f8fc`;
- CI `30239380676`;
- Spec132 `30239380662`;
- Release `30239381932`;
- Deploy `30240189485`;
- live and all managed surfaces `0.9.129-dev`;
- automatic updater enabled/active and final systemd no-op cycle successful.

### 12.3 Final Focusa proof: `v0.9.134-dev`

The corrected exact-candidate cycle produced these immutable receipts:

- exact candidate CI `30255479290`: 588 seconds;
- exact candidate Spec132 `30257573965`: 720 seconds;
- Release `30258399825`: 533 seconds;
- Deploy `30258961394`: 40 seconds;
- tag-workflow start to live completion: 571 seconds, down from 1082 seconds
  for `v0.9.129-dev` (**47.2% faster**);
- live daemon, CLI, TUI, Pi extension, and installer: `0.9.134-dev`;
- refreshed root scheduler unit contains explicit `/usr/local` CLI/TUI/daemon
  targets and completed a successful automatic no-op cycle.

The proof also exercised fail-closed recovery. Audit Recorder moved `main` after
an earlier candidate push; PR-only CI cancellation preserved later locked-main
runs. A recovery retry then found required Spec132 coverage without an exact-SHA
run. The helper now preserves an already-stamped retry SHA and dispatches a
missing Spec132 run only while remote `main` still equals that candidate; a
scope mismatch fails before tagging.

### 12.4 Cross-software master-cycle conformance

Reference contracts:

- `config/release-adapters/focusa.json`;
- `config/release-adapters/cli-library.json`;
- `config/release-adapters/uiai-engine.json`;
- `config/release-adapters/single-package.json`;
- `config/release-adapters/service-container-web.json`;
- matching topologies under `config/release-topologies/` plus Focusa's canonical
  `config/focusa-release-topology.json`.

Proof commands:

```bash
cargo test -p focusa-core release_orchestrator --lib
cargo test -p focusa-core release_adapters --lib
cargo test -p focusa-core release_calibration --lib
cargo test -p focusa-core release_ledger --lib
python3 tests/spec145_canonical_release_cycle_static_test.py
focusa release cycle validate-adapter \
  --manifest config/release-adapters/uiai-engine.json \
  --topology config/release-topologies/uiai-engine.json
```

The conformance suite executes all four profiles through one kernel, verifies
Canvas/terminal/headless plan parity, exercises an external JSON plugin process,
and proves that a later calibration changes topology-wave concurrency. This is
architecture proof, not a claim that UIAI production was deployed; a real UIAI
release remains a separate project-authority operation using this adapter.

## 13. Security and supply chain

- Least-privilege workflow permissions per job.
- Environment protection for production deploy.
- OIDC or short-lived provider credentials where supported.
- No secret in logs, release pages, manifests, or benchmark packets.
- Pin or policy-govern third-party Actions.
- Verify archive path safety before extraction.
- Resolve package runtime dependencies from explicit scheduler environment.
- Preserve append-only audit and rollback receipts.
- Require independent verification for provenance and production promotion.

## 14. Compatibility and migration

### 14.1 Current Focusa workflows

Existing CI/Release/Deploy workflows remain the first adapter. Changes are
incremental: trigger ownership, concurrency, cache, evidence reuse, and typed
kernel validation. No flag-day provider replacement.

### 14.2 Existing OTA hosts

Reinstalling `focusa update scheduler --install` refreshes the unit and runtime
PATH. Existing policy files remain valid. `dev_mode_override=true` explicitly
authorizes unattended dev-channel updates on trusted developer hosts.

On multi-user Linux hosts, `/usr/local/bin/focusa` and `focusa-tui` remain real
world-executable files. They never symlink into `/root` or another private home.
The root scheduler resolves and atomically updates those global paths; per-user
installs remain under that user's own install root.

### 14.3 Existing Pi extensions

Tracked source checkout: notify-only. Untracked installed package: verified
atomic OTA. Activation receipt records target version and restart/reload need.

### 14.4 Failed historical releases

Historical tags and failures remain immutable. Later candidates reference the
failure and superseding proof; they do not rewrite history.

## 15. Detailed acceptance

A release architecture slice passes only when:

1. topology validation rejects duplicate, unknown, self, and cyclic edges;
2. candidate transitions reject skips and wrong-SHA evidence;
3. release lock blocks unrelated work;
4. blocker fixes are bounded and invalidate prior evidence explicitly;
5. Spec132 triggers for every runner/updater ownership path;
6. Windows/macOS/Linux targets are green before promotion;
7. Release workflow triggers only on immutable tags;
8. all intended assets have checksums/signatures/provenance;
9. deploy consumes release artifacts and emits health/version proof;
10. every installed/running surface matches the candidate;
11. stale untracked Pi packages update automatically;
12. tracked Pi source checkouts remain notify-only;
13. updater rollback returns a nonzero process status;
14. scheduler has the runtime PATH required for package activation;
15. timer is enabled, active, persistent, and policy-authorized;
16. benchmark packet names critical path and measured delta;
17. release page contains exact evidence, limitations, install, upgrade, and rollback;
18. final state is CLOSED, ROLLED_BACK, or CANCELLED with receipt;
19. issue/Bead/Workpoint/Trajectory state reconciles;
20. no unrelated feature entered the locked candidate.

## 16. Proof commands

```bash
cargo test -p focusa-core release_cycle
cargo test -p focusa-cli commands::update
cargo clippy -p focusa-core -p focusa-cli -- -D warnings
bash tests/spec128_update_status_static_test.sh
bash tests/release_deploy_automation_static_test.sh
python3 tests/spec143_ota_installability_release_gate_test.py

gh pr checks <pr>
gh run view <ci-run>
gh run view <spec132-run>
gh run view <release-run>
gh run view <deploy-run>
gh release view <tag>

focusa --json update plan
focusa --json update policy show
focusa --json update scheduler
systemctl show focusa-update.service -p Result,ExecMainStatus,Environment
systemctl is-enabled focusa-update.timer
curl -fsS http://127.0.0.1:8787/v1/health
```

## 17. Rollback

- Source/workflow defect before tag: fix branch, rerun invalidated gates.
- Published candidate defect: new immutable candidate; never retag.
- Binary promotion defect: atomic backup restore and health proof.
- Pi extension defect: restore package directory and activation receipt.
- Scheduler defect: restore prior unit/drop-in, daemon-reload, verify timer.
- Architecture defect: retain event/evidence history, disable only the faulty
  adapter capability, and preserve the core candidate record.

## 18. Completion boundary

Spec145 is complete only when one final immutable Focusa release executes this
cycle, improves or truthfully explains the v0.9.127 baseline, automatically
updates all managed server surfaces, survives restart, emits exact proof, and
closes or evidence-supersedes #55/#56 and their Beads.
