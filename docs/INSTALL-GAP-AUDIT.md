# Install Gap Audit — Process + Spec

**Audit date:** 2026-06-26
**Audited against:** Live installer at https://install.focusa.dev/focusa (234 lines)
**Audited against:** GitHub release v0.9.25-dev (13 assets)
**Audited against:** `docs/INSTALL-BINARY-SPEC.md` (1051 lines)

---

## A. Gaps in the **REAL INSTALL PROCESS**

### A1. CRITICAL — Real installer does NOT install binaries at all
**Evidence:**
- Installer line 168: `exec python3 -m focusa_cli "$@"` — drops Python launcher stub
- Installer comment: `# Real binary arrives when the engine binary is wired in.`
- Installer NEVER downloads `focusa`, `focusa-daemon`, or `focusa-tui` from GitHub releases
- Installer NEVER checks asset existence, never calls `gh release` API

**Impact:** Anyone running `curl ... | bash` today gets a Python launcher, not the actual Rust binaries. The 13 published GitHub release assets (17MB daemon, 10MB CLI, 3MB TUI) are never used.

### A2. CRITICAL — Linux musl asset does NOT exist in release
**Evidence:**
- GitHub release assets: only `x86_64-unknown-linux-gnu` and `aarch64-unknown-linux-gnu` (glibc only)
- No `*-unknown-linux-musl` assets published
- Alpine Linux users cannot install

**Impact:** Alpine, Void (musl), Chimera Linux users excluded despite being listed as supported in spec §2.2.

### A3. CRITICAL — Windows ARM64 asset missing
**Evidence:** No `aarch64-pc-windows-msvc` or `arm64-pc-windows-msvc` asset in release.

**Impact:** Windows on ARM (Surface Pro X, etc.) excluded.

### A4. CRITICAL — License validation fails to actually set license state correctly
**Evidence:**
- Installer sets `TIER` from JSON response but only when Python's `json.load` succeeds
- Sets `eval: $([ "$EVAL" -eq 1 ] && echo true || echo false)` — but license.json doesn't carry tier or feature info
- `license.json` written to `$HOME/.config/focusa/license.json` — but daemon doesn't read this file (no code in installer to install daemon)

**Impact:** License info gets written but daemon never installed to consume it.

### A5. CRITICAL — macOS daemon install EXPLICITLY SKIPPED
**Evidence:** Installer line: `log "macOS detected; skipping launchd plist for now (Phase 2)."`

**Impact:** Per operator rule (Section 0), this MUST be Phase 1.5. Currently broken on every macOS install.

### A6. CRITICAL — No macOS code signing verification
**Evidence:** Installer has zero `codesign` invocations. No Developer ID check. No notarization check.

**Impact:** On Apple Silicon, Gatekeeper will block un-signed binaries with "cannot be opened because the developer cannot be verified" error. Real users will see this and quit.

### A7. CRITICAL — No SHA256SUMS verification
**Evidence:** Installer never downloads `SHA256SUMS.txt`. Never runs `sha256sum -c`. No checksums in any GitHub release.

**Impact:** Bit-flips during download, MITM attacks, and corrupted artifacts go undetected.

### A8. CRITICAL — No GPG / cosign signature verification
**Evidence:** Installer has zero `gpg --verify` or `cosign verify-blob` calls.

**Impact:** Supply chain attacks undetectable.

### A9. CRITICAL — No Windows PowerShell installer
**Evidence:** `curl -sI https://install.focusa.dev/install.ps1` → 404
`curl -sI https://install.focusa.dev/install.cmd` → 404
`curl -sI https://install.focusa.dev/install/windows` → 404

**Impact:** Windows users have NO installer at all (except via WSL).

### A10. CRITICAL — No daemon systemd unit installed
**Evidence:** Installer has zero `systemctl` calls. No `unit file` template. No `/etc/systemd/system/focusa-daemon.service` template.

**Impact:** Daemon (when it actually installs) would never auto-start. User must run manually.

### A11. CRITICAL — Pi extension not installed
**Evidence:** `--with-pi` flag references `https://raw.githubusercontent.com/mariozechner/pi-coding-agent/main/install.sh` but doesn't validate Pi is even available or installed.

**Impact:** Silent failure if Pi is not installed. No error to user.

### A12. CRITICAL — OpenClaw bridges explicitly placeholder
**Evidence:** Installer line: `warn "OpenClaw bridges arrive in Phase 2 of the install gateway spec."`

