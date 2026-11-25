# Release Rules — Canonical Release Cycle — 2026-08-20 — DECISIVE, NO OPTIONS, AGENT-REMOVED

## Authority: ONE canonical path. If you ship, you follow this exactly.

No variants, shortcuts, `--no-verify`, manual manifest edits or immutable-tag rewrites. Use canonical scripts and actual terminal receipts; a deterministic design is not proof that a run completed. [The cohesive project journey](../agent/02-focusa-cohesive-project-flow.md) distinguishes source, public publication, installed activation and consumer acceptance. Approved development reload/deploy/testing is separate; an ordinary edit does not require a production release.

### Vocabulary — strict, no drift (enforced in CI)

- **Operator directive (2026-08-22):** When the operator says **"release"**,
  it means **FULL stable Release** — never default to a dev release or a
  tag-only push. The default is the full canonical stable Release unless
  the operator explicitly says **"dev release"** or **"tag release"**.
- **Release** = **stable canonical**. Every surface (CLI, daemon, TUI, session runner, Pi extension, menubar, updater, docs), every OS (`x86_64-unknown-linux-gnu`, `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-gnu`, `x86_64-pc-windows-msvc`, `aarch64-pc-windows-msvc`, `x86_64-apple-darwin`, `aarch64-apple-darwin`), every artifact. Must appear as **Latest** in GitHub sidebar (`isLatest=true`, `isPrerelease=false`), green required gates, exact candidate identity, the complete canonical artifact/signature/provenance matrix and verified installed/update/rollback outcomes. Asset count, an unsigned checksum file or `gh release view` success alone is not "shipped".
- **Dev release** = **`vX.Y.Z-dev` prerelease**. Also full surfaces + full OS and all currently required canonical matrix obligations, marked `prerelease`. Job counts are discovered from the current workflow/receipts, not this dated prose. No reduced matrix.
- **Temporary macOS proof delegation (until GitHub macOS returns):** GitHub's
  billing-locked `macos-latest` job is not a release veto when the matching
  Codemagic `menubar-macos-package-proof` release-tag build is green. That
  Codemagic receipt is mandatory release evidence, not an optional check;
  `codemagic.yaml` documents its exact package and codesign contract. The
  full temporary provider map and one-change-set GitHub restoration protocol
  are in `docs/178-focusa-temporary-ci-provider-parity-and-github-restoration-spec.md`.
  Remove this exception when GitHub-hosted macOS proves the same contract green.
- **Windows provider selection:** the current canonical workflow selects its approved producer; the owned OVH controller is an implemented source route, not proof every installer/signature is complete. AppVeyor is not a mandatory dependency merely because older receipts used it. When the optional AppVeyor route is selected, its one-job public lane admits
  only exact stable/`-dev` release tags or the explicit immutable recovery
  controller. It locates the gated GitHub draft by enumerating the authenticated
  release collection; the draft-blind `/releases/tags/{tag}` endpoint is banned.
  Ordinary branches, pull requests, and Nightlies must not consume this serial
  release lane.
