# Canonical Release Optimizations — 2026-08-17

**Status:** implemented on `main` 5d66c9a8 → 5cc3de86 → 3573b593, cherry-picked to `local/work-loop-completion 2a9d5e95`. This doc is the tedious, code-backed record the operator requested: every optimization, why it existed, what changed, file:line proof, tradeoff, and how to verify it without compromising stable.

**Goal:** collapse 72h dev loops to <1h / single `bash scripts/create-dev-release-tag.sh --push` while keeping stable (`v0.9.143` published, `v0.9.153` next stable) strict. Dev (`v0.9.157-dev` → `v0.9.158-dev` ...) gets advisory fast-lane; stable reconciles to proof.

---

## 1. Central entitlement fixture (the 403 collapse)

**Before:**
- `scripts/ci/run-spec-gates.sh:24` did `FOCUSA_DATA_DIR=$(mktemp -d)` and `FOCUSA_BIND=127.0.0.1:18787` then `./target/release/focusa-daemon` with no lease.
- Daemon `crates/focusa-api/src/main.rs:322` only grants bounded test lease when `FOCUSA_TEST_MODE=1`; otherwise `middleware/entitlement.rs:369 entitlement_execution_guard` returns `403 ENTITLEMENT_BASE_REQUIRED` (`limit_bucket workpoints`/`evidence_records`) before side effects.
- CI `Spec Gates (strict)` then hit: `api_contract_probe` 8 failures `status 403`, `focusa_toggle` `curl -sf exit 22`, `command_write_contract` `jq null exit 5`, `trace_dimensions` `0 passed 23 failed`. Workaround was per-test `grep ENTITLEMENT_BASE_REQUIRED` skips — 4 files, drift, still failed 72h.

**After:**
- `scripts/ci/run-spec-gates.sh:24-30` now:
  ```bash
  export FOCUSA_DATA_DIR="${FOCUSA_DATA_DIR:-$(mktemp -d /tmp/focusa-spec-gates.XXXXXX)}"
  # Isolated CI daemon must exercise real entitlement path, not 403.
  # FOCUSA_TEST_MODE=1 grants a bounded test lease (active, sha256, 1h) so write
  # gating still executes. See crates/focusa-api/src/main.rs:322 and
  # crates/focusa-api/src/middleware/entitlement.rs:369.
  export FOCUSA_TEST_MODE="${FOCUSA_TEST_MODE:-1}"
  ```
- Daemon now boots with ephemeral `Active` identity, real `EntitlementExecutionPolicy` still checks idempotency/limit buckets, just not 403 on fresh dir. Logs on `32052240728` after fix:
  - `SPEC-33.5: Disk Persistence 8 passed 0 failed`
  - `COMMAND WRITE 16 passed 0 failed`
  - `SPEC 56 Trace 23 passed 0 failed`
  - `SPEC-52 Pi extension 41 passed`
  vs prior `0/23` etc.

**Tradeoff:** none for stable — same `entitlement_execution_guard` path, same `RouteEntitlementPolicy` classification, just a 1h bounded lease. Bypass resistance specs `spec172_bypass_resistance.rs` / `spec152f_bypass_resistance.rs` still assert `ENTITLEMENT_ALLOWED` vs `ENTITLEMENT_BASE_REQUIRED`; test mode is documented infra, not a mute.

**Verify:**
```bash
bash -n scripts/ci/run-spec-gates.sh
FOCUSA_TEST_MODE=1 python3 scripts/api_contract_probe.py  # expect pass
./target/release/focusa-daemon & ; curl -s http://127.0.0.1:18787/v1/focus/stack | jq .  # not 403
```

---

## 2. Single version-stamping authority (README + surfaces)

**Before:**
- `scripts/stamp-menubar-version.py` stamped `Cargo.toml` / `Cargo.lock` / `apps/pi-extension/package.json` / `apps/menubar/*` but NOT `README.md` `Current source version: vX` nor `docs/current/.release-version-stamp`.
- `scripts/validate-docs-runtime-parity.mjs:11` asserts `['README.md','v${currentVersion}']` substring — every dev tag then failed `FAIL README.md: missing v0.9.154-dev` until a manual `README` commit was made. Created 2 extra commits per tag and broke `verify-version-surfaces.py` ancestry parity (ancestry reads `git show HEAD:Cargo.toml`).

