//! Spec 152F.04.07 — premium limit, idempotency, and grant isolation adversarial
//! matrix.
//!
//! Proves that no premium family or limit can be widened by client metadata,
//! presentation layer, replay, race, stale state, or another product/account.
//! Every case runs through the canonical premium decision
//! (`resolve_premium_family` / `resolve_export_packaged`) and the limit
//! reservation service (`LimitReservationService`), driven by the deterministic
//! product fixtures in `tests/fixtures/`.

use chrono::{Duration, Utc};
use focusa_license::authority::{EntitlementSnapshot, EntitlementState};
use focusa_license::{
    CapabilityFamily, LimitReservationService, PremiumFamilyDenial, ReservationError,
    ReservationScope, declared_server_owned_limit_buckets, family_limit_buckets,
};
use serde_json::Value;

const FIXTURE_JSON: &str =
    include_str!("fixtures/spec152f-premium-family-adversarial-fixtures.v1.json");

fn fixtures() -> Value {
    serde_json::from_str(FIXTURE_JSON).expect("fixture JSON must parse")
}

fn entry_snapshot(list: &Value, id: &str) -> EntitlementSnapshot {
    let entries = list.as_array().expect("fixture entries array");
    let entry = entries
        .iter()
        .find(|entry| entry["id"] == id)
        .unwrap_or_else(|| panic!("missing fixture entry {id}"));
    serde_json::from_value(entry["snapshot"].clone()).expect("snapshot must deserialize")
}

fn case_snapshot(id: &str) -> EntitlementSnapshot {
    entry_snapshot(&fixtures()["cases"], id)
}

fn product_snapshot(id: &str) -> EntitlementSnapshot {
    entry_snapshot(&fixtures()["products"], id)
}

fn scope_of(snapshot: &EntitlementSnapshot) -> ReservationScope {
    ReservationScope::from_snapshot(snapshot)
}

// ── Positive control: fixture products resolve their exact grants ──────────

#[test]
fn spec152f_premium_adversarial_fixture_products_resolve_exact_family_features() {
    let mut service = LimitReservationService::new();
    let now = Utc::now();

    let automation = product_snapshot("focusa-automation-operator");
    let grant = service
        .reserve(
            &automation,
            CapabilityFamily::Automation,
            "focusa.agent.silent_sessions",
            "concurrent_agents",
            "matrix-auto",
            4,
            now,
        )
        .expect("automation operator capacity 4");
    assert_eq!(grant.lease_sequence, 11);
    assert!(!grant.offline_cached);

    let team = product_snapshot("focusa-team-remote-operator");
    let grant = service
        .reserve(
            &team,
            CapabilityFamily::TeamRemote,
            "focusa.team.multi_operator",
            "team_operators",
            "matrix-team",
            3,
            now,
        )
        .expect("team operator capacity 3");
    assert_eq!(grant.family, CapabilityFamily::TeamRemote);

    let release = product_snapshot("focusa-release-proof-operator");
    service
        .reserve(
            &release,
            CapabilityFamily::ReleaseProof,
            "focusa.release.proof",
            "governed_proof_packets",
            "matrix-release",
            2,
            now,
        )
        .expect("release-proof capacity 2");

    let updates = product_snapshot("focusa-premium-updates-operator");
    service
        .reserve(
            &updates,
            CapabilityFamily::PremiumUpdates,
            "focusa.update.unattended",
            "managed_rollout_targets",
            "matrix-updates",
            5,
            now,
        )
        .expect("premium-updates capacity 5");

    let export = product_snapshot("focusa-export-packaged-operator");
    service
        .reserve(
            &export,
            CapabilityFamily::CustomerDataExport,
            "focusa.export.packaged",
            "packaged_export_bundles",
            "matrix-export",
            2,
            now,
        )
        .expect("packaged export capacity 3");
}

// ── Wrong product ──────────────────────────────────────────────────────────

