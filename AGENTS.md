# Agent Instructions

## Agent-KB API Default Reference

For KH/OVH/operator policy, inherit `/root/AGENTS.md`: query `agent-kb-api` first, verify freshness, use exact document lookup after empty searches, and treat local Agent KB files as read-only fallback.

## Focusa agent docs entry point

Before broad Focusa code changes or after context loss, read `docs/agent/01-focusa-agent-docs-index.md`. It is the bounded, public-safe architecture/commands/API/Workpoint/Trajectory/private-boundary guide for agents.

## Current agent-readiness fast path

1. Verify `project_root + continuity_id` with `focusa_project_identity`/`focusa_project_verify`; a Git worktree is a typed working subpath under that authority.
2. Resume Trajectory and the canonical Workpoint before acting; transcript tails, cached aliases, and predictions do not grant authority.
3. Discover capabilities progressively: `focusa_agent_card` → `focusa_tool_search` → `focusa_tool_describe`/`focusa_tool_graph`.
4. All Focusa Pi tools must remain one-to-one across runtime registration, `docs/contracts/spec141/generated-capability-v2/pi-tools.json`, capability descriptors, and `docs/focusa-tools/tools/`.
5. Load the matching `.pi/skills/<name>/SKILL.md`, then its numbered runbook only when the workflow requires detail. Packaged copies live under `apps/pi-extension/skills/` and must be byte-identical.
6. For durable background execution, use daemon-native Silent Sessions with exact session/run/generation and approval/idempotency fields—never raw tmux or shell aliases.
7. For context pressure, preserve canonical Workpoint/Trajectory state and governed auto-rollover; do not treat transcript compaction as authority.
8. Customer lifecycle changes must prove install or repair/rerun, trusted OTA/update rollback, and uninstall with user data preserved unless purge is explicit.

Current surfaces: Mission Canvas/Work Rail and generated UI (`docs/135-series-current-manifest.md`), Silent Sessions (`docs/133-silent-sessions-final-release-proof.md`), all-tool/skill machine contracts (`docs/contracts/spec141/generated-capability-v2/`), and public onboarding (`README.md`, `docs/current/FOCUSA_FRIENDLY_ONBOARDING.md`).

## Terminal-blocking queries (TBQs) must run asynchronously (mandatory)

The operator terminal must never stop flowing. Any terminal-blocking query —
builds, test suites, migrations, long scans, waits for remote jobs — MUST be
dispatched through the canonical background-execution surface, and the agent
must continue other work immediately. Blocking is allowed only for
sub-second commands and commands with an explicit short bound whose output
is required immediately.

CANONICAL DISPATCH — `focusa bg` is the ONLY background-execution
mechanism. No raw shells, no alternatives:

```bash
setsid nohup /usr/local/bin/focusa bg run --name <job> -- <command...> &
```

(the setsid/nohup above only detaches the bg monitor itself from the
terminal; the JOB runs through focusa bg.)

- `focusa bg run` creates the durable job row, executes detached,
  streams output to the job log, records completion durably, then
  broadcasts `background_job_completion` with the bounded output_tail
  on the daemon SSE stream.
- The Pi extension delivers completion + output tail INTO the agent's
  front terminal (notify + appendEntry). `focusa bg wait --job <id>`
  long-polls for harnesses without SSE. `bg status`/`bg list` are the
  only status queries.
- BANNED: raw `setsid nohup ... > log &` job dispatch, `tail` polling,
  `sleep N; tail` chains, and treating the envelope as advisory.
  Completion + output_tail IS the delivery path (docs/165).
- Multi-agent work = N workloop-bound SILENT SESSIONS with the existing
  completion stream + bg receipts (docs/168) — never raw shells.
- Fast-forward multiplier (2x/4x/6x/8x…): operator-conceived #312 —
  FanoutPlan divides work items round-robin across parallel sessions;
  per-lane policy budgets, wait-for-all join (docs/169).

## Production consistency (mandatory default for every feature)

