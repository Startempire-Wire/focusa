//! Spec137 trusted-clock, precision, suspend/reboot, and holdover authority policy.

use crate::temporal::{TemporalClockDomain, TemporalClockSample, TemporalConfidence};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClockSynchronizationStatus {
    Synchronized,
    Holdover,
    Acquiring,
    Disagreeing,
    Unauthenticated,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClockAuthenticationPolicy {
    Nts,
    AuthenticatedProvider,
    PrivateDisciplinedSource,
    NotRequiredByProfile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClockDisagreementAction {
    Block,
    Degrade,
    QuarantineSource,
    OperatorReview,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemporalDomainClockPolicy {
    pub policy_id: String,
    pub domain: TemporalClockDomain,
    pub suspend_consumes_interval: bool,
    pub reboot_consumes_interval: bool,
    pub allowed_clock_domains: Vec<TemporalClockDomain>,
    pub max_sample_age_ms: u64,
    pub max_measurement_uncertainty_ns: u128,
    pub on_unavailable: ClockDisagreementAction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClockTrustProfile {
    pub profile_id: String,
    pub required_source_count: u32,
    pub required_independent_source_count: u32,
    pub required_authentication: ClockAuthenticationPolicy,
    pub disagreement_threshold_ns: u128,
    pub max_sync_age_ms: u64,
    pub max_holdover_ms: u64,
    pub max_offset_ns: u128,
    pub max_root_distance_ns: u128,
    pub on_disagreement: ClockDisagreementAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClockSourceStatus {
    Healthy,
    Quarantined,
    Recovering,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClockSourceObservation {
    pub source_id: String,
    pub diversity_class: String,
    pub authenticated: bool,
    pub replay_protected: bool,
    pub request_response_bound: bool,
    pub observed_at: DateTime<Utc>,
    pub synchronization_age_ms: u64,
    pub offset_ns: i128,
    pub delay_ns: u128,
    pub jitter_ns: u128,
    pub dispersion_ns: u128,
    pub root_distance_ns: u128,
    pub frequency_error_ppb: i64,
    pub status: ClockSourceStatus,
    pub quarantine_reason: Option<String>,
    pub recovery_evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClockSourceTrustError {
    InsufficientHealthySources,
    InsufficientDiversity,
    AuthenticationMissing,
    ReplayProtectionMissing,
    RequestResponseBindingMissing,
    StaleSource,
    SourceQuarantineUnsettled,
}

pub fn evaluate_clock_sources(
    profile: &ClockTrustProfile,
    sources: &[ClockSourceObservation],
) -> Result<(), ClockSourceTrustError> {
    let healthy = sources
        .iter()
        .filter(|source| source.status == ClockSourceStatus::Healthy)
        .collect::<Vec<_>>();
    if healthy.len() < profile.required_source_count as usize {
        return Err(ClockSourceTrustError::InsufficientHealthySources);
    }
    let diversity = healthy
        .iter()
        .map(|source| source.diversity_class.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    if diversity.len() < profile.required_independent_source_count as usize {
        return Err(ClockSourceTrustError::InsufficientDiversity);
    }
    if profile.required_authentication != ClockAuthenticationPolicy::NotRequiredByProfile
        && healthy.iter().any(|source| !source.authenticated)
    {
        return Err(ClockSourceTrustError::AuthenticationMissing);
    }
    if healthy.iter().any(|source| !source.replay_protected) {
        return Err(ClockSourceTrustError::ReplayProtectionMissing);
    }
    if healthy.iter().any(|source| !source.request_response_bound) {
        return Err(ClockSourceTrustError::RequestResponseBindingMissing);
    }
    if healthy
        .iter()
        .any(|source| source.synchronization_age_ms > profile.max_sync_age_ms)
    {
        return Err(ClockSourceTrustError::StaleSource);
    }
    if sources.iter().any(|source| {
        source.status != ClockSourceStatus::Healthy
            && (source.quarantine_reason.is_none() || source.recovery_evidence_refs.is_empty())
    }) {
        return Err(ClockSourceTrustError::SourceQuarantineUnsettled);
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemporalAuthority {
    pub authority_id: String,
    pub host_id: String,
    pub operator_timezone: String,
    pub tzdb_version: Option<String>,
    pub boot_id: String,
    pub synchronization_status: ClockSynchronizationStatus,
    pub observed_offset_ns: Option<i128>,
    pub observed_jitter_ns: Option<u128>,
    pub observed_root_distance_ns: Option<u128>,
    pub source_count: u32,
    pub independent_source_count: u32,
    pub sources_authenticated: bool,
    pub holdover_started_at: Option<DateTime<Utc>>,
    pub last_sample_at: Option<DateTime<Utc>>,
    pub measurement_uncertainty_ns: u128,
    pub confidence: TemporalConfidence,
    pub schema_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClockAuthorityError {
    WrongDomain,
    StaleSample,
    ExcessiveUncertainty,
    InsufficientSources,
    InsufficientSourceDiversity,
    AuthenticationRequired,
    OffsetExceedsPolicy,
    RootDistanceExceedsPolicy,
    HoldoverExpired,
    Disagreement,
    Unavailable,
}

pub fn evaluate_clock_sample(
    authority: &TemporalAuthority,
    profile: &ClockTrustProfile,
    domain: &TemporalDomainClockPolicy,
    sample: &TemporalClockSample,
    now: DateTime<Utc>,
) -> Result<TemporalConfidence, ClockAuthorityError> {
    if sample.domain != domain.domain || !domain.allowed_clock_domains.contains(&sample.domain) {
        return Err(ClockAuthorityError::WrongDomain);
    }
    let sample_age = now
        .signed_duration_since(sample.wall_utc)
        .num_milliseconds()
        .max(0) as u64;
    if sample_age > domain.max_sample_age_ms.min(profile.max_sync_age_ms) {
        return Err(ClockAuthorityError::StaleSample);
    }
    if sample.measurement_uncertainty_ns > domain.max_measurement_uncertainty_ns {
        return Err(ClockAuthorityError::ExcessiveUncertainty);
    }
    if authority.source_count < profile.required_source_count {
        return Err(ClockAuthorityError::InsufficientSources);
    }
    if authority.independent_source_count < profile.required_independent_source_count {
        return Err(ClockAuthorityError::InsufficientSourceDiversity);
    }
    if profile.required_authentication != ClockAuthenticationPolicy::NotRequiredByProfile
        && !authority.sources_authenticated
    {
        return Err(ClockAuthorityError::AuthenticationRequired);
    }
    if authority
        .observed_offset_ns
        .is_some_and(|offset| offset.unsigned_abs() > profile.max_offset_ns)
    {
        return Err(ClockAuthorityError::OffsetExceedsPolicy);
    }
    if authority
        .observed_root_distance_ns
        .is_some_and(|distance| distance > profile.max_root_distance_ns)
    {
        return Err(ClockAuthorityError::RootDistanceExceedsPolicy);
    }
    match authority.synchronization_status {
        ClockSynchronizationStatus::Synchronized => Ok(TemporalConfidence::Verified),
        ClockSynchronizationStatus::Holdover => {
            let age = authority
                .holdover_started_at
                .map(|started| now.signed_duration_since(started).num_milliseconds().max(0) as u64)
                .unwrap_or(u64::MAX);
            if age > profile.max_holdover_ms {
                Err(ClockAuthorityError::HoldoverExpired)
            } else {
                Ok(TemporalConfidence::Medium)
            }
        }
        ClockSynchronizationStatus::Disagreeing => Err(ClockAuthorityError::Disagreement),
        ClockSynchronizationStatus::Unauthenticated => {
            Err(ClockAuthorityError::AuthenticationRequired)
        }
        ClockSynchronizationStatus::Acquiring | ClockSynchronizationStatus::Unavailable => {
            Err(ClockAuthorityError::Unavailable)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> (
        TemporalAuthority,
        ClockTrustProfile,
        TemporalDomainClockPolicy,
        TemporalClockSample,
    ) {
        let now = Utc::now();
        (
            TemporalAuthority {
                authority_id: "authority".into(),
                host_id: "host".into(),
                operator_timezone: "America/Los_Angeles".into(),
                tzdb_version: Some("2026a".into()),
                boot_id: "boot".into(),
                synchronization_status: ClockSynchronizationStatus::Synchronized,
                observed_offset_ns: Some(1),
                observed_jitter_ns: Some(1),
                observed_root_distance_ns: Some(2),
                source_count: 3,
                independent_source_count: 2,
                sources_authenticated: true,
                holdover_started_at: None,
                last_sample_at: Some(now),
                measurement_uncertainty_ns: 5,
                confidence: TemporalConfidence::Verified,
                schema_version: "focusa.temporal_authority.v1".into(),
            },
            ClockTrustProfile {
                profile_id: "trusted".into(),
                required_source_count: 2,
                required_independent_source_count: 2,
                required_authentication: ClockAuthenticationPolicy::Nts,
                disagreement_threshold_ns: 100,
                max_sync_age_ms: 5_000,
                max_holdover_ms: 60_000,
                max_offset_ns: 100,
                max_root_distance_ns: 1_000,
                on_disagreement: ClockDisagreementAction::Block,
            },
            TemporalDomainClockPolicy {
                policy_id: "external-deadline".into(),
                domain: TemporalClockDomain::WallUtc,
                suspend_consumes_interval: true,
                reboot_consumes_interval: true,
                allowed_clock_domains: vec![TemporalClockDomain::WallUtc],
                max_sample_age_ms: 5_000,
                max_measurement_uncertainty_ns: 100,
                on_unavailable: ClockDisagreementAction::Block,
            },
            TemporalClockSample {
                sample_id: "sample".into(),
                domain: TemporalClockDomain::WallUtc,
                wall_utc: now,
                monotonic_ns: None,
                suspend_aware_ns: None,
                boot_id: Some("boot".into()),
                timezone: "America/Los_Angeles".into(),
                tzdb_version: Some("2026a".into()),
                source: "nts".into(),
                observed_offset_ns: Some(1),
                measurement_uncertainty_ns: 5,
                confidence: TemporalConfidence::Verified,
            },
        )
    }

    #[test]
    fn synchronized_diverse_authenticated_clock_is_verified() {
        let (authority, profile, domain, sample) = fixture();
        assert_eq!(
            evaluate_clock_sample(&authority, &profile, &domain, &sample, sample.wall_utc),
            Ok(TemporalConfidence::Verified)
        );
    }

    #[test]
    fn disagreement_and_expired_holdover_fail_closed() {
        let (mut authority, profile, domain, sample) = fixture();
        authority.synchronization_status = ClockSynchronizationStatus::Disagreeing;
        assert_eq!(
            evaluate_clock_sample(&authority, &profile, &domain, &sample, sample.wall_utc),
            Err(ClockAuthorityError::Disagreement)
        );
        authority.synchronization_status = ClockSynchronizationStatus::Holdover;
        authority.holdover_started_at = Some(sample.wall_utc - chrono::Duration::minutes(2));
        assert_eq!(
            evaluate_clock_sample(&authority, &profile, &domain, &sample, sample.wall_utc),
            Err(ClockAuthorityError::HoldoverExpired)
        );
    }

    fn source(id: &str, diversity: &str, status: ClockSourceStatus) -> ClockSourceObservation {
        ClockSourceObservation {
            source_id: id.into(),
            diversity_class: diversity.into(),
            authenticated: true,
            replay_protected: true,
            request_response_bound: true,
            observed_at: Utc::now(),
            synchronization_age_ms: 10,
            offset_ns: 1,
            delay_ns: 2,
            jitter_ns: 1,
            dispersion_ns: 1,
            root_distance_ns: 4,
            frequency_error_ppb: 1,
            status,
            quarantine_reason: None,
            recovery_evidence_refs: Vec::new(),
        }
    }

    #[test]
    fn independent_authenticated_replay_protected_sources_are_trusted() {
        let (_, profile, _, _) = fixture();
        let sources = vec![
            source("nts-a", "provider-a", ClockSourceStatus::Healthy),
            source("nts-b", "provider-b", ClockSourceStatus::Healthy),
        ];
        assert_eq!(evaluate_clock_sources(&profile, &sources), Ok(()));
    }

    #[test]
    fn quarantine_and_recovery_require_explicit_evidence() {
        let (_, profile, _, _) = fixture();
        let mut sources = vec![
            source("nts-a", "provider-a", ClockSourceStatus::Healthy),
            source("nts-b", "provider-b", ClockSourceStatus::Healthy),
            source("nts-c", "provider-c", ClockSourceStatus::Quarantined),
        ];
        assert_eq!(
            evaluate_clock_sources(&profile, &sources),
            Err(ClockSourceTrustError::SourceQuarantineUnsettled)
        );
        sources[2].quarantine_reason = Some("offset disagreement".into());
        sources[2].recovery_evidence_refs = vec!["receipt:clock-source-quarantine".into()];
        assert_eq!(evaluate_clock_sources(&profile, &sources), Ok(()));
    }
}
