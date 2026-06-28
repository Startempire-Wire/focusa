//! Persistence-backed pairing store (focusa-oefm).
//!
//! Wraps focusa-core SQLite to read/write pairing codes and connect sessions
//! so daemon restart preserves in-flight pairing flows.

use anyhow::Result;
use serde_json::Value;

use crate::server::AppState;

pub struct PersistedCode {
    pub device_id: String,
    pub device_name: Option<String>,
    pub platform: Option<String>,
    pub daemon_base_url: Option<String>,
    pub scopes: Vec<String>,
    pub expires_at: String,
}

pub struct PersistedSession {
    pub server_url: String,
    pub expires_at: String,
    pub mac_callback: Option<String>,
    pub status: String,
}

#[allow(clippy::too_many_arguments)]
pub fn put_code(
    state: &AppState,
    code: &str,
    device_id: &str,
    device_name: Option<&str>,
    platform: Option<&str>,
    scopes: &[String],
    daemon_base_url: Option<&str>,
    created_at: &str,
    expires_at: &str,
) -> Result<()> {
    let scopes_json = serde_json::to_string(scopes).unwrap_or_else(|_| "[]".into());
    state.persistence.put_pairing_code(
        code,
        device_id,
        device_name,
        platform,
        Some(&scopes_json),
        daemon_base_url,
        created_at,
        expires_at,
    )
}

pub fn get_code(state: &AppState, code: &str) -> Result<Option<PersistedCode>> {
    let row = state.persistence.get_pairing_code(code)?;
    Ok(row.map(|(device_id, expires_at, scopes_json)| {
        // The sqlite row only returns 3 columns here; the full set was written
        // by put_pairing_code. The richer fields are filled in by callers if
        // they have the original DevicePairCode in memory.
        let scopes: Vec<String> = serde_json::from_str(&scopes_json).unwrap_or_default();
        PersistedCode {
            device_id,
            device_name: None,
            platform: None,
            daemon_base_url: None,
            scopes,
            expires_at,
        }
    }))
}

pub fn consume_code(state: &AppState, code: &str) -> Result<()> {
    state.persistence.consume_pairing_code(code)
}

#[allow(clippy::too_many_arguments)]
pub fn put_session(
    state: &AppState,
    connect_id: &str,
    device_id: Option<&str>,
    mac_nonce: Option<&str>,
    mac_pubkey: Option<&str>,
    mac_callback: Option<&str>,
    server_url: &str,
    scopes: Option<&[String]>,
    created_at: &str,
    expires_at: &str,
) -> Result<()> {
    let scopes_json = scopes
        .map(|s| serde_json::to_string(s).unwrap_or_else(|_| "[]".into()))
        .unwrap_or_else(|| "[]".into());
    state.persistence.put_connect_session(
        connect_id,
        device_id,
        mac_nonce,
        mac_pubkey,
        mac_callback,
        server_url,
        Some(&scopes_json),
        created_at,
        expires_at,
    )?;
    // V2: PairingStore durability — force a WAL checkpoint so a SIGKILL
    // of the daemon immediately after put does not lose the row. Without
    // this, the implicit transaction may sit in the WAL buffer and be
    // discarded when the process dies before commit.
    let _ = state.persistence.checkpoint_wal();
    Ok(())
}

pub fn get_session(state: &AppState, connect_id: &str) -> Result<Option<PersistedSession>> {
    let row = state.persistence.get_connect_session(connect_id)?;
    Ok(row.map(|(server_url, expires_at, mac_callback, status)| {
        PersistedSession {
            server_url,
            expires_at,
            mac_callback,
            status,
        }
    }))
}

pub fn complete_session(state: &AppState, connect_id: &str) -> Result<()> {
    state.persistence.complete_connect_session(connect_id)?;
    // Same WAL-checkpoint rationale as put_session above.
    let _ = state.persistence.checkpoint_wal();
    Ok(())
}

/// Helper: parse `scopes` array from a Value, returning an empty Vec on error.
pub fn parse_scopes(v: &Value) -> Vec<String> {
    v.as_array()
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}