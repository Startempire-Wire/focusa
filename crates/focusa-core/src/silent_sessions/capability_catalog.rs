use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::HarnessKind;

pub const HARNESS_CAPABILITY_NAMES: &[&str] = &[
    "structured_events",
    "stdout_stderr_split",
    "semantic_agent_state",
    "model_preflight",
    "model_observation",
    "model_switch",
    "thinking_control",
    "native_session_resume",
    "prompt_delivery",
    "steering",
    "followup_queue",
    "special_keys",
    "native_abort",
    "hard_pause",
    "token_usage",
    "cost_usage",
    "subscription_entitlement_probe",
];

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityFactState {
    Supported,
    Unsupported,
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CatalogFreshness {
    Fresh,
    Stale,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityFact {
    pub state: CapabilityFactState,
    pub source: String,
    pub observed_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub freshness: CatalogFreshness,
}

impl CapabilityFact {
    pub fn unknown(source: impl Into<String>) -> Self {
        Self {
            state: CapabilityFactState::Unknown,
            source: source.into(),
            observed_at: None,
            expires_at: None,
            freshness: CatalogFreshness::Unknown,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HarnessCapabilityCatalog {
    pub harness: HarnessKind,
    pub adapter_registered: CapabilityFact,
    pub capabilities: BTreeMap<String, CapabilityFact>,
    pub catalog_freshness: CatalogFreshness,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PreflightStatus {
    Passed,
    Blocked,
    Degraded,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PreflightCheck {
    pub name: String,
    pub state: CapabilityFactState,
    pub required: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityPreflightResult {
    pub status: PreflightStatus,
    pub strict: bool,
    pub checks: Vec<PreflightCheck>,
    pub catalog_freshness: CatalogFreshness,
    pub mutation_allowed: bool,
}

pub fn known_harnesses() -> Vec<HarnessKind> {
    vec![
        HarnessKind::Pi,
        HarnessKind::Codex,
        HarnessKind::Claude,
        HarnessKind::Opencode,
        HarnessKind::GenericRpc,
        HarnessKind::GenericPty,
    ]
}

pub fn unknown_harness_catalog(harness: HarnessKind) -> HarnessCapabilityCatalog {
    HarnessCapabilityCatalog {
        harness,
        adapter_registered: CapabilityFact::unknown("adapter_registry_unavailable"),
        capabilities: HARNESS_CAPABILITY_NAMES
            .iter()
            .map(|name| {
                (
                    (*name).to_string(),
                    CapabilityFact::unknown("adapter_probe_unavailable"),
                )
            })
            .collect(),
        catalog_freshness: CatalogFreshness::Unknown,
    }
}

pub fn strict_unknown_preflight(required_checks: &[&str]) -> CapabilityPreflightResult {
    CapabilityPreflightResult {
        status: PreflightStatus::Blocked,
        strict: true,
        checks: required_checks
            .iter()
            .map(|name| PreflightCheck {
                name: (*name).to_string(),
                state: CapabilityFactState::Unknown,
                required: true,
                reason: "no fresh canonical probe result exists".into(),
            })
            .collect(),
        catalog_freshness: CatalogFreshness::Unknown,
        mutation_allowed: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enum_identity_never_implies_runtime_capability() {
        let catalog = unknown_harness_catalog(HarnessKind::Pi);
        assert_eq!(
            catalog.adapter_registered.state,
            CapabilityFactState::Unknown
        );
        assert!(catalog.capabilities.values().all(|fact| {
            fact.state == CapabilityFactState::Unknown
                && fact.freshness == CatalogFreshness::Unknown
                && fact.observed_at.is_none()
        }));
    }

    #[test]
    fn strict_unknown_preflight_blocks_mutation() {
        let result = strict_unknown_preflight(&["entitlement", "model_availability"]);
        assert_eq!(result.status, PreflightStatus::Blocked);
        assert!(!result.mutation_allowed);
        assert!(result.checks.iter().all(|check| check.required));
    }
}