#[test]
fn spec152f_premium_adversarial_wrong_product_cannot_reserve() {
    let mut service = LimitReservationService::new();
    let snapshot = case_snapshot("wrong-product");

    for (family, feature, bucket) in [
        (
            CapabilityFamily::Automation,
            "focusa.agent.silent_sessions",
            "concurrent_agents",
        ),
        (
            CapabilityFamily::TeamRemote,
            "focusa.team.multi_operator",
            "team_operators",
        ),
        (
            CapabilityFamily::ReleaseProof,
            "focusa.release.proof",
            "governed_proof_packets",
        ),
    ] {
        let denial = service
            .reserve(
                &snapshot,
                family,
                feature,
                bucket,
                "wrong-key",
                1,
                Utc::now(),
            )
            .expect_err("a non-focusa product must never reserve premium capacity");
        assert!(
            matches!(
                denial,
                ReservationError::FamilyDenied(PremiumFamilyDenial::BaseProductRequired { .. })
            ),
            "expected base-product denial, got {denial:?}"
        );
    }
    assert_eq!(service.outstanding_count(), 0);
}

// ── Caller-supplied feature / bucket ───────────────────────────────────────

#[test]
fn spec152f_premium_adversarial_caller_supplied_feature_is_rejected() {
    let mut service = LimitReservationService::new();
    let snapshot = case_snapshot("caller-feature");

    // A caller-invented feature identifier never resolves.
    let denial = service
        .reserve(
            &snapshot,
            CapabilityFamily::Automation,
            "focusa.agent.caller_invented",
            "concurrent_agents",
            "caller-feat",
            1,
            Utc::now(),
        )
        .expect_err("caller-invented feature must be rejected");
    assert!(
        matches!(
            denial,
            ReservationError::FamilyDenied(PremiumFamilyDenial::FeatureNotRegistered { .. })
        ),
        "expected FeatureNotRegistered, got {denial:?}"
    );

    // A feature granted for another family cannot be requested through this
    // family (cross-family by feature).
    let denial = service
        .reserve(
            &snapshot,
            CapabilityFamily::Automation,
            "focusa.team.multi_operator",
            "concurrent_agents",
            "caller-cross",
            1,
            Utc::now(),
        )
        .expect_err("cross-family feature must be rejected");
    assert!(
        matches!(
            denial,
            ReservationError::FamilyDenied(PremiumFamilyDenial::FeatureNotRegistered { .. })
        ),
        "expected FeatureNotRegistered, got {denial:?}"
    );
}

#[test]
fn spec152f_premium_adversarial_caller_supplied_bucket_is_rejected() {
    let mut service = LimitReservationService::new();
    let snapshot = case_snapshot("caller-feature");

    let denial = service
        .reserve(
            &snapshot,
            CapabilityFamily::Automation,
            "focusa.agent.silent_sessions",
            "unlimited_everything",
            "caller-bucket",
            1,
            Utc::now(),
        )
        .expect_err("caller-invented limit bucket must be rejected");
    assert!(matches!(
        denial,
        ReservationError::UnknownLimitBucket { .. }
    ));

    // A declared bucket from another family is not available here either.
    let denial = service
        .reserve(
            &snapshot,
            CapabilityFamily::Automation,
            "focusa.agent.silent_sessions",
            "team_operators",
            "caller-bucket-2",
            1,
            Utc::now(),
        )
        .expect_err("another family's bucket must be rejected");
    assert!(matches!(
        denial,
        ReservationError::UnknownLimitBucket { .. }
    ));
}

// ── Stale lease ────────────────────────────────────────────────────────────

