# Real Work Loop Dogfood — 2026-07-19

## Scope

- Branch runtime: `local/work-loop-completion`
- Real project: `/Volumes/Macintosh HD/Users/vsmith/focusa-workloop`
- Real provider data: `.beads/issues.jsonl`
- Real transport: installed Pi RPC with `openai-codex/gpt-5.6-sol`
- Isolation: branch daemon used a temporary data directory and port `127.0.0.1:18787`; no live daemon, deployment, or release mutation
- Root WorkItem: `focusa-a6yq6`
- Budget: 3 turns / 15 minutes / 1 retry / destructive actions disabled

## Verdict

**FAIL — not yet functional enough to dogfood the remaining implementation.**

## Proven working

1. Branch daemon health and scoped Workpoint creation.
2. Provider-neutral root binding, writer fencing, and renewable budget status.
3. Real BD graph enumeration after fixes.
4. Nested descendant-leaf selection chose the correct real leaf `focusa-a6yq6.3.2`.
5. Real Pi RPC process attached to the exact WorkItem/Workpoint partition.
6. Work Loop autonomously dispatched the initial prompt.
7. Process-tree cleanup left no branch daemon or Pi RPC child.

## Defects found and fixed

1. Unbounded database-mode `bd show` froze daemon health and scheduling.
   - Fixed by bounded `tokio` process execution, `kill_on_drop`, and JSONL-first `--no-db`.
   - Commit: `d42edb6`.
2. Global provider limit hid all Spec133 children and N+1 `show` was too slow.
   - Fixed with one complete provider snapshot parsed directly.
   - Commit: `6a0d0e1`.
3. Exact-parent scheduling could not reach nested runnable leaves.
   - Fixed with provider-neutral descendant traversal and leaf-before-parent-gate ordering.
   - Commit: `a11bb21`.

## Remaining blockers observed in the real run

1. Selected `focusa-a6yq6.3.2` had `linked_spec_refs: []`; governance rejected the turn as missing authoritative spec grounding.
2. Spec133 sibling leaves lack execution-order dependencies. After deferral, the loop moved through `.3.3`, `.3.4`, and `.3.5` instead of resolving `.3.2`.
3. Repeated planning-only turns exhausted low-productivity/retry budgets and paused without source changes or completion proof.

No tracked source files were modified by the failed Pi run.

## Follow-up beads

- `focusa-workloop-completion.9.2` — authoritative spec/acceptance packet grounding
- `focusa-workloop-completion.9.3` — normative dependency order and blocker-churn prevention
- `focusa-workloop-completion.9.4` — repeat real dogfood and require one material evidence-closed task

## Final bounded rerun verdict

**PASS — the real Work Loop materially implemented, tested, committed, and evidence-closed real Spec133 work.**

The rerun used the same real BD graph, root `focusa-a6yq6`, branch daemon, and installed Pi RPC/model. No fake provider, fake graph, or fake Pi was introduced. The oversized `.3.3` leaf was decomposed into ordered §23 child beads after runtime evidence showed that lifecycle, observation/SSE, input, and config were independently verifiable slices.

### Autonomous implementation proof

The supervised Pi produced these real source commits while remaining under the Spec133 root:

- `a3710e1` — lifecycle API surface
- `d86c62f` — exact run generation persistence
- `4fc09f0` — exact event resume cursor
- `45c9157` — §23 route registration and tests
- `3085ad1` — resumed-event generation fencing
- `82f86c6` — durable run status
- `f9b39e2` — durable observation projections

Focused verification passed: 9/9 `routes::silent_sessions` API tests and `cargo check -p focusa-api`. Provider-neutral scoped closure reconciled:

- `focusa-a6yq6.3.3.1` via claim `claim_bd_019f7b63a70e7363`
- `focusa-a6yq6.3.3.2` via claim `claim_bd_019f7b63ace37430`

The provider reports both child beads `closed`. Remaining `.3.3` children preserve explicit lifecycle-mutation, interactive-input, and transactional-config gaps; the parent remains open and cannot be falsely promoted.

### Dogfood defects fixed during the rerun

1. Child Pi now inherits the owning branch daemon endpoint and exact cwd.
2. Headless Pi uses warn-only vital information and documented extension UI responses.
3. Supervised Pi disables extension, skill, and prompt-template discovery so daemon governance remains the sole orchestration authority.
4. Post-turn root selection no longer falls back to global ready work.
5. Outcome governance waits for `agent_end`, not intermediate tool `turn_end` events.
6. Productive `Continue` cycles no longer consume retry/failure budgets or trigger completion-only evidence gates.

The branch daemon, heartbeat, and Pi process tree were terminated after proof collection. No live daemon, deployment, push, merge, or release was mutated.

## Continued real-task completion

After the dogfood gate, the same governed Work Loop continued onto `focusa-a6yq6.3.3.3` and implemented the complete lifecycle mutation slice across bounded runs. Autonomous commits include lifecycle CAS/schema v5, pause/resume/cancel/start/adopt/interrupt/restart, superseded-run fencing, and governed preflight/create (`0853dc3`, `0e10e03`, `fc5354f`, `d4223fd`, `c664c5c`, `8e4c92f`, `4828c4b`, `92089b9`, `f62b1aa`).

Independent proof passed:

- 10/10 `routes::silent_sessions` API tests
- 14/14 SQLite persistence tests
- `cargo check -p focusa-api`
- 375 core tests and strict core clippy for the Work Loop continuation fixes

Scoped closure claim `claim_bd_019f7bdd03d67312` reconciled `focusa-a6yq6.3.3.3` to provider status `closed`.

The Work Loop then implemented exact-run interactive APIs in `ba4946d`. Independent proof passed 11/11 focused API tests and `cargo check -p focusa-api`; scoped claim `claim_bd_019f7bfd665f7791` reconciled `focusa-a6yq6.3.3.4`.

The final config child was implemented in `72568ed`, `660fccd`, `9c861d7`, and `6099608`. Proof passed 12/12 API tests, 4/4 config tests, 14/14 persistence tests, and API check. Claim `claim_bd_019f7c3920487540` closed `.3.3.5`; aggregate claim `claim_bd_019f7c3926de7240` closed the complete `.3.3` §23 API tranche.

Capability, harness, provider/model catalog and preflight APIs landed in `fb10ad68`. Thirteen focused API tests prove freshness/auth/entitlement metadata and truthful unknown/degraded semantics; claim `claim_bd_019f7c72435e7de0` reconciled `.3.4`.
