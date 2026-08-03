use agent_stateful_cognitive_runtime::{RuntimeBinding, RuntimeMode};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentLifecycleStatus {
    Alive,
    Dead,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentCheckpoint {
    pub schema: String,
    pub binding: RuntimeBinding,
    pub lifecycle_status: AgentLifecycleStatus,
    pub state_revision: u64,
    pub memory_projection_digests: BTreeMap<String, String>,
    pub turn_receipt_refs: Vec<String>,
    pub afterlife_snapshot_ref: Option<String>,
    pub checkpoint_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestoredAgentCheckpoint {
    pub checkpoint: AgentCheckpoint,
    pub provider_access_allowed: bool,
    pub writable_memory_allowed: bool,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CheckpointError {
    #[error("checkpoint schema is unsupported")]
    UnsupportedSchema,
    #[error("runtime binding is invalid")]
    InvalidBinding,
    #[error("checkpoint digest is invalid")]
    InvalidDigest,
    #[error("checkpoint contains an invalid reference")]
    InvalidReference,
    #[error("dead agent checkpoint requires a read-only afterlife snapshot")]
    MissingAfterlifeSnapshot,
}

impl AgentCheckpoint {
    pub const SCHEMA: &'static str = "focusa.letta_agent_checkpoint.v1";

    pub fn create(
        binding: RuntimeBinding,
        lifecycle_status: AgentLifecycleStatus,
        state_revision: u64,
        memory_projection_digests: BTreeMap<String, String>,
        turn_receipt_refs: Vec<String>,
        afterlife_snapshot_ref: Option<String>,
    ) -> Result<Self, CheckpointError> {
        binding
            .validate()
            .map_err(|_| CheckpointError::InvalidBinding)?;
        validate_refs(memory_projection_digests.values().map(String::as_str))?;
        validate_refs(turn_receipt_refs.iter().map(String::as_str))?;
        if lifecycle_status == AgentLifecycleStatus::Dead
            && afterlife_snapshot_ref
                .as_deref()
                .is_none_or(|value| value.trim().is_empty())
        {
            return Err(CheckpointError::MissingAfterlifeSnapshot);
        }
        if let Some(reference) = afterlife_snapshot_ref.as_deref() {
            validate_refs([reference])?;
        }
        let mut checkpoint = Self {
            schema: Self::SCHEMA.into(),
            binding,
            lifecycle_status,
            state_revision,
            memory_projection_digests,
            turn_receipt_refs,
            afterlife_snapshot_ref,
            checkpoint_digest: String::new(),
        };
        checkpoint.checkpoint_digest = checkpoint.compute_digest();
        Ok(checkpoint)
    }

    pub fn restore(self) -> Result<RestoredAgentCheckpoint, CheckpointError> {
        if self.schema != Self::SCHEMA {
            return Err(CheckpointError::UnsupportedSchema);
        }
        self.binding
            .validate()
            .map_err(|_| CheckpointError::InvalidBinding)?;
        if self.compute_digest() != self.checkpoint_digest {
            return Err(CheckpointError::InvalidDigest);
        }
        if self.lifecycle_status == AgentLifecycleStatus::Dead
            && self
                .afterlife_snapshot_ref
                .as_deref()
                .is_none_or(|value| value.trim().is_empty())
        {
            return Err(CheckpointError::MissingAfterlifeSnapshot);
        }
        let alive = self.lifecycle_status == AgentLifecycleStatus::Alive;
        let provider_access_allowed = alive && self.binding.mode == RuntimeMode::LettaManaged;
        Ok(RestoredAgentCheckpoint {
            checkpoint: self,
            provider_access_allowed,
            writable_memory_allowed: alive,
        })
    }

    fn compute_digest(&self) -> String {
        let bytes = serde_json::to_vec(&(
            &self.schema,
            &self.binding,
            self.lifecycle_status,
            self.state_revision,
            &self.memory_projection_digests,
            &self.turn_receipt_refs,
            &self.afterlife_snapshot_ref,
        ))
        .expect("checkpoint tuple must serialize");
        format!("sha256:{}", hex::encode(Sha256::digest(bytes)))
    }
}

fn validate_refs<'a>(refs: impl IntoIterator<Item = &'a str>) -> Result<(), CheckpointError> {
    if refs.into_iter().any(|reference| {
        let value = reference.trim();
        value.is_empty()
            || value.contains('\n')
            || value.contains('\r')
            || value.to_ascii_lowercase().contains("secret")
    }) {
        Err(CheckpointError::InvalidReference)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_stateful_cognitive_runtime::{CognitiveLoopOwner, RuntimeEpochIdentity};
    use std::collections::BTreeSet;
    use uuid::Uuid;

    fn binding() -> RuntimeBinding {
        RuntimeBinding {
            schema: RuntimeBinding::SCHEMA.into(),
            mode: RuntimeMode::LettaManaged,
            owner: CognitiveLoopOwner::Letta,
            epoch: RuntimeEpochIdentity {
                epoch_id: Uuid::now_v7(),
                project_root: "/project".into(),
                continuity_id: "continuity".into(),
                agent_instance_id: "agent".into(),
                native_session_id: None,
            },
            provider_agent_id: Some("letta-agent".into()),
            admitted_client_tools: BTreeSet::new(),
        }
    }

    #[test]
    fn live_checkpoint_restores_provider_access_only_for_letta_managed_mode() {
        let checkpoint = AgentCheckpoint::create(
            binding(),
            AgentLifecycleStatus::Alive,
            7,
            BTreeMap::from([("working_memory".into(), "sha256:memory".into())]),
            vec!["receipt:turn-1".into()],
            None,
        )
        .unwrap();
        let restored = checkpoint.restore().unwrap();
        assert!(restored.provider_access_allowed);
        assert!(restored.writable_memory_allowed);
    }

    #[test]
    fn dead_checkpoint_cannot_resurrect_provider_or_writable_memory() {
        let checkpoint = AgentCheckpoint::create(
            binding(),
            AgentLifecycleStatus::Dead,
            9,
            BTreeMap::new(),
            vec!["receipt:death".into()],
            Some("snapshot:afterlife-read-only".into()),
        )
        .unwrap();
        let restored = checkpoint.restore().unwrap();
        assert!(!restored.provider_access_allowed);
        assert!(!restored.writable_memory_allowed);
    }

    #[test]
    fn edited_or_secret_bearing_checkpoint_is_rejected() {
        assert!(matches!(
            AgentCheckpoint::create(
                binding(),
                AgentLifecycleStatus::Alive,
                1,
                BTreeMap::new(),
                vec!["secret:raw-token".into()],
                None,
            ),
            Err(CheckpointError::InvalidReference)
        ));
        let mut checkpoint = AgentCheckpoint::create(
            binding(),
            AgentLifecycleStatus::Alive,
            1,
            BTreeMap::new(),
            Vec::new(),
            None,
        )
        .unwrap();
        checkpoint.state_revision = 2;
        assert!(matches!(
            checkpoint.restore(),
            Err(CheckpointError::InvalidDigest)
        ));
    }
}
