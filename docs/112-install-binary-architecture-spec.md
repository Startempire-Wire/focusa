# Spec 112 — Install Binary Architecture

**Spec number:** 112
**Status:** Specification (no implementation yet)
**Operator ask:** Smart, system-detecting installer that installs proper binary and weaves into license authority (WordPress website).
**Source URL:** https://install.focusa.dev/focusa
**Last verified:** 2026-06-26
**Related docs:** `docs/INSTALL-GAP-AUDIT.md` (41 gaps identified)

---

## 0. CRITICAL CONSTRAINT: No Deferrals for Compatible Systems

**If a system is compatible with Focusa, we CANNOT defer support for it.**

This means:
- ❌ macOS LaunchAgent is NOT "deferred to Phase 2" — it MUST ship in Phase 1.5
- ❌ Windows PowerShell installer is NOT "future work" — it MUST exist at MVP launch
- ❌ Linux musl support is NOT "nice to have" — if asset builds, it's Phase 1.5
- ❌ Any "Phase 2 placeholder" comment in the installer MUST be removed

The current installer at install.focusa.dev/focusa says:
> "macOS detected; **Phase 1.5: LaunchAgent deployment MANDATORY — no deferral allowed**"

**This is a bug, not a feature.** If macOS is a supported platform (Apple Silicon + Intel), the daemon MUST install as a LaunchAgent. No deferrals allowed.

The same applies to:
- Windows native support: If Windows is supported, there MUST be a `focusa.ps1` at install.focusa.dev
- WSL fallback: If WSL works, document it; if not, don't pretend it does
- musl/alpine: If we build musl assets, support musl

**Rule:** A system is either compatible (full support in Phase 1.5) or not in our compatibility matrix at all. No middle ground.

---

## 1. Goal

Make `curl -fsSL https://install.focusa.dev/focusa | bash -s -- ...` work reliably across all supported platforms, install the correct binary for the user's system, and integrate seamlessly with the WordPress-hosted license authority.

The installer must be:
- **Smart** — detect OS, arch, libc, init system, dependencies
- **Self-discovering** — find the latest stable release from GitHub
- **Verifying** — checksum + signature validation on every artifact
- **Atomic** — succeed fully or roll back cleanly
- **License-aware** — validate against WP REST API before commercial install
- **AX-correct** — clear progress, actionable errors, recovery hints (Spec92)

---

## 2. System Detection Specification

### 2.1 OS Detection

The installer MUST detect:

| Signal | Source | Purpose |
|--------|--------|---------|
| Kernel name | `uname -s` | Linux / Darwin / Windows |
| Kernel release | `uname -r` | Linux version compatibility |
| Distro | `/etc/os-release` or `/etc/lsb-release` | Package manager selection |
| Architecture | `uname -m` | Binary asset selection |
| libc variant | `ldd --version` | glibc vs musl |
| Init system | systemctl / launchctl / service | Service management |
| Shell | `$SHELL` or `/etc/passwd` | Profile updates |
| PATH layout | `$PATH` | Symlink placement |
| User permissions | `id` / `sudo -n` | Privilege escalation |
| Disk space | `df -P "$PREFIX"` | Pre-flight check |
| Existing install | `~/.focusa/`, `/usr/local/bin/focusa` | Upgrade path |

### 2.2 Architecture → Asset Mapping

The installer MUST map `uname -m` + libc detection to a GitHub release asset:

| `uname -m` | libc | Asset suffix |
|------------|------|--------------|
| x86_64 | glibc | `x86_64-unknown-linux-gnu` |
| x86_64 | musl | `x86_64-unknown-linux-musl` |
| aarch64 | glibc | `aarch64-unknown-linux-gnu` |
| aarch64 | musl | `aarch64-unknown-linux-musl` |
| armv7 | glibc | `armv7-unknown-linux-gnueabihf` |
| x86_64 | (macOS) | `x86_64-apple-darwin` |
| aarch64 | (macOS) | `aarch64-apple-darwin` |
| x86_64 | (Windows) | `x86_64-pc-windows-msvc` |

Fallback chain when asset missing:
1. Try exact match
2. Try glibc/musl swap
3. Try x86_64 → aarch64 (via Rosetta on macOS, no fallback on Linux)
4. Print actionable error with `recovery_hint`

### 2.3 Init System Detection

| Detected | Service file location | Enable command |
|----------|----------------------|-----------------|
| `systemctl` (system) | `/etc/systemd/system/focusa-daemon.service` | `sudo systemctl enable --now focusa-daemon` |
| `systemctl --user` | `~/.config/systemd/user/focusa-daemon.service` | `systemctl --user enable --now focusa-daemon` |
| `launchctl` (macOS) | `~/Library/LaunchAgents/com.startempire.focusa-daemon.plist` | `launchctl load -w <plist>` |
| None (container) | Skip service install with `--no-service` flag |

The installer MUST NOT fail if no init system is detected — it should print a `recovery_hint` like "Run `focusa-daemon &` manually or use --no-service flag".

### 2.4 Dependency Detection

The installer MUST check for required dependencies BEFORE downloading:

| Tool | Required for | Fallback |
|------|-------------|----------|
| `curl` or `wget` | Download | die with recovery_hint |
| `tar` or `unzip` | Extract | die with recovery_hint |
| `sha256sum` | Verification | die (security) |
| `systemctl` | Daemon service | warn, suggest `--no-service` |
| `python3` (eval mode only) | Stub launcher | Skip stub if not needed |

The installer MUST use `command -v` not `which` for portability.

---

## 3. Binary Selection Specification

**Implementation note:** Binary selection and download are executed by the Rust `focusa install` subcommand per §15A. The bash / PowerShell installers do NOT perform these steps directly — they delegate after detecting platform. See §15A for the canonical architecture.

### 3.1 Asset Types to Install

For each release, GitHub publishes these assets (verified in v0.9.25-dev):

| Binary | Source asset | Install location | Purpose |
|--------|--------------|------------------|---------|
| `focusa` | `focusa-{VERSION}-{target}` | `~/.focusa/bin/focusa` | CLI (also serves as `focusa install` orchestrator) |
| `focusa-daemon` | `focusa-daemon-{VERSION}-{target}` | `~/.focusa/bin/focusa-daemon` | Daemon |
| `focusa-tui` | `focusa-tui-{VERSION}-{target}` | `~/.focusa/bin/focusa-tui` | TUI dashboard |

The Rust `focusa install` orchestrator (§15A) selects, downloads, SHA256-verifies, and places all three. The shell bootstrapper only downloads the `focusa` CLI itself (which then does the rest).

### 3.2 Optional Components

| Component | Asset | Install when |
|-----------|-------|--------------|
| UIAI Engine | (separate `engine` script) | `--with-engine` flag |
| Pi wrapper | (downstream install.sh) | `--with-pi` flag |
| OpenClaw bridges | (Phase 2 placeholder) | `--with-openclaw` flag | — **REMOVED: if --with-openclaw is supported, bridges MUST exist in Phase 1.5** |
| Menubar app | `Focusa_*.dmg` | macOS only |

### 3.3 Symlink Strategy

```
~/.focusa/bin/focusa        # real binary
~/.focusa/bin/focusa-daemon # real binary
~/.focusa/bin/focusa-tui    # real binary
~/.local/bin/focusa -> ~/.focusa/bin/focusa        # if ~/.local/bin exists and is on PATH
/usr/local/bin/focusa -> ~/.focusa/bin/focusa      # if /usr/local/bin is writable
```

Priority order:
1. `~/.local/bin/` (XDG-style, no sudo)
2. `/usr/local/bin/` (system-wide, requires sudo)
3. Print `recovery_hint` to add one of these to PATH

---

## 4. License Authority Integration

### 4.1 WordPress REST API (verified live)

The license authority lives at:
- Base: `https://install.focusa.dev`
- Endpoint: `/wp-json/wpuiai-ai-cloud/v1/license/validate`

### 4.2 License Flow

```
[1] User runs: curl ... | bash -s -- --license-key focusa_live_xxxxx
[2] Installer extracts LICENSE_KEY from args
[3] Installer POSTs to /license/validate with the key
[4] WordPress responds with {valid: true, tier: "operator|enterprise", ...}
[5] Installer writes license.json to ~/.config/focusa/license.json
[6] focusa-daemon reads license.json on startup, validates against registry
```

### 4.3 License Validation Request

```http
POST /wp-json/wpuiai-ai-cloud/v1/license/validate
Headers:
  Content-Type: application/json
  X-License-Key: focusa_live_xxxxx
Body:
  {"license_key": "focusa_live_xxxxx"}
```

### 4.4 License Validation Response

```json
{
  "valid": true,
  "tier": "operator",
  "issued_at": "2026-01-15T00:00:00Z",
  "expires_at": "2027-01-15T00:00:00Z",
  "features": ["daemon", "tui", "menubar"],
  "max_users": 5
}
```

### 4.5 Eval Mode (`--eval`)

When `--eval` is passed:
- Skip WP validation
- Set `eval: true` in `~/.config/focusa/license.json`
- Daemon reads `eval: true` and applies rate limits (e.g., 100 req/min)
- Daemon marks itself as `tier: "evaluation"` in `/v1/health`

### 4.6 License Persistence

The daemon MUST periodically re-validate against WP (e.g., every 24h) to handle license revocation:
- If WP returns `valid: false`, daemon enters "license_expired" mode
- Daemon refuses new mutations but allows reads
- Daemon emits alert via `/v1/doctor` and Focus Slice

---

## 5. Verification Specification

### 5.1 Checksum Verification

Release artifacts MUST be published with `SHA256SUMS.txt`:

