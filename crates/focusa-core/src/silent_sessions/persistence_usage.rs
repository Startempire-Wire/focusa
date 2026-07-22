use rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::runtime::persistence_sqlite::SqlitePersistence;

use super::{SilentSessionId, SilentSessionRunId};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SilentSessionUsageSummary {
    pub silent_session_id: SilentSessionId,
    pub run_id: SilentSessionRunId,
    pub lifecycle_event_count: u64,
    pub stream_event_count: u64,
    pub stream_chunk_count: u64,
    pub uncompressed_bytes: u64,
    pub compressed_bytes: u64,
}

pub fn load_usage_summary(
    persistence: &SqlitePersistence,
    session_id: SilentSessionId,
    run_id: SilentSessionRunId,
) -> anyhow::Result<SilentSessionUsageSummary> {
    persistence.with_connection_mut(|connection| {
        let lifecycle_event_count = connection.query_row(
            "SELECT COUNT(*) FROM silent_session_events WHERE silent_session_id=?1 AND run_id=?2",
            params![session_id.to_string(), run_id.to_string()],
            |row| row.get::<_, i64>(0),
        )?;
        let (chunks, events, uncompressed, compressed) = connection.query_row(
            r#"SELECT COUNT(*),COALESCE(SUM(event_count),0),
               COALESCE(SUM(uncompressed_bytes),0),COALESCE(SUM(compressed_bytes),0)
               FROM silent_session_stream_indexes WHERE silent_session_id=?1 AND run_id=?2"#,
            params![session_id.to_string(), run_id.to_string()],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                ))
            },
        )?;
        Ok(SilentSessionUsageSummary {
            silent_session_id: session_id,
            run_id,
            lifecycle_event_count: lifecycle_event_count as u64,
            stream_event_count: events as u64,
            stream_chunk_count: chunks as u64,
            uncompressed_bytes: uncompressed as u64,
            compressed_bytes: compressed as u64,
        })
    })
}
