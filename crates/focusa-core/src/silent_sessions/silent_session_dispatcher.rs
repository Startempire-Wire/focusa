//! Spec133 §7/§13 dispatch: turn an accepted request into a signed launch that a
//! live runner actually receives.
//!
//! Truthfulness rule: every fact in the manifest is supplied by the caller from
//! verified state. This module never invents trust, executables or missions, and
//! it returns an error rather than reporting success when delivery fails.

use anyhow::{Context, bail};
use chrono::{Duration as ChronoDuration, Utc};
use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;

use super::{SilentSessionId, SilentSessionRunId};
use super::launch_manifest::{
    LaunchManifest, LaunchResourceLimits, LaunchResourceMode, LaunchTrustPolicy, MissionDelivery,
    ProcessBackend, ResourceModeRequirement, ResourceModeResolution, StdioMode,
};
use super::runner_client::send_runner_request;
use super::runner_protocol::{
    RUNNER_PROTOCOL_SCHEMA, RunnerLaunchSpec, RunnerOperation, RunnerWireRequest,
};
use super::runner_provisioning::RunnerPaths;
use super::runner_security::{AuthenticatedRunnerCommand, RunnerCommandClaims};
use super::runner_supervisor::{launch as launch_runner, RunnerLaunchConfig};
use crate::runtime::persistence_sqlite::SqlitePersistence;

/// Verified launch facts. Constructed by the caller from session config and
/// project/operator authority; nothing here is defaulted to a hopeful value.
#[derive(Debug, Clone)]
pub struct LaunchPlan {
    pub executable: String,
    pub argv: Vec<String>,
    pub cwd: String,
    pub os_user: String,
    pub mission_delivery: MissionDelivery,
    pub adapter_id: String,
    pub adapter_version: String,
    pub config_revision_id: String,
    pub model_binding: String,
    pub thinking_level: String,
    pub bootstrap_packet_ref: String,
    pub trust: LaunchTrustPolicy,
}

#[derive(Debug, Clone)]
pub struct DispatchReceipt {
    pub pid: Option<u32>,
    pub command_id: String,
    pub socket: String,
}

pub fn build_manifest(plan: &LaunchPlan) -> LaunchManifest {
    // Spec133 §13.3: the required noninteractive trust flag must reach the
    // harness. Enforce it here so a caller cannot forget it and produce a
    // manifest that hangs on a trust prompt.
    let mut argv = plan.argv.clone();
    if let Some(flag) = &plan.trust.required_noninteractive_flag {
        if !argv.iter().any(|argument| argument == flag) {
            argv.push(flag.clone());
        }
    }
    LaunchManifest {
        schema: LaunchManifest::SCHEMA.to_string(),
        executable: plan.executable.clone(),
        argv,
        cwd: plan.cwd.clone(),
        safe_env: BTreeMap::new(),
        secret_env_refs: Vec::new(),
        mission_delivery: plan.mission_delivery.clone(),
        stdin_mode: StdioMode::Null,
        stdout_mode: StdioMode::Pipe,
        stderr_mode: StdioMode::Pipe,
        process_backend: if cfg!(windows) {
            ProcessBackend::WindowsJobObject
        } else {
            ProcessBackend::UnixProcessGroup
        },
        os_user: plan.os_user.clone(),
        resource_limits: LaunchResourceLimits {
            max_runtime_seconds: None,
            max_memory_bytes: None,
            max_processes: None,
            max_open_files: None,
        },
        resource_mode: ResourceModeResolution {
            requested: LaunchResourceMode::Normal,
            effective: LaunchResourceMode::Normal,
            requirement: ResourceModeRequirement::Advisory,
            resolved: true,
            degraded_reason: None,
        },
        trust_policy: plan.trust.clone(),
        adapter_config: BTreeMap::new(),
        adapter_id: plan.adapter_id.clone(),
        adapter_version: plan.adapter_version.clone(),
        config_revision_id: plan.config_revision_id.clone(),
        model_binding: plan.model_binding.clone(),
        thinking_level: plan.thinking_level.clone(),
        bootstrap_packet_ref: plan.bootstrap_packet_ref.clone(),
    }
}

