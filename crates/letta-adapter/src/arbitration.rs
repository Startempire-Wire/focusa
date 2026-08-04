use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CognitiveOwner {
    Pi,
    Letta,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeAuthorityScope {
    pub project_root: String,
    pub continuity_id: String,
    pub workpoint_id: String,
    pub native_session_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeOwnershipLease {
    pub lease_id: String,
    pub scope: RuntimeAuthorityScope,
    pub owner: CognitiveOwner,
    pub adapter_instance_id: String,
    pub epoch_id: Uuid,
    pub generation: u64,
    pub expires_at_unix_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnershipAcquireRequest {
    pub lease_id: String,
    pub scope: RuntimeAuthorityScope,
    pub owner: CognitiveOwner,
    pub adapter_instance_id: String,
    pub epoch_id: Uuid,
    pub now_unix_ms: i64,
    pub expires_at_unix_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RuntimeOwnershipRegistry {
    pub leases: Vec<RuntimeOwnershipLease>,
    pub completed_event_keys: BTreeSet<String>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ArbitrationError {
    #[error("runtime ownership identity is incomplete")]
    IncompleteIdentity,
    #[error("exact Workpoint already has an active cognitive owner")]
    CompetingOwner,
    #[error("runtime ownership lease is absent, stale, or foreign")]
    ForeignLease,
    #[error("turn event has already been admitted")]
    DuplicateTurn,
    #[error("UIAI is a client-tool executor and cannot own the cognitive loop")]
    UiaiCannotOwnLoop,
}

impl RuntimeOwnershipRegistry {
    pub fn acquire(
        &mut self,
        request: OwnershipAcquireRequest,
    ) -> Result<RuntimeOwnershipLease, ArbitrationError> {
        validate_identity(
            &request.lease_id,
            &request.adapter_instance_id,
            &request.scope,
        )?;
        if request.expires_at_unix_ms <= request.now_unix_ms {
            return Err(ArbitrationError::ForeignLease);
        }
        if self.leases.iter().any(|lease| {
            lease.scope == request.scope && lease.expires_at_unix_ms > request.now_unix_ms
        }) {
            return Err(ArbitrationError::CompetingOwner);
        }
        let generation = self
            .leases
            .iter()
            .filter(|lease| lease.scope == request.scope)
            .map(|lease| lease.generation)
            .max()
            .unwrap_or(0)
            + 1;
        self.leases.retain(|lease| lease.scope != request.scope);
        let lease = RuntimeOwnershipLease {
            lease_id: request.lease_id,
            scope: request.scope,
            owner: request.owner,
            adapter_instance_id: request.adapter_instance_id,
            epoch_id: request.epoch_id,
            generation,
            expires_at_unix_ms: request.expires_at_unix_ms,
        };
        self.leases.push(lease.clone());
        Ok(lease)
    }

    pub fn handoff(
        &mut self,
        current: &RuntimeOwnershipLease,
        new_lease_id: &str,
        new_owner: CognitiveOwner,
        new_adapter_instance_id: &str,
        new_epoch_id: Uuid,
        expires_at_unix_ms: i64,
    ) -> Result<RuntimeOwnershipLease, ArbitrationError> {
        validate_identity(new_lease_id, new_adapter_instance_id, &current.scope)?;
        let stored = self
            .leases
            .iter_mut()
            .find(|stored| stored.scope == current.scope)
            .filter(|stored| *stored == current)
            .ok_or(ArbitrationError::ForeignLease)?;
        *stored = RuntimeOwnershipLease {
            lease_id: new_lease_id.into(),
            scope: current.scope.clone(),
            owner: new_owner,
            adapter_instance_id: new_adapter_instance_id.into(),
            epoch_id: new_epoch_id,
            generation: current.generation + 1,
            expires_at_unix_ms,
        };
        Ok(stored.clone())
    }

    pub fn authorize_turn(
        &mut self,
        lease: &RuntimeOwnershipLease,
        event_id: &str,
        now_unix_ms: i64,
    ) -> Result<String, ArbitrationError> {
        if event_id.trim().is_empty() {
            return Err(ArbitrationError::IncompleteIdentity);
        }
        let current = self
            .leases
            .iter()
            .find(|current| current.scope == lease.scope)
            .filter(|current| *current == lease && current.expires_at_unix_ms > now_unix_ms)
            .ok_or(ArbitrationError::ForeignLease)?;
        let event_key = format!(
            "{}:{}:{}:{}",
            current.scope.continuity_id, current.scope.workpoint_id, current.generation, event_id
        );
        if !self.completed_event_keys.insert(event_key.clone()) {
            return Err(ArbitrationError::DuplicateTurn);
        }
        Ok(event_key)
    }

    pub fn authorize_uiai_client_tool(
        &self,
        lease: &RuntimeOwnershipLease,
        parent_event_key: &str,
        now_unix_ms: i64,
    ) -> Result<(), ArbitrationError> {
        if !self.completed_event_keys.contains(parent_event_key) {
            return Err(ArbitrationError::ForeignLease);
        }
        self.leases
            .iter()
            .find(|current| *current == lease && current.expires_at_unix_ms > now_unix_ms)
            .map(|_| ())
            .ok_or(ArbitrationError::ForeignLease)
    }
}

fn validate_identity(
    lease_id: &str,
    adapter_instance_id: &str,
    scope: &RuntimeAuthorityScope,
) -> Result<(), ArbitrationError> {
    if [
        lease_id,
        adapter_instance_id,
        &scope.project_root,
        &scope.continuity_id,
        &scope.workpoint_id,
        &scope.native_session_id,
    ]
    .iter()
    .any(|value| value.trim().is_empty())
    {
        Err(ArbitrationError::IncompleteIdentity)
    } else {
        Ok(())
    }
}