#[test]
fn spec152f_premium_adversarial_stale_lease_fails_closed() {
    let mut service = LimitReservationService::new();
    let now = Utc::now();

    // The stale-lease fixture is an expired authority lease.
    let expired = case_snapshot("stale-lease");
    let denial = service
        .reserve(
            &expired,
            CapabilityFamily::ReleaseProof,
            "focusa.release.proof",
            "governed_proof_packets",
            "stale-expired",
            1,
            now,
        )
        .expect_err("expired lease must be rejected");
    assert!(
        matches!(
            denial,
            ReservationError::FamilyDenied(PremiumFamilyDenial::ActiveLeaseExpired)
        ),
        "expected ActiveLeaseExpired, got {denial:?}"
    );

    // A reservation bound to one lease identity becomes stale when a higher
    // authority sequence supersedes it.
    let current = product_snapshot("focusa-release-proof-operator");
    service
        .reserve(
            &current,
            CapabilityFamily::ReleaseProof,
            "focusa.release.proof",
            "governed_proof_packets",
            "stale-superseded",
            1,
            now,
        )
        .expect("current lease reserves");

    let mut superseded = current.clone();
    superseded.sequence = Some(superseded.sequence.unwrap() + 1);
    superseded.lease_digest = Some("sha256:superseded".to_string());

    let denial = service
        .revalidate(&superseded, "stale-superseded", now)
        .expect_err("superseded lease must fail revalidation");
    assert!(
        matches!(
            &denial,
            ReservationError::StaleLease { reason, .. } if reason == "lease_identity_changed"
        ),
        "expected stale lease, got {denial:?}"
    );

    let denial = service
        .settle(&superseded, "stale-superseded", 0)
        .expect_err("superseded lease must fail settlement");
    assert!(matches!(denial, ReservationError::StaleLease { .. }));
}

// ── Duplicate request / replay idempotency ─────────────────────────────────

#[test]
fn spec152f_premium_adversarial_duplicate_request_replays_without_double_reserve() {
    let mut service = LimitReservationService::new();
    let snapshot = product_snapshot("focusa-automation-operator");

    let first = service
        .reserve(
            &snapshot,
            CapabilityFamily::Automation,
            "focusa.agent.silent_sessions",
            "concurrent_agents",
            "dup-request",
            2,
            Utc::now(),
        )
        .expect("first request reserves 2 of 4");
    let replay = service
        .reserve(
            &snapshot,
            CapabilityFamily::Automation,
            "focusa.agent.silent_sessions",
            "concurrent_agents",
            "dup-request",
            2,
            Utc::now(),
        )
        .expect("replay returns the same grant");
    assert_eq!(first, replay, "replay must return the identical grant");
    assert_eq!(
        service.outstanding_count(),
        1,
        "replay must not double-reserve"
    );

    // The same key with a different payload is a conflict, never a new grant.
    let denial = service
        .reserve(
            &snapshot,
            CapabilityFamily::Automation,
            "focusa.agent.silent_sessions",
            "concurrent_agents",
            "dup-request",
            3,
            Utc::now(),
        )
        .expect_err("same key with different units must conflict");
    assert!(matches!(
        denial,
        ReservationError::IdempotencyConflict { .. }
    ));

    // Capacity accounting is still exact: 2 reserved, 2 remain.
    let scope = scope_of(&snapshot);
    assert_eq!(service.reserved_units("concurrent_agents", &scope), 2);
    service
        .reserve(
            &snapshot,
            CapabilityFamily::Automation,
            "focusa.agent.silent_sessions",
            "concurrent_agents",
            "dup-request-2",
            2,
            Utc::now(),
        )
        .expect("remaining capacity 2 is reservable");
    let denial = service
        .reserve(
            &snapshot,
            CapabilityFamily::Automation,
            "focusa.agent.silent_sessions",
            "concurrent_agents",
            "dup-request-3",
            1,
            Utc::now(),
        )
        .expect_err("capacity is exhausted after 4 units");
    assert!(matches!(denial, ReservationError::LimitExhausted { .. }));
}

// ── Exhausted limit ────────────────────────────────────────────────────────