- **Signed recovery:** rerun the canonical workflow from the exact immutable tag after correcting its supported inputs. Source/workflow defects require a newly gated stable candidate; existing tags remain unchanged. There is no skip-validation staged recovery lane. Both external receipt gates, exact-source checks, complete asset verification, signatures, provenance, deployment and consumer acceptance remain mandatory.
- **Signature separation:** Tauri's `.sig` files contain base64 Minisign signature boxes; `latest.json` preserves that text without re-encoding it. Release Ed25519 signatures for those same native updater assets use `.ed25519.sig` and the signed manifest, never overwrite the provider `.sig`. Release signing requires and verifies the exact `release.yml@refs/tags/<tag>` OIDC identity; existing approved repository secrets are automated inputs, not per-release manual key creation.
- **Cross-host resource health:** the OVH launcher reads KH's loopback-only Agent-KB health through the existing `kh` SSH alias (`FOCUSA_KH_RESOURCE_HEALTH_SSH_HOST`); OVH health uses its approved private URL. The bounded check creates no listener or service. A failed SSH/HTTP check blocks with its error; both hosts require disk below 90%, at least 15 GiB free, and zero pending replication. For a missing `kh` alias or other distinct failures, see the [2026-11-24 recovery and verification checklist](RELEASE_FAILURE_MODE_CATALOG_2026-08-17.md#2026-11-24--documentation-sweep-and-release-preflight-incident).
- **Journal routing:** `AGENT_KB_RELEASE_API_URL` selects the existing private Agent-KB master for journal reads/writes independently of KH resource health; absent that override, the existing `AGENT_KB_API_URL` contract remains. `FOCUSA_API_SSH_HOST=kh` sends Focusa journal/learning requests to KH's existing loopback daemon through bounded SSH with verified host keys. Authorization and request bodies travel only through encrypted stdin; no listener, service, second daemon, automatic mutation retry or trust bypass is created. An explicitly configured fresh primary must prove the complete durable event and matching append hash by authenticated readback; this is recorded as direct-master acceptance, not a fabricated replication receipt. Relay submissions retain their existing master-replication acknowledgment requirement.
- **Publication ordering:** every signed artifact publication, including an unsuffixed stable tag, remains a downloadable prerelease with `Latest=false`. Only the existing deployment acceptance step may set stable/Latest. Push and explicit dispatch share the same tag-based release concurrency group; serialization is not a substitute for actual terminal receipts.
- **Continuation reconciliation:** Genesis must commit scoped HLT ladder history through the shared goal-event compiler before recording readiness. Explicit confirmed takeover with a NEW replay key may reconcile an existing same-continuity initialization; exact replays retain the original receipt. This is the atomic constructor, not a marker/data edit or a relaxed mutation-admission rule.
- **Tag ≠ Release.** `git push --tags` only enqueues CI. `Latest` is valid only after all required packaging, signatures, compatibility, deployment and acceptance receipts settle for the exact candidate. Say "tag pushed, CI queued" vs "Release published as Latest". Never "pushed full release" when only tag exists.
- **Proof, not ticket closure, gates delivery.** Open issues remain open until their
  actual acceptance criteria are proven. They do not prevent building the signed
  candidate needed to collect installed evidence. Exact-source checks, scoped PR
  inclusion, signed artifacts, installed distribution parity, trusted OTA
  installability and final promotion checks remain mandatory. The standalone
  compatibility canary is not in the live workflow; its absence is not a passed
  canary or proof that a new candidate has been rolled back. Deferred work is
  not silently declared complete or added to the current release.

- **No partial releases.** No OS-only, surface-only, or docs-only ship without explicit operator written approval.
- **Production authority is artifact-bound.** Every Linux, Windows, and macOS Rust release provider receives the public `FOCUSA_AUTHORITY_ROOT_KEYS_JSON` at compile time (including `cross` container passthrough). Before upload, `scripts/verify-embedded-authority-root.py` must prove each authority-verifying CLI and daemon binary contains every configured production key ID and public key. TUI presence remains mandatory, but it does not link the license verifier and must not be given a decorative trust root. Liveness, a runtime environment drop-in, or an existing lease never substitutes for this binary proof.
- **Distribution parity is manifest-bound.** `distribution-manifest.json` carries full SHA-256 tree contracts for Rust runtime source, Pi extension, agent skills, current documentation, generated clients, installers, capability registries, and canonical installed paths. Every `v0.9.188+` release lane publishes and signs that manifest as a required asset and embeds the same bytes in the checksummed agent-context archive; only stable Release may proceed through full deployment and `Latest` promotion. Rust install promotes it to `/usr/local/lib/focusa/distribution-manifest.json` inside the same rollback boundary as all four binaries, systemd, health, and CallGraph acceptance. Releases before `v0.9.188` remain installable but cannot claim this manifest parity.
- **Containerized cross builds have one cancellation owner.** Every workflow invokes `scripts/ci/run-cancellation-safe-cross.sh`, which labels containers with the exact GitHub run ID, run attempt, job, and target; traps exit and cancellation; and removes only containers whose inspected labels match all four values. Each cross-bearing step also has an `always()` finalizer invoking the same owner as `cleanup-owned --target <exact-target>` when that build step actually ran: parent-shell cancellation may prevent the launcher's trap from running. Cleanup-only mode never starts a compiler, requires no cross installation, and remains identity-checked and idempotent. Finalizer errors are blocking. Broad process kills, global container cleanup, run-attempt-only matching, and manual cleanup before the exact provider run is terminal are forbidden. Force-kill/host-outage residue still requires verified recovery; unit/finalizer wiring tests are not a substitute for a real provider cancellation receipt with unrelated work preserved.

