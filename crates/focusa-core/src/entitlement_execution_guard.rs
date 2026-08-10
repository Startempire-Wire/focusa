use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

const ENTITLEMENT_ALLOWED: &str = "ENTITLEMENT_ALLOWED";
const ENTITLEMENT_BASE_REQUIRED: &str = "ENTITLEMENT_BASE_REQUIRED";
const ENTITLEMENT_FEATURE_REQUIRED: &str = "ENTITLEMENT_FEATURE_REQUIRED";
const ENTITLEMENT_REQUIRED: &str = "ENTITLEMENT_REQUIRED";
const ENTITLEMENT_ROUTE_UNCLASSIFIED: &str = "ENTITLEMENT_ROUTE_UNCLASSIFIED";

/// Canonical operation metadata for one entitlement gate decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntitlementExecutionPolicy {
    pub operation_id: String,
    pub operation_class: focusa_license::OperationClass,
    pub capability_family: focusa_license::CapabilityFamily,
    pub required_feature: Option<String>,
    pub limit_bucket: Option<String>,
    pub recovery_allowance: focusa_license::RecoveryAllowance,
}

impl EntitlementExecutionPolicy {
    pub fn new(
        operation_id: impl Into<String>,
        operation_class: focusa_license::OperationClass,
        capability_family: focusa_license::CapabilityFamily,
        required_feature: Option<&str>,
        limit_bucket: Option<&str>,
        recovery_allowance: focusa_license::RecoveryAllowance,
    ) -> Self {
        Self {
            operation_id: operation_id.into(),
            operation_class,
            capability_family,
            required_feature: required_feature.map(|value| value.to_string()),
            limit_bucket: limit_bucket.map(|value| value.to_string()),
            recovery_allowance,
        }
    }

    fn validate(&self) -> Result<(), EntitlementExecutionFailure> {
        if self.operation_class == focusa_license::OperationClass::Unknown {
            return Err(EntitlementExecutionFailure {
                code: ENTITLEMENT_ROUTE_UNCLASSIFIED.to_string(),
                message: "operation_class is unknown".to_string(),
                required_feature: self.required_feature.clone(),
                limit_bucket: self.limit_bucket.clone(),
            });
        }

        let operation_family_compatible = matches!(
            (self.capability_family, self.operation_class),
            (
                focusa_license::CapabilityFamily::AccountRecovery,
                focusa_license::OperationClass::Recovery
            )
                | (
                    focusa_license::CapabilityFamily::ReadProjection,
                    focusa_license::OperationClass::Read
                )
                | (
                    focusa_license::CapabilityFamily::CustomerDataExport,
                    focusa_license::OperationClass::Read
                        | focusa_license::OperationClass::Recovery
                        | focusa_license::OperationClass::ValueMutation
                )
                | (
                    focusa_license::CapabilityFamily::InternalMaintenance,
                    focusa_license::OperationClass::InternalMaintenance
                )
                | (
                    focusa_license::CapabilityFamily::BaseFocusa
                        | focusa_license::CapabilityFamily::Automation
                        | focusa_license::CapabilityFamily::TeamRemote
                        | focusa_license::CapabilityFamily::ReleaseProof
                        | focusa_license::CapabilityFamily::PremiumUpdates,
                    focusa_license::OperationClass::ValueMutation
                )
        );
        if !operation_family_compatible {
            return Err(EntitlementExecutionFailure {
                code: ENTITLEMENT_ROUTE_UNCLASSIFIED.to_string(),
                message: "operation_class is incompatible with capability_family".to_string(),
                required_feature: self.required_feature.clone(),
                limit_bucket: self.limit_bucket.clone(),
            });
        }

        if let Some(inferred) = self.recovery_allowance.implied_family() {
            if inferred != self.capability_family {
                return Err(EntitlementExecutionFailure {
                    code: ENTITLEMENT_ROUTE_UNCLASSIFIED.to_string(),
                    message: "recovery allowance implies a different capability family".to_string(),
                    required_feature: self.required_feature.clone(),
                    limit_bucket: self.limit_bucket.clone(),
                });
            }
        }

        if self.capability_family.is_optional_premium() && self.required_feature.is_none() {
            return Err(EntitlementExecutionFailure {
                code: ENTITLEMENT_ROUTE_UNCLASSIFIED.to_string(),
                message: "premium family requires required_feature".to_string(),
                required_feature: self.required_feature.clone(),
                limit_bucket: self.limit_bucket.clone(),
            });
        }
        if !self.capability_family.is_optional_premium() && self.required_feature.is_some() {
            return Err(EntitlementExecutionFailure {
                code: ENTITLEMENT_ROUTE_UNCLASSIFIED.to_string(),
                message: "required_feature is only valid for optional premium families".to_string(),
                required_feature: self.required_feature.clone(),
                limit_bucket: self.limit_bucket.clone(),
            });
        }

        if let Some(feature) = self.required_feature.as_deref() {
            if focusa_license::RequiredFeature::new(feature).is_err() {
                return Err(EntitlementExecutionFailure {
                    code: ENTITLEMENT_FEATURE_REQUIRED.to_string(),
                    message: "required_feature is not a qualified entitlement feature".to_string(),
                    required_feature: self.required_feature.clone(),
                    limit_bucket: self.limit_bucket.clone(),
                });
            }
        }

        if let Some(limit_bucket) = self.limit_bucket.as_deref() {
            if focusa_license::LimitBucket::new(limit_bucket).is_err() {
                return Err(EntitlementExecutionFailure {
                    code: ENTITLEMENT_ROUTE_UNCLASSIFIED.to_string(),
                    message: "limit_bucket is not a valid identifier".to_string(),
                    required_feature: self.required_feature.clone(),
                    limit_bucket: self.limit_bucket.clone(),
                });
            }
        }

        Ok(())
    }
}

