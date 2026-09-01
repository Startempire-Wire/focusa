use crate::silent_session::{SilentSessionId, SilentSessionRunId};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, SyncSender, TrySendError, sync_channel};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub const SILENT_STREAM_MANIFEST_SCHEMA: &str = "focusa.silent_stream_manifest.v1";
pub const SILENT_STREAM_COMPLETION_SCHEMA: &str = "focusa.silent_stream_completion.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SilentStreamKind {
    Stdout,
    Stderr,
    Semantic,
}

impl SilentStreamKind {
    fn file_stem(self) -> &'static str {
        match self {
            Self::Stdout => "stdout",
            Self::Stderr => "stderr",
            Self::Semantic => "semantic",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SilentStreamCursor {
    pub run_id: SilentSessionRunId,
    pub seq: u64,
    checksum: String,
}

impl SilentStreamCursor {
    pub fn new(run_id: SilentSessionRunId, seq: u64) -> Self {
        let checksum = cursor_checksum(run_id, seq);
        Self {
            run_id,
            seq,
            checksum,
        }
    }

    pub fn encode(&self) -> String {
        hex::encode(serde_json::to_vec(self).expect("cursor serialization is infallible"))
    }

    pub fn decode(value: &str) -> anyhow::Result<Self> {
        let bytes =
            hex::decode(value).map_err(|_| anyhow::anyhow!("invalid stream cursor encoding"))?;
        let cursor: Self = serde_json::from_slice(&bytes)
            .map_err(|_| anyhow::anyhow!("invalid stream cursor payload"))?;
        anyhow::ensure!(
            cursor.checksum == cursor_checksum(cursor.run_id, cursor.seq),
            "invalid stream cursor checksum"
        );
        Ok(cursor)
    }
}

fn cursor_checksum(run_id: SilentSessionRunId, seq: u64) -> String {
    let mut digest = Sha256::new();
    digest.update(b"focusa-silent-stream-cursor-v1\0");
    digest.update(run_id.to_string());
    digest.update(seq.to_le_bytes());
    hex::encode(digest.finalize())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SilentStreamChunkManifest {
    pub schema: String,
    pub session_id: SilentSessionId,
    pub run_id: SilentSessionRunId,
    pub config_hash: String,
    pub stream: SilentStreamKind,
    pub chunk_index: u64,
    pub first_cursor: SilentStreamCursor,
    pub last_cursor: SilentStreamCursor,
    pub byte_count: u64,
    pub content_sha256: String,
    pub file_name: String,
    pub redaction_applied: bool,
}

#[derive(Debug, Serialize)]
struct StreamRecord<'a> {
    seq: u64,
    cursor: String,
    data: &'a str,
}

pub struct SecureSilentStreamChunk {
    session_id: SilentSessionId,
    run_id: SilentSessionRunId,
    config_hash: String,
    stream: SilentStreamKind,
    chunk_index: u64,
    run_root: PathBuf,
    partial_path: PathBuf,
    writer: BufWriter<File>,
    digest: Sha256,
    first_seq: Option<u64>,
    last_seq: Option<u64>,
    byte_count: u64,
    redaction_applied: bool,
}

impl SecureSilentStreamChunk {
    pub fn create(
        data_root: &Path,
        session_id: SilentSessionId,
        run_id: SilentSessionRunId,
        config_hash: impl Into<String>,
        stream: SilentStreamKind,
        chunk_index: u64,
    ) -> anyhow::Result<Self> {
        anyhow::ensure!(
            data_root.is_absolute(),
            "silent stream root must be absolute"
        );
        reject_symlink(data_root)?;
        let sessions_root = data_root.join("silent-sessions");
        let session_root = sessions_root.join(session_id.to_string());
        let run_root = session_root.join(run_id.to_string());
        secure_dir(&sessions_root)?;
        secure_dir(&session_root)?;
        secure_dir(&run_root)?;
        secure_dir(&run_root.join("streams"))?;
        secure_dir(&run_root.join("artifacts"))?;
        secure_dir(&run_root.join("recovery"))?;
        let partial_path = run_root.join("streams").join(format!(
            "{}-{chunk_index:06}.jsonl.partial-{}",
            stream.file_stem(),
            Uuid::now_v7()
        ));
        let file = secure_new_file(&partial_path)?;
        Ok(Self {
            session_id,
            run_id,
            config_hash: config_hash.into(),
            stream,
            chunk_index,
            run_root,
            partial_path,
            writer: BufWriter::new(file),
            digest: Sha256::new(),
            first_seq: None,
            last_seq: None,
            byte_count: 0,
            redaction_applied: false,
        })
    }

    pub fn append(
        &mut self,
        seq: u64,
        data: &str,
        secrets: &[String],
    ) -> anyhow::Result<SilentStreamCursor> {
        if let Some(last) = self.last_seq {
            anyhow::ensure!(
                seq == last + 1,
                "stream sequence must be monotonic and contiguous"
            );
        }
        let mut redacted = data.to_string();
        for secret in secrets.iter().filter(|secret| !secret.is_empty()) {
            if redacted.contains(secret) {
                redacted = redacted.replace(secret, "[REDACTED]");
                self.redaction_applied = true;
            }
        }
        let cursor = SilentStreamCursor::new(self.run_id, seq);
        let mut bytes = serde_json::to_vec(&StreamRecord {
            seq,
            cursor: cursor.encode(),
            data: &redacted,
        })?;
        bytes.push(b'\n');
        self.writer.write_all(&bytes)?;
        self.digest.update(&bytes);
        self.byte_count += u64::try_from(bytes.len())?;
        self.first_seq.get_or_insert(seq);
        self.last_seq = Some(seq);
        Ok(cursor)
    }

    pub fn close(mut self) -> anyhow::Result<SilentStreamChunkManifest> {
        let first_seq = self
            .first_seq
            .ok_or_else(|| anyhow::anyhow!("cannot close empty stream chunk"))?;
        let last_seq = self.last_seq.expect("first sequence implies last sequence");
        self.writer.flush()?;
        self.writer.get_ref().sync_all()?;
        let final_name = format!("{}-{:06}.jsonl", self.stream.file_stem(), self.chunk_index);
        let final_path = self.run_root.join("streams").join(&final_name);
        anyhow::ensure!(!final_path.exists(), "stream chunk already exists");
        crate::durable_fs::atomic_replace(&self.partial_path, &final_path)?;
        crate::durable_fs::sync_directory(final_path.parent().expect("stream file has parent"))?;
        let manifest = SilentStreamChunkManifest {
            schema: SILENT_STREAM_MANIFEST_SCHEMA.into(),
            session_id: self.session_id,
            run_id: self.run_id,
            config_hash: self.config_hash,
            stream: self.stream,
            chunk_index: self.chunk_index,
            first_cursor: SilentStreamCursor::new(self.run_id, first_seq),
            last_cursor: SilentStreamCursor::new(self.run_id, last_seq),
            byte_count: self.byte_count,
            content_sha256: hex::encode(self.digest.finalize()),
            file_name: final_name,
            redaction_applied: self.redaction_applied,
        };
        atomic_secure_json(
            &self.run_root.join(format!(
                "manifest-{}-{:06}.json",
                self.stream.file_stem(),
                self.chunk_index
            )),
            &manifest,
        )?;
        Ok(manifest)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SilentStreamCompletionArtifact {
    pub schema: String,
    pub session_id: SilentSessionId,
    pub run_id: SilentSessionRunId,
    pub chunks: Vec<SilentStreamChunkManifest>,
    pub total_bytes: u64,
    pub last_cursor: Option<SilentStreamCursor>,
}

pub struct RotatingSilentStreamWriter {
    data_root: PathBuf,
    session_id: SilentSessionId,
    run_id: SilentSessionRunId,
    config_hash: String,
    stream: SilentStreamKind,
    max_chunk_bytes: u64,
    next_chunk_index: u64,
    active: Option<SecureSilentStreamChunk>,
    closed: Vec<SilentStreamChunkManifest>,
}

impl RotatingSilentStreamWriter {
    pub fn new(
        data_root: PathBuf,
        session_id: SilentSessionId,
        run_id: SilentSessionRunId,
        config_hash: impl Into<String>,
        stream: SilentStreamKind,
        max_chunk_bytes: u64,
    ) -> anyhow::Result<Self> {
        anyhow::ensure!(max_chunk_bytes > 0, "stream chunk size must be positive");
        Ok(Self {
            data_root,
            session_id,
            run_id,
            config_hash: config_hash.into(),
            stream,
            max_chunk_bytes,
            next_chunk_index: 1,
            active: None,
            closed: vec![],
        })
    }

    pub fn append(
        &mut self,
        seq: u64,
        data: &str,
        secrets: &[String],
    ) -> anyhow::Result<SilentStreamCursor> {
        if self.active.is_none() {
            self.active = Some(SecureSilentStreamChunk::create(
                &self.data_root,
                self.session_id,
                self.run_id,
                self.config_hash.clone(),
                self.stream,
                self.next_chunk_index,
            )?);
            self.next_chunk_index += 1;
        }
        let active = self
            .active
            .as_mut()
            .expect("active stream chunk was created");
        let cursor = active.append(seq, data, secrets)?;
        if active.byte_count >= self.max_chunk_bytes {
            self.close_active()?;
        }
        Ok(cursor)
    }

    fn close_active(&mut self) -> anyhow::Result<()> {
        if let Some(active) = self.active.take() {
            self.closed.push(active.close()?);
        }
        Ok(())
    }

    pub fn finish(mut self) -> anyhow::Result<SilentStreamCompletionArtifact> {
        self.close_active()?;
        let artifact = SilentStreamCompletionArtifact {
            schema: SILENT_STREAM_COMPLETION_SCHEMA.into(),
            session_id: self.session_id,
            run_id: self.run_id,
            total_bytes: self.closed.iter().map(|chunk| chunk.byte_count).sum(),
            last_cursor: self.closed.last().map(|chunk| chunk.last_cursor.clone()),
            chunks: self.closed,
        };
        let artifact_path = self
            .data_root
            .join("silent-sessions")
            .join(self.session_id.to_string())
            .join(self.run_id.to_string())
            .join("artifacts/stream-completion.json");
        atomic_secure_json(&artifact_path, &artifact)?;
        Ok(artifact)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StreamCaptureRecord {
    pub seq: u64,
    pub data: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureAdmission {
    Queued,
    BackpressureApplied,
    ConsumerDisconnected,
    DurabilityFailed,
}

#[derive(Clone)]
pub struct NonBlockingStreamCapture {
    sender: SyncSender<StreamCaptureRecord>,
    overflow_spool: Arc<Mutex<File>>,
}

impl NonBlockingStreamCapture {
    pub fn durable_bounded(
        capacity: usize,
        overflow_spool_path: impl AsRef<Path>,
    ) -> anyhow::Result<(Self, Receiver<StreamCaptureRecord>)> {
        anyhow::ensure!(capacity > 0, "capture queue capacity must be positive");
        let path = overflow_spool_path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let overflow_spool = OpenOptions::new().create(true).append(true).open(path)?;
        let (sender, receiver) = sync_channel(capacity);
        Ok((
            Self {
                sender,
                overflow_spool: Arc::new(Mutex::new(overflow_spool)),
            },
            receiver,
        ))
    }

    pub fn try_capture(&self, record: StreamCaptureRecord) -> CaptureAdmission {
        match self.sender.try_send(record) {
            Ok(()) => CaptureAdmission::Queued,
            Err(TrySendError::Full(record)) => {
                if self.persist_overflow(&record) {
                    CaptureAdmission::BackpressureApplied
                } else {
                    CaptureAdmission::DurabilityFailed
                }
            }
            Err(TrySendError::Disconnected(record)) => {
                if self.persist_overflow(&record) {
                    CaptureAdmission::ConsumerDisconnected
                } else {
                    CaptureAdmission::DurabilityFailed
                }
            }
        }
    }

    fn persist_overflow(&self, record: &StreamCaptureRecord) -> bool {
        let Ok(mut spool) = self.overflow_spool.lock() else {
            return false;
        };
        serde_json::to_writer(&mut *spool, record).is_ok()
            && spool.write_all(b"\n").is_ok()
            && spool.flush().is_ok()
            && spool.sync_data().is_ok()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StreamRecoveryReport {
    pub scanned_runs: usize,
    pub valid_chunks: usize,
    pub quarantined_paths: Vec<PathBuf>,
    pub degraded_sessions: Vec<SilentSessionId>,
}

pub fn recover_registered_streams(
    data_root: &Path,
    registered_runs: &[(SilentSessionId, SilentSessionRunId)],
) -> anyhow::Result<StreamRecoveryReport> {
    let mut report = StreamRecoveryReport {
        scanned_runs: 0,
        valid_chunks: 0,
        quarantined_paths: vec![],
        degraded_sessions: vec![],
    };
    for (session_id, run_id) in registered_runs {
        let run_root = data_root
            .join("silent-sessions")
            .join(session_id.to_string())
            .join(run_id.to_string());
        if !run_root.is_dir() {
            continue;
        }
        reject_symlink(&run_root)?;
        report.scanned_runs += 1;
        let recovery = run_root.join("recovery");
        secure_dir(&recovery)?;
        let streams = run_root.join("streams");
        let mut indexed = std::collections::BTreeSet::new();
        for entry in fs::read_dir(&run_root)? {
            let path = entry?.path();
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if !name.starts_with("manifest-") || !name.ends_with(".json") {
                continue;
            }
            let verification = (|| -> anyhow::Result<String> {
                reject_symlink(&path)?;
                let manifest: SilentStreamChunkManifest =
                    serde_json::from_slice(&fs::read(&path)?)?;
                anyhow::ensure!(
                    manifest.session_id == *session_id && manifest.run_id == *run_id,
                    "manifest scope mismatch"
                );
                let chunk_path = streams.join(&manifest.file_name);
                reject_symlink(&chunk_path)?;
                let bytes = fs::read(&chunk_path)?;
                anyhow::ensure!(
                    u64::try_from(bytes.len())? == manifest.byte_count,
                    "chunk byte count mismatch"
                );
                anyhow::ensure!(
                    hex::encode(Sha256::digest(&bytes)) == manifest.content_sha256,
                    "chunk hash mismatch"
                );
                Ok(manifest.file_name)
            })();
            match verification {
                Ok(file_name) => {
                    indexed.insert(file_name);
                    report.valid_chunks += 1;
                }
                Err(_) => quarantine(&path, &recovery, &mut report.quarantined_paths)?,
            }
        }
        if streams.is_dir() {
            for entry in fs::read_dir(&streams)? {
                let path = entry?.path();
                let name = path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or_default();
                if name.contains(".partial-")
                    || (name.ends_with(".jsonl") && !indexed.contains(name))
                {
                    quarantine(&path, &recovery, &mut report.quarantined_paths)?;
                }
            }
        }
        if report
            .quarantined_paths
            .iter()
            .any(|path| path.starts_with(&recovery))
            && !report.degraded_sessions.contains(session_id)
        {
            report.degraded_sessions.push(*session_id);
        }
    }
    Ok(report)
}

fn quarantine(path: &Path, recovery: &Path, recorded: &mut Vec<PathBuf>) -> anyhow::Result<()> {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown");
    let destination = recovery.join(format!("{name}.quarantine-{}", Uuid::now_v7()));
    fs::rename(path, &destination)?;
    recorded.push(destination);
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LegacyImportRecord {
    pub alias: String,
    pub session_id: SilentSessionId,
    pub run_id: SilentSessionRunId,
    pub original_log_path: PathBuf,
    pub imported_bytes: u64,
    pub trust: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LegacyImportReport {
    pub records: Vec<LegacyImportRecord>,
    pub rejected_aliases: Vec<String>,
    pub stored_commands_executed: bool,
}

pub fn import_untrusted_legacy_registry(
    registry_path: &Path,
    data_root: &Path,
    secrets: &[String],
) -> anyhow::Result<LegacyImportReport> {
    reject_symlink(registry_path)?;
    let registry_metadata = fs::metadata(registry_path)?;
    anyhow::ensure!(
        registry_metadata.is_file(),
        "legacy registry must be a regular file"
    );
    let value: serde_json::Value = serde_json::from_slice(&fs::read(registry_path)?)?;
    let records = value
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("legacy registry must be an object"))?;
    let sessions_root = data_root.join("silent-sessions");
    secure_dir(&sessions_root)?;
    let mapping_path = sessions_root.join("legacy-import-map.json");
    let mut stable_ids: std::collections::BTreeMap<String, (SilentSessionId, SilentSessionRunId)> =
        if mapping_path.is_file() {
            reject_symlink(&mapping_path)?;
            serde_json::from_slice(&fs::read(&mapping_path)?)?
        } else {
            std::collections::BTreeMap::new()
        };
    let mut report = LegacyImportReport {
        records: vec![],
        rejected_aliases: vec![],
        stored_commands_executed: false,
    };
    for (alias, record) in records {
        let Some(log_path) = record
            .get("log_path")
            .and_then(|value| value.as_str())
            .map(PathBuf::from)
        else {
            report.rejected_aliases.push(alias.clone());
            continue;
        };
        let accepted = (|| -> anyhow::Result<LegacyImportRecord> {
            anyhow::ensure!(log_path.is_absolute(), "legacy log path must be absolute");
            reject_symlink(&log_path)?;
            let metadata = fs::metadata(&log_path)?;
            anyhow::ensure!(metadata.is_file(), "legacy log must be a regular file");
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                anyhow::ensure!(
                    metadata.uid() == registry_metadata.uid(),
                    "legacy registry/log owner mismatch"
                );
            }
            let bytes = fs::read(&log_path)?;
            anyhow::ensure!(
                bytes.len() <= 16 * 1024 * 1024,
                "legacy log exceeds bounded import size"
            );
            let stable = format!("{}\0{}", alias, log_path.display());
            let (session_id, run_id) = *stable_ids
                .entry(stable)
                .or_insert_with(|| (SilentSessionId::new(), SilentSessionRunId::new()));
            let completion_path = sessions_root
                .join(session_id.to_string())
                .join(run_id.to_string())
                .join("artifacts/stream-completion.json");
            let imported_bytes = if completion_path.is_file() {
                let artifact: SilentStreamCompletionArtifact =
                    serde_json::from_slice(&fs::read(completion_path)?)?;
                artifact.total_bytes
            } else {
                let mut writer = RotatingSilentStreamWriter::new(
                    data_root.to_path_buf(),
                    session_id,
                    run_id,
                    "legacy_unverified",
                    SilentStreamKind::Stdout,
                    1024 * 1024,
                )?;
                writer.append(1, &String::from_utf8_lossy(&bytes), secrets)?;
                writer.finish()?.total_bytes
            };
            Ok(LegacyImportRecord {
                alias: alias.clone(),
                session_id,
                run_id,
                original_log_path: log_path.clone(),
                imported_bytes,
                trust: "legacy_unverified".into(),
            })
        })();
        match accepted {
            Ok(record) => report.records.push(record),
            Err(_) => report.rejected_aliases.push(alias.clone()),
        }
    }
    if mapping_path.exists() {
        let previous = mapping_path.with_extension("previous");
        fs::rename(&mapping_path, &previous)?;
        atomic_secure_json(&mapping_path, &stable_ids)?;
        fs::remove_file(previous)?;
    } else {
        atomic_secure_json(&mapping_path, &stable_ids)?;
    }
    Ok(report)
}

fn reject_symlink(path: &Path) -> anyhow::Result<()> {
    if path.exists() {
        anyhow::ensure!(
            !fs::symlink_metadata(path)?.file_type().is_symlink(),
            "silent stream root cannot be a symlink"
        );
    }
    Ok(())
}

fn secure_dir(path: &Path) -> anyhow::Result<()> {
    reject_symlink(path)?;
    fs::create_dir(path).or_else(|error| {
        if error.kind() == std::io::ErrorKind::AlreadyExists {
            Ok(())
        } else {
            Err(error)
        }
    })?;
    reject_symlink(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

fn secure_new_file(path: &Path) -> anyhow::Result<File> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    Ok(options.open(path)?)
}

fn atomic_secure_json<T: Serialize>(path: &Path, value: &T) -> anyhow::Result<()> {
    let temporary = path.with_extension(format!("tmp-{}", Uuid::now_v7()));
    let mut file = secure_new_file(&temporary)?;
    serde_json::to_writer(&mut file, value)?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    crate::durable_fs::atomic_replace(&temporary, path)?;
    crate::durable_fs::sync_directory(path.parent().expect("manifest has parent"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secured_stream_redacts_flushes_and_resumes_from_opaque_cursor() {
        let root = std::env::temp_dir().join(format!("focusa-stream-{}", Uuid::now_v7()));
        fs::create_dir_all(&root).unwrap();
        let session_id = SilentSessionId::new();
        let run_id = SilentSessionRunId::new();
        let mut chunk = SecureSilentStreamChunk::create(
            &root,
            session_id,
            run_id,
            "config-hash",
            SilentStreamKind::Stdout,
            1,
        )
        .unwrap();
        let cursor = chunk
            .append(41, "token=secret", &["secret".into()])
            .unwrap();
        chunk.append(42, "done", &[]).unwrap();
        let manifest = chunk.close().unwrap();
        assert_eq!(
            SilentStreamCursor::decode(&cursor.encode()).unwrap(),
            cursor
        );
        assert_eq!(manifest.first_cursor.seq, 41);
        assert_eq!(manifest.last_cursor.seq, 42);
        assert!(manifest.redaction_applied);
        let body = fs::read_to_string(
            root.join("silent-sessions")
                .join(session_id.to_string())
                .join(run_id.to_string())
                .join("streams/stdout-000001.jsonl"),
        )
        .unwrap();
        assert!(body.contains("[REDACTED]"));
        assert!(!body.contains("secret"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn secured_stream_rejects_sequence_gaps_and_symlink_roots() {
        let root = std::env::temp_dir().join(format!("focusa-stream-{}", Uuid::now_v7()));
        fs::create_dir_all(&root).unwrap();
        let mut chunk = SecureSilentStreamChunk::create(
            &root,
            SilentSessionId::new(),
            SilentSessionRunId::new(),
            "hash",
            SilentStreamKind::Stderr,
            1,
        )
        .unwrap();
        chunk.append(1, "one", &[]).unwrap();
        assert!(chunk.append(3, "gap", &[]).is_err());
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            let link = root.with_extension("link");
            symlink(&root, &link).unwrap();
            assert!(
                SecureSilentStreamChunk::create(
                    &link,
                    SilentSessionId::new(),
                    SilentSessionRunId::new(),
                    "hash",
                    SilentStreamKind::Stdout,
                    1,
                )
                .is_err()
            );
            fs::remove_file(link).unwrap();
        }
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn rotating_stream_emits_terminal_completion_artifact() {
        let root = std::env::temp_dir().join(format!("focusa-rotate-{}", Uuid::now_v7()));
        fs::create_dir_all(&root).unwrap();
        let session_id = SilentSessionId::new();
        let run_id = SilentSessionRunId::new();
        let mut writer = RotatingSilentStreamWriter::new(
            root.clone(),
            session_id,
            run_id,
            "config",
            SilentStreamKind::Semantic,
            1,
        )
        .unwrap();
        writer.append(1, "first", &[]).unwrap();
        writer.append(2, "second", &[]).unwrap();
        let artifact = writer.finish().unwrap();
        assert_eq!(artifact.chunks.len(), 2);
        assert_eq!(artifact.last_cursor.unwrap().seq, 2);
        assert!(artifact.total_bytes > 0);
        assert!(
            root.join("silent-sessions")
                .join(session_id.to_string())
                .join(run_id.to_string())
                .join("artifacts/stream-completion.json")
                .is_file()
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn recovery_quarantines_corrupt_registered_chunks_and_marks_degraded() {
        let root = std::env::temp_dir().join(format!("focusa-recovery-{}", Uuid::now_v7()));
        fs::create_dir_all(&root).unwrap();
        let session_id = SilentSessionId::new();
        let run_id = SilentSessionRunId::new();
        let mut writer = RotatingSilentStreamWriter::new(
            root.clone(),
            session_id,
            run_id,
            "config",
            SilentStreamKind::Stdout,
            1024,
        )
        .unwrap();
        writer.append(1, "valid before tamper", &[]).unwrap();
        let artifact = writer.finish().unwrap();
        let chunk = root
            .join("silent-sessions")
            .join(session_id.to_string())
            .join(run_id.to_string())
            .join("streams")
            .join(&artifact.chunks[0].file_name);
        fs::write(&chunk, "tampered").unwrap();
        let report = recover_registered_streams(&root, &[(session_id, run_id)]).unwrap();
        assert_eq!(report.scanned_runs, 1);
        assert!(report.degraded_sessions.contains(&session_id));
        assert!(report.quarantined_paths.len() >= 2);
        assert!(!chunk.exists());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn legacy_import_is_stable_redacted_and_never_executes_stored_commands() {
        let source = std::env::temp_dir().join(format!("focusa-legacy-{}", Uuid::now_v7()));
        let import_root = source.join("imported");
        fs::create_dir_all(&import_root).unwrap();
        let log = source.join("legacy.log");
        fs::write(&log, "credential=secret").unwrap();
        let registry = source.join("registry.json");
        fs::write(
            &registry,
            serde_json::to_vec(&serde_json::json!({
                "old-session": {"log_path": log, "command": "touch /tmp/must-not-run"},
                "unsafe": {"log_path": "relative.log", "command": "false"}
            }))
            .unwrap(),
        )
        .unwrap();
        let first =
            import_untrusted_legacy_registry(&registry, &import_root, &["secret".into()]).unwrap();
        let second =
            import_untrusted_legacy_registry(&registry, &import_root, &["secret".into()]).unwrap();
        assert_eq!(first.records.len(), 1);
        assert_eq!(first.records[0].session_id, second.records[0].session_id);
        assert_eq!(first.records[0].trust, "legacy_unverified");
        assert_eq!(first.rejected_aliases, vec!["unsafe"]);
        assert!(!first.stored_commands_executed);
        let imported = fs::read_to_string(
            import_root
                .join("silent-sessions")
                .join(first.records[0].session_id.to_string())
                .join(first.records[0].run_id.to_string())
                .join("streams/stdout-000001.jsonl"),
        )
        .unwrap();
        assert!(imported.contains("[REDACTED]"));
        assert!(!imported.contains("secret"));
        let _ = fs::remove_dir_all(source);
    }

    #[test]
    fn bounded_capture_reports_backpressure_without_blocking_or_truth_loss() {
        let root =
            std::env::temp_dir().join(format!("focusa-capture-overflow-{}", uuid::Uuid::now_v7()));
        let spool = root.join("overflow.jsonl");
        let (capture, receiver) = NonBlockingStreamCapture::durable_bounded(1, &spool).unwrap();
        assert_eq!(
            capture.try_capture(StreamCaptureRecord {
                seq: 1,
                data: "one".into()
            }),
            CaptureAdmission::Queued
        );
        assert_eq!(
            capture.try_capture(StreamCaptureRecord {
                seq: 2,
                data: "two".into()
            }),
            CaptureAdmission::BackpressureApplied
        );
        assert_eq!(receiver.recv().unwrap().seq, 1);
        drop(receiver);
        assert_eq!(
            capture.try_capture(StreamCaptureRecord {
                seq: 3,
                data: "three".into()
            }),
            CaptureAdmission::ConsumerDisconnected
        );
        let overflow = fs::read_to_string(&spool).unwrap();
        let records = overflow
            .lines()
            .map(|line| serde_json::from_str::<StreamCaptureRecord>(line).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(
            records.iter().map(|record| record.seq).collect::<Vec<_>>(),
            vec![2, 3]
        );
        assert_eq!(records[0].data, "two");
        assert_eq!(records[1].data, "three");
        let _ = fs::remove_dir_all(root);
    }
}
