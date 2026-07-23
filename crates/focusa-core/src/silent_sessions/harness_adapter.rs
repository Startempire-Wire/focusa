//! Versioned harness-adapter and process-backend contracts for Spec133.

use std::collections::{BTreeMap, VecDeque};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    LaunchManifest, OutputChannel, SilentSessionEvent, SilentSessionId, SilentSessionRunId,
};

pub const HARNESS_ADAPTER_PROTOCOL_MAJOR: u16 = 1;
pub const HARNESS_ADAPTER_PROTOCOL_MINOR: u16 = 0;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CapabilitySupport {
    Supported,
    Unsupported,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HarnessCapabilities {
    pub adapter_id: String,
    pub adapter_version: String,
    pub protocol_major: u16,
    pub protocol_minor: u16,
    pub capabilities: BTreeMap<HarnessCapability, CapabilitySupport>,
}

impl HarnessCapabilities {
    pub fn support(&self, capability: HarnessCapability) -> CapabilitySupport {
        self.capabilities
            .get(&capability)
            .copied()
            .unwrap_or(CapabilitySupport::Unsupported)
    }

    pub fn negotiate(&self, required_major: u16, minimum_minor: u16) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.protocol_major == required_major,
            "harness protocol major version mismatch"
        );
        anyhow::ensure!(
            self.protocol_minor >= minimum_minor,
            "harness protocol minor version is too old"
        );
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HarnessState {
    Starting,
    Running,
    WaitingInput,
    Completed,
    Failed,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HarnessRunRef {
    pub session_id: SilentSessionId,
    pub run_id: SilentSessionRunId,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PromptPayload {
    pub artifact_ref: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InputPayload {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ObservedHarnessModel {
    pub provider: String,
    pub model: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HarnessPreflightResult {
    pub allowed: bool,
    pub reason: String,
    pub observed_capabilities: HarnessCapabilities,
}

#[derive(Debug, thiserror::Error)]
pub enum HarnessAdapterError {
    #[error("harness capability is explicitly unsupported: {0:?}")]
    Unsupported(HarnessCapability),
    #[error("harness exact run target is unknown")]
    UnknownRun,
    #[error("harness protocol response is invalid: {0}")]
    InvalidResponse(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub trait HarnessAdapter {
    fn capabilities(&self) -> HarnessCapabilities;
    fn preflight(&self, manifest: &LaunchManifest) -> HarnessPreflightResult;
    fn build_launch_manifest(
        &self,
        manifest: LaunchManifest,
    ) -> Result<LaunchManifest, HarnessAdapterError>;
    fn parse_event(&self, frame: &[u8]) -> Result<Vec<SilentSessionEvent>, HarnessAdapterError>;
    fn send_prompt(
        &mut self,
        run: &HarnessRunRef,
        prompt: PromptPayload,
    ) -> Result<(), HarnessAdapterError>;
    fn send_input(
        &mut self,
        run: &HarnessRunRef,
        input: InputPayload,
    ) -> Result<(), HarnessAdapterError>;
    fn abort(&mut self, run: &HarnessRunRef) -> Result<(), HarnessAdapterError>;
    fn query_state(&self, run: &HarnessRunRef) -> Result<HarnessState, HarnessAdapterError>;
    fn query_model(&self, run: &HarnessRunRef)
    -> Result<ObservedHarnessModel, HarnessAdapterError>;
    fn resume_native_session(&mut self, native_ref: &str) -> Result<(), HarnessAdapterError>;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PiRpcAdapter {
    pub endpoint: String,
    pub adapter_version: String,
}

impl PiRpcAdapter {
    fn unsupported(&self, capability: HarnessCapability) -> HarnessAdapterError {
        HarnessAdapterError::Unsupported(capability)
    }
}

impl HarnessAdapter for PiRpcAdapter {
    fn capabilities(&self) -> HarnessCapabilities {
        let supported = [
            HarnessCapability::StructuredEvents,
            HarnessCapability::ModelPreflight,
        ];
        explicit_capabilities("pi_rpc", &self.adapter_version, &supported)
    }

    fn preflight(&self, manifest: &LaunchManifest) -> HarnessPreflightResult {
        let validation = manifest.validate();
        HarnessPreflightResult {
            allowed: validation.is_ok() && self.endpoint.starts_with("unix://"),
            reason: validation
                .err()
                .map(|error| error.to_string())
                .unwrap_or_else(|| "typed Pi RPC preflight passed".into()),
            observed_capabilities: self.capabilities(),
        }
    }

    fn build_launch_manifest(
        &self,
        manifest: LaunchManifest,
    ) -> Result<LaunchManifest, HarnessAdapterError> {
        manifest.validate()?;
        Ok(manifest)
    }

    fn parse_event(&self, frame: &[u8]) -> Result<Vec<SilentSessionEvent>, HarnessAdapterError> {
        serde_json::from_slice(frame)
            .map(|event| vec![event])
            .map_err(|error| HarnessAdapterError::InvalidResponse(error.to_string()))
    }

    fn send_prompt(
        &mut self,
        _run: &HarnessRunRef,
        _prompt: PromptPayload,
    ) -> Result<(), HarnessAdapterError> {
        Err(self.unsupported(HarnessCapability::PromptDelivery))
    }

    fn send_input(
        &mut self,
        _run: &HarnessRunRef,
        _input: InputPayload,
    ) -> Result<(), HarnessAdapterError> {
        Err(self.unsupported(HarnessCapability::Steering))
    }

    fn abort(&mut self, _run: &HarnessRunRef) -> Result<(), HarnessAdapterError> {
        Err(self.unsupported(HarnessCapability::NativeAbort))
    }

    fn query_state(&self, _run: &HarnessRunRef) -> Result<HarnessState, HarnessAdapterError> {
        Err(self.unsupported(HarnessCapability::SemanticAgentState))
    }

    fn query_model(
        &self,
        _run: &HarnessRunRef,
    ) -> Result<ObservedHarnessModel, HarnessAdapterError> {
        Err(self.unsupported(HarnessCapability::ModelObservation))
    }

    fn resume_native_session(&mut self, _native_ref: &str) -> Result<(), HarnessAdapterError> {
        Err(self.unsupported(HarnessCapability::NativeSessionResume))
    }
}

#[derive(Debug, Clone)]
pub struct DeterministicFakeAdapter {
    pub manifest: LaunchManifest,
    pub state: HarnessState,
    pub model: ObservedHarnessModel,
    pub parsed_events: Vec<SilentSessionEvent>,
    pub operations: VecDeque<String>,
}

impl HarnessAdapter for DeterministicFakeAdapter {
    fn capabilities(&self) -> HarnessCapabilities {
        explicit_capabilities(
            "deterministic_fake",
            "1",
            &[
                HarnessCapability::StructuredEvents,
                HarnessCapability::ModelPreflight,
                HarnessCapability::ModelObservation,
                HarnessCapability::PromptDelivery,
                HarnessCapability::Steering,
                HarnessCapability::NativeAbort,
                HarnessCapability::NativeSessionResume,
            ],
        )
    }

    fn preflight(&self, manifest: &LaunchManifest) -> HarnessPreflightResult {
        HarnessPreflightResult {
            allowed: manifest.validate().is_ok(),
            reason: "deterministic fake preflight".into(),
            observed_capabilities: self.capabilities(),
        }
    }

    fn build_launch_manifest(
        &self,
        manifest: LaunchManifest,
    ) -> Result<LaunchManifest, HarnessAdapterError> {
        manifest.validate()?;
        Ok(manifest)
    }

    fn parse_event(&self, _frame: &[u8]) -> Result<Vec<SilentSessionEvent>, HarnessAdapterError> {
        Ok(self.parsed_events.clone())
    }

    fn send_prompt(
        &mut self,
        run: &HarnessRunRef,
        _prompt: PromptPayload,
    ) -> Result<(), HarnessAdapterError> {
        self.operations.push_back(format!("prompt:{}", run.run_id));
        Ok(())
    }

    fn send_input(
        &mut self,
        run: &HarnessRunRef,
        _input: InputPayload,
    ) -> Result<(), HarnessAdapterError> {
        self.operations.push_back(format!("input:{}", run.run_id));
        Ok(())
    }

    fn abort(&mut self, run: &HarnessRunRef) -> Result<(), HarnessAdapterError> {
        self.operations.push_back(format!("abort:{}", run.run_id));
        self.state = HarnessState::Failed;
        Ok(())
    }

    fn query_state(&self, _run: &HarnessRunRef) -> Result<HarnessState, HarnessAdapterError> {
        Ok(self.state)
    }

    fn query_model(
        &self,
        _run: &HarnessRunRef,
    ) -> Result<ObservedHarnessModel, HarnessAdapterError> {
        Ok(self.model.clone())
    }

    fn resume_native_session(&mut self, native_ref: &str) -> Result<(), HarnessAdapterError> {
        self.operations.push_back(format!("resume:{native_ref}"));
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProcessBackendCapabilities {
    pub pty: CapabilitySupport,
    pub rpc: CapabilitySupport,
    pub hard_pause: CapabilitySupport,
    pub process_tree_kill: CapabilitySupport,
    pub output_channels: CapabilitySupport,
}

pub trait ProcessBackendAdapter {
    fn backend_id(&self) -> &'static str;
    fn capabilities(&self) -> ProcessBackendCapabilities;
    fn supports_channel(&self, channel: OutputChannel) -> CapabilitySupport;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DirectProcessBackend;

impl ProcessBackendAdapter for DirectProcessBackend {
    fn backend_id(&self) -> &'static str {
        "direct_process_group"
    }

    fn capabilities(&self) -> ProcessBackendCapabilities {
        ProcessBackendCapabilities {
            pty: CapabilitySupport::Unsupported,
            rpc: CapabilitySupport::Unsupported,
            hard_pause: CapabilitySupport::Supported,
            process_tree_kill: CapabilitySupport::Supported,
            output_channels: CapabilitySupport::Supported,
        }
    }

    fn supports_channel(&self, _channel: OutputChannel) -> CapabilitySupport {
        CapabilitySupport::Supported
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct GenericRpcBackend;

impl ProcessBackendAdapter for GenericRpcBackend {
    fn backend_id(&self) -> &'static str {
        "generic_rpc"
    }

    fn capabilities(&self) -> ProcessBackendCapabilities {
        ProcessBackendCapabilities {
            pty: CapabilitySupport::Unsupported,
            rpc: CapabilitySupport::Supported,
            hard_pause: CapabilitySupport::Unsupported,
            process_tree_kill: CapabilitySupport::Unsupported,
            output_channels: CapabilitySupport::Supported,
        }
    }

    fn supports_channel(&self, channel: OutputChannel) -> CapabilitySupport {
        if channel == OutputChannel::Stdout || channel == OutputChannel::Stderr {
            CapabilitySupport::Unsupported
        } else {
            CapabilitySupport::Supported
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct GenericPtyBackend;

impl ProcessBackendAdapter for GenericPtyBackend {
    fn backend_id(&self) -> &'static str {
        "generic_pty"
    }

    fn capabilities(&self) -> ProcessBackendCapabilities {
        ProcessBackendCapabilities {
            pty: CapabilitySupport::Supported,
            rpc: CapabilitySupport::Unsupported,
            hard_pause: CapabilitySupport::Unsupported,
            process_tree_kill: CapabilitySupport::Supported,
            output_channels: CapabilitySupport::Supported,
        }
    }

    fn supports_channel(&self, channel: OutputChannel) -> CapabilitySupport {
        if channel == OutputChannel::Stdout {
            CapabilitySupport::Supported
        } else {
            CapabilitySupport::Unsupported
        }
    }
}

pub fn explicit_capabilities(
    adapter_id: &str,
    adapter_version: &str,
    supported: &[HarnessCapability],
) -> HarnessCapabilities {
    let all = [
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
    let capabilities = all
        .into_iter()
        .map(|capability| {
            let support = if supported.contains(&capability) {
                CapabilitySupport::Supported
            } else {
                CapabilitySupport::Unsupported
            };
            (capability, support)
        })
        .collect();
    HarnessCapabilities {
        adapter_id: adapter_id.into(),
        adapter_version: adapter_version.into(),
        protocol_major: HARNESS_ADAPTER_PROTOCOL_MAJOR,
        protocol_minor: HARNESS_ADAPTER_PROTOCOL_MINOR,
        capabilities,
    }
}
