# Release-session failure-mode catalog (2026-08-16/17)

For every future agent touching the focusa release line. Each entry = a mode we hit, the cost, and the rule that prevents it.

## A. Squash / branch-interleave modes (the big ones)

1. **Never big-squash a branch hundreds of commits behind main.** Mixing two lineages file-by-file produces an unbounded error-wave grind (we burned ~4h). RULE: main-wins ALL shared files; re-apply only (a) additive new files, (b) surgical known patches. Keep a written list of the surgical patches FIRST.
2. **`git add -A` after a squash stages main-only files as deletions.** RULE: after any squash, `git diff --name-status HEAD~1 HEAD | grep ^D` must be empty — restore every D path from origin/main.
3. **Conflict markers survive into committed files.** We shipped markers in Cargo.lock, docs/INDEX.md, types.rs. RULE: `grep -rl '<<<<<<<' --include=...` across the tree before the squash commit; gate on zero.
4. **The push clone needs `git reset --hard origin/<br>` + `git checkout FETCH_HEAD -- <file>`.** `git checkout <local-branch> -- file` silently no-ops after a fetch. RULE: always reset hard to origin, then use FETCH_HEAD.

## B. OVH build-host modes

5. **The focusa-ovh-build wrapper re-syncs FOCUSA_SOURCE_ROOT (default `/home/wirebot/focusa`) to the mirror on EVERY cargo invocation.** Gates dispatched from a worktree still built the SESSION tree (caused the 404 saga). RULE: for worktree gates, run `ssh focusa-build-ovh /tmp/gate-script.sh` directly with the right source, or set FOCUSA_SOURCE_ROOT; never trust cwd.
6. **Manual rsyncs race the wrapper's own sync** → mixed-revision proof + mid-merge Cargo.lock on the mirror. RULE: one sync path only; no manual rsync while a wrapper job runs.
7. **`cargo check` produces NO binaries.** A green check ≠ a fresh daemon binary; tests spawn the daemon from PATH/`CARGO_TARGET_DIR` and can hit stale binaries → 404 for new routes. RULE: `cargo build -p focusa-api --bin focusa-daemon` before e2e tests.
8. **A stale long-running daemon holds :8787 on OVH**; new-daemon probes and CLI-spawned tests hit the OLD process → 404. RULE: `pkill -f focusa-daemon` before test/probe runs.
9. **FOCUSA_TEST_MODE=1 is required** for API tests — without it the entitlement middleware 403s (ENTITLEMENT_IDEMPOTENCY_REQUIRED). The ovh-test-runner sets it; direct ssh scripts must export it.
10. **bg jobs inject FOCUSA_OVH_BUILD** — every cargo command inside a bg job runs remotely. "Local" checks aren't local. Also: the wrapper intercepts `npx tsc` too (use ./node_modules/.bin/tsc).
11. **bg false-greens**: a gate chain whose last command echoes 0 reports success. RULE: `set -e; set -o pipefail` + explicit `echo X=FAIL; exit N` markers.
12. **No sleep-poll chains (TBQ).** `sleep N; grep` repeatedly = banned; act on bg completion notifications. (We violated this repeatedly under pressure — it wastes real time.)

## C. Rust interleave modes

13. **Generic regex loops over match arms/initializers corrupt code** (inserted `workstream: _,` into fn params → syntax errors). RULE: patch at exact anchors only; verify count==1 before replacing.
14. **"cannot find attribute `serde`" at a `#[serde(...)]` attr = the struct/enum LACKS the derive**, not a missing import. RULE: every inserted type gets its full derive line (Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize) in one edit with the struct.
15. **TrajectoryMilestoneStatus variants are NotStarted/Active/Blocked/Verified/Superseded** (session contract) — do not invent variants.
16. **main's TrajectoryProjectionRecord has MANUAL Deserialize + Default impls** — adding derives conflicts (E0119); patch the manual impls instead.
17. **Adding enum variants requires arms in EVERY exhaustive match** (E0004), including inside types.rs itself.
18. **New core modules must be `pub mod` in lib.rs** (api crate consumes them) — and check for accidental `pub pub` (the double-pub from a prior edit).
19. **json_guard validates any key named `scope_kind` against ONE vocabulary** — it must accept BOTH the typed ScopeKind enum ("Project"/"Host"/lowercase) and query kinds. And the ScopeKind WIRE format is lowercase `"project"`.
20. **workset_digest returns String (no unwrap); `(completed - *started)` needs BOTH refs deref'd in the bg ETA path; steer_targets_item must match parent/child refs both directions.**
21. **Workspace-member crates that aren't members are invisible to every gate** (the letta line existed for weeks without check/test/clippy). RULE: any new crate dir → add to Cargo.toml members + [workspace.dependencies] entries INSIDE the table (orphaned top-level entries are silently ignored).

## D. Release-cycle / gate modes

22. **Commit types are a closed set** (feat|fix|docs|test|refactor|perf|build|ci|chore|revert|proof|merge). `release(...)` fails the hook; use `chore(release):`.
23. **The spec145 static gate validates the WHOLE canonical release surface**: release.yml must declare `on.push.tags` with GLOB patterns (fnmatch — `v[0-9]*.[0-9]*.[0-9]*`; `[0-9]+` is regex syntax and fails), update.rs OTA truth strings, CI concurrency + rust-cache, release-candidate locking strings. Pre-run `python3 tests/spec145_canonical_release_cycle_static_test.py` locally before the tag.
24. **The release-learning guard includes cross-server disk pressure** — the OVH host must be <90% / >15GB free. Clean build caches first (target/, exact-*, audit-*, cal-diy/tmp).
25. **Spec 152 doc gate** forbids legacy self-issued `--eval` commands + requires Spec 152/authority-issued/recovery concepts in every operator guide.
26. **The error-envelope middleware wraps non-JSON 4xx/5xx** — when a route returns "blocked" with a generic message, the handler's typed reason was masked or the guard rejected earlier. Check the json_guard + the handler's blocked() before assuming the route logic.

## E. Canonical cycle discipline (the meta-lesson)

27. **Use the canonical release cycle; do not re-invent gates.** `scripts/create-dev-release-tag.sh --push` is the pipeline; GitHub CI (with correct env) is the authority for the full test chain. Pre-run only the FAST local pre-gates (spec145, spec152, tag-trigger, learning-guard), fix them ONCE, then dispatch the tag and act on CI results. Manual full-gate loops on OVH are the failure mode this whole catalog exists to prevent.
28. **Time-box the release path to <1h**: squash (policy: main-wins + additive + surgical list) → local pre-gates → tag dispatch → CI → deploy. Anything longer means one of the modes above is active — stop and re-strategize at the forest level, not the tree level.

29. **Local `npm`/`npx`/`tsc` on PATH are OVH-wrapper stubs** — `npm install` "succeeds" but creates no node_modules; `npx tsc` prints "not the tsc you are looking for". RULE: use `/opt/cpanel/ea-nodejs20/bin/npm` and `node_modules/.bin/*` directly for local JS work.
