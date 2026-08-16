//! Shared Pi package activation transaction (issue #309, Spec 112 §15A).
//!
//! Owns Pi extension discovery hygiene and atomic package activation:
//! - recognizes only Focusa-owned retired packages: package identity must be
//!   `focusa-pi-bridge` AND the entry name must match a known Focusa
//!   legacy/backup/old pattern;
//! - preserves retired packages under a sibling non-discovery root
//!   (`~/.pi/agent/retired-extensions/`), never under the live discovery root;
//! - places activation rollback backups outside the active `extensions/` root;
//! - returns a typed activation receipt that callers commit only after their
//!   wider transaction settles, or roll back after any downstream failure;
//! - never moves unrelated extensions.

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

use super::install::InstalledAsset;

pub const PACKAGE_IDENTITY: &str = "focusa-pi-bridge";
pub const CANONICAL_ENTRY: &str = "focusa";
pub const RETIRED_ROOT_NAME: &str = "retired-extensions";
pub const RECEIPT_SCHEMA: &str = "focusa.pi_activation_receipt.v1";
pub const FAULT_AFTER_PI_ACTIVATION: &str = "FOCUSA_UPDATE_FAULT_AFTER_PI_ACTIVATION";

pub fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

/// Entry-name patterns Focusa itself has used for legacy, backup, old,
/// rollback, and disabled copies of its Pi package. Unrelated extensions
/// never match these patterns, and the identity gate still applies.
pub fn is_focusa_retired_entry_name(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    n == "focusa"
        || n == "focusa-runtime"
        || n == "focusa-pi-bridge"
        || n.starts_with(".focusa-backup-")
        || n.starts_with("focusa-backup-")
        || n.starts_with("focusa-legacy-")
        || n.starts_with("focusa-old-")
        || n.starts_with("focusa-rollback-")
        || n.starts_with("focusa-disabled-")
        || n.ends_with(".legacy")
        || n.ends_with(".old")
        || n.ends_with(".backup")
        || n.ends_with(".disabled")
        || n.ends_with(".rollback")
        || n.contains(".legacy-")
        || n.contains(".backup-")
        || n.contains(".old-")
        || n.contains("-legacy-")
        || n.contains("-rollback-")
        || n.contains("-disabled-")
}

/// The `name` field of `package.json` inside `path`, when readable.
pub fn package_identity_of(path: &Path) -> Option<String> {
    let bytes = fs::read(path.join("package.json")).ok()?;
    let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    value.get("name")?.as_str().map(str::to_string)
}

/// True only when the package verifiably declares the Focusa Pi bridge identity.
pub fn is_focusa_package(path: &Path) -> bool {
    package_identity_of(path).as_deref() == Some(PACKAGE_IDENTITY)
}

/// Sibling non-discovery root for retired packages and activation backups.
/// For `~/.pi/agent/extensions` this is `~/.pi/agent/retired-extensions`.
pub fn retired_root_for(discovery_root: &Path) -> PathBuf {
    discovery_root
        .parent()
        .unwrap_or(discovery_root)
        .join(RETIRED_ROOT_NAME)
}

/// Resolve the Pi extension discovery root (explicit argument, then
/// `FOCUSA_PI_EXT_DIR`, then `$HOME/.pi/agent/extensions`).
pub fn resolve_pi_extensions_root(destination_root: Option<&Path>) -> Result<PathBuf> {
    destination_root
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("FOCUSA_PI_EXT_DIR").map(PathBuf::from))
        .or_else(|| {
            std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".pi/agent/extensions"))
        })
        .ok_or_else(|| anyhow!("HOME is unavailable; cannot locate Pi extensions"))
}

fn rename_dir_or_cross_device(source: &Path, destination: &Path) -> Result<()> {
    match fs::rename(source, destination) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::CrossesDevices => {
            move_dir_cross_device_safe(source, destination)
        }
        Err(error) => Err(error).with_context(|| {
            format!(
                "rename {} -> {}",
                source.display(),
                destination.display()
            )
        }),
    }
}

fn move_dir_cross_device_safe(source: &Path, destination: &Path) -> Result<()> {
    match fs::rename(source, destination) {
        Ok(()) => return Ok(()),
        Err(error) if error.kind() != std::io::ErrorKind::CrossesDevices => {
            return Err(error).with_context(|| {
                format!(
                    "rename {} -> {}",
                    source.display(),
                    destination.display()
                )
            });
        }
        Err(_) => {}
    }
    copy_dir_recursive(source, destination)?;
    fs::remove_dir_all(source)
        .with_context(|| format!("remove source after copy {}", source.display()))?;
    Ok(())
}

