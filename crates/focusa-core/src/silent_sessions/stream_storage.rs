use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use crate::runtime::persistence_sqlite::SqlitePersistence;
use chrono::Utc;
use rusqlite::{OptionalExtension, params};
use serde::{Deserialize, Serialize};

use super::{
    CanonicalStreamEvent, OutputChannel, STREAM_CHUNK_CODEC_VERSION, SilentSessionId,
    SilentSessionRunId, StreamCursor, compress_chunk, decompress_chunk,
    secure_fs::{
        atomic_publish, create_secure_descendants, create_secure_root, reject_symlink,
        relative_ref, secure_read,
    },
    sha256_hex,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StreamChunkManifest {
    pub session_id: SilentSessionId,
    pub run_id: SilentSessionRunId,
    pub channel: OutputChannel,
    pub chunk_sequence: u64,
    pub first_event_sequence: u64,
    pub last_event_sequence: u64,
    pub event_count: u64,
    pub uncompressed_bytes: u64,
    pub compressed_bytes: u64,
    pub codec_version: u32,
    pub chunk_hash: String,
    pub chunk_ref: String,
    pub redaction_applied: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishedChunk {
    pub manifest: StreamChunkManifest,
    pub cursor: String,
    pub replayed: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum StreamStorageError {
    #[error("stream root must be absolute")]
    RootNotAbsolute,
    #[error("stream storage path is outside the registered root")]
    PathOutsideRoot,
    #[error("symlink or non-directory detected in stream storage path: {0}")]
    UnsafePath(String),
    #[error("stream chunk must contain at least one event")]
    EmptyChunk,
    #[error(
        "stream events must have matching session, run, channel and strictly increasing sequence"
    )]
    InvalidEventOrder,
    #[error("stream chunk sequence or event sequence is not monotonic")]
    IndexPositionMismatch,
    #[error("unindexed stream chunk already exists: {0}")]
    OrphanChunk(String),
    #[error("durable chunk checksum mismatch")]
    ChecksumMismatch,
    #[error("cursor run does not match requested run")]
    CursorRunMismatch,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Clone)]
pub struct SecureStreamStore {
    root: PathBuf,
    persistence: SqlitePersistence,
}