```
# SHA256SUMS for focusa v0.9.25-dev
<hash>  focusa-v0.9.25-dev-x86_64-unknown-linux-gnu
<hash>  focusa-daemon-v0.9.25-dev-x86_64-unknown-linux-gnu
<hash>  focusa-tui-v0.9.25-dev-x86_64-unknown-linux-gnu
```

Installer verifies each downloaded asset:
```bash
sha256sum -c --ignore-missing SHA256SUMS.txt
```

If checksum mismatch: `die` with recovery_hint pointing to https://install.focusa.dev/help/security.

### 5.2 Signature Verification (P1)

Optional GPG signing using `cosign`:
```bash
cosign verify-blob focusa-daemon-{VERSION}-{target} \
  --signature focusa-daemon-{VERSION}-{target}.sig \
  --certificate-identity-regexp 'https://github.com/Startempire-Wire/focusa'
```

If verification fails: warn but don't block (P1) or die (P0).

### 5.3 License Signature Verification

The license.json MAY be signed by WP for tamper detection. Daemon validates signature on load.

---

## 5A. Platform-Specific Detection (Mac / Windows)

### 5A.1 macOS Detection Specification

When `uname -s` returns `Darwin`, the installer MUST:

1. **Architecture detection:**
   - `uname -m` returns `arm64` on Apple Silicon
   - `uname -m` returns `x86_64` on Intel
   - Map to `aarch64-apple-darwin` or `x86_64-apple-darwin`

2. **macOS version detection:**
   - `sw_vers -productVersion` → e.g., `14.5`
   - Reject if macOS < 11.0 (Big Sur) — focusa requires Apple Silicon or Intel macOS 11+

3. **Code signing verification:**
   - Run `codesign -dv <binary>` on downloaded assets
   - Verify `Developer ID Application: Startempire Wire Inc.` (must be issued by Apple)
   - If unsigned or wrong identity: warn with `recovery_hint: 'Contact support@focusa.dev for signed binaries'`

4. **Gatekeeper / Notarization:**
   - `xattr -p com.apple.quarantine <binary>` must be empty (notarized)
   - If quarantined: `xattr -d com.apple.quarantine <binary>`
   - If notarization missing: warn (not blocking for CLI)

5. **LaunchAgent deployment (Phase 1.5 mandatory):**
   - Service file: `~/Library/LaunchAgents/com.startempire.focusa-daemon.plist`
   - Load: `launchctl load -w ~/Library/LaunchAgents/com.startempire.focusa-daemon.plist`
   - The current installer at install.focusa.dev/focusa says: "macOS detected; **Phase 1.5: LaunchAgent deployment MANDATORY — no deferral allowed**" — this MUST be fixed

