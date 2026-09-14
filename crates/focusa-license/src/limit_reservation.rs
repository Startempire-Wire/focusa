//! Spec 152F.04.07 — premium limit reservation service and declared limit registry.
//!
//! Chokepoint 4 of Spec 152F §6: "Limit reservation service: atomically reserves
//! limited operations before execution and settles them afterward."
//!
//! The service re-resolves the canonical premium family decision on every
//! reservation, reads capacity only from authority-owned lease limits, binds each
//! reservation to the lease identity (sequence + digest) and the issuing
//! account/node scope, and enforces idempotency so replay and duplicate requests
//! can neither widen a grant nor double-reserve capacity. No family or limit can
//! be widened by client metadata, presentation layer, replay, race, stale state,
//! or another product/account: capacity is never caller-supplied, and the only
//! feature/bucket identifiers that resolve are the registered ones.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::authority::EntitlementSnapshot;
use crate::{
    CapabilityFamily, PremiumFamilyDecision, PremiumFamilyDenial, resolve_export_packaged,
    resolve_premium_family,
};

/// Declared server-owned limit buckets for the automation family.
pub const AUTOMATION_LIMIT_BUCKETS: [&str; 2] = ["concurrent_agents", "scheduled_runs"];
/// Declared server-owned limit buckets for the team_remote family (node/seat
/// reservation plus remote-stream usage).
pub const TEAM_REMOTE_LIMIT_BUCKETS: [&str; 3] =
    ["nodes", "team_operators", "remote_stream_minutes"];
/// Declared server-owned limit buckets for the release_proof family.
pub const RELEASE_PROOF_LIMIT_BUCKETS: [&str; 1] = ["governed_proof_packets"];
/// Declared server-owned limit buckets for the premium_updates family.
pub const PREMIUM_UPDATES_LIMIT_BUCKETS: [&str; 1] = ["managed_rollout_targets"];
/// Declared server-owned limit buckets for the export premium packaging feature.
pub const CUSTOMER_DATA_EXPORT_LIMIT_BUCKETS: [&str; 1] = ["packaged_export_bundles"];

/// The frozen union of server-owned limit buckets (Spec 152F §9 "Limit bucket"
/// dimension: active only for declared server-owned limits). A reservation may
/// only name a bucket declared here; caller-invented buckets never become
/// capacity and never alter authorization.
pub const DECLARED_SERVER_OWNED_LIMIT_BUCKETS: [&str; 8] = [
    "concurrent_agents",
    "scheduled_runs",
    "nodes",
    "team_operators",
    "remote_stream_minutes",
    "governed_proof_packets",
    "managed_rollout_targets",
    "packaged_export_bundles",
];

/// Return the declared server-owned limit buckets for one optional family.
/// Non-premium families return an empty set and cannot reserve capacity.
pub const fn family_limit_buckets(family: CapabilityFamily) -> &'static [&'static str] {
    match family {
        CapabilityFamily::Automation => &AUTOMATION_LIMIT_BUCKETS,
        CapabilityFamily::TeamRemote => &TEAM_REMOTE_LIMIT_BUCKETS,
        CapabilityFamily::ReleaseProof => &RELEASE_PROOF_LIMIT_BUCKETS,
        CapabilityFamily::PremiumUpdates => &PREMIUM_UPDATES_LIMIT_BUCKETS,
        CapabilityFamily::CustomerDataExport => &CUSTOMER_DATA_EXPORT_LIMIT_BUCKETS,
        _ => &[],
    }
}

/// The frozen list of declared server-owned limit buckets.
pub const fn declared_server_owned_limit_buckets() -> &'static [&'static str] {
    &DECLARED_SERVER_OWNED_LIMIT_BUCKETS
}

/// The account/node/product scope a reservation is bound to. Two snapshots from
/// different products, nodes, or accounts never share reservation capacity and
/// never settle each other's reservations.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReservationScope {
    pub product: String,
    pub node_id: String,
    pub subject_id: Option<String>,
}

impl ReservationScope {
    pub fn from_snapshot(snapshot: &EntitlementSnapshot) -> Self {
        Self {
            product: snapshot.product.clone(),
            node_id: snapshot.node_id.clone(),
            subject_id: snapshot.subject_id.clone(),
        }
    }
}

/// One granted, lease-bound reservation for a limited premium operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReservationGrant {
    pub idempotency_key: String,
    pub family: CapabilityFamily,
    pub feature: String,
    pub bucket: String,
    pub reserved_units: u64,
    pub scope: ReservationScope,
    pub lease_sequence: u64,
    pub lease_digest: String,
    pub offline_cached: bool,
}