impl SecureStreamStore {
    pub fn new(
        root: impl Into<PathBuf>,
        persistence: SqlitePersistence,
    ) -> Result<Self, StreamStorageError> {
        let root = root.into();
        if !root.is_absolute() {
            return Err(StreamStorageError::RootNotAbsolute);
        }
        create_secure_root(&root)?;
        Ok(Self { root, persistence })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn publish_chunk(
        &self,
        session_id: SilentSessionId,
        run_id: SilentSessionRunId,
        channel: OutputChannel,
        chunk_sequence: u64,
        events: &[CanonicalStreamEvent],
    ) -> Result<PublishedChunk, StreamStorageError> {
        validate_events(session_id, run_id, channel, events)?;
        let uncompressed = encode_events(events)?;
        let compressed = compress_chunk(&uncompressed);
        let chunk_hash = sha256_hex(&compressed);
        let first_sequence = events[0].seq;
        let last_sequence = events[events.len() - 1].seq;
        let directory = self.chunk_directory(session_id, run_id, channel)?;
        let final_path = directory.join(format!("chunk-{chunk_sequence:020}.fss"));
        let chunk_ref = relative_ref(&self.root, &final_path)?;
        let manifest = StreamChunkManifest {
            session_id,
            run_id,
            channel,
            chunk_sequence,
            first_event_sequence: first_sequence,
            last_event_sequence: last_sequence,
            event_count: events.len() as u64,
            uncompressed_bytes: uncompressed.len() as u64,
            compressed_bytes: compressed.len() as u64,
            codec_version: STREAM_CHUNK_CODEC_VERSION,
            chunk_hash: chunk_hash.clone(),
            chunk_ref: chunk_ref.clone(),
            redaction_applied: true,
        };

        if let Some(existing) = self.load_index(session_id, run_id, channel, chunk_sequence)? {
            if existing.chunk_hash != chunk_hash {
                return Err(StreamStorageError::ChecksumMismatch);
            }
            return Ok(PublishedChunk {
                cursor: StreamCursor::new(run_id, existing.last_event_sequence)
                    .encode()
                    .map_err(anyhow::Error::from)?,
                manifest: existing,
                replayed: true,
            });
        }
        if final_path.exists() {
            return Err(StreamStorageError::OrphanChunk(
                final_path.display().to_string(),
            ));
        }
        self.validate_index_position(&manifest)?;

        atomic_publish(&directory, &final_path, &compressed)?;
        self.insert_index(&manifest)?;
        Ok(PublishedChunk {
            cursor: StreamCursor::new(run_id, last_sequence)
                .encode()
                .map_err(anyhow::Error::from)?,
            manifest,
            replayed: false,
        })
    }

    pub fn resume_position(
        &self,
        session_id: SilentSessionId,
        run_id: SilentSessionRunId,
        channel: OutputChannel,
    ) -> Result<(u64, u64), StreamStorageError> {
        let position = self.persistence.with_connection_mut(|connection| {
            let next_chunk = connection.query_row(
                "SELECT COALESCE(MAX(chunk_sequence)+1,0) FROM silent_session_stream_indexes WHERE silent_session_id=?1 AND run_id=?2 AND stream_name=?3",
                params![session_id.to_string(), run_id.to_string(), channel.as_str()],
                |row| row.get::<_, u64>(0),
            )?;
            let last_sequence = connection.query_row(
                "SELECT COALESCE(MAX(last_event_sequence),0) FROM silent_session_stream_indexes WHERE silent_session_id=?1 AND run_id=?2",
                params![session_id.to_string(), run_id.to_string()],
                |row| row.get::<_, u64>(0),
            )?;
            Ok((next_chunk, last_sequence))
        })?;
        Ok(position)
    }

    pub fn read_after(
        &self,
        session_id: SilentSessionId,
        run_id: SilentSessionRunId,
        channel: OutputChannel,
        cursor: Option<&str>,
        max_events: usize,
    ) -> Result<(Vec<CanonicalStreamEvent>, Option<String>), StreamStorageError> {
        let after_sequence = match cursor {
            Some(encoded) => {
                let decoded = StreamCursor::decode(encoded).map_err(anyhow::Error::from)?;
                if decoded.run_id != run_id {
                    return Err(StreamStorageError::CursorRunMismatch);
                }
                decoded.sequence
            }
            None => 0,
        };
        if max_events == 0 {
            return Ok((Vec::new(), cursor.map(ToOwned::to_owned)));
        }
        let indexes = self.load_indexes_after(session_id, run_id, channel, after_sequence)?;
        let mut events = Vec::new();
        let mut seen = HashSet::new();
        for manifest in indexes {
            let path = self.resolve_chunk_ref(&manifest.chunk_ref)?;
            let compressed = secure_read(&path, manifest.compressed_bytes)?;
            if sha256_hex(&compressed) != manifest.chunk_hash {
                return Err(StreamStorageError::ChecksumMismatch);
            }
            let uncompressed = decompress_chunk(&compressed).map_err(anyhow::Error::from)?;
            for event in decode_events(&uncompressed)? {
                if event.seq > after_sequence && seen.insert(event.event_id) {
                    events.push(event);
                    if events.len() == max_events {
                        let cursor = StreamCursor::new(run_id, events[events.len() - 1].seq)
                            .encode()
                            .map_err(anyhow::Error::from)?;
                        return Ok((events, Some(cursor)));
                    }
                }
            }
        }
        let next_cursor = events
            .last()
            .map(|event| StreamCursor::new(run_id, event.seq).encode())
            .transpose()
            .map_err(anyhow::Error::from)?
            .or_else(|| cursor.map(ToOwned::to_owned));
        Ok((events, next_cursor))
    }

    fn chunk_directory(
        &self,
        session_id: SilentSessionId,
        run_id: SilentSessionRunId,
        channel: OutputChannel,
    ) -> Result<PathBuf, StreamStorageError> {
        let path = self
            .root
            .join(session_id.to_string())
            .join(run_id.to_string())
            .join(channel.as_str());
        create_secure_descendants(&self.root, &path)?;
        Ok(path)
    }

    fn resolve_chunk_ref(&self, chunk_ref: &str) -> Result<PathBuf, StreamStorageError> {
        let path = self.root.join(chunk_ref);
        if !path.starts_with(&self.root) || chunk_ref.contains("..") {
            return Err(StreamStorageError::PathOutsideRoot);
        }
        reject_symlink(&path)?;
        Ok(path)
    }

    fn validate_index_position(
        &self,
        manifest: &StreamChunkManifest,
    ) -> Result<(), StreamStorageError> {
        let valid = self.persistence.with_connection_mut(|connection| {
            let previous_chunk = connection.query_row(
                "SELECT MAX(chunk_sequence) FROM silent_session_stream_indexes WHERE silent_session_id=?1 AND run_id=?2 AND stream_name=?3",
                params![
                    manifest.session_id.to_string(),
                    manifest.run_id.to_string(),
                    manifest.channel.as_str(),
                ],
                |row| row.get::<_, Option<u64>>(0),
            )?;
            let previous_event = connection.query_row(
                "SELECT MAX(last_event_sequence) FROM silent_session_stream_indexes WHERE silent_session_id=?1 AND run_id=?2",
                params![manifest.session_id.to_string(), manifest.run_id.to_string()],
                |row| row.get::<_, Option<u64>>(0),
            )?;
            let expected_chunk = previous_chunk.map_or(0, |value| value + 1);
            Ok(manifest.chunk_sequence == expected_chunk
                && previous_event.is_none_or(|value| manifest.first_event_sequence > value))
        })?;
        if !valid {
            return Err(StreamStorageError::IndexPositionMismatch);
        }
        Ok(())
    }

    fn insert_index(&self, manifest: &StreamChunkManifest) -> Result<(), StreamStorageError> {
        self.persistence.with_connection_mut(|connection| {
            let transaction = connection.transaction()?;
            transaction.execute(
                r#"INSERT INTO silent_session_stream_indexes(
                   silent_session_id,run_id,stream_name,chunk_sequence,chunk_ref,byte_start,byte_end,
                   chunk_hash,codec_version,first_event_sequence,last_event_sequence,event_count,
                   uncompressed_bytes,compressed_bytes,redaction_applied,created_at
                   ) VALUES (?1,?2,?3,?4,?5,0,?6,?7,?8,?9,?10,?11,?12,?13,1,?14)"#,
                params![
                    manifest.session_id.to_string(),
                    manifest.run_id.to_string(),
                    manifest.channel.as_str(),
                    manifest.chunk_sequence,
                    manifest.chunk_ref,
                    manifest.compressed_bytes,
                    manifest.chunk_hash,
                    manifest.codec_version,
                    manifest.first_event_sequence,
                    manifest.last_event_sequence,
                    manifest.event_count,
                    manifest.uncompressed_bytes,
                    manifest.compressed_bytes,
                    Utc::now().to_rfc3339(),
                ],
            )?;
            transaction.commit()?;
            Ok(())
        })?;
        Ok(())
    }