6. **Path differences:**
   - `~/.local/bin/` (XDG, but macOS doesn't auto-add to PATH)
   - `/usr/local/bin/` (requires sudo on Apple Silicon due to SIP)
   - Symlink path: `/opt/homebrew/bin/` (Apple Silicon) or `/usr/local/bin/` (Intel)

### 5A.2 Windows Detection Specification

When the installer detects Windows (NOT bash on WSL — that's Linux), the installer MUST:

1. **Detection path:**
   - The bash installer at install.focusa.dev/focusa is POSIX-only
   - Windows users currently get 404 for `/install.ps1` and `/install.cmd`
   - Need SEPARATE installer at install.focusa.dev/install.ps1

2. **Windows PowerShell installer MUST:**
   - Detect PowerShell version: `$PSVersionTable.PSVersion`
   - Require PowerShell 5.1+ (Windows 10 1809+) or PowerShell Core 7+
   - Detect architecture: `[System.Environment]::Is64BitOperatingSystem`
   - Map to `x86_64-pc-windows-msvc`

3. **Windows SCM service deployment:**
   - `sc.exe create focusa-daemon binPath= "C:\Program Files\Focusa\focusa-daemon.exe"`
   - `sc.exe start focusa-daemon`
   - Requires admin (UAC) — use `Start-Process -Verb RunAs`

4. **Path differences:**
   - `%LOCALAPPDATA%\Programs\Focusa\` (user install, no admin)
   - `%ProgramFiles%\Focusa\` (system install, requires admin)
   - Add to PATH: `[Environment]::SetEnvironmentVariable('PATH', $env:PATH + ';C:\Program Files\Focusa', 'User')`

5. **WSL / Git Bash fallback:**
   - If bash is available but Windows native: warn "Running on Windows via bash. Consider installing PowerShell version for native SCM support."
   - Still allow install via bash — daemon will run as foreground process

### 5A.3 Cross-Platform Verification Matrix

| Platform | Architecture | Asset | Service | Path | Code Sign |
|----------|--------------|-------|---------|------|-----------|
| Linux x86_64 | glibc | focusa-x86_64-unknown-linux-gnu | systemd | /usr/local/bin | GPG |
| Linux x86_64 | musl | focusa-x86_64-unknown-linux-musl | systemd | /usr/local/bin | GPG |
| Linux aarch64 | glibc | focusa-aarch64-unknown-linux-gnu | systemd | /usr/local/bin | GPG |
| macOS Intel | x86_64 | focusa-x86_64-apple-darwin | launchd | /usr/local/bin | Apple Developer ID |
| macOS Apple Silicon | arm64 | focusa-aarch64-apple-darwin | launchd | /opt/homebrew/bin | Apple Developer ID |
| Windows x86_64 | x86_64 | focusa-x86_64-pc-windows-msvc | SCM | C:\Program Files | Authenticode |

**Current status:** Only Linux x86_64 + macOS DMG + Windows .app archives exist as GitHub release assets. The Windows .exe/.msi installer is MISSING. The PowerShell installer at install.focusa.dev/install.ps1 is MISSING.

---

## 6. Atomicity & Rollback Specification

### 6.1 Stash Strategy

Before overwriting existing install:
```bash
if [ -d "$PREFIX" ]; then
  log "Stashing existing install to $PREFIX.state/last-known-good/"
  run "mv $PREFIX $PREFIX.state/last-known-good"
fi
```

### 6.2 Rollback on Failure

```bash
if ! post_install_health_check; then
  log "Install failed health check. Rolling back."
  rm -rf "$PREFIX"
  if [ -d "$PREFIX.state/last-known-good" ]; then
    mv "$PREFIX.state/last-known-good" "$PREFIX"
    log "Rolled back to previous version."
  fi
  exit 1
fi
```

### 6.3 Success Cleanup

```bash
rm -rf "$PREFIX.state/last-known-good"
log "Install complete."
```

---

## 7. AX/UX Specification (Spec92 Compliance)

### 7.1 Progress Output

The installer MUST print progress with:
- `[focusa-install]` prefix for log lines
- Color: blue=info, yellow=warn, red=error
- Clear next-action hints after each phase

### 7.2 Failure Modes (Must Each Have recovery_hint)

| Failure | recovery_hint |
|---------|---------------|
| Network unreachable | `Check internet connection or use --offline with local tarball` |
| Checksum mismatch | `Re-download from https://install.focusa.dev (do not retry from cache)` |
| No init system | `Run focusa-daemon manually or use --no-service` |
| License validation failed | `Purchase at https://focusa.dev or pass --eval for evaluation` |
| Permission denied (symlink) | `Run with sudo or use --prefix ~/.local instead` |
| Unsupported architecture | `Open an issue at https://github.com/Startempire-Wire/focusa/issues with your platform info` |
| Disk full | `Free ${NEEDED}MB or use --prefix to a larger partition` |
| Daemon won't start | `Check journalctl -u focusa-daemon or run focusa-daemon --foreground` |

### 7.3 Dry-Run Mode

`--dry-run` MUST:
- Print what WOULD happen
- Not write any files
- Not modify systemd
- Not call license authority
- Exit 0 with plan summary

---

## 8. Update Mechanism Specification (P1)

### 8.1 `focusa update` Command

When the daemon/CLI is installed, `focusa update` MUST:
1. Query WP REST API for latest stable version: `/wp-json/wpuiai-ai-cloud/v1/releases/latest`
2. Compare to local version
3. If newer: download + verify + atomic swap
4. If same: no-op
5. If older: warn (don't auto-downgrade)

### 8.2 Channel Selection

```bash
focusa update                  # stable channel (default)
focusa update --channel preview
focusa update --channel nightly
```

### 8.3 Rollback on Failed Update

Update MUST rollback to `last-known-good` if post-update smoke tests fail.

---

## 9. Acceptance Criteria

The installer is ready for MVP Cohort when:

### 9.1 Smart Detection (P0)
- [ ] Detects OS via `uname -s` correctly on Linux/Darwin/Windows
- [ ] Detects arch via `uname -m` and maps to correct GitHub asset
- [ ] Detects libc (glibc vs musl) and selects correct asset
- [ ] Detects init system and deploys correct service file
- [ ] Detects existing install and runs upgrade vs fresh install

### 9.2 Binary Selection (P0)
- [ ] Downloads `focusa`, `focusa-daemon`, `focusa-tui` for detected platform
- [ ] Verifies SHA256 against `SHA256SUMS.txt`
- [ ] Places binaries at `~/.focusa/bin/`
- [ ] Symlinks to `~/.local/bin/` and/or `/usr/local/bin/`

### 9.3 License Authority (P0)
- [ ] Validates `--license-key` against WP REST API
- [ ] Writes `license.json` with `valid: true, tier, expires_at`
- [ ] Honors `--eval` mode (sets `eval: true`)
- [ ] Daemon reads license.json on startup
- [ ] Daemon re-validates every 24h

### 9.4 Atomicity (P0)
- [ ] Stashes existing install before overwrite
- [ ] Rolls back to stash on post-install health check failure
- [ ] Removes stash only on success

### 9.5 AX (P0)
- [ ] All failure modes have actionable `recovery_hint`
- [ ] `--dry-run` previews without writing
- [ ] Progress output with `[focusa-install]` prefix
- [ ] Exit codes: 0=success, 1=install failed (recoverable), 2=license error, 3=unsupported platform

### 9.6 Updates (P1)
- [ ] `focusa update` command works
- [ ] Stable/preview/nightly channels
- [ ] Atomic update with rollback
- [ ] Re-verifies checksum on update

---

## 10. Verification Status (Live as of 2026-06-26)

### Verified Working
- `curl -fsSL https://install.focusa.dev/focusa` returns 234-line installer
- Installer has license validation flag (`--license-key`)
- Installer has eval mode (`--eval`)
- Installer has dry-run flag (`--dry-run`)
- Installer has uninstall flag (`--uninstall`)
- WP REST endpoint `/wp-json/wpuiai-ai-cloud/v1/license/validate` exists
- GitHub releases publish 13 assets per version (3 binaries × 4 platforms + 2 DMGs + 2 .app archives)

### Verified Gaps (Live HTTP Probes)
- ❌ `https://install.focusa.dev/install.ps1` → 404 (no PowerShell installer)
- ❌ `https://install.focusa.dev/install.cmd` → 404 (no Windows cmd installer)
- ❌ `https://install.focusa.dev/install/windows` → 404
- ⚠️ macOS branch in installer says: `"macOS detected; **Phase 1.5: LaunchAgent deployment MANDATORY — no deferral allowed**"` — macOS daemon install is explicitly deferred
- ⚠️ Spec mentions Windows `x86_64-pc-windows-msvc` asset but no native Windows installer exists
- ⚠️ No macOS code signing requirement specified (Apple requires `codesign` for binaries)
- ⚠️ No Windows Service Control Manager (SCM) detection specified

### Not Yet Verified (Gaps)
- No SHA256SUMS.txt in any release
- No GPG signatures
- No `focusa update` command
- Daemon doesn't install (Phase 1.4 stub)
- No systemd unit deployment
- No post-install smoke tests
- No rollback on failure
- No `--channel preview` actually queries WP

### DOES the spec grant detection for local Mac or Windows systems?

**Honest answer: PARTIAL.** 

- ✅ **Spec mentions Mac/Windows** in §2.1 OS Detection, §2.2 Architecture Mapping, §2.3 Init System
- ❌ **No PowerShell installer exists** at install.focusa.dev
- ❌ **macOS daemon install deferred** ("**Phase 1.5: LaunchAgent deployment MANDATORY — no deferral allowed**")
- ❌ **No code signing requirement** specified for macOS
- ❌ **No Windows SCM detection** specified
- ⚠️ **Windows users would need WSL or git-bash** to run the bash installer

For TRUE Mac/Windows support, the spec needs:
1. PowerShell installer (`install.ps1`) with Windows SCM detection
2. macOS codesign requirement in release.yml
3. LaunchAgent plist template for macOS daemon
4. Path differences: `/usr/local/bin/` (Unix) vs `%ProgramFiles%` (Windows)

---

## 11. Implementation Roadmap

### Phase 1.5 — Real Binary Install (P0)

Canonical implementation architecture is **Rust-orchestrated**; see §15.

- Replace Python stub with real Rust binary download
- **Add `focusa install` Rust subcommand** at `crates/focusa-cli/src/commands/install.rs` — the single canonical install orchestrator
- **Slim `scripts/install-focusa.sh` and `scripts/install-focusa.ps1`** to thin bootstrappers: detect → download → SHA256SUMS verify → place on PATH → `exec focusa install --target=<detected>`
- Add SHA256SUMS verification (in Rust)
- Add daemon systemd unit deployment via existing `crates/focusa-cli/src/commands/service.rs`
- Add post-install `focusa doctor` smoke test
- **macOS:** LaunchAgent plist deployment via existing `service::run_launchd_user` (NOT deferred, real in Phase 1.5)
- **macOS:** Add `codesign` requirement to release.yml (Apple requires signed binaries)
- **Windows:** Deploy `install.focusa.dev/install.ps1` (currently 404) — boot to `focusa install --target=windows-<arch>`
- **Windows:** Windows SCM `sc.exe` service deployment via same Rust `service` module
- **Windows:** Detect if running under WSL/git-bash and fall back to bash bootstrapper

### Phase 2.0 — Update + Rollback (POST-MVP — not blocking)
- Implement `focusa update` command (Linux + Mac + Windows)
- Add atomic swap with rollback
- Add `--channel preview` support

### Phase 2.5 — Signing (POST-MVP — not blocking)
- Add cosign signing to release.yml
- Verify signatures on install
- Publish `SHA256SUMS.txt.sig`
- **macOS:** Verify Developer ID signature (Apple Gatekeeper)
- **Windows:** Verify Authenticode signature

### Phase 3.0 — Package Managers (POST-MVP — not blocking)
- Build `.deb` and `.rpm` packages in CI (Linux)
- Build `.pkg` installer for macOS (Homebrew Cask)
- Build `.msi` for Windows (winget)
- Publish to apt/winget/brew repositories

---

## 12. Open Questions

1. **License revocation latency**: How quickly does daemon detect license revocation? (Spec: 24h re-validation, but is that acceptable?)
2. **Multi-user eval**: Does `--eval` allow multiple users on same machine? (Currently ambiguous)
3. **Daemon license enforcement**: Does daemon refuse to start with invalid license, or just emit warnings?
4. **Upgrade interruption**: What happens if upgrade is interrupted mid-install? (Need atomic swap with rollback)
5. **Channel stability**: Does `preview` channel break API stability guarantees? (Spec92 implications)

---

## 13. Bead Reference

- `focusa-iqqi` — PORTABILITY: Real architecture needed beyond shell scripts. **NOW ACTIVE Phase 1.5 work** — see §15 for the canonical implementation architecture (Rust `install` subcommand orchestrating thin shell bootstrapper)
- `focusa-7wgk` — Tier2-2: Fresh-operator dry-run on a clean VPS
- `focusa-cme3` — Tier2-4: Tauri release artifacts in release.yml
- `focusa-foyr` — closes when shell installer delegates to Rust `service` module (work is half-done; daemon systemd unit rendered in Rust, shell still duplicates)
- The shell-installer codepaths (`scripts/install-focusa.sh`, `scripts/install-focusa.ps1`) are now bootstrappers, not orchestrators — see §15
---

## 14. Complete Cross-Platform Compatibility Specification

**This section specifies the installer must work for ALL compatible systems identified in §13 (Acceptance Criteria matrix).**

### 14.1 Installer Variant Strategy

The Focusa installer ships as **TWO variants** to cover all compatible systems:

| Variant | URL | Targets |
|---------|-----|---------|
| POSIX bash | `https://install.focusa.dev/focusa` | Linux, macOS, WSL, Git Bash |
| PowerShell | `https://install.focusa.dev/focusa.ps1` | Windows native, PowerShell Core |

The Windows installer is **NOT optional** — Windows users without WSL must have a native installer.

### 14.2 PowerShell Installer Specification (`focusa.ps1`)

```powershell
<#
.SYNOPSIS
  Focusa Installer for Windows
.DESCRIPTION
  Downloads, verifies, and installs Focusa daemon + CLI + TUI on Windows.
  Detects architecture, chooses correct asset, validates SHA256SUMS,
  integrates with WordPress license authority, deploys as SCM service.
.PARAMETER Eval
  Install in evaluation mode (no commercial use)
.PARAMETER LicenseKey
  Activate a Focusa license key (focusa_live_xxxxx)
.PARAMETER Channel
  stable | preview (default: stable)
.PARAMETER Prefix
  Install prefix (default: $env:LOCALAPPDATA\Programs\Focusa)
.PARAMETER DryRun
  Show what would happen; do not write
.PARAMETER Uninstall
  Remove installed files
.PARAMETER NoService
  Do not install Windows SCM service
.PARAMETER AcceptLicense
  Accept BSL 1.1
#>
```

#### Detection Logic (PowerShell)

```powershell
# OS detection
$IsWindows = [System.Environment]::OSVersion.Platform -eq "Win32NT"
$PSVersion = $PSVersionTable.PSVersion
$Arch = if ([System.Environment]::Is64BitOperatingSystem) { "x86_64" } else { "x86" }
$AssetSuffix = "x86_64-pc-windows-msvc"

# Channel/version discovery from WP REST API
$LatestVersion = Invoke-RestMethod `
    -Uri "https://install.focusa.dev/wp-json/wpuiai-ai-cloud/v1/releases/latest" `
    -Headers @{ "X-Channel" = $Channel }

# Asset URL construction
$AssetBase = "https://github.com/Startempire-Wire/focusa/releases/download/v$LatestVersion"
$Assets = @{
    "focusa"        = "$AssetBase/focusa-v$LatestVersion-$AssetSuffix.exe"
    "focusa-daemon"  = "$AssetBase/focusa-daemon-v$LatestVersion-$AssetSuffix.exe"
    "focusa-tui"     = "$AssetBase/focusa-tui-v$LatestVersion-$AssetSuffix.exe"
}

# Checksum verification
$Sha256Sums = Invoke-RestMethod -Uri "$AssetBase/SHA256SUMS.txt"
foreach ($asset in $Assets.GetEnumerator()) {
    $expectedHash = ($Sha256Sums | Select-String $asset.Key).ToString().Split(" ")[0]
    $actualHash = (Get-FileHash $asset.Value -Algorithm SHA256).Hash
    if ($expectedHash -ne $actualHash) {
        throw "Checksum mismatch for $($asset.Key)"
    }
}

# Code signing verification (Authenticode)
foreach ($asset in $Assets.GetEnumerator()) {
    $signature = Get-AuthenticodeSignature $asset.Value
    if ($signature.SignerCertificate.Subject -notmatch "Startempire Wire") {
        throw "Authenticode signature missing or invalid for $($asset.Key)"
    }
}

# SCM service deployment
$svcName = "focusa-daemon"
$svcPath = Join-Path $Prefix "bin\focusa-daemon.exe"
New-Service -Name $svcName -BinaryPathName $svcPath `
    -DisplayName "Focusa Daemon" `
    -Description "Local-first cognitive runtime for agent continuity" `
    -StartupType Automatic
Start-Service $svcName
```

#### Path Strategy (Windows)

```
%LOCALAPPDATA%\Programs\Focusa\        # user install (no admin)
  ├── bin\
  │   ├── focusa.exe
  │   ├── focusa-daemon.exe
  │   └── focusa-tui.exe
  ├── state\
  ├── config\
  └── libexec\

HKEY_CURRENT_USER\...\PATH             # add bin to user PATH
```

### 14.3 macOS Detection Specification

The bash installer at `install.focusa.dev/focusa` MUST detect macOS:

```bash
# macOS detection
if [ "$(uname -s)" = "Darwin" ]; then
  IS_MACOS=1
  MACOS_ARCH="$(uname -m)"  # arm64 or x86_64
  MACOS_VERSION="$(sw_vers -productVersion 2>/dev/null || echo 0)"
  
  # Apple Silicon vs Intel
  if [ "$MACOS_ARCH" = "arm64" ]; then
    ASSET_SUFFIX="aarch64-apple-darwin"
    SYMLINK_PATH="/opt/homebrew/bin/focusa"  # Apple Silicon default
  else
    ASSET_SUFFIX="x86_64-apple-darwin"
    SYMLINK_PATH="/usr/local/bin/focusa"  # Intel default
  fi
  
  # macOS version check (reject < Big Sur 11.0)
  MAJOR_VERSION=$(echo "$MACOS_VERSION" | cut -d. -f1)
  if [ "$MAJOR_VERSION" -lt 11 ]; then
    die "macOS $MACOS_VERSION < 11.0 not supported. Update to macOS 11+ (Big Sur or later)."
  fi
  
  # Code signing verification
  for binary in focusa focusa-daemon focusa-tui; do
    SIGNER=$(codesign -dv "$PREFIX/bin/$binary" 2>&1 | grep "Developer ID" | head -1)
    if [ -z "$SIGNER" ]; then
      die "Binary $binary is not signed. Run codesign manually or contact support."
    fi
  done
  
  # Gatekeeper / notarization
  for binary in focusa focusa-daemon focusa-tui; do
    QUARANTINE=$(xattr -p com.apple.quarantine "$PREFIX/bin/$binary" 2>/dev/null || true)
    if [ -n "$QUARANTINE" ]; then
      xattr -d com.apple.quarantine "$PREFIX/bin/$binary"
    fi
  done
fi
```

#### macOS LaunchAgent Deployment (NOT DEFERRED)

```bash
# LaunchAgent plist for focusa-daemon
cat > "$HOME/Library/LaunchAgents/com.startempire.focusa-daemon.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>com.startempire.focusa-daemon</string>
  <key>ProgramArguments</key>
  <array>
    <string>$PREFIX/bin/focusa-daemon</string>
  </array>
  <key>RunAtLoad</key>
  <true/>
  <key>KeepAlive</key>
  <true/>
  <key>StandardOutPath</key>
  <string>$HOME/Library/Logs/focusa-daemon.log</string>
  <key>StandardErrorPath</key>
  <string>$HOME/Library/Logs/focusa-daemon.err.log</string>
</dict>
</plist>
PLIST

# Load (NOT skip — the current installer explicitly skips this)
launchctl load -w "$HOME/Library/LaunchAgents/com.startempire.focusa-daemon.plist"
```

### 14.4 Linux Detection (Existing)

The bash installer already handles Linux correctly. This section documents the **baseline** that macOS and Windows must match:

```bash
# Linux detection
if [ "$(uname -s)" = "Linux" ]; then
  IS_LINUX=1
  ARCH="$(uname -m)"
  # glibc vs musl detection
  if ldd --version 2>&1 | grep -q musl; then
    LIBC="musl"
  else
    LIBC="gnu"
  fi
  ASSET_SUFFIX="${ARCH}-unknown-linux-${LIBC}"
  
  # systemd vs user systemd
  if systemctl --user status >/dev/null 2>&1; then
    SYSTEMD_SCOPE="user"
  elif systemctl status >/dev/null 2>&1; then
    SYSTEMD_SCOPE="system"
  fi
  
  # WSL detection (fall back to bash, note as WSL)
  if grep -qi microsoft /proc/version 2>/dev/null; then
    IS_WSL=1
    warn "Running on Windows via WSL. For native Windows install, use PowerShell installer."
  fi
fi
```

### 14.5 Dispatch Matrix (Smart Detection)

The POSIX bash installer MUST dispatch based on detection:

```bash
case "$OS" in
  Linux)
    install_linux
    ;;
  Darwin)
    install_macos
    ;;
  MINGW*|CYGWIN*|MSYS*)
    warn "Bash detected on Windows. Recommend PowerShell installer."
    warn "Falling back to bash mode — daemon will run as foreground process."
    install_bash_on_windows
    ;;
  *)
    die "Unsupported OS: $OS. See https://install.focusa.dev for supported platforms."
    ;;
