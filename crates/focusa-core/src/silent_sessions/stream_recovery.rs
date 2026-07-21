use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{
    MigrationMode, OutputChannel, SecureStreamStore, SilentSessionId, SilentSessionRunId,
    StreamChunkManifest, StreamStorageError, migrate_silent_session_schema,
    secure_fs::{create_secure_descendants, secure_read},
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StreamRecoveryAction {
    Verified,
    IndexRebuilt,
    Quarantined,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StreamRecoveryEvent {
    pub session_id: SilentSessionId,
    pub run_id: SilentSessionRunId,
    pub artifact_ref: String,
    pub action: StreamRecoveryAction,
    pub reason: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct StreamRecoveryReport {
    pub events: Vec<StreamRecoveryEvent>,
    pub degraded: bool,
}

impl SecureStreamStore {
    /// Audits only one explicitly registered session/run directory. Valid
    /// sidecar manifests can rebuild a missing index; malformed artifacts are
    /// preserved under the run's secured recovery/quarantine directory.
    pub fn recover_registered_run(
        &self,
        session_id: SilentSessionId,
        run_id: SilentSessionRunId,
    ) -> Result<StreamRecoveryReport, StreamStorageError> {
        let migration = migrate_silent_session_schema(&self.persistence, MigrationMode::DryRun)?;
        if migration.previous_version != migration.target_version {
            return Err(anyhow::anyhow!(
                "silent session schema migration required before recovery audit"
            )
            .into());
        }
        let run_root = self
            .root
            .join(session_id.to_string())
            .join(run_id.to_string());
        if !run_root.exists() {
            return Ok(StreamRecoveryReport::default());
        }
        reject_symlink_or_non_dir(&run_root)?;
        let mut report = StreamRecoveryReport::default();
        for channel in channels() {
            let directory = run_root.join(channel.as_str());
            if !directory.exists() {
                continue;
            }
            reject_symlink_or_non_dir(&directory)?;
            let mut sidecars = fs::read_dir(&directory)
                .map_err(anyhow::Error::from)?
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .filter(|path| {
                    path.file_name()
                        .is_some_and(|name| name.to_string_lossy().ends_with(".manifest.json"))
                })
                .collect::<Vec<_>>();
            sidecars.sort();
            for sidecar in sidecars {
                self.recover_sidecar(
                    session_id,
                    run_id,
                    channel,
                    &run_root,
                    &sidecar,
                    &mut report,
                )?;
            }
        }
        Ok(report)
    }

    fn recover_sidecar(
        &self,
        session_id: SilentSessionId,
        run_id: SilentSessionRunId,
        channel: OutputChannel,
        run_root: &Path,
        sidecar: &Path,
        report: &mut StreamRecoveryReport,
    ) -> Result<(), StreamStorageError> {
        let artifact_ref = sidecar
            .strip_prefix(&self.root)
            .unwrap_or(sidecar)
            .to_string_lossy()
            .into_owned();
        let outcome = (|| -> Result<(StreamChunkManifest, PathBuf), anyhow::Error> {
            let metadata = fs::symlink_metadata(sidecar)?;
            if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
                anyhow::bail!("manifest is not a regular no-follow file");
            }
            if metadata.len() > 1_048_576 {
                anyhow::bail!("manifest exceeds the recovery size limit");
            }
            let bytes = secure_read(sidecar, metadata.len())?;
            let manifest: StreamChunkManifest = serde_json::from_slice(&bytes)?;
            if manifest.session_id != session_id
                || manifest.run_id != run_id
                || manifest.channel != channel
            {
                anyhow::bail!("manifest identity does not match registered directory");
            }
            let chunk = self.resolve_chunk_ref(&manifest.chunk_ref)?;
            if chunk.parent() != sidecar.parent() {
                anyhow::bail!("manifest chunk reference escapes its registered channel");
            }
            let compressed = secure_read(&chunk, manifest.compressed_bytes)?;
            if hex::encode(Sha256::digest(&compressed)) != manifest.chunk_hash {
                anyhow::bail!("chunk checksum mismatch");
            }
            Ok((manifest, chunk))
        })();
        match outcome {
            Ok((manifest, _chunk)) => {
                let action = if self
                    .load_index(session_id, run_id, channel, manifest.chunk_sequence)?
                    .is_some()
                {
                    StreamRecoveryAction::Verified
                } else {
                    self.insert_index(&manifest)?;
                    StreamRecoveryAction::IndexRebuilt
                };
                report.events.push(StreamRecoveryEvent {
                    session_id,
                    run_id,
                    artifact_ref,
                    action,
                    reason: "manifest and content hash verified".into(),
                });
            }
            Err(error) => {
                let quarantine = run_root.join("recovery").join("quarantine");
                create_secure_descendants(&self.root, &quarantine)?;
                if let Some(paired_chunk) = paired_chunk_path(sidecar) {
                    if paired_chunk.exists() {
                        quarantine_file(&paired_chunk, &quarantine)?;
                    }
                }
                quarantine_file(sidecar, &quarantine)?;
                report.degraded = true;
                report.events.push(StreamRecoveryEvent {
                    session_id,
                    run_id,
                    artifact_ref,
                    action: StreamRecoveryAction::Quarantined,
                    reason: error.to_string(),
                });
            }
        }
        Ok(())
    }
}

fn reject_symlink_or_non_dir(path: &Path) -> Result<(), StreamStorageError> {
    let metadata = fs::symlink_metadata(path).map_err(anyhow::Error::from)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(StreamStorageError::UnsafePath(path.display().to_string()));
    }
    Ok(())
}

fn paired_chunk_path(sidecar: &Path) -> Option<PathBuf> {
    let name = sidecar.file_name()?.to_str()?;
    let stem = name.strip_suffix(".manifest.json")?;
    Some(sidecar.with_file_name(format!("{stem}.fss")))
}

fn quarantine_file(path: &Path, quarantine: &Path) -> Result<(), StreamStorageError> {
    let name = path
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("artifact has no file name"))?;
    let destination = quarantine.join(format!(
        "{}-{}",
        uuid::Uuid::now_v7(),
        name.to_string_lossy()
    ));
    fs::rename(path, destination).map_err(anyhow::Error::from)?;
    Ok(())
}

fn channels() -> [OutputChannel; 10] {
    [
        OutputChannel::Stdout,
        OutputChannel::Stderr,
        OutputChannel::StructuredHarnessEvents,
        OutputChannel::AssistantText,
        OutputChannel::ThinkingText,
        OutputChannel::ToolCalls,
        OutputChannel::ToolOutput,
        OutputChannel::FocusaControlEvents,
        OutputChannel::OperatorInput,
        OutputChannel::SystemDiagnostics,
    ]
}