    fn load_index(
        &self,
        session_id: SilentSessionId,
        run_id: SilentSessionRunId,
        channel: OutputChannel,
        chunk_sequence: u64,
    ) -> Result<Option<StreamChunkManifest>, StreamStorageError> {
        let result = self.persistence.with_connection_mut(|connection| {
            connection
                .query_row(
                    &index_select("AND chunk_sequence=?4"),
                    params![
                        session_id.to_string(),
                        run_id.to_string(),
                        channel.as_str(),
                        chunk_sequence,
                    ],
                    index_from_row,
                )
                .optional()
                .map_err(Into::into)
        })?;
        Ok(result)
    }

    fn load_indexes_after(
        &self,
        session_id: SilentSessionId,
        run_id: SilentSessionRunId,
        channel: OutputChannel,
        sequence: u64,
    ) -> Result<Vec<StreamChunkManifest>, StreamStorageError> {
        let indexes = self.persistence.with_connection_mut(|connection| {
            let mut statement = connection.prepare(&format!(
                "{} AND last_event_sequence>?4 ORDER BY chunk_sequence",
                index_select("")
            ))?;
            let rows = statement.query_map(
                params![
                    session_id.to_string(),
                    run_id.to_string(),
                    channel.as_str(),
                    sequence,
                ],
                index_from_row,
            )?;
            rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
        })?;
        Ok(indexes)
    }
}