esac
```

### 14.6 Verification Status — All Compatible Systems

| System | Detection | Asset | Service | Verify | Status |
|--------|-----------|-------|---------|--------|--------|
| Linux x86_64 glibc | ✅ | ✅ exists | ✅ systemd | ✅ spec'd | 🟡 needs PowerShell fallback if WSL |
| Linux x86_64 musl | ✅ | ✅ exists | ✅ systemd | ✅ spec'd | 🟡 asset verification |
| Linux aarch64 glibc | ✅ | ✅ exists | ✅ systemd | ✅ spec'd | 🟢 ready |
| Linux aarch64 musl | ❌ | ❌ | ✅ systemd | ✅ spec'd | 🟡 asset missing in release |
| macOS Intel | ✅ | ✅ exists | ❌ **Phase 1.5** | ✅ spec'd | 🔴 LaunchAgent deployment MUST be Phase 1.5 |
| macOS Apple Silicon | ✅ | ✅ exists | ❌ **Phase 1.5** | ✅ spec'd | 🔴 LaunchAgent deployment MUST be Phase 1.5 |
| Windows x86_64 | ❌ | ✅ exists | ❌ | ❌ | 🔴 **NO INSTALLER** (404 on /install.ps1) |
| Windows WSL | ✅ | ✅ | ❌ | ✅ spec'd | 🟡 bash works but no SCM service |

### 14.7 Compatibility Test Matrix (Spec92)

For each compatible system, the following MUST be tested in CI:

| Test | Linux x64 | Linux arm64 | macOS Intel | macOS AS | Windows x64 |
|------|-----------|-------------|-------------|----------|-------------|
| Detect OS | ✅ | ✅ | ✅ | ✅ | ✅ |
| Detect arch | ✅ | ✅ | ✅ | ✅ | ✅ |
| Detect libc | ✅ | ✅ | n/a | n/a | n/a |
| Detect init | ✅ systemd | ✅ systemd | ⏳ launchd | ⏳ launchd | ⏳ SCM |
| Download correct asset | ✅ | ✅ | ✅ | ✅ | ✅ |
| Verify SHA256 | ⏳ spec | ⏳ spec | ⏳ spec | ⏳ spec | ⏳ spec |
| Verify codesign | n/a | n/a | ⏳ spec | ⏳ spec | ⏳ spec |
| Install binary | ⏳ | ⏳ | ⏳ | ⏳ | ⏳ |
| Deploy service | ⏳ | ⏳ | ⏳ Phase 1.5 | ⏳ Phase 1.5 | ⏳ |
| License validation | ✅ | ✅ | ✅ | ✅ | ✅ |
| Smoke test | ⏳ spec | ⏳ spec | ⏳ spec | ⏳ spec | ⏳ spec |
| `focusa update` | ⏳ | ⏳ | ⏳ | ⏳ | ⏳ |

Legend: ✅ = working, ⏳ = spec'd not yet implemented, ❌ = missing

### 14.8 Smart Detection Priorities (Order of Operations)

The installer MUST detect in this order:

1. **Shell type** (bash vs PowerShell) → chooses installer variant
2. **OS family** (Linux/Darwin/Windows/WSL) → chooses install strategy
3. **Architecture** (x86_64/arm64/aarch64) → chooses asset
4. **libc** (glibc/musl on Linux only) → refines asset selection
5. **Init system** (systemd/launchd/SCM/none) → chooses service manager
6. **Existing install** (upgrade vs fresh) → chooses upgrade path
7. **Privileges** (root/user) → chooses install prefix
8. **PATH layout** → chooses symlink location
9. **Dependencies** (curl/wget/tar/sha256sum) → validates pre-flight
10. **Disk space** → checks capacity

### 14.9 Cross-Platform Fallback Chain

When a system is partially supported:

```
[1] Try primary install path
    ↓ fail?
[2] Try documented fallback (e.g., WSL bash mode, musl→glibc swap)
    ↓ fail?
[3] Print actionable recovery_hint with link to docs
    e.g.: "Windows detected but no install.ps1 at install.focusa.dev.
           Open issue at https://github.com/Startempire-Wire/focusa/issues
           with your platform: Windows 11 x64"
    ↓ fail?
