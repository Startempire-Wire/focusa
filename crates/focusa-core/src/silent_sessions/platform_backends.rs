//! Optional generic, migration, Herdr, macOS, and Windows backend truth.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ClaimConfidence {
    HeuristicLow,
    HeuristicMedium,
    Verified,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParsedRuntimeClaim {
    pub kind: String,
    pub value: String,
    pub confidence: ClaimConfidence,
    pub evidence_ref: Option<String>,
}
impl ParsedRuntimeClaim {
    pub fn verify_structured(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.confidence == ClaimConfidence::Verified && self.evidence_ref.is_some(),
            "heuristic output cannot become a structured fact"
        );
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GenericAdapterCapabilities {
    pub rpc: bool,
    pub pty: bool,
    pub output: bool,
    pub text_control: bool,
    pub key_control: bool,
    pub semantic_state: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TmuxMigrationBackend {
    pub optional: bool,
    pub imported_aliases: Vec<String>,
    pub imported_log_refs: Vec<String>,
    pub attach_supported: bool,
    pub canonical_identity_owner: bool,
    pub canonical_state_owner: bool,
    pub model_owner: bool,
    pub health_owner: bool,
}
impl TmuxMigrationBackend {
    pub fn verify(&self) -> anyhow::Result<()> {
        anyhow::ensure!(self.optional, "tmux migration backend must remain optional");
        anyhow::ensure!(
            !self.canonical_identity_owner
                && !self.canonical_state_owner
                && !self.model_owner
                && !self.health_owner,
            "tmux cannot own canonical identity, state, model, or health"
        );
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HerdrBackend {
    pub available: bool,
    pub attach: bool,
    pub stream: bool,
    pub semantic_state: bool,
    pub reconnect: bool,
    pub fallback: bool,
    pub capabilities_negotiated: bool,
    pub daemon_canonical_authority: bool,
}
impl HerdrBackend {
    pub fn verify(&self) -> anyhow::Result<()> {
        if self.available {
            anyhow::ensure!(
                self.capabilities_negotiated && self.daemon_canonical_authority,
                "Herdr requires capability negotiation and daemon authority"
            );
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PlatformSupport {
    Unsupported,
    Experimental,
    Proven,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlatformBackendProof {
    pub platform: String,
    pub support: PlatformSupport,
    pub process_tree: bool,
    pub streams: bool,
    pub controls: bool,
    pub pause_declared: bool,
    pub recovery: bool,
    pub owner_execution: bool,
    pub resources_declared: bool,
    pub job_object: bool,
    pub conpty: bool,
    pub runtime_suite_ref: Option<String>,
}
impl PlatformBackendProof {
    pub fn authorize_support_claim(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.support == PlatformSupport::Proven,
            "platform remains explicitly unsupported until runtime proof is green"
        );
        anyhow::ensure!(
            self.process_tree
                && self.streams
                && self.controls
                && self.recovery
                && self.owner_execution
                && self.resources_declared,
            "platform proof incomplete"
        );
        anyhow::ensure!(
            self.runtime_suite_ref
                .as_deref()
                .is_some_and(|r| !r.is_empty()),
            "platform runtime suite evidence required"
        );
        if self.platform.eq_ignore_ascii_case("windows") {
            anyhow::ensure!(
                self.job_object && self.conpty,
                "Windows requires Job Object and ConPTY proof"
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heuristic_claims_never_promote_without_verified_evidence() {
        let heuristic = ParsedRuntimeClaim {
            kind: "prompt".into(),
            value: "continue?".into(),
            confidence: ClaimConfidence::HeuristicMedium,
            evidence_ref: Some("terminal:1".into()),
        };
        assert!(heuristic.verify_structured().is_err());
        let verified = ParsedRuntimeClaim {
            confidence: ClaimConfidence::Verified,
            evidence_ref: Some("rpc:event:1".into()),
            ..heuristic
        };
        assert!(verified.verify_structured().is_ok());
    }

    #[test]
    fn tmux_migration_backend_cannot_claim_canonical_authority() {
        let valid = TmuxMigrationBackend {
            optional: true,
            imported_aliases: vec!["legacy".into()],
            imported_log_refs: vec!["log:1".into()],
            attach_supported: true,
            canonical_identity_owner: false,
            canonical_state_owner: false,
            model_owner: false,
            health_owner: false,
        };
        assert!(valid.verify().is_ok());
        assert!(
            TmuxMigrationBackend {
                canonical_state_owner: true,
                ..valid
            }
            .verify()
            .is_err()
        );
    }

    #[test]
    fn herdr_requires_negotiated_capabilities_and_daemon_authority() {
        let valid = HerdrBackend {
            available: true,
            attach: true,
            stream: true,
            semantic_state: true,
            reconnect: true,
            fallback: true,
            capabilities_negotiated: true,
            daemon_canonical_authority: true,
        };
        assert!(valid.verify().is_ok());
        assert!(
            HerdrBackend {
                capabilities_negotiated: false,
                ..valid
            }
            .verify()
            .is_err()
        );
    }

    #[test]
    fn platform_support_claims_require_runtime_evidence_and_windows_native_primitives() {
        let proven = PlatformBackendProof {
            platform: "linux".into(),
            support: PlatformSupport::Proven,
            process_tree: true,
            streams: true,
            controls: true,
            pause_declared: true,
            recovery: true,
            owner_execution: true,
            resources_declared: true,
            job_object: false,
            conpty: false,
            runtime_suite_ref: Some("ci:linux:1".into()),
        };
        assert!(proven.authorize_support_claim().is_ok());
        assert!(
            PlatformBackendProof {
                platform: "windows".into(),
                job_object: false,
                conpty: false,
                ..proven.clone()
            }
            .authorize_support_claim()
            .is_err()
        );
        assert!(
            PlatformBackendProof {
                support: PlatformSupport::Unsupported,
                ..proven
            }
            .authorize_support_claim()
            .is_err()
        );
    }
}
