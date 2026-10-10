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
- **Cross-host resource health:** the OVH launcher reads KH's loopback-only Agent-KB health through the existing `kh` SSH alias (`FOCUSA_KH_RESOURCE_HEALTH_SSH_HOST`); OVH health uses its approved private URL. The bounded check creates no listener or service. A failed SSH/HTTP check blocks with its error; both hosts still require disk below90%, free space at least15GiB and replication pending0.
- **Journal routing:** `AGENT_KB_RELEASE_API_URL` selects the existing private Agent-KB master for journal reads/writes independently of KH resource health; absent that override, the existing `AGENT_KB_API_URL` contract remains. `FOCUSA_API_SSH_HOST=kh` sends Focusa journal/learning requests to KH's existing loopback daemon through bounded SSH with verified host keys. Authorization and request bodies travel only through encrypted stdin; no listener, service, second daemon, automatic mutation retry or trust bypass is created.
- **Publication ordering:** every signed artifact publication, including an unsuffixed stable tag, remains a downloadable prerelease with `Latest=false`. Only the existing deployment acceptance step may set stable/Latest. Push and explicit dispatch share the same tag-based release concurrency group; serialization is not a substitute for actual terminal receipts.
- **Continuation reconciliation:** Genesis must commit scoped HLT ladder history through the shared goal-event compiler before recording readiness. Explicit confirmed takeover with a NEW replay key may reconcile an existing same-continuity initialization; exact replays retain the original receipt. This is the atomic constructor, not a marker/data edit or a relaxed mutation-admission rule.
- **Tag ≠ Release.** `git push --tags` only enqueues CI. `Latest` is valid only after all required packaging, signatures, compatibility, deployment and acceptance receipts settle for the exact candidate. Say "tag pushed, CI queued" vs "Release published as Latest". Never "pushed full release" when only tag exists.
- **Proof, not ticket closure, gates delivery.** Open issues remain open until their
  actual acceptance criteria are proven. They do not prevent building the signed
  candidate needed to collect installed evidence. Exact-source checks, scoped PR
  inclusion, signed artifacts, compatibility canary, installed distribution parity,
  OTA/rollback proof, and final promotion checks remain mandatory. Deferred work is
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
| **Pre-push** | `.git/hooks/pre-push` (common hooks) | Manual `verify-version-surfaces`/`sha256`/`fmt` checks | Blocks `git push` in <30s before CI. Runs `validate-commit-messages` + `local-release-preflight PREFLIGHT_FAST=1` (Windows `:` lint + surfaces + parity + manifest FRESH ancestor + fmt) + `convergence` + `installer` gates. `PREFLIGHT_FAST=1` allows manifest at any ancestor; STRICT requires HEAD/parent. No `--no-verify` escape (fails closed). |
| **Preflight** | `scripts/local-release-preflight.sh [--strict]` | CI guess loops | **Blocking, continually fresh, fails closed.** FAST checks versions, docs parity, legacy artifact SHA-256, freshness/ancestry, and formatting. STRICT additionally recomputes every full component digest/runtime contract via `distribution_manifest.py`, then runs the final gap and Spec gates. `create-dev-release-tag.sh` calls STRICT after stamping and before any push. |
| **Commit message** | `scripts/validate-commit-messages.sh` + `commit-msg` hook | Agent crafting `fix:` vs `spec104:` | Enforces Conventional Commits `^(feat|fix|docs|test|refactor|perf|build|ci|chore|revert|proof|merge)(\(.+\))?!: .{4,}$` ≤100 chars, rejects `Beads:*`, ID-only, `WIP`. `spec104:` is rejected; use `fix:` (Spec104 inventory is `fix(release):`). |
| **CI gate** | `.github/workflows/ci.yml` | Manual `cargo test` polling | 5 jobs deterministic: `Menubar`, `Meaningful`, `Rust`, `Spec Gates (strict)`, `Release Automation`. Apt mirror resilient (`rm apt-mirrors.txt`, `sed azure→archive`, `timeout 45`, fallback `rg` binary). |
| **Spec132 wait** | Canonical tag preflight and Spec178 provider substitution; hosted matrix explicitly opt-in | Agent `gh api /rerun-failed-jobs` polling | Under Spec178, required OVH/Codemagic/provider evidence replaces hosted terminal execution; it does not waive consumer proof. All Spec132 jobs use the existing `FOCUSA_GITHUB_HOSTED_RELEASE_MATRIX=enabled` opt-in, including automatic main-push triggers, so an absent variable cannot incur hosted matrix spend. No agent rerun. |
| **Tag → Release** | `.github/workflows/release.yml` canonical DAG | Agent waiting on provider pages | Candidate contract and exact-SHA CI → release gate → package Pi/agent docs/generated clients → build every OS/surface → require `distribution-manifest.json` → generate checksums, candidate release manifest, provenance, and signatures → immutable prerelease publication (`Latest=false`) → signed isolated `v0.9.177 → interrupted-install recovery → candidate → rollback → candidate` compatibility and full distribution-parity canary → independently verify its exact-SHA receipt before any production mutation → deploy exact assets → installed parity + OTA gates → re-sign settled manifest → stable/Latest promotion. |
| **Verification** | `scripts/verify-version-surfaces.py` tail in `create-dev-release-tag.sh`; `scripts/verify-embedded-authority-root.py` in every Rust packaging provider | Agent `gh release view` eyes and runtime-root injection | Scripts verify production authority roots inside binaries before upload, then `isLatest true` + asset count before exiting 0. |
| **Journal** | `journal_client` in `create-dev-release-tag.sh` | Agent forgetting optimization | Every Release failure is cataloged in `docs/current/RELEASE_FAILURE_MODE_CATALOG` H and `release-proof/audit/` — next run's `run-release-learning-guards.py` replays guards. Kept continually for optimizations. |