fn copy_dir_recursive(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)
        .with_context(|| format!("read directory {}", source.display()))?
    {
        let entry = entry?;
        let target = destination.join(entry.file_name());
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            copy_dir_recursive(&entry.path(), &target)?;
        } else if file_type.is_symlink() {
            let link = fs::read_link(entry.path())?;
            #[cfg(unix)]
            std::os::unix::fs::symlink(&link, &target)?;
            #[cfg(windows)]
            if link.is_dir() {
                std::os::windows::fs::symlink_dir(&link, &target)?;
            } else {
                std::os::windows::fs::symlink_file(&link, &target)?;
            }
        } else {
            fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

/// Move verified Focusa-owned retired entries out of the discovery root into
/// `retired_root`, removing stale Focusa-owned aliases. Returns the list of
/// retired/removed paths. The active canonical target and unrelated
/// extensions are never touched.
pub fn retire_focusa_packages(discovery_root: &Path, retired_root: &Path) -> Result<Vec<PathBuf>> {
    let mut retired = Vec::new();
    let canonical = discovery_root.join(CANONICAL_ENTRY);
    let canonical_real = fs::canonicalize(&canonical).ok();
    let entries = match fs::read_dir(discovery_root) {
        Ok(entries) => entries,
        Err(_) => return Ok(retired),
    };
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let name = match entry.file_name().into_string() {
            Ok(name) => name,
            Err(_) => continue,
        };
        if name == CANONICAL_ENTRY {
            // Active canonical target; never retire.
            continue;
        }
        if !is_focusa_retired_entry_name(&name) {
            // Unrelated extension, file, or directory; never touch.
            continue;
        }
        let path = entry.path();
        let is_symlink = fs::symlink_metadata(&path)
            .map(|metadata| metadata.file_type().is_symlink())
            .unwrap_or(false);
        if is_symlink {
            if let Ok(target) = fs::canonicalize(&path) {
                if canonical_real.as_ref() == Some(&target) {
                    // Compatibility alias resolving to the canonical target.
                    continue;
                }
            }
            // Stale Focusa-owned alias: dangling or pointing elsewhere.
            fs::remove_file(&path)
                .with_context(|| format!("remove stale Focusa alias {}", path.display()))?;
            retired.push(path);
            continue;
        }
        if !path.is_dir() {
            continue;
        }
        // Identity gate: only move packages that verifiably declare the
        // Focusa Pi bridge identity. Partial or unverifiable entries are
        // left alone rather than risk moving an unrelated extension.
        if !is_focusa_package(&path) {
            continue;
        }
        fs::create_dir_all(retired_root)?;
        let destination = retired_root.join(format!("{}-{}", name, uuid::Uuid::now_v7()));
        move_dir_cross_device_safe(&path, &destination).with_context(|| {
            format!(
                "retire Focusa package {} to {}",
                name,
                destination.display()
            )
        })?;
        retired.push(destination);
    }
    Ok(retired)
}

/// Prior-package state recorded in the activation receipt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorPiPackage {
    /// Backup location, always outside the active discovery root.
    pub backup: PathBuf,
    /// SHA-256 of the prior package.json (empty when unavailable).
    pub sha256: String,
}

/// Typed Pi activation receipt. Callers persist it in their transaction state
/// and call `commit_pi_activation` after settlement or
/// `rollback_pi_activation` after any downstream failure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PiActivationReceipt {
    pub schema: String,
    pub destination: PathBuf,
    pub version: String,
    pub prior: Option<PriorPiPackage>,
    pub retired: Vec<PathBuf>,
    pub activated_at: u64,
}

/// Staged package produced by `prepare_pi_package`.
pub struct PreparedPiPackage {
    pub staged: PathBuf,
    stage_root: PathBuf,
}

impl PreparedPiPackage {
    /// Remove the (now-empty) staging root after activation.
    pub fn cleanup(&self) {
        let _ = fs::remove_dir_all(&self.stage_root);
    }
}

fn prior_package_sha256(path: &Path) -> String {
    fs::read(path.join("package.json"))
        .map(|bytes| hex::encode(Sha256::digest(&bytes)))
        .unwrap_or_default()
}