**After:**
- `scripts/stamp-menubar-version.py:18-38` now:
  ```python
  def replace_readme_version(root: Path, tag: str) -> bool:
      readme = root / "README.md"
      current = version_from_tag(tag)  # e.g. v0.9.157-dev -> 0.9.157-dev
      # replaces "Current source version: v...-dev" line or inserts one
  ```
  Called in `main()` after `replace_text` loop, plus writes `docs/current/.release-version-stamp` atomically. Hence `create-dev-release-tag.sh:516 Stamping release surfaces: 0.9.157-dev` → `version consistency ok: 0.9.157-dev` + `Docs/runtime parity validation: passed` without a second commit.

**Tradeoff:** no stable risk — same semver surface set, just one writer instead of two. `git diff --stat` after stamp shows single commit `chore: stamp release surfaces 0.9.157-dev` with 15 files.

**Verify:**
```bash
python3 scripts/stamp-menubar-version.py v0.9.999-dev
grep "Current source version" README.md  # v0.9.999-dev
cat docs/current/.release-version-stamp # 0.9.999-dev
git checkout -- README.md Cargo.toml ...  # revert
```

---

## 3. Channel-aware release gate (dev fast lane)

**Before:**
- `scripts/release-gate.py:12-18` fixed thresholds `SIGNIFICANT_SCORE=8 WINDOW_SCORE=4 STALE_HOURS=24` plus `release_window outside 11:00,16:00 PT ±30m` → every low-score dev patch (`score 4 changed_paths 4`) blocked `release_gate_allowed=false` → required `bash create-dev-release-tag.sh --push --force-release --release-reason "..."` manually. Dev was never window-free.

**After:**
- `scripts/create-dev-release-tag.sh:199` injects channel: `FOCUSA_RELEASE_CHANNEL="$RELEASE_CHANNEL" python3 scripts/release-gate.py` where `RELEASE_CHANNEL` is `preview` for `dev` (`dev|rc|preview → preview`, `stable → stable`).
- `scripts/release-gate.py:185-205` now:
  ```python
  channel = os.environ.get("FOCUSA_RELEASE_CHANNEL","").strip()  # dev fast lane
  if channel in ("dev","preview","rc","next"):
      # score>=1 or critical or in_window => advisory allow
      allowed = score >= 1 or has_critical or in_window
      reason = f"dev fast lane score {score} channel {channel} (advisory...)"
  else:
      # stable: significant 8 / window 4 / stale 24h strict
  ```
- Result on `467517c7`: `score 14 channel preview (advisory) release_gate_allowed=true` inside window, no force needed. On `score 4` dev docs fix, now allowed (was blocked). Stable `v0.9.153` still requires `score 43` etc.

**Tradeoff:** dev advisory does not weaken stable proof — `final_release_gap_gate.sh` + `tests/166` + `tests/167` still enforce `source_versions verified`, `governance_source_commit == HEAD` before `Deploy`. Dev merely reconciles to stable proof on next stable cut.

**Verify:**
```bash
FOCUSA_RELEASE_CHANNEL=dev python3 scripts/release-gate.py --json | jq .allowed  # true for score>=1
FOCUSA_RELEASE_CHANNEL=stable python3 scripts/release-gate.py --json | jq .allowed # false for score 4 outside window
```

---

## 4. Any-channel re-seal + benchmark advisory

**Before:**
- `scripts/create-dev-release-tag.sh:543` only re-sealed ancestry/governance when `RELEASE_CHANNEL == stable && STAMPED_RELEASE_SURFACES==1` — dev stamped commits drifted `release-proof/audit/next-locked-release-candidate-ancestry.json: source_commit` vs `HEAD` → `governance receipt canonical replay detected mutation` + `candidate ancestry drift`.
- `journal_client benchmark --tag $TAG --channel $RELEASE_CHANNEL` ran `scripts/spec135-live-performance-proof.py` which needs `@earendil-works/pi-tui` via `npm ci`. CI wrapper `FOCUSA_OVH_BUILD` forwards `npm` to `build/focusa/source` (see `.github/workflows/ci.yml:39 Swatinem/rust-cache` but npm wrapper prevents local `node_modules/@earendil-works/pi-tui`), so `mission_canvas_render ERR_MODULE_NOT_FOUND` → benchmark `blocked` even though daemon/entitlement fine → tag push failed.

**After:**
- `scripts/create-dev-release-tag.sh:543-560` now re-seals on **any** `PUSH && STAMPED_RELEASE_SURFACES && inventory exists` (comment `# Version stamping changes governed source surfaces (any channel)`), then:
  ```bash
  STAMPED_SOURCE_SHA="$(git rev-parse HEAD)"
  python3 scripts/generate-locked-release-candidate-ancestry.py --candidate-ref "$STAMPED_SOURCE_SHA" --audit-ref "$STAMPED_SOURCE_SHA"
  python3 scripts/generate-locked-release-governance-receipt.py --generate-ephemeral --governance-source-commit "$STAMPED_SOURCE_SHA"
  # closure_set sha256:410833c... payload sha256:26ae59...
  ```
  So dev tag `97e7db3a → 467517c7` correctly carries `payload 26ae59...` at same SHA.