**Impact:** Per operator rule (Section 0): if `--with-openclaw` is documented as a flag, the feature MUST exist in Phase 1.5. Currently it's a no-op that warns.

### A13. CRITICAL — License endpoint requires POST but installer uses default
**Evidence:** 
- Installer: `VALIDATE_RESP=$(curl -ksS -X POST ...)` — correct, uses POST
- Spec §4.3 says POST — correct
- BUT installer doesn't handle all license response codes (valid:false vs network error vs invalid format)

**Impact:** Unclear error messages for users with invalid keys.

### A14. MEDIUM — No version pinning
**Evidence:** Installer doesn't say `focusa v0.9.25-dev`. Doesn't compare local version to latest. No `focusa update` command implemented.

**Impact:** Users don't know what version they have. Stale installs.

### A15. MEDIUM — Stale macOS label
**Evidence:** Plist example uses `com.startempire.focusa-daemon` — consistent, but no Bundle ID validation.

**Impact:** Bundle ID conflict on shared systems.

### A16. MEDIUM — No post-install smoke test
**Evidence:** Installer has zero `focusa doctor` or `curl /v1/health` calls.

**Impact:** Installer reports success even when daemon is broken.

### A17. MEDIUM — No rollback on failure
**Evidence:** No stash of previous install. No atomic swap.

**Impact:** Failed upgrade leaves system in broken state.

### A18. MEDIUM — `--with-engine` calls engine installer but doesn't verify it succeeded
**Evidence:** `run "bash /tmp/engine-install.sh --prefix "$PREFIX" --no-service"` — no error handling, no verification.

**Impact:** Engine install failure not surfaced to user.

### A19. LOW — `LICENSE_REGISTRY` defaults to `https://install.focusa.dev` even though spec says `https://focusa.dev`
**Evidence:** Spec §4.1 says license validation base is `install.focusa.dev`. WP JSON shows endpoints at `/wp-json/wpuiai-ai-cloud/v1/license/*`. Consistent but worth noting.

### A20. LOW — Color codes broken in non-TTY
**Evidence:** `\033[1;34m` escape codes print literally in pipes/redirects.

**Impact:** Logs messy when piped.

### A21. LOW — `trap` not set on errors
**Evidence:** `set -euo pipefail` is set but no trap to clean up `/tmp/engine-install.sh`, `/tmp/pi-install.sh`, `/tmp/focusa-install.sh`.

**Impact:** /tmp files leak.

---

## B. Gaps in the **SPEC I WROTE**

### B1. CRITICAL — Spec says `verify_signature` is "(P1)" but no test for signature
**Location:** §5.2 Signature Verification
**Issue:** Spec says "Optional GPG signing using cosign" but if Apple requires Developer ID, this is NOT optional on macOS — it's P0.

### B2. CRITICAL — §9 Acceptance Criteria is OUTDATED
**Location:** §9 (lines ~360) vs §15 (lines ~900)
**Issue:** §9 says P0 for things now moved to Phase 2 (§15 added new P0 list). The two acceptance criteria lists contradict each other.

### B3. CRITICAL — §10 says "Daemon doesn't install (Phase 1.4 stub)" — inaccurate
**Issue:** Should say "Daemon NEVER installs (entire installer is stub launcher — zero Rust binary deployment)"

### B4. CRITICAL — §10 "DAEMON_INSTALL_BUG" missing from gaps list
**Location:** Real installer at install.focusa.dev/focusa line ~167
**Bug:** `cat > "$PREFIX/bin/focusa" <<'LAUNCHER'` writes Python launcher. NEVER downloads or installs the actual Rust binaries.

### B5. MEDIUM — §11 says "Phase 1.5" without specifying what code goes there
**Issue:** Spec lists many Phase 1.5 items but doesn't say WHICH FILE or PR adds them. Phase 1.5 should have explicit file/PR breakdown.

### B6. MEDIUM — §12 Open Questions has 5 unresolved items but no deadlines
**Issue:** All 5 questions marked "TBD" — no one assigned, no deadline.

### B7. MEDIUM — §13 and §17 are duplicate "Bead Reference" sections
**Issue:** §13 lists 3 beads. §17 lists 4 beads. Inconsistent.

