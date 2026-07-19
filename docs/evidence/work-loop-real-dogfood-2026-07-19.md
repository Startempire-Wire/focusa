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