### What is deterministic and agent-removed (fewer failure points = less agent)

Every step below is code, not human memory. The agent never manually runs it; `git` or `create-dev-release-tag.sh` runs it.

| Step | Deterministic code | Agent removed | How |
|------|-------------------|---------------|-----|
| **Version surfaces** | `scripts/stamp-menubar-version.py vX.Y.Z` | Hand-editing release surfaces | ONLY writer for workspace, Pi extension, menubar, installer, agent-card, README, release-stamp, and `distribution-manifest.json` versions. The shared `scripts/distribution_manifest.py` contract computes every file/tree SHA-256, component count, capability digest, runtime path, `source_commit`, and UTC `generated_at`; it never writes. Never hand-edit the manifest. |
| **Pre-push** | `.git/hooks/pre-push` (common hooks) | Manual `verify-version-surfaces`/`sha256`/`fmt` checks | Runs commit-message, source preflight, convergence and installer checks. No `--no-verify`. |
| **Preflight** | `scripts/local-release-preflight.sh [--strict]` | Duplicate test builds | Blocking Windows path, version, digest, source ancestry and format checks. `--strict` is a compatibility spelling. No daemon compilation or live evidence claim. |
| **Commit message** | `scripts/validate-commit-messages.sh` + `commit-msg` hook | Agent crafting `fix:` vs `spec104:` | Enforces Conventional Commits `^(feat|fix|docs|test|refactor|perf|build|ci|chore|revert|proof|merge)(\(.+\))?!: .{4,}$` ≤100 chars, rejects `Beads:*`, ID-only, `WIP`. `spec104:` is rejected; use `fix:` (Spec104 inventory is `fix(release):`). |
| **CI gate** | `.github/workflows/ci.yml` | Manual `cargo test` polling | 5 jobs deterministic: `Menubar`, `Meaningful`, `Rust`, `Spec Gates (strict)`, `Release Automation`. Apt mirror resilient (`rm apt-mirrors.txt`, `sed azure→archive`, `timeout 45`, fallback `rg` binary). |
| **Spec132 wait** | Canonical tag preflight and Spec178 provider substitution; hosted matrix explicitly opt-in | Agent `gh api /rerun-failed-jobs` polling | Under Spec178, required OVH/Codemagic/provider evidence replaces hosted terminal execution; it does not waive consumer proof. All Spec132 jobs use the existing `FOCUSA_GITHUB_HOSTED_RELEASE_MATRIX=enabled` opt-in, including automatic main-push triggers, so an absent variable cannot incur hosted matrix spend. No agent rerun. |
| **Tag → Release** | `.github/workflows/release.yml` | Repeating CI inside Release | Require successful main CI on the exact tag commit; do not rebuild test fixtures or wait for CI in the Release job. Package every supported artifact, verify production trust roots, publish signed candidate (`Latest=false`), then deploy and verify installed parity and trusted OTA installability. The deployment workflow alone promotes stable/Latest. |
| **Verification** | Deployment workflow plus read-only release helper | Early or manual promotion | A download, signed candidate, queued deploy, and accepted production release are different states. The helper verifies the channel only after the applicable workflow has succeeded; it never edits it. |
| **Journal** | `journal_client` in `create-dev-release-tag.sh` | Agent forgetting optimization | Every Release failure is cataloged in `docs/current/RELEASE_FAILURE_MODE_CATALOG` H and `release-proof/audit/` — next run's `run-release-learning-guards.py` replays guards. Kept continually for optimizations. |

