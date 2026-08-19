# Release Rules — 2026-08-19 — DECISIVE, NO OPTIONS

## Authority: ONE canonical path. No alternatives. No guessing.

**This is the ONLY way to ship. If you are about to tag, you follow this exactly. No variants, no shortcuts, no `--force`.**

### Vocabulary (mandatory)

- **Release** = full canonical stable. Every surface, every OS, every artifact. Shows as **Latest** in repo sidebar, green CI badge, `gh release view vX.Y.Z` exists. Nothing else counts as shipped.
- **Dev release** = nightly/dev channel. Also full surfaces and OS. Marked `prerelease`. No reduced matrix.
- **No partial releases.** No OS-only or surface-only ships. If you think you need one, stop and get explicit operator approval in writing.

Say: "tag pushed, CI queued" or "Release published as Latest" — never "pushed full release" when only tag exists. Tag ≠ Release.

### Preflight — THE blocking gate (fails closed, continually fresh)

`scripts/local-release-preflight.sh` is the ONLY gate before any tag push. It is blocking and non-stale. It fails closed. No bypass.

**What it checks (all must PASS, <30s non-strict, <2m --strict):**

1. **Windows path lint** — `git ls-files | grep ":"` and illegal chars `?*\"<>|` → `FAIL` (would block `windows-conpty` + `aarch64-pc-windows-msvc` on NTFS).
2. **Version surfaces** — `python3 scripts/verify-version-surfaces.py vX.Y.Z` checks 16 surfaces (`Cargo.toml`, `Cargo.lock`, `apps/pi-extension/package.json`, `apps/menubar/package.json`, `tauri.conf.json`, `Settings.svelte`, `install-focusa.sh`, `agent-card.json`, `distribution-manifest.json::release_version`, etc.) → `All checked version surfaces match X.Y.Z` required.
3. **Docs/runtime parity** — `node scripts/validate-docs-runtime-parity.mjs` → PASS.
4. **Distribution-manifest freshness (continually fresh)** — `release_version == Cargo.toml version`, `source_commit in (HEAD, HEAD~1 if manifest touched in HEAD)`, every artifact `sha256` recomputed, `generated_at <24h` → `manifest FRESH` else `FAIL stale source_commit` / `FAIL stale sha256` / `FAIL stale generated_at` (hint: `run stamp-menubar-version.py`).
5. **Gap gate** (`--strict` only) — `bash tests/final_release_gap_gate.sh` → PASS.
6. **Spec gates** (`--strict` only, `FOCUSA_TEST_MODE=1`) — `bash scripts/ci/run-spec-gates.sh` or static `spec104` → PASS.
7. **Format** — `cargo fmt --all -- --check` → 0.

**Result:** `=== local preflight: DONE — PASS (may tag)` means you may tag. Any `FAIL` means fix, rerun preflight, no options.

### Stamping — single-source, no hand edits

`scripts/stamp-menubar-version.py vX.Y.Z` is the ONLY writer for all version surfaces **including** `docs/contracts/spec141/generated-capability-v2/distribution-manifest.json`.

What it does atomically:
- Updates 16 surfaces: `Cargo.toml`, `Cargo.lock` (9 root packages), `apps/pi-extension/package.json` + `package-lock.json`, `scripts/install-focusa.sh`, `apps/menubar/package.json` + `package-lock.json` + `tauri.conf.json` + `Cargo.toml` + `Cargo.lock`, `Settings.svelte`, `auto-compaction.ts::EXTENSION_BUILD`, `agent-card.json` (with `card_digest` recomputed), `README.md`, `.release-version-stamp`.
- **Regenerates `distribution-manifest.json`**: recomputes `sha256` for every artifact, sets `source_commit` to `git rev-parse --short HEAD`, sets `generated_at` to now UTC, sets `release_version` to `X.Y.Z`.

**Never hand-edit `distribution-manifest.json`.** If preflight says `FAIL stale sha256`, run `stamp-menubar-version.py` again.

