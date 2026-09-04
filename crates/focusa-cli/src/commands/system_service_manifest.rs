//! Installed distribution-manifest transaction for the canonical system lifecycle.

use super::{restore_file_bytes, write_new_file};
use anyhow::{Context, Result, bail};
use serde_json::Value;
use std::path::{Path, PathBuf};

const MANIFEST_NAME: &str = "distribution-manifest.json";
const STAGED_NAME: &str = ".distribution-manifest.staged";
const BACKUP_NAME: &str = ".distribution-manifest.rollback";
const JOURNAL_NAME: &str = ".distribution-manifest.transaction.json";

pub(crate) struct DistributionManifestTransaction {
    state_dir: PathBuf,
    manifest_path: PathBuf,
    staged_path: PathBuf,
    backup_path: PathBuf,
    journal_path: PathBuf,
    prior_manifest: Option<Vec<u8>>,
    settled: bool,
}

fn recover_interrupted_transaction(state_dir: &Path) -> Result<()> {
    let manifest_path = state_dir.join(MANIFEST_NAME);
    let staged_path = state_dir.join(STAGED_NAME);
    let backup_path = state_dir.join(BACKUP_NAME);
    let journal_path = state_dir.join(JOURNAL_NAME);
    if !journal_path.exists() {
        if backup_path.exists() {
            let backup = std::fs::read(&backup_path)?;
            let active = std::fs::read(&manifest_path).ok();
            if active.as_deref() != Some(backup.as_slice()) {
                bail!(
                    "ambiguous orphan distribution-manifest rollback {}; preserve state for recovery",
                    backup_path.display()
                );
            }
            std::fs::remove_file(&backup_path)?;
        }
        if staged_path.exists() {
            std::fs::remove_file(&staged_path)?;
        }
        focusa_core::durable_fs::sync_directory(state_dir)?;
        return Ok(());
    }
    let journal: Value = serde_json::from_slice(&std::fs::read(&journal_path)?)
        .context("parse distribution-manifest transaction journal")?;
    if journal.get("phase").and_then(Value::as_str) == Some("committed") {
        let _ = std::fs::remove_file(&backup_path);
        let _ = std::fs::remove_file(&staged_path);
        std::fs::remove_file(&journal_path)?;
        focusa_core::durable_fs::sync_directory(state_dir)?;
        return Ok(());
    }
    let prior_present = journal
        .get("prior_present")
        .and_then(Value::as_bool)
        .context("distribution-manifest journal omitted prior_present")?;
    let prior = if prior_present {
        Some(
            std::fs::read(&backup_path)
                .context("interrupted distribution-manifest rollback backup is missing")?,
        )
    } else {
        None
    };
    restore_file_bytes(&manifest_path, &staged_path, prior.as_deref())?;
    let _ = std::fs::remove_file(&backup_path);
    std::fs::remove_file(&journal_path)?;
    focusa_core::durable_fs::sync_directory(state_dir)?;
    Ok(())
}

pub(crate) fn prepare_distribution_manifest(
    state_dir: &Path,
    source: Option<&Path>,
    expected_tag: &str,
) -> Result<DistributionManifestTransaction> {
    let state_dir = state_dir.to_path_buf();
    std::fs::create_dir_all(&state_dir)
        .with_context(|| format!("create preserved state root {}", state_dir.display()))?;
    recover_interrupted_transaction(&state_dir)?;
    let manifest_path = state_dir.join(MANIFEST_NAME);
    let staged_path = state_dir.join(STAGED_NAME);
    let backup_path = state_dir.join(BACKUP_NAME);
    let journal_path = state_dir.join(JOURNAL_NAME);
    let prior_manifest = match std::fs::read(&manifest_path) {
        Ok(bytes) => Some(bytes),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(error).context(format!("read {}", manifest_path.display())),
    };
    if let Some(bytes) = &prior_manifest {
        write_new_file(&backup_path, bytes)?;
        focusa_core::durable_fs::sync_directory(&state_dir)?;
    }
    write_new_file(
        &journal_path,
        &serde_json::to_vec(&serde_json::json!({
            "schema": "focusa.distribution_manifest_transaction.v1",
            "phase": "prepared",
            "prior_present": prior_manifest.is_some(),
            "expected_tag": expected_tag,
        }))?,
    )?;
    focusa_core::durable_fs::sync_directory(&state_dir)?;
    let publish_result = match source {
        Some(source) => validate_distribution_manifest(source, expected_tag).and_then(|bytes| {
            write_new_file(&staged_path, &bytes)?;
            focusa_core::durable_fs::atomic_replace(&staged_path, &manifest_path)
                .with_context(|| format!("promote {}", manifest_path.display()))
        }),
        None => match std::fs::remove_file(&manifest_path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error).context(format!("remove {}", manifest_path.display())),
        },
    };
    let publish_result = publish_result.and_then(|()| {
        focusa_core::durable_fs::sync_directory(&state_dir)
            .with_context(|| format!("sync {}", state_dir.display()))
    });
    if let Err(error) = publish_result {
        restore_file_bytes(&manifest_path, &staged_path, prior_manifest.as_deref())?;
        let _ = std::fs::remove_file(&backup_path);
        let _ = std::fs::remove_file(&journal_path);
        return Err(error);
    }
    Ok(DistributionManifestTransaction {
        state_dir,
        manifest_path,
        staged_path,
        backup_path,
        journal_path,
        prior_manifest,
        settled: false,
    })
}

