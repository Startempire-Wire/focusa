# Release Rules — Canonical Release Cycle — 2026-08-20 — DECISIVE, NO OPTIONS, AGENT-REMOVED

## Authority: ONE canonical path. If you ship, you follow this exactly.

No variants, no shortcuts, no `--no-verify`, no hand-editing `distribution-manifest.json`. The cycle is deterministic; the agent is removed from every check that does not need a brain.

### Vocabulary — strict, no drift (enforced in CI)

- **Operator directive (2026-08-22):** When the operator says **"release"**,
  it means **FULL stable Release** — never default to a dev release or a
  tag-only push. The default is the full canonical stable Release unless
  the operator explicitly says **"dev release"** or **"tag release"**.
- **Release** = **stable canonical**. Every surface (daemon, TUI, CLI, Pi extension, menubar, updater, docs), every OS (`x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `x86_64-pc-windows-msvc`, `aarch64-pc-windows-msvc`, `x86_64-apple-darwin`, `aarch64-apple-darwin`), every artifact. Must appear as **Latest** in GitHub sidebar (`isLatest=true`, `isPrerelease=false`), green badge, `gh release view vX.Y.Z` succeeds with 30+ assets + `SHA256SUMS`. Nothing else is "shipped".
- **Dev release** = **`vX.Y.Z-dev` prerelease**. Also full surfaces + full OS, same 14-job Release matrix, marked `prerelease`. No reduced matrix.
- **Temporary macOS proof delegation (until GitHub macOS returns):** GitHub's
  billing-locked `macos-latest` job is not a release veto when the matching
  Codemagic `menubar-macos-package-proof` release-tag build is green. That
  Codemagic receipt is mandatory release evidence, not an optional check;
  `codemagic.yaml` documents its exact package and codesign contract. The
  full temporary provider map and one-change-set GitHub restoration protocol
  are in `docs/178-focusa-temporary-ci-provider-parity-and-github-restoration-spec.md`.
  Remove this exception when GitHub-hosted macOS proves the same contract green.
- **Tag ≠ Release.** `git push --tags` only enqueues CI. `Latest` flips only after `Release 14/14 green`. Say "tag pushed, CI queued" vs "Release published as Latest". Never "pushed full release" when only tag exists.
- **No partial releases.** No OS-only, surface-only, or docs-only ship without explicit operator written approval.
- **Production authority is artifact-bound.** Every Linux, Windows, and macOS Rust release provider receives the public `FOCUSA_AUTHORITY_ROOT_KEYS_JSON` at compile time (including `cross` container passthrough). Before upload, `scripts/verify-embedded-authority-root.py` must prove each authority-verifying CLI and daemon binary contains every configured production key ID and public key. TUI presence remains mandatory, but it does not link the license verifier and must not be given a decorative trust root. Liveness, a runtime environment drop-in, or an existing lease never substitutes for this binary proof.

### What is deterministic and agent-removed (fewer failure points = less agent)

Every step below is code, not human memory. The agent never manually runs it; `git` or `create-dev-release-tag.sh` runs it.

| Step | Deterministic code | Agent removed | How |
|------|-------------------|---------------|-----|
| **Version surfaces** | `scripts/stamp-menubar-version.py vX.Y.Z` | Hand-editing 16 files | ONLY writer for `Cargo.toml`, `Cargo.lock`, `apps/pi-extension/package.json` + `package-lock.json`, `apps/menubar/package.json` + `package-lock.json` + `tauri.conf.json` + `Cargo.toml` + `Cargo.lock` + `Settings.svelte` + `auto-compaction.ts::EXTENSION_BUILD`, `scripts/install-focusa.sh`, `agent-card.json` (+ `card_digest`), `README.md`, `.release-version-stamp`, **and** `distribution-manifest.json` (`sha256` recompute for 5 artifacts, `source_commit=$(git rev-parse --short HEAD)`, `generated_at` UTC now). Never hand-edit manifest. |
| **Pre-push** | `.git/hooks/pre-push` (common hooks) | Manual `verify-version-surfaces`/`sha256`/`fmt` checks | Blocks `git push` in <30s before CI. Runs `validate-commit-messages` + `local-release-preflight PREFLIGHT_FAST=1` (Windows `:` lint + surfaces + parity + manifest FRESH ancestor + fmt) + `convergence` + `installer` gates. `PREFLIGHT_FAST=1` allows manifest at any ancestor; STRICT requires HEAD/parent. No `--no-verify` escape (fails closed). |
| **Preflight** | `scripts/local-release-preflight.sh [--strict]` | 12m CI guess loop | **Blocking, continually fresh, fails closed. One command, one result.** Checks: 1) Windows NTFS lint (`git ls-files | grep ":"` + `?*\"<>|`), 2) Version surfaces (16), 3) Docs/runtime parity, 4) Manifest freshness (`release_version==Cargo`, `sha256` recomputed, `generated_at<24h`, `source_commit` ancestor in FAST / HEAD|parent in STRICT), 5) `cargo fmt --check`, 6) with `--strict`: `final_release_gap_gate` + Spec Gates (`FOCUSA_TEST_MODE=1`). Prints `DONE — PASS (may tag)` or `FAIL` with fix hint. `create-dev-release-tag.sh` calls it with `--strict` after stamp before any push. |
| **Commit message** | `scripts/validate-commit-messages.sh` + `commit-msg` hook | Agent crafting `fix:` vs `spec104:` | Enforces Conventional Commits `^(feat|fix|docs|test|refactor|perf|build|ci|chore|revert|proof|merge)(\(.+\))?!: .{4,}$` ≤100 chars, rejects `Beads:*`, ID-only, `WIP`. `spec104:` is rejected; use `fix:` (Spec104 inventory is `fix(release):`). |
| **CI gate** | `.github/workflows/ci.yml` | Manual `cargo test` polling | 5 jobs deterministic: `Menubar`, `Meaningful`, `Rust`, `Spec Gates (strict)`, `Release Automation`. Apt mirror resilient (`rm apt-mirrors.txt`, `sed azure→archive`, `timeout 45`, fallback `rg` binary). |
| **Spec132 wait** | `.github/workflows/release.yml` `require_success_with_wait` | Agent `gh api /rerun-failed-jobs` polling | When terminal paths changed, Release now **waits up to 20m** for `Spec 132 terminal matrix 11/11` (`windows-conpty` + `aarch64-pc-windows-msvc`) on same `headSha` instead of immediate `Missing successful Spec 132 terminal matrix candidate gate` FAIL. No agent rerun. |
| **Tag → Release** | `.github/workflows/release.yml` 14 jobs | Agent waiting on `gh run list` | `Release Contract Check` (open-issue gate + PR inclusion + candidate-SHA receipts with wait) → `OTA installability` → `Spec104` → `Exact tag CI proof` (waits for stamped SHA CI) → `Final release gap gate` → `Lock candidate` → `Create Release` → `Package Pi` → `Build Menubar 2x` → `Rust Binaries 6x` → `Publish SHA256SUMS` → `Dispatch Deploy`. |
| **Verification** | `scripts/verify-version-surfaces.py` tail in `create-dev-release-tag.sh`; `scripts/verify-embedded-authority-root.py` in every Rust packaging provider | Agent `gh release view` eyes and runtime-root injection | Scripts verify production authority roots inside binaries before upload, then `isLatest true` + asset count before exiting 0. |
| **Journal** | `journal_client` in `create-dev-release-tag.sh` | Agent forgetting optimization | Every Release failure is cataloged in `docs/current/RELEASE_FAILURE_MODE_CATALOG` H and `release-proof/audit/` — next run's `run-release-learning-guards.py` replays guards. Kept continually for optimizations. |

**What still needs agent (and only this):** fixing a real code `FAIL` (e.g., `cargo clippy -D warnings` 1 warning, `spec104` drift) — code change, then rerun `--strict`. No decision, no polling.

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
6. git tag -f vX.Y.Z HEAD -m "Release vX.Y.Z stable canonical all surfaces and OS" + push tag — enqueues Release
7. Release workflow waits deterministically for tag CI proof, then publishes Release 14/14 and flips Latest
8. verify gh release view vX.Y.Z isLatest=true isPrerelease=false assets 30+
```

For **Dev release**: same, but tag is `vX.Y.Z-dev` and Release shows `isPrerelease=true`. No other difference.

### Preflight detail — continually fresh, never stale

- **FAST** (`PREFLIGHT_FAST=1`, used by `pre-push`): `source_commit` may be any ancestor of `HEAD` (`git merge-base --is-ancestor`). Allows `docs/ci` commits without churning `distribution-manifest.json` on every push. Still checks `release_version==Cargo`, `sha256` match, `generated_at<24h`.
- **STRICT** (used by `create-dev-release-tag.sh --push` before tag): `source_commit` must be `HEAD` or `HEAD~1` with `distribution-manifest.json` touched in `HEAD`. Ensures the Release tag's manifest is at most one commit old and reflects the stamped SHA. Prints `FAIL stale source_commit X != HEAD Y nor parent Z (touched=False)` with hint `run stamp-menubar-version.py`.

**Nothing is ever allowed to be stale.** If `local-release-preflight.sh` says `FAIL stale …`, that failure is real and blocks `git push` / tag push. Fix is always `bash scripts/stamp-menubar-version.py v$(cat docs/current/.release-version-stamp)` then rerun preflight.

### If Release fails (deterministic recovery, no agent guessing)

- `Missing successful Spec 132 terminal matrix candidate gate` → **no longer happens**; Release now waits 20m for it. If it still timeouts, fix Spec132 code, rerun preflight `--strict`, re-push tag (same SHA).
- `Exact tag CI proof: failure` → CI failed on stamped SHA. Read `gh run view --log-failed`, fix code, rerun preflight `--strict`, then `git tag -f` moves tag to fixed SHA.
- `distribution parity drift blocks this release` → stamp was missed. `bash scripts/stamp-menubar-version.py vX.Y.Z` then preflight.
- Any other job failure → `gh run view <id> --log`, fix, preflight `--strict`, continue at failed step. Never skip preflight.

### Hotfix / rollback

- Hotfix: `vX.Y.Z-hotfix.1` — same checklist, full matrix, same 14 jobs.
- Rollback: `git tag -f vX.Y.(Z-1)` + `git push --force origin tag` only with operator explicit written approval; `release-pipeline-watchdog` and `deploy-live-daemon` heal deploys, but Latest flip still requires the 7 steps.

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

`Dry run complete; reverted stamped files.` means `create-dev-release-tag.sh --push` would green.