### The 7-step checklist — execute in order, no skipping, no reordering

This is the checklist that makes releases buttery smooth. Do not guess. Do not invent steps.

```
1. git status — must be clean, no untracked evidence files with ':'.
2. bash scripts/stamp-menubar-version.py vX.Y.Z     # stamps 16 surfaces + manifest atomically
3. bash scripts/local-release-preflight.sh --strict  # must print DONE — PASS (may tag)
4. git add -A && git commit -m "chore: stamp release surfaces X.Y.Z" && git push origin main
   # wait for CI on that SHA to go green (poll gh api)
5. gh api repos/.../actions/runs/<CI>/jobs → all 5 CI jobs: Menubar success, Meaningful success, Rust success, Spec Gates (strict) success, Release Automation success
6. If Spec132 terminal matrix touches terminal evidence: wait for Spec132 11/11 green (windows-conpty success, aarch64-pc-windows-msvc success)
7. git tag -f vX.Y.Z HEAD -m "Release vX.Y.Z stable canonical all surfaces and OS" && git push --force origin tag vX.Y.Z
   # wait for Release workflow 14/14 green (Release Contract Check success, Exact tag CI proof success, Create Release success, Package Pi success, Build Menubar 2x success, Rust Binaries 6x success, Publish SHA256SUMS success, Dispatch Deploy success)
8. gh release view vX.Y.Z — must show isLatest=true, isPrerelease=false, publishedAt now, assets for every OS.
```

**Stable vs Dev:** Same checklist, same artifacts. `Release` = stable Latest, `Dev release` = prerelease tag `vX.Y.Z-dev`. No matrix reduction ever.

### Why the old way was painful (so you don't repeat it)

- **16 surfaces, not one** — missing one = `verify-version-surfaces` FAIL. Fixed by single-source stamp.
- **Manifest hand-edits → stale sha256** — now stamp recomputes.
- **PT window + score blocks** → batch by score, not clock; don't use `--force-release`.
- **CI queue 30 jobs** → use `sccache` + `rust-cache`, single ubuntu job, `cargo nextest`.
- **No pre-push guard** → preflight now gates in 15s locally, not 12m in CI.
- **Tag ≠ Release** → Release workflow must be green before sidebar shows Latest.

### Linting — always in pre-push + CI (fail locally, not after push)

- `cargo fmt --all -- --check` (rustfmt 1.91 pinned)
- `cargo clippy --workspace --all-targets -- -D warnings`
- `python3 -m py_compile` + `php -l`
- `svelte-check` (`/opt/node-v22.22.3-linux-x64/bin/npm --prefix apps/menubar run check`)
- `tests/spec104_singleton_inventory_gate.py --closure`
- `python3 scripts/verify-version-surfaces.py <tag>`
- `git ls-files | grep ":"` (Windows lint)

All <30s, so they belong in pre-push.

### If Release fails

- `Release Contract Check: Missing successful Spec 132 terminal matrix candidate gate for <SHA>` → Spec132 not yet green on same SHA. Wait for Spec132 `completed success`, then `gh api .../runs/<Release>/rerun-failed-jobs --method POST`. Do NOT retag.
- `Exact tag CI proof: failure` → CI not yet green on tag SHA. Wait for CI, then rerun Release.
- Any other failure → read `gh api .../jobs/<id>/logs`, fix, rerun preflight --strict, then continue checklist at the failed step. Never skip preflight.

### Hotfix / rollback

- Hotfix: `vX.Y.Z-hotfix.1` — same 7 steps, full matrix, no window bypass without operator approval.
- Rollback: `git tag -f vX.Y.(Z-1)` only with operator approval; Watchdog heals deploy, but tag move requires written approval.

### Evidence

After `v0.9.177` proven: `CI 32273345113 5/5 green`, `Spec132 32274875930 11/11 green`, `Release 32274874713 14/14 green`, `gh release view v0.9.177 Latest true`.
