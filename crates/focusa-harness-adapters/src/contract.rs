//! Versioned HarnessAdapter interface and capability negotiation.

use focusa_core::silent_session::{
    HarnessKind, ModelBinding, ObservationProvenance, SilentSessionRunId,
};
use focusa_core::silent_session_config::EffectiveSilentSessionConfig;
use focusa_core::silent_session_launch::{LaunchManifest, LaunchPreparationError};
use focusa_core::silent_session_protocol::{
    CapabilityRequirement, CapabilitySupport, ProtocolVersionNegotiationError, ProtocolVersionOffer,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use thiserror::Error;

pub const HARNESS_ADAPTER_PROTOCOL_SCHEMA: &str = "focusa.harness_adapter_protocol.v1";
pub const HARNESS_ADAPTER_PROTOCOL_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HarnessCapability {
    StructuredEvents,
    StdoutStderrSplit,
    SemanticAgentState,
    ModelPreflight,
    ModelObservation,
    ModelSwitch,
    ThinkingControl,
    NativeSessionResume,
    PromptDelivery,
    Steering,
    FollowupQueue,
    SpecialKeys,
    NativeAbort,
    HardPause,
    TokenUsage,
    CostUsage,
    SubscriptionEntitlementProbe,
}

pub const ALL_HARNESS_CAPABILITIES: [HarnessCapability; 17] = [
    HarnessCapability::StructuredEvents,
    HarnessCapability::StdoutStderrSplit,
    HarnessCapability::SemanticAgentState,
    HarnessCapability::ModelPreflight,
    HarnessCapability::ModelObservation,
    HarnessCapability::ModelSwitch,
    HarnessCapability::ThinkingControl,
    HarnessCapability::NativeSessionResume,
    HarnessCapability::PromptDelivery,
    HarnessCapability::Steering,
    HarnessCapability::FollowupQueue,
    HarnessCapability::SpecialKeys,
    HarnessCapability::NativeAbort,
    HarnessCapability::HardPause,
    HarnessCapability::TokenUsage,
    HarnessCapability::CostUsage,
    HarnessCapability::SubscriptionEntitlementProbe,
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HarnessCapabilities {
    pub structured_events: CapabilitySupport,
    pub stdout_stderr_split: CapabilitySupport,
    pub semantic_agent_state: CapabilitySupport,
    pub model_preflight: CapabilitySupport,
    pub model_observation: CapabilitySupport,
    pub model_switch: CapabilitySupport,
    pub thinking_control: CapabilitySupport,
    pub native_session_resume: CapabilitySupport,
    pub prompt_delivery: CapabilitySupport,
    pub steering: CapabilitySupport,
    pub followup_queue: CapabilitySupport,
    pub special_keys: CapabilitySupport,
    pub native_abort: CapabilitySupport,
    pub hard_pause: CapabilitySupport,
    pub token_usage: CapabilitySupport,
    pub cost_usage: CapabilitySupport,
    pub subscription_entitlement_probe: CapabilitySupport,
}

impl HarnessCapabilities {
    pub fn all(support: CapabilitySupport) -> Self {
        Self {
            structured_events: support,
            stdout_stderr_split: support,
            semantic_agent_state: support,
            model_preflight: support,
            model_observation: support,
            model_switch: support,
            thinking_control: support,
            native_session_resume: support,
            prompt_delivery: support,
            steering: support,
            followup_queue: support,
            special_keys: support,
            native_abort: support,
            hard_pause: support,
            token_usage: support,
            cost_usage: support,
            subscription_entitlement_probe: support,
        }
    }

    pub fn support(&self, capability: HarnessCapability) -> CapabilitySupport {
        match capability {
            HarnessCapability::StructuredEvents => self.structured_events,
            HarnessCapability::StdoutStderrSplit => self.stdout_stderr_split,
            HarnessCapability::SemanticAgentState => self.semantic_agent_state,
            HarnessCapability::ModelPreflight => self.model_preflight,
            HarnessCapability::ModelObservation => self.model_observation,
            HarnessCapability::ModelSwitch => self.model_switch,
            HarnessCapability::ThinkingControl => self.thinking_control,
            HarnessCapability::NativeSessionResume => self.native_session_resume,
            HarnessCapability::PromptDelivery => self.prompt_delivery,
            HarnessCapability::Steering => self.steering,
            HarnessCapability::FollowupQueue => self.followup_queue,
            HarnessCapability::SpecialKeys => self.special_keys,
            HarnessCapability::NativeAbort => self.native_abort,
            HarnessCapability::HardPause => self.hard_pause,
            HarnessCapability::TokenUsage => self.token_usage,
            HarnessCapability::CostUsage => self.cost_usage,
            HarnessCapability::SubscriptionEntitlementProbe => self.subscription_entitlement_probe,
        }
    }

    pub fn explicit_entries(&self) -> BTreeMap<HarnessCapability, CapabilitySupport> {
        ALL_HARNESS_CAPABILITIES
            .into_iter()
            .map(|capability| (capability, self.support(capability)))
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpstreamProtocolVersioning {
    Declared,
    Undeclared,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpstreamProtocolDescriptor {
    pub protocol_id: String,
    pub versioning: UpstreamProtocolVersioning,
    pub observed_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HarnessAdapterDescriptor {
    pub schema: String,
    pub adapter_id: String,
    pub adapter_version: String,
    pub protocol_versions: ProtocolVersionOffer,
    pub upstream_protocol: UpstreamProtocolDescriptor,
    pub capabilities: HarnessCapabilities,
    pub limitations: Vec<String>,
}

impl HarnessAdapterDescriptor {
    pub fn negotiate(
        &self,
        request: &HarnessNegotiationRequest,
    ) -> Result<NegotiatedHarnessContract, HarnessNegotiationError> {
        if self.schema != HARNESS_ADAPTER_PROTOCOL_SCHEMA
            || self.adapter_id.trim().is_empty()
            || self.adapter_version.trim().is_empty()
        {
            return Err(HarnessNegotiationError::InvalidDescriptor);
        }
        let selected_protocol_version = self
            .protocol_versions
            .negotiate_highest_common(&request.protocol_versions)?;
        for (capability, requirement) in &request.required_capabilities {
            let actual = self.capabilities.support(*capability);
            if !actual.satisfies(*requirement) {
                return Err(HarnessNegotiationError::RequiredCapabilityMissing {
                    capability: *capability,
                    requirement: *requirement,
                    actual,
                });
            }
        }
        Ok(NegotiatedHarnessContract {
            schema: HARNESS_ADAPTER_PROTOCOL_SCHEMA.into(),
            adapter_id: self.adapter_id.clone(),
            adapter_version: self.adapter_version.clone(),
            selected_protocol_version,
            capabilities: self.capabilities.clone(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HarnessNegotiationRequest {
    pub protocol_versions: ProtocolVersionOffer,
    pub required_capabilities: BTreeMap<HarnessCapability, CapabilityRequirement>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NegotiatedHarnessContract {
    pub schema: String,
    pub adapter_id: String,
    pub adapter_version: String,
    pub selected_protocol_version: u32,
    pub capabilities: HarnessCapabilities,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum HarnessNegotiationError {
    #[error("harness adapter descriptor is invalid")]
    InvalidDescriptor,
    #[error("harness adapter protocol is incompatible: {0}")]
    Protocol(#[from] ProtocolVersionNegotiationError),
    #[error(
        "required harness capability {capability:?} needs {requirement:?}, actual support is {actual:?}"
    )]
    RequiredCapabilityMissing {
        capability: HarnessCapability,
        requirement: CapabilityRequirement,
        actual: CapabilitySupport,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct EffectiveConfig {
    pub session: EffectiveSilentSessionConfig,
    pub launch_manifest: LaunchManifest,
    pub negotiation: HarnessNegotiationRequest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreflightStatus {
    Passed,
    Blocked,
    Degraded,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreflightResult {
    pub status: PreflightStatus,
    pub negotiated_contract: Option<NegotiatedHarnessContract>,
    pub failure_class: Option<String>,
    pub message: String,
}

impl PreflightResult {
    pub fn passed(contract: NegotiatedHarnessContract) -> Self {
        Self {
            status: PreflightStatus::Passed,
            negotiated_contract: Some(contract),
            failure_class: None,
            message: "harness adapter contract preflight passed".into(),
        }
    }

    pub fn blocked(failure_class: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            status: PreflightStatus::Blocked,
            negotiated_contract: None,
            failure_class: Some(failure_class.into()),
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RunRef {
    pub run_id: SilentSessionRunId,
    pub generation: u64,
}

impl RunRef {
    pub fn validate(&self) -> Result<(), HarnessAdapterError> {
        if !self.run_id.is_uuid_v7() || self.generation == 0 {
            return Err(HarnessAdapterError::InvalidRunRef);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImagePayload {
    pub data_base64: String,
    pub mime_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptPayload {
    pub message: String,
    pub images: Vec<ImagePayload>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputKind {
    Steering,
    Followup,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputPayload {
    pub kind: InputKind,
    pub message: String,
    pub images: Vec<ImagePayload>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HarnessActivity {
    Initializing,
    Working,
    WaitingInput,
    Idle,
    Aborted,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HarnessState {
    pub activity: HarnessActivity,
    pub is_streaming: bool,
    pub pending_message_count: u64,
    pub native_session_ref: Option<String>,
    pub raw: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HarnessEvent {
    pub kind: String,
    pub source: String,
    pub provenance: ObservationProvenance,
    pub payload: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum HarnessAdapterError {
    #[error("run reference must contain a UUIDv7 run id and positive generation")]
    InvalidRunRef,
    #[error("harness adapter effective configuration is invalid: {0}")]
    InvalidConfig(String),
    #[error("harness adapter capability is unsupported: {capability:?} ({support:?})")]
    UnsupportedCapability {
        capability: HarnessCapability,
        support: CapabilitySupport,
    },
    #[error("harness adapter protocol negotiation failed: {0}")]
    Negotiation(#[from] HarnessNegotiationError),
    #[error("launch manifest validation failed: {0}")]
    Launch(#[from] LaunchPreparationError),
    #[error("harness frame is invalid: {0}")]
    InvalidFrame(String),
    #[error("harness transport failed: {0}")]
    Transport(String),
    #[error("harness response is invalid: {0}")]
    InvalidResponse(String),
}

pub trait HarnessAdapter {
    fn descriptor(&self) -> HarnessAdapterDescriptor;

    fn capabilities(&self) -> HarnessCapabilities {
        self.descriptor().capabilities
    }

    fn preflight(&self, config: &EffectiveConfig) -> PreflightResult;
    fn build_launch_manifest(
        &self,
        config: &EffectiveConfig,
    ) -> Result<LaunchManifest, HarnessAdapterError>;
    fn parse_event(&self, frame: &[u8]) -> Result<Vec<HarnessEvent>, HarnessAdapterError>;
    fn send_prompt(
        &mut self,
        run: RunRef,
        prompt: PromptPayload,
    ) -> Result<(), HarnessAdapterError>;
    fn send_input(&mut self, run: RunRef, input: InputPayload) -> Result<(), HarnessAdapterError>;
    fn abort(&mut self, run: RunRef) -> Result<(), HarnessAdapterError>;
    fn query_state(&mut self, run: RunRef) -> Result<HarnessState, HarnessAdapterError>;
    fn query_model(&mut self, run: RunRef) -> Result<ModelBinding, HarnessAdapterError>;
    fn resume_native_session(&mut self, native_ref: &str) -> Result<(), HarnessAdapterError>;
}

pub(crate) fn validate_effective_config(
    descriptor: &HarnessAdapterDescriptor,
    expected_kind: HarnessKind,
    config: &EffectiveConfig,
) -> Result<NegotiatedHarnessContract, HarnessAdapterError> {
    if config.session.effective_config.harness.kind != expected_kind
        || config.launch_manifest.harness_kind != expected_kind
        || config.session.effective_config.harness.adapter_version != descriptor.adapter_version
        || config.launch_manifest.reproducibility.adapter_version != descriptor.adapter_version
    {
        return Err(HarnessAdapterError::InvalidConfig(
            "harness kind or adapter version mismatch".into(),
        ));
    }
    config.launch_manifest.validate()?;
    Ok(descriptor.negotiate(&config.negotiation)?)
}

pub(crate) fn require_capability(
    capabilities: &HarnessCapabilities,
    capability: HarnessCapability,
    requirement: CapabilityRequirement,
) -> Result<(), HarnessAdapterError> {
    let support = capabilities.support(capability);
    if support.satisfies(requirement) {
        Ok(())
    } else {
        Err(HarnessAdapterError::UnsupportedCapability {
            capability,
            support,
        })
    }
}