pub(crate) fn validate_distribution_manifest(source: &Path, expected_tag: &str) -> Result<Vec<u8>> {
    let bytes = std::fs::read(source)
        .with_context(|| format!("read distribution manifest {}", source.display()))?;
    let value: Value = serde_json::from_slice(&bytes).context("parse distribution manifest")?;
    let expected_version = expected_tag.strip_prefix('v').unwrap_or(expected_tag);
    let runtime = value.pointer("/components/runtime_contract");
    if value.get("schema").and_then(Value::as_str) != Some("focusa.distribution_manifest.v1")
        || value.get("release_version").and_then(Value::as_str) != Some(expected_version)
        || value.get("digest_contract").and_then(Value::as_str) != Some("sha256-tree-v1")
        || runtime
            .and_then(|contract| contract.get("installed_manifest_path"))
            .and_then(Value::as_str)
            != Some("/usr/local/lib/focusa/distribution-manifest.json")
        || runtime
            .and_then(|contract| contract.get("manifest_required_from"))
            .and_then(Value::as_str)
            != Some("0.9.188")
        || runtime
            .and_then(|contract| contract.pointer("/binary_paths/cli"))
            .and_then(Value::as_str)
            != Some("/usr/local/bin/focusa")
        || runtime
            .and_then(|contract| contract.pointer("/binary_paths/daemon"))
            .and_then(Value::as_str)
            != Some("/usr/local/bin/focusa-daemon")
        || runtime
            .and_then(|contract| contract.pointer("/binary_paths/tui"))
            .and_then(Value::as_str)
            != Some("/usr/local/bin/focusa-tui")
        || runtime
            .and_then(|contract| contract.pointer("/binary_paths/session_runner"))
            .and_then(Value::as_str)
            != Some("/usr/local/bin/focusa-session-runner")
    {
        bail!("distribution manifest identity/runtime contract mismatch");
    }
    Ok(bytes)
}

impl DistributionManifestTransaction {
    pub(crate) fn commit(mut self) -> Result<()> {
        let committed = self.journal_path.with_extension("json.committed");
        write_new_file(
            &committed,
            &serde_json::to_vec(&serde_json::json!({
                "schema": "focusa.distribution_manifest_transaction.v1",
                "phase": "committed",
                "prior_present": self.prior_manifest.is_some(),
            }))?,
        )?;
        focusa_core::durable_fs::atomic_replace(&committed, &self.journal_path)?;
        focusa_core::durable_fs::sync_directory(&self.state_dir)?;
        self.settled = true;
        for path in [&self.backup_path, &self.staged_path, &self.journal_path] {
            if path.exists()
                && let Err(error) = std::fs::remove_file(path)
            {
                eprintln!(
                    "warning: distribution manifest committed but transaction cleanup {} failed: {error}",
                    path.display()
                );
            }
        }
        if let Err(error) = focusa_core::durable_fs::sync_directory(&self.state_dir) {
            eprintln!(
                "warning: distribution manifest committed but cleanup directory sync failed: {error}"
            );
        }
        Ok(())
    }

