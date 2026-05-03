//! Shared bounded read-response helpers for large/read-heavy API surfaces.
//!
//! These helpers make truncation explicit without changing canonical state.
//! Route handlers own domain-specific selection/rehydration; this module owns
//! consistent limit resolution and metadata envelopes.

use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RehydrateHint {
    pub mode: &'static str,
    pub parameter: &'static str,
    pub value: &'static str,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PressureStatus {
    pub status: &'static str,
    pub active: bool,
    pub configured: bool,
    pub rss_kb: Option<u64>,
    pub threshold_kb: Option<u64>,
    pub mode: &'static str,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct BoundedReadMetadata {
    pub total: usize,
    pub returned: usize,
    pub omitted: usize,
    pub truncated: bool,
    pub limit: usize,
    pub requested_limit: Option<usize>,
    pub default_limit: usize,
    pub full_limit: usize,
    pub include_full_payload: bool,
    pub summary_only: bool,
    pub cursor: Option<String>,
    pub next_cursor: Option<String>,
    pub rehydrate: Option<RehydrateHint>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundedReadOptions {
    pub requested_limit: Option<usize>,
    pub include_full_payload: bool,
    pub summary_only: bool,
    pub cursor: Option<String>,
    pub default_limit: usize,
    pub full_limit: usize,
}

impl BoundedReadOptions {
    pub fn resolved_limit(&self) -> usize {
        let default_limit = self.default_limit.max(1);
        let full_limit = self.full_limit.max(default_limit);
        let ceiling = if self.include_full_payload {
            full_limit
        } else {
            default_limit
        };
        self.requested_limit.unwrap_or(ceiling).clamp(1, ceiling)
    }
}

pub fn env_limit(name: &str, fallback: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(fallback)
        .max(1)
}

fn parse_status_value_kb(status_text: &str, key: &str) -> Option<u64> {
    status_text.lines().find_map(|line| {
        let (line_key, rest) = line.split_once(':')?;
        if line_key != key {
            return None;
        }
        rest.split_whitespace().next()?.parse::<u64>().ok()
    })
}

pub fn current_rss_kb() -> Option<u64> {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|text| parse_status_value_kb(&text, "VmRSS"))
}

pub fn pressure_status() -> PressureStatus {
    let threshold_kb = std::env::var("FOCUSA_MEMORY_PRESSURE_RSS_KB")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| *value > 0);
    let rss_kb = current_rss_kb();
    let active = threshold_kb
        .zip(rss_kb)
        .map(|(threshold, rss)| rss >= threshold)
        .unwrap_or(false);
    PressureStatus {
        status: if active { "pressure" } else { "ok" },
        active,
        configured: threshold_kb.is_some(),
        rss_kb,
        threshold_kb,
        mode: if active {
            "summary_only_by_default"
        } else {
            "normal"
        },
    }
}

pub fn full_payload_blocked_by_pressure(
    include_full_payload: bool,
    force_full_payload: bool,
) -> bool {
    include_full_payload && !force_full_payload && pressure_status().active
}

pub fn bounded_metadata(
    total: usize,
    returned: usize,
    options: BoundedReadOptions,
) -> BoundedReadMetadata {
    let limit = options.resolved_limit();
    let omitted = total.saturating_sub(returned);
    let truncated = omitted > 0;
    BoundedReadMetadata {
        total,
        returned,
        omitted,
        truncated,
        limit,
        requested_limit: options.requested_limit,
        default_limit: options.default_limit.max(1),
        full_limit: options.full_limit.max(options.default_limit.max(1)),
        include_full_payload: options.include_full_payload,
        summary_only: options.summary_only,
        cursor: options.cursor,
        next_cursor: None,
        rehydrate: truncated.then_some(RehydrateHint {
            mode: "full_payload_opt_in",
            parameter: "include_full_payload",
            value: "true",
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_default_limit_without_full_payload() {
        let options = BoundedReadOptions {
            requested_limit: Some(500),
            include_full_payload: false,
            summary_only: true,
            cursor: None,
            default_limit: 100,
            full_limit: 1000,
        };
        assert_eq!(options.resolved_limit(), 100);
    }

    #[test]
    fn resolves_full_limit_with_explicit_opt_in() {
        let options = BoundedReadOptions {
            requested_limit: Some(500),
            include_full_payload: true,
            summary_only: false,
            cursor: None,
            default_limit: 100,
            full_limit: 1000,
        };
        assert_eq!(options.resolved_limit(), 500);
    }

    #[test]
    fn pressure_status_is_safe_without_threshold() {
        let status = pressure_status();
        assert!(matches!(status.status, "ok" | "pressure"));
        assert!(matches!(status.mode, "normal" | "summary_only_by_default"));
    }

    #[test]
    fn full_payload_pressure_block_respects_force_flag() {
        // The active pressure bit is environment/runtime dependent; this still proves
        // that non-full requests are never blocked and forced requests are never blocked.
        assert!(!full_payload_blocked_by_pressure(false, false));
        assert!(!full_payload_blocked_by_pressure(true, true));
    }

    #[test]
    fn metadata_makes_truncation_and_rehydrate_path_explicit() {
        let metadata = bounded_metadata(
            10,
            3,
            BoundedReadOptions {
                requested_limit: Some(3),
                include_full_payload: false,
                summary_only: true,
                cursor: Some("0".to_string()),
                default_limit: 5,
                full_limit: 50,
            },
        );
        assert_eq!(metadata.total, 10);
        assert_eq!(metadata.returned, 3);
        assert_eq!(metadata.omitted, 7);
        assert!(metadata.truncated);
        assert_eq!(
            metadata.rehydrate.unwrap().parameter,
            "include_full_payload"
        );
    }
}
