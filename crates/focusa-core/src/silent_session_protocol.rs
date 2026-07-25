//! Shared capability truth and protocol-version negotiation for Silent Session
//! harness adapters and process backends.
//!
//! Capability absence is a typed value, never an omitted field or an implicit
//! fallback. Adapter/backend crates define their own capability enums while
//! reusing these support and negotiation rules.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilitySupport {
    Unsupported,
    Heuristic,
    Emulated,
    Native,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityRequirement {
    Available,
    Deterministic,
    Native,
}

impl CapabilitySupport {
    pub fn satisfies(self, requirement: CapabilityRequirement) -> bool {
        match requirement {
            CapabilityRequirement::Available => self != Self::Unsupported,
            CapabilityRequirement::Deterministic => {
                matches!(self, Self::Emulated | Self::Native)
            }
            CapabilityRequirement::Native => self == Self::Native,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolVersionOffer {
    pub supported_versions: BTreeSet<u32>,
}

impl ProtocolVersionOffer {
    pub fn new(supported_versions: impl IntoIterator<Item = u32>) -> Self {
        Self {
            supported_versions: supported_versions.into_iter().collect(),
        }
    }

    pub fn negotiate_highest_common(
        &self,
        remote: &Self,
    ) -> Result<u32, ProtocolVersionNegotiationError> {
        if self.supported_versions.is_empty() || remote.supported_versions.is_empty() {
            return Err(ProtocolVersionNegotiationError::EmptyVersionOffer);
        }
        self.supported_versions
            .intersection(&remote.supported_versions)
            .max()
            .copied()
            .ok_or(ProtocolVersionNegotiationError::ProtocolIncompatible)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ProtocolVersionNegotiationError {
    #[error("protocol version offers must not be empty")]
    EmptyVersionOffer,
    #[error("protocol peers have no common supported version")]
    ProtocolIncompatible,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn support_requirements_distinguish_heuristic_deterministic_and_native_truth() {
        assert!(CapabilitySupport::Heuristic.satisfies(CapabilityRequirement::Available));
        assert!(!CapabilitySupport::Heuristic.satisfies(CapabilityRequirement::Deterministic));
        assert!(CapabilitySupport::Emulated.satisfies(CapabilityRequirement::Deterministic));
        assert!(!CapabilitySupport::Emulated.satisfies(CapabilityRequirement::Native));
        assert!(CapabilitySupport::Native.satisfies(CapabilityRequirement::Native));
        assert!(!CapabilitySupport::Unsupported.satisfies(CapabilityRequirement::Available));
    }

    #[test]
    fn version_negotiation_selects_highest_common_and_fails_closed() {
        let local = ProtocolVersionOffer::new([1, 2, 3]);
        assert_eq!(
            local.negotiate_highest_common(&ProtocolVersionOffer::new([1, 2])),
            Ok(2)
        );
        assert_eq!(
            local.negotiate_highest_common(&ProtocolVersionOffer::new([4])),
            Err(ProtocolVersionNegotiationError::ProtocolIncompatible)
        );
        assert_eq!(
            local.negotiate_highest_common(&ProtocolVersionOffer::new([])),
            Err(ProtocolVersionNegotiationError::EmptyVersionOffer)
        );
    }
}