#[test]
fn spec152f_premium_adversarial_exhausted_limit_is_denied() {
    let mut service = LimitReservationService::new();
    let snapshot = product_snapshot("focusa-team-remote-operator");

    // Requesting more than the lease grants can never widen the limit.
    let denial = service
        .reserve(
            &snapshot,
            CapabilityFamily::TeamRemote,
            "focusa.team.multi_operator",
            "team_operators",
            "over-request",
            4,
            Utc::now(),
        )
        .expect_err("4 seats with capacity 3 must be denied");
    assert!(
        matches!(
            denial,
            ReservationError::LimitExhausted {
                capacity: 3,
                requested: 4,
                ..
            }
        ),
        "got {denial:?}"
    );

    // Fill the bucket to capacity, then the next request is exhausted.
    service
        .reserve(
            &snapshot,
            CapabilityFamily::TeamRemote,
            "focusa.team.multi_operator",
            "team_operators",
            "seat-1",
            2,
            Utc::now(),
        )
        .expect("2 of 3 seats reserve");
    let denial = service
        .reserve(
            &snapshot,
            CapabilityFamily::TeamRemote,
            "focusa.remote.stream",
            "team_operators",
            "seat-2",
            2,
            Utc::now(),
        )
        .expect_err("2 + 2 > 3 must be exhausted");
    assert!(matches!(denial, ReservationError::LimitExhausted { .. }));
}

// ── Concurrent reservation ─────────────────────────────────────────────────

#[test]
fn spec152f_premium_adversarial_concurrent_reservation_never_double_allocates() {
    // A race between two arrivals cannot allocate the same capacity twice:
    // the service checks outstanding reservations at reserve time.
    let now = Utc::now();
    let mut snapshot = product_snapshot("focusa-automation-operator");
    snapshot.limits.insert("concurrent_agents".to_string(), 1);

    let mut service = LimitReservationService::new();
    service
        .reserve(
            &snapshot,
            CapabilityFamily::Automation,
            "focusa.agent.silent_sessions",
            "concurrent_agents",
            "race-a",
            1,
            now,
        )
        .expect("first arrival reserves the single slot");

    let denial = service
        .reserve(
            &snapshot,
            CapabilityFamily::Automation,
            "focusa.agent.silent_sessions",
            "concurrent_agents",
            "race-b",
            1,
            now,
        )
        .expect_err("second arrival sees the slot taken");
    assert!(matches!(denial, ReservationError::LimitExhausted { .. }));

    // Settlement releases the slot for the next arrival.
    service
        .settle(&snapshot, "race-a", 1)
        .expect("settle race-a");
    service
        .reserve(
            &snapshot,
            CapabilityFamily::Automation,
            "focusa.agent.silent_sessions",
            "concurrent_agents",
            "race-c",
            1,
            now,
        )
        .expect("settled slot is available again");
}

// ── Evaluation omission ────────────────────────────────────────────────────

#[test]
fn spec152f_premium_adversarial_evaluation_omission_denies_premium() {
    // The evaluation-omission fixture is a valid Focusa lease that simply does
    // not include any premium feature. Evaluation grants premium only when the
    // grant includes it; omission can never be widened by the caller.
    let snapshot = case_snapshot("evaluation-omission");
    let mut service = LimitReservationService::new();

    for (family, feature, bucket) in [
        (
            CapabilityFamily::Automation,
            "focusa.agent.silent_sessions",
            "concurrent_agents",
        ),
        (
            CapabilityFamily::TeamRemote,
            "focusa.team.multi_operator",
            "team_operators",
        ),
        (
            CapabilityFamily::ReleaseProof,
            "focusa.release.proof",
            "governed_proof_packets",
        ),
        (
            CapabilityFamily::PremiumUpdates,
            "focusa.update.unattended",
            "managed_rollout_targets",
        ),
    ] {
        let denial = service
            .reserve(
                &snapshot,
                family,
                feature,
                bucket,
                "eval-omission",
                1,
                Utc::now(),
            )
            .expect_err("omitted premium feature must be denied");
        assert!(
            matches!(
                denial,
                ReservationError::FamilyDenied(PremiumFamilyDenial::MissingFeature { .. })
            ),
            "expected MissingFeature for {family:?}, got {denial:?}"
        );
    }
    assert_eq!(service.outstanding_count(), 0);
}

