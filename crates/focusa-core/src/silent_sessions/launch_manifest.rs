//! Typed, reproducible process launch contract for Spec133 Silent Sessions.

use std::collections::BTreeMap;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SecretEnvironmentRef {
    pub env_name: String,
    pub secret_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum MissionDelivery {
    Rpc {
        artifact_ref: String,
        sha256: String,
    },
    Stdin {
        artifact_path: String,
        sha256: String,
    },
    SecureArtifact {
        artifact_path: String,
        sha256: String,
    },
    TypedArgument {
        argv_index: usize,
        sha256: String,
        max_bytes: usize,
    },
}

impl MissionDelivery {
    pub fn artifact_hash(&self) -> &str {
        match self {
            Self::Rpc { sha256, .. }
            | Self::Stdin { sha256, .. }
            | Self::SecureArtifact { sha256, .. }
            | Self::TypedArgument { sha256, .. } => sha256,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StdioMode {
    Null,
    Inherit,
    Pipe,
    MissionArtifact,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProcessBackend {
    UnixProcessGroup,
    WindowsJobObject,
    EmbeddedSameUser,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LaunchResourceMode {
    Auto,
    Normal,
    Constrained,
    Lowmem,
    Emergency,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResourceModeRequirement {
    Required,
    Advisory,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResourceModeResolution {
    pub requested: LaunchResourceMode,
    pub effective: LaunchResourceMode,
    pub requirement: ResourceModeRequirement,
    pub resolved: bool,
    pub degraded_reason: Option<String>,
}

pub trait TypedResourceModeController {
    fn activate(&self, requested: LaunchResourceMode) -> anyhow::Result<LaunchResourceMode>;
}

pub fn resolve_resource_mode(
    controller: &dyn TypedResourceModeController,
    requested: LaunchResourceMode,
    requirement: ResourceModeRequirement,
    current: LaunchResourceMode,
) -> anyhow::Result<ResourceModeResolution> {
    match controller.activate(requested) {
        Ok(effective) if effective == requested => Ok(ResourceModeResolution {
            requested,
            effective,
            requirement,
            resolved: true,
            degraded_reason: None,
        }),
        Ok(effective) if requirement == ResourceModeRequirement::Advisory => {
            Ok(ResourceModeResolution {
                requested,
                effective,
                requirement,
                resolved: false,
                degraded_reason: Some(
                    "typed ResourceMode controller returned a different mode".into(),
                ),
            })
        }
        Ok(_) => anyhow::bail!("required typed ResourceMode activation returned a different mode"),
        Err(error) if requirement == ResourceModeRequirement::Advisory => {
            Ok(ResourceModeResolution {
                requested,
                effective: current,
                requirement,
                resolved: false,
                degraded_reason: Some(error.to_string()),
            })
        }
        Err(error) => Err(error).context("required typed ResourceMode activation failed"),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LaunchResourceLimits {
    pub max_runtime_seconds: Option<u64>,
    pub max_memory_bytes: Option<u64>,
    pub max_processes: Option<u32>,
    pub max_open_files: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LaunchTrustPolicy {
    pub project_verified: bool,
    pub workspace_verified: bool,
    pub operator_approved: bool,
    pub context_authority_allowed: bool,
    pub trust_preflight_passed: bool,
    pub required_noninteractive_flag: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LaunchManifest {
    pub schema: String,
    pub executable: String,
    #[serde(default)]
    pub argv: Vec<String>,
    pub cwd: String,
    #[serde(default)]
    pub safe_env: BTreeMap<String, String>,
    #[serde(default)]
    pub secret_env_refs: Vec<SecretEnvironmentRef>,
    pub mission_delivery: MissionDelivery,
    pub stdin_mode: StdioMode,
    pub stdout_mode: StdioMode,
    pub stderr_mode: StdioMode,
    pub process_backend: ProcessBackend,
    pub os_user: String,
    pub resource_limits: LaunchResourceLimits,
    pub resource_mode: ResourceModeResolution,
    pub trust_policy: LaunchTrustPolicy,
    #[serde(default)]
    pub adapter_config: BTreeMap<String, Value>,
    pub adapter_id: String,
    pub adapter_version: String,
    pub config_revision_id: String,
    pub model_binding: String,
    pub thinking_level: String,
    pub bootstrap_packet_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RedactedLaunchManifest {
    pub manifest: LaunchManifest,
    pub manifest_digest: String,
    pub secret_reference_count: usize,
}

impl LaunchManifest {
    pub const SCHEMA: &'static str = "focusa.launch_manifest.v1";

    pub fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.schema == Self::SCHEMA,
            "unsupported launch manifest schema"
        );
        anyhow::ensure!(
            std::path::Path::new(&self.executable).is_absolute(),
            "launch executable must be absolute"
        );
        anyhow::ensure!(
            std::path::Path::new(&self.cwd).is_absolute(),
            "launch cwd must be absolute"
        );
        anyhow::ensure!(
            !self.os_user.trim().is_empty(),
            "launch OS user is required"
        );
        anyhow::ensure!(
            self.argv.len() <= 4_096,
            "launch argv exceeds the bounded limit"
        );
        anyhow::ensure!(
            self.adapter_config.len() <= 256,
            "adapter config exceeds the bounded limit"
        );
        validate_sha256(self.mission_delivery.artifact_hash())?;
        match &self.mission_delivery {
            MissionDelivery::Rpc { artifact_ref, .. } => {
                anyhow::ensure!(
                    !artifact_ref.trim().is_empty(),
                    "mission RPC artifact reference is required"
                );
            }
            MissionDelivery::Stdin { artifact_path, .. }
            | MissionDelivery::SecureArtifact { artifact_path, .. } => {
                anyhow::ensure!(
                    std::path::Path::new(artifact_path).is_absolute(),
                    "mission artifact path must be absolute"
                );
            }
            MissionDelivery::TypedArgument {
                argv_index,
                sha256,
                max_bytes,
            } => {
                anyhow::ensure!(
                    *max_bytes > 0,
                    "typed mission argument bound must be positive"
                );
                let value = self.argv.get(*argv_index).ok_or_else(|| {
                    anyhow::anyhow!("typed mission argument index is out of bounds")
                })?;
                anyhow::ensure!(
                    value.len() <= *max_bytes,
                    "typed mission argument exceeds its bound"
                );
                anyhow::ensure!(
                    hex::encode(Sha256::digest(value.as_bytes())) == *sha256,
                    "typed mission argument hash mismatch"
                );
            }
        }
        for (name, value) in &self.safe_env {
            validate_environment_name(name)?;
            anyhow::ensure!(
                !sensitive_name(name),
                "sensitive environment values must use secret_env_refs"
            );
            anyhow::ensure!(
                value.len() <= 32_768,
                "safe environment value exceeds bounded length"
            );
        }
        for secret in &self.secret_env_refs {
            validate_environment_name(&secret.env_name)?;
            anyhow::ensure!(
                secret.secret_ref.starts_with("env://")
                    || secret.secret_ref.starts_with("secret://"),
                "secret environment reference uses an unsupported scheme"
            );
            anyhow::ensure!(
                secret.secret_ref.len() <= 1_024,
                "secret reference exceeds bounded length"
            );
        }
        anyhow::ensure!(
            self.trust_policy.project_verified
                && self.trust_policy.workspace_verified
                && self.trust_policy.operator_approved
                && self.trust_policy.context_authority_allowed
                && self.trust_policy.trust_preflight_passed,
            "launch trust preflight is incomplete"
        );
        if let Some(flag) = &self.trust_policy.required_noninteractive_flag {
            anyhow::ensure!(
                self.argv.iter().any(|argument| argument == flag),
                "required noninteractive trust flag is absent"
            );
        }
        if self.resource_mode.requirement == ResourceModeRequirement::Required {
            anyhow::ensure!(
                self.resource_mode.resolved,
                "required ResourceMode was not resolved before launch"
            );
            anyhow::ensure!(
                self.resource_mode.requested == self.resource_mode.effective,
                "required ResourceMode activation did not reach the requested mode"
            );
        } else if !self.resource_mode.resolved
            || self.resource_mode.requested != self.resource_mode.effective
        {
            anyhow::ensure!(
                self.resource_mode
                    .degraded_reason
                    .as_deref()
                    .is_some_and(|reason| !reason.trim().is_empty()),
                "advisory ResourceMode degradation requires an explicit reason"
            );
        }
        anyhow::ensure!(
            self.stdout_mode != StdioMode::MissionArtifact
                && self.stderr_mode != StdioMode::MissionArtifact,
            "mission artifact mode is valid only for stdin"
        );
        if self.stdin_mode == StdioMode::MissionArtifact {
            anyhow::ensure!(
                matches!(self.mission_delivery, MissionDelivery::Stdin { .. }),
                "mission stdin mode requires stdin artifact delivery"
            );
        }
        if matches!(self.mission_delivery, MissionDelivery::Stdin { .. }) {
            anyhow::ensure!(
                self.stdin_mode == StdioMode::MissionArtifact,
                "stdin mission delivery requires mission artifact stdin mode"
            );
        }
        Ok(())
    }

    pub fn digest(&self) -> anyhow::Result<String> {
        self.validate()?;
        Ok(hex::encode(Sha256::digest(serde_json::to_vec(self)?)))
    }

    pub fn redacted(&self) -> anyhow::Result<RedactedLaunchManifest> {
        self.validate()?;
        let mut manifest = self.clone();
        redact_values(&mut manifest.adapter_config);
        Ok(RedactedLaunchManifest {
            manifest,
            manifest_digest: self.digest()?,
            secret_reference_count: self.secret_env_refs.len(),
        })
    }
}

pub fn validate_environment_name(name: &str) -> anyhow::Result<()> {
    anyhow::ensure!(
        !name.is_empty() && name.len() <= 128,
        "invalid environment name length"
    );
    let mut bytes = name.bytes();
    let first = bytes
        .next()
        .ok_or_else(|| anyhow::anyhow!("environment name is empty"))?;
    anyhow::ensure!(
        first == b'_' || first.is_ascii_alphabetic(),
        "invalid environment name prefix"
    );
    anyhow::ensure!(
        bytes.all(|byte| byte == b'_' || byte.is_ascii_alphanumeric()),
        "invalid environment name character"
    );
    Ok(())
}

fn validate_sha256(value: &str) -> anyhow::Result<()> {
    anyhow::ensure!(
        value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit()),
        "artifact hash must be a SHA-256 hex digest"
    );
    Ok(())
}

fn sensitive_name(name: &str) -> bool {
    let lowercase = name.to_ascii_lowercase();
    [
        "secret",
        "token",
        "password",
        "credential",
        "private_key",
        "api_key",
    ]
    .iter()
    .any(|needle| lowercase.contains(needle))
}

fn redact_values(values: &mut BTreeMap<String, Value>) {
    for (key, value) in values {
        if sensitive_name(key) {
            *value = Value::String("[REDACTED]".into());
        } else if let Value::Object(object) = value {
            let mut nested = object.clone().into_iter().collect();
            redact_values(&mut nested);
            *value = Value::Object(nested.into_iter().collect());
        }
    }
}
