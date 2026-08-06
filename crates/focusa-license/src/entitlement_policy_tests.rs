use super::*;
use serde::de::DeserializeOwned;

fn round_trip<T>(value: T, stable_name: &str)
where
    T: Serialize + DeserializeOwned + PartialEq + std::fmt::Debug,
{
    let encoded = serde_json::to_string(&value).unwrap();
    assert_eq!(encoded, format!("\"{stable_name}\""));
    assert_eq!(serde_json::from_str::<T>(&encoded).unwrap(), value);
}

#[test]
fn entitlement_policy_types_round_trip_registry_names() {
    for (value, name) in [
        (CapabilityFamily::AccountRecovery, "account_recovery"),
        (CapabilityFamily::ReadProjection, "read_projection"),
        (CapabilityFamily::BaseFocusa, "base_focusa"),
        (CapabilityFamily::Automation, "automation"),
        (CapabilityFamily::TeamRemote, "team_remote"),
        (CapabilityFamily::ReleaseProof, "release_proof"),
        (CapabilityFamily::PremiumUpdates, "premium_updates"),
        (CapabilityFamily::CustomerDataExport, "customer_data_export"),
        (
            CapabilityFamily::InternalMaintenance,
            "internal_maintenance",
        ),
    ] {
        round_trip(value, name);
    }
    for (value, name) in [
        (
            PolicyEntitlementState::PendingUnverified,
            "pending_unverified",
        ),
        (PolicyEntitlementState::VerifiedNoGrant, "verified_no_grant"),
        (PolicyEntitlementState::Evaluation, "evaluation"),
        (PolicyEntitlementState::ActivePaid, "active_paid"),
        (PolicyEntitlementState::OfflineGrace, "offline_grace"),
        (PolicyEntitlementState::Expired, "expired"),
        (
            PolicyEntitlementState::RefundedOrRevoked,
            "refunded_or_revoked",
        ),
        (
            PolicyEntitlementState::MissingOrCorrupt,
            "missing_or_corrupt",
        ),
    ] {
        round_trip(value, name);
    }
    for (value, name) in [
        (OperationClass::Read, "read"),
        (OperationClass::ValueMutation, "value_mutation"),
        (OperationClass::Recovery, "recovery"),
        (OperationClass::InternalMaintenance, "internal_maintenance"),
        (OperationClass::Unknown, "unknown"),
    ] {
        round_trip(value, name);
    }
    for (value, name) in [
        (CommercialTreatment::AlwaysAvailable, "always_available"),
        (CommercialTreatment::ReadAllowance, "read_allowance"),
        (CommercialTreatment::BaseEntitlement, "base_entitlement"),
        (CommercialTreatment::OptionalPremium, "optional_premium"),
        (
            CommercialTreatment::AlwaysAvailableBasicWithOptionalPremiumPackaging,
            "always_available_basic_with_optional_premium_packaging",
        ),
        (
            CommercialTreatment::InheritInitiatingOperation,
            "inherit_initiating_operation",
        ),
    ] {
        round_trip(value, name);
    }
    for (value, name) in [
        (PolicyActivation::Active, "active"),
        (PolicyActivation::Dormant, "dormant"),
        (
            PolicyActivation::ActiveOnlyWhenDeclared,
            "active_only_when_declared",
        ),
        (PolicyActivation::DormantForCommerce, "dormant_for_commerce"),
        (
            PolicyActivation::ActiveForPreviewNightlyAndUnattended,
            "active_for_preview_nightly_and_unattended",
        ),
    ] {
        round_trip(value, name);
    }
    for (value, name) in [
        (RecoveryAllowance::None, "none"),
        (RecoveryAllowance::AccountRecovery, "account_recovery"),
        (RecoveryAllowance::ReadProjection, "read_projection"),
        (
            RecoveryAllowance::CustomerDataExport,
            "customer_data_export",
        ),
        (
            RecoveryAllowance::StableSecurityUpdate,
            "stable_security_update",
        ),
        (RecoveryAllowance::RepairRollback, "repair_rollback"),
        (RecoveryAllowance::Uninstall, "uninstall"),
    ] {
        round_trip(value, name);
    }
    for (value, name) in [
        (DecisionReason::Allow, "allow"),
        (DecisionReason::Read, "read"),
        (DecisionReason::ReadLocalOnly, "read_local_only"),
        (
            DecisionReason::AllowExistingLocalOnly,
            "allow_existing_local_only",
        ),
        (DecisionReason::AllowOfflineOnly, "allow_offline_only"),
        (DecisionReason::RequireBase, "require_base"),
        (DecisionReason::RequireFeature, "require_feature"),
        (
            DecisionReason::RequireCachedFeature,
            "require_cached_feature",
        ),
        (
            DecisionReason::RequireCachedFeatureWhenSafe,
            "require_cached_feature_when_safe",
        ),
        (DecisionReason::Inherit, "inherit"),
        (DecisionReason::Deny, "deny"),
    ] {
        round_trip(value, name);
    }
    round_trip(
        RequiredFeature::new("focusa.release.proof").unwrap(),
        "focusa.release.proof",
    );
    round_trip(
        LimitBucket::new("release_proofs").unwrap(),
        "release_proofs",
    );
}

