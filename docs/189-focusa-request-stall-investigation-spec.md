#189 — Focusa request-stall investigation and bounded repair

Status: isolated source defect reproduced and source correction verified; no installed correction or root-cause attribution to the customer is claimed.

## Owner, scope and retained work

The operator prioritizes fixing the reported macOS daemon freeze, delivering and actively testing the correction, then resuming core bug closure. Track the incident in GitHub#696 and `focusa-ak5sv.20`, beneath the existing Spec187 release epic. Preserve all19 Spec187 requirements, the existing f7f65a invocation-identity repair, client state and other worktrees. No locks/databases are deleted, no customer-machine mutation is inferred from email, and paid builds remain prohibited.

## Observations and hypothesis

The reported process stays alive while status/identity/discovery requests time out; a restart briefly answered200 before recurrence. Installed customer revision, process stacks and health-route behavior during the stall remain unknown. Current source executes synchronous workspace filesystem inspection directly inside async identity/discovery handlers. Historical issue12 addressed off-core persistence; it does not prove this incident's cause.

## Verified baseline and current correction

The unchanged f7f65a handler plus test registration failed the liveness regression as intended: one failed test, no passed tests, with `synchronous project inspection starved an unrelated HTTP request`. The test itself took1.01seconds; the323.85-second receipt includes compilation. Evidence: `ovh:/home/wirebot/build/focusa/189-stall-control/1731-baseline/receipt.json`.

The scoped correction moves list/current, discovery, identity and verification inspection into the existing Tokio blocking-worker facility while keeping their successful payload construction unchanged. A shared local response adapter uses the canonical `ToolResultV1` failure owner and never claims verified/canonical identity when a worker fails. Regression coverage includes discovery/identity/list responsiveness, unchanged unverified payloads and worker-failure handling. This does not establish the cause on the affected Mac.

The operator's latest direction fixes the next release batch at seven priority issues:696,685,680,660,695,691 and686. This supersedes the earlier2–3 additional-fix target. Required installed acceptance remains a prerequisite to truthful issue closure; no new feature program is added to the batch.

The expanded identity fixture exposed repeated reads of the same marker. The owned1732 test unit was stopped after preserving its partial receipt and logs; this was a fixture stall, not a passing quality run. The corrected fixture atomically replaces only its synthetic marker path with a regular file after the first descriptor opens, so later reads remain finite. The corrected run verified an inline-execution negative control still detects blocked HTTP progress.

Quality receipt `ovh:/home/wirebot/build/focusa/189-stall-control/1734-quality/receipt.json` records43 project tests passed, zero failed, formatting and strict clippy passed, and hashes of the verified source. The43 include six new liveness/payload/failure regressions; total quality elapsed304.301seconds includes compilation. The formatted verified files were synchronized back only after checking local input hashes, preserving unrelated changes. Full release and affected-client verification remain pending.

## Bounded execution and evidence

1. Trace actual request handlers and reuse their canonical inspection/error owners.
2. Add a producer regression in `crates/focusa-api/src/routes/project_liveness_tests.rs`, referenced by `project.rs`. A synthetic named-pipe marker delays a read for one second; an independent thread guarantees release. Verify an unrelated HTTP request can execute while inspection remains blocked. Use no production data, signed authority or live daemon.
3. Run the unchanged handler plus regression on OVH as an expected failure: `cargo test --locked --offline -p focusa-api --bin focusa-daemon slow_workspace_metadata -- --nocapture --test-threads=1`. Keep execution bounded by a managed unit, at most two compile jobs and6GiB memory. Preserve terminal exit status and logs.
4. Only after expected failure, apply the smallest correction in the existing project-inspection owner and repeat the regression. Preserve existing JSON, identity/provenance, selection and admission semantics. Add identity/verification coverage if source tracing establishes the same affected call stack.
5. Run affected formatting, lint, producer/consumer and compatibility checks, followed by the required full release gates for the changed candidate. The earlier strict preflight remains evidence for f7f65a only.
6. Obtain safe customer revision/process diagnostics; delivery and actual installed macOS consumer proof are required before closing#696. Reproduce the reported workflow and preserve restart/lock ownership behavior.

## Stop, rollback and delivery boundaries

A synthetic regression establishes a source defect, not automatically the customer's cause. Refine the diagnosis when client evidence disagrees. Roll back only the new scoped source patch through ordinary reviewed history; preserve existing work and all forensic data. Use the canonical complete release/install path and approved destination; do not create a partial release, trigger paid providers or bypass daemon admission. Reuse the previously owner-approved isolated OVH source-test recovery route while the native dispatch defect remains unresolved; report it rather than presenting that exception as normal operation.