// ── Offline Grace expansion ────────────────────────────────────────────────

#[test]
fn spec152f_premium_adversarial_offline_grace_expansion_cannot_expand_grants() {
    let mut service = LimitReservationService::new();
    let now = Utc::now();

    // Grace without the feature: expansion is impossible.
    let grace = case_snapshot("offline-grace-expansion");
    let denial = service
        .reserve(
            &grace,
            CapabilityFamily::Automation,
            "focusa.agent.silent_sessions",
            "concurrent_agents",
            "grace-expand",
            1,
            now,
        )
        .expect_err("offline grace cannot create a premium grant");
    assert!(
        matches!(
            denial,
            ReservationError::FamilyDenied(PremiumFamilyDenial::MissingFeature { .. })
        ),
        "got {denial:?}"
    );

    // Grace with the feature but outside the signed window: cached grant is gone.
    let mut expired_grace = product_snapshot("focusa-automation-operator");
    expired_grace.state = EntitlementState::OfflineGrace;
    expired_grace.offline_grace_until = Some(now - Duration::minutes(1));
    let denial = service
        .reserve(
            &expired_grace,
            CapabilityFamily::Automation,
            "focusa.agent.silent_sessions",
            "concurrent_agents",
            "grace-expired",
            1,
            now,
        )
        .expect_err("expired grace window must deny");
    assert!(
        matches!(
            denial,
            ReservationError::FamilyDenied(PremiumFamilyDenial::CachedGrantExpired)
        ),
        "got {denial:?}"
    );

    // Grace with the feature inside the window is a bounded cached feature: it
    // stays within the signed feature set and lease capacity.
    let mut valid_grace = product_snapshot("focusa-automation-operator");
    valid_grace.state = EntitlementState::OfflineGrace;
    valid_grace.offline_grace_until = Some(now + Duration::minutes(5));
    let grant = service
        .reserve(
            &valid_grace,
            CapabilityFamily::Automation,
            "focusa.agent.silent_sessions",
            "concurrent_agents",
            "grace-cached",
            2,
            now,
        )
        .expect("cached feature inside the window is allowed");
    assert!(grant.offline_cached, "offline-cached grant is descriptive");
    assert_eq!(grant.lease_sequence, 11);
}

// ── Cross-family ───────────────────────────────────────────────────────────

#[test]
fn spec152f_premium_adversarial_cross_family_feature_is_rejected() {
    let mut service = LimitReservationService::new();
    let team = product_snapshot("focusa-team-remote-operator");

    // Automation feature requested under team_remote.
    let denial = service
        .reserve(
            &team,
            CapabilityFamily::TeamRemote,
            "focusa.agent.silent_sessions",
            "team_operators",
            "cross-1",
            1,
            Utc::now(),
        )
        .expect_err("automation feature under team_remote must be rejected");
    assert!(
        matches!(
            denial,
            ReservationError::FamilyDenied(PremiumFamilyDenial::FeatureNotRegistered {
                family: CapabilityFamily::TeamRemote,
                ..
            })
        ),
        "got {denial:?}"
    );

    // Release-proof feature requested under premium_updates.
    let updates = product_snapshot("focusa-premium-updates-operator");
    let denial = service
        .reserve(
            &updates,
            CapabilityFamily::PremiumUpdates,
            "focusa.release.proof",
            "managed_rollout_targets",
            "cross-2",
            1,
            Utc::now(),
        )
        .expect_err("release feature under premium_updates must be rejected");
    assert!(
        matches!(
            denial,
            ReservationError::FamilyDenied(PremiumFamilyDenial::FeatureNotRegistered {
                family: CapabilityFamily::PremiumUpdates,
                ..
            })
        ),
        "got {denial:?}"
    );
}

// ── Cross-account ──────────────────────────────────────────────────────────