/// Provision, launch and deliver one signed `Launch` operation.
pub async fn dispatch_launch(
    persistence: &SqlitePersistence,
    state_root: &Path,
    session_id: SilentSessionId,
    run_id: SilentSessionRunId,
    plan: &LaunchPlan,
    owner_uid: u32,
    delivery_timeout: Duration,
) -> anyhow::Result<DispatchReceipt> {
    let principal_id = format!("session-{session_id}");
    let socket_scope = format!("run-{run_id}");
    let paths = RunnerPaths::resolve(state_root, &principal_id, &socket_scope)?;
    let key = paths.ensure_material(owner_uid)?;

    let manifest = build_manifest(plan);
    manifest.validate().context("launch manifest is not valid")?;
    let operation = RunnerOperation::Launch {
        spec: Box::new(RunnerLaunchSpec {
            manifest: manifest.clone(),
        }),
    };
    let payload = serde_json::to_vec(&operation).context("serialize launch operation")?;
    let now = Utc::now();
    let claims = RunnerCommandClaims {
        session_id,
        run_id,
        runner: RunnerPaths::identity(&plan.os_user, &principal_id, &socket_scope),
        action_digest: operation.action_digest()?,
        nonce: uuid::Uuid::now_v7().to_string(),
        issued_at: now,
        expires_at: now + ChronoDuration::seconds(60),
    };
    let command = AuthenticatedRunnerCommand::issue(claims, &payload, &key)
        .map_err(|error| anyhow::anyhow!("sign runner command: {error}"))?;
    let request = RunnerWireRequest {
        schema: RUNNER_PROTOCOL_SCHEMA.to_string(),
        command,
        operation,
    };
    let command_id = request.command.command_id.to_string();

    // A live socket means a runner is already serving this run; never spawn a
    // second one (single-writer, §13.1).
    let launched_pid = if paths.socket.exists() {
        None
    } else {
        let process = launch_runner(&RunnerLaunchConfig::new(
            paths.clone(),
            RunnerPaths::identity(&plan.os_user, &principal_id, &socket_scope),
            owner_uid,
        ))?;
        Some(process.pid)
    };

    if let Err(error) = send_runner_request(
        persistence,
        &paths.socket,
        owner_uid,
        &request,
        delivery_timeout,
    )
    .await
    {
        bail!("runner rejected launch delivery: {error}");
    }

    Ok(DispatchReceipt {
        pid: launched_pid,
        command_id,
        socket: paths.socket.display().to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn plan() -> LaunchPlan {
        LaunchPlan {
            executable: "/bin/true".into(),
            argv: vec![],
            cwd: std::env::temp_dir().display().to_string(),
            os_user: "tester".into(),
            mission_delivery: MissionDelivery::Rpc {
                artifact_ref: "artifacts/mission.md".into(),
                sha256: "0".repeat(64),
            },
            adapter_id: "pi_rpc".into(),
            adapter_version: "pi_rpc.v1".into(),
            config_revision_id: "cfgrev-1".into(),
            model_binding: "openai/gpt-6.1-sol".into(),
            thinking_level: "default".into(),
            bootstrap_packet_ref: "bootstrap/1".into(),
            trust: LaunchTrustPolicy {
                project_verified: true,
                workspace_verified: true,
                operator_approved: true,
                context_authority_allowed: true,
                trust_preflight_passed: true,
                required_noninteractive_flag: Some("-a".into()),
            },
        }
    }

    #[test]
    fn manifest_is_valid_and_preserves_verified_facts() {
        let manifest = build_manifest(&plan());
        manifest.validate().expect("manifest must validate");
        assert_eq!(manifest.adapter_id, "pi_rpc");
        assert_eq!(manifest.executable, "/bin/true");
        #[cfg(unix)]
        assert_eq!(manifest.process_backend, ProcessBackend::UnixProcessGroup);
        #[cfg(windows)]
        assert_eq!(manifest.process_backend, ProcessBackend::WindowsJobObject);
    }

    #[test]
    fn required_noninteractive_flag_is_not_dropped() {
        let manifest = build_manifest(&plan());
        assert_eq!(
            manifest.trust_policy.required_noninteractive_flag.as_deref(),
            Some("-a")
        );
        assert!(manifest.argv.iter().any(|arg| arg == "-a"), "flag must reach argv");
    }

    #[test]
    fn paths_are_scoped_per_run() {
        let _ = PathBuf::from("/tmp");
        let a = RunnerPaths::resolve(Path::new("/tmp/x"), "session-a", "run-1").unwrap();
        let b = RunnerPaths::resolve(Path::new("/tmp/x"), "session-a", "run-2").unwrap();
        assert_ne!(a.socket, b.socket);
    }
}
