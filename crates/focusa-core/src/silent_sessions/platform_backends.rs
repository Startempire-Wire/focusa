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