#[test]
fn spec152f_premium_adversarial_cross_account_scope_is_isolated() {
    let mut service = LimitReservationService::new();
    let now = Utc::now();

    let account_a = product_snapshot("focusa-team-remote-operator");
    let account_b = case_snapshot("cross-account");

    service
        .reserve(
            &account_a,
            CapabilityFamily::TeamRemote,
            "focusa.team.multi_operator",
            "team_operators",
            "a-seat",
            3,
            now,
        )
        .expect("account A fills its 3 seats");

    // Account B sees its own independent capacity, never A's.
    let b_scope = scope_of(&account_b);
    assert_eq!(service.reserved_units("team_operators", &b_scope), 0);
    service
        .reserve(
            &account_b,
            CapabilityFamily::TeamRemote,
            "focusa.team.multi_operator",
            "team_operators",
            "b-seat",
            3,
            now,
        )
        .expect("account B has independent capacity");

    // Account B cannot settle or revalidate account A's reservation.
    let denial = service
        .settle(&account_b, "a-seat", 0)
        .expect_err("foreign account cannot settle");
    assert!(
        matches!(
            &denial,
            ReservationError::StaleLease { reason, .. } if reason == "scope_changed"
        ),
        "got {denial:?}"
    );
    let denial = service
        .revalidate(&account_b, "a-seat", now)
        .expect_err("foreign account cannot revalidate");
    assert!(matches!(denial, ReservationError::StaleLease { .. }));
}

// ── Server-owned limit registry ────────────────────────────────────────────

#[test]
fn spec152f_premium_adversarial_declared_limit_buckets_are_server_owned() {
    // Every family bucket is declared; the declaration set is frozen; and no
    // bucket belongs to more than one family, so capacity can never be aliased.
    let declared = declared_server_owned_limit_buckets();
    assert_eq!(declared.len(), 8, "declared bucket set is frozen");

    let automation = family_limit_buckets(CapabilityFamily::Automation);
    let team = family_limit_buckets(CapabilityFamily::TeamRemote);
    let release = family_limit_buckets(CapabilityFamily::ReleaseProof);
    let updates = family_limit_buckets(CapabilityFamily::PremiumUpdates);
    let export = family_limit_buckets(CapabilityFamily::CustomerDataExport);

    let all_family_buckets = [automation, team, release, updates, export];
    for buckets in all_family_buckets {
        assert!(
            !buckets.is_empty(),
            "premium families declare limit buckets"
        );
        for bucket in buckets {
            assert!(
                declared.contains(bucket),
                "{bucket} must be a declared server-owned limit bucket"
            );
        }
    }

    let mut seen: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    for buckets in all_family_buckets {
        for bucket in buckets {
            assert!(
                seen.insert(bucket),
                "{bucket} must belong to exactly one family"
            );
        }
    }

    // Non-premium families cannot reserve capacity at all.
    assert!(family_limit_buckets(CapabilityFamily::BaseFocusa).is_empty());
    assert!(family_limit_buckets(CapabilityFamily::AccountRecovery).is_empty());
}

// ── Unknown / already-settled replay ───────────────────────────────────────

#[test]
fn spec152f_premium_adversarial_unknown_and_settled_replay_fail_closed() {
    let mut service = LimitReservationService::new();
    let snapshot = product_snapshot("focusa-automation-operator");

    let denial = service
        .revalidate(&snapshot, "never-reserved", Utc::now())
        .expect_err("unknown reservation must fail revalidation");
    assert!(matches!(
        denial,
        ReservationError::UnknownReservation { .. }
    ));

    service
        .reserve(
            &snapshot,
            CapabilityFamily::Automation,
            "focusa.agent.silent_sessions",
            "concurrent_agents",
            "settled-key",
            1,
            Utc::now(),
        )
        .expect("reserve");
    service
        .settle(&snapshot, "settled-key", 1)
        .expect("settle releases the reservation");

    // Replaying an already-settled key cannot resurrect capacity.
    let denial = service
        .revalidate(&snapshot, "settled-key", Utc::now())
        .expect_err("settled reservation must not revalidate");
    assert!(matches!(
        denial,
        ReservationError::UnknownReservation { .. }
    ));
}
