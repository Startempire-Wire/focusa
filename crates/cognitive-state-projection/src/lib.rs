use agent_stateful_cognitive_runtime::{
    MemoryAuthority, MemoryNamespace, canonical_memory_namespaces,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CognitiveStateProjection {
    pub schema: String,
    pub revision: u64,
    pub binding_epoch_id: String,
    pub namespaces: BTreeMap<String, Value>,
    pub projection_digest: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WritableMemoryDiff {
    pub schema: String,
    pub binding_epoch_id: String,
    pub namespace: String,
    pub prior_digest: Option<String>,
    pub value: Value,
    pub value_digest: String,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ProjectionError {
    #[error("binding epoch is missing")]
    MissingEpoch,
    #[error("namespace is unknown: {0}")]
    UnknownNamespace(String),
    #[error("namespace is not readable by a provider: {0}")]
    NamespaceNotReadable(String),
    #[error("namespace is not writable by an agent: {0}")]
    NamespaceNotWritable(String),
    #[error("state contains a forbidden secret-like field: {0}")]
    SecretField(String),
    #[error("writable-memory diff schema is unsupported")]
    UnsupportedDiffSchema,
    #[error("writable-memory diff digest is invalid")]
    InvalidDigest,
}

pub fn build_read_projection(
    revision: u64,
    binding_epoch_id: &str,
    canonical_state: &BTreeMap<String, Value>,
) -> Result<CognitiveStateProjection, ProjectionError> {
    if binding_epoch_id.trim().is_empty() {
        return Err(ProjectionError::MissingEpoch);
    }
    let authority = authority_map();
    let mut namespaces = BTreeMap::new();
    for (name, value) in canonical_state {
        match authority.get(name).copied() {
            Some(MemoryAuthority::ReadOnlyProjection) => {
                reject_secret_fields(value, name)?;
                namespaces.insert(name.clone(), value.clone());
            }
            Some(MemoryAuthority::AgentWritable) => {}
            Some(MemoryAuthority::CanonicalForbidden) => {
                return Err(ProjectionError::NamespaceNotReadable(name.clone()));
            }
            None => return Err(ProjectionError::UnknownNamespace(name.clone())),
        }
    }
    let projection_digest = digest(&(revision, binding_epoch_id, &namespaces));
    Ok(CognitiveStateProjection {
        schema: "focusa.cognitive_state_projection.v1".into(),
        revision,
        binding_epoch_id: binding_epoch_id.into(),
        namespaces,
        projection_digest,
    })
}

pub fn validate_writable_diff(diff: &WritableMemoryDiff) -> Result<(), ProjectionError> {
    if diff.schema != "focusa.writable_memory_diff.v1" {
        return Err(ProjectionError::UnsupportedDiffSchema);
    }
    if diff.binding_epoch_id.trim().is_empty() {
        return Err(ProjectionError::MissingEpoch);
    }
    let authority = authority_map();
    match authority.get(&diff.namespace).copied() {
        Some(MemoryAuthority::AgentWritable) => {}
        Some(_) => {
            return Err(ProjectionError::NamespaceNotWritable(
                diff.namespace.clone(),
            ));
        }
        None => return Err(ProjectionError::UnknownNamespace(diff.namespace.clone())),
    }
    reject_secret_fields(&diff.value, &diff.namespace)?;
    if digest(&diff.value) != diff.value_digest {
        return Err(ProjectionError::InvalidDigest);
    }
    Ok(())
}

pub fn value_digest(value: &Value) -> String {
    digest(value)
}

fn authority_map() -> BTreeMap<String, MemoryAuthority> {
    canonical_memory_namespaces()
        .into_iter()
        .map(|MemoryNamespace { name, authority }| (name, authority))
        .collect()
}

fn reject_secret_fields(value: &Value, path: &str) -> Result<(), ProjectionError> {
    match value {
        Value::Object(values) => {
            for (key, nested) in values {
                let normalized = key.to_ascii_lowercase();
                if [
                    "password",
                    "secret",
                    "token",
                    "api_key",
                    "private_key",
                    "cookie",
                    "credential",
                    "wallet_key",
                ]
                .iter()
                .any(|forbidden| normalized.contains(forbidden))
                {
                    return Err(ProjectionError::SecretField(format!("{path}.{key}")));
                }
                reject_secret_fields(nested, &format!("{path}.{key}"))?;
            }
        }
        Value::Array(values) => {
            for (index, nested) in values.iter().enumerate() {
                reject_secret_fields(nested, &format!("{path}[{index}]"))?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn digest<T: Serialize>(value: &T) -> String {
    let bytes = serde_json::to_vec(value).expect("projection values must serialize");
    format!("sha256:{}", hex::encode(Sha256::digest(bytes)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn read_projection_excludes_writable_memory_and_rejects_canonical_truth() {
        let state = BTreeMap::from([
            ("identity_lineage".into(), json!({"agent":"a"})),
            ("working_memory".into(), json!({"scratch":"bounded"})),
        ]);
        let projection = build_read_projection(1, "epoch-1", &state).unwrap();
        assert!(projection.namespaces.contains_key("identity_lineage"));
        assert!(!projection.namespaces.contains_key("working_memory"));

        let forbidden = BTreeMap::from([("predictions".into(), json!({"canonical":true}))]);
        assert_eq!(
            build_read_projection(1, "epoch-1", &forbidden),
            Err(ProjectionError::NamespaceNotReadable("predictions".into()))
        );
    }

    #[test]
    fn writable_diff_accepts_beliefs_but_rejects_owner_truth_and_secrets() {
        let value = json!({"thesis":"bounded"});
        let valid = WritableMemoryDiff {
            schema: "focusa.writable_memory_diff.v1".into(),
            binding_epoch_id: "epoch-1".into(),
            namespace: "beliefs".into(),
            prior_digest: None,
            value_digest: value_digest(&value),
            value,
        };
        assert!(validate_writable_diff(&valid).is_ok());

        let mut owner = valid.clone();
        owner.namespace = "owner_truth".into();
        assert!(matches!(
            validate_writable_diff(&owner),
            Err(ProjectionError::NamespaceNotWritable(_))
        ));

        let secret_value = json!({"api_key":"not-allowed"});
        let mut secret = valid;
        secret.value_digest = value_digest(&secret_value);
        secret.value = secret_value;
        assert!(matches!(
            validate_writable_diff(&secret),
            Err(ProjectionError::SecretField(_))
        ));
    }
}