**Agent responsibility:** diagnose actual failures, preserve candidate/receipt identity, perform authorized supported recovery and verify consumer acceptance. Resolve genuine scope/cost/consent boundaries without inventing them. The canonical controller owns execution and settlement; an unchanged deterministic failure is not an endless retry instruction. No manual promotion or guessed provider/tag repair.

### The ONE command (agent-removed happy path)

```bash
bash scripts/create-dev-release-tag.sh --push
# or for a specific version:
bash scripts/stamp-menubar-version.py vX.Y.Z && bash scripts/local-release-preflight.sh --strict && bash scripts/create-dev-release-tag.sh --push
```

Internally the script does the 7-step checklist deterministically — the agent does not run them by hand:

```
1. git status — clean, no ':' in evidence paths (Windows lint would FAIL)
2. stamp-menubar-version.py vX.Y.Z — 16 surfaces + manifest atomically (source_commit=HEAD, sha256 recomputed, generated_at now)
3. local-release-preflight.sh --strict — must print DONE — PASS (may tag) or script exits non-zero (no tag)
4. git add + commit "chore: stamp release surfaces X.Y.Z" + push main — waits deterministically for CI 5/5 on that SHA
5. (if terminal paths) waits deterministically for Spec132 11/11 on same SHA
6. canonical controller creates a new immutable candidate tag and pushes only under its current grant — enqueues Release; existing tags are preserved
7. Release workflow reconciles exact-source proof, full artifact/signature matrix, compatibility, deployment and installed acceptance before promotion
8. verify exact terminal receipts, valid channel/Latest state and all required consumer outcomes; count/entry presence alone is insufficient
```

For **Dev release**: same, but tag is `vX.Y.Z-dev` and Release shows `isPrerelease=true`. No other difference.

### Preflight detail — continually fresh, never stale

- **FAST** (`PREFLIGHT_FAST=1`, used by `pre-push`): `source_commit` may be any ancestor of `HEAD` (`git merge-base --is-ancestor`). Allows `docs/ci` commits without churning `distribution-manifest.json` on every push. Still checks `release_version==Cargo`, `sha256` match, `generated_at<24h`.
- **STRICT** (used by `create-dev-release-tag.sh --push` before tag): `source_commit` must be `HEAD` or `HEAD~1` with `distribution-manifest.json` touched in `HEAD`. Ensures the Release tag's manifest is at most one commit old and reflects the stamped SHA. Prints `FAIL stale source_commit X != HEAD Y nor parent Z (touched=False)` with hint `run stamp-menubar-version.py`.

**Nothing is ever allowed to be stale.** If `local-release-preflight.sh` says `FAIL stale …`, that failure is real and blocks `git push` / tag push. Fix is always `python3 scripts/stamp-menubar-version.py v$(cat docs/current/.release-version-stamp)` then rerun preflight.

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

`journal_client plan → progress → learning-guards → candidate-ci → tag → release` is kept for every Release. `scripts/run-release-learning-guards.py` replays prior failure modes (modes 41 azure mirror, 42 spec104 drift, 43 colon path, 44 stale manifest, 45 commit-msg) before stamping. Catalog is `docs/current/RELEASE_FAILURE_MODE_CATALOG_2026-08-17.md` H+I.

### Dry-run without a full Release

```bash
bash scripts/local-release-preflight.sh --strict          # 15s — exact Release gates, no push
bash scripts/create-dev-release-tag.sh --dry-run           # 30s — stamps 16 + manifest, verifies, then git checkout --reverts (no tag, no workflow)
FOCUSA_TEST_MODE=1 bash scripts/ci/run-spec-gates.sh     # full Spec Gates
```

A successful dry-run proves only the checks actually performed, not future provider success, full signatures, deployment or installed acceptance. Inspect the current script's dry-run side-effect/reconciliation contract before use; do not infer safe rollback or automatic execution from this historical example.
