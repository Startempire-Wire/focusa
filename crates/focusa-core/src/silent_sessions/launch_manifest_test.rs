use std::collections::BTreeMap;

use serde_json::json;
use sha2::{Digest, Sha256};

use crate::silent_sessions::*;

fn manifest() -> LaunchManifest {
    let mission = "quotes: ' \"; newline:\n$(not-a-shell)";
    LaunchManifest {
        schema: LaunchManifest::SCHEMA.into(),
        executable: crate::test_support::executable_path()
            .to_string_lossy()
            .into_owned(),
        argv: vec!["-a".into(), mission.into()],
        cwd: crate::test_support::absolute_path_string("silent-launch-manifest-project"),
        safe_env: BTreeMap::from([(
            "PATH".into(),
            crate::test_support::absolute_path_string("bin"),
        )]),
        secret_env_refs: vec![SecretEnvironmentRef {
            env_name: "PROVIDER_TOKEN".into(),
            secret_ref: "secret://provider-token".into(),
        }],
        mission_delivery: MissionDelivery::TypedArgument {
            argv_index: 1,
            sha256: hex::encode(Sha256::digest(mission.as_bytes())),
            max_bytes: 4_096,
        },
        stdin_mode: StdioMode::Null,
        stdout_mode: StdioMode::Pipe,
        stderr_mode: StdioMode::Pipe,
        process_backend: if cfg!(windows) {
            ProcessBackend::WindowsJobObject
        } else {
            ProcessBackend::UnixProcessGroup
        },
        os_user: "wirebot".into(),
        resource_limits: LaunchResourceLimits {
            max_runtime_seconds: Some(3_600),
            max_memory_bytes: Some(4 * 1024 * 1024 * 1024),
            max_processes: Some(64),
            max_open_files: Some(1_024),
        },
        resource_mode: ResourceModeResolution {
            requested: LaunchResourceMode::Lowmem,
            effective: LaunchResourceMode::Lowmem,
            requirement: ResourceModeRequirement::Required,
            resolved: true,
            degraded_reason: None,
        },
        trust_policy: LaunchTrustPolicy {
            project_verified: true,
            workspace_verified: true,
            operator_approved: true,
            context_authority_allowed: true,
            trust_preflight_passed: true,
            required_noninteractive_flag: Some("-a".into()),
        },
        adapter_config: BTreeMap::from([
            ("provider_token".into(), json!("must-redact")),
            ("transport".into(), json!("rpc")),
        ]),
        adapter_id: "pi-rpc".into(),
        adapter_version: "1".into(),
        config_revision_id: "revision:1".into(),
        model_binding: "provider:model".into(),
        thinking_level: "high".into(),
        bootstrap_packet_ref: "bootstrap:1".into(),
    }
}

#[test]
fn typed_manifest_preserves_shell_symbols_without_shell_composition() {
    let manifest = manifest();
    manifest.validate().unwrap();
    let serialized = serde_json::to_string(&manifest).unwrap();
    assert!(serialized.contains("$(not-a-shell)"));
    assert_eq!(manifest.digest().unwrap().len(), 64);
    let redacted = manifest.redacted().unwrap();
    assert_eq!(
        redacted.manifest.adapter_config["provider_token"],
        json!("[REDACTED]")
    );
    assert_eq!(redacted.secret_reference_count, 1);
}

#[test]
fn required_lowmem_and_trust_fail_closed_before_spawn() {
    let mut lowmem_manifest = manifest();
    lowmem_manifest.resource_mode.resolved = false;
    assert!(
        lowmem_manifest
            .validate()
            .unwrap_err()
            .to_string()
            .contains("ResourceMode")
    );

    let mut trust_manifest = manifest();
    trust_manifest.argv.retain(|argument| argument != "-a");
    assert!(trust_manifest.validate().is_err());
}

#[test]
fn raw_sensitive_environment_is_rejected() {
    let mut manifest = manifest();
    manifest
        .safe_env
        .insert("API_TOKEN".into(), "raw-secret".into());
    assert!(
        manifest
            .validate()
            .unwrap_err()
            .to_string()
            .contains("secret_env_refs")
    );
}

struct FailingResourceModeController;

impl TypedResourceModeController for FailingResourceModeController {
    fn activate(&self, _requested: LaunchResourceMode) -> anyhow::Result<LaunchResourceMode> {
        anyhow::bail!("invalid HTTP content type")
    }
}

#[test]
fn lowmem_failure_is_typed_and_never_shell_chained() {
    assert!(
        resolve_resource_mode(
            &FailingResourceModeController,
            LaunchResourceMode::Lowmem,
            ResourceModeRequirement::Required,
            LaunchResourceMode::Normal,
        )
        .is_err()
    );
    let advisory = resolve_resource_mode(
        &FailingResourceModeController,
        LaunchResourceMode::Lowmem,
        ResourceModeRequirement::Advisory,
        LaunchResourceMode::Normal,
    )
    .unwrap();
    assert!(!advisory.resolved);
    assert_eq!(advisory.effective, LaunchResourceMode::Normal);
    assert!(advisory.degraded_reason.unwrap().contains("content type"));
}