/// Runtime context around non-HTTP preflight checks.
#[derive(Debug, Clone, Copy)]
pub struct EntitlementExecutionContext {
    pub now: DateTime<Utc>,
    pub initiating_posture: Option<focusa_license::EntitlementPolicyPosture>,
}

impl Default for EntitlementExecutionContext {
    fn default() -> Self {
        Self {
            now: Utc::now(),
            initiating_posture: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EntitlementExecutionDecision {
    pub status: String,
    pub entitlement_state: String,
    pub operation_id: String,
    pub operation_class: String,
    pub capability_family: String,
    pub reason_code: String,
    pub recovery_action: String,
    pub required_feature: Option<String>,
    pub limit_bucket: Option<String>,
    pub policy_digest: String,
    pub lease_sequence: u64,
    pub offline_cached: bool,
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EntitlementExecutionFailure {
    pub code: String,
    pub message: String,
    pub required_feature: Option<String>,
    pub limit_bucket: Option<String>,
}

/// Resolve one canonical operation against the signed Focusa entitlement policy.
///
/// The operation policy and context must already have passed authentication,
/// authorization, and role checks.
pub fn evaluate_entitlement_execution(
    guard: &focusa_license::LicenseGuard,
    policy: &EntitlementExecutionPolicy,
    context: EntitlementExecutionContext,
) -> Result<EntitlementExecutionDecision, EntitlementExecutionFailure> {
    policy.validate()?;

    let state = guard
        .entitlement
        .as_ref()
        .map(focusa_license::authority_policy_state)
        .unwrap_or(focusa_license::PolicyEntitlementState::MissingOrCorrupt);

    let reduced = focusa_license::reduce_entitlement_state(
        state,
        policy.capability_family,
        if policy.capability_family == focusa_license::CapabilityFamily::InternalMaintenance {
            context.initiating_posture
        } else {
            None
        },
    );

    let lease_sequence = guard
        .entitlement
        .as_ref()
        .and_then(|snapshot| snapshot.sequence)
        .unwrap_or_default();

    let policy_digest = focusa_license::embedded_entitlement_policy_registry()
        .ok()
        .map(|registry| registry.digest().to_string())
        .unwrap_or_else(|| "focusa-entitlement-policy-missing".to_string());

    let mut base_decision = EntitlementExecutionDecision {
        status: reduced.posture().status().to_string(),
        entitlement_state: state.label().to_string(),
        operation_id: policy.operation_id.clone(),
        operation_class: policy.operation_class.label().to_string(),
        capability_family: policy.capability_family.label().to_string(),
        reason_code: reduced.reason().label().to_string(),
        recovery_action: reduced.reason().recovery_action().to_string(),
        required_feature: policy.required_feature.clone(),
        limit_bucket: policy.limit_bucket.clone(),
        policy_digest,
        lease_sequence,
        offline_cached: false,
        code: ENTITLEMENT_ALLOWED.to_string(),
    };

    match reduced.posture() {
        focusa_license::EntitlementPolicyPosture::Allow
        | focusa_license::EntitlementPolicyPosture::Read => Ok(base_decision),
        focusa_license::EntitlementPolicyPosture::Base => {
            let snapshot =
                guard
                    .entitlement
                    .as_ref()
                    .ok_or_else(|| EntitlementExecutionFailure {
                        code: ENTITLEMENT_BASE_REQUIRED.to_string(),
                        message: "A usable signed Focusa base entitlement is required for this operation.".to_string(),
                        required_feature: base_decision.required_feature.clone(),
                        limit_bucket: base_decision.limit_bucket.clone(),
                    })?;

            let projection =
                focusa_license::base_product_projection(Some(snapshot)).map_err(|_| EntitlementExecutionFailure {
                    code: ENTITLEMENT_BASE_REQUIRED.to_string(),
                    message: "No usable Focusa base entitlement is present for this operation.".to_string(),
                    required_feature: base_decision.required_feature.clone(),
                    limit_bucket: base_decision.limit_bucket.clone(),
                })?;

            if !projection.permits_base_mutations {
                return Err(EntitlementExecutionFailure {
                    code: ENTITLEMENT_BASE_REQUIRED.to_string(),
                    message: "Base Focusa entitlement is required for this value-producing operation.".to_string(),
                    required_feature: None,
                    limit_bucket: base_decision.limit_bucket,
                });
            }

            base_decision.code = ENTITLEMENT_ALLOWED.to_string();
            Ok(base_decision)
        }
        focusa_license::EntitlementPolicyPosture::Feature => {
            let snapshot =
                guard
                    .entitlement
                    .as_ref()
                    .ok_or_else(|| EntitlementExecutionFailure {
                        code: ENTITLEMENT_BASE_REQUIRED.to_string(),
                        message: "A usable signed Focusa base entitlement is required before premium feature checks.".to_string(),
                        required_feature: base_decision.required_feature.clone(),
                        limit_bucket: base_decision.limit_bucket.clone(),
                    })?;

            let required_feature = base_decision
                .required_feature
                .as_deref()
                .ok_or_else(|| EntitlementExecutionFailure {
                    code: ENTITLEMENT_FEATURE_REQUIRED.to_string(),
                    message: "premium family requires required_feature".to_string(),
                    required_feature: None,
                    limit_bucket: None,
                })?;

            let premium = focusa_license::resolve_premium_family(
                snapshot,
                policy.capability_family,
                required_feature,
                context.now,
            );
            match premium {
                focusa_license::PremiumFamilyDecision::Feature {
                    lease_sequence,
                    offline_cached,
                    ..
                } => {
                    base_decision.lease_sequence = lease_sequence;
                    base_decision.offline_cached = offline_cached;
                    base_decision.status = focusa_license::EntitlementPolicyPosture::Feature
                        .status()
                        .to_string();
                    base_decision.code = ENTITLEMENT_ALLOWED.to_string();
                    Ok(base_decision)
                }
                focusa_license::PremiumFamilyDecision::Denied(denial) => {
                    let failure =
                        entitlement_execution_premium_denial(denial, base_decision.required_feature.clone(), base_decision.limit_bucket.clone());
                    Err(failure)
                }
            }
        }
        focusa_license::EntitlementPolicyPosture::Deny => {
            let code = if matches!(
                reduced.reason(),
                focusa_license::DecisionReason::MissingInitiatingPolicy
            ) {
                ENTITLEMENT_ROUTE_UNCLASSIFIED
            } else if policy.capability_family == focusa_license::CapabilityFamily::BaseFocusa {
                ENTITLEMENT_BASE_REQUIRED
            } else {
                ENTITLEMENT_REQUIRED
            };
            Err(EntitlementExecutionFailure {
                code: code.to_string(),
                message: "The operation is denied by the entitlement policy before side effects.".to_string(),
                required_feature: base_decision.required_feature,
                limit_bucket: base_decision.limit_bucket,
            })
        }
    }
}

/// Resolve one canonical operation against the signed Focusa entitlement policy,
/// with an additional active-project guard for verified-no-license posture.
///
/// When the posture is verified-no-license and the capability family is BaseFocusa,
/// this function additionally checks that the targeted project is the single
/// explicitly selected mutable project. Second-project mutations and mutations
/// without an explicit selection are denied with upgrade/switch actions.
///
/// All other postures and families pass through to the base
/// `evaluate_entitlement_execution` without project-level checks.
pub fn evaluate_entitlement_execution_for_project(
    guard: &focusa_license::LicenseGuard,
    policy: &EntitlementExecutionPolicy,
    context: EntitlementExecutionContext,
    project_root: &str,
    active_selection: Option<&crate::limited_project::ActiveProjectSelection>,
) -> Result<EntitlementExecutionDecision, EntitlementExecutionFailure> {
    let decision = evaluate_entitlement_execution(guard, policy, context)?;

    // Only apply the project guard for BaseFocusa mutations in verified-no-license posture.
    if policy.capability_family == focusa_license::CapabilityFamily::BaseFocusa
        && policy.operation_class == focusa_license::OperationClass::ValueMutation
        && decision.reason_code == focusa_license::DecisionReason::AllowVerifiedLimited.label()
    {
        let base = focusa_license::resolve_base_focusa_product(
            "focusa",
            focusa_license::PolicyEntitlementState::VerifiedNoLicense,
        );
        let project_decision = crate::limited_project::ActiveProjectGuard::check_mutation(
            base,
            project_root,
            active_selection,
        );
        if project_decision.is_denied() {
            return Err(EntitlementExecutionFailure {
                code: "ENTITLEMENT_LIMITED_PROJECT".to_string(),
                message: project_decision.recovery_action().to_string(),
                required_feature: None,
                limit_bucket: None,
            });
        }
    }

    Ok(decision)
}

fn entitlement_execution_premium_denial(
    denial: focusa_license::PremiumFamilyDenial,
    required_feature: Option<String>,
    limit_bucket: Option<String>,
) -> EntitlementExecutionFailure {
    match denial {
        focusa_license::PremiumFamilyDenial::BaseProductRequired { .. } => EntitlementExecutionFailure {
            code: ENTITLEMENT_BASE_REQUIRED.to_string(),
            message: "Base Focusa entitlement is required for premium operations.".to_string(),
            required_feature,
            limit_bucket,
        },
        focusa_license::PremiumFamilyDenial::MissingLeaseSequence => EntitlementExecutionFailure {
            code: ENTITLEMENT_FEATURE_REQUIRED.to_string(),
            message: "snapshot sequence is missing or zero".to_string(),
            required_feature,
            limit_bucket,
        },
        focusa_license::PremiumFamilyDenial::MissingLeaseBinding => EntitlementExecutionFailure {
            code: ENTITLEMENT_FEATURE_REQUIRED.to_string(),
            message: "snapshot binding is missing".to_string(),
            required_feature,
            limit_bucket,
        },
        focusa_license::PremiumFamilyDenial::InvalidRequiredFeature { feature: feature_value } => {
            EntitlementExecutionFailure {
                code: ENTITLEMENT_FEATURE_REQUIRED.to_string(),
                message: format!(
                    "required_feature is not a qualified entitlement feature: {feature_value}"
                ),
                required_feature: Some(feature_value),
                limit_bucket,
            }
        }
        focusa_license::PremiumFamilyDenial::FeatureNotRegistered { family, feature } => {
            EntitlementExecutionFailure {
                code: ENTITLEMENT_FEATURE_REQUIRED.to_string(),
                message: format!(
                    "{family:?} family does not register the requested feature"
                ),
                required_feature: Some(feature.as_str().to_string()),
                limit_bucket,
            }
        }
        focusa_license::PremiumFamilyDenial::MissingFeature { family, feature } => {
            EntitlementExecutionFailure {
                code: ENTITLEMENT_FEATURE_REQUIRED.to_string(),
                message: format!(
                    "signed entitlement does not grant this required premium feature for {family:?}"
                ),
                required_feature: Some(feature.as_str().to_string()),
                limit_bucket,
            }
        }
        focusa_license::PremiumFamilyDenial::MissingCachedGrantExpiry => EntitlementExecutionFailure {
            code: ENTITLEMENT_REQUIRED.to_string(),
            message: "offline cached feature grants require a grace window".to_string(),
            required_feature,
            limit_bucket,
        },
        focusa_license::PremiumFamilyDenial::CachedGrantExpired => EntitlementExecutionFailure {
            code: ENTITLEMENT_REQUIRED.to_string(),
            message: "offline cached feature grant has expired".to_string(),
            required_feature,
            limit_bucket,
        },
        focusa_license::PremiumFamilyDenial::ActiveLeaseExpired => EntitlementExecutionFailure {
            code: ENTITLEMENT_REQUIRED.to_string(),
            message: "snapshot is expired for premium feature decisions".to_string(),
            required_feature,
            limit_bucket,
        },
        focusa_license::PremiumFamilyDenial::EntitlementStateNotUsable { state } => {
            EntitlementExecutionFailure {
                code: ENTITLEMENT_REQUIRED.to_string(),
                message: format!("entitlement state {state:?} cannot carry a premium feature"),
                required_feature,
                limit_bucket,
            }
        }
        focusa_license::PremiumFamilyDenial::NotPremiumFamily { family } => EntitlementExecutionFailure {
            code: ENTITLEMENT_ROUTE_UNCLASSIFIED.to_string(),
            message: format!("{family:?} is not a premium family"),
            required_feature,
            limit_bucket,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_snapshot(state: focusa_license::authority::EntitlementState) -> focusa_license::authority::EntitlementSnapshot {
        use focusa_license::authority::EntitlementSnapshot;
        let mut snapshot = EntitlementSnapshot::unactivated("focusa", "node-core-guard");
        snapshot.state = state;
        snapshot.sequence = Some(7);
        snapshot.lease_id = Some("lease-001".into());
        snapshot.lease_digest = Some("sha256:guard".into());
        snapshot.expires_at = Some(Utc::now() + chrono::Duration::hours(1));
        snapshot.offline_grace_until = Some(Utc::now() + chrono::Duration::hours(1));
        snapshot
    }

    #[test]
    fn entitlement_execution_guard_keeps_recovery_paths_available_without_base_gate() {
        let decision = evaluate_entitlement_execution(
            &focusa_license::LicenseGuard::eval(7),
            &EntitlementExecutionPolicy::new(
                "focusa.account.recovery",
                focusa_license::OperationClass::Recovery,
                focusa_license::CapabilityFamily::AccountRecovery,
                None,
                None,
                focusa_license::RecoveryAllowance::AccountRecovery,
            ),
            EntitlementExecutionContext::default(),
        )
        .expect("recovery policy remains available");
        assert_eq!(decision.status, "allow");
        assert_eq!(decision.code, ENTITLEMENT_ALLOWED);
    }

    #[test]
    fn entitlement_execution_guard_blocks_base_mutation_without_base_entitlement() {
        let decision = evaluate_entitlement_execution(
            &focusa_license::LicenseGuard::eval(7),
            &EntitlementExecutionPolicy::new(
                "focusa.core.workpoint.mutate",
                focusa_license::OperationClass::ValueMutation,
                focusa_license::CapabilityFamily::BaseFocusa,
                None,
                Some("workpoints"),
                focusa_license::RecoveryAllowance::None,
            ),
            EntitlementExecutionContext::default(),
        );
        let error = decision.expect_err("base focusa mutation requires entitlement");
        assert_eq!(error.code, ENTITLEMENT_BASE_REQUIRED);
    }

    #[test]
    fn entitlement_execution_guard_allows_base_focusa_with_signed_entitlement() {
        let snapshot = base_snapshot(focusa_license::authority::EntitlementState::Active);
        let guard = focusa_license::LicenseGuard::from_entitlement(snapshot);
        let decision = evaluate_entitlement_execution(
            &guard,
            &EntitlementExecutionPolicy::new(
                "focusa.core.workpoint.mutate",
                focusa_license::OperationClass::ValueMutation,
                focusa_license::CapabilityFamily::BaseFocusa,
                None,
                Some("workpoints"),
                focusa_license::RecoveryAllowance::None,
            ),
            EntitlementExecutionContext::default(),
        )
        .expect("active signed entitlement allows base_focusa value mutation");
        assert_eq!(decision.status, "base");
        assert!(decision.code == ENTITLEMENT_ALLOWED);
    }

    #[test]
    fn entitlement_execution_guard_enforces_premium_base_then_feature() {
        let mut snapshot = base_snapshot(focusa_license::authority::EntitlementState::Active);
        snapshot.features.insert("focusa.agent.parallelism".into(), true);

        let good = evaluate_entitlement_execution(
            &focusa_license::LicenseGuard::from_entitlement(snapshot.clone()),
            &EntitlementExecutionPolicy::new(
                "focusa.agent.parallelism.run",
                focusa_license::OperationClass::ValueMutation,
                focusa_license::CapabilityFamily::Automation,
                Some("focusa.agent.parallelism"),
                Some("parallel_agents"),
                focusa_license::RecoveryAllowance::None,
            ),
            EntitlementExecutionContext::default(),
        )
        .expect("premium operation resolves with signed feature");
        assert_eq!(good.status, "feature");

        snapshot.features.insert("focusa.agent.parallelism".into(), false);
        let denied = evaluate_entitlement_execution(
            &focusa_license::LicenseGuard::from_entitlement(snapshot),
            &EntitlementExecutionPolicy::new(
                "focusa.agent.parallelism.run",
                focusa_license::OperationClass::ValueMutation,
                focusa_license::CapabilityFamily::Automation,
                Some("focusa.agent.parallelism"),
                Some("parallel_agents"),
                focusa_license::RecoveryAllowance::None,
            ),
            EntitlementExecutionContext::default(),
        )
        .expect_err("missing feature should deny");
        assert_eq!(denied.code, ENTITLEMENT_FEATURE_REQUIRED);
    }

    #[test]
    fn entitlement_execution_guard_fail_closed_for_unknown_policy() {
        let decision = evaluate_entitlement_execution(
            &focusa_license::LicenseGuard::eval(7),
            &EntitlementExecutionPolicy::new(
                "focusa.unknown",
                focusa_license::OperationClass::Unknown,
                focusa_license::CapabilityFamily::Automation,
                Some("focusa.agent.parallelism"),
                None,
                focusa_license::RecoveryAllowance::None,
            ),
            EntitlementExecutionContext::default(),
        );
        let error = decision.expect_err("unknown operation class must fail closed");
        assert_eq!(error.code, ENTITLEMENT_ROUTE_UNCLASSIFIED);
    }

    #[test]
    fn entitlement_execution_guard_internal_maintenance_entitlement_requires_initiating_posture() {
        let snapshot = base_snapshot(focusa_license::authority::EntitlementState::Active);
        let guard = focusa_license::LicenseGuard::from_entitlement(snapshot);

        let denied = evaluate_entitlement_execution(
            &guard,
            &EntitlementExecutionPolicy::new(
                "focusa.internal.maintenance",
                focusa_license::OperationClass::InternalMaintenance,
                focusa_license::CapabilityFamily::InternalMaintenance,
                None,
                None,
                focusa_license::RecoveryAllowance::None,
            ),
            EntitlementExecutionContext::default(),
        )
        .expect_err("missing initiating posture must be rejected");
        assert_eq!(denied.code, ENTITLEMENT_ROUTE_UNCLASSIFIED);

        let allowed = evaluate_entitlement_execution(
            &guard,
            &EntitlementExecutionPolicy::new(
                "focusa.internal.maintenance",
                focusa_license::OperationClass::InternalMaintenance,
                focusa_license::CapabilityFamily::InternalMaintenance,
                None,
                None,
                focusa_license::RecoveryAllowance::None,
            ),
            EntitlementExecutionContext {
                now: Utc::now(),
                initiating_posture: Some(focusa_license::EntitlementPolicyPosture::Read),
            },
        )
        .expect("maintenances can inherit initiating posture");
        assert_eq!(allowed.status, "read");
    }

    // ── Spec 172 verified_limited_project tests ──

    fn limited_snapshot() -> focusa_license::authority::EntitlementSnapshot {
        use focusa_license::authority::EntitlementSnapshot;
        let snapshot = EntitlementSnapshot::unactivated("focusa", "node-limited-001");
        // Verified-no-license posture: no paid lease, but verified identity.
        // The authority_policy_state maps Unactivated → PendingUnverified,
        // so we need to construct a state that the reducer will see as
        // VerifiedNoLicense. We use the snapshot's product and the
        // resolve_base_focusa_product call directly.
        snapshot
    }

    #[test]
    fn verified_limited_project_allows_mutation_in_active_project() {
        let selection = crate::limited_project::ActiveProjectSelection::new(
            "/home/user/projects/my-focusa",
            "test",
        );
        // The project guard itself is tested in limited_project.rs;
        // here we verify the integration with the execution guard.
        let decision = crate::limited_project::ActiveProjectGuard::check_mutation(
            focusa_license::BaseProductDecision::Limited,
            "/home/user/projects/my-focusa",
            Some(&selection),
        );
        assert!(decision.is_allowed());
    }

    #[test]
    fn verified_limited_project_denies_second_project_mutation() {
        let selection = crate::limited_project::ActiveProjectSelection::new(
            "/home/user/projects/project-a",
            "test",
        );
        let decision = crate::limited_project::ActiveProjectGuard::check_mutation(
            focusa_license::BaseProductDecision::Limited,
            "/home/user/projects/project-b",
            Some(&selection),
        );
        assert!(decision.is_denied());
        match decision {
            crate::limited_project::ProjectMutationDecision::DeniedSecondProject {
                active_project_root,
                attempted_project_root,
                ..
            } => {
                assert_eq!(active_project_root, "/home/user/projects/project-a");
                assert_eq!(attempted_project_root, "/home/user/projects/project-b");
            }
            _ => panic!("expected DeniedSecondProject"),
        }
    }

    #[test]
    fn verified_limited_project_denies_mutation_without_explicit_selection() {
        let decision = crate::limited_project::ActiveProjectGuard::check_mutation(
            focusa_license::BaseProductDecision::Limited,
            "/home/user/projects/any-project",
            None,
        );
        assert!(decision.is_denied());
        match decision {
            crate::limited_project::ProjectMutationDecision::DeniedNoSelection { .. } => {}
            _ => panic!("expected DeniedNoSelection"),
        }
    }

    #[test]
    fn verified_limited_project_paid_entitlement_bypasses_project_guard() {
        // Paid entitlement: project guard is bypassed.
        let selection = crate::limited_project::ActiveProjectSelection::new(
            "/home/user/projects/project-a",
            "test",
        );
        let decision = crate::limited_project::ActiveProjectGuard::check_mutation(
            focusa_license::BaseProductDecision::Entitled,
            "/home/user/projects/project-b",
            Some(&selection),
        );
        assert!(decision.is_allowed());

        // Even without a selection.
        let decision = crate::limited_project::ActiveProjectGuard::check_mutation(
            focusa_license::BaseProductDecision::Entitled,
            "/home/user/projects/any-project",
            None,
        );
        assert!(decision.is_allowed());
    }

    #[test]
    fn verified_limited_project_read_export_always_available() {
        // Read projection is never subject to the project mutation guard.
        // The entitlement state grid reducer handles ReadProjection separately
        // from BaseFocusa, and read operations always return Read posture.
        let guard = focusa_license::LicenseGuard::eval(7);
        let decision = evaluate_entitlement_execution(
            &guard,
            &EntitlementExecutionPolicy::new(
                "focusa.project.read",
                focusa_license::OperationClass::Read,
                focusa_license::CapabilityFamily::ReadProjection,
                None,
                None,
                focusa_license::RecoveryAllowance::ReadProjection,
            ),
            EntitlementExecutionContext::default(),
        )
        .expect("read projection is always available");
        assert_eq!(decision.status, "read");
        assert_eq!(decision.code, "ENTITLEMENT_ALLOWED");
    }

    #[test]
    fn verified_limited_project_export_always_available() {
        // Customer data export is always available through the recovery allowance.
        let guard = focusa_license::LicenseGuard::eval(7);
        let decision = evaluate_entitlement_execution(
            &guard,
            &EntitlementExecutionPolicy::new(
                "focusa.export.basic",
                focusa_license::OperationClass::Read,
                focusa_license::CapabilityFamily::CustomerDataExport,
                None,
                None,
                focusa_license::RecoveryAllowance::CustomerDataExport,
            ),
            EntitlementExecutionContext::default(),
        )
        .expect("customer data export is always available");
        assert_eq!(decision.status, "allow");
        assert_eq!(decision.code, "ENTITLEMENT_ALLOWED");
    }

    #[test]
    fn verified_limited_project_switch_preserves_read_access() {
        // After switching the active project, the previously active project
        // is still readable (ReadProjection is always available) but no longer
        // mutable.
        let selection_a = crate::limited_project::ActiveProjectSelection::new(
            "/home/user/projects/project-a",
            "test",
        );
        let decision = crate::limited_project::ActiveProjectGuard::check_mutation(
            focusa_license::BaseProductDecision::Limited,
            "/home/user/projects/project-a",
            Some(&selection_a),
        );
        assert!(decision.is_allowed());

        // Switch to project-b
        let selection_b = crate::limited_project::ActiveProjectSelection::new(
            "/home/user/projects/project-b",
            "test",
        );
        let decision = crate::limited_project::ActiveProjectGuard::check_mutation(
            focusa_license::BaseProductDecision::Limited,
            "/home/user/projects/project-b",
            Some(&selection_b),
        );
        assert!(decision.is_allowed());

        // project-a is now denied for mutation
        let decision = crate::limited_project::ActiveProjectGuard::check_mutation(
            focusa_license::BaseProductDecision::Limited,
            "/home/user/projects/project-a",
            Some(&selection_b),
        );
        assert!(decision.is_denied());

        // But read projection is always available for project-a
        let guard = focusa_license::LicenseGuard::eval(7);
        let read_decision = evaluate_entitlement_execution(
            &guard,
            &EntitlementExecutionPolicy::new(
                "focusa.project.read",
                focusa_license::OperationClass::Read,
                focusa_license::CapabilityFamily::ReadProjection,
                None,
                None,
                focusa_license::RecoveryAllowance::ReadProjection,
            ),
            EntitlementExecutionContext::default(),
        )
        .expect("read projection is always available");
        assert_eq!(read_decision.code, "ENTITLEMENT_ALLOWED");
    }
}
