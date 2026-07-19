use crate::silent_session::{SilentSessionId, SilentSessionRunId};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub const SILENT_STREAM_MANIFEST_SCHEMA: &str = "focusa.silent_stream_manifest.v1";

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
        fs::rename(&self.partial_path, &final_path)?;
        sync_directory(final_path.parent().expect("stream file has parent"))?;
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
        atomic_secure_json(&self.run_root.join("manifest.json"), &manifest)?;
        Ok(manifest)
    }
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
    fs::rename(&temporary, path)?;
    sync_directory(path.parent().expect("manifest has parent"))
}

fn sync_directory(path: &Path) -> anyhow::Result<()> {
    File::open(path)?.sync_all()?;
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
}
