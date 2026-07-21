use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    ConfigLayer, ConfigMutationClass, ConfigResolutionError, ConfigRevisionId,
    EffectiveSilentSessionConfig, SilentSessionConfig, mutation_class,
    resolve_silent_session_config,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConfigFieldDiff {
    pub field_path: String,
    pub before: Value,
    pub after: Value,
    pub mutation_class: ConfigMutationClass,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConfigRevisionPlan {
    pub revision_id: ConfigRevisionId,
    pub prior_config: SilentSessionConfig,
    pub candidate: EffectiveSilentSessionConfig,
    pub effective_diff: Vec<ConfigFieldDiff>,
    pub hot_fields: Vec<String>,
    pub restart_required_fields: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConfigRevisionStage {
    Previewed,
    Validated,
    GateApproved,
    Persisted,
    Applied,
    RestartPlanned,
    Verified,
    Committed,
    RolledBack,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConfigRevisionOutcome {
    pub revision_id: ConfigRevisionId,
    pub stage: ConfigRevisionStage,
    pub committed: bool,
    pub restart_required: bool,
    pub failure: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigRevisionError {
    #[error(transparent)]
    Resolution(#[from] ConfigResolutionError),
    #[error("immutable fields cannot be revised: {0}")]
    ImmutableMutation(String),
    #[error("context authority or operator approval is required")]
    GateRequired,
    #[error("resource limits may only be tightened while running: {0}")]
    ResourceLimitLoosened(String),
    #[error("revision persistence failed before apply: {0}")]
    Persistence(String),
    #[error("revision rollback failed after {failure}: {rollback}")]
    RollbackFailed { failure: String, rollback: String },
}

pub trait ConfigRevisionBackend {
    fn persist_pending(&mut self, plan: &ConfigRevisionPlan) -> anyhow::Result<()>;
    fn apply_hot(&mut self, config: &SilentSessionConfig, fields: &[String]) -> anyhow::Result<()>;
    fn create_restart_plan(
        &mut self,
        config: &SilentSessionConfig,
        fields: &[String],
    ) -> anyhow::Result<()>;
    fn verify(&mut self, plan: &ConfigRevisionPlan) -> anyhow::Result<bool>;
    fn commit(&mut self, plan: &ConfigRevisionPlan) -> anyhow::Result<()>;
    fn rollback(&mut self, prior: &SilentSessionConfig) -> anyhow::Result<()>;
}

pub fn preview_config_revision(
    current: SilentSessionConfig,
    requested: SilentSessionConfig,
    layers: Vec<ConfigLayer>,
) -> Result<ConfigRevisionPlan, ConfigRevisionError> {
    let candidate = resolve_silent_session_config(requested, layers)?;
    let before = serde_json::to_value(&current)
        .map_err(|error| ConfigResolutionError::InvalidEffectiveConfig(error.to_string()))?;
    let after = serde_json::to_value(&candidate.resolved_effective_config)
        .map_err(|error| ConfigResolutionError::InvalidEffectiveConfig(error.to_string()))?;
    let mut paths = Vec::new();
    changed_paths("", &before, &after, &mut paths);
    let mut effective_diff = Vec::with_capacity(paths.len());
    let mut hot_fields = Vec::new();
    let mut restart_required_fields = Vec::new();
    let mut immutable_fields = Vec::new();
    for path in paths {
        let class = mutation_class(&path);
        let before_value = value_at(&before, &path).cloned().unwrap_or(Value::Null);
        let after_value = value_at(&after, &path).cloned().unwrap_or(Value::Null);
        if path.starts_with("resources.max_")
            && !resource_limit_is_tightened(&before_value, &after_value)
        {
            return Err(ConfigRevisionError::ResourceLimitLoosened(path));
        }
        match class {
            ConfigMutationClass::HotMutable => hot_fields.push(path.clone()),
            ConfigMutationClass::RestartRequired => restart_required_fields.push(path.clone()),
            ConfigMutationClass::Immutable => immutable_fields.push(path.clone()),
        }
        effective_diff.push(ConfigFieldDiff {
            before: before_value,
            after: after_value,
            field_path: path,
            mutation_class: class,
        });
    }
    if !immutable_fields.is_empty() {
        return Err(ConfigRevisionError::ImmutableMutation(
            immutable_fields.join(","),
        ));
    }
    Ok(ConfigRevisionPlan {
        revision_id: ConfigRevisionId::new(),
        prior_config: current,
        candidate,
        effective_diff,
        hot_fields,
        restart_required_fields,
    })
}

pub fn execute_config_revision(
    plan: &ConfigRevisionPlan,
    gate_approved: bool,
    backend: &mut impl ConfigRevisionBackend,
) -> Result<ConfigRevisionOutcome, ConfigRevisionError> {
    if !gate_approved {
        return Err(ConfigRevisionError::GateRequired);
    }
    backend
        .persist_pending(plan)
        .map_err(|error| ConfigRevisionError::Persistence(error.to_string()))?;
    let restart_required = !plan.restart_required_fields.is_empty();
    let apply_result = if restart_required {
        backend.create_restart_plan(
            &plan.candidate.resolved_effective_config,
            &plan.restart_required_fields,
        )
    } else {
        backend.apply_hot(&plan.candidate.resolved_effective_config, &plan.hot_fields)
    };
    if let Err(error) = apply_result {
        return rollback_outcome(plan, restart_required, error.to_string(), backend);
    }
    match backend.verify(plan) {
        Ok(true) => {}
        Ok(false) => {
            return rollback_outcome(
                plan,
                restart_required,
                "effective revision verification returned false".into(),
                backend,
            );
        }
        Err(error) => {
            return rollback_outcome(plan, restart_required, error.to_string(), backend);
        }
    }
    if let Err(error) = backend.commit(plan) {
        return rollback_outcome(plan, restart_required, error.to_string(), backend);
    }
    Ok(ConfigRevisionOutcome {
        revision_id: plan.revision_id,
        stage: ConfigRevisionStage::Committed,
        committed: true,
        restart_required,
        failure: None,
    })
}

fn rollback_outcome(
    plan: &ConfigRevisionPlan,
    restart_required: bool,
    failure: String,
    backend: &mut impl ConfigRevisionBackend,
) -> Result<ConfigRevisionOutcome, ConfigRevisionError> {
    if let Err(error) = backend.rollback(&plan.prior_config) {
        return Err(ConfigRevisionError::RollbackFailed {
            failure,
            rollback: error.to_string(),
        });
    }
    Ok(ConfigRevisionOutcome {
        revision_id: plan.revision_id,
        stage: ConfigRevisionStage::RolledBack,
        committed: false,
        restart_required,
        failure: Some(failure),
    })
}

fn changed_paths(prefix: &str, before: &Value, after: &Value, out: &mut Vec<String>) {
    match (before, after) {
        (Value::Object(left), Value::Object(right)) => {
            let keys = left.keys().chain(right.keys()).collect::<BTreeSet<_>>();
            for key in keys {
                let path = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{prefix}.{key}")
                };
                changed_paths(
                    &path,
                    left.get(key).unwrap_or(&Value::Null),
                    right.get(key).unwrap_or(&Value::Null),
                    out,
                );
            }
        }
        _ if before != after => out.push(prefix.into()),
        _ => {}
    }
}

fn resource_limit_is_tightened(before: &Value, after: &Value) -> bool {
    match (before, after) {
        (Value::Null, Value::Number(_)) => true,
        (Value::Number(old), Value::Number(new)) => old
            .as_f64()
            .zip(new.as_f64())
            .is_some_and(|(old, new)| new <= old),
        _ => before == after,
    }
}

fn value_at<'a>(root: &'a Value, path: &str) -> Option<&'a Value> {
    path.split('.')
        .try_fold(root, |value, part| value.get(part))
}
