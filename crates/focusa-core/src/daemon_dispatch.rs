use crate::daemon_multiplex::{DaemonRegistryError, DaemonRegistryProjection, ProjectRouteKey};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WriterLease {
    pub lease_id: String,
    pub route: ProjectRouteKey,
    pub daemon_id: String,
    pub generation: u64,
    pub expires_at_unix_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MutationEnvelope {
    pub mutation_id: String,
    pub route: ProjectRouteKey,
    pub writer_lease_id: String,
    pub writer_lease_generation: u64,
    pub payload_digest: String,
    pub operation: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DispatchStatus {
    Prepared,
    Acknowledged,
    Uncertain,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DispatchReceipt {
    pub mutation_id: String,
    pub route: ProjectRouteKey,
    pub daemon_id: String,
    pub writer_lease_id: String,
    pub payload_digest: String,
    pub status: DispatchStatus,
    pub effect_receipt_ref: Option<String>,
    pub failure_class: Option<String>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DispatchError {
    #[error("mutation identity is incomplete: {0}")]
    MissingIdentity(&'static str),
    #[error("exact daemon route is unavailable: {0}")]
    Route(#[from] DaemonRegistryError),
    #[error("writer lease does not match route or daemon")]
    LeaseScopeMismatch,
    #[error("writer lease is expired")]
    LeaseExpired,
    #[error("writer lease generation is stale")]
    LeaseGenerationMismatch,
    #[error("mutation id was replayed with different content")]
    IdempotencyConflict,
    #[error("mutation outcome cannot transition from its current state")]
    InvalidTransition,
    #[error("effect receipt is required for acknowledgement")]
    MissingEffectReceipt,
    #[error("mutation is unknown")]
    UnknownMutation,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MutationDispatchLedger {
    receipts: BTreeMap<String, DispatchReceipt>,
}

impl MutationDispatchLedger {
    pub fn prepare(
        &mut self,
        registry: &DaemonRegistryProjection,
        lease: &WriterLease,
        mutation: &MutationEnvelope,
        observed_at_unix_ms: i64,
    ) -> Result<&DispatchReceipt, DispatchError> {
        for (value, field) in [
            (&mutation.mutation_id, "mutation_id"),
            (&mutation.payload_digest, "payload_digest"),
            (&mutation.operation, "operation"),
            (&mutation.writer_lease_id, "writer_lease_id"),
        ] {
            if value.trim().is_empty() {
                return Err(DispatchError::MissingIdentity(field));
            }
        }
        match self.receipts.entry(mutation.mutation_id.clone()) {
            std::collections::btree_map::Entry::Occupied(existing) => {
                if existing.get().payload_digest != mutation.payload_digest
                    || existing.get().writer_lease_id != mutation.writer_lease_id
                {
                    return Err(DispatchError::IdempotencyConflict);
                }
                Ok(existing.into_mut())
            }
            std::collections::btree_map::Entry::Vacant(vacant) => {
                let daemon = registry.resolve(&mutation.route)?;
                if lease.route != mutation.route
                    || lease.daemon_id != daemon.daemon_id
                    || lease.lease_id != mutation.writer_lease_id
                {
                    return Err(DispatchError::LeaseScopeMismatch);
                }
                if lease.generation != mutation.writer_lease_generation {
                    return Err(DispatchError::LeaseGenerationMismatch);
                }
                if lease.expires_at_unix_ms <= observed_at_unix_ms {
                    return Err(DispatchError::LeaseExpired);
                }
                Ok(vacant.insert(DispatchReceipt {
                    mutation_id: mutation.mutation_id.clone(),
                    route: mutation.route.clone(),
                    daemon_id: daemon.daemon_id.clone(),
                    writer_lease_id: lease.lease_id.clone(),
                    payload_digest: mutation.payload_digest.clone(),
                    status: DispatchStatus::Prepared,
                    effect_receipt_ref: None,
                    failure_class: None,
                }))
            }
        }
    }

    pub fn settle_acknowledged(
        &mut self,
        mutation_id: &str,
        effect_receipt_ref: &str,
    ) -> Result<&DispatchReceipt, DispatchError> {
        if effect_receipt_ref.trim().is_empty() {
            return Err(DispatchError::MissingEffectReceipt);
        }
        let receipt = self
            .receipts
            .get_mut(mutation_id)
            .ok_or(DispatchError::UnknownMutation)?;
        match receipt.status {
            DispatchStatus::Prepared | DispatchStatus::Uncertain => {
                receipt.status = DispatchStatus::Acknowledged;
                receipt.effect_receipt_ref = Some(effect_receipt_ref.into());
                receipt.failure_class = None;
                Ok(receipt)
            }
            DispatchStatus::Acknowledged
                if receipt.effect_receipt_ref.as_deref() == Some(effect_receipt_ref) =>
            {
                Ok(receipt)
            }
            _ => Err(DispatchError::InvalidTransition),
        }
    }

    pub fn settle_uncertain(
        &mut self,
        mutation_id: &str,
        failure_class: &str,
    ) -> Result<&DispatchReceipt, DispatchError> {
        let receipt = self
            .receipts
            .get_mut(mutation_id)
            .ok_or(DispatchError::UnknownMutation)?;
        if receipt.status != DispatchStatus::Prepared {
            return Err(DispatchError::InvalidTransition);
        }
        receipt.status = DispatchStatus::Uncertain;
        receipt.failure_class = Some(failure_class.into());
        Ok(receipt)
    }

    pub fn receipt(&self, mutation_id: &str) -> Option<&DispatchReceipt> {
        self.receipts.get(mutation_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::daemon_multiplex::{
        DaemonHealth, DaemonRegistration, DaemonRegistryEvent, reduce_daemon_registry,
    };
    use std::collections::BTreeSet;

    fn route() -> ProjectRouteKey {
        ProjectRouteKey {
            project_root: "/srv/focusa".into(),
            continuity_id: "continuity".into(),
            working_subpath_id: "working-subpath:main".into(),
        }
    }

    fn registry() -> DaemonRegistryProjection {
        registry_for("daemon-1", route())
    }

    fn registry_for(daemon_id: &str, route: ProjectRouteKey) -> DaemonRegistryProjection {
        reduce_daemon_registry([
            DaemonRegistryEvent::Enrolled {
                registration: DaemonRegistration {
                    daemon_id: daemon_id.into(),
                    controller_id: "controller-1".into(),
                    endpoint: format!("https://{daemon_id}.example.test"),
                    auth_fingerprint: format!("sha256:{daemon_id}"),
                    version: "0.9.143".into(),
                    capabilities: BTreeSet::from(["workpoint".into()]),
                    health: DaemonHealth::Healthy,
                    generation: 1,
                },
            },
            DaemonRegistryEvent::ScopeAssigned {
                daemon_id: daemon_id.into(),
                generation: 1,
                route,
            },
        ])
    }

    fn lease() -> WriterLease {
        WriterLease {
            lease_id: "lease-1".into(),
            route: route(),
            daemon_id: "daemon-1".into(),
            generation: 7,
            expires_at_unix_ms: 10_000,
        }
    }

    fn mutation(digest: &str) -> MutationEnvelope {
        MutationEnvelope {
            mutation_id: "mutation-1".into(),
            route: route(),
            writer_lease_id: "lease-1".into(),
            writer_lease_generation: 7,
            payload_digest: digest.into(),
            operation: "workpoint.checkpoint".into(),
        }
    }

    #[test]
    fn concurrent_project_reads_and_receipts_never_cross_route() {
        let route_a = route();
        let route_b = ProjectRouteKey {
            project_root: "/srv/other".into(),
            continuity_id: "continuity-other".into(),
            working_subpath_id: "working-subpath:feature".into(),
        };
        let registry_a = registry_for("daemon-a", route_a.clone());
        let registry_b = registry_for("daemon-b", route_b.clone());
        let handles = [
            std::thread::spawn(move || registry_a.resolve(&route_a).unwrap().daemon_id.clone()),
            std::thread::spawn(move || registry_b.resolve(&route_b).unwrap().daemon_id.clone()),
        ];
        let [first, second] = handles.map(|handle| handle.join().unwrap());
        assert_eq!((first.as_str(), second.as_str()), ("daemon-a", "daemon-b"));

        let mut ledger = MutationDispatchLedger::default();
        let receipt = ledger
            .prepare(&registry(), &lease(), &mutation("sha256:a"), 1)
            .unwrap();
        assert_eq!(receipt.route, route());
        assert_eq!(receipt.daemon_id, "daemon-1");
        let persisted = serde_json::to_vec(&ledger).unwrap();
        let restarted: MutationDispatchLedger = serde_json::from_slice(&persisted).unwrap();
        assert_eq!(restarted.receipt("mutation-1").unwrap().route, route());
    }

    #[test]
    fn duplicate_delivery_has_one_effect_identity() {
        let mut ledger = MutationDispatchLedger::default();
        let first = ledger
            .prepare(&registry(), &lease(), &mutation("sha256:a"), 1)
            .unwrap();
        assert_eq!(first.status, DispatchStatus::Prepared);
        let duplicate = ledger
            .prepare(&registry(), &lease(), &mutation("sha256:a"), 2)
            .unwrap();
        assert_eq!(duplicate.mutation_id, "mutation-1");
        assert_eq!(
            ledger.prepare(&registry(), &lease(), &mutation("sha256:b"), 3),
            Err(DispatchError::IdempotencyConflict)
        );
    }

    #[test]
    fn uncertain_outcome_reconciles_only_with_effect_receipt() {
        let mut ledger = MutationDispatchLedger::default();
        ledger
            .prepare(&registry(), &lease(), &mutation("sha256:a"), 1)
            .unwrap();
        ledger
            .settle_uncertain("mutation-1", "network_timeout")
            .unwrap();
        let settled = ledger
            .settle_acknowledged("mutation-1", "effect:receipt-1")
            .unwrap();
        assert_eq!(settled.status, DispatchStatus::Acknowledged);
        assert_eq!(
            settled.effect_receipt_ref.as_deref(),
            Some("effect:receipt-1")
        );
    }
}