/// Inspect, extract, and dependency-setup the archive under a unique staging
/// root inside `install_root`. Returns the staged package directory. The
/// caller activates or discards it; nothing under the discovery root is
/// touched here.
pub fn prepare_pi_package(
    asset: &InstalledAsset,
    install_root: &Path,
    npm_binary: Option<&Path>,
) -> Result<PreparedPiPackage> {
    let archive = Path::new(&asset.install_path);
    let listing = crate::commands::install::tar_command()
        .args(["-tzf"])
        .arg(archive)
        .output()
        .context("inspect Pi extension archive")?;
    if !listing.status.success() {
        anyhow::bail!("Pi extension archive listing failed");
    }
    let listing = String::from_utf8_lossy(&listing.stdout);
    if listing.lines().any(|entry| {
        entry.starts_with('/')
            || entry.split('/').any(|component| component == "..")
            || !(entry == "pi-extension" || entry.starts_with("pi-extension/"))
    }) || !listing
        .lines()
        .any(|entry| entry == "pi-extension/package.json")
    {
        anyhow::bail!("Pi extension archive contains unsafe or incomplete paths");
    }
    let stage_root = install_root.join(format!(".pi-extension-stage-{}", uuid::Uuid::now_v7()));
    fs::create_dir_all(&stage_root)?;
    let cleanup = || {
        let _ = fs::remove_dir_all(&stage_root);
    };
    let extracted = crate::commands::install::tar_command()
        .args(["-xzf"])
        .arg(archive)
        .arg("-C")
        .arg(&stage_root)
        .status()
        .context("extract Pi extension archive")?;
    if !extracted.success() {
        cleanup();
        anyhow::bail!("Pi extension archive extraction failed");
    }
    let staged = stage_root.join("pi-extension");
    let npm = std::process::Command::new(npm_binary.unwrap_or_else(|| Path::new("npm")))
        .args(["install", "--omit=dev", "--ignore-scripts"])
        .current_dir(&staged)
        .output()
        .context("run npm dependency setup for Pi extension")?;
    if !npm.status.success() {
        cleanup();
        let detail: String = String::from_utf8_lossy(&npm.stderr)
            .chars()
            .take(512)
            .collect();
        anyhow::bail!(
            "Pi extension dependency setup failed: {}",
            crate::commands::install::redact_url(&detail)
        );
    }
    Ok(PreparedPiPackage {
        staged,
        stage_root,
    })
}

/// Atomically activate the staged package as the one canonical `focusa` entry
/// under `destination_root`:
/// 1. retire verified Focusa-owned legacy/backup/old entries;
/// 2. back the prior package up OUTSIDE the discovery root;
/// 3. promote the staged package; on failure restore the prior package.
///
/// Returns a typed receipt. The backup is NOT deleted here — the caller must
/// `commit_pi_activation` after settlement or `rollback_pi_activation` on
/// failure.
pub fn activate_pi_package(
    staged: &Path,
    destination_root: &Path,
    version: &str,
) -> Result<PiActivationReceipt> {
    fs::create_dir_all(destination_root)?;
    let retired_root = retired_root_for(destination_root);
    fs::create_dir_all(&retired_root)?;
    let retired = retire_focusa_packages(destination_root, &retired_root)?;
    let destination = destination_root.join(CANONICAL_ENTRY);
    let mut prior: Option<PriorPiPackage> = None;
    if destination.exists() {
        let backups = retired_root.join("backups");
        fs::create_dir_all(&backups)?;
        let backup = backups.join(format!("focusa-backup-{}", uuid::Uuid::now_v7()));
        let sha256 = prior_package_sha256(&destination);
        rename_dir_or_cross_device(&destination, &backup).with_context(|| {
            format!(
                "backup active Pi package to {}",
                backup.display()
            )
        })?;
        prior = Some(PriorPiPackage { backup, sha256 });
    }
    if let Err(error) = rename_dir_or_cross_device(staged, &destination) {
        if let Some(prior) = &prior {
            if prior.backup.exists() {
                let _ = rename_dir_or_cross_device(&prior.backup, &destination);
            }
        }
        return Err(error).context("activate Pi extension");
    }
    Ok(PiActivationReceipt {
        schema: RECEIPT_SCHEMA.to_string(),
        destination,
        version: version.trim_start_matches('v').to_string(),
        prior,
        retired,
        activated_at: now_unix(),
    })
}

/// Settle a successful activation: remove the prior-package backup.
/// Safe to call only after the caller's wider transaction completed.
pub fn commit_pi_activation(receipt: &PiActivationReceipt) -> Result<()> {
    if let Some(prior) = &receipt.prior {
        if prior.backup.exists() {
            fs::remove_dir_all(&prior.backup).with_context(|| {
                format!(
                    "commit Pi activation: remove prior backup {}",
                    prior.backup.display()
                )
            })?;
        }
    }
    Ok(())
}

/// Restore the exact prior package after a failed transaction. With a prior
/// backup: the promoted package is set aside, the backup is restored, and the
/// set-aside copy is removed. Without one: the promoted package is removed.
pub fn rollback_pi_activation(receipt: &PiActivationReceipt) -> Result<()> {
    let destination = &receipt.destination;
    let backup = match &receipt.prior {
        Some(prior) if prior.backup.exists() => Some(&prior.backup),
        _ => None,
    };
    match backup {
        Some(backup) => {
            let failed = destination.with_extension(format!(
                "focusa-failed-{}",
                uuid::Uuid::now_v7()
            ));
            if destination.exists() {
                rename_dir_or_cross_device(destination, &failed).with_context(|| {
                    format!(
                        "set aside failed Pi package {}",
                        destination.display()
                    )
                })?;
            }
            if let Err(error) = rename_dir_or_cross_device(backup, destination) {
                if failed.exists() {
                    let _ = rename_dir_or_cross_device(&failed, destination);
                }
                return Err(error).context("restore prior Pi package");
            }
            if failed.exists() {
                fs::remove_dir_all(&failed)?;
            }
            Ok(())
        }
        None => {
            if destination.exists() {
                fs::remove_dir_all(destination)?;
            }
            Ok(())
        }
    }
}
