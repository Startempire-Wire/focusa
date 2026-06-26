# Install Binary Architecture Spec

**Status:** Planning (from operator's first ask of MVP session)
**Source URL:** https://install.focusa.dev/focusa
**Last Verified:** 2026-06-26

---

## 1. The Reality (Verified Live)

`curl -fsSL https://install.focusa.dev/focusa` returns a **real bash installer** hosted on install.focusa.dev:

- 234 lines, 7800 bytes
- Last modified: 2026-06-17
- Has options: `--eval`, `--license-key`, `--prefix`, `--channel`, `--dry-run`, `--uninstall`, `--with-engine`, `--with-pi`, `--with-openclaw`, `--no-service`, `--accept-license`

### What it currently does:
1. Creates `~/.focusa/{bin,state,config,libexec}` layout
2. Drops a launcher stub `~/.focusa/bin/focusa` (currently Python-based, "Real binary arrives when the engine binary is wired in")
3. Validates license key against `https://install.focusa.dev/wp-json/wpuiai-ai-cloud/v1/license/validate`
4. Writes license state to `~/.config/focusa/license.json`
5. Optionally installs UIAI Engine, Pi wrapper, OpenClaw bridges
6. Symlinks `/usr/local/bin/focusa -> ~/.focusa/bin/focusa`
7. **Phase 1.4 status** — explicit "minimal stub for Phase 1.4"

### What it does NOT do (gaps):
1. ❌ No checksums/signature verification on downloaded binaries
2. ❌ No release binary download — drops stub instead
3. ❌ No daemon install — Phase 1.4 doesn't yet deploy focusa-daemon
4. ❌ No systemd unit for daemon — only symlinks CLI launcher
5. ❌ No update mechanism (`focusa update` doesn't exist)
6. ❌ No TUI install despite release.yml producing `focusa-tui-*` binaries
7. ❌ No rollback on failure
8. ❌ No smoke tests post-install
9. ❌ No package manager support (deb/rpm/brew)

---

## 2. Proposed Architecture (Integration Plan)

### 2.1 One-Liner Install (Already Documented)
```bash
curl -fsSL https://install.focusa.dev/focusa | bash -s -- --accept-license
curl -fsSL https://install.focusa.dev/focusa | bash -s -- --eval
curl -fsSL https://install.focusa.dev/focusa | bash -s -- --license-key "$FOCUSA_LICENSE_KEY"
```

### 2.2 What Phase 1.5+ Must Do (to match our actual binaries)

The installer must:
1. **Detect platform**: `uname -s` + `uname -m` → pick `focusa-{VERSION}-{target}` from GitHub release
2. **Verify integrity**:
   - Download `SHA256SUMS.txt` from release
   - Verify with `sha256sum -c` for the platform asset
   - Optional: GPG signature verification (`cosign verify` or `gpg --verify`)
3. **Install multiple binaries**:
   - `focusa` (CLI) → `~/.focusa/bin/focusa`
   - `focusa-daemon` → `~/.focusa/bin/focusa-daemon`
   - `focusa-tui` → `~/.focusa/bin/focusa-tui`
4. **Deploy systemd unit** (Linux):
   - `/etc/systemd/system/focusa-daemon.service` (or `~/.config/systemd/user/`)
   - `systemctl --user enable --now focusa-daemon`
5. **Smoke tests**:
   - `focusa doctor --json | jq '.status' == "ok"`
   - `~/.focusa/bin/focusa-daemon --health` (or `curl /v1/health`)
6. **Rollback on failure**:
   - Stash old binaries at `~/.focusa/state/last-known-good/`
   - On install failure, restore from stash
7. **Update mechanism**:
   - `focusa update` — pulls latest stable release
   - `focusa update --channel preview`
8. **Package manager integration** (Phase 2):
   - `apt install focusa` (Debian/Ubuntu)
   - `dnf install focusa` (Fedora/RHEL)
   - `brew install focusa` (macOS)

### 2.3 Source Layout (Where the Installer Lives)

The installer script lives at:
- `https://install.focusa.dev/focusa` (live gateway)
- `crates/focusa-installer/` or `scripts/install/focusa-install.sh` (source of truth)

The GitHub repo should mirror the live installer for transparency:
```
crates/focusa-installer/
  src/
    install.sh        # bash source (mirrors install.focusa.dev/focusa)
    verify.sh         # checksum/GPG verification
    daemon-systemd.sh # systemd unit install
    smoke.sh          # post-install health checks
  tests/
    install_test.sh
    upgrade_test.sh
    rollback_test.sh
  Cargo.toml          # meta-package for `cargo install --path crates/focusa-installer`
```

### 2.4 CI/CD Pipeline Integration

`.github/workflows/release.yml` must:
1. Sign release artifacts (GPG or cosign)
2. Generate `SHA256SUMS.txt` and `SHA256SUMS.txt.sig`
3. Update install.focusa.dev to point at latest release
4. Run installer integration tests against the new release

### 2.5 Verification Flow (Agent Experience)

Per Spec92 AX, install must be agent-friendly:
- `--eval` for evaluation mode (no commercial use)
- `--dry-run` to preview actions
- `--accept-license` for production
- `--license-key` with validation
- Progress output with next-action hints
- Rollback on failure with recovery hint

---

## 3. Current Gap vs Spec

| Requirement | Status | Gap |
|-------------|--------|-----|
| One-liner install | ✅ Done | install.focusa.dev/focusa exists |
| License validation | ✅ Done | Validates against WP REST API |
| Eval mode | ✅ Done | `--eval` flag |
| Dry-run | ✅ Done | `--dry-run` flag |
| Uninstall | ✅ Done | `--uninstall` flag |
| Channel selection | ✅ Done | `stable\|preview` |
| **Release binary download** | ❌ **Missing** | Drops Python stub instead |
| **Checksum verification** | ❌ **Missing** | No SHA256SUMS published |
| **GPG signatures** | ❌ **Missing** | No signing |
| **Daemon install** | ❌ **Missing** | Phase 1.4 doesn't deploy daemon |
| **systemd unit** | ❌ **Missing** | Only symlinks CLI |
| **TUI install** | ❌ **Missing** | focusa-tui-* binaries exist but not deployed |
| **Update mechanism** | ❌ **Missing** | No `focusa update` command |
| **Smoke tests** | ❌ **Missing** | No post-install verification |
| **Rollback** | ❌ **Missing** | No recovery on failure |
| **Package managers** | ❌ **Missing** | No deb/rpm/brew |

---

## 4. Acceptance Criteria

1. `curl -fsSL https://install.focusa.dev/focusa | bash -s -- --accept-license` deploys:
   - `~/.focusa/bin/focusa` (real Rust binary, not stub)
   - `~/.focusa/bin/focusa-daemon` (real daemon)
   - `~/.focusa/bin/focusa-tui` (real TUI)
   - systemd unit enabled and running
2. SHA256SUMS verified on download
3. Post-install smoke test passes (`focusa doctor` returns ok)
4. `focusa update` upgrades in-place with rollback
5. Uninstall leaves no residue except `~/.config/focusa/license.json` (intentional for audit)

---

## 5. Action Items

1. **P0**: Add SHA256SUMS generation to release.yml
2. **P0**: Update install.focusa.dev/focusa to download real binaries (not stub)
3. **P0**: Add daemon systemd unit installer to script
4. **P1**: Add GPG/cosign signing to release artifacts
5. **P1**: Implement `focusa update` command
6. **P1**: Add post-install smoke tests
7. **P2**: Add rollback on failure
8. **P2**: Package manager support (deb/rpm/brew)

---

## 6. Bead

- `focusa-iqqi` (open) — PORTABILITY: Manual cp / shell-script installer is not real architecture
- `focusa-iqqi` should be expanded to include: install.focusa.dev integration spec, checksum verification, GPG signing, daemon systemd unit, update mechanism