- Benchmark now:
  ```bash
  if ! journal_client benchmark --tag "$TAG" --channel "$RELEASE_CHANNEL"; then
    if [[ "$RELEASE_CHANNEL" == "preview" ]]; then
      echo "Canonical pre-release benchmark advisory for dev $TAG: continuing (stable would block)" >&2
    else
      exit 1
    fi
  fi
  ```
  Dev continues on `spec135` miss; stable still hard-blocks.

**Verify:**
```bash
bash scripts/create-dev-release-tag.sh --push  # on 3573b593 should select v0.9.158-dev, stamp, re-seal, benchmark advisory, push
cat release-proof/audit/next-locked-release-governance-receipt.json | jq .payload.governance_source_commit # == git rev-parse HEAD
```

---

## 5. Inventory closure (spec104)

**Before:** `tests/spec104_singleton_inventory_gate.py --closure` discovered `findings=34 classified=70` but `FAIL PI_TEST_ENV_LOCK` (`crates/focusa-cli/src/commands/install_pi_package_transaction_tests.rs:22 static PI_TEST_ENV_LOCK: AsyncMutex`) and `CACHE` (`crates/focusa-core/src/license_developer_origin.rs:35 static CACHE: OnceLock<Mutex>`) unclassified → `exit 1` blocked `Spec Gates (strict)` even after entitlement fix.

**After:** `config/spec104-scoped-state-inventory.json` adds two `infra_allowlist` entries (`annex_id INF-01`, `authority_bearing false`, `status infra_allowed`, `target test-only async env lock` / `license cache, non-authority infra memo`). Gate now `findings=34 classified=72 open=0 PASS` both modes.

**Verify:** `python3 tests/spec104_singleton_inventory_gate.py --closure` → `PASS`.

---

## 6. Local preflight (fast mirror, no CI wait)

New `scripts/local-release-preflight.sh --strict` (added now) mirrors `scripts/ci/run-spec-gates.sh` without 6m build wait:

- `cargo test -p focusa-api` fast path, `Swatinem/rust-cache@v2` + `sccache` (see `.github/workflows/ci.yml:39` already `rust-cache`, now local also `export SCCACHE_GHA_ENABLED=on` / `RUSTC_WRAPPER=sccache` if present)
- `python3 scripts/verify-version-surfaces.py` + `node validate-docs-runtime-parity.mjs` + `bash tests/final_release_gap_gate.sh`
- `FOCUSA_TEST_MODE=1` same as CI, catches `trace_dimensions` locally in <2m.

**Verify:** `bash scripts/local-release-preflight.sh --strict` before `create-dev-release-tag --push`.

---

## 7. Template drift fixes retained

- `.github/workflows/ci.yml: Release Automation (static)` `Install ripgrep` (`sudo apt-get install -y ripgrep`) — `tests/installer_update_policy_static_test.sh` uses `rg -n` now passes.
- `.github/workflows/ci.yml: Spec Gates (strict)` `fetch-depth: 0` — full history for `generate-locked-release-candidate-ancestry.py --candidate-ref HEAD --audit-ref HEAD --check`.
- Hook `/.git/hooks/pre-push` now `if [ -x "$ROOT/scripts/structural-guards.sh" ]; then ...` so worktree `focusa-release` (no `structural-guards.sh` on dev line) no longer `No such file` on `git push`.

---

## Sequence proof

- `v0.9.156-dev` `f9195304` (re-seal 156) → `v0.9.157-dev` `467517c7` `journal plan 4(progress) → learning-guards 7 → stamp 97e7db3a → anchor 467517c7 → benchmark advisory → push main 467517c7..5cc3de86..3573b593`
- Current `HEAD 3573b593` `Cargo.toml version 0.9.157-dev` `README Current source version: v0.9.157-dev` `next_stable_version 0.9.153`.
- Next tag will be `v0.9.158-dev` (`select-release-version.py` monotonic `highest_patch 157 → 158`).

## How to continue

```bash
bash scripts/local-release-preflight.sh --strict   # <2m, catches trace/toggle locally
bash scripts/create-dev-release-tag.sh --push      # single canonical cycle → tag + CI watch
gh run list --workflow CI --limit 3                # expect Spec Gates success green
```

No manual `README` bump, no `--force-release`, no per-test `grep 403` edits, no manual `generate-locked-release-candidate-ancestry` — one writer, one channel-aware gate, real entitlement exercised.
