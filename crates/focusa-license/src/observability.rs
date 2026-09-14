//! Spec 152F.06.06 — bounded entitlement observability counters.
//!
//! Counters are fixed-size arrays keyed only by canonical family/reason/posture
//! labels. They never retain lease ids, digests, customer identifiers, paths,
//! keys, tokens, or feature claims, and they never perform I/O. The only escape
//! is `snapshot()`, which returns label-only counts safe for logs, metrics, and
//! the benchmark receipt.
//!
//! Memory is bounded by construction: three fixed arrays, one slot per canonical
//! enum variant. Caller-supplied strings can never grow the counter set, so a
//! high-cardinality key space cannot exhaust memory or leak into telemetry.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::{CapabilityFamily, DecisionReason, EntitlementPolicyPosture};

/// Number of canonical capability families (see `CapabilityFamily`).
const FAMILY_COUNT: usize = 9;
/// Number of canonical decision reasons (see `DecisionReason`).
const REASON_COUNT: usize = 13;
/// Number of canonical policy postures (see `EntitlementPolicyPosture`).
const POSTURE_COUNT: usize = 5;

/// Fixed-capacity, thread-safe counters for entitlement decisions.
///
/// Recording touches only enum discriminants — never strings — so the counter
/// set is exactly `FAMILY_COUNT + REASON_COUNT + POSTURE_COUNT` slots forever.
#[derive(Debug)]
pub struct EntitlementDecisionCounters {
    by_family: [AtomicU64; FAMILY_COUNT],
    by_reason: [AtomicU64; REASON_COUNT],
    by_posture: [AtomicU64; POSTURE_COUNT],
}

impl Default for EntitlementDecisionCounters {
    fn default() -> Self {
        Self::new()
    }
}

impl EntitlementDecisionCounters {
    /// All-zero counters.
    pub fn new() -> Self {
        Self {
            by_family: std::array::from_fn(|_| AtomicU64::new(0)),
            by_reason: std::array::from_fn(|_| AtomicU64::new(0)),
            by_posture: std::array::from_fn(|_| AtomicU64::new(0)),
        }
    }

    /// Record one entitlement decision against its canonical family, reason,
    /// and posture. Only enum discriminants are touched — never strings.
    pub fn record_decision(
        &self,
        family: CapabilityFamily,
        reason: DecisionReason,
        posture: EntitlementPolicyPosture,
    ) {
        self.by_family[family as usize].fetch_add(1, Ordering::Relaxed);
        self.by_reason[reason as usize].fetch_add(1, Ordering::Relaxed);
        self.by_posture[posture as usize].fetch_add(1, Ordering::Relaxed);
    }

    /// Total recorded decisions (any family/reason/posture).
    pub fn total(&self) -> u64 {
        self.by_family
            .iter()
            .map(|counter| counter.load(Ordering::Relaxed))
            .sum()
    }

    /// Recorded decisions for one canonical family.
    pub fn family_count(&self, family: CapabilityFamily) -> u64 {
        self.by_family[family as usize].load(Ordering::Relaxed)
    }

    /// Recorded decisions for one canonical reason.
    pub fn reason_count(&self, reason: DecisionReason) -> u64 {
        self.by_reason[reason as usize].load(Ordering::Relaxed)
    }

    /// Recorded decisions for one canonical posture.
    pub fn posture_count(&self, posture: EntitlementPolicyPosture) -> u64 {
        self.by_posture[posture as usize].load(Ordering::Relaxed)
    }

    /// Fixed capacity in slots; independent of any recorded workload.
    pub const fn capacity(&self) -> usize {
        FAMILY_COUNT + REASON_COUNT + POSTURE_COUNT
    }

    /// Label-only snapshot for logs and metrics. Contains canonical family,
    /// reason, and posture labels with counts; never raw authority data.
    pub fn snapshot(&self) -> BTreeMap<String, u64> {
        let mut out = BTreeMap::new();
        for family in [
            CapabilityFamily::AccountRecovery,
            CapabilityFamily::ReadProjection,
            CapabilityFamily::BaseFocusa,
            CapabilityFamily::Automation,
            CapabilityFamily::TeamRemote,
            CapabilityFamily::ReleaseProof,
            CapabilityFamily::PremiumUpdates,
            CapabilityFamily::CustomerDataExport,
            CapabilityFamily::InternalMaintenance,
        ] {
            out.insert(
                format!("family.{}.count", family.label()),
                self.family_count(family),
            );
        }
        for reason in [
            DecisionReason::Allow,
            DecisionReason::AllowVerifiedLimited,
            DecisionReason::Read,
            DecisionReason::ReadLocalOnly,
            DecisionReason::AllowExistingLocalOnly,
            DecisionReason::AllowOfflineOnly,
            DecisionReason::RequireBase,
            DecisionReason::RequireFeature,
            DecisionReason::RequireCachedFeature,
            DecisionReason::RequireCachedFeatureWhenSafe,
            DecisionReason::Inherit,
            DecisionReason::MissingInitiatingPolicy,
            DecisionReason::Deny,
        ] {
            out.insert(
                format!("reason.{}.count", reason.label()),
                self.reason_count(reason),
            );
        }
        for posture in [
            EntitlementPolicyPosture::Allow,
            EntitlementPolicyPosture::Read,
            EntitlementPolicyPosture::Base,
            EntitlementPolicyPosture::Feature,
            EntitlementPolicyPosture::Deny,
        ] {
            out.insert(
                format!("posture.{}.count", posture.status()),
                self.posture_count(posture),
            );
        }
        out
    }
}