**Agent responsibility:** diagnose actual failures, preserve candidate/receipt identity, perform authorized supported recovery and verify consumer acceptance. Resolve genuine scope/cost/consent boundaries without inventing them. The canonical controller owns execution and settlement; an unchanged deterministic failure is not an endless retry instruction. No manual promotion or guessed provider/tag repair.

### The ONE command (agent-removed happy path)

```bash
bash scripts/create-dev-release-tag.sh --push
# or for a specific version:
bash scripts/create-dev-release-tag.sh --tag vX.Y.Z --push
```

On the single eligible builder, use `--no-wait-ci`: preparation checks exact-candidate CI once and exits **78 (`source_gate_pending`) before any tag** while that SHA is pending. This is an incomplete preparation phase, not release success. The caller exits to yield the builder; let the existing canonical source-CI workflow finish, then resume the **same approved tag and candidate** through the same helper. Failed, foreign-SHA, unreadable or malformed CI results never authorize tagging; provider failures remain errors rather than an empty queue. Explicit wait mode retains its wait. No new controller, journal, signing path or gate bypass is introduced.

Internally the script does the 7-step checklist deterministically — the agent does not run them by hand:

```
1. git status — clean, no ':' in evidence paths (Windows lint would FAIL)
2. stamp-menubar-version.py vX.Y.Z — 16 surfaces + manifest atomically (source_commit=HEAD, sha256 recomputed, generated_at now)
3. local-release-preflight.sh --strict — checks source integrity only
4. Commit stamped surfaces and candidate proof, then push main and require exact-source CI on the final SHA. Pending exit 78 yields without tagging.
5. Retain Spec178 platform proof obligations; binding provider receipts come from packaging, not from duplicate source tests.
6. Create and push the immutable candidate tag only after exact-source CI passes. Keep previous tags unchanged.
7. Release reuses source CI, publishes the signed full candidate, and dispatches stable deployments. Install, parity and OTA proof precede promotion.
8. verify exact terminal receipts, valid channel/Latest state and all required consumer outcomes; count/entry presence alone is insufficient
```

For **Dev release**: same, but tag is `vX.Y.Z-dev` and Release shows `isPrerelease=true`. No other difference.

### Source integrity and surface-specific acceptance

**Acceptance follows the surface being shipped.** For a Pi extension update, test Pi command loading, terminal behavior, settings, and session/scope isolation in Pi. For an OpenClaw awareness update, test the built plugin, gateway activation, and tool invocation in OpenClaw. Neither requires a browser or a daemon upgrade. An extension-only deployment is not a full stable release. The Spec 135 U3 browser evidence audit (`python3 tests/spec135_u3_browser_eval_suite_test.py`) applies only when claiming browser acceptance for generated UI; it is not a generic release prerequisite. Incomplete browser evidence remains incomplete, and browser isolation and privacy checks remain in the generic suite.

- The manifest's `source_commit` must resolve to an unambiguous ancestor; all covered bytes and version surfaces must match. `generated_at` is valid UTC provenance, not an expiry date. If covered bytes change, regenerate through the stamp writer. The exact tag SHA is independently bound by source CI, release provenance and deployment receipts.
- Cadence and file-count scoring advise on batching unattended releases. An explicitly requested small fix does not require a score override. Version safety and publication checks still apply.
- Spec 114 benchmark/Observatory and Spec 117 marketing walkthrough, postcard and docs-link checks are standalone checks for those surfaces. Task-closure proof belongs to the actual task, not a guess based on commit subjects. Neither is a generic publication veto.

### If Release fails (deterministic recovery, no agent guessing)

- `Missing successful Spec 132 terminal matrix candidate gate` → inspect the actual wait/receipt for the exact candidate. The documented wait can still time out or expose missing proof. Recover through the advertised controller; preserve the immutable tag and rerun only when the current contract/grant admits it.
- `Exact tag CI proof: failure` → CI failed on stamped SHA. Read `gh run view --log-failed`, fix code and rerun preflight `--strict`. Never move an existing release tag: controller-only repairs use the immutable recovery inputs; candidate-code changes require a new release version.
- `distribution parity drift blocks this release` → stamp was missed. `bash scripts/stamp-menubar-version.py vX.Y.Z` then preflight.
- Any other job failure → `gh run view <id> --log`, fix, preflight `--strict`, continue at failed step. Never skip preflight.

