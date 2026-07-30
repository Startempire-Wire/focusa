//! Spec138 profile activation and full-conformance truth gate.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PredictionProfile {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProfileStatus {
    VerifiedComplete,
    InactiveEvidenceBacked,
    UnsupportedOpen,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileConformanceRecord {
    pub profile: PredictionProfile,
    pub label: String,
    pub status: ProfileStatus,
    pub capability_refs: Vec<String>,
    pub runtime_refs: Vec<String>,
    pub test_refs: Vec<String>,
    pub evidence_refs: Vec<String>,
    pub receipt_refs: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConformanceClaim {
    VerifiedSubset,
    FullSpec138Conformance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileConformanceRequest {
    pub claim: ConformanceClaim,
    pub subset_label: Option<String>,
    pub profiles: Vec<ProfileConformanceRecord>,
    pub required_scorer_count: usize,
    pub durable_append_only_history: bool,
    pub migration_verified: bool,
    pub client_parity_verified: bool,
    pub security_verified: bool,
    pub evidence_refs: Vec<String>,
    pub receipt_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileConformanceError {
    DuplicateProfile,
    MissingProfile(PredictionProfile),
    UnknownProfile(PredictionProfile),
    MissingProof(PredictionProfile),
    FullProfileNotVerified(PredictionProfile),
    ScorerRegistryIncomplete,
    DurableHistoryRequired,
    MigrationRequired,
    ClientParityRequired,
    SecurityRequired,
    SubsetLabelRequired,
    MissingEvidence,
    MissingReceipt,
}

pub fn validate_profile_conformance(
    request: &ProfileConformanceRequest,
) -> Result<(), ProfileConformanceError> {
    let mut profiles = BTreeSet::new();
    for record in &request.profiles {
        if !profiles.insert(record.profile) {
            return Err(ProfileConformanceError::DuplicateProfile);
        }
        if matches!(record.status, ProfileStatus::Unknown) {
            return Err(ProfileConformanceError::UnknownProfile(record.profile));
        }
        if record.capability_refs.is_empty()
            || record.runtime_refs.is_empty()
            || record.test_refs.is_empty()
            || record.evidence_refs.is_empty()
            || record.receipt_refs.is_empty()
        {
            return Err(ProfileConformanceError::MissingProof(record.profile));
        }
    }
    if request.evidence_refs.is_empty() {
        return Err(ProfileConformanceError::MissingEvidence);
    }
    if request.receipt_ref.trim().is_empty() {
        return Err(ProfileConformanceError::MissingReceipt);
    }
    match request.claim {
        ConformanceClaim::VerifiedSubset => {
            if request.subset_label.as_deref().is_none_or(str::is_empty) {
                return Err(ProfileConformanceError::SubsetLabelRequired);
            }
        }
        ConformanceClaim::FullSpec138Conformance => {
            for profile in [
                PredictionProfile::A,
                PredictionProfile::B,
                PredictionProfile::C,
                PredictionProfile::D,
                PredictionProfile::E,
                PredictionProfile::F,
                PredictionProfile::G,
                PredictionProfile::H,
            ] {
                let record = request
                    .profiles
                    .iter()
                    .find(|record| record.profile == profile)
                    .ok_or(ProfileConformanceError::MissingProfile(profile))?;
                if !matches!(record.status, ProfileStatus::VerifiedComplete) {
                    return Err(ProfileConformanceError::FullProfileNotVerified(profile));
                }
            }
            if request.required_scorer_count != 31 {
                return Err(ProfileConformanceError::ScorerRegistryIncomplete);
            }
            if !request.durable_append_only_history {
                return Err(ProfileConformanceError::DurableHistoryRequired);
            }
            if !request.migration_verified {
                return Err(ProfileConformanceError::MigrationRequired);
            }
            if !request.client_parity_verified {
                return Err(ProfileConformanceError::ClientParityRequired);
            }
            if !request.security_verified {
                return Err(ProfileConformanceError::SecurityRequired);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn record(profile: PredictionProfile) -> ProfileConformanceRecord {
        ProfileConformanceRecord {
            profile,
            label: format!("Profile {profile:?}"),
            status: ProfileStatus::VerifiedComplete,
            capability_refs: vec!["capability".into()],
            runtime_refs: vec!["runtime".into()],
            test_refs: vec!["test".into()],
            evidence_refs: vec!["evidence".into()],
            receipt_refs: vec!["receipt".into()],
        }
    }
    fn full() -> ProfileConformanceRequest {
        ProfileConformanceRequest {
            claim: ConformanceClaim::FullSpec138Conformance,
            subset_label: None,
            profiles: [
                PredictionProfile::A,
                PredictionProfile::B,
                PredictionProfile::C,
                PredictionProfile::D,
                PredictionProfile::E,
                PredictionProfile::F,
                PredictionProfile::G,
                PredictionProfile::H,
            ]
            .into_iter()
            .map(record)
            .collect(),
            required_scorer_count: 31,
            durable_append_only_history: true,
            migration_verified: true,
            client_parity_verified: true,
            security_verified: true,
            evidence_refs: vec!["evidence:full".into()],
            receipt_ref: "receipt:full".into(),
        }
    }
    #[test]
    fn full_conformance_requires_all_eight_profiles_and_cross_cutting_gates() {
        assert!(validate_profile_conformance(&full()).is_ok());
        let mut missing = full();
        missing.profiles.pop();
        assert_eq!(
            validate_profile_conformance(&missing),
            Err(ProfileConformanceError::MissingProfile(
                PredictionProfile::H
            ))
        );
        let mut no_migration = full();
        no_migration.migration_verified = false;
        assert_eq!(
            validate_profile_conformance(&no_migration),
            Err(ProfileConformanceError::MigrationRequired)
        );
    }
    #[test]
    fn profile_subsets_require_truthful_labels() {
        let mut subset = full();
        subset.claim = ConformanceClaim::VerifiedSubset;
        subset.profiles.truncate(2);
        subset.subset_label = None;
        assert_eq!(
            validate_profile_conformance(&subset),
            Err(ProfileConformanceError::SubsetLabelRequired)
        );
        subset.subset_label = Some("Profiles A-B verified subset".into());
        assert!(validate_profile_conformance(&subset).is_ok());
    }
}
