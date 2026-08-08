use crate::daemon_multiplex::{DaemonRegistryError, DaemonRegistryProjection, ProjectRouteKey};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

use crate::license::{
    evaluate_entitlement_execution,
    EntitlementExecutionContext,
    EntitlementExecutionPolicy,
};

const ENTITLEMENT_ROUTE_UNCLASSIFIED: &str = "ENTITLEMENT_ROUTE_UNCLASSIFIED";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WriterLease {
    pub lease_id: String,
    pub route: ProjectRouteKey,
    pub daemon_id: String,
    pub generation: u64,
    pub expires_at_unix_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteLeaseGeneration {
    pub route: ProjectRouteKey,
    pub generation: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WriterLeaseRegistry {
    pub active: Vec<WriterLease>,
    pub generations: Vec<RouteLeaseGeneration>,
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
    #[error("an unexpired writer lease already owns the exact route")]
    WriterLeaseBusy,
    #[error("mutation id was replayed with different content")]
    IdempotencyConflict,
    #[error("mutation outcome cannot transition from its current state")]
    InvalidTransition,
    #[error("effect receipt is required for acknowledgement")]
    MissingEffectReceipt,
    #[error("mutation is unknown")]
    UnknownMutation,
    #[error("operation entitlement denied ({code}): {message}")]
    EntitlementDenied {
        code: String,
        message: String,
        required_feature: Option<String>,
        limit_bucket: Option<String>,
    },
}

fn entitlement_policy_for_daemon_mutation(operation: &str) -> Result<EntitlementExecutionPolicy, DispatchError> {
    let operation = operation.trim();
    let (family, required_feature, limit_bucket) = match operation {
        "focusa.workpoint.checkpoint" | "focusa.workpoint.resume" | "focusa.trajectory.propose_workpoint" | "focusa.trajectory.checkpoint" | "focusa.trajectory.resume" => {
            (
                focusa_license::CapabilityFamily::BaseFocusa,
                None,
                None,
            )
        }
        "focusa.silent_session.writer_admission" | "focusa.silent_session.dispatch" => (
            focusa_license::CapabilityFamily::Automation,
            Some("focusa.agent.silent_sessions"),
            Some("silent_session_runs"),
        ),
        _ => {
            return Err(DispatchError::EntitlementDenied {
                code: ENTITLEMENT_ROUTE_UNCLASSIFIED.to_string(),
                message: "daemon dispatch operation is not mapped to a canonical entitlement policy".into(),
                required_feature: None,
                limit_bucket: None,
            });
        }
    };

    Ok(EntitlementExecutionPolicy::new(
        operation,
        focusa_license::OperationClass::ValueMutation,
        family,
        required_feature,
        limit_bucket,
        focusa_license::RecoveryAllowance::None,
    ))
}

fn evaluate_daemon_mutation_entitlement(
    entitlement_guard: &focusa_license::LicenseGuard,
    mutation: &MutationEnvelope,
) -> Result<(), DispatchError> {
    let policy = entitlement_policy_for_daemon_mutation(&mutation.operation)?;
    evaluate_entitlement_execution(
        entitlement_guard,
        &policy,
        EntitlementExecutionContext::default(),
    )
    .map(|_| ())
    .map_err(|error| DispatchError::EntitlementDenied {
        code: error.code,
        message: error.message,
        required_feature: error.required_feature,
        limit_bucket: error.limit_bucket,
    })
}

impl WriterLeaseRegistry {
    pub fn acquire(
        &mut self,
        registry: &DaemonRegistryProjection,
        route: ProjectRouteKey,
        daemon_id: &str,
        lease_id: &str,
        observed_at_unix_ms: i64,
        expires_at_unix_ms: i64,
    ) -> Result<&WriterLease, DispatchError> {
        if lease_id.trim().is_empty() {
            return Err(DispatchError::MissingIdentity("lease_id"));
        }
        let daemon = registry.resolve(&route)?;
        if daemon.daemon_id != daemon_id {
            return Err(DispatchError::LeaseScopeMismatch);
        }
        if expires_at_unix_ms <= observed_at_unix_ms {
            return Err(DispatchError::LeaseExpired);
        }
        if self
            .active
            .iter()
            .any(|lease| lease.route == route && lease.expires_at_unix_ms > observed_at_unix_ms)
        {
            return Err(DispatchError::WriterLeaseBusy);
        }
        self.active.retain(|lease| lease.route != route);
        let generation = if let Some(current) = self
            .generations
            .iter_mut()
            .find(|current| current.route == route)
        {
            current.generation += 1;
            current.generation
        } else {
            self.generations.push(RouteLeaseGeneration {
                route: route.clone(),
                generation: 1,
            });
            1
        };
        self.active.push(WriterLease {
            lease_id: lease_id.into(),
            route,
            daemon_id: daemon_id.into(),
            generation,
            expires_at_unix_ms,
        });
        Ok(self.active.last().expect("writer lease was inserted"))
    }

    pub fn require_active(
        &self,
        route: &ProjectRouteKey,
        lease_id: &str,
        generation: u64,
        observed_at_unix_ms: i64,
    ) -> Result<&WriterLease, DispatchError> {
        let lease = self
            .active
            .iter()
            .find(|lease| lease.route == *route && lease.lease_id == lease_id)
            .ok_or(DispatchError::LeaseScopeMismatch)?;
        if lease.generation != generation {
            return Err(DispatchError::LeaseGenerationMismatch);
        }
        if lease.expires_at_unix_ms <= observed_at_unix_ms {
            return Err(DispatchError::LeaseExpired);
        }
        Ok(lease)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MutationDispatchLedger {
    receipts: BTreeMap<String, DispatchReceipt>,
}

impl MutationDispatchLedger {
    pub fn prepare(
        &mut self,
        registry: &DaemonRegistryProjection,
        leases: &WriterLeaseRegistry,
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
                let lease = leases.require_active(
                    &mutation.route,
                    &mutation.writer_lease_id,
                    mutation.writer_lease_generation,
                    observed_at_unix_ms,
                )?;
                if lease.route != mutation.route
                    || lease.daemon_id != daemon.daemon_id
                    || lease.lease_id != mutation.writer_lease_id
                {
                    return Err(DispatchError::LeaseScopeMismatch);
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

    /// Evaluate a canonical entitlement policy before persisting the mutation envelope.
    ///
    /// Existing callers continue to use [`prepare`] for backwards compatibility.
    pub fn prepare_with_entitlement(
        &mut self,
        registry: &DaemonRegistryProjection,
        leases: &WriterLeaseRegistry,
        mutation: &MutationEnvelope,
        observed_at_unix_ms: i64,
        entitlement_guard: &focusa_license::LicenseGuard,
    ) -> Result<&DispatchReceipt, DispatchError> {
        evaluate_daemon_mutation_entitlement(entitlement_guard, mutation)?;
        self.prepare(registry, leases, mutation, observed_at_unix_ms)
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

    pub fn recovery_queue(&self) -> Vec<&DispatchReceipt> {
        self.receipts
            .values()
            .filter(|receipt| {
                matches!(
                    receipt.status,
                    DispatchStatus::Prepared | DispatchStatus::Uncertain
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use crate::daemon_multiplex::{
        DaemonHealth, DaemonRegistration, DaemonRegistryEvent, reduce_daemon_registry,
    };
    use std::collections::BTreeSet;
    use focusa_license::authority::{EntitlementSnapshot, EntitlementState};

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
                    allowed_native_sessions: BTreeSet::from(["session-1".into()]),
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

    fn leases() -> WriterLeaseRegistry {
        let mut leases = WriterLeaseRegistry::default();
        leases
            .acquire(&registry(), route(), "daemon-1", "lease-1", 0, 10_000)
            .unwrap();
        leases
    }

    fn mutation(digest: &str) -> MutationEnvelope {
        MutationEnvelope {
            mutation_id: "mutation-1".into(),
            route: route(),
            writer_lease_id: "lease-1".into(),
            writer_lease_generation: 1,
            payload_digest: digest.into(),
            operation: "workpoint.checkpoint".into(),
        }
    }

    fn signed_base_snapshot() -> focusa_license::LicenseGuard {
        let now = Utc::now();
        let mut snapshot = EntitlementSnapshot::unactivated("focusa", "node-core-daemon");
        snapshot.state = EntitlementState::Active;
        snapshot.sequence = Some(7);
        snapshot.lease_id = Some("lease-1".into());
        snapshot.lease_digest = Some("sha256:daemon".into());
        snapshot.expires_at = Some(now + chrono::Duration::hours(1));
        snapshot.offline_grace_until = Some(now + chrono::Duration::hours(1));
        focusa_license::LicenseGuard::from_entitlement(snapshot)
    }

    #[test]
    fn daemon_dispatch_rejects_unknown_mutation_operations_before_ledger_side_effect() {
        let mut ledger = MutationDispatchLedger::default();
        let mut mutation = mutation("sha256:unknown");
        mutation.operation = "focusa.internal.secret".into();
        assert!(matches!(
            ledger.prepare_with_entitlement(&registry(), &leases(), &mutation, 1, &focusa_license::LicenseGuard::eval(7)),
            Err(DispatchError::EntitlementDenied { code, .. }) if code == "ENTITLEMENT_ROUTE_UNCLASSIFIED"
        ));
    }

    #[test]
    fn daemon_dispatch_with_entitlement_uses_base_focusa_policy_for_known_mutations() {
        let mut ledger = MutationDispatchLedger::default();
        let guard = signed_base_snapshot();
        let receipt = ledger
            .prepare_with_entitlement(&registry(), &leases(), &mutation("sha256:a"), 1, &guard)
            .expect("known workpoint mutation should pass base policy");
        assert_eq!(receipt.status, DispatchStatus::Prepared);
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
            .prepare(&registry(), &leases(), &mutation("sha256:a"), 1)
            .unwrap();
        assert_eq!(receipt.route, route());
        assert_eq!(receipt.daemon_id, "daemon-1");
        let persisted = serde_json::to_vec(&ledger).unwrap();
        let restarted: MutationDispatchLedger = serde_json::from_slice(&persisted).unwrap();
        assert_eq!(restarted.receipt("mutation-1").unwrap().route, route());
    }

    #[test]
    fn writer_lease_is_exclusive_replayable_and_generation_fenced() {
        let mut authority = WriterLeaseRegistry::default();
        authority
            .acquire(&registry(), route(), "daemon-1", "lease-1", 0, 10)
            .unwrap();
        assert_eq!(
            authority.acquire(&registry(), route(), "daemon-1", "lease-2", 1, 20),
            Err(DispatchError::WriterLeaseBusy)
        );
        authority
            .acquire(&registry(), route(), "daemon-1", "lease-2", 10, 20)
            .unwrap();
        assert_eq!(authority.active[0].generation, 2);
        assert_eq!(
            authority.require_active(&route(), "lease-2", 1, 11),
            Err(DispatchError::LeaseGenerationMismatch)
        );
        let persisted = serde_json::to_vec(&authority).unwrap();
        let restarted: WriterLeaseRegistry = serde_json::from_slice(&persisted).unwrap();
        assert_eq!(restarted, authority);
    }

    #[test]
    fn duplicate_and_reordered_delivery_have_one_effect_identity() {
        let mut ledger = MutationDispatchLedger::default();
        assert_eq!(
            ledger.settle_acknowledged("mutation-1", "effect:receipt-1"),
            Err(DispatchError::UnknownMutation)
        );
        let first = ledger
            .prepare(&registry(), &leases(), &mutation("sha256:a"), 1)
            .unwrap();
        assert_eq!(first.status, DispatchStatus::Prepared);
        let duplicate = ledger
            .prepare(&registry(), &leases(), &mutation("sha256:a"), 2)
            .unwrap();
        assert_eq!(duplicate.mutation_id, "mutation-1");
        assert_eq!(
            ledger.prepare(&registry(), &leases(), &mutation("sha256:b"), 3),
            Err(DispatchError::IdempotencyConflict)
        );
        ledger
            .settle_acknowledged("mutation-1", "effect:receipt-1")
            .unwrap();
        let duplicate_receipt = ledger
            .settle_acknowledged("mutation-1", "effect:receipt-1")
            .unwrap();
        assert_eq!(
            duplicate_receipt.effect_receipt_ref.as_deref(),
            Some("effect:receipt-1")
        );
    }

    #[test]
    fn expired_writer_fails_over_to_the_new_exact_route_owner() {
        let failover_registry = reduce_daemon_registry([
            DaemonRegistryEvent::Enrolled {
                registration: DaemonRegistration {
                    daemon_id: "daemon-old".into(),
                    controller_id: "controller-1".into(),
                    endpoint: "https://old.example.test".into(),
                    auth_fingerprint: "sha256:old".into(),
                    version: "0.9.143".into(),
                    capabilities: BTreeSet::from(["workpoint".into()]),
                    allowed_native_sessions: BTreeSet::from(["session-1".into()]),
                    health: DaemonHealth::Offline,
                    generation: 1,
                },
            },
            DaemonRegistryEvent::ScopeAssigned {
                daemon_id: "daemon-old".into(),
                generation: 1,
                route: route(),
            },
            DaemonRegistryEvent::Enrolled {
                registration: DaemonRegistration {
                    daemon_id: "daemon-new".into(),
                    controller_id: "controller-1".into(),
                    endpoint: "https://new.example.test".into(),
                    auth_fingerprint: "sha256:new".into(),
                    version: "0.9.143".into(),
                    capabilities: BTreeSet::from(["workpoint".into()]),
                    allowed_native_sessions: BTreeSet::from(["session-1".into()]),
                    health: DaemonHealth::Healthy,
                    generation: 1,
                },
            },
            DaemonRegistryEvent::ScopeAssigned {
                daemon_id: "daemon-new".into(),
                generation: 1,
                route: route(),
            },
        ]);
        let mut authority = WriterLeaseRegistry {
            active: vec![WriterLease {
                lease_id: "lease-old".into(),
                route: route(),
                daemon_id: "daemon-old".into(),
                generation: 1,
                expires_at_unix_ms: 10,
            }],
            generations: vec![RouteLeaseGeneration {
                route: route(),
                generation: 1,
            }],
        };
        let failover = authority
            .acquire(
                &failover_registry,
                route(),
                "daemon-new",
                "lease-new",
                10,
                20,
            )
            .unwrap();
        assert_eq!(failover.daemon_id, "daemon-new");
        assert_eq!(failover.generation, 2);
    }

    #[test]
    fn restart_preserves_acknowledged_and_blocks_uncertain_until_reconciled() {
        let mut before_restart = MutationDispatchLedger::default();
        before_restart
            .prepare(&registry(), &leases(), &mutation("sha256:a"), 1)
            .unwrap();
        before_restart
            .settle_uncertain("mutation-1", "daemon_disconnected")
            .unwrap();
        let bytes = serde_json::to_vec(&before_restart).unwrap();
        let mut recovered: MutationDispatchLedger = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(recovered.recovery_queue().len(), 1);
        assert_eq!(
            recovered
                .prepare(&registry(), &leases(), &mutation("sha256:a"), 2)
                .unwrap()
                .status,
            DispatchStatus::Uncertain
        );
        assert_eq!(
            recovered.settle_acknowledged("mutation-1", ""),
            Err(DispatchError::MissingEffectReceipt)
        );
        recovered
            .settle_acknowledged("mutation-1", "effect:durable-1")
            .unwrap();
        assert!(recovered.recovery_queue().is_empty());
        let after_second_restart: MutationDispatchLedger =
            serde_json::from_slice(&serde_json::to_vec(&recovered).unwrap()).unwrap();
        let receipt = after_second_restart.receipt("mutation-1").unwrap();
        assert_eq!(receipt.status, DispatchStatus::Acknowledged);
        assert_eq!(
            receipt.effect_receipt_ref.as_deref(),
            Some("effect:durable-1")
        );
    }

    #[test]
    fn uncertain_outcome_reconciles_only_with_effect_receipt() {
        let mut ledger = MutationDispatchLedger::default();
        ledger
            .prepare(&registry(), &leases(), &mutation("sha256:a"), 1)
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
