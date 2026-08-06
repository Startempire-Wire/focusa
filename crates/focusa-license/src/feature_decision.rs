use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{EntitlementPolicyTypeError, LimitBucket, RequiredFeature};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeatureOperationClass {
    Read,
    Write,
    Execute,
    Export,
    Admin,
    Update,
    Install,
    Recovery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeatureRecoveryPosture {
    AlwaysAvailable,
    EntitlementRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeatureDiscoverability {
    Visible,
    Advanced,
    Internal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductFeatureDefinition {
    pub key: String,
    pub product: String,
    pub operation_class: FeatureOperationClass,
    pub recovery_posture: FeatureRecoveryPosture,
    pub limit_bucket: Option<String>,
    pub limit_unit: Option<String>,
    pub discoverability: FeatureDiscoverability,
    pub owner: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductFeatureRegistry {
    pub schema: String,
    pub product: String,
    pub features: Vec<ProductFeatureDefinition>,
}

impl ProductFeatureDefinition {
    pub fn required_feature(&self) -> Result<RequiredFeature, EntitlementPolicyTypeError> {
        RequiredFeature::new(self.key.as_str())
    }

    pub fn typed_limit_bucket(&self) -> Result<Option<LimitBucket>, EntitlementPolicyTypeError> {
        self.limit_bucket
            .as_deref()
            .map(LimitBucket::new)
            .transpose()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FeatureDecision {
    Granted { reserved_units: u64 },
    RecoveryAllowed,
    Denied(FeatureDecisionDenial),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeatureDecisionDenial {
    UnknownFeature,
    WrongProduct,
    FeatureNotGranted,
    LimitNotGranted,
    LimitExhausted,
    InvalidRegistry,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum FeatureRegistryError {
    #[error("unsupported feature registry schema or product")]
    UnsupportedIdentity,
    #[error("feature registry contains duplicate, unqualified, or inconsistent entries")]
    InvalidFeature,
}

impl ProductFeatureRegistry {
    pub fn validate(&self) -> Result<(), FeatureRegistryError> {
        if self.schema != "focusa.feature_registry.v1" || self.product != "focusa" {
            return Err(FeatureRegistryError::UnsupportedIdentity);
        }
        let mut seen = BTreeSet::new();
        for feature in &self.features {
            if feature.product != self.product
                || !feature.key.starts_with(&format!("{}.", feature.product))
                || feature.owner.trim().is_empty()
                || !seen.insert(feature.key.as_str())
                || feature.limit_bucket.is_some() != feature.limit_unit.is_some()
                || feature.required_feature().is_err()
                || feature.typed_limit_bucket().is_err()
            {
                return Err(FeatureRegistryError::InvalidFeature);
            }
        }
        Ok(())
    }

    pub fn decide(
        &self,
        product: &str,
        feature_key: &str,
        requested_units: u64,
        granted_features: &BTreeSet<String>,
        remaining_limits: &BTreeMap<String, u64>,
    ) -> FeatureDecision {
        if self.validate().is_err() {
            return FeatureDecision::Denied(FeatureDecisionDenial::InvalidRegistry);
        }
        let Some(feature) = self
            .features
            .iter()
            .find(|feature| feature.key == feature_key)
        else {
            return FeatureDecision::Denied(FeatureDecisionDenial::UnknownFeature);
        };
        if product != self.product || feature.product != product {
            return FeatureDecision::Denied(FeatureDecisionDenial::WrongProduct);
        }
        if feature.recovery_posture == FeatureRecoveryPosture::AlwaysAvailable {
            return FeatureDecision::RecoveryAllowed;
        }
        if !granted_features.contains(feature_key) {
            return FeatureDecision::Denied(FeatureDecisionDenial::FeatureNotGranted);
        }
        let Some(bucket) = feature.limit_bucket.as_ref() else {
            return FeatureDecision::Granted { reserved_units: 0 };
        };
        let Some(remaining) = remaining_limits.get(bucket) else {
            return FeatureDecision::Denied(FeatureDecisionDenial::LimitNotGranted);
        };
        if requested_units == 0 || requested_units > *remaining {
            return FeatureDecision::Denied(FeatureDecisionDenial::LimitExhausted);
        }
        FeatureDecision::Granted {
            reserved_units: requested_units,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn registry() -> ProductFeatureRegistry {
        ProductFeatureRegistry {
            schema: "focusa.feature_registry.v1".into(),
            product: "focusa".into(),
            features: vec![
                ProductFeatureDefinition {
                    key: "focusa.core.mission".into(),
                    product: "focusa".into(),
                    operation_class: FeatureOperationClass::Write,
                    recovery_posture: FeatureRecoveryPosture::EntitlementRequired,
                    limit_bucket: Some("missions".into()),
                    limit_unit: Some("mission".into()),
                    discoverability: FeatureDiscoverability::Visible,
                    owner: "focusa-core".into(),
                },
                ProductFeatureDefinition {
                    key: "focusa.repair.execute".into(),
                    product: "focusa".into(),
                    operation_class: FeatureOperationClass::Recovery,
                    recovery_posture: FeatureRecoveryPosture::AlwaysAvailable,
                    limit_bucket: None,
                    limit_unit: None,
                    discoverability: FeatureDiscoverability::Advanced,
                    owner: "focusa-core".into(),
                },
            ],
        }
    }

    #[test]
    fn unknown_and_ungranted_features_fail_closed_while_recovery_survives() {
        let registry = registry();
        assert_eq!(
            registry.decide(
                "focusa",
                "focusa.unknown",
                1,
                &BTreeSet::new(),
                &BTreeMap::new()
            ),
            FeatureDecision::Denied(FeatureDecisionDenial::UnknownFeature)
        );
        assert_eq!(
            registry.decide(
                "focusa",
                "focusa.core.mission",
                1,
                &BTreeSet::new(),
                &BTreeMap::new()
            ),
            FeatureDecision::Denied(FeatureDecisionDenial::FeatureNotGranted)
        );
        assert_eq!(
            registry.decide(
                "focusa",
                "focusa.repair.execute",
                1,
                &BTreeSet::new(),
                &BTreeMap::new()
            ),
            FeatureDecision::RecoveryAllowed
        );
    }

    #[test]
    fn signed_limit_bucket_is_required_and_bounded() {
        let features = BTreeSet::from(["focusa.core.mission".into()]);
        assert_eq!(
            registry().decide(
                "focusa",
                "focusa.core.mission",
                1,
                &features,
                &BTreeMap::new()
            ),
            FeatureDecision::Denied(FeatureDecisionDenial::LimitNotGranted)
        );
        let limits = BTreeMap::from([("missions".into(), 1)]);
        assert_eq!(
            registry().decide("focusa", "focusa.core.mission", 2, &features, &limits),
            FeatureDecision::Denied(FeatureDecisionDenial::LimitExhausted)
        );
        assert_eq!(
            registry().decide("focusa", "focusa.core.mission", 1, &features, &limits),
            FeatureDecision::Granted { reserved_units: 1 }
        );
    }
}
