//! Typed, shell-free launch manifests for daemon-native Silent Sessions.
//!
//! A manifest contains only reproducible, redacted launch configuration. Mission
//! bytes and resolved secret values are deliberately separate, non-serializable
//! preparation inputs.

use crate::silent_session::{HarnessKind, ModelBinding};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::PathBuf;
use thiserror::Error;

pub const LAUNCH_MANIFEST_SCHEMA: &str = "focusa.launch_manifest.v1";
pub const MAX_MISSION_ARTIFACT_BYTES: u64 = 16 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StdinMode {
    Null,
    Mission,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputMode {
    Null,
    Piped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessBackendKind {
    PosixDirect,
    GenericPty,
    TmuxCompatibility,
    WindowsJobConpty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceMode {
    Normal,
    Constrained,
    LowMem,
    Emergency,
}

impl ResourceMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Constrained => "constrained",
            Self::LowMem => "lowmem",
            Self::Emergency => "emergency",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceModeRequirement {
    Required,
    Advisory,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceModeRequest {
    pub mode: ResourceMode,
    pub requirement: ResourceModeRequirement,
    pub reason: String,
    pub policy_ref: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceModeResolutionStatus {
    Activated,
    AlreadyActive,
    Degraded,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceModeResolution {
    pub requested_mode: ResourceMode,
    pub effective_mode: Option<ResourceMode>,
    pub status: ResourceModeResolutionStatus,
    pub evidence_ref: Option<String>,
    pub failure_class: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{failure_class}: {message}")]
pub struct ResourceModeFailure {
    pub failure_class: String,
    pub message: String,
}

/// Daemon-internal typed ResourceMode control boundary. Implementations may
/// update daemon state directly or use a typed API client; launch code never
/// composes an HTTP/curl shell fragment.
pub trait ResourceModeController {
    fn resolve(
        &mut self,
        request: &ResourceModeRequest,
    ) -> Result<ResourceModeResolution, ResourceModeFailure>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissionArtifact {
    pub artifact_ref: String,
    pub sha256: String,
    pub byte_len: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MissionDelivery {
    Rpc { method: String },
    Stdin,
    SecureArtifact,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecretEnvironmentRef {
    pub env_key: String,
    pub secret_ref: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrustMode {
    ApprovedNonInteractive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnexpectedTrustPromptPolicy {
    Block,
    WaitingInput,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustPolicy {
    pub mode: TrustMode,
    pub operator_approval_ref: String,
    pub context_authority_verdict_ref: String,
    pub project_identity_ref: String,
    pub workspace_ref: String,
    pub unexpected_prompt_policy: UnexpectedTrustPromptPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LaunchResourceLimits {
    pub max_wall_clock_seconds: Option<u64>,
    pub max_cpu_percent_basis_points: Option<u32>,
    pub max_memory_bytes: Option<u64>,
    pub max_pids: Option<u32>,
    pub max_disk_bytes: Option<u64>,
    pub max_output_bytes: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LaunchReproducibility {
    pub config_revision_ref: String,
    pub project_identity_ref: String,
    pub workspace_ref: String,
    pub bootstrap_packet_ref: String,
    pub bootstrap_packet_sha256: String,
    pub model_binding: ModelBinding,
    pub thinking_level: Option<String>,
    pub adapter_version: String,
    pub process_backend_version: String,
    pub resource_policy_ref: String,
}

/// Secret-free effective process configuration. Safe environment values are
/// retained exactly; secret values can appear only in the separate runner
/// resolution input and only their references are reproducible here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LaunchManifest {
    pub schema: String,
    pub executable: PathBuf,
    pub argv: Vec<String>,
    pub cwd: PathBuf,
    pub safe_env: BTreeMap<String, String>,
    pub secret_env_refs: Vec<SecretEnvironmentRef>,
    pub mission_artifact: MissionArtifact,
    pub mission_delivery: MissionDelivery,
    pub stdin_mode: StdinMode,
    pub stdout_mode: OutputMode,
    pub stderr_mode: OutputMode,
    pub process_backend: ProcessBackendKind,
    pub os_user: String,
    pub resource_limits: LaunchResourceLimits,
    pub resource_mode: ResourceModeRequest,
    pub trust_policy: TrustPolicy,
    pub harness_kind: HarnessKind,
    pub reproducibility: LaunchReproducibility,
}

impl LaunchManifest {
    pub fn validate(&self) -> Result<(), LaunchPreparationError> {
        if self.schema != LAUNCH_MANIFEST_SCHEMA {
            return Err(LaunchPreparationError::UnsupportedSchema);
        }
        if !self.executable.is_absolute() || !self.cwd.is_absolute() {
            return Err(LaunchPreparationError::PathMustBeAbsolute);
        }
        if self.executable.to_str().is_none() || self.cwd.to_str().is_none() {
            return Err(LaunchPreparationError::PathNotUtf8);
        }
        if is_shell_executable(&self.executable) {
            return Err(LaunchPreparationError::ShellExecutableForbidden);
        }
        if self.os_user.trim().is_empty() {
            return Err(LaunchPreparationError::ExecutionUserMissing);
        }
        if self.argv.iter().any(|argument| argument.contains('\0')) {
            return Err(LaunchPreparationError::ArgumentContainsNul);
        }
        validate_environment(&self.safe_env, &self.secret_env_refs)?;
        validate_mission(self)?;
        validate_hash(
            &self.reproducibility.bootstrap_packet_sha256,
            LaunchPreparationError::InvalidBootstrapHash,
        )?;
        validate_reproducibility(self)?;
        validate_trust(self)?;
        if self.resource_mode.reason.trim().is_empty()
            || self.resource_mode.policy_ref.trim().is_empty()
        {
            return Err(LaunchPreparationError::InvalidResourceModeRequest);
        }
        Ok(())
    }

    pub fn redacted_json(&self) -> Result<Vec<u8>, LaunchPreparationError> {
        self.validate()?;
        serde_json::to_vec(self).map_err(|_| LaunchPreparationError::SerializationFailed)
    }

    pub fn redacted_sha256(&self) -> Result<String, LaunchPreparationError> {
        Ok(sha256_hex(&self.redacted_json()?))
    }
}

/// Exact mission bytes are bounded and hash-verified, but intentionally do not
/// implement Serialize and redact their Debug representation.
#[derive(Clone, PartialEq, Eq)]
pub struct MissionPayload(Vec<u8>);

impl MissionPayload {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Debug for MissionPayload {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MissionPayload")
            .field("byte_len", &self.0.len())
            .field("sha256", &sha256_hex(&self.0))
            .finish()
    }
}

pub struct PreparedLaunchManifest {
    manifest: LaunchManifest,
    redacted_sha256: String,
    mission_payload: MissionPayload,
    resource_mode_resolution: ResourceModeResolution,
}

impl PreparedLaunchManifest {
    pub fn prepare(
        manifest: LaunchManifest,
        mission_payload: MissionPayload,
        resource_mode_controller: &mut impl ResourceModeController,
    ) -> Result<Self, LaunchPreparationError> {
        manifest.validate()?;
        validate_mission_payload(&manifest.mission_artifact, &mission_payload)?;
        let redacted_sha256 = manifest.redacted_sha256()?;
        let resource_mode_resolution =
            match resource_mode_controller.resolve(&manifest.resource_mode) {
                Ok(resolution) => {
                    if resolution.requested_mode != manifest.resource_mode.mode {
                        return Err(LaunchPreparationError::ResourceModeResponseMismatch);
                    }
                    resolution
                }
                Err(failure)
                    if manifest.resource_mode.requirement == ResourceModeRequirement::Advisory =>
                {
                    ResourceModeResolution {
                        requested_mode: manifest.resource_mode.mode,
                        effective_mode: None,
                        status: ResourceModeResolutionStatus::Degraded,
                        evidence_ref: None,
                        failure_class: Some(failure.failure_class),
                    }
                }
                Err(failure) => {
                    return Err(LaunchPreparationError::RequiredResourceModeFailed(failure));
                }
            };
        Ok(Self {
            manifest,
            redacted_sha256,
            mission_payload,
            resource_mode_resolution,
        })
    }

    pub fn manifest(&self) -> &LaunchManifest {
        &self.manifest
    }

    pub fn redacted_sha256(&self) -> &str {
        &self.redacted_sha256
    }

    pub fn mission_payload(&self) -> &MissionPayload {
        &self.mission_payload
    }

    pub fn resource_mode_resolution(&self) -> &ResourceModeResolution {
        &self.resource_mode_resolution
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum LaunchPreparationError {
    #[error("launch manifest schema is unsupported")]
    UnsupportedSchema,
    #[error("launch executable and cwd must be absolute")]
    PathMustBeAbsolute,
    #[error("launch executable and cwd must be valid UTF-8 for reproducibility")]
    PathNotUtf8,
    #[error("shell executables are forbidden; provide the harness executable and argv")]
    ShellExecutableForbidden,
    #[error("OS execution user is required")]
    ExecutionUserMissing,
    #[error("launch argv cannot contain NUL")]
    ArgumentContainsNul,
    #[error("environment key is invalid: {0}")]
    InvalidEnvironmentKey(String),
    #[error("safe environment key is secret-shaped and must use secret_env_refs: {0}")]
    UnsafeEnvironmentKey(String),
    #[error("environment values cannot contain NUL")]
    EnvironmentValueContainsNul,
    #[error("safe and secret environment entries overlap: {0}")]
    EnvironmentKeyOverlap(String),
    #[error("secret environment references must be unique and ordered by env_key")]
    SecretEnvironmentRefsNotCanonical,
    #[error("secret environment reference is invalid")]
    InvalidSecretEnvironmentRef,
    #[error("mission artifact reference is required")]
    MissionArtifactRefMissing,
    #[error("mission artifact hash is invalid")]
    InvalidMissionHash,
    #[error("mission artifact length is zero or exceeds the launch bound")]
    MissionArtifactLengthInvalid,
    #[error("mission stdin mode does not match mission delivery")]
    MissionStdinModeMismatch,
    #[error("mission payload length does not match its artifact")]
    MissionPayloadLengthMismatch,
    #[error("mission payload hash does not match its artifact")]
    MissionPayloadHashMismatch,
    #[error("RPC mission method is required")]
    RpcMethodMissing,
    #[error("bootstrap packet hash is invalid")]
    InvalidBootstrapHash,
    #[error("launch reproducibility metadata is incomplete")]
    ReproducibilityIncomplete,
    #[error("trust preflight metadata is incomplete")]
    TrustPreflightIncomplete,
    #[error("Pi autonomous launch requires the exact -a trust flag")]
    PiTrustFlagMissing,
    #[error("resource mode request is incomplete")]
    InvalidResourceModeRequest,
    #[error("typed ResourceMode response does not match the request")]
    ResourceModeResponseMismatch,
    #[error("required ResourceMode activation failed: {0}")]
    RequiredResourceModeFailed(ResourceModeFailure),
    #[error("launch manifest serialization failed")]
    SerializationFailed,
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn validate_hash(value: &str, error: LaunchPreparationError) -> Result<(), LaunchPreparationError> {
    if value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        Ok(())
    } else {
        Err(error)
    }
}

fn validate_environment(
    safe_env: &BTreeMap<String, String>,
    secret_env_refs: &[SecretEnvironmentRef],
) -> Result<(), LaunchPreparationError> {
    for (key, value) in safe_env {
        validate_environment_key(key)?;
        if secret_shaped_environment_key(key) {
            return Err(LaunchPreparationError::UnsafeEnvironmentKey(key.clone()));
        }
        if value.contains('\0') {
            return Err(LaunchPreparationError::EnvironmentValueContainsNul);
        }
    }

    let mut previous = None;
    let mut keys = BTreeSet::new();
    for secret in secret_env_refs {
        validate_environment_key(&secret.env_key)?;
        if secret.secret_ref.trim().is_empty()
            || secret.secret_ref.contains('\0')
            || secret.secret_ref.chars().any(char::is_control)
        {
            return Err(LaunchPreparationError::InvalidSecretEnvironmentRef);
        }
        if previous.is_some_and(|value: &str| value >= secret.env_key.as_str())
            || !keys.insert(secret.env_key.as_str())
        {
            return Err(LaunchPreparationError::SecretEnvironmentRefsNotCanonical);
        }
        if safe_env.contains_key(&secret.env_key) {
            return Err(LaunchPreparationError::EnvironmentKeyOverlap(
                secret.env_key.clone(),
            ));
        }
        previous = Some(secret.env_key.as_str());
    }
    Ok(())
}

fn validate_environment_key(key: &str) -> Result<(), LaunchPreparationError> {
    let mut characters = key.chars();
    let valid_first = characters
        .next()
        .is_some_and(|character| character == '_' || character.is_ascii_alphabetic());
    if !valid_first
        || !characters.all(|character| character == '_' || character.is_ascii_alphanumeric())
    {
        return Err(LaunchPreparationError::InvalidEnvironmentKey(
            key.to_owned(),
        ));
    }
    Ok(())
}

fn secret_shaped_environment_key(key: &str) -> bool {
    let upper = key.to_ascii_uppercase();
    upper.contains("PASSWORD")
        || upper.contains("TOKEN")
        || upper.contains("SECRET")
        || upper.contains("CREDENTIAL")
        || upper.ends_with("API_KEY")
        || upper.ends_with("PRIVATE_KEY")
}

fn validate_mission(manifest: &LaunchManifest) -> Result<(), LaunchPreparationError> {
    if manifest.mission_artifact.artifact_ref.trim().is_empty() {
        return Err(LaunchPreparationError::MissionArtifactRefMissing);
    }
    validate_hash(
        &manifest.mission_artifact.sha256,
        LaunchPreparationError::InvalidMissionHash,
    )?;
    if manifest.mission_artifact.byte_len == 0
        || manifest.mission_artifact.byte_len > MAX_MISSION_ARTIFACT_BYTES
    {
        return Err(LaunchPreparationError::MissionArtifactLengthInvalid);
    }
    match &manifest.mission_delivery {
        MissionDelivery::Stdin if manifest.stdin_mode != StdinMode::Mission => {
            Err(LaunchPreparationError::MissionStdinModeMismatch)
        }
        MissionDelivery::Rpc { method } if method.trim().is_empty() => {
            Err(LaunchPreparationError::RpcMethodMissing)
        }
        MissionDelivery::Rpc { .. } | MissionDelivery::SecureArtifact
            if manifest.stdin_mode != StdinMode::Null =>
        {
            Err(LaunchPreparationError::MissionStdinModeMismatch)
        }
        _ => Ok(()),
    }
}

fn validate_mission_payload(
    artifact: &MissionArtifact,
    payload: &MissionPayload,
) -> Result<(), LaunchPreparationError> {
    if u64::try_from(payload.as_bytes().len()).ok() != Some(artifact.byte_len) {
        return Err(LaunchPreparationError::MissionPayloadLengthMismatch);
    }
    if sha256_hex(payload.as_bytes()) != artifact.sha256 {
        return Err(LaunchPreparationError::MissionPayloadHashMismatch);
    }
    Ok(())
}

fn validate_reproducibility(manifest: &LaunchManifest) -> Result<(), LaunchPreparationError> {
    let metadata = &manifest.reproducibility;
    let required = [
        metadata.config_revision_ref.as_str(),
        metadata.project_identity_ref.as_str(),
        metadata.workspace_ref.as_str(),
        metadata.bootstrap_packet_ref.as_str(),
        metadata.model_binding.provider.as_str(),
        metadata.model_binding.model.as_str(),
        metadata.adapter_version.as_str(),
        metadata.process_backend_version.as_str(),
        metadata.resource_policy_ref.as_str(),
    ];
    if required.iter().any(|value| value.trim().is_empty())
        || metadata.project_identity_ref != manifest.trust_policy.project_identity_ref
        || metadata.workspace_ref != manifest.trust_policy.workspace_ref
        || metadata.resource_policy_ref != manifest.resource_mode.policy_ref
    {
        return Err(LaunchPreparationError::ReproducibilityIncomplete);
    }
    Ok(())
}

fn validate_trust(manifest: &LaunchManifest) -> Result<(), LaunchPreparationError> {
    let trust = &manifest.trust_policy;
    if trust.operator_approval_ref.trim().is_empty()
        || trust.context_authority_verdict_ref.trim().is_empty()
        || trust.project_identity_ref.trim().is_empty()
        || trust.workspace_ref.trim().is_empty()
    {
        return Err(LaunchPreparationError::TrustPreflightIncomplete);
    }
    if manifest.harness_kind == HarnessKind::Pi
        && manifest
            .argv
            .iter()
            .filter(|argument| argument.as_str() == "-a")
            .count()
            != 1
    {
        return Err(LaunchPreparationError::PiTrustFlagMissing);
    }
    Ok(())
}

fn is_shell_executable(executable: &std::path::Path) -> bool {
    executable
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| {
            matches!(
                name.to_ascii_lowercase().as_str(),
                "sh" | "bash" | "dash" | "zsh" | "fish" | "csh" | "tcsh" | "ksh"
            )
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeResourceMode {
        result: Result<ResourceModeResolution, ResourceModeFailure>,
        calls: Vec<ResourceModeRequest>,
    }

    impl ResourceModeController for FakeResourceMode {
        fn resolve(
            &mut self,
            request: &ResourceModeRequest,
        ) -> Result<ResourceModeResolution, ResourceModeFailure> {
            self.calls.push(request.clone());
            self.result.clone()
        }
    }

    fn manifest(mission: &[u8]) -> LaunchManifest {
        let mission_sha256 = sha256_hex(mission);
        LaunchManifest {
            schema: LAUNCH_MANIFEST_SCHEMA.into(),
            executable: crate::test_support::executable_path(),
            argv: vec![
                "-a".into(),
                "--model".into(),
                "gpt-test".into(),
                "literal 'quotes' \"double\" $HOME; $(touch nope) | & > <".into(),
            ],
            cwd: crate::test_support::absolute_path("silent-launch-worktree"),
            safe_env: BTreeMap::from([
                ("LANG".into(), "en_US.UTF-8".into()),
                ("SAFE_MULTILINE".into(), "line one\nline two;$HOME".into()),
            ]),
            secret_env_refs: vec![SecretEnvironmentRef {
                env_key: "OPENAI_API_KEY".into(),
                secret_ref: "secret:provider/openai".into(),
            }],
            mission_artifact: MissionArtifact {
                artifact_ref: "artifact:mission/019f".into(),
                sha256: mission_sha256,
                byte_len: mission.len() as u64,
            },
            mission_delivery: MissionDelivery::Stdin,
            stdin_mode: StdinMode::Mission,
            stdout_mode: OutputMode::Null,
            stderr_mode: OutputMode::Null,
            process_backend: ProcessBackendKind::PosixDirect,
            os_user: "alice".into(),
            resource_limits: LaunchResourceLimits {
                max_wall_clock_seconds: Some(3600),
                max_cpu_percent_basis_points: Some(10_000),
                max_memory_bytes: Some(1_000_000_000),
                max_pids: Some(64),
                max_disk_bytes: Some(2_000_000_000),
                max_output_bytes: Some(50_000_000),
            },
            resource_mode: ResourceModeRequest {
                mode: ResourceMode::LowMem,
                requirement: ResourceModeRequirement::Required,
                reason: "session admission policy".into(),
                policy_ref: "resource-policy:lowmem".into(),
            },
            trust_policy: TrustPolicy {
                mode: TrustMode::ApprovedNonInteractive,
                operator_approval_ref: "approval:launch".into(),
                context_authority_verdict_ref: "verdict:launch".into(),
                project_identity_ref: "project:focusa".into(),
                workspace_ref: "workspace:isolated".into(),
                unexpected_prompt_policy: UnexpectedTrustPromptPolicy::Block,
            },
            harness_kind: HarnessKind::Pi,
            reproducibility: LaunchReproducibility {
                config_revision_ref: "config:019f".into(),
                project_identity_ref: "project:focusa".into(),
                workspace_ref: "workspace:isolated".into(),
                bootstrap_packet_ref: "bootstrap:019f".into(),
                bootstrap_packet_sha256: "b".repeat(64),
                model_binding: ModelBinding {
                    provider: "openai".into(),
                    model: "gpt-test".into(),
                    thinking: Some("high".into()),
                },
                thinking_level: Some("high".into()),
                adapter_version: "pi-rpc.v1".into(),
                process_backend_version: "posix-direct.v1".into(),
                resource_policy_ref: "resource-policy:lowmem".into(),
            },
        }
    }

    fn successful_lowmem() -> FakeResourceMode {
        FakeResourceMode {
            result: Ok(ResourceModeResolution {
                requested_mode: ResourceMode::LowMem,
                effective_mode: Some(ResourceMode::LowMem),
                status: ResourceModeResolutionStatus::Activated,
                evidence_ref: Some("resource-mode:transition/019f".into()),
                failure_class: None,
            }),
            calls: vec![],
        }
    }

    #[test]
    fn exact_redacted_reproduction_preserves_argv_and_safe_env_not_mission_or_secrets() {
        let mission =
            b"Fix 'quotes' and \"double quotes\".\n```sh\necho $HOME; touch /tmp/nope | cat &\n```";
        let manifest = manifest(mission);
        let mut resource_mode = successful_lowmem();
        let prepared = PreparedLaunchManifest::prepare(
            manifest.clone(),
            MissionPayload::new(mission.to_vec()),
            &mut resource_mode,
        )
        .expect("typed launch should prepare");

        assert_eq!(resource_mode.calls, vec![manifest.resource_mode.clone()]);
        assert_eq!(prepared.manifest(), &manifest);
        assert_eq!(
            prepared.redacted_sha256(),
            manifest.redacted_sha256().unwrap()
        );
        let redacted = String::from_utf8(manifest.redacted_json().unwrap()).unwrap();
        assert!(redacted.contains("literal 'quotes'"));
        assert!(redacted.contains("SAFE_MULTILINE"));
        assert!(redacted.contains("secret:provider/openai"));
        assert!(!redacted.contains("Fix 'quotes'"));
        assert!(
            !format!("{:?}", MissionPayload::new(mission.to_vec())).contains("touch /tmp/nope")
        );
    }

    #[test]
    fn pi_trust_preflight_requires_exact_a_flag_and_shell_executables_are_rejected() {
        let mission = b"mission";
        let mut missing_flag = manifest(mission);
        missing_flag.argv.retain(|argument| argument != "-a");
        assert_eq!(
            missing_flag.validate(),
            Err(LaunchPreparationError::PiTrustFlagMissing)
        );

        let mut shell = manifest(mission);
        shell.executable = PathBuf::from("/bin/sh");
        shell.argv = vec!["-c".into(), "pi -a '$MISSION'".into()];
        assert_eq!(
            shell.validate(),
            Err(LaunchPreparationError::ShellExecutableForbidden)
        );
    }

    #[test]
    fn lowmem_required_failure_blocks_and_advisory_failure_is_explicitly_degraded() {
        let mission = b"mission";
        let failure = ResourceModeFailure {
            failure_class: "resource_mode_activation_failed".into(),
            message: "typed daemon ResourceMode update rejected".into(),
        };
        let mut required_controller = FakeResourceMode {
            result: Err(failure.clone()),
            calls: vec![],
        };
        let required = PreparedLaunchManifest::prepare(
            manifest(mission),
            MissionPayload::new(mission.to_vec()),
            &mut required_controller,
        );
        assert_eq!(
            required.err(),
            Some(LaunchPreparationError::RequiredResourceModeFailed(
                failure.clone()
            ))
        );

        let mut advisory_manifest = manifest(mission);
        advisory_manifest.resource_mode.requirement = ResourceModeRequirement::Advisory;
        let mut advisory_controller = FakeResourceMode {
            result: Err(failure),
            calls: vec![],
        };
        let advisory = PreparedLaunchManifest::prepare(
            advisory_manifest,
            MissionPayload::new(mission.to_vec()),
            &mut advisory_controller,
        )
        .expect("advisory LowMem failure should not compose with or terminate a process");
        assert_eq!(
            advisory.resource_mode_resolution().status,
            ResourceModeResolutionStatus::Degraded
        );
        assert_eq!(
            advisory.resource_mode_resolution().failure_class.as_deref(),
            Some("resource_mode_activation_failed")
        );
    }

    #[test]
    fn mission_hash_and_secret_environment_boundaries_fail_closed() {
        let mission = b"mission";
        let mut wrong_hash = manifest(mission);
        wrong_hash.mission_artifact.sha256 = "0".repeat(64);
        let mut resource_mode = successful_lowmem();
        assert_eq!(
            PreparedLaunchManifest::prepare(
                wrong_hash,
                MissionPayload::new(mission.to_vec()),
                &mut resource_mode,
            )
            .err(),
            Some(LaunchPreparationError::MissionPayloadHashMismatch)
        );

        let mut inline_secret = manifest(mission);
        inline_secret
            .safe_env
            .insert("OPENAI_API_TOKEN".into(), "raw-secret".into());
        assert_eq!(
            inline_secret.validate(),
            Err(LaunchPreparationError::UnsafeEnvironmentKey(
                "OPENAI_API_TOKEN".into()
            ))
        );
    }
}