/// Fail-closed reasons a reservation cannot be granted, revalidated, or settled.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ReservationError {
    /// The canonical premium family decision denied the operation. The inner
    /// denial is the authority-owned reason (base product, missing feature,
    /// stale/expired lease, unregistered feature, etc.).
    #[error("premium family decision denied the reservation: {0:?}")]
    FamilyDenied(PremiumFamilyDenial),
    /// The requested limit bucket is not declared for this family (Spec 152F §9:
    /// limits are active only for declared server-owned buckets).
    #[error("limit bucket {bucket} is not declared for family {family:?}")]
    UnknownLimitBucket {
        bucket: String,
        family: CapabilityFamily,
    },
    /// The authority snapshot carries no capacity for this declared bucket.
    #[error("authority snapshot has no capacity for limit bucket {bucket}")]
    LimitNotGranted { bucket: String },
    /// The requested units exceed the authority-owned capacity, counting
    /// outstanding reservations in the same scope.
    #[error(
        "limit bucket {bucket} exhausted: capacity {capacity}, reserved {reserved}, requested {requested}"
    )]
    LimitExhausted {
        bucket: String,
        capacity: u64,
        requested: u64,
        reserved: u64,
    },
    /// A malformed reservation request (zero units, etc.).
    #[error("invalid reservation request: {reason}")]
    InvalidRequest { reason: String },
    /// The same idempotency key was replayed with a different request payload,
    /// bucket, or scope. Replay must return the same grant, never a new one.
    #[error("idempotency key {idempotency_key} replayed with a different request: {reason}")]
    IdempotencyConflict {
        idempotency_key: String,
        reason: String,
    },
    /// The lease bound to an outstanding reservation no longer matches the
    /// current authority snapshot (sequence/digest/scope changed, refunded,
    /// revoked, expired, or offline-grace closed). Stale reservations cannot
    /// revalidate or settle.
    #[error("reservation {idempotency_key} is bound to a stale lease: {reason}")]
    StaleLease {
        idempotency_key: String,
        reason: String,
    },
    /// No outstanding reservation exists for this idempotency key (unknown or
    /// already settled).
    #[error("no outstanding reservation for idempotency key {idempotency_key}")]
    UnknownReservation { idempotency_key: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReservationRecord {
    grant: ReservationGrant,
}

/// Atomic limit reservation service (Spec 152F §6 chokepoint 4).
///
/// Every reservation re-resolves the canonical premium family decision and
/// checks capacity against authority-owned lease limits at reserve time, so a
/// race or replay can never double-allocate or widen a grant.
#[derive(Debug, Default)]
pub struct LimitReservationService {
    records: BTreeMap<String, ReservationRecord>,
    settled: BTreeMap<String, u64>,
}

impl LimitReservationService {
    pub fn new() -> Self {
        Self::default()
    }

    /// Atomically reserve `requested_units` from the authority-owned limit
    /// bucket `bucket` for one premium operation.
    ///
    /// `feature` is operation metadata (the registered family feature), never a
    /// grant request; `idempotency_key` is the client-supplied request
    /// identifier used for replay detection; `now` is explicit so Offline Grace
    /// cannot be extended by a caller. Capacity comes only from
    /// `snapshot.limits`, which is authority-owned.
    ///
    /// These parameters are the frozen atomic reservation contract.
    #[allow(clippy::too_many_arguments)]
    pub fn reserve(
        &mut self,
        snapshot: &EntitlementSnapshot,
        family: CapabilityFamily,
        feature: &str,
        bucket: &str,
        idempotency_key: &str,
        requested_units: u64,
        now: DateTime<Utc>,
    ) -> Result<ReservationGrant, ReservationError> {
        // 1. Fail closed: re-resolve the canonical premium family decision.
        let (required_feature, lease_sequence, offline_cached) =
            match self.resolve_decision(snapshot, family, feature, now) {
                PremiumFamilyDecision::Feature {
                    required_feature,
                    lease_sequence,
                    offline_cached,
                    ..
                } => (required_feature, lease_sequence, offline_cached),
                PremiumFamilyDecision::Denied(denial) => {
                    return Err(ReservationError::FamilyDenied(denial));
                }
            };

        // 2. The bucket must be declared for this family (server-owned registry).
        if !family_limit_buckets(family).contains(&bucket) {
            return Err(ReservationError::UnknownLimitBucket {
                bucket: bucket.to_string(),
                family,
            });
        }

        // 3. Capacity comes only from the authority-owned snapshot limits.
        let Some(capacity) = snapshot.limits.get(bucket).copied() else {
            return Err(ReservationError::LimitNotGranted {
                bucket: bucket.to_string(),
            });
        };
        if requested_units == 0 {
            return Err(ReservationError::InvalidRequest {
                reason: "zero_units".to_string(),
            });
        }

        // 4. Idempotency: a replayed request must return the same grant, never
        //    reserve twice. A replayed key with a different payload, bucket, or
        //    scope is a conflict; a replayed key against a different lease is stale.
        let scope = ReservationScope::from_snapshot(snapshot);
        if let Some(existing) = self.records.get(idempotency_key) {
            let grant = &existing.grant;
            if grant.scope == scope
                && grant.bucket == bucket
                && grant.reserved_units == requested_units
                && grant.lease_sequence == lease_sequence
                && grant.lease_digest == snapshot.lease_digest.clone().unwrap_or_default()
            {
                return Ok(grant.clone());
            }
            if grant.scope != scope || grant.bucket != bucket {
                return Err(ReservationError::IdempotencyConflict {
                    idempotency_key: idempotency_key.to_string(),
                    reason: "different_scope_or_bucket".to_string(),
                });
            }
            if grant.lease_sequence != lease_sequence
                || grant.lease_digest != snapshot.lease_digest.clone().unwrap_or_default()
            {
                return Err(ReservationError::StaleLease {
                    idempotency_key: idempotency_key.to_string(),
                    reason: "lease_identity_changed".to_string(),
                });
            }
            return Err(ReservationError::IdempotencyConflict {
                idempotency_key: idempotency_key.to_string(),
                reason: "different_units_or_feature".to_string(),
            });
        }

        // 5. Exhaustion against outstanding reservations in the SAME scope.
        let reserved: u64 = self
            .records
            .values()
            .filter(|record| record.grant.scope == scope && record.grant.bucket == bucket)
            .map(|record| record.grant.reserved_units)
            .sum();
        if reserved + requested_units > capacity {
            return Err(ReservationError::LimitExhausted {
                bucket: bucket.to_string(),
                capacity,
                requested: requested_units,
                reserved,
            });
        }

        // 6. Bind the reservation to the lease identity and scope.
        let grant = ReservationGrant {
            idempotency_key: idempotency_key.to_string(),
            family,
            feature: required_feature.as_str().to_string(),
            bucket: bucket.to_string(),
            reserved_units: requested_units,
            scope,
            lease_sequence,
            lease_digest: snapshot.lease_digest.clone().unwrap_or_default(),
            offline_cached,
        };
        self.records.insert(
            idempotency_key.to_string(),
            ReservationRecord {
                grant: grant.clone(),
            },
        );
        Ok(grant)
    }

    /// Dispatch-time revalidation (Spec 152F §7 Worker/scheduler row): the
    /// outstanding reservation is re-checked against the CURRENT authority
    /// snapshot before a delayed side effect. Refund/revoke, expiry, offline-
    /// grace closure, a higher authority sequence, a changed lease digest, or a
    /// different scope all fail closed.
    pub fn revalidate(
        &self,
        snapshot: &EntitlementSnapshot,
        idempotency_key: &str,
        now: DateTime<Utc>,
    ) -> Result<&ReservationGrant, ReservationError> {
        let record = self.records.get(idempotency_key).ok_or_else(|| {
            ReservationError::UnknownReservation {
                idempotency_key: idempotency_key.to_string(),
            }
        })?;
        let grant = &record.grant;
        if grant.scope != ReservationScope::from_snapshot(snapshot) {
            return Err(ReservationError::StaleLease {
                idempotency_key: idempotency_key.to_string(),
                reason: "scope_changed".to_string(),
            });
        }
        if grant.lease_sequence != snapshot.sequence.unwrap_or(0)
            || grant.lease_digest != snapshot.lease_digest.clone().unwrap_or_default()
        {
            return Err(ReservationError::StaleLease {
                idempotency_key: idempotency_key.to_string(),
                reason: "lease_identity_changed".to_string(),
            });
        }
        // Re-resolve the canonical decision against the current snapshot: a
        // refund/revoke or expired/closed grant stops the delayed side effect.
        match self.resolve_decision(snapshot, grant.family, &grant.feature, now) {
            PremiumFamilyDecision::Feature { .. } => Ok(grant),
            PremiumFamilyDecision::Denied(denial) => Err(ReservationError::FamilyDenied(denial)),
        }
    }

    /// Settle an outstanding reservation, releasing its reserved capacity after
    /// the operation completes. Settlement requires the same lease identity and
    /// scope; a stale or foreign caller cannot settle another grant.
    pub fn settle(
        &mut self,
        snapshot: &EntitlementSnapshot,
        idempotency_key: &str,
        used_units: u64,
    ) -> Result<(), ReservationError> {
        // Validate against the outstanding record first so a failed settle never
        // releases capacity: only a matching lease identity and scope settle.
        {
            let record = self.records.get(idempotency_key).ok_or_else(|| {
                ReservationError::UnknownReservation {
                    idempotency_key: idempotency_key.to_string(),
                }
            })?;
            let grant = &record.grant;
            if grant.scope != ReservationScope::from_snapshot(snapshot) {
                return Err(ReservationError::StaleLease {
                    idempotency_key: idempotency_key.to_string(),
                    reason: "scope_changed".to_string(),
                });
            }
            if grant.lease_sequence != snapshot.sequence.unwrap_or(0)
                || grant.lease_digest != snapshot.lease_digest.clone().unwrap_or_default()
            {
                return Err(ReservationError::StaleLease {
                    idempotency_key: idempotency_key.to_string(),
                    reason: "lease_identity_changed".to_string(),
                });
            }
            if used_units > grant.reserved_units {
                return Err(ReservationError::InvalidRequest {
                    reason: "used_units_exceed_reserved".to_string(),
                });
            }
        }
        self.records.remove(idempotency_key);
        self.settled.insert(idempotency_key.to_string(), used_units);
        Ok(())
    }

    /// Outstanding reserved units for one bucket within one scope.
    pub fn reserved_units(&self, bucket: &str, scope: &ReservationScope) -> u64 {
        self.records
            .values()
            .filter(|record| record.grant.bucket == bucket && record.grant.scope == *scope)
            .map(|record| record.grant.reserved_units)
            .sum()
    }

    /// Number of outstanding reservations (all scopes).
    pub fn outstanding_count(&self) -> usize {
        self.records.len()
    }

    /// Recorded used units for a settled reservation.
    pub fn settled_units(&self, idempotency_key: &str) -> Option<u64> {
        self.settled.get(idempotency_key).copied()
    }

    fn resolve_decision(
        &self,
        snapshot: &EntitlementSnapshot,
        family: CapabilityFamily,
        feature: &str,
        now: DateTime<Utc>,
    ) -> PremiumFamilyDecision {
        if family == CapabilityFamily::CustomerDataExport {
            resolve_export_packaged(snapshot, feature, now)
        } else {
            resolve_premium_family(snapshot, family, feature, now)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authority::EntitlementState;
    use chrono::Duration;

    fn active_snapshot(product: &str, node: &str, subject: &str) -> EntitlementSnapshot {
        let now = Utc::now();
        let mut snapshot = EntitlementSnapshot::unactivated(product, node);
        snapshot.state = EntitlementState::Active;
        snapshot.subject_id = Some(subject.to_string());
        snapshot.lease_id = Some(format!("lease-{subject}"));
        snapshot.sequence = Some(9);
        snapshot.lease_digest = Some("sha256:lease".to_string());
        snapshot.expires_at = Some(now + Duration::hours(1));
        snapshot.offline_grace_until = Some(now + Duration::hours(1));
        snapshot
    }

    fn automation_snapshot() -> EntitlementSnapshot {
        let mut snapshot = active_snapshot("focusa", "node-auto", "account-alpha");
        snapshot
            .features
            .insert("focusa.agent.silent_sessions".to_string(), true);
        snapshot.limits.insert("concurrent_agents".to_string(), 2);
        snapshot
    }

    #[test]
    fn reservation_exhausts_authority_capacity_and_settles() {
        let mut service = LimitReservationService::new();
        let snapshot = automation_snapshot();

        let first = service
            .reserve(
                &snapshot,
                CapabilityFamily::Automation,
                "focusa.agent.silent_sessions",
                "concurrent_agents",
                "req-1",
                2,
                Utc::now(),
            )
            .expect("capacity 2 available");
        assert_eq!(first.reserved_units, 2);

        // A concurrent arrival sees the outstanding reservation and fails.
        let denial = service
            .reserve(
                &snapshot,
                CapabilityFamily::Automation,
                "focusa.agent.silent_sessions",
                "concurrent_agents",
                "req-2",
                1,
                Utc::now(),
            )
            .expect_err("capacity exhausted");
        assert!(matches!(denial, ReservationError::LimitExhausted { .. }));

        // Settle releases capacity for a later reservation.
        service
            .settle(&snapshot, "req-1", 1)
            .expect("settle within the same lease");
        assert_eq!(service.settled_units("req-1"), Some(1));
        let after = service
            .reserve(
                &snapshot,
                CapabilityFamily::Automation,
                "focusa.agent.silent_sessions",
                "concurrent_agents",
                "req-3",
                1,
                Utc::now(),
            )
            .expect("capacity released by settlement");
        assert_eq!(after.reserved_units, 1);
    }

    #[test]
    fn duplicate_request_replays_without_double_reserving() {
        let mut service = LimitReservationService::new();
        let snapshot = automation_snapshot();

        let first = service
            .reserve(
                &snapshot,
                CapabilityFamily::Automation,
                "focusa.agent.silent_sessions",
                "concurrent_agents",
                "dup-key",
                1,
                Utc::now(),
            )
            .unwrap();
        let replay = service
            .reserve(
                &snapshot,
                CapabilityFamily::Automation,
                "focusa.agent.silent_sessions",
                "concurrent_agents",
                "dup-key",
                1,
                Utc::now(),
            )
            .unwrap();
        assert_eq!(first, replay);
        assert_eq!(service.outstanding_count(), 1);

        // Same key with different units is a conflict, never a new grant.
        let conflict = service
            .reserve(
                &snapshot,
                CapabilityFamily::Automation,
                "focusa.agent.silent_sessions",
                "concurrent_agents",
                "dup-key",
                2,
                Utc::now(),
            )
            .expect_err("different units must conflict");
        assert!(matches!(
            conflict,
            ReservationError::IdempotencyConflict { .. }
        ));
    }

    #[test]
    fn stale_lease_cannot_revalidate_or_settle() {
        let mut service = LimitReservationService::new();
        let snapshot = automation_snapshot();
        service
            .reserve(
                &snapshot,
                CapabilityFamily::Automation,
                "focusa.agent.silent_sessions",
                "concurrent_agents",
                "stale-key",
                1,
                Utc::now(),
            )
            .unwrap();

        // A higher authority sequence with a new lease digest supersedes the grant.
        let mut superseded = snapshot.clone();
        superseded.sequence = Some(10);
        superseded.lease_digest = Some("sha256:superseded".to_string());

        let stale = service
            .revalidate(&superseded, "stale-key", Utc::now())
            .expect_err("stale lease must fail revalidation");
        assert!(matches!(
            stale,
            ReservationError::StaleLease { reason, .. } if reason == "lease_identity_changed"
        ));

        let stale = service
            .settle(&superseded, "stale-key", 1)
            .expect_err("stale lease must fail settlement");
        assert!(matches!(stale, ReservationError::StaleLease { .. }));
    }

    #[test]
    fn cross_account_scope_is_isolated() {
        let mut service = LimitReservationService::new();
        let account_a = automation_snapshot();
        let mut account_b = active_snapshot("focusa", "node-auto", "account-beta");
        account_b
            .features
            .insert("focusa.agent.silent_sessions".to_string(), true);
        account_b.limits.insert("concurrent_agents".to_string(), 2);

        service
            .reserve(
                &account_a,
                CapabilityFamily::Automation,
                "focusa.agent.silent_sessions",
                "concurrent_agents",
                "a-key",
                2,
                Utc::now(),
            )
            .unwrap();

        // Account B has its own authority capacity and cannot see A's reservation.
        let b_scope = ReservationScope::from_snapshot(&account_b);
        assert_eq!(service.reserved_units("concurrent_agents", &b_scope), 0);
        service
            .reserve(
                &account_b,
                CapabilityFamily::Automation,
                "focusa.agent.silent_sessions",
                "concurrent_agents",
                "b-key",
                2,
                Utc::now(),
            )
            .expect("account B has independent capacity");

        // Account B cannot settle account A's reservation, and the failed
        // settle leaves A's reservation outstanding (fail-closed).
        let denied = service
            .settle(&account_b, "a-key", 0)
            .expect_err("foreign account cannot settle");
        assert!(matches!(
            denied,
            ReservationError::StaleLease { reason, .. } if reason == "scope_changed"
        ));
        assert_eq!(service.outstanding_count(), 2);
    }
}