[4] Exit with non-zero code
```

### 14.10 PowerShell Installer — Full Script Outline

The `focusa.ps1` MUST implement:

```powershell
# Detect platform
$OS = [System.Environment]::OSVersion
$Arch = [System.Environment]::Is64BitOperatingSystem ? "x86_64" : "x86"

# Resolve asset suffix
$AssetSuffix = switch ($Arch) {
    "x86_64" { "x86_64-pc-windows-msvc" }
    "arm64"  { "aarch64-pc-windows-msvc" }  # Windows on ARM
    default  { throw "Unsupported arch: $Arch" }
}

# Resolve install prefix
$Prefix = if ($Prefix) { $Prefix } else {
    if (Test-IsAdmin) { "${env:ProgramFiles}\Focusa" }
    else { "${env:LOCALAPPDATA}\Programs\Focusa" }
}

# Create directories
New-Item -Path $Prefix -ItemType Directory -Force
New-Item -Path "$Prefix\bin" -ItemType Directory -Force
New-Item -Path "$Prefix\state" -ItemType Directory -Force
New-Item -Path "$Prefix\config" -ItemType Directory -Force

# Download + verify + install (each binary)
foreach ($binary in @("focusa", "focusa-daemon", "focusa-tui")) {
    $url = "https://github.com/.../releases/download/v$Version/$binary-v$Version-$AssetSuffix.exe"
    $path = "$Prefix\bin\$binary.exe"
    Invoke-WebRequest -Uri $url -OutFile $path
    
    # Verify SHA256
    $expected = (Get-Content "$tmp\SHA256SUMS.txt" | Where { $_ -match $binary }).Split()[0]
    $actual = (Get-FileHash $path -Algorithm SHA256).Hash
    if ($expected -ne $actual) { throw "Checksum mismatch: $binary" }
    
    # Verify Authenticode signature
    $sig = Get-AuthenticodeSignature $path
    if ($sig.SignerCertificate.Subject -notmatch "Startempire Wire") {
        throw "Authenticode signature invalid: $binary"
    }
}

# SCM service (if not --no-service)
if (-not $NoService) {
    $svcPath = "$Prefix\bin\focusa-daemon.exe"
    New-Service -Name "focusa-daemon" -BinaryPathName $svcPath -StartupType Automatic
    Start-Service "focusa-daemon"
}

# PATH
$currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($currentPath -notlike "*$Prefix\bin*") {
    [Environment]::SetEnvironmentVariable("PATH", "$currentPath;$Prefix\bin", "User")
}

# Smoke test
& "$Prefix\bin\focusa.exe" doctor --json | ConvertFrom-Json | ForEach-Object {
    if ($_.status -ne "ok") { throw "Smoke test failed" }
}

Write-Host "Focusa installed at $Prefix"
Write-Host "Run: '$Prefix\bin\focusa.exe doctor' to verify"
```

### 14.11 Updated Roadmap (After This Spec)

**Phase 1.5 (Critical — blocks MVP Cohort):**
- [ ] Create `install.focusa.dev/focusa.ps1` (Windows PowerShell installer)
- [ ] Implement macOS LaunchAgent deployment (MUST be Phase 1.5 (real installer currently defers))
- [ ] Add SHA256SUMS.txt to all GitHub releases
- [ ] Add Apple Developer ID signing in release.yml
- [ ] Add Authenticode signing in release.yml
- [ ] Add `focusa doctor` smoke test post-install
- [ ] Implement smart detection §14.5

**Phase 1.6 (Linux/macOS service completeness):**
- [ ] systemd unit deployment (Linux)
- [ ] LaunchAgent plist deployment (macOS — Phase 1.5, NOT deferred)
- [ ] SCM service deployment (Windows)

**Phase 2.0 (Update + Rollback):**
- [ ] `focusa update` command for all platforms
- [ ] Atomic swap with rollback

**Phase 2.5 (Signing):**
- [ ] GPG/cosign (Linux)
- [ ] Apple notarization (macOS)
- [ ] Authenticode (Windows)

**Phase 3.0 (Package Managers):**
- [ ] apt (Debian/Ubuntu)
- [ ] dnf (Fedora/RHEL)
- [ ] Homebrew (macOS)
- [ ] winget (Windows)

---

## 15. Updated Acceptance Criteria (All Compatible Systems)

The installer is ready for MVP Cohort when ALL of these are ✅:

### 15.1 Detection (All Platforms)
- [ ] Linux x86_64 glibc → installs `focusa-x86_64-unknown-linux-gnu`
- [ ] Linux x86_64 musl → installs `focusa-x86_64-unknown-linux-musl`
- [ ] Linux aarch64 glibc → installs `focusa-aarch64-unknown-linux-gnu`
- [ ] Linux aarch64 musl → installs `focusa-aarch64-unknown-linux-musl` (asset missing in release)
- [ ] macOS Intel → installs `focusa-x86_64-apple-darwin`
- [ ] macOS Apple Silicon → installs `focusa-aarch64-apple-darwin`
- [ ] Windows x86_64 → installs `focusa-x86_64-pc-windows-msvc` (no installer currently)
- [ ] Windows ARM64 → installs `focusa-aarch64-pc-windows-msvc` (asset missing)
- [ ] WSL → falls back to bash installer with warning
- [ ] Linux without systemd → warns, suggests `--no-service`
- [ ] macOS < 11 → rejected with version requirement

### 15.2 Service Deployment (All Platforms)
- [ ] Linux systemd (system or user scope)
- [ ] macOS LaunchAgent (Phase 1.5 — mandatory)
- [ ] Windows SCM (New-Service) (currently no installer)

### 15.3 Code Signing (All Platforms)
- [ ] Linux: GPG or cosign signature verified on download
- [ ] macOS: Apple Developer ID signature verified via codesign
- [ ] Windows: Authenticode signature verified via Get-AuthenticodeSignature

### 15.4 License Authority (All Platforms)
- [ ] All platforms validate against `https://install.focusa.dev/wp-json/wpuiai-ai-cloud/v1/license/validate`
- [ ] All platforms write `license.json` to platform-appropriate config dir:
  - Linux: `~/.config/focusa/license.json`
  - macOS: `~/.config/focusa/license.json` (XDG-style)
  - Windows: `%APPDATA%\Focusa\license.json`

### 15.5 Updates (All Platforms)
- [ ] `focusa update` works on Linux
- [ ] `focusa update` works on macOS
- [ ] `focusa update` works on Windows
- [ ] All update channels (stable/preview) work
- [ ] Rollback on failed update

### 15.6 AX (All Platforms)
- [ ] All failure modes have `recovery_hint`
- [ ] `--dry-run` works on all platforms
- [ ] Progress output uses platform-appropriate terminal

---

## 15A. Implementation Architecture (Canonical Phase 1.5)

**Status:** Authoritative. Supersedes any prior §3 / §5A / §7 / §14 implementation examples that show shell logic. The user-facing install surface (`curl | bash`, `irm | iex`) is preserved per §1 and §14.1; this section specifies the **internal** architecture.

### 15A.1 Canonical entry: `focusa install`

The installer is implemented as a **Rust subcommand** at `crates/focusa-cli/src/commands/install.rs`. All install logic — license validation, asset download, SHA256SUMS verification, symlink placement, service rendering, atomicity, rollback, dry-run, recovery hints — lives in Rust and is unit-testable.

The bash and PowerShell installers (`install.focusa.dev/focusa`, `install.focusa.dev/focusa.ps1`) are **thin bootstrappers** whose sole job is:

1. Detect OS / arch / libc
2. Download the `focusa` binary from GitHub releases
3. Verify SHA256SUMS
4. Place `focusa` on `PATH`
5. `exec focusa install --target=<detected>` and exit

After step 5, the shell's job is done. All install work flows through Rust.

### 15A.2 License authority coupling

`focusa install` validates the license via the existing
`crates/focusa-cli/src/commands/license.rs::registry_validate()`, which already POSTs to
`https://install.focusa.dev/wp-json/wpuiai-ai-cloud/v1/license/validate`.
License JSON is written to `~/.config/focusa/license.json` with the exact
shape `crates/focusa-core/src/license.rs::load_license_status()` reads
(must be audited for field parity before Rust goes canonical).

`focusa install` always re-validates (even if shell pre-validated) so a
revoked license cannot slip through.

### 15A.3 Service rendering delegation

`focusa install` delegates service unit / LaunchAgent rendering to the
existing `crates/focusa-cli/src/commands/service.rs`:

- Linux → `service::run_systemd_user` (renders `~/.config/systemd/user/focusa-daemon.service`)
- macOS → `service::run_launchd_user` (renders `~/Library/LaunchAgents/com.startempire.focusa-daemon.plist` + `launchctl load -w`)
- Windows → `service::run_scm` (renders `sc.exe` registration; future work, Phase 2.0)

The shell installer MUST NOT duplicate service rendering. This is the
bead `focusa-foyr` closure condition.

### 15A.4 PowerShell parity

`install.focusa.dev/focusa.ps1` follows the same pattern:

1. Detect Windows native vs WSL/git-bash
2. Download + SHA256SUMS verify
3. `& focusa install --target=windows-<arch>` and exit

WSL/git-bash fall back to the bash bootstrapper, which uses the same
final `exec focusa install` flow.

### 15A.5 Acceptance (architectural)

- [ ] `focusa install --target=<linux|darwin|windows-x64|windows-arm64>` exists and is the only place install orchestration lives
- [ ] `scripts/install-focusa.sh` and `scripts/install-focusa.ps1` are each ≤ 100 lines, with no logic beyond detect / download / verify / exec
- [ ] Unit tests cover `focusa install` end-to-end for each target
- [ ] `focusa-foyr` and `focusa-3cok` closed (shell installer no longer renders service units directly)
- [ ] License JSON shape parity audit passes (`focusa-cli::commands::license::RegistryValidateResponse` ↔ `focusa-core::license::LicenseStatus`)

