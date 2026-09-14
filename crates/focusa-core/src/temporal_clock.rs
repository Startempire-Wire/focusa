use chrono::{DateTime, Offset, SecondsFormat, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
#[cfg(not(unix))]
use std::{
    sync::OnceLock,
    time::{Instant, SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

use crate::temporal::{TemporalClockDomain, TemporalClockSample, TemporalConfidence};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemporalActionEnvelope {
    pub schema_version: String,
    pub envelope_id: String,
    pub action_id: String,
    pub prediction_id: Option<String>,
    pub captured_at_utc: DateTime<Utc>,
    pub utc_unix_ns: i128,
    pub operator_timezone: String,
    pub operator_timezone_source: String,
    pub operator_local_rfc3339: String,
    pub operator_utc_offset_seconds: i32,
    pub monotonic_ns: u128,
    pub realtime_resolution_ns: u64,
    pub monotonic_resolution_ns: u64,
    pub capture_latency_ns: u128,
    pub capture_uncertainty_ns: u128,
    pub wall_clock_accuracy_uncertainty_ns: Option<u128>,
    pub microsecond_representation_supported: bool,
    pub microsecond_wall_clock_accuracy_verified: bool,
    pub clock_source: String,
    pub synchronization_source: Option<String>,
    pub boot_id: Option<String>,
    pub process_id: u32,
    pub thread_id: String,
    pub confidence: TemporalConfidence,
    pub capture_failure: Option<String>,
    pub calibration_lineage: Vec<String>,
    pub clock_sample: TemporalClockSample,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemporalActionCaptureError {
    InvalidOperatorTimezone(String),
    RealtimeClockUnavailable,
    MonotonicClockUnavailable,
    RealtimeResolutionUnavailable,
    MonotonicResolutionUnavailable,
    InvalidRealtimeSample,
}

impl TemporalActionEnvelope {
    pub fn unavailable(reason: impl Into<String>) -> Self {
        let epoch = DateTime::<Utc>::from_timestamp(0, 0).expect("Unix epoch is valid");
        Self {
            schema_version: "focusa.temporal_action_envelope.v1".to_string(),
            envelope_id: "temporal:unavailable".to_string(),
            action_id: "action:unavailable".to_string(),
            prediction_id: None,
            captured_at_utc: epoch,
            utc_unix_ns: 0,
            operator_timezone: "unknown".to_string(),
            operator_timezone_source: "unavailable".to_string(),
            operator_local_rfc3339: "unavailable".to_string(),
            operator_utc_offset_seconds: 0,
            monotonic_ns: 0,
            realtime_resolution_ns: 0,
            monotonic_resolution_ns: 0,
            capture_latency_ns: 0,
            capture_uncertainty_ns: 0,
            wall_clock_accuracy_uncertainty_ns: None,
            microsecond_representation_supported: false,
            microsecond_wall_clock_accuracy_verified: false,
            clock_source: "unavailable".to_string(),
            synchronization_source: None,
            boot_id: None,
            process_id: 0,
            thread_id: "unavailable".to_string(),
            confidence: TemporalConfidence::Unavailable,
            capture_failure: Some(reason.into()),
            calibration_lineage: vec![],
            clock_sample: TemporalClockSample {
                sample_id: "temporal-sample:unavailable".to_string(),
                domain: TemporalClockDomain::WallUtc,
                wall_utc: epoch,
                monotonic_ns: None,
                suspend_aware_ns: None,
                boot_id: None,
                timezone: "unknown".to_string(),
                tzdb_version: None,
                source: "unavailable".to_string(),
                observed_offset_ns: None,
                measurement_uncertainty_ns: u64::MAX as u128,
                confidence: TemporalConfidence::Unavailable,
            },
        }
    }
}

impl Default for TemporalActionEnvelope {
    fn default() -> Self {
        Self::unavailable("legacy_event_missing_temporal_action_envelope")
    }
}

pub fn capture_operator_temporal_action_envelope() -> TemporalActionEnvelope {
    capture_operator_temporal_action_envelope_from_values(
        std::env::var("FOCUSA_OPERATOR_TIMEZONE").ok().as_deref(),
        std::env::var("TZ").ok().as_deref(),
    )
}

fn capture_operator_temporal_action_envelope_from_values(
    configured_timezone: Option<&str>,
    process_timezone: Option<&str>,
) -> TemporalActionEnvelope {
    let (timezone, source, timezone_known) = if let Some(value) = configured_timezone {
        (value, "FOCUSA_OPERATOR_TIMEZONE", true)
    } else if let Some(value) = process_timezone {
        (value, "TZ", true)
    } else {
        // Operator-local rendering is unavailable, but wall-clock capture is not.
        // Capture against UTC and preserve that current timestamp rather than
        // replacing a successful realtime sample with the Unix epoch.
        ("UTC", "unavailable", false)
    };
    match capture_temporal_action_envelope(timezone, None) {
        Ok(mut envelope) => {
            envelope.operator_timezone_source = source.to_string();
            if !timezone_known {
                envelope.operator_timezone = "unknown".to_string();
                envelope.operator_local_rfc3339 = "unavailable".to_string();
                envelope.capture_failure = Some("operator_timezone_unavailable".to_string());
                envelope.clock_sample.timezone = "unknown".to_string();
                envelope
                    .calibration_lineage
                    .push("operator_timezone_unavailable:wall_clock_preserved".to_string());
            }
            envelope
        }
        Err(error) => TemporalActionEnvelope::unavailable(format!(
            "operator_temporal_capture_failed:{error:?}"
        )),
    }
}

#[derive(Clone, Copy)]
enum ClockKind {
    Monotonic,
    Realtime,
    SuspendAware,
}

#[cfg(target_os = "linux")]
fn clock_ns(kind: ClockKind) -> Option<u128> {
    let clock_id = match kind {
        ClockKind::Monotonic => libc::CLOCK_MONOTONIC,
        ClockKind::Realtime => libc::CLOCK_REALTIME,
        ClockKind::SuspendAware => libc::CLOCK_BOOTTIME,
    };
    unix_clock_ns(clock_id)
}

#[cfg(all(unix, not(target_os = "linux")))]
fn clock_ns(kind: ClockKind) -> Option<u128> {
    let clock_id = match kind {
        ClockKind::Monotonic => libc::CLOCK_MONOTONIC,
        ClockKind::Realtime => libc::CLOCK_REALTIME,
        ClockKind::SuspendAware => return None,
    };
    unix_clock_ns(clock_id)
}

#[cfg(unix)]
fn unix_clock_ns(clock_id: libc::clockid_t) -> Option<u128> {
    let mut sample = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // SAFETY: `sample` is valid writable storage and clock_id is a platform constant.
    let result = unsafe { libc::clock_gettime(clock_id, &mut sample) };
    if result != 0 || sample.tv_sec < 0 || sample.tv_nsec < 0 {
        return None;
    }
    Some((sample.tv_sec as u128) * 1_000_000_000 + sample.tv_nsec as u128)
}

#[cfg(not(unix))]
fn clock_ns(kind: ClockKind) -> Option<u128> {
    static MONOTONIC_ORIGIN: OnceLock<Instant> = OnceLock::new();
    match kind {
        ClockKind::Monotonic => Some(
            MONOTONIC_ORIGIN
                .get_or_init(Instant::now)
                .elapsed()
                .as_nanos(),
        ),
        ClockKind::Realtime => Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .ok()?
                .as_nanos(),
        ),
        ClockKind::SuspendAware => None,
    }
}

#[cfg(target_os = "linux")]
fn clock_resolution_ns(kind: ClockKind) -> Option<u64> {
    let clock_id = match kind {
        ClockKind::Monotonic => libc::CLOCK_MONOTONIC,
        ClockKind::Realtime => libc::CLOCK_REALTIME,
        ClockKind::SuspendAware => libc::CLOCK_BOOTTIME,
    };
    unix_clock_resolution_ns(clock_id)
}

#[cfg(all(unix, not(target_os = "linux")))]
fn clock_resolution_ns(kind: ClockKind) -> Option<u64> {
    let clock_id = match kind {
        ClockKind::Monotonic => libc::CLOCK_MONOTONIC,
        ClockKind::Realtime => libc::CLOCK_REALTIME,
        ClockKind::SuspendAware => return None,
    };
    unix_clock_resolution_ns(clock_id)
}

#[cfg(unix)]
fn unix_clock_resolution_ns(clock_id: libc::clockid_t) -> Option<u64> {
    let mut resolution = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // SAFETY: `resolution` is valid writable storage and clock_id is a platform constant.
    let result = unsafe { libc::clock_getres(clock_id, &mut resolution) };
    if result != 0 || resolution.tv_sec < 0 || resolution.tv_nsec < 0 {
        return None;
    }
    let nanos = (resolution.tv_sec as u128) * 1_000_000_000 + resolution.tv_nsec as u128;
    u64::try_from(nanos.max(1)).ok()
}

#[cfg(not(unix))]
fn clock_resolution_ns(kind: ClockKind) -> Option<u64> {
    match kind {
        ClockKind::Monotonic | ClockKind::Realtime => Some(1_000_000),
        ClockKind::SuspendAware => None,
    }
}

fn current_boot_id() -> Option<String> {
    std::fs::read_to_string("/proc/sys/kernel/random/boot_id")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub fn capture_temporal_action_envelope(
    operator_timezone: &str,
    calibration: Option<&ClockUncertaintyBudget>,
) -> Result<TemporalActionEnvelope, TemporalActionCaptureError> {
    let timezone = operator_timezone.parse::<Tz>().map_err(|_| {
        TemporalActionCaptureError::InvalidOperatorTimezone(operator_timezone.into())
    })?;
    let monotonic_before = clock_ns(ClockKind::Monotonic)
        .ok_or(TemporalActionCaptureError::MonotonicClockUnavailable)?;
    let realtime_ns = clock_ns(ClockKind::Realtime)
        .ok_or(TemporalActionCaptureError::RealtimeClockUnavailable)?;
    let captured_at_utc = DateTime::<Utc>::from_timestamp(
        (realtime_ns / 1_000_000_000) as i64,
        (realtime_ns % 1_000_000_000) as u32,
    )
    .ok_or(TemporalActionCaptureError::InvalidRealtimeSample)?;
    let monotonic_after = clock_ns(ClockKind::Monotonic)
        .ok_or(TemporalActionCaptureError::MonotonicClockUnavailable)?;
    let realtime_resolution_ns = clock_resolution_ns(ClockKind::Realtime)
        .ok_or(TemporalActionCaptureError::RealtimeResolutionUnavailable)?;
    let monotonic_resolution_ns = clock_resolution_ns(ClockKind::Monotonic)
        .ok_or(TemporalActionCaptureError::MonotonicResolutionUnavailable)?;
    let capture_latency_ns = monotonic_after.saturating_sub(monotonic_before);
    let capture_uncertainty_ns = (capture_latency_ns / 2)
        .saturating_add(realtime_resolution_ns as u128)
        .saturating_add(monotonic_resolution_ns as u128);
    let wall_clock_accuracy_uncertainty_ns =
        calibration.map(|budget| budget.expanded_uncertainty_ns.ceil().max(0.0) as u128);
    let microsecond_representation_supported = realtime_resolution_ns <= 1_000;
    let microsecond_wall_clock_accuracy_verified = microsecond_representation_supported
        && calibration.is_some_and(|budget| {
            budget.expanded_uncertainty_ns <= 1_000.0
                && budget.coverage_probability >= 0.95
                && !budget.calibration_lineage.is_empty()
        });
    let confidence = if microsecond_wall_clock_accuracy_verified {
        TemporalConfidence::Verified
    } else if calibration.is_some() {
        TemporalConfidence::High
    } else {
        TemporalConfidence::Low
    };
    let local = captured_at_utc.with_timezone(&timezone);
    let operator_local_rfc3339 = local.to_rfc3339_opts(SecondsFormat::Nanos, true);
    let operator_utc_offset_seconds = local.offset().fix().local_minus_utc();
    let monotonic_ns = monotonic_before.saturating_add(capture_latency_ns / 2);
    let calibration_lineage = calibration
        .map(|budget| budget.calibration_lineage.clone())
        .unwrap_or_default();
    let synchronization_source = calibration.map(|budget| budget.method.clone());

    let action_id = Uuid::now_v7().to_string();
    Ok(TemporalActionEnvelope {
        schema_version: "focusa.temporal_action_envelope.v1".to_string(),
        envelope_id: Uuid::now_v7().to_string(),
        action_id,
        prediction_id: None,
        captured_at_utc,
        utc_unix_ns: realtime_ns as i128,
        operator_timezone: operator_timezone.to_string(),
        operator_timezone_source: "explicit_argument".to_string(),
        operator_local_rfc3339,
        operator_utc_offset_seconds,
        monotonic_ns,
        realtime_resolution_ns,
        monotonic_resolution_ns,
        capture_latency_ns,
        capture_uncertainty_ns,
        wall_clock_accuracy_uncertainty_ns,
        microsecond_representation_supported,
        microsecond_wall_clock_accuracy_verified,
        clock_source: "clock_gettime(CLOCK_REALTIME)+clock_gettime(CLOCK_MONOTONIC)".to_string(),
        synchronization_source,
        boot_id: current_boot_id(),
        process_id: std::process::id(),
        thread_id: format!("{:?}", std::thread::current().id()),
        confidence,
        capture_failure: None,
        calibration_lineage,
        clock_sample: TemporalClockSample {
            sample_id: Uuid::now_v7().to_string(),
            domain: TemporalClockDomain::WallUtc,
            wall_utc: captured_at_utc,
            monotonic_ns: Some(monotonic_ns),
            suspend_aware_ns: clock_ns(ClockKind::SuspendAware),
            boot_id: current_boot_id(),
            timezone: operator_timezone.to_string(),
            tzdb_version: None,
            source: "clock_gettime".to_string(),
            observed_offset_ns: calibration.map(|budget| budget.offset_ns),
            measurement_uncertainty_ns: wall_clock_accuracy_uncertainty_ns
                .unwrap_or(capture_uncertainty_ns),
            confidence,
        },
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemporalClockCapabilities {
    pub realtime: bool,
    pub suspend_excluding_monotonic: bool,
    pub suspend_aware_monotonic: bool,
    pub process_cpu: bool,
    pub thread_cpu: bool,
    pub tai: bool,
    pub fallback_behavior: Option<String>,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClockUncertaintyBudget {
    pub method: String,
    pub standard_uncertainty_ns: f64,
    pub expanded_uncertainty_ns: f64,
    pub coverage_factor: f64,
    pub coverage_probability: f64,
    pub offset_ns: i128,
    pub delay_ns: u128,
    pub jitter_ns: u128,
    pub dispersion_ns: u128,
    pub root_distance_ns: u128,
    pub frequency_error_ppb: f64,
    pub sample_age_ms: u64,
    pub calibration_lineage: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemporalVersionLineage {
    pub schema_version: String,
    pub policy_version: String,
    pub adapter_version: String,
    pub calendar_version: Option<String>,
    pub tzdb_version: Option<String>,
    pub estimator_version: Option<String>,
    pub clock_profile_version: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClockSamplePair {
    pub before: TemporalClockSample,
    pub after: TemporalClockSample,
    pub elapsed_lower_ns: u128,
    pub elapsed_upper_ns: Option<u128>,
    pub uncertainty: ClockUncertaintyBudget,
    pub crosses_boot_epoch: bool,
    pub crosses_suspend: bool,
    pub lineage: TemporalVersionLineage,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClockSamplePairError {
    ScopeEpochMismatch,
    NegativeElapsed,
    MissingCrossEpochBound,
    InvalidUncertainty,
    UntrustedSample,
}

pub fn validate_clock_sample_pair(pair: &ClockSamplePair) -> Result<(), ClockSamplePairError> {
    if pair
        .elapsed_upper_ns
        .is_some_and(|upper| upper < pair.elapsed_lower_ns)
    {
        return Err(ClockSamplePairError::NegativeElapsed);
    }
    if pair.crosses_boot_epoch && pair.elapsed_upper_ns.is_none() {
        return Err(ClockSamplePairError::MissingCrossEpochBound);
    }
    if pair.before.boot_id != pair.after.boot_id && !pair.crosses_boot_epoch {
        return Err(ClockSamplePairError::ScopeEpochMismatch);
    }
    if pair.before.confidence == TemporalConfidence::Unavailable
        || pair.after.confidence == TemporalConfidence::Unavailable
    {
        return Err(ClockSamplePairError::UntrustedSample);
    }
    if !pair.uncertainty.standard_uncertainty_ns.is_finite()
        || pair.uncertainty.standard_uncertainty_ns < 0.0
        || !pair.uncertainty.expanded_uncertainty_ns.is_finite()
        || pair.uncertainty.expanded_uncertainty_ns < pair.uncertainty.standard_uncertainty_ns
        || !(0.0..=1.0).contains(&pair.uncertainty.coverage_probability)
        || pair.uncertainty.coverage_factor < 1.0
        || pair.uncertainty.calibration_lineage.is_empty()
    {
        return Err(ClockSamplePairError::InvalidUncertainty);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn microsecond_calibration() -> ClockUncertaintyBudget {
        ClockUncertaintyBudget {
            method: "verified_ntp_calibration".to_string(),
            standard_uncertainty_ns: 250.0,
            expanded_uncertainty_ns: 500.0,
            coverage_factor: 2.0,
            coverage_probability: 0.95,
            offset_ns: 0,
            delay_ns: 100,
            jitter_ns: 100,
            dispersion_ns: 100,
            root_distance_ns: 500,
            frequency_error_ppb: 1.0,
            sample_age_ms: 1,
            calibration_lineage: vec!["ntp-proof:test".to_string()],
        }
    }

    #[test]
    fn missing_operator_timezone_preserves_current_wall_clock() {
        let before = Utc::now();
        let envelope = capture_operator_temporal_action_envelope_from_values(None, None);
        let after = Utc::now();

        assert!(envelope.captured_at_utc >= before);
        assert!(envelope.captured_at_utc <= after);
        assert!(envelope.utc_unix_ns > 0);
        assert_eq!(envelope.operator_timezone, "unknown");
        assert_eq!(envelope.operator_timezone_source, "unavailable");
        assert_eq!(envelope.operator_local_rfc3339, "unavailable");
        assert_eq!(
            envelope.capture_failure.as_deref(),
            Some("operator_timezone_unavailable")
        );
        assert_ne!(envelope.clock_source, "unavailable");
    }

    #[test]
    fn temporal_action_capture_fails_closed_for_unknown_timezone() {
        assert!(matches!(
            capture_temporal_action_envelope("Mars/Olympus_Mons", None),
            Err(TemporalActionCaptureError::InvalidOperatorTimezone(_))
        ));
    }

    #[test]
    fn uncalibrated_capture_never_claims_microsecond_wall_clock_accuracy() {
        let envelope = capture_temporal_action_envelope("America/Los_Angeles", None).unwrap();
        assert_eq!(
            envelope.schema_version,
            "focusa.temporal_action_envelope.v1"
        );
        assert_eq!(envelope.operator_timezone, "America/Los_Angeles");
        assert!(
            envelope.operator_local_rfc3339.contains("-07:00")
                || envelope.operator_local_rfc3339.contains("-08:00")
        );
        assert!(envelope.realtime_resolution_ns > 0);
        assert!(envelope.monotonic_resolution_ns > 0);
        assert!(envelope.monotonic_ns > 0);
        assert!(envelope.capture_uncertainty_ns > 0);
        assert!(!envelope.microsecond_wall_clock_accuracy_verified);
        assert_eq!(envelope.wall_clock_accuracy_uncertainty_ns, None);
        assert_eq!(envelope.confidence, TemporalConfidence::Low);
    }

    #[test]
    fn calibrated_capture_can_verify_microsecond_wall_clock_accuracy() {
        let calibration = microsecond_calibration();
        let envelope = capture_temporal_action_envelope("UTC", Some(&calibration)).unwrap();
        assert_eq!(
            envelope.microsecond_wall_clock_accuracy_verified,
            envelope.microsecond_representation_supported
        );
        assert_eq!(envelope.wall_clock_accuracy_uncertainty_ns, Some(500));
        assert_eq!(
            envelope.confidence,
            if envelope.microsecond_representation_supported {
                TemporalConfidence::Verified
            } else {
                TemporalConfidence::High
            }
        );
        assert_eq!(envelope.calibration_lineage, vec!["ntp-proof:test"]);
        assert_eq!(envelope.operator_utc_offset_seconds, 0);
    }

    #[test]
    fn temporal_action_envelope_round_trip_preserves_original_time_identity() {
        let envelope = capture_temporal_action_envelope("UTC", None).unwrap();
        let encoded = serde_json::to_vec(&envelope).unwrap();
        let decoded: TemporalActionEnvelope = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(decoded, envelope);
        assert_eq!(decoded.envelope_id, envelope.envelope_id);
        assert_eq!(decoded.monotonic_ns, envelope.monotonic_ns);
        assert_eq!(decoded.utc_unix_ns, envelope.utc_unix_ns);
    }
}
