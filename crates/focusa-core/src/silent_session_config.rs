use crate::silent_session::{
    ConfigValidationResult, SILENT_SESSION_CONFIG_REVISION_SCHEMA, SilentSessionConfig,
    SilentSessionConfigRevision, SilentSessionConfigRevisionId, SilentSessionId,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigSource {
    BuiltInDefault,
    GlobalDefault,
    ProjectDefault,
    RoleProfile,
    InvocationPreset,
    ExplicitOverride,
    OperatorEdit,
    PolicyLock,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigLayer {
    pub source: ConfigSource,
    pub patch: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SilentSessionProfile {
    pub profile_id: String,
    pub persistent_defaults: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SilentSessionPreset {
    pub preset_id: String,
    pub invocation_patch: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigPolicyLock {
    pub json_pointer: String,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigMutationClass {
    HotMutable,
    RestartRequired,
    Immutable,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EffectiveSilentSessionConfig {
    pub requested_config: SilentSessionConfig,
    pub effective_config: SilentSessionConfig,
    pub field_provenance: BTreeMap<String, String>,
    pub policy_locks: Vec<ConfigPolicyLock>,
    pub mutation_classes: BTreeMap<String, ConfigMutationClass>,
    pub warnings: Vec<String>,
    pub validation: ConfigValidationResult,
    pub redacted_config_hash: String,
}

pub struct SilentSessionConfigManager {
    session_id: SilentSessionId,
    current: SilentSessionConfigRevision,
    history: BTreeMap<SilentSessionConfigRevisionId, SilentSessionConfigRevision>,
    policy_locks: Vec<ConfigPolicyLock>,
}

impl SilentSessionConfigManager {
    pub fn new(
        session_id: SilentSessionId,
        config: SilentSessionConfig,
        policy_locks: Vec<ConfigPolicyLock>,
    ) -> anyhow::Result<Self> {
        Self::new_with_revision_id(
            session_id,
            SilentSessionConfigRevisionId::new(),
            config,
            policy_locks,
        )
    }

    pub fn new_with_revision_id(
        session_id: SilentSessionId,
        revision_id: SilentSessionConfigRevisionId,
        config: SilentSessionConfig,
        policy_locks: Vec<ConfigPolicyLock>,
    ) -> anyhow::Result<Self> {
        anyhow::ensure!(
            revision_id.is_uuid_v7(),
            "config revision id must be UUIDv7"
        );
        let validation = validate_config(&config);
        anyhow::ensure!(
            validation.valid,
            "initial silent-session config is invalid: {:?}",
            validation.errors
        );
        let current = SilentSessionConfigRevision {
            schema: SILENT_SESSION_CONFIG_REVISION_SCHEMA.into(),
            revision_id,
            session_id,
            parent_revision_id: None,
            requested_changes: Value::Object(Map::new()),
            effective_diff: Value::Object(Map::new()),
            field_provenance: BTreeMap::new(),
            policy_lock_results: BTreeMap::new(),
            operator_approval_ref: None,
            validation_result: validation,
            applied_at: Some(Utc::now()),
            rollback_target: None,
            config,
        };
        let history = BTreeMap::from([(current.revision_id, current.clone())]);
        Ok(Self {
            session_id,
            current,
            history,
            policy_locks,
        })
    }

    pub fn restore(
        session_id: SilentSessionId,
        current_revision_id: SilentSessionConfigRevisionId,
        revisions: Vec<SilentSessionConfigRevision>,
        policy_locks: Vec<ConfigPolicyLock>,
    ) -> anyhow::Result<Self> {
        let history: BTreeMap<_, _> = revisions
            .into_iter()
            .map(|revision| (revision.revision_id, revision))
            .collect();
        let current = history
            .get(&current_revision_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("current config revision is not durable"))?;
        anyhow::ensure!(
            !history.is_empty()
                && history
                    .values()
                    .all(|revision| revision.session_id == session_id),
            "config revision history scope mismatch"
        );
        Ok(Self {
            session_id,
            current,
            history,
            policy_locks,
        })
    }

    pub fn current(&self) -> &SilentSessionConfigRevision {
        &self.current
    }

    pub fn preview(
        &self,
        mut layers: Vec<ConfigLayer>,
    ) -> anyhow::Result<EffectiveSilentSessionConfig> {
        layers.sort_by_key(|layer| layer.source);
        let requested_config = self.current.config.clone();
        let mut value = serde_json::to_value(&requested_config)?;
        let mut baseline_leaves = vec![];
        collect_leaf_paths("", &value, &mut baseline_leaves);
        let mut provenance: BTreeMap<_, _> = baseline_leaves
            .into_iter()
            .map(|pointer| {
                (
                    pointer,
                    format!("CurrentRevision:{}", self.current.revision_id),
                )
            })
            .collect();
        for layer in layers {
            reject_inline_secrets(&layer.patch)?;
            let mut leaves = vec![];
            collect_leaf_paths("", &layer.patch, &mut leaves);
            for pointer in &leaves {
                if let Some(lock) = self.policy_locks.iter().find(|lock| {
                    pointer == &lock.json_pointer
                        || pointer.starts_with(&(lock.json_pointer.clone() + "/"))
                }) {
                    anyhow::bail!("config field {pointer} is locked by {}", lock.source);
                }
                provenance.insert(pointer.clone(), format!("{:?}", layer.source));
            }
            merge_json(&mut value, layer.patch);
        }
        let effective_config: SilentSessionConfig = serde_json::from_value(value)?;
        let validation = validate_config(&effective_config);
        let mutation_classes = classify_changes(&self.current.config, &effective_config)?;
        let redacted_config_hash = redacted_config_hash(&effective_config)?;
        Ok(EffectiveSilentSessionConfig {
            requested_config,
            effective_config,
            field_provenance: provenance,
            policy_locks: self.policy_locks.clone(),
            mutation_classes,
            warnings: validation.warnings.clone(),
            validation,
            redacted_config_hash,
        })
    }

    pub fn apply(
        &mut self,
        expected_revision: SilentSessionConfigRevisionId,
        layers: Vec<ConfigLayer>,
        operator_approval_ref: Option<String>,
    ) -> anyhow::Result<SilentSessionConfigRevision> {
        anyhow::ensure!(
            self.current.revision_id == expected_revision,
            "stale config revision"
        );
        let preview = self.preview(layers.clone())?;
        anyhow::ensure!(preview.validation.valid, "config validation rejected apply");
        anyhow::ensure!(
            !preview
                .mutation_classes
                .values()
                .any(|class| *class == ConfigMutationClass::Immutable),
            "immutable session config cannot be revised in place"
        );
        if preview
            .mutation_classes
            .values()
            .any(|class| *class == ConfigMutationClass::RestartRequired)
        {
            anyhow::ensure!(
                operator_approval_ref.is_some(),
                "restart-required config change needs approval"
            );
        }
        let requested_changes = serde_json::to_value(&layers)?;
        let effective_diff = serde_json::to_value(&preview.mutation_classes)?;
        let revision = SilentSessionConfigRevision {
            schema: SILENT_SESSION_CONFIG_REVISION_SCHEMA.into(),
            revision_id: SilentSessionConfigRevisionId::new(),
            session_id: self.session_id,
            parent_revision_id: Some(self.current.revision_id),
            config: preview.effective_config,
            requested_changes,
            effective_diff,
            field_provenance: preview.field_provenance,
            policy_lock_results: self
                .policy_locks
                .iter()
                .map(|lock| (lock.json_pointer.clone(), true))
                .collect(),
            operator_approval_ref,
            validation_result: preview.validation,
            applied_at: Some(Utc::now()),
            rollback_target: None,
        };
        self.history.insert(revision.revision_id, revision.clone());
        self.current = revision.clone();
        Ok(revision)
    }

    pub fn verify_current_hash(&self, expected_hash: &str) -> anyhow::Result<()> {
        anyhow::ensure!(
            redacted_config_hash(&self.current.config)? == expected_hash,
            "effective config verification mismatch"
        );
        Ok(())
    }

    pub fn rollback(
        &mut self,
        expected_revision: SilentSessionConfigRevisionId,
        target: SilentSessionConfigRevisionId,
        operator_approval_ref: String,
    ) -> anyhow::Result<SilentSessionConfigRevision> {
        anyhow::ensure!(
            self.current.revision_id == expected_revision,
            "stale config revision"
        );
        let target_revision = self
            .history
            .get(&target)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("unknown rollback target"))?;
        let mut revision = target_revision;
        revision.revision_id = SilentSessionConfigRevisionId::new();
        revision.parent_revision_id = Some(self.current.revision_id);
        revision.rollback_target = Some(target);
        revision.operator_approval_ref = Some(operator_approval_ref);
        revision.applied_at = Some(Utc::now());
        self.history.insert(revision.revision_id, revision.clone());
        self.current = revision.clone();
        Ok(revision)
    }
}

fn validate_config(config: &SilentSessionConfig) -> ConfigValidationResult {
    let mut errors = vec![];
    let mut warnings = vec![];
    if config.schema != crate::silent_session::SILENT_SESSION_CONFIG_SCHEMA {
        errors.push("unsupported config schema".into());
    }
    if !config.identity.project_root.is_absolute() {
        errors.push("project_root must be absolute".into());
    }
    if config.identity.continuity_id.trim().is_empty() {
        errors.push("continuity_id is required".into());
    }
    if config.model.auth_profile_ref.trim().is_empty() {
        errors.push("auth_profile_ref is required; raw credentials are forbidden".into());
    }
    if config.output.chunk_max_bytes == 0 {
        errors.push("chunk_max_bytes must be positive".into());
    }
    if let Err(error) =
        crate::silent_session_retry::validate_retry_budgets(&config.supervision.retry_budgets)
    {
        errors.push(format!("invalid typed retry budgets: {error}"));
    }
    if config.model.allowed_fallbacks.is_empty()
        && matches!(
            config.model.fallback_policy,
            crate::silent_session::ModelFallbackPolicy::ExplicitAllowList
        )
    {
        errors.push("explicit fallback policy requires allowed_fallbacks".into());
    }
    if config.resources.max_wall_clock_seconds.is_none() {
        warnings.push("wall-clock budget is unbounded".into());
    }
    ConfigValidationResult {
        valid: errors.is_empty(),
        errors,
        warnings,
    }
}

fn reject_inline_secrets(value: &Value) -> anyhow::Result<()> {
    if let Value::Object(map) = value {
        for (key, value) in map {
            let normalized = key.to_ascii_lowercase();
            let secret_shaped = normalized.contains("password")
                || normalized.contains("api_key")
                || normalized.contains("access_token")
                || normalized.contains("refresh_token")
                || normalized == "secret"
                || normalized.ends_with("_secret")
                || normalized.contains("credential");
            anyhow::ensure!(
                !secret_shaped || normalized.ends_with("_ref"),
                "inline secrets are forbidden; use a secret reference"
            );
            reject_inline_secrets(value)?;
        }
    }
    Ok(())
}

fn merge_json(target: &mut Value, patch: Value) {
    match (target, patch) {
        (Value::Object(target), Value::Object(patch)) => {
            for (key, value) in patch {
                merge_json(target.entry(key).or_insert(Value::Null), value);
            }
        }
        (target, patch) => *target = patch,
    }
}

fn collect_leaf_paths(prefix: &str, value: &Value, output: &mut Vec<String>) {
    if let Value::Object(map) = value {
        for (key, value) in map {
            let path = format!("{prefix}/{}", key.replace('~', "~0").replace('/', "~1"));
            collect_leaf_paths(&path, value, output);
        }
    } else {
        output.push(prefix.to_string());
    }
}

fn classify_changes(
    before: &SilentSessionConfig,
    after: &SilentSessionConfig,
) -> anyhow::Result<BTreeMap<String, ConfigMutationClass>> {
    let before = serde_json::to_value(before)?;
    let after = serde_json::to_value(after)?;
    let mut paths = BTreeSet::new();
    changed_paths("", &before, &after, &mut paths);
    Ok(paths
        .into_iter()
        .map(|path| {
            let class = if path.starts_with("/identity/project_identity_ref")
                || path.starts_with("/identity/continuity_id")
            {
                ConfigMutationClass::Immutable
            } else if path.starts_with("/harness")
                || path.starts_with("/model")
                || path.starts_with("/identity/project_root")
                || path.starts_with("/workspace")
            {
                ConfigMutationClass::RestartRequired
            } else {
                ConfigMutationClass::HotMutable
            };
            (path, class)
        })
        .collect())
}

fn changed_paths(prefix: &str, before: &Value, after: &Value, output: &mut BTreeSet<String>) {
    match (before, after) {
        (Value::Object(a), Value::Object(b)) => {
            for key in a.keys().chain(b.keys()).collect::<BTreeSet<_>>() {
                let path = format!("{prefix}/{}", key.replace('~', "~0").replace('/', "~1"));
                changed_paths(
                    &path,
                    a.get(key).unwrap_or(&Value::Null),
                    b.get(key).unwrap_or(&Value::Null),
                    output,
                );
            }
        }
        _ if before != after => {
            output.insert(prefix.to_string());
        }
        _ => {}
    }
}

pub fn redacted_config_hash(config: &SilentSessionConfig) -> anyhow::Result<String> {
    let bytes = serde_json::to_vec(config)?;
    Ok(hex::encode(Sha256::digest(bytes)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::persistence_sqlite::SqlitePersistence;
    use crate::silent_session::*;
    use crate::silent_session_authorization::{
        SILENT_SESSION_APPROVAL_SCHEMA, SilentSessionApproval,
    };
    use crate::types::FocusaConfig;
    use std::path::PathBuf;

    fn config() -> SilentSessionConfig {
        SilentSessionConfig {
            schema: SILENT_SESSION_CONFIG_SCHEMA.into(),
            identity: SilentSessionIdentityConfig {
                display_name: "test".into(),
                project_root: crate::test_support::absolute_path("silent-config-project"),
                project_identity_ref: "project:test".into(),
                continuity_id: "main".into(),
                work_item_ref: Some("item".into()),
                mission: "mission".into(),
                agent_identity_ref: "agent".into(),
                role_profile_ref: "role".into(),
            },
            harness: HarnessConfig {
                kind: HarnessKind::Pi,
                adapter_version: "1".into(),
                native_resume_policy: NativeResumePolicy::Prefer,
            },
            model: SilentSessionModelConfig {
                requested: ModelBinding {
                    provider: "openai".into(),
                    model: "gpt".into(),
                    thinking: None,
                },
                selection_policy: ModelSelectionPolicy::Exact,
                fallback_policy: ModelFallbackPolicy::Disabled,
                allowed_fallbacks: vec![],
                auth_profile_ref: "auth:test".into(),
                require_entitlement_preflight: true,
                require_runtime_model_confirmation: true,
            },
            workspace: WorkspaceConfig {
                strategy: WorkspaceStrategy::IsolatedWorktree,
                source_root: crate::test_support::absolute_path("silent-config-project"),
                worktree_root: Some(crate::test_support::absolute_path("silent-config-worktree")),
                base_ref: Some("main".into()),
                branch_name: Some("work".into()),
                integration_policy: IntegrationPolicy::Manual,
            },
            bootstrap_target_profile: "pi".into(),
            bootstrap_packet_mode: "rules_and_context".into(),
            bootstrap_verification_required: true,
            supervision: SupervisionConfig {
                restart_policy: "on_failure".into(),
                max_process_restarts: 1,
                max_transport_retries: 2,
                retry_backoff_ms: 100,
                retry_budgets: crate::silent_session_retry::default_retry_budgets(),
                soft_pause_timeout_ms: 1000,
                graceful_stop_timeout_ms: 1000,
                checkpoint_interval_seconds: 60,
                checkpoint_event_interval: 100,
                waiting_input_timeout_seconds: 300,
                silent_output_warning_seconds: 120,
            },
            resources: ResourceLimits {
                priority: 0,
                max_wall_clock_seconds: Some(3600),
                max_cpu_percent: Some(100.0),
                max_memory_bytes: Some(1_000_000),
                max_pids: Some(10),
                max_disk_bytes: Some(1_000_000),
                max_output_bytes: Some(1_000_000),
                max_tokens: Some(10_000),
                max_cost_usd: Some(1.0),
                max_turns: Some(20),
            },
            output: OutputPolicy {
                persist_stdout: true,
                persist_stderr: true,
                persist_semantic_events: true,
                chunk_max_bytes: 1024,
                chunk_max_seconds: 60,
                redaction_profile_ref: "redact".into(),
                operator_projection_budget: 1000,
                raw_retention_policy_ref: "retention".into(),
            },
            governance: GovernancePolicy {
                context_authority_required: true,
                risky_mutation_preflight_required: true,
                destructive_actions_allowed: false,
                writer_lease_required: true,
                completion_receipt_required: true,
                evidence_policy_ref: "evidence".into(),
                policy_locks: vec![],
            },
            notifications: NotificationPolicy {
                waiting_input: true,
                blocked: true,
                failed: true,
                completed: true,
                model_mismatch: true,
                budget_pressure: true,
                channels: vec!["operator".into()],
            },
            retention: RetentionConfig {
                policy_ref: "retention".into(),
                evidence_hold: false,
            },
        }
    }

    #[test]
    fn precedence_locks_and_mutation_classes_fail_closed() {
        let session_id = SilentSessionId::new();
        let manager = SilentSessionConfigManager::new(
            session_id,
            config(),
            vec![ConfigPolicyLock {
                json_pointer: "/governance/destructive_actions_allowed".into(),
                source: "org".into(),
            }],
        )
        .unwrap();
        assert!(
            manager
                .preview(vec![ConfigLayer {
                    source: ConfigSource::OperatorEdit,
                    patch: serde_json::json!({"governance":{"destructive_actions_allowed":true}}),
                }])
                .is_err()
        );
        assert!(
            manager
                .preview(vec![ConfigLayer {
                    source: ConfigSource::ExplicitOverride,
                    patch: serde_json::json!({"model":{"api_key":"raw-secret"}}),
                }])
                .is_err()
        );
        let hot = manager
            .preview(vec![ConfigLayer {
                source: ConfigSource::ExplicitOverride,
                patch: serde_json::json!({"notifications":{"completed":false}}),
            }])
            .unwrap();
        assert_eq!(
            hot.mutation_classes["/notifications/completed"],
            ConfigMutationClass::HotMutable
        );
        let immutable = manager
            .preview(vec![ConfigLayer {
                source: ConfigSource::ExplicitOverride,
                patch: serde_json::json!({"identity":{"continuity_id":"other"}}),
            }])
            .unwrap();
        assert_eq!(
            immutable.mutation_classes["/identity/continuity_id"],
            ConfigMutationClass::Immutable
        );
    }

    #[test]
    fn typed_retry_budget_config_requires_every_independent_class() {
        let mut incomplete = config();
        incomplete
            .supervision
            .retry_budgets
            .remove(&crate::silent_session_retry::RetryClass::WorkItem);
        assert!(
            SilentSessionConfigManager::new(SilentSessionId::new(), incomplete, vec![]).is_err()
        );

        let mut invalid = config();
        invalid
            .supervision
            .retry_budgets
            .get_mut(&crate::silent_session_retry::RetryClass::Provider)
            .unwrap()
            .max_retries = 0;
        assert!(SilentSessionConfigManager::new(SilentSessionId::new(), invalid, vec![]).is_err());
    }

    #[test]
    fn transactional_apply_requires_cas_and_approval_then_can_verify_and_rollback() {
        let mut manager =
            SilentSessionConfigManager::new(SilentSessionId::new(), config(), vec![]).unwrap();
        let original = manager.current().revision_id;
        let layer = ConfigLayer {
            source: ConfigSource::OperatorEdit,
            patch: serde_json::json!({"model":{"requested":{"model":"gpt-next"}}}),
        };
        assert!(manager.apply(original, vec![layer.clone()], None).is_err());
        let applied = manager
            .apply(original, vec![layer], Some("approval:test".into()))
            .unwrap();
        let hash = redacted_config_hash(&applied.config).unwrap();
        manager.verify_current_hash(&hash).unwrap();
        assert!(manager.apply(original, vec![], None).is_err());
        let rolled_back = manager
            .rollback(applied.revision_id, original, "approval:rollback".into())
            .unwrap();
        assert_eq!(rolled_back.config, config());
        assert_eq!(rolled_back.rollback_target, Some(original));
    }

    #[test]
    fn config_revisions_survive_restart_and_persistence_cas_rejects_stale_writers() {
        let dir = std::env::temp_dir().join(format!("focusa-config-test-{}", uuid::Uuid::now_v7()));
        std::fs::create_dir_all(&dir).unwrap();
        let daemon_config = FocusaConfig {
            data_dir: dir.to_string_lossy().into_owned(),
            ..FocusaConfig::default()
        };
        let persistence = SqlitePersistence::new(&daemon_config).unwrap();
        let session_id = SilentSessionId::new();
        let run_id = SilentSessionRunId::new();
        let mut manager = SilentSessionConfigManager::new(session_id, config(), vec![]).unwrap();
        let original = manager.current().clone();
        let now = Utc::now();
        let mut session = SilentSession {
            schema: SILENT_SESSION_SCHEMA.into(),
            versions: SilentSessionVersions::default(),
            session_id,
            display_name: "config persistence".into(),
            created_at: now,
            created_by_actor_ref: "actor:test".into(),
            operator_principal_ref: "operator:test".into(),
            os_execution_user: "test".into(),
            project_root: crate::test_support::absolute_path("silent-config-project"),
            project_identity_ref: "project:test".into(),
            continuity_id: "main".into(),
            trajectory_ref: None,
            workpoint_ref: None,
            work_item_ref: None,
            operator_ask: crate::silent_session::OperatorAskBinding::capture(
                "ask:config-test",
                "persist config",
                1,
                Utc::now(),
            ),
            mission: "persist config".into(),
            lifecycle_state: SilentSessionLifecycleState::Draft,
            health: SilentSessionHealth::Healthy,
            semantic_observation: None,
            active_run_id: Some(run_id),
            config_revision_id: original.revision_id,
            writer_lease_ref: None,
            retention_policy_ref: "retention:test".into(),
            receipt_refs: vec![],
        };
        let event = SilentSessionEvent {
            schema: SILENT_SESSION_EVENT_SCHEMA.into(),
            event_id: SilentSessionEventId::new(),
            session_id,
            run_id,
            seq: 1,
            occurred_at: now,
            observed_at: now,
            kind: "lifecycle.draft".into(),
            source: "test".into(),
            provenance: ObservationProvenance::VerificationConfirmed,
            canonical: true,
            payload: serde_json::json!({}),
            artifact_refs: vec![],
            correlation_id: uuid::Uuid::now_v7(),
            redaction: RedactionMetadata {
                applied: true,
                classes: vec!["config".into()],
            },
        };
        persistence
            .persist_silent_session_event(&session, &event)
            .unwrap();
        persistence
            .put_initial_silent_session_config_revision(
                &session,
                &original,
                &redacted_config_hash(&original.config).unwrap(),
            )
            .unwrap();

        let applied = manager
            .apply(
                original.revision_id,
                vec![ConfigLayer {
                    source: ConfigSource::OperatorEdit,
                    patch: serde_json::json!({"notifications":{"completed":false}}),
                }],
                None,
            )
            .unwrap();
        session.config_revision_id = applied.revision_id;
        let approval = SilentSessionApproval {
            schema: SILENT_SESSION_APPROVAL_SCHEMA.into(),
            approval_id: "approval:config-persistence".into(),
            operator_actor_ref: "operator:test".into(),
            action: "config.revise".into(),
            project_identity_ref: session.project_identity_ref.clone(),
            continuity_id: session.continuity_id.clone(),
            workpoint_ref: None,
            session_id: Some(session_id),
            run_id: session.active_run_id,
            config_hash: redacted_config_hash(&applied.config).unwrap(),
            action_digest: "digest:config-persistence".into(),
            model_binding: "test".into(),
            workspace_ref: "test".into(),
            risk_class: "controlled".into(),
            expires_at: Utc::now() + chrono::Duration::minutes(5),
            permitted_side_effects: vec!["config:apply".into()],
        };
        persistence.put_silent_session_approval(&approval).unwrap();
        persistence
            .persist_silent_session_config_revision_cas(
                original.revision_id,
                &approval.approval_id,
                &approval.action_digest,
                Utc::now(),
                &session,
                &applied,
                &redacted_config_hash(&applied.config).unwrap(),
            )
            .unwrap();
        let mut stale_approval = approval.clone();
        stale_approval.approval_id = "approval:stale-config-persistence".into();
        stale_approval.action_digest = "digest:stale-config-persistence".into();
        persistence
            .put_silent_session_approval(&stale_approval)
            .unwrap();
        assert!(
            persistence
                .persist_silent_session_config_revision_cas(
                    original.revision_id,
                    &stale_approval.approval_id,
                    &stale_approval.action_digest,
                    Utc::now(),
                    &session,
                    &applied,
                    &redacted_config_hash(&applied.config).unwrap(),
                )
                .is_err()
        );
        assert_eq!(
            persistence
                .load_silent_session_approval(&stale_approval.approval_id)
                .unwrap(),
            Some(stale_approval),
            "failed CAS must not consume its approval"
        );
        drop(persistence);

        let reopened = SqlitePersistence::new(&daemon_config).unwrap();
        let history = reopened
            .load_silent_session_config_history(session_id)
            .unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].0, original);
        assert_eq!(history[1].0, applied);
        assert_eq!(
            reopened
                .load_silent_session(session_id)
                .unwrap()
                .unwrap()
                .config_revision_id,
            session.config_revision_id
        );
    }
}