Every Focusa feature ships only when all five proofs exist: versioned
contract, producer tests, CONSUMER-side tests (producer-green is not
delivery-green), cross-version interop, and the live e2e proof across
supported environments. Policy:
docs/current/PRODUCTION_CONSISTENCY_POLICY.md. The bg-notification
feature is the reference implementation of the policy.

## Disk headroom (mandatory)

Never allow the operator filesystem or user quota to reach capacity.
Always remove safe removables **first and proactively**: build caches
(`target/`, `node_modules/`), toolchain caches, age-bounded rollback
backups, staging clones, and temp artifacts. Check `df` and user quota
before and after large operations; when headroom drops, free rebuildable
space immediately — never reactively under pressure. Live data (daemon
databases, evidence, ledgers, user files) is never a removable.

## De-duplication discipline (deslop, mandatory)

Before writing a new helper, envelope block, or test setup, check the
deslop analysis for an existing similar implementation (`deslop` CLI in
CI reports; the Deslop MCP `find-similar` when connected). Renamed
copies of existing helpers are rejected in review; converge intentional
boilerplate (error envelopes, tool results) through the canonical
constructors (focusa_core::error_envelope, tool_result_v1) instead of
re-typing them. The duplication ceiling lives in `.deslop.toml`.

## Pre-work rule: always check remote first (mandatory)

Before any durable state change (commit, push, branch switch, merge, rebase, tag), or before resuming work after a session reload, agent context drift, or gap in continuity:

1. `git fetch origin` to discover remote commits you do not yet have locally.
2. `git status` to see local uncommitted work and any rebase-incompatibility risk.
3. If you have unstaged changes and the remote has moved, **stash first**, then `git pull --rebase`, then `git stash pop`. Resolve any conflicts before continuing.
4. Only then proceed to the canonical build/deploy chain below.

Why: shipping from a stale local head duplicates or reverts remote work, and creates
phantom commits in the operator's log. The discipline is: **see the world before you change it.**

## Release vocabulary (mandatory, plain language)

- **Release** = full canonical stable. All surfaces, all operating systems, all artifacts. Shows as **Latest** on the repo sidebar and makes the green CI badge. This is the only thing that counts as "shipped."
- **Dev release** = nightly/development channel. Same full surfaces and operating systems, but marked prerelease. Early adopters can opt in. It is still full — no missing OS, no missing surface.
- **No partial releases.** Do not ship an OS or a surface by itself. If you think you need one, write a one-line reason and get explicit approval. Default is no.

## Canonical build/deploy rule (mandatory)

**Build and deploy ONLY through the full live GitHub release pipeline.**

- Canonical command: `scripts/create-dev-release-tag.sh --push` (for stable) or `scripts/create-dev-release-tag.sh --base 0.9 --push` (auto-picks next patch). For an exact stable: `scripts/create-dev-release-tag.sh --tag v0.9.177 --push`.
- Required chain: `CI` → `Release` → `Deploy Live Daemon` → audit/self-heal/watchdog. The release is not done until `gh release view vX.Y.Z` exists and CI is green.
- Do **not** build release artifacts locally with `cargo build --release`.
- Do **not** deploy from `target/release` or call `install-daemon.sh --binary target/release/...`.
- Do **not** run only a partial deploy workflow as a shortcut.
- Do **not** hand-edit `distribution-manifest.json` or version files. Use the stamp script — it is the single source of truth.
- If the pipeline fails, fix the pipeline/system and push; Auto Heal + Watchdog must recover future failures.

See `docs/canonical-live-release-pipeline.md` before any build/deploy work.

This project uses **bd** (beads) for issue tracking. Run `bd onboard` to get started.

## Commit message policy

Run `scripts/dev.sh hooks` after cloning or after any Beads hook reinstall.
Commit subjects must remain meaningful Conventional Commit descriptions because
GitHub changelogs and tagged release summaries use the first line. Bead IDs may
appear only below the subject as a `Beads:` body trailer; ID-only subjects are
rejected by local hooks, CI, and the release-tag gate.

