//! Versioned process-backend declarations and direct POSIX backend binding.
//!
//! A backend reports only capabilities proved by its current implementation.
//! Unsupported PTY, Windows, resource-control, and restart-survival behavior is
//! represented explicitly and cannot trigger a silent fallback.

use focusa_core::silent_session_protocol::{
    CapabilityRequirement, CapabilitySupport, ProtocolVersionNegotiationError, ProtocolVersionOffer,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

#[cfg(unix)]
use crate::identity::VerifiedExecutionContext;
#[cfg(unix)]
use crate::process_posix::{
    ControlledStopPolicy, ControlledStopReport, ExitedRunRecord, HeartbeatSnapshot,
    NativeAbortDisposition, PosixProcessSupervisor, PosixSpawnRequest, ProcessControlReport,
    SupervisorError,
};
#[cfg(unix)]
use crate::protocol::{ActiveRunRecord, AdoptionDecision, AdoptionExpectation};
#[cfg(unix)]
use chrono::{DateTime, Utc};
#[cfg(unix)]
use focusa_core::silent_session::SilentSessionRunId;

pub const PROCESS_BACKEND_PROTOCOL_SCHEMA: &str = "focusa.process_backend_protocol.v1";
pub const PROCESS_BACKEND_PROTOCOL_VERSION: u32 = 1;
pub const POSIX_DIRECT_BACKEND_ID: &str = "posix_direct";
pub const POSIX_DIRECT_BACKEND_VERSION: &str = "posix_direct.v1";
pub const GENERIC_PTY_BACKEND_ID: &str = "generic_pty";
pub const WINDOWS_JOB_CONPTY_BACKEND_ID: &str = "windows_job_conpty";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackendPlatform {
    Posix,
    Windows,
    PortableUnknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessBackendCapability {
    DetachedExecution,
    ReconnectAfterClientExit,
    SurviveDaemonRestart,
    SurviveMachineReboot,
    StdoutStderrCapture,
    Pty,
    Attach,
    SendText,
    SendKeys,
    ProcessTreeKill,
    HardPause,
    CpuLimit,
    MemoryLimit,
    PidLimit,
    DiskLimit,
}

pub const ALL_PROCESS_BACKEND_CAPABILITIES: [ProcessBackendCapability; 15] = [
    ProcessBackendCapability::DetachedExecution,
    ProcessBackendCapability::ReconnectAfterClientExit,
    ProcessBackendCapability::SurviveDaemonRestart,
    ProcessBackendCapability::SurviveMachineReboot,
    ProcessBackendCapability::StdoutStderrCapture,
    ProcessBackendCapability::Pty,
    ProcessBackendCapability::Attach,
    ProcessBackendCapability::SendText,
    ProcessBackendCapability::SendKeys,
    ProcessBackendCapability::ProcessTreeKill,
    ProcessBackendCapability::HardPause,
    ProcessBackendCapability::CpuLimit,
    ProcessBackendCapability::MemoryLimit,
    ProcessBackendCapability::PidLimit,
    ProcessBackendCapability::DiskLimit,
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessBackendCapabilities {
    pub platform: BackendPlatform,
    pub detached_execution: CapabilitySupport,
    pub reconnect_after_client_exit: CapabilitySupport,
    pub survive_daemon_restart: CapabilitySupport,
    pub survive_machine_reboot: CapabilitySupport,
    pub stdout_stderr_capture: CapabilitySupport,
    pub pty: CapabilitySupport,
    pub attach: CapabilitySupport,
    pub send_text: CapabilitySupport,
    pub send_keys: CapabilitySupport,
    pub process_tree_kill: CapabilitySupport,
    pub hard_pause: CapabilitySupport,
    pub cpu_limit: CapabilitySupport,
    pub memory_limit: CapabilitySupport,
    pub pid_limit: CapabilitySupport,
    pub disk_limit: CapabilitySupport,
}

impl ProcessBackendCapabilities {
    pub fn all(platform: BackendPlatform, support: CapabilitySupport) -> Self {
        Self {
            platform,
            detached_execution: support,
            reconnect_after_client_exit: support,
            survive_daemon_restart: support,
            survive_machine_reboot: support,
            stdout_stderr_capture: support,
            pty: support,
            attach: support,
            send_text: support,
            send_keys: support,
            process_tree_kill: support,
            hard_pause: support,
            cpu_limit: support,
            memory_limit: support,
            pid_limit: support,
            disk_limit: support,
        }
    }

    pub fn support(&self, capability: ProcessBackendCapability) -> CapabilitySupport {
        match capability {
            ProcessBackendCapability::DetachedExecution => self.detached_execution,
            ProcessBackendCapability::ReconnectAfterClientExit => self.reconnect_after_client_exit,
            ProcessBackendCapability::SurviveDaemonRestart => self.survive_daemon_restart,
            ProcessBackendCapability::SurviveMachineReboot => self.survive_machine_reboot,
            ProcessBackendCapability::StdoutStderrCapture => self.stdout_stderr_capture,
            ProcessBackendCapability::Pty => self.pty,
            ProcessBackendCapability::Attach => self.attach,
            ProcessBackendCapability::SendText => self.send_text,
            ProcessBackendCapability::SendKeys => self.send_keys,
            ProcessBackendCapability::ProcessTreeKill => self.process_tree_kill,
            ProcessBackendCapability::HardPause => self.hard_pause,
            ProcessBackendCapability::CpuLimit => self.cpu_limit,
            ProcessBackendCapability::MemoryLimit => self.memory_limit,
            ProcessBackendCapability::PidLimit => self.pid_limit,
            ProcessBackendCapability::DiskLimit => self.disk_limit,
        }
    }

    pub fn explicit_entries(&self) -> BTreeMap<ProcessBackendCapability, CapabilitySupport> {
        ALL_PROCESS_BACKEND_CAPABILITIES
            .into_iter()
            .map(|capability| (capability, self.support(capability)))
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessBackendDescriptor {
    pub schema: String,
    pub backend_id: String,
    pub backend_version: String,
    pub protocol_versions: ProtocolVersionOffer,
    pub capabilities: ProcessBackendCapabilities,
    pub limitations: Vec<String>,
}

impl ProcessBackendDescriptor {
    pub fn negotiate(
        &self,
        request: &ProcessBackendNegotiationRequest,
    ) -> Result<NegotiatedProcessBackend, ProcessBackendNegotiationError> {
        if self.schema != PROCESS_BACKEND_PROTOCOL_SCHEMA
            || self.backend_id.trim().is_empty()
            || self.backend_version.trim().is_empty()
        {
            return Err(ProcessBackendNegotiationError::InvalidDescriptor);
        }
        let selected_protocol_version = self
            .protocol_versions
            .negotiate_highest_common(&request.protocol_versions)?;
        for (capability, requirement) in &request.required_capabilities {
            let actual = self.capabilities.support(*capability);
            if !actual.satisfies(*requirement) {
                return Err(ProcessBackendNegotiationError::RequiredCapabilityMissing {
                    capability: *capability,
                    requirement: *requirement,
                    actual,
                });
            }
        }
        Ok(NegotiatedProcessBackend {
            schema: PROCESS_BACKEND_PROTOCOL_SCHEMA.into(),
            backend_id: self.backend_id.clone(),
            backend_version: self.backend_version.clone(),
            selected_protocol_version,
            capabilities: self.capabilities.clone(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessBackendNegotiationRequest {
    pub protocol_versions: ProtocolVersionOffer,
    pub required_capabilities: BTreeMap<ProcessBackendCapability, CapabilityRequirement>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NegotiatedProcessBackend {
    pub schema: String,
    pub backend_id: String,
    pub backend_version: String,
    pub selected_protocol_version: u32,
    pub capabilities: ProcessBackendCapabilities,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ProcessBackendNegotiationError {
    #[error("process backend descriptor is invalid")]
    InvalidDescriptor,
    #[error("process backend protocol is incompatible: {0}")]
    Protocol(#[from] ProtocolVersionNegotiationError),
    #[error(
        "required process backend capability {capability:?} needs {requirement:?}, actual support is {actual:?}"
    )]
    RequiredCapabilityMissing {
        capability: ProcessBackendCapability,
        requirement: CapabilityRequirement,
        actual: CapabilitySupport,
    },
}

pub trait ProcessBackend {
    fn descriptor(&self) -> ProcessBackendDescriptor;

    fn capabilities(&self) -> ProcessBackendCapabilities {
        self.descriptor().capabilities
    }
}

pub fn posix_direct_descriptor() -> ProcessBackendDescriptor {
    let mut capabilities =
        ProcessBackendCapabilities::all(BackendPlatform::Posix, CapabilitySupport::Unsupported);
    capabilities.detached_execution = CapabilitySupport::Native;
    capabilities.reconnect_after_client_exit = CapabilitySupport::Native;
    capabilities.process_tree_kill = CapabilitySupport::Native;
    capabilities.hard_pause = CapabilitySupport::Native;
    ProcessBackendDescriptor {
        schema: PROCESS_BACKEND_PROTOCOL_SCHEMA.into(),
        backend_id: POSIX_DIRECT_BACKEND_ID.into(),
        backend_version: POSIX_DIRECT_BACKEND_VERSION.into(),
        protocol_versions: ProtocolVersionOffer::new([PROCESS_BACKEND_PROTOCOL_VERSION]),
        capabilities,
        limitations: vec![
            "current direct backend launches null stdin/stdout/stderr; RPC capture is not yet negotiated".into(),
            "daemon restart and machine reboot survival are unsupported".into(),
            "PTY, attach, text/key delivery, and OS resource limits are unsupported".into(),
        ],
    }
}

pub fn generic_pty_backend_descriptor() -> ProcessBackendDescriptor {
    ProcessBackendDescriptor {
        schema: PROCESS_BACKEND_PROTOCOL_SCHEMA.into(),
        backend_id: GENERIC_PTY_BACKEND_ID.into(),
        backend_version: "generic_pty.v1".into(),
        protocol_versions: ProtocolVersionOffer::new([PROCESS_BACKEND_PROTOCOL_VERSION]),
        capabilities: ProcessBackendCapabilities::all(
            BackendPlatform::PortableUnknown,
            CapabilitySupport::Unsupported,
        ),
        limitations: vec![
            "generic PTY backend is declared but has no runtime implementation".into(),
            "selection must fail capability negotiation rather than falling back to an untracked process".into(),
        ],
    }
}

pub fn windows_job_conpty_backend_descriptor() -> ProcessBackendDescriptor {
    ProcessBackendDescriptor {
        schema: PROCESS_BACKEND_PROTOCOL_SCHEMA.into(),
        backend_id: WINDOWS_JOB_CONPTY_BACKEND_ID.into(),
        backend_version: "windows_job_conpty.v1".into(),
        protocol_versions: ProtocolVersionOffer::new([PROCESS_BACKEND_PROTOCOL_VERSION]),
        capabilities: ProcessBackendCapabilities::all(
            BackendPlatform::Windows,
            CapabilitySupport::Unsupported,
        ),
        limitations: vec![
            "Windows Job Object/ConPTY runtime proof has not been completed for Silent Sessions"
                .into(),
            "Windows support is unavailable; no direct-process fallback is permitted".into(),
        ],
    }
}

/// The canonical initial direct backend is a thin capability-bearing binding
/// over the protected owner-scoped POSIX supervisor.
#[cfg(unix)]
pub struct DirectProcessBackend {
    supervisor: PosixProcessSupervisor,
}

#[cfg(unix)]
impl DirectProcessBackend {
    pub fn for_current_user(runner_id: impl Into<String>) -> Result<Self, SupervisorError> {
        Ok(Self {
            supervisor: PosixProcessSupervisor::for_current_user(runner_id)?,
        })
    }

    pub fn active_run_count(&self) -> usize {
        self.supervisor.active_run_count()
    }

    pub fn spawn(
        &mut self,
        context: &VerifiedExecutionContext,
        request: PosixSpawnRequest,
        spawned_at: DateTime<Utc>,
    ) -> Result<ActiveRunRecord, SupervisorError> {
        self.supervisor.spawn(context, request, spawned_at)
    }

    pub fn heartbeat(
        &mut self,
        observed_at: DateTime<Utc>,
    ) -> Result<HeartbeatSnapshot, SupervisorError> {
        self.supervisor.heartbeat(observed_at)
    }

    pub fn evaluate_adoption(
        &mut self,
        expectation: &AdoptionExpectation,
        observed_at: DateTime<Utc>,
    ) -> Result<AdoptionDecision, SupervisorError> {
        self.supervisor.evaluate_adoption(expectation, observed_at)
    }

    pub fn hard_pause(
        &mut self,
        run_id: SilentSessionRunId,
        generation: u64,
        observed_at: DateTime<Utc>,
    ) -> Result<ProcessControlReport, SupervisorError> {
        self.supervisor.hard_pause(run_id, generation, observed_at)
    }

    pub fn hard_resume(
        &mut self,
        run_id: SilentSessionRunId,
        generation: u64,
        observed_at: DateTime<Utc>,
    ) -> Result<ProcessControlReport, SupervisorError> {
        self.supervisor.hard_resume(run_id, generation, observed_at)
    }

    pub async fn controlled_stop<F>(
        &mut self,
        run_id: SilentSessionRunId,
        policy: ControlledStopPolicy,
        native_abort: F,
    ) -> Result<ControlledStopReport, SupervisorError>
    where
        F: FnMut(&ActiveRunRecord) -> NativeAbortDisposition,
    {
        self.supervisor
            .controlled_stop(run_id, policy, native_abort)
            .await
    }

    pub async fn force_terminate(
        &mut self,
        run_id: SilentSessionRunId,
        observed_at: DateTime<Utc>,
    ) -> Result<ExitedRunRecord, SupervisorError> {
        self.supervisor.force_terminate(run_id, observed_at).await
    }
}

#[cfg(unix)]
impl ProcessBackend for DirectProcessBackend {
    fn descriptor(&self) -> ProcessBackendDescriptor {
        posix_direct_descriptor()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn requirement(
        capability: ProcessBackendCapability,
        support: CapabilityRequirement,
    ) -> ProcessBackendNegotiationRequest {
        ProcessBackendNegotiationRequest {
            protocol_versions: ProtocolVersionOffer::new([PROCESS_BACKEND_PROTOCOL_VERSION]),
            required_capabilities: BTreeMap::from([(capability, support)]),
        }
    }

    #[test]
    fn direct_backend_negotiates_only_implemented_process_capabilities() {
        let descriptor = posix_direct_descriptor();
        assert_eq!(
            descriptor.capabilities.explicit_entries().len(),
            ALL_PROCESS_BACKEND_CAPABILITIES.len()
        );
        assert!(
            descriptor
                .negotiate(&requirement(
                    ProcessBackendCapability::ProcessTreeKill,
                    CapabilityRequirement::Native,
                ))
                .is_ok()
        );
        assert!(
            descriptor
                .negotiate(&requirement(
                    ProcessBackendCapability::HardPause,
                    CapabilityRequirement::Native,
                ))
                .is_ok()
        );
        assert_eq!(
            descriptor.negotiate(&requirement(
                ProcessBackendCapability::StdoutStderrCapture,
                CapabilityRequirement::Available,
            )),
            Err(ProcessBackendNegotiationError::RequiredCapabilityMissing {
                capability: ProcessBackendCapability::StdoutStderrCapture,
                requirement: CapabilityRequirement::Available,
                actual: CapabilitySupport::Unsupported,
            })
        );
    }

    #[test]
    fn unimplemented_pty_and_windows_backends_are_explicitly_unsupported() {
        for descriptor in [
            generic_pty_backend_descriptor(),
            windows_job_conpty_backend_descriptor(),
        ] {
            assert!(
                descriptor
                    .capabilities
                    .explicit_entries()
                    .values()
                    .all(|support| *support == CapabilitySupport::Unsupported)
            );
            assert!(matches!(
                descriptor.negotiate(&requirement(
                    ProcessBackendCapability::DetachedExecution,
                    CapabilityRequirement::Available,
                )),
                Err(ProcessBackendNegotiationError::RequiredCapabilityMissing { .. })
            ));
        }
    }

    #[test]
    fn process_backend_protocol_mismatch_blocks_selection() {
        let request = ProcessBackendNegotiationRequest {
            protocol_versions: ProtocolVersionOffer::new([99]),
            required_capabilities: BTreeMap::new(),
        };
        assert_eq!(
            posix_direct_descriptor().negotiate(&request),
            Err(ProcessBackendNegotiationError::Protocol(
                ProtocolVersionNegotiationError::ProtocolIncompatible
            ))
        );
    }
}