### 15A.6 PATH automation + first install walkthrough

After `focusa install` finishes, the operator must be able to run `focusa --version`
in a fresh shell without surprises. Two gaps caused evaluators to feel lost
post-install: (a) the binary lived at `~/.focusa/bin/focusa` but PATH
did not include it, and (b) no human or agent first-install guide was emitted.

**PATH automation (mandatory):**

- [ ] After symlink placement, `focusa install` MUST verify that `focusa` is reachable
      via the user's current shell PATH on the target shell family (`bash`, `zsh`, `fish`, `sh`,
      `pwsh`).
- [ ] If not reachable: print a structured recovery_hint containing the EXACT
      `export PATH=...` line for the target shell family (POSIX variants use
      `export PATH="$HOME/.local/bin:$PATH"`; Windows Pwsh uses
      `$env:PATH = "$env:USERPROFILE\.local\bin;$env:PATH"`).
- [ ] Persist the PATH addition to the user's shell rc file when interactive
      (`~/.bashrc`, `~/.zshrc`, `~/.config/fish/config.fish`) and skip silently
      for non-interactive CI / agent contexts. Detection via `$SHELL` +
      `[ -t 1 ]` or `--persist-path` / `--no-persist-path` flags.
- [ ] Idempotent: never duplicate the PATH line on re-run.

**First-install walkthrough (mandatory for MVP Cohort):**

- [ ] `focusa install --on-shell <bash|zsh|fish|pwsh>` MUST emit a structured
      next-steps card on success. Both forms print **inline to the same
      terminal session** where `focusa install` was invoked — no separate
      wizard TUI, no browser redirect, no detached process. The operator
      sees one CLI log out of which they can copy commands directly.
      Two forms:
      - **Human form**: ASCII card with 6 actionable steps: (1) PATH now on,
        (2) verify with `focusa --version`, (3) start daemon with
        `focusa start`, (4) verify daemon with `focusa doctor`, (5) approve
        a test pairing flow with `focusa pair --dry-run`,
        (6) read docs at https://focusa.dev/start. Each step has expected
        outcome and recovery_hint.
      - **Agent form**: `--json` flag emits a `first_install_walkthrough.v1`
        envelope with `next_steps[]`, each containing `command`, `intent`,
        `expected_outcome`, `recovery_hint`, plus an `environment_summary`
        block listing PATH, version, daemon status, license status,
        scope_key, and a `recovery_hint_root` array. Output goes to stdout
        in JSON; nothing else is emitted.
- [ ] The agent envelope is the bridge into Spec 111 preload artifacts so an
      agent bootstrapped after install has what it needs without re-running
      the install.

**Acceptance: launch a fresh shell, run `focusa install`, then run `focusa doctor`,
and the operator must NOT need to manually edit PATH or guess the next command.**

### 15A.7 Multi-agent integration (real deployed for ALL coding agents)

After `focusa install` succeeds, the operator must be in a **working state
inside whichever coding agent they use**, not just inside their shell.
The 1st-install contract must cover Pi (already supported), Cursor,
Continue, Cline, OpenCode, Aider, Claude Code, and any future MCP-compatible
agent. An evaluator using Cursor must be able to install focusa and have
their agent immediately able to call focusa tools — no further setup.

**Agent detection (mandatory):**

- [ ] `focusa install` MUST detect which coding agents are installed on the
      host (Pi, Cursor, Continue, Cline, OpenCode, Aider, Claude Code,
      Antigravity, Zed, Windsurf, others).
      Zed, Windsurf, others). Detection is **read-only** — never auto-install
      a new agent.
- [ ] Detection heuristics live in `crates/focusa-cli/src/agents/detect.rs`
      and check for known config paths under `~/.config/<agent>/`,
      `~/.<agent>/`, `~/Library/Application Support/<agent>/`, and the host's
      `$PATH` for agent CLI binaries.

**MCP integration (universal):**

- [ ] `focusa install` MUST emit a **machine-readable MCP config block**
      (`focusa.mcp.v1`) listing the daemon URL (`http://127.0.0.1:8787/mcp`)
      plus the `tool_manifest` URL and supported capability scopes. Any
      MCP-compatible agent (Cursor, Continue, Cline, Claude Code, OpenCode)
      can drop this block into its `mcp.json` / `claude_desktop_config.json`
      / `.cursor/mcp.json` and immediately call focusa tools.
- [ ] `focusa install --agent <pi|cursor|continue|cline|opencode|aider|claude-code>`
      emits the MCP config block formatted for that specific agent's config
      file path (with comment-anchored placement so re-runs are idempotent).

**VS Code/Cursor extension (dedicated):**

- [ ] `apps/vscode-extension/` — VS Code/Cursor extension package that:
      - Reads focusa daemon URL + license state on startup
      - Registers as an MCP server so Cursor's @-mention tool surface uses focusa
      - Provides a sidebar status bar item (green/yellow/red)
      - Offers commands `Focusa: Doctor`, `Focusa: Pair`, `Focusa: Show Workpoint`
- [ ] Extension bundles `focusa-daemon-mcp` shim that proxies the daemon's
      `/mcp` endpoint as a stdio MCP server for clients that prefer stdio.

**Pi extension parity (already shipped):**

- [ ] Existing `apps/pi-extension/` integration is preserved and remains the
      canonical Pi integration. Verify still works post-slim.

**Agent-aware walkthrough (extends §15A.6):**

- [ ] First-install walkthrough MUST include agent-specific steps when agents
      are detected. For Cursor: "Open Cursor, then Ctrl+Shift+P → 'Developer:
      Reload Window' to activate the focusa MCP server." For Pi: "Pi auto-loads
      apps/pi-extension on next session; nothing to do."
- [ ] The agent envelope (`first_install_walkthrough.v1`) gains an
      `agent_integrations[]` array listing each detected agent, its
      integration status, the next-step command for that agent, and the
      expected outcome.

**Acceptance: install focusa on a host with Cursor and Pi. Launch Cursor,
type `@focusa doctor` (or equivalent), and the agent must successfully call
`focusa_daemon_doctor` without operator setup. Same for Pi in a fresh shell.**

### 15A.8 Why this architecture