### B8. MEDIUM — §14.6 Verification Status table is INCONSISTENT
**Issue:** 
- macOS Intel: ❌ Phase 1.5 / 🔴 LaunchAgent deployment skipped
- macOS Apple Silicon: ❌ Phase 1.5 / 🔴 LaunchAgent deployment skipped
- But §10 says macOS daemon install is "DEFERRED" 
And §15 says macOS LaunchAgent "MUST be Phase 1.5"
The table cells say "🔴" but the meaning is unclear (red status? red work?)

### B9. MEDIUM — §14.7 Compatibility Test Matrix has "⏳" markers but no targets
**Issue:** ⏳ means "spec'd not yet implemented" but no owner, no PR, no date.

### B10. MEDIUM — §15 Acceptance Criteria contradicts itself
**Issue:** §15.1 has 11 detection criteria. §15.2 has 3 service criteria. §15.3 has 3 code signing criteria. But §9 has 24 acceptance criteria. Which is authoritative?

### B11. MEDIUM — §15.6 AX (All Platforms) only lists 3 items
**Issue:** Original §7 has 8 failure modes with recovery_hint. §15 only lists 3. Where did the other 5 go?

### B12. MEDIUM — §16 "Summary of Gaps Per Platform" uses unclear emoji states
**Issue:** 🟢/🟡/🔴 not defined. Reader has to infer.

### B13. LOW — §4.5 Eval Mode doesn't say HOW daemon reads eval
**Issue:** Says "Daemon reads `eval: true`" but doesn't say which file or how.

### B14. LOW — §4.6 License Persistence doesn't say HOW daemon re-validates
**Issue:** Says "every 24h" but no implementation detail. Where is the timer? In daemon process? systemd timer?

### B15. LOW — §5A.3 Verification Matrix has macOS Apple Silicon row missing
**Issue:** Table has macOS Intel but Apple Silicon row says "Apple Silicon" — should be specific about Apple Silicon M1/M2/M3

### B16. LOW — §14.11 Roadmap has contradictory "Phase 1.5 (Critical — blocks MVP Cohort)" vs "Phase 2.0 (POST-MVP)"
**Issue:** Phase 2.0 is POST-MVP but Roadmap item says "Implement focusa update command (Linux + Mac + Windows)". If it's POST-MVP, when does the user actually get update capability?

### B17. LOW — §0 CRITICAL CONSTRAINT contradicts itself
**Issue:** Says "If a system is compatible, we CANNOT defer support" but then §9 still has Phase 1/Phase 2 acceptance criteria mixing. Section 0 needs explicit "ALL compatible platforms MUST ship in same release" rule.

### B18. LOW — Spec doesn't say how license tier/features map to daemon behavior
**Issue:** §4.4 shows license response with `features: ["daemon", "tui", "menubar"]` but doesn't specify what happens if feature is missing — does daemon refuse to start? Run in degraded mode?

### B19. LOW — Spec doesn't define "tier"
**Issue:** Says `TIER="operator"` default but doesn't define what tiers exist or what each enables.

### B20. LOW — Spec doesn't address Docker images
**Issue:** Containerized deployment is mentioned in spec §17 as "Phase 3 package managers" but Docker is THE primary deployment for many users (k8s, ECS, etc.).

---

## C. Summary: 21 install process gaps + 20 spec gaps = 41 total

| Severity | Install Process | Spec | Total |
|----------|----------------|------|-------|
| CRITICAL | 13 (A1-A13) | 4 (B1-B4) | 17 |
| MEDIUM | 5 (A14-A18) | 7 (B5-B11) | 12 |
| LOW | 3 (A19-A21) | 9 (B12-B20) | 12 |
| **Total** | **21** | **20** | **41** |

## D. Recommended Actions

1. **Fix the real installer FIRST** (A1, A4, A5, A6, A7, A9, A10, A11, A12) — these are blockers
2. **Then publish missing assets** (A2, A3)
3. **Then reconcile spec sections** (B2, B5, B8, B10, B16) — contradictions need resolution
4. **Then resolve open questions** (B6, B12, B18, B19)
5. **Then add low-priority features** (A14-A18, B14, B15, B17, B20) — updates, smoke tests, rollback

## E. Bead References
- `focusa-iqqi` — INSTALL-BINARY-SPEC scope (this audit identifies gaps in spec + process)
- `focusa-cqhi` — Tauri/menubar local artifact (macOS daemon install)
- `focusa-cme3` — Tauri release artifacts
- `focusa-7wgk` — Fresh-operator dry-run (would catch A1, A16)
