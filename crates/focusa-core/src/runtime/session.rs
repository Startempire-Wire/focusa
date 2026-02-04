//! Session management.
//!
//! Session Identity Invariants:
//!   1. All state mutations must include session_id
//!   2. Reducer rejects cross-session writes
//!   3. Events without session_id are invalid

use crate::types::{SessionId, SessionMeta, SessionStatus};
use chrono::Utc;
use uuid::Uuid;

/// Create a new session.
pub fn create_session(
    adapter_id: Option<String>,
    workspace_id: Option<String>,
) -> SessionMeta {
    SessionMeta {
        session_id: Uuid::now_v7(),
        created_at: Utc::now(),
        adapter_id,
        workspace_id,
        status: SessionStatus::Active,
    }
}

/// Close an existing session.
pub fn close_session(session: &mut SessionMeta) {
    session.status = SessionStatus::Closed;
}

/// Validate a session_id matches the active session.
pub fn validate_session(
    active: Option<&SessionMeta>,
    session_id: SessionId,
) -> Result<(), String> {
    match active {
        Some(s) if s.session_id == session_id => Ok(()),
        Some(s) => Err(format!(
            "Cross-session write rejected: active={}, attempted={}",
            s.session_id, session_id
        )),
        None => Err("No active session".into()),
    }
}