    fn restore_prior_state(&mut self) -> Result<()> {
        restore_file_bytes(
            &self.manifest_path,
            &self.staged_path,
            self.prior_manifest.as_deref(),
        )?;
        let _ = std::fs::remove_file(&self.backup_path);
        let _ = std::fs::remove_file(&self.journal_path);
        focusa_core::durable_fs::sync_directory(&self.state_dir)?;
        self.settled = true;
        Ok(())
    }
}

impl Drop for DistributionManifestTransaction {
    fn drop(&mut self) {
        if !self.settled
            && let Err(error) = self.restore_prior_state()
        {
            eprintln!(
                "warning: automatic distribution manifest rollback failed: {error}; retained rollback={}",
                self.backup_path.display()
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn distribution_manifest(version: &str) -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({
            "schema": "focusa.distribution_manifest.v1",
            "release_version": version,
            "digest_contract": "sha256-tree-v1",
            "components": {
                "runtime_contract": {
                    "installed_manifest_path": "/usr/local/lib/focusa/distribution-manifest.json",
                    "manifest_required_from": "0.9.188",
                    "binary_paths": {
                        "cli": "/usr/local/bin/focusa",
                        "daemon": "/usr/local/bin/focusa-daemon",
                        "tui": "/usr/local/bin/focusa-tui",
                        "session_runner": "/usr/local/bin/focusa-session-runner"
                    }
                }
            }
        }))
        .unwrap()
    }

    #[test]
    fn promotion_rolls_back_and_commits_with_runtime() {
        let root =
            std::env::temp_dir().join(format!("focusa-system-manifest-{}", uuid::Uuid::now_v7()));
        std::fs::create_dir_all(&root).unwrap();
        let installed = root.join("distribution-manifest.json");
        let candidate = root.join("candidate.json");
        std::fs::write(&installed, distribution_manifest("0.9.187")).unwrap();
        std::fs::write(&candidate, distribution_manifest("0.9.188")).unwrap();

        {
            let _transaction =
                prepare_distribution_manifest(&root, Some(&candidate), "v0.9.188").unwrap();
            let active: Value =
                serde_json::from_slice(&std::fs::read(&installed).unwrap()).unwrap();
            assert_eq!(active["release_version"], "0.9.188");
        }
        let restored: Value = serde_json::from_slice(&std::fs::read(&installed).unwrap()).unwrap();
        assert_eq!(restored["release_version"], "0.9.187");

        prepare_distribution_manifest(&root, Some(&candidate), "v0.9.188")
            .unwrap()
            .commit()
            .unwrap();
        let committed: Value = serde_json::from_slice(&std::fs::read(&installed).unwrap()).unwrap();
        assert_eq!(committed["release_version"], "0.9.188");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn interrupted_precommit_transaction_recovers_prior_manifest() {
        let root = std::env::temp_dir().join(format!(
            "focusa-system-manifest-interrupted-{}",
            uuid::Uuid::now_v7()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let installed = root.join(MANIFEST_NAME);
        let candidate = root.join("candidate.json");
        let prior = distribution_manifest("0.9.187");
        std::fs::write(&installed, &prior).unwrap();
        std::fs::write(&candidate, distribution_manifest("0.9.188")).unwrap();

        let interrupted =
            prepare_distribution_manifest(&root, Some(&candidate), "v0.9.188").unwrap();
        std::mem::forget(interrupted);
        let recovered = prepare_distribution_manifest(&root, Some(&candidate), "v0.9.188").unwrap();
        drop(recovered);
        assert_eq!(std::fs::read(&installed).unwrap(), prior);
        assert!(!root.join(JOURNAL_NAME).exists());
        assert!(!root.join(BACKUP_NAME).exists());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn identity_fails_before_replacing_prior_receipt() {
        let root = std::env::temp_dir().join(format!(
            "focusa-system-manifest-invalid-{}",
            uuid::Uuid::now_v7()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let installed = root.join("distribution-manifest.json");
        let candidate = root.join("candidate.json");
        let prior = distribution_manifest("0.9.187");
        std::fs::write(&installed, &prior).unwrap();
        std::fs::write(&candidate, distribution_manifest("0.9.189")).unwrap();

        assert!(prepare_distribution_manifest(&root, Some(&candidate), "v0.9.188").is_err());
        assert_eq!(std::fs::read(&installed).unwrap(), prior);
        std::fs::remove_dir_all(root).unwrap();
    }
}