- Single source of truth — no logic drift between bash, PowerShell, and Rust
- Unit-testable installer logic (today's bash is untested)
- Atomicity (§6) and update mechanism (§8) compose naturally with the Rust Cargo build/test/release pipeline
- Windows / Linux / macOS all share the typed `RegistryValidateResponse`, `LicenseStatus`, `ServiceManager` Rust types
- Bead `focusa-iqqi` (PORTABILITY) is the canonical name for this work; `focusa-foyr` (systemd installer) closes when the shell installer stops duplicating systemd rendering

---

## 16. Summary of Gaps Per Platform

### Linux
- 🟢 Asset: ✅ All 4 variants (gnu/musl × x86_64/aarch64) exist
- 🟢 systemd: ✅ spec'd
- 🟢 Signing: ✅ GPG/cosign spec'd
- 🟢 License: ✅ WP REST API spec'd
- 🟢 Update: ✅ spec'd

### macOS
- 🟢 Asset: ✅ Both Intel + Apple Silicon exist
- 🟡 LaunchAgent: ❌ DEFERRED in real installer — MUST FIX (Phase 1.5)
- 🔴 Code signing: ❌ Not in spec (Apple Developer ID required)
- 🟢 License: ✅ WP REST API spec'd
- 🟡 Update: ✅ spec'd but not implemented

### Windows
- 🔴 Asset: ✅ x86_64 exists, ARM64 missing
- 🔴 Installer: ❌ NO PowerShell installer (404)
- 🔴 SCM service: ❌ Not in spec
- 🔴 Code signing: ❌ Not in spec (Authenticode required)
- 🟢 License: ✅ WP REST API spec'd
- 🔴 Update: ❌ No installer to update

---

## 17. Bead Reference

- `focusa-iqqi` — Updated to reflect comprehensive cross-platform spec
- `focusa-cqhi` — Menubar Tauri local artifact
- `focusa-cme3` — Tauri release artifacts
- All MVP-launch blockers until installer variants shipped

---

## 18. Reconciliation: Authoritative Acceptance Criteria (Replaces §9 and §15)

**Section 9 and §15 contradicted each other. This section is the SINGLE authoritative list.**

### 18.1 Detection (P0 — blocks MVP Cohort)

- [ ] **Linux x86_64 glibc** → detects via `uname -m` + `ldd --version`, installs `focusa-x86_64-unknown-linux-gnu`
- [ ] **Linux x86_64 musl** → detects musl libc, installs `focusa-x86_64-unknown-linux-musl` (**ASSET MISSING — see §19 Phase 1.5 P0**)
- [ ] **Linux aarch64 glibc** → detects via `uname -m`, installs `focusa-aarch64-unknown-linux-gnu`
- [ ] **Linux aarch64 musl** → detects musl libc, installs `focusa-aarch64-unknown-linux-musl` (**ASSET MISSING — see §19 Phase 1.5 P0**)
- [ ] **macOS Intel** → detects `uname -s=Darwin` + `uname -m=x86_64`, installs `focusa-x86_64-apple-darwin`
- [ ] **macOS Apple Silicon** → detects `uname -m=arm64`, installs `focusa-aarch64-apple-darwin`
- [ ] **macOS < 11** → rejected with version requirement message
- [ ] **Windows x86_64** → installs `focusa-x86_64-pc-windows-msvc` via `focusa.ps1` (**INSTALLER MISSING — see §19 Phase 1.5 P0**)
- [ ] **Windows ARM64** → installs `focusa-aarch64-pc-windows-msvc` (**ASSET MISSING — see §19 Phase 1.5 P0**)
- [ ] **WSL** → falls back to bash installer with explicit warning
- [ ] **Linux without systemd** → `--no-service` flag works
- [ ] **macOS without launchd** → daemon still installs (not blocked)

### 18.2 Binary Download (P0)

- [ ] Installer queries GitHub API for latest stable release: `https://api.github.com/repos/Startempire-Wire/focusa/releases/latest`
- [ ] **CRITICAL FIX (A1):** Installer downloads `focusa`, `focusa-daemon`, `focusa-tui` from release (NOT Python stub)
- [ ] Detects user's OS+arch+libc and picks correct asset

### 18.3 Verification (P0)

- [ ] **CRITICAL FIX (A7):** `SHA256SUMS.txt` published in every release
- [ ] Installer verifies each downloaded asset: `sha256sum -c --ignore-missing SHA256SUMS.txt`
- [ ] **CRITICAL FIX (A6):** macOS binaries verified via `codesign -dv` (Developer ID Application: Startempire Wire Inc.)
- [ ] **CRITICAL FIX (A6):** macOS Gatekeeper quarantine removed: `xattr -d com.apple.quarantine`
- [ ] **CRITICAL FIX (A8):** GPG/cosign signature verification for Linux
- [ ] **CRITICAL FIX (A8):** Authenticode verification for Windows: `Get-AuthenticodeSignature`

### 18.4 Service Deployment (P0)

- [ ] **CRITICAL FIX (A10):** Linux systemd unit installed: `/etc/systemd/system/focusa-daemon.service`
- [ ] **CRITICAL FIX (A5):** macOS LaunchAgent deployed: `~/Library/LaunchAgents/com.startempire.focusa-daemon.plist` (NO deferral)
- [ ] **CRITICAL FIX (A9):** Windows SCM service: `New-Service focusa-daemon -BinaryPathName ...`

### 18.5 License Authority (P0)

- [ ] **CRITICAL FIX (A4):** License info actually written AND consumed by daemon
- [ ] License endpoint validated: `POST /wp-json/wpuiai-ai-cloud/v1/license/validate` (returns 200 with `valid: bool`)
- [ ] Tier read correctly from response (Python `json.load` has fallback to `operator` default)
- [ ] Eval mode: `--eval` sets `eval: true` in license.json
- [ ] License re-validation every 24h via daemon scheduler
- [ ] License revocation → daemon enters `license_expired` mode (reads refuse, mutations refuse)
- [ ] License path: Linux/macOS `$HOME/.config/focusa/license.json`, Windows `%APPDATA%\Focusa\license.json`

### 18.6 AX / Recovery Hints (Spec92)

- [ ] All 8 failure modes from §7 have `recovery_hint`
- [ ] `--dry-run` works on all platforms
- [ ] Color codes disabled when stdout is not a TTY (`[ -t 1 ]` check)
- [ ] Cleanup trap set for `/tmp/*.sh` temporary files

### 18.7 Tiers and Features (Resolves B18/B19)

| Tier | Features | Daemon Behavior |
|------|----------|-----------------|
| `evaluation` (eval mode) | daemon, tui (rate limited 100 req/min) | daemon starts with `tier: "evaluation"` in /v1/health |
| `operator` | daemon, tui | daemon starts normally |
| `enterprise` | daemon, tui, menubar | all surfaces available |

If a tier is missing a feature in `features[]` array:
- Daemon refuses to start that feature
- `/v1/doctor` reports `feature_disabled: <feature>`
- Daemon runs in degraded mode (other features work)

### 18.8 Update Mechanism (P1 — not blocking MVP)

- [ ] `focusa update` command works on Linux/macOS/Windows
- [ ] Channel selection: `stable | preview | nightly`
- [ ] Atomic swap with rollback
- [ ] Re-verifies SHA256SUMS on update

### 18.9 Package Managers (P2 — post-MVP)

- [ ] Linux: apt (Debian/Ubuntu), dnf (Fedora/RHEL)
- [ ] macOS: Homebrew Cask
- [ ] Windows: winget, Chocolatey

---

## 19. Phase 1.5 Mandatory Items (MVP Cohort Blocker)

**Per operator rule: "If the system is compatible, we cannot defer support."**

These items MUST ship before MVP Cohort. None may be deferred:

### 19.1 Real Installer Fixes (CRITICAL)

| # | Item | File/Action |
|---|------|-------------|
| 1 | A1: Replace Python stub with real Rust binary download | `install.focusa.dev/focusa` line ~167 |
| 2 | A2: Publish Linux musl assets to release | Add `--target x86_64-unknown-linux-musl` to `release.yml` |
| 3 | A3: Publish Windows ARM64 asset | Add `--target aarch64-pc-windows-msvc` to `release.yml` |
| 4 | A5: Implement macOS LaunchAgent (NO deferral) | Add plist generation + `launchctl load` to bash installer |
| 5 | A6: Add Apple Developer ID signing + Gatekeeper handling | `release.yml` codesign step + installer `codesign -dv` |
| 6 | A7: Publish `SHA256SUMS.txt` | Add sha256sum step to `release.yml` |
| 7 | A8: Add GPG/cosign signing | Add signing step to `release.yml` |
| 8 | A9: Create `install.focusa.dev/install.ps1` | New PowerShell installer file |
| 9 | A10: Add daemon systemd unit deployment | Install `/etc/systemd/system/focusa-daemon.service` |
| 10 | A11: Fix `--with-pi` silent failure | Check Pi presence, error if missing |
| 11 | A12: Implement OpenClaw bridges OR remove `--with-openclaw` flag | Pick one — either ship or remove |
| 12 | A13: Handle all license response codes | Add network error, format error, expired license cases |

### 19.2 Release Pipeline Updates (`release.yml`)

```yaml
# Required new steps in release.yml:
- name: Generate SHA256SUMS
  run: |
    cd release-artifacts
    sha256sum *.exe *.tar.gz *.dmg *.app.tar.gz focusa-* focusa-daemon-* focusa-tui-* > SHA256SUMS.txt

- name: Sign artifacts with cosign
  run: |
    for f in focusa-* focusa-daemon-* focusa-tui-* SHA256SUMS.txt; do
      cosign sign-blob --yes "$f"
    done

- name: Code sign macOS binaries
  if: matrix.platform == 'macos-latest'
  run: |
    for f in focusa-* focusa-daemon-* focusa-tui-*; do
      codesign --sign "Developer ID Application: Startempire Wire Inc." --options runtime "$f"
      codesign --verify --deep --strict "$f"
    done
    # Submit for notarization
    xcrun altool --notarize-app --file "$f" --primary-bundle-id "com.startempire.focusa"

- name: Authenticode sign Windows binaries
  if: matrix.platform == 'windows-latest'
  run: |
    for f in focusa-* focusa-daemon-* focusa-tui-*; do
      signtool sign /fd SHA256 /a /tr http://timestamp.digicert.com "$f"
    done

- name: Add musl targets
  strategy:
    matrix:
      include:
        - target: x86_64-unknown-linux-gnu
        - target: x86_64-unknown-linux-musl    # NEW
        - target: aarch64-unknown-linux-gnu
        - target: aarch64-unknown-linux-musl   # NEW
        - target: x86_64-pc-windows-msvc
        - target: aarch64-pc-windows-msvc       # NEW (Windows on ARM)
```

### 19.3 Daemon Install Locations (Service Files)

**Linux systemd unit** (`/etc/systemd/system/focusa-daemon.service`):
```ini
[Unit]
Description=Focusa Daemon
After=network.target

[Service]
Type=simple
User=%i
ExecStart=/usr/local/bin/focusa-daemon
Restart=on-failure
RestartSec=5
Environment=FOCUSA_DATA_DIR=/var/lib/focusa

[Install]
WantedBy=multi-user.target
```

**macOS LaunchAgent** (`~/Library/LaunchAgents/com.startempire.focusa-daemon.plist`):
```xml
<?xml version="1.0" encoding="UTF-8"?>
<plist version="1.0">
<dict>
  <key>Label</key><string>com.startempire.focusa-daemon</string>
  <key>ProgramArguments</key><array><string>/usr/local/bin/focusa-daemon</string></array>
  <key>RunAtLoad</key><true/>
  <key>KeepAlive</key><true/>
  <key>StandardOutPath</key><string>/tmp/focusa-daemon.out</string>
  <key>StandardErrorPath</key><string>/tmp/focusa-daemon.err</string>
</dict>
</plist>
```

**Windows SCM service** (via PowerShell):
```powershell
New-Service -Name "focusa-daemon" `
  -BinaryPathName "C:\Program Files\Focusa\focusa-daemon.exe" `
  -DisplayName "Focusa Daemon" `
  -StartupType Automatic
Start-Service "focusa-daemon"
```

### 19.4 PowerShell Installer Outline (`focusa.ps1`)

Required to ship to install.focusa.dev/install.ps1:

```powershell
# Detection
$OS = [System.Environment]::OSVersion
$Arch = [System.Environment]::Is64BitOperatingSystem ? "x86_64" : "x86"
$PSVer = $PSVersionTable.PSVersion.Major

# Channel/version discovery (from WP REST API)
$Latest = Invoke-RestMethod "https://install.focusa.dev/wp-json/wpuiai-ai-cloud/v1/releases/latest"

# Asset selection
$AssetSuffix = "x86_64-pc-windows-msvc"
$Assets = @{
    "focusa"       = "focusa-v$($Latest.version)-$AssetSuffix.exe"
    "focusa-daemon" = "focusa-daemon-v$($Latest.version)-$AssetSuffix.exe"
    "focusa-tui"    = "focusa-tui-v$($Latest.version)-$AssetSuffix.exe"
}

# Download + verify
foreach ($name in $Assets.Keys) {
    $url = "https://github.com/Startempire-Wire/focusa/releases/download/v$($Latest.version)/$($Assets[$name])"
    $path = "$Prefix\bin\$name.exe"
    Invoke-WebRequest -Uri $url -OutFile $path

    # Verify SHA256
    $expected = (Get-Content "$tmp\SHA256SUMS.txt" | Where { $_ -match $name }).Split()[0]
    $actual = (Get-FileHash $path -Algorithm SHA256).Hash
    if ($expected -ne $actual) { throw "Checksum mismatch: $name" }

    # Verify Authenticode
    $sig = Get-AuthenticodeSignature $path
    if ($sig.SignerCertificate.Subject -notmatch "Startempire Wire") {
        throw "Authenticode signature invalid: $name"
    }
}

# SCM service
New-Service -Name "focusa-daemon" -BinaryPathName "$Prefix\bin\focusa-daemon.exe" -StartupType Automatic
Start-Service "focusa-daemon"

# PATH
[Environment]::SetEnvironmentVariable("PATH", "$env:PATH;$Prefix\bin", "User")

# Smoke test
& "$Prefix\bin\focusa.exe" doctor --json | ConvertFrom-Json | ForEach-Object {
    if ($_.status -ne "ok") { throw "Smoke test failed" }
}
```

---

## 20. Phase 2+ Items (Post-MVP, NOT blocking)

These are explicitly POST-MVP and will NOT block MVP Cohort. Listed here for future planning:

- `focusa update` command (P1)
- Atomic swap with rollback (P1)
- Channel selection `stable | preview | nightly` (P1)
- GPG signing automation (P1)
- Homebrew Cask (P2)
- winget (P2)
- apt/dnf packages (P2)
- Docker images (P2)

---

## 21. Open Questions with Owners and Deadlines

Resolves gap B6 (open questions had no owners/deadlines):

| # | Question | Owner | Deadline |
|---|----------|-------|----------|
| 1 | License revocation latency: 24h acceptable? | TBD | Before MVP Cohort |
| 2 | Does `--eval` allow multiple users on same machine? | TBD | Before MVP Cohort |
| 3 | Daemon refuses to start with invalid license OR just warns? | TBD | Before MVP Cohort |
| 4 | What happens if upgrade is interrupted mid-install? | TBD | Before MVP Cohort |
| 5 | Does `preview` channel break API stability guarantees (Spec92)? | TBD | Before MVP Cohort |
| 6 | What is the Linux aarch64 musl performance vs glibc? | TBD | Phase 2 |
| 7 | Should `--eval` rate limit apply per-user or per-machine? | TBD | Before MVP Cohort |
| 8 | macOS: full path sign or just daemon binary? | TBD | Phase 1.5 |

---

## 22. Bead Cross-Reference (Authoritative — Replaces §13 and §17)

### 22.1 Phase 1.5 Beads Implementing §15A

EPIC `focusa-112-arch` tracks all §15A work. Children decomposed into discrete, close-with-evidence items:

| Bead ID | §15A coverage | Status |
|---------|---------------|--------|
| `focusa-112-license-parity-audit` | §15A.2 license JSON shape | Open |
| `focusa-112-wp-error-envelope` | §15A.2 WP error handling | Open |
| `focusa-112-install-cmd` | §15A.1 \`focusa install\` subcommand skeleton | Open |
| `focusa-112-license-revalidate` | §15A.2 always re-validate | Open |
| `focusa-112-asset-download` | §3.1 + §15A.1 GH release API | Open |
| `focusa-112-checksum` | §5.1 + §15A.1 SHA256SUMS verify | Open |
| `focusa-112-symlinks` | §3.3 + §15A.1 ~/.local/bin priority | Open |
| `focusa-112-atomicity` | §6 + §15A.1 stash + rollback | Open |
| `focusa-112-target-detect` | §15A.1 --target=auto via uname | Open |
| `focusa-112-service-delegate` | §15A.3 delegate to service.rs | Open |
| `focusa-112-shell-slim` | §15A.1 + §15A.4 install-focusa.sh <=100 lines | Open |
| `focusa-112-ps1-slim` | §15A.1 + §15A.4 install-focusa.ps1 <=100 lines | Open |
| `focusa-112-codesign-subcmd` | §15A macOS codesign sign + notarize | Open |
| `focusa-112-codesign-verify` | §15A macOS install verifies signed binaries | Open |
| `focusa-112-windows-arm64-asset` | §15A + focusa-pydn CI matrix | Open |
| `focusa-112-musl-asset` | §15A + focusa-xvhd CI matrix | Open |
| `focusa-112-uninstall-cmd` | §15A.1 uninstall mirror | Open |
| `focusa-112-install-static-test` | §15A.5 static guard | Open |
| `focusa-112-install-smoke-test` | §15A.5 integration smoke | Open |
| `focusa-112-foyr-close` | §15A.5 closes `focusa-foyr` | Open |
| `focusa-112-3cok-close` | §15A.5 closes `focusa-3cok` | Open |
| `focusa-112-epic-close` | §15A.5 iqqi-arc closure | Open |

EPIC close requires every child P0 closed. Pre-existing P0 installer beads (`focusa-foyr`, `focusa-3cok`, `focusa-covz`, `focusa-iwft`, `focusa-pydn`, `focusa-xvhd`, `focusa-nwhx`) get closed transitively via the new graph.

### 22.2 Related Beads (Existing)

| Bead ID | Title | Closed by |
|---------|-------|-----------|
| `focusa-iqqi` | PORTABILITY: install binary architecture | `focusa-112-epic-close` |
| `focusa-foyr` | No daemon systemd unit installed | `focusa-112-foyr-close` |
| `focusa-3cok` | macOS LaunchAgent deferral | `focusa-112-3cok-close` |
| `focusa-covz` | No macOS code signing in installer | `focusa-112-codesign-verify` |
| `focusa-iwft` | install.focusa.dev/install.ps1 404 | `focusa-112-ps1-slim` |
| `focusa-pydn` | Windows ARM64 asset missing | `focusa-112-windows-arm64-asset` |
| `focusa-xvhd` | Linux musl asset missing | `focusa-112-musl-asset` |
| `focusa-b40j` | Real macOS codesign on Mac (MVP release gate) | Operator Mac required; tracked under `focusa-112-codesign-subcmd` |
| `focusa-nwhx` | codesign sign + notarize subcommand | `focusa-112-codesign-subcmd` |
| `focusa-cme3` | Tauri release artifacts in release.yml | `focusa-112-codesign-subcmd` |
| `focusa-cqhi` | Menubar Tauri local artifact | (separate workstream) |
| `focusa-7wgk` | Fresh-operator dry-run on clean VPS | (separate workstream) |
| `focusa-9im1` | GATE Wave 0 (parent gate) | `focusa-112-epic-close` |

### 22.3 Beads Closed by Phase 1.5 Completion

Closing `focusa-112-epic-close` AND `focusa-9im1` and all sub-beads of the §15A graph satisfies the Phase 1.5 P0 catalog.

---

## 23. Verification Status (Live — 2026-06-26)

### Verified Working
- `curl -fsSL https://install.focusa.dev/focusa` returns 234-line installer
- `--eval`, `--license-key`, `--dry-run`, `--uninstall` flags present
- WP REST endpoint `POST /license/validate` returns 200 with `valid: false` for test key
- GitHub release v0.9.25-dev publishes 13 assets: 3 binaries × 4 platforms (Linux gnu x86_64/arm64, Mac x86_64/arm64) + DMG + .app archives

### Verified NOT Working (Live HTTP Probes)
- `https://install.focusa.dev/install.ps1` → 404
- `https://install.focusa.dev/install.cmd` → 404
- `https://install.focusa.dev/install/windows` → 404
- No `SHA256SUMS.txt` in any release asset (verified: `gh release view v0.9.25-dev`)
- No musl assets in release
- No Windows ARM64 asset in release
- macOS daemon install deferred in real installer (`"skipping launchd plist for now (Phase 2)"`)

### Verified Spec Gaps
- §9 and §15 contradicted each other (now reconciled in §18)
- §10 said "Phase 1.4 stub" (now corrected throughout)
- Spec §5.2 marked signature as (P1) but Apple requires it (P0)
- 41 total gaps identified in `docs/INSTALL-GAP-AUDIT.md`

---

## 24. Spec Metadata

- **Spec number:** 112
- **Authoritative version:** This document supersedes all prior sections §9, §15 (contradictory)
- **Status:** Specification, awaiting Phase 1.5 implementation
- **Owner:** TBD (spec author)
- **Last verified:** 2026-06-26
- **Next review:** After Phase 1.5 P0 beads complete

---

## 25. Cross-References

- `docs/INSTALL-BINARY-SPEC.md` — This spec (Spec 112)
- `docs/INSTALL-GAP-AUDIT.md` — 41-gap audit (process + spec)
- `docs/MVP-AX-AUDIT.md` — AX audit (related to installer AX)
- `docs/current/ERROR_EMPTY_STATES.md` — Error envelope spec
- `docs/105-agent-dx-ux-merged-scope-spec.md` — DX/UX spec (Spec105)
- `docs/102-trajectory-ladder-consolidated-spec.md` — Trajectory spec (Spec102)
- `docs/92-agent-first-polish-hooks-efficiency-spec.md` — Polish spec (Spec92)
- `.github/workflows/release.yml` — Release pipeline (needs Phase 1.5 updates per §19.2)
