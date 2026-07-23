//! Typed protected protocol between the daemon and a per-user session runner.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{
    AuthenticatedRunnerCommand, LaunchManifest, SilentSessionAction, SilentSessionId,
    SilentSessionRunId,
};

pub const RUNNER_PROTOCOL_SCHEMA: &str = "focusa.session_runner_protocol.v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum RunnerExecutionMode {
    EmbeddedSameUser,
    PerUserSocket { socket_scope: String },
}

pub fn select_runner_execution_mode(
    daemon_os_user: &str,
    project_owner_os_user: &str,
    socket_scope: Option<&str>,
) -> anyhow::Result<RunnerExecutionMode> {
    if daemon_os_user == project_owner_os_user {
        return Ok(RunnerExecutionMode::EmbeddedSameUser);
    }
    let socket_scope = socket_scope
        .filter(|scope| !scope.trim().is_empty())
        .ok_or_else(|| {
            anyhow::anyhow!("cross-user execution requires a protected user-scoped runner socket")
        })?;
    Ok(RunnerExecutionMode::PerUserSocket {
        socket_scope: socket_scope.to_string(),
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunnerLaunchSpec {
    pub manifest: LaunchManifest,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RunnerSignal {
    Pause,
    Resume,
    Interrupt,
    Cancel,
    ForceKill,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "operation", rename_all = "snake_case")]
pub enum RunnerOperation {
    Launch { spec: RunnerLaunchSpec },
    Signal { signal: RunnerSignal },
    Query,
    Heartbeat,
    Adopt { expected: ProcessTreeIdentity },
}

impl RunnerOperation {
    pub fn required_action(&self) -> SilentSessionAction {
        match self {
            Self::Launch { .. } => SilentSessionAction::Start,
            Self::Signal { signal } => match signal {
                RunnerSignal::Pause => SilentSessionAction::Pause,
                RunnerSignal::Resume => SilentSessionAction::Resume,
                RunnerSignal::Interrupt => SilentSessionAction::Interrupt,
                RunnerSignal::Cancel => SilentSessionAction::Cancel,
                RunnerSignal::ForceKill => SilentSessionAction::ForceKill,
            },
            Self::Query | Self::Heartbeat => SilentSessionAction::Show,
            Self::Adopt { .. } => SilentSessionAction::Adopt,
        }
    }

    pub fn action_digest(&self) -> anyhow::Result<String> {
        let action = self.required_action();
        Ok(hex::encode(Sha256::digest(serde_json::to_vec(&action)?)))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunnerWireRequest {
    pub schema: String,
    pub command: AuthenticatedRunnerCommand,
    pub operation: RunnerOperation,
}

impl RunnerWireRequest {
    pub fn payload_bytes(&self) -> anyhow::Result<Vec<u8>> {
        serde_json::to_vec(&self.operation).map_err(Into::into)
    }

    pub fn validate_binding(&self) -> anyhow::Result<Vec<u8>> {
        anyhow::ensure!(
            self.schema == RUNNER_PROTOCOL_SCHEMA,
            "unsupported runner protocol schema"
        );
        anyhow::ensure!(
            self.command.action_digest == self.operation.action_digest()?,
            "runner action digest mismatch"
        );
        let payload = self.payload_bytes()?;
        anyhow::ensure!(
            self.command.payload_hash == hex::encode(Sha256::digest(&payload)),
            "runner payload hash mismatch"
        );
        Ok(payload)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProcessTreeIdentity {
    pub session_id: SilentSessionId,
    pub run_id: SilentSessionRunId,
    pub pid: u32,
    pub process_group_id: i32,
    pub owner_os_user: String,
    pub workspace: String,
    pub manifest_digest: String,
    pub started_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunnerHeartbeat {
    pub runner_principal_id: String,
    pub owner_os_user: String,
    pub socket_scope: String,
    pub observed_at: DateTime<Utc>,
    pub active_processes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RunnerProcessState {
    Running,
    Paused,
    Exited,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunnerProcessProjection {
    pub identity: ProcessTreeIdentity,
    pub state: RunnerProcessState,
    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RunnerWireResponse {
    pub schema: String,
    pub ok: bool,
    pub status: String,
    pub session_id: SilentSessionId,
    pub run_id: SilentSessionRunId,
    pub replayed: bool,
    #[serde(default)]
    pub process: Option<RunnerProcessProjection>,
    #[serde(default)]
    pub heartbeat: Option<RunnerHeartbeat>,
    #[serde(default)]
    pub details: BTreeMap<String, serde_json::Value>,
}
