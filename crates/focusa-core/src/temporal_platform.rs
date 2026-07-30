use std::{
    sync::OnceLock,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    temporal::{TemporalClockDomain, TemporalClockSample, TemporalConfidence},
    temporal_clock::TemporalClockCapabilities,
};

static PROCESS_MONOTONIC_ORIGIN: OnceLock<Instant> = OnceLock::new();

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformClockCapture {
    pub realtime_utc: DateTime<Utc>,
    pub suspend_excluding_monotonic_ns: u128,
    pub suspend_aware_monotonic_ns: Option<u128>,
    pub process_cpu_ns: Option<u128>,
    pub thread_cpu_ns: Option<u128>,
    pub tai_ns: Option<u128>,
    pub boot_id: Option<String>,
    pub capabilities: TemporalClockCapabilities,
}

#[cfg(unix)]
fn clock_gettime_ns(clock_id: libc::clockid_t) -> Option<u128> {
    let mut value = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // SAFETY: value points to writable timespec storage and clock_id is a platform constant.
    let result = unsafe { libc::clock_gettime(clock_id, &mut value) };
    (result == 0 && value.tv_sec >= 0 && value.tv_nsec >= 0).then(|| {
        (value.tv_sec as u128)
            .saturating_mul(1_000_000_000)
            .saturating_add(value.tv_nsec as u128)
    })
}

#[cfg(not(unix))]
fn clock_gettime_ns(_clock_id: i32) -> Option<u128> {
    None
}

fn boot_id() -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        return std::fs::read_to_string("/proc/sys/kernel/random/boot_id")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
    }
    #[allow(unreachable_code)]
    None
}

pub fn capture_platform_clocks() -> PlatformClockCapture {
    let realtime_utc: DateTime<Utc> = SystemTime::now().into();
    let active = PROCESS_MONOTONIC_ORIGIN
        .get_or_init(Instant::now)
        .elapsed()
        .as_nanos();
    #[cfg(target_os = "linux")]
    let suspend_aware = clock_gettime_ns(libc::CLOCK_BOOTTIME);
    #[cfg(not(target_os = "linux"))]
    let suspend_aware = None;
    #[cfg(unix)]
    let process_cpu = clock_gettime_ns(libc::CLOCK_PROCESS_CPUTIME_ID);
    #[cfg(not(unix))]
    let process_cpu = None;
    #[cfg(unix)]
    let thread_cpu = clock_gettime_ns(libc::CLOCK_THREAD_CPUTIME_ID);
    #[cfg(not(unix))]
    let thread_cpu = None;
    #[cfg(target_os = "linux")]
    let tai = clock_gettime_ns(libc::CLOCK_TAI);
    #[cfg(not(target_os = "linux"))]
    let tai = None;
    let fallback_behavior = (suspend_aware.is_none() || tai.is_none()).then(|| {
        "unsupported clock domains remain absent; no wall-clock substitution or false precision"
            .into()
    });
    PlatformClockCapture {
        realtime_utc,
        suspend_excluding_monotonic_ns: active,
        suspend_aware_monotonic_ns: suspend_aware,
        process_cpu_ns: process_cpu,
        thread_cpu_ns: thread_cpu,
        tai_ns: tai,
        boot_id: boot_id(),
        capabilities: TemporalClockCapabilities {
            realtime: true,
            suspend_excluding_monotonic: true,
            suspend_aware_monotonic: suspend_aware.is_some(),
            process_cpu: process_cpu.is_some(),
            thread_cpu: thread_cpu.is_some(),
            tai: tai.is_some(),
            fallback_behavior,
            evidence_refs: vec!["runtime:platform-clock-capability-probe".into()],
        },
    }
}

pub fn capture_temporal_clock_sample(
    timezone: impl Into<String>,
    tzdb_version: Option<String>,
) -> TemporalClockSample {
    let capture = capture_platform_clocks();
    TemporalClockSample {
        sample_id: Uuid::now_v7().to_string(),
        domain: if capture.suspend_aware_monotonic_ns.is_some() {
            TemporalClockDomain::SuspendAwareElapsed
        } else {
            TemporalClockDomain::MonotonicActive
        },
        wall_utc: capture.realtime_utc,
        monotonic_ns: Some(capture.suspend_excluding_monotonic_ns),
        suspend_aware_ns: capture.suspend_aware_monotonic_ns,
        boot_id: capture.boot_id,
        timezone: timezone.into(),
        tzdb_version,
        source: "focusa-platform-clock-adapter.v1".into(),
        observed_offset_ns: None,
        measurement_uncertainty_ns: if capture.capabilities.suspend_aware_monotonic {
            1_000_000
        } else {
            10_000_000
        },
        confidence: if capture.capabilities.suspend_aware_monotonic {
            TemporalConfidence::High
        } else {
            TemporalConfidence::Low
        },
    }
}

pub fn unix_epoch_ns() -> Option<u128> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_nanos())
}
