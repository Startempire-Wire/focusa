use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use super::SilentSessionConfig;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ConfigLayerKind {
    CompiledDefaults,
    ExecutionProfile,
    ProjectPolicy,
    BehavioralPreset,
    ContextAuthority,
    SessionRequest,
    OperatorRevision,
    ConstitutionalPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConfigLayer {
    pub kind: ConfigLayerKind,
    pub source_ref: String,
    pub values: Value,
    pub locks: Vec<ConfigPolicyLock>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NamedExecutionProfile {
    pub name: String,
    pub values: Value,
}

impl NamedExecutionProfile {
    pub fn into_layer(self) -> Result<ConfigLayer, ConfigResolutionError> {
        validate_layer_fields(
            &self.values,
            &[
                "harness",
                "model",
                "workspace",
                "identity.agent_identity_ref",
                "identity.role_profile_ref",
            ],
            "execution profile",
        )?;
        Ok(ConfigLayer {
            kind: ConfigLayerKind::ExecutionProfile,
            source_ref: format!("profile:{}", self.name),
            values: self.values,
            locks: Vec::new(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NamedBehavioralPreset {
    pub name: String,
    pub values: Value,
}

impl NamedBehavioralPreset {
    pub fn into_layer(self) -> Result<ConfigLayer, ConfigResolutionError> {
        validate_layer_fields(
            &self.values,
            &[
                "supervision",
                "resources",
                "output",
                "governance",
                "notifications",
            ],
            "behavioral preset",
        )?;
        Ok(ConfigLayer {
            kind: ConfigLayerKind::BehavioralPreset,
            source_ref: format!("preset:{}", self.name),
            values: self.values,
            locks: Vec::new(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConfigPolicyLock {
    pub field_path: String,
    pub expected_value: Value,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FieldProvenance {
    pub layer: ConfigLayerKind,
    pub source_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConfigValidation {
    pub valid: bool,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EffectiveSilentSessionConfig {
    pub requested_config: SilentSessionConfig,
    pub resolved_effective_config: SilentSessionConfig,
    pub field_provenance: BTreeMap<String, FieldProvenance>,
    pub policy_locks: Vec<ConfigPolicyLock>,
    pub restart_required_fields: Vec<String>,
    pub warnings: Vec<String>,
    pub validation: ConfigValidation,
    pub redacted_config_hash: String,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ConfigResolutionError {
    #[error("configuration layers are not in precedence order")]
    PrecedenceOrder,
    #[error("configuration layer must be a JSON object: {0}")]
    LayerNotObject(String),
    #[error("policy lock violation at {field_path}: {reason}")]
    PolicyLockViolation { field_path: String, reason: String },
    #[error("invalid policy lock path: {0}")]
    InvalidLockPath(String),
    #[error("invalid effective config: {0}")]
    InvalidEffectiveConfig(String),
    #[error("{layer} cannot set field {field_path}")]
    LayerFieldNotAllowed { layer: String, field_path: String },
}

pub fn resolve_silent_session_config(
    requested: SilentSessionConfig,
    layers: Vec<ConfigLayer>,
) -> Result<EffectiveSilentSessionConfig, ConfigResolutionError> {
    if !layers.windows(2).all(|pair| pair[0].kind <= pair[1].kind) {
        return Err(ConfigResolutionError::PrecedenceOrder);
    }
    let mut resolved = serde_json::to_value(&requested)
        .map_err(|error| ConfigResolutionError::InvalidEffectiveConfig(error.to_string()))?;
    let base_value = resolved.clone();
    let mut requested_value = resolved.clone();
    let mut provenance = BTreeMap::new();
    let mut base_leaves = Vec::new();
    collect_leaves("", &resolved, &mut base_leaves);
    for (path, _) in base_leaves {
        provenance.insert(
            path,
            FieldProvenance {
                layer: ConfigLayerKind::CompiledDefaults,
                source_ref: "compiled:safe-defaults".into(),
            },
        );
    }
    let mut active_locks: BTreeMap<String, ConfigPolicyLock> = BTreeMap::new();
    for layer in &layers {
        if !layer.values.is_object() {
            return Err(ConfigResolutionError::LayerNotObject(
                layer.source_ref.clone(),
            ));
        }
        let mut leaves = Vec::new();
        collect_leaves("", &layer.values, &mut leaves);
        for (path, value) in leaves {
            if let Some(lock) = active_locks.get(&path) {
                if lock.expected_value != *value {
                    return Err(ConfigResolutionError::PolicyLockViolation {
                        field_path: path,
                        reason: lock.reason.clone(),
                    });
                }
            }
            set_path(&mut resolved, &path, value.clone())?;
            if layer.kind <= ConfigLayerKind::SessionRequest {
                set_path(&mut requested_value, &path, value.clone())?;
            }
            provenance.insert(
                path,
                FieldProvenance {
                    layer: layer.kind,
                    source_ref: layer.source_ref.clone(),
                },
            );
        }
        for lock in &layer.locks {
            let current = get_path(&resolved, &lock.field_path)
                .ok_or_else(|| ConfigResolutionError::InvalidLockPath(lock.field_path.clone()))?;
            if current != &lock.expected_value {
                return Err(ConfigResolutionError::PolicyLockViolation {
                    field_path: lock.field_path.clone(),
                    reason: lock.reason.clone(),
                });
            }
            active_locks.insert(lock.field_path.clone(), lock.clone());
        }
    }
    let validation = validate_effective_value(&resolved);
    if !validation.valid {
        return Err(ConfigResolutionError::InvalidEffectiveConfig(
            validation.errors.join("; "),
        ));
    }
    let effective: SilentSessionConfig = serde_json::from_value(resolved.clone())
        .map_err(|error| ConfigResolutionError::InvalidEffectiveConfig(error.to_string()))?;
    let requested_config: SilentSessionConfig = serde_json::from_value(requested_value.clone())
        .map_err(|error| ConfigResolutionError::InvalidEffectiveConfig(error.to_string()))?;
    let mut changed = Vec::new();
    collect_changed_paths("", &base_value, &resolved, &mut changed);
    let restart_required_fields = changed
        .into_iter()
        .filter(|path| mutation_class(path) == ConfigMutationClass::RestartRequired)
        .collect::<Vec<_>>();
    let canonical = serde_json::to_vec(&resolved)
        .map_err(|error| ConfigResolutionError::InvalidEffectiveConfig(error.to_string()))?;
    Ok(EffectiveSilentSessionConfig {
        requested_config,
        resolved_effective_config: effective,
        field_provenance: provenance,
        policy_locks: active_locks.into_values().collect(),
        restart_required_fields,
        warnings: Vec::new(),
        validation,
        redacted_config_hash: hex::encode(Sha256::digest(canonical)),
    })
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConfigMutationClass {
    HotMutable,
    RestartRequired,
    Immutable,
}

pub fn mutation_class(path: &str) -> ConfigMutationClass {
    const IMMUTABLE: &[&str] = &[
        "identity.project_root",
        "identity.continuity_id",
        "retention.evidence_hold",
    ];
    const RESTART: &[&str] = &[
        "harness",
        "model.provider",
        "model.model",
        "model.thinking",
        "model.fallback_policy",
        "model.auth_profile_ref",
        "workspace.strategy",
        "workspace.worktree_root",
        "identity.project_root",
    ];
    if IMMUTABLE
        .iter()
        .any(|prefix| path == *prefix || path.starts_with(&format!("{prefix}.")))
    {
        ConfigMutationClass::Immutable
    } else if RESTART
        .iter()
        .any(|prefix| path == *prefix || path.starts_with(&format!("{prefix}.")))
    {
        ConfigMutationClass::RestartRequired
    } else {
        ConfigMutationClass::HotMutable
    }
}

fn validate_layer_fields(
    values: &Value,
    allowed_prefixes: &[&str],
    layer: &str,
) -> Result<(), ConfigResolutionError> {
    if !values.is_object() {
        return Err(ConfigResolutionError::LayerNotObject(layer.into()));
    }
    let mut leaves = Vec::new();
    collect_leaves("", values, &mut leaves);
    for (path, _) in leaves {
        if !allowed_prefixes
            .iter()
            .any(|prefix| path == *prefix || path.starts_with(&format!("{prefix}.")))
        {
            return Err(ConfigResolutionError::LayerFieldNotAllowed {
                layer: layer.into(),
                field_path: path,
            });
        }
    }
    Ok(())
}

fn validate_effective_value(value: &Value) -> ConfigValidation {
    let mut errors = Vec::new();
    validate_secret_refs("", value, &mut errors);
    ConfigValidation {
        valid: errors.is_empty(),
        errors,
    }
}

fn validate_secret_refs(prefix: &str, value: &Value, errors: &mut Vec<String>) {
    if let Value::Object(map) = value {
        for (key, child) in map {
            let path = if prefix.is_empty() {
                key.clone()
            } else {
                format!("{prefix}.{key}")
            };
            let lower = key.to_ascii_lowercase();
            let sensitive = ["secret", "password", "credential"]
                .iter()
                .any(|needle| lower.contains(needle))
                || lower == "token"
                || lower.ends_with("_token");
            if sensitive && !key.ends_with("_ref") {
                errors.push(format!("raw secret field forbidden: {path}"));
            }
            validate_secret_refs(&path, child, errors);
        }
    }
}

fn collect_leaves<'a>(prefix: &str, value: &'a Value, out: &mut Vec<(String, &'a Value)>) {
    if let Value::Object(map) = value {
        for (key, child) in map {
            let path = if prefix.is_empty() {
                key.clone()
            } else {
                format!("{prefix}.{key}")
            };
            if child.is_object() {
                collect_leaves(&path, child, out);
            } else {
                out.push((path, child));
            }
        }
    }
}

fn set_path(root: &mut Value, path: &str, value: Value) -> Result<(), ConfigResolutionError> {
    let mut current = root;
    let mut parts = path.split('.').peekable();
    while let Some(part) = parts.next() {
        if parts.peek().is_none() {
            let map = current
                .as_object_mut()
                .ok_or_else(|| ConfigResolutionError::InvalidLockPath(path.into()))?;
            map.insert(part.into(), value);
            return Ok(());
        }
        current = current
            .get_mut(part)
            .ok_or_else(|| ConfigResolutionError::InvalidLockPath(path.into()))?;
    }
    Err(ConfigResolutionError::InvalidLockPath(path.into()))
}

fn get_path<'a>(root: &'a Value, path: &str) -> Option<&'a Value> {
    path.split('.')
        .try_fold(root, |value, part| value.get(part))
}

fn collect_changed_paths(prefix: &str, before: &Value, after: &Value, out: &mut Vec<String>) {
    match (before, after) {
        (Value::Object(left), Value::Object(right)) => {
            let keys = left.keys().chain(right.keys()).collect::<BTreeSet<_>>();
            for key in keys {
                let path = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{prefix}.{key}")
                };
                collect_changed_paths(
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