#[test]
fn entitlement_policy_types_reject_unknown_or_malformed_active_values() {
    assert!(serde_json::from_str::<CapabilityFamily>("\"invented_family\"").is_err());
    assert!(serde_json::from_str::<PolicyEntitlementState>("\"licensed\"").is_err());
    assert!(serde_json::from_str::<RequiredFeature>("\"release.proof\"").is_err());
    assert!(serde_json::from_str::<LimitBucket>("\"Release-Proofs\"").is_err());
    assert_eq!(
        ResolvedEntitlementPolicy::try_new(
            OperationClass::Unknown,
            CapabilityFamily::BaseFocusa,
            CommercialTreatment::BaseEntitlement,
            PolicyActivation::Active,
            PolicyEntitlementState::ActivePaid,
            None,
            None,
            RecoveryAllowance::None,
            DecisionReason::RequireBase,
        ),
        Err(EntitlementPolicyTypeError::UnknownOperationClass)
    );
    assert_eq!(
        ResolvedEntitlementPolicy::try_new(
            OperationClass::ValueMutation,
            CapabilityFamily::BaseFocusa,
            CommercialTreatment::BaseEntitlement,
            PolicyActivation::Dormant,
            PolicyEntitlementState::ActivePaid,
            None,
            None,
            RecoveryAllowance::None,
            DecisionReason::RequireBase,
        ),
        Err(EntitlementPolicyTypeError::DormantPolicyActivation)
    );
}

#[test]
fn entitlement_policy_types_reject_illegal_combinations() {
    let feature = RequiredFeature::new("focusa.agent.silent_sessions").unwrap();
    let valid = ResolvedEntitlementPolicy::try_new(
        OperationClass::ValueMutation,
        CapabilityFamily::Automation,
        CommercialTreatment::OptionalPremium,
        PolicyActivation::Active,
        PolicyEntitlementState::ActivePaid,
        Some(feature.clone()),
        Some(LimitBucket::new("silent_sessions").unwrap()),
        RecoveryAllowance::None,
        DecisionReason::RequireFeature,
    )
    .unwrap();
    assert_eq!(valid.required_feature(), Some(&feature));
    assert_eq!(valid.capability_family(), CapabilityFamily::Automation);
    assert_eq!(
        ResolvedEntitlementPolicy::try_new(
            OperationClass::ValueMutation,
            CapabilityFamily::Automation,
            CommercialTreatment::BaseEntitlement,
            PolicyActivation::Active,
            PolicyEntitlementState::ActivePaid,
            Some(feature.clone()),
            None,
            RecoveryAllowance::None,
            DecisionReason::RequireFeature,
        ),
        Err(EntitlementPolicyTypeError::FamilyTreatmentMismatch)
    );
    assert_eq!(
        ResolvedEntitlementPolicy::try_new(
            OperationClass::ValueMutation,
            CapabilityFamily::Automation,
            CommercialTreatment::OptionalPremium,
            PolicyActivation::Active,
            PolicyEntitlementState::ActivePaid,
            None,
            None,
            RecoveryAllowance::None,
            DecisionReason::RequireFeature,
        ),
        Err(EntitlementPolicyTypeError::FeatureReasonMismatch)
    );
    assert_eq!(
        ResolvedEntitlementPolicy::try_new(
            OperationClass::Read,
            CapabilityFamily::ReadProjection,
            CommercialTreatment::ReadAllowance,
            PolicyActivation::Active,
            PolicyEntitlementState::Expired,
            None,
            Some(LimitBucket::new("reads").unwrap()),
            RecoveryAllowance::ReadProjection,
            DecisionReason::Read,
        ),
        Err(EntitlementPolicyTypeError::InactiveLimitBucket)
    );
    assert_eq!(
        ResolvedEntitlementPolicy::try_new(
            OperationClass::Recovery,
            CapabilityFamily::AccountRecovery,
            CommercialTreatment::AlwaysAvailable,
            PolicyActivation::Active,
            PolicyEntitlementState::RefundedOrRevoked,
            Some(feature),
            None,
            RecoveryAllowance::AccountRecovery,
            DecisionReason::Allow,
        ),
        Err(EntitlementPolicyTypeError::FeatureReasonMismatch)
    );
}