## One canonical Focusa Pi package (mandatory)

Exactly **one canonical Focusa Pi package** may be loadable from each Pi
extension discovery root (`~/.pi/agent/extensions/`, or `FOCUSA_PI_EXT_DIR`).
Backup, stage, legacy, rollback, disabled, and quarantine copies must live
under the sibling non-discovery root `~/.pi/agent/retired-extensions/`.
Compatibility symlinks may resolve only to that same canonical target without
duplicate registration. Starting Pi with `-ne`/`--no-extensions` never
satisfies acceptance: a fresh Pi process must start with zero duplicate tool
and zero duplicate flag errors. Install and OTA activation flow through the
typed receipt in `crates/focusa-cli/src/commands/pi_package.rs`.

## Per-turn metacognition + prediction loop (mandatory)

Every turn ends with the learning loop, recorded through Focusa itself:

1. **Reflect** — capture what worked and what failed that turn (`POST /v1/metacognition/capture`, or the `focusa_metacog_capture` tool) with kind `reflection` + a strategy_class.
2. **Predict** — record a bounded prediction for the next task (`POST /v1/predictions`: predicted_outcome, confidence, recommended_action, why).
3. **Evaluate** — on the next turn, evaluate the prior prediction against the actual outcome (`POST /v1/predictions/capture-outcome`).
4. **Retrieve** — before a related ask, `POST /v1/metacognition/retrieve` so prior lessons apply.

Predictions use a typed scope body (`scope.root_scope.scope_kind` = `Project`/`Host`).

## Tool flywheel + health discipline (mandatory)

Every tool family must close the loop with the others — no isolated tool, no broken tool. Before any feature work on tooling, and after any route/guard change, run `scripts/audit-route-health.mjs` and require a healthy sweep. Broken-tool reports are release blockers, not backlog. The ecosystem audit (docs/170) orders the cross-family work.

## Dynamic scope discipline

No hard-coded paths or magic roots anywhere in requests, tests, or fixtures. Scopes derive from the caller's actual project root; safety classification lives in one place (`scope_safety.rs`) and the json_guard accepts both the typed ScopeKind enum and query scope kinds.

## Turn closure

Every turn ends with the suggested next logical step, stated in one line.

## Quick Reference

```bash
bd ready              # Find available work
bd show <id>          # View issue details
bd update <id> --status in_progress  # Claim work
bd close <id>         # Complete work
bd sync               # Sync with git
```

## Public / Private Docs Boundary

Private operator docs may exist locally at `.focusa-private/`.

Agents must read `.focusa-private/INDEX.md` before touching SaaS strategy, SignalOS, commercial pricing/caps, install/purchase backend, raw proof, launch planning, or vendor/license registry work.

Agents must never commit `.focusa-private/`, raw transcripts, runtime objects, local host paths, admin URLs, customer data, or license data.

## Landing the Plane (Session Completion)

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:

   ```bash
   git pull --rebase
   bd sync
   git push
   git status  # MUST show "up to date with origin"
   ```

5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**

- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds

## RELEASE STRATEGY & VERSIONING

- Canonical policy: `docs/release-strategy.md` — read it before any release work.
- Three lanes: **0.9.x = patch** (security/critical only), **0.10.x = minor**
  (batched features, on cadence), **>= 1.0 = major** (breaking + migration notes).
- Pre-1.0 rule: breaking changes bump MINOR (`0.10.0`), never a `0.9.x` patch.
- Before tagging, classify the range: `python3 scripts/next-version.py`
  (used by CI in `.github/workflows/release-version-policy.yml`).
- Never tag outside the canonical pipeline
  (`scripts/create-dev-release-tag.sh --base <MAJOR.MINOR> --push`).
- Issue triage: `security` + `lane:patch` for security/critical;
  `lane:minor` for features/non-critical; `lane:major` for breaking plans.
