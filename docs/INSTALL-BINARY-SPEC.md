# Install Binary Architecture Spec

**Status:** Specification (no implementation yet)
**Operator ask:** Smart, system-detecting installer that installs proper binary and weaves into license authority (WordPress website).
**Source URL:** https://install.focusa.dev/focusa
**Last verified:** 2026-06-26

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

### 3.1 Asset Types to Install

For each release, GitHub publishes these assets (verified in v0.9.25-dev):

| Binary | Source asset | Install location | Purpose |
|--------|--------------|------------------|---------|
| `focusa` | `focusa-{VERSION}-{target}` | `~/.focusa/bin/focusa` | CLI |
| `focusa-daemon` | `focusa-daemon-{VERSION}-{target}` | `~/.focusa/bin/focusa-daemon` | Daemon |
| `focusa-tui` | `focusa-tui-{VERSION}-{target}` | `~/.focusa/bin/focusa-tui` | TUI dashboard |

### 3.2 Optional Components

| Component | Asset | Install when |
|-----------|-------|--------------|
| UIAI Engine | (separate `engine` script) | `--with-engine` flag |
| Pi wrapper | (downstream install.sh) | `--with-pi` flag |
| OpenClaw bridges | (Phase 2 placeholder) | `--with-openclaw` flag |
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

### Not Yet Verified (Gaps)
- No SHA256SUMS.txt in any release
- No GPG signatures
- No `focusa update` command
- Daemon doesn't install (Phase 1.4 stub)
- No systemd unit deployment
- No post-install smoke tests
- No rollback on failure
- No `--channel preview` actually queries WP

---

## 11. Implementation Roadmap

### Phase 1.5 — Real Binary Install (P0)
- Replace Python stub with real Rust binary download
- Add SHA256SUMS verification
- Add daemon systemd unit deployment
- Add post-install `focusa doctor` smoke test

### Phase 2.0 — Update + Rollback (P1)
- Implement `focusa update` command
- Add atomic swap with rollback
- Add `--channel preview` support

### Phase 2.5 — Signing (P1)
- Add cosign signing to release.yml
- Verify signatures on install
- Publish `SHA256SUMS.txt.sig`

### Phase 3.0 — Package Managers (P2)
- Build `.deb` and `.rpm` packages in CI
- Publish to apt/winget/brew repositories
- Maintain parallel install paths

---

## 12. Open Questions

1. **License revocation latency**: How quickly does daemon detect license revocation? (Spec: 24h re-validation, but is that acceptable?)
2. **Multi-user eval**: Does `--eval` allow multiple users on same machine? (Currently ambiguous)
3. **Daemon license enforcement**: Does daemon refuse to start with invalid license, or just emit warnings?
4. **Upgrade interruption**: What happens if upgrade is interrupted mid-install? (Need atomic swap with rollback)
5. **Channel stability**: Does `preview` channel break API stability guarantees? (Spec92 implications)

---

## 13. Bead Reference

- `focusa-iqqi` — PORTABILITY: Real architecture needed beyond shell scripts
- `focusa-7wgk` — Tier2-2: Fresh-operator dry-run on a clean VPS
- `focusa-cme3` — Tier2-4: Tauri release artifacts in release.yml
- All three blocked by this spec needing implementation