### Hotfix / rollback

- Hotfix: `vX.Y.Z-hotfix.1` — same checklist, full matrix, same 14 jobs.
- Rollback: use the supported verified prior-release install/update rollback transaction and prove preserved customer data/rights. Never rewrite a previous release tag as a rollback mechanism. Watchdog/deploy capabilities do not grant arbitrary retries or production mutation; current recovery and promotion receipts remain required.

### Evidence — v0.9.177 proven baseline

`CI 32273345113 5/5 green` → `Spec132 32274875930 11/11 green` (`windows-conpty`, `aarch64-pc-windows-msvc`) → `Release 32274874713 14/14 green` → `gh release view v0.9.177` `isLatest true 2026-08-19T16:37:02Z` `30+ assets`.

### Linting — always in pre-push + CI (fail locally)

`cargo fmt --all -- --check` (rustfmt 1.91, `unsafe { set_var }` required), `cargo clippy --workspace --all-targets -- -D warnings` 0, `php -l`, `python -m py_compile`, `svelte-check`, `spec104 --closure` (`STATIC_RE` + `MUTABLE_MARKERS`, `INF-01` for `TEST_MUTEX`), `verify-version-surfaces`, `git ls-files | grep ":"`. All <30s, so they gate `pre-push`.

### Release notes — script-owned, never agent (drastically detailed)

`scripts/generate-release-notes.py --tag vX.Y.Z --output /tmp/release-notes.md` is the ONLY writer. `release.yml: Generate release notes` calls it; the agent never hand-writes body.

What it emits (486 lines for `v0.9.177`): `TL;DR`, `Important software features & additions` (feat + area inference), `Breaking changes`, `Detailed changes by type` (feat/fix/perf/refactor/docs/build/ci/test/chore + `!`), `Changes by area` (table + file `+/−`), collapsible `File-level +/-` (top 80), `PRs merged`, `Issues resolved`, `Known issues`, `Contributors`, `Full commit audit` (every `SHA subject — author`), Upgrade/rollback/integrity, Downloads + Quick Start. Source is `git log RANGE --no-merges`, `git diff --numstat --shortstat`, `gh issue/pr` filtered by `closedAt>prev_published`, never stale strings.

Preview locally: `python3 scripts/generate-release-notes.py --tag v0.9.177 --preview | head -n 80` or `--dry-run` before `create-dev-release-tag.sh --push`.

### Journal — kept continually for optimizations

`journal_client plan → progress → learning-guards → candidate-ci → tag → release` is kept for every Release. Before stamping, `scripts/run-release-learning-guards.py` handles release resource safety. Source regressions run in their existing CI and manifest checks, not in a second dependency installation or compiler run.

In `auto` mode, a refused connection to the existing Focusa daemon defers optional learning through the existing Agent-KB journal: unavailable retrieval, prediction creation, capture and evaluation are explicit, with no invented IDs or measurements. Known prediction IDs survive a partial attempt; mutations are not automatically replayed. `required` mode keeps the refusal fatal. Authentication/scope denials, TLS errors, malformed responses and ambiguous timeout effects remain failures, not empty history. Journal publication/readback and recurrence guards remain. The journal records agent-intelligence benchmarks as **unmeasured** until there are execution-backed observations. Comparative performance claims require separate real measurements; an unrelated release never depends on the definition-only benchmark runner. Exact-source CI, signatures, installation and consumer acceptance are unchanged.

### Dry-run without a full Release

```bash
bash scripts/local-release-preflight.sh --strict       # source integrity, no push
bash scripts/create-dev-release-tag.sh --dry-run        # isolated clean checkout only; stamps and restores
FOCUSA_TEST_MODE=1 bash scripts/ci/run-spec-gates.sh      # explicit test suite on an approved isolated build host
```

A successful dry-run proves only the checks actually performed, not future provider success, full signatures, deployment or installed acceptance. Inspect the current script's dry-run side-effect/reconciliation contract before use; do not infer safe rollback or automatic execution from this historical example.
