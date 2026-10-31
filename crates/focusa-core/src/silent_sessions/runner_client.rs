//! Daemon-side protected local transport for per-user session runners.

use std::{path::Path, time::Duration};

#[cfg(unix)]
use std::os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt};

#[cfg(unix)]
use chrono::Utc;
#[cfg(any(unix, windows))]
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    time::timeout,
};
#[cfg(unix)]
use tokio::net::UnixStream;

use crate::runtime::persistence_sqlite::SqlitePersistence;

#[cfg(unix)]
use super::{RUNNER_PROTOCOL_SCHEMA, consume_runner_nonce};
use super::{RunnerWireRequest, RunnerWireResponse};

#[cfg(any(unix, windows))]
const MAX_RESPONSE_BYTES: usize = 1024 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum RunnerClientError {
    #[error("runner socket path must be absolute")]
    RelativeSocket,
    #[error("runner socket is not a Unix socket")]
    NotSocket,
    #[error("runner socket owner does not match verified project owner")]
    OwnerMismatch,
    #[error("runner socket permissions allow group or other access")]
    UnsafePermissions,
    #[error("runner nonce was already durably consumed")]
    Replay,
    #[error("runner request delivery timed out and must be reconciled by exact query")]
    AmbiguousDelivery,
    #[error("runner response exceeds one MiB")]
    OversizedResponse,
    #[error("protected runner socket transport is unavailable on this platform")]
    UnsupportedPlatform,
    #[error("runner response target or schema mismatch")]
    ResponseMismatch,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[cfg(unix)]
pub async fn send_runner_request(
    persistence: &SqlitePersistence,
    socket_path: &Path,
    expected_owner_uid: u32,
    request: &RunnerWireRequest,
    timeout_duration: Duration,
) -> Result<RunnerWireResponse, RunnerClientError> {
    verify_runner_socket(socket_path, expected_owner_uid)?;
    request
        .validate_binding()
        .map_err(RunnerClientError::Other)?;
    if !consume_runner_nonce(persistence, &request.command, Utc::now())? {
        return Err(RunnerClientError::Replay);
    }
    let exchange = async {
        let mut stream = UnixStream::connect(socket_path).await?;
        stream.write_all(&serde_json::to_vec(request)?).await?;
        stream.write_all(b"\n").await?;
        let reader = BufReader::new(stream);
        let mut limited = reader.take((MAX_RESPONSE_BYTES + 1) as u64);
        let mut response = Vec::new();
        limited.read_until(b'\n', &mut response).await?;
        if response.len() > MAX_RESPONSE_BYTES {
            return Err(RunnerClientError::OversizedResponse);
        }
        let raw = response.clone();
        let response: RunnerWireResponse = serde_json::from_slice(&raw).map_err(|error| {
            RunnerClientError::Other(anyhow::anyhow!(
                "runner did not return a protocol response ({error}); body: {}",
                String::from_utf8_lossy(&raw).trim()
            ))
        })?;
        if response.schema != RUNNER_PROTOCOL_SCHEMA
            || response.session_id != request.command.session_id
            || response.run_id != request.command.run_id
        {
            return Err(RunnerClientError::ResponseMismatch);
        }
        Ok(response)
    };
    timeout(timeout_duration, exchange)
        .await
        .map_err(|_| RunnerClientError::AmbiguousDelivery)?
}

#[cfg(windows)]
pub async fn send_runner_request(
    persistence: &SqlitePersistence,
    socket_path: &Path,
    _expected_owner_uid: u32,
    request: &RunnerWireRequest,
    timeout_duration: Duration,
) -> Result<RunnerWireResponse, RunnerClientError> {
    use tokio::net::windows::named_pipe::ClientOptions;

    // Windows transports are named pipes. The pipe ACL is established by the
    // runner when it creates the pipe (Spec 133 P6); the daemon only accepts a
    // canonical pipe path so it can never be pointed at an arbitrary device.
    let pipe_name = socket_path
        .to_str()
        .ok_or(RunnerClientError::RelativeSocket)?;
    if !pipe_name.starts_with(r"\\.\pipe\") {
        return Err(RunnerClientError::NotSocket);
    }
    request
        .validate_binding()
        .map_err(RunnerClientError::Other)?;
    if !consume_runner_nonce(persistence, &request.command, Utc::now())? {
        return Err(RunnerClientError::Replay);
    }
    let exchange = async {
        let mut stream = ClientOptions::new()
            .open(pipe_name)
            .map_err(RunnerClientError::from)?;
        stream
            .write_all(&serde_json::to_vec(request)?)
            .await?;
        stream.write_all(b"\n").await?;
        let reader = BufReader::new(stream);
        let mut limited = reader.take((MAX_RESPONSE_BYTES + 1) as u64);
        let mut response = Vec::new();
        limited.read_until(b'\n', &mut response).await?;
        if response.len() > MAX_RESPONSE_BYTES {
            return Err(RunnerClientError::OversizedResponse);
        }
        let response: RunnerWireResponse = serde_json::from_slice(&response)?;
        if response.schema != RUNNER_PROTOCOL_SCHEMA
            || response.session_id != request.command.session_id
            || response.run_id != request.command.run_id
        {
            return Err(RunnerClientError::ResponseMismatch);
        }
        Ok(response)
    };
    timeout(timeout_duration, exchange)
        .await
        .map_err(|_| RunnerClientError::AmbiguousDelivery)?
}

#[cfg(unix)]
fn verify_runner_socket(
    socket_path: &Path,
    expected_owner_uid: u32,
) -> Result<(), RunnerClientError> {
    if !socket_path.is_absolute() {
        return Err(RunnerClientError::RelativeSocket);
    }
    let metadata = std::fs::symlink_metadata(socket_path).map_err(anyhow::Error::from)?;
    if !metadata.file_type().is_socket() {
        return Err(RunnerClientError::NotSocket);
    }
    if metadata.uid() != expected_owner_uid {
        return Err(RunnerClientError::OwnerMismatch);
    }
    if metadata.permissions().mode() & 0o077 != 0 {
        return Err(RunnerClientError::UnsafePermissions);
    }
    Ok(())
}