fn index_select(suffix: &str) -> String {
    format!(
        r#"SELECT silent_session_id,run_id,stream_name,chunk_sequence,chunk_ref,chunk_hash,
           codec_version,first_event_sequence,last_event_sequence,event_count,uncompressed_bytes,
           compressed_bytes,redaction_applied FROM silent_session_stream_indexes
           WHERE silent_session_id=?1 AND run_id=?2 AND stream_name=?3 {suffix}"#
    )
}

fn index_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<StreamChunkManifest> {
    let channel: String = row.get(2)?;
    Ok(StreamChunkManifest {
        session_id: parse_id(row.get::<_, String>(0)?)?,
        run_id: parse_id(row.get::<_, String>(1)?)?,
        channel: parse_channel(&channel)?,
        chunk_sequence: row.get(3)?,
        chunk_ref: row.get(4)?,
        chunk_hash: row.get(5)?,
        codec_version: row.get(6)?,
        first_event_sequence: row.get(7)?,
        last_event_sequence: row.get(8)?,
        event_count: row.get(9)?,
        uncompressed_bytes: row.get(10)?,
        compressed_bytes: row.get(11)?,
        redaction_applied: row.get(12)?,
    })
}

fn parse_id<T: std::str::FromStr>(value: String) -> rusqlite::Result<T> {
    value.parse().map_err(|_| {
        rusqlite::Error::FromSqlConversionFailure(
            value.len(),
            rusqlite::types::Type::Text,
            "invalid UUIDv7 identifier".into(),
        )
    })
}

fn parse_channel(value: &str) -> rusqlite::Result<OutputChannel> {
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
    .into_iter()
    .find(|channel| channel.as_str() == value)
    .ok_or_else(|| {
        rusqlite::Error::FromSqlConversionFailure(
            value.len(),
            rusqlite::types::Type::Text,
            "invalid output channel".into(),
        )
    })
}

fn validate_events(
    session_id: SilentSessionId,
    run_id: SilentSessionRunId,
    channel: OutputChannel,
    events: &[CanonicalStreamEvent],
) -> Result<(), StreamStorageError> {
    if events.is_empty() {
        return Err(StreamStorageError::EmptyChunk);
    }
    let mut previous = 0;
    let mut event_ids = HashSet::new();
    for event in events {
        event.validate().map_err(anyhow::Error::from)?;
        if event.session_id != session_id
            || event.run_id != run_id
            || event.channel != channel
            || event.seq <= previous
            || !event_ids.insert(event.event_id)
        {
            return Err(StreamStorageError::InvalidEventOrder);
        }
        previous = event.seq;
    }
    Ok(())
}

fn encode_events(events: &[CanonicalStreamEvent]) -> Result<Vec<u8>, StreamStorageError> {
    let mut encoded = Vec::new();
    for event in events {
        serde_json::to_writer(&mut encoded, event).map_err(anyhow::Error::from)?;
        encoded.push(b'\n');
    }
    Ok(encoded)
}

fn decode_events(bytes: &[u8]) -> Result<Vec<CanonicalStreamEvent>, StreamStorageError> {
    bytes
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| serde_json::from_slice(line).map_err(anyhow::Error::from))
        .collect::<Result<Vec<_>, _>>()
        .map_err(StreamStorageError::from)
}
