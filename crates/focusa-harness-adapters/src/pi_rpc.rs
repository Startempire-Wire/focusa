//! Pi's LF-delimited JSON RPC protocol adapter.
//!
//! The adapter owns only protocol translation. A runner-owned transport writes
//! and reads frames for the exact run; daemon governance remains outside this
//! crate.

use crate::contract::*;
use focusa_core::silent_session::{HarnessKind, ModelBinding, ObservationProvenance};
use focusa_core::silent_session_launch::{LaunchManifest, MissionDelivery};
use focusa_core::silent_session_protocol::{
    CapabilityRequirement, CapabilitySupport, ProtocolVersionOffer,
};
use serde_json::{Map, Value, json};

pub const PI_RPC_ADAPTER_ID: &str = "pi_rpc";
pub const PI_RPC_ADAPTER_VERSION: &str = "pi_rpc.v1";

/// Synchronous request boundary used by the protocol adapter. The production
/// runner transport may bridge this boundary onto its async request correlator;
/// tests use deterministic scripted responses.
pub trait PiRpcTransport {
    fn request(&mut self, run: Option<&RunRef>, command: Value) -> Result<Value, String>;
}

pub struct PiRpcAdapter<T> {
    transport: T,
}

impl<T> PiRpcAdapter<T> {
    pub fn new(transport: T) -> Self {
        Self { transport }
    }

    pub fn transport(&self) -> &T {
        &self.transport
    }

    pub fn transport_mut(&mut self) -> &mut T {
        &mut self.transport
    }
}

pub fn pi_rpc_descriptor() -> HarnessAdapterDescriptor {
    HarnessAdapterDescriptor {
        schema: HARNESS_ADAPTER_PROTOCOL_SCHEMA.into(),
        adapter_id: PI_RPC_ADAPTER_ID.into(),
        adapter_version: PI_RPC_ADAPTER_VERSION.into(),
        protocol_versions: ProtocolVersionOffer::new([HARNESS_ADAPTER_PROTOCOL_VERSION]),
        upstream_protocol: UpstreamProtocolDescriptor {
            protocol_id: "pi.rpc.jsonl".into(),
            // Pi RPC currently publishes command/event shapes but no native
            // protocol-version handshake. Focusa therefore negotiates its own
            // adapter protocol and records the upstream limit explicitly.
            versioning: UpstreamProtocolVersioning::Undeclared,
            observed_version: None,
        },
        capabilities: HarnessCapabilities {
            structured_events: CapabilitySupport::Native,
            stdout_stderr_split: CapabilitySupport::Native,
            semantic_agent_state: CapabilitySupport::Native,
            model_preflight: CapabilitySupport::Native,
            model_observation: CapabilitySupport::Native,
            model_switch: CapabilitySupport::Native,
            thinking_control: CapabilitySupport::Native,
            native_session_resume: CapabilitySupport::Native,
            prompt_delivery: CapabilitySupport::Native,
            steering: CapabilitySupport::Native,
            followup_queue: CapabilitySupport::Native,
            special_keys: CapabilitySupport::Unsupported,
            native_abort: CapabilitySupport::Native,
            hard_pause: CapabilitySupport::Unsupported,
            token_usage: CapabilitySupport::Native,
            cost_usage: CapabilitySupport::Native,
            subscription_entitlement_probe: CapabilitySupport::Unsupported,
        },
        limitations: vec![
            "Pi RPC does not declare an upstream protocol version handshake".into(),
            "special keys and hard pause require separately negotiated backend support".into(),
            "configured model discovery is not a subscription entitlement probe".into(),
        ],
    }
}

impl<T: PiRpcTransport> HarnessAdapter for PiRpcAdapter<T> {
    fn descriptor(&self) -> HarnessAdapterDescriptor {
        pi_rpc_descriptor()
    }

    fn preflight(&self, config: &EffectiveConfig) -> PreflightResult {
        match build_pi_manifest(&self.descriptor(), config) {
            Ok(_) => match self.descriptor().negotiate(&config.negotiation) {
                Ok(contract) => PreflightResult::passed(contract),
                Err(error) => PreflightResult::blocked("protocol_incompatible", error.to_string()),
            },
            Err(error) => {
                let failure_class = match &error {
                    HarnessAdapterError::Negotiation(
                        HarnessNegotiationError::RequiredCapabilityMissing { .. },
                    ) => "capability_missing",
                    HarnessAdapterError::Negotiation(HarnessNegotiationError::Protocol(_)) => {
                        "protocol_incompatible"
                    }
                    _ => "config_invalid",
                };
                PreflightResult::blocked(failure_class, error.to_string())
            }
        }
    }

    fn build_launch_manifest(
        &self,
        config: &EffectiveConfig,
    ) -> Result<LaunchManifest, HarnessAdapterError> {
        build_pi_manifest(&self.descriptor(), config)
    }

    fn parse_event(&self, frame: &[u8]) -> Result<Vec<HarnessEvent>, HarnessAdapterError> {
        let value: Value = serde_json::from_slice(frame)
            .map_err(|error| HarnessAdapterError::InvalidFrame(error.to_string()))?;
        let event_type = value
            .get("type")
            .and_then(Value::as_str)
            .ok_or_else(|| HarnessAdapterError::InvalidFrame("frame type is required".into()))?;
        if event_type == "response" {
            return Ok(vec![]);
        }

        let kind = match event_type {
            "agent_start" => "agent.started",
            "agent_end" => "agent.run_ended",
            "agent_settled" => "agent.settled",
            "turn_start" => "agent.turn_started",
            "turn_end" => "agent.turn_ended",
            "message_start" => "agent.message_started",
            "message_end" => "agent.message_ended",
            "tool_execution_start" => "tool.started",
            "tool_execution_update" => "tool.output",
            "tool_execution_end" if value.get("isError").and_then(Value::as_bool) == Some(true) => {
                "tool.failed"
            }
            "tool_execution_end" => "tool.completed",
            "queue_update" => "agent.queue_updated",
            "compaction_start" => "agent.compaction_started",
            "compaction_end" => "agent.compaction_ended",
            "auto_retry_start" => "retry.scheduled",
            "auto_retry_end" => "retry.completed",
            "extension_error" => "agent.error",
            "extension_ui_request" => "prompt.detected",
            "message_update" => match value
                .get("assistantMessageEvent")
                .and_then(|event| event.get("type"))
                .and_then(Value::as_str)
            {
                Some("text_delta") => "assistant.text_delta",
                Some("thinking_delta") => "assistant.thinking_delta",
                Some("toolcall_start" | "toolcall_delta" | "toolcall_end") => {
                    "agent.tool_call_delta"
                }
                Some("error") => "agent.error",
                _ => "agent.message_delta",
            },
            _ => "harness.unknown",
        };
        Ok(vec![HarnessEvent {
            kind: kind.into(),
            source: PI_RPC_ADAPTER_ID.into(),
            provenance: ObservationProvenance::RuntimeObserved,
            payload: value,
        }])
    }

    fn send_prompt(
        &mut self,
        run: RunRef,
        prompt: PromptPayload,
    ) -> Result<(), HarnessAdapterError> {
        run.validate()?;
        require_capability(
            &self.capabilities(),
            HarnessCapability::PromptDelivery,
            CapabilityRequirement::Deterministic,
        )?;
        if prompt.message.is_empty() {
            return Err(HarnessAdapterError::InvalidConfig(
                "prompt message must not be empty".into(),
            ));
        }
        let command = prompt_command("prompt", prompt.message, prompt.images);
        let response = self
            .transport
            .request(Some(&run), command)
            .map_err(HarnessAdapterError::Transport)?;
        expect_success(response, "prompt")?;
        Ok(())
    }

    fn send_input(&mut self, run: RunRef, input: InputPayload) -> Result<(), HarnessAdapterError> {
        run.validate()?;
        if input.message.is_empty() {
            return Err(HarnessAdapterError::InvalidConfig(
                "input message must not be empty".into(),
            ));
        }
        let (command_name, capability) = match input.kind {
            InputKind::Steering => ("steer", HarnessCapability::Steering),
            InputKind::Followup => ("follow_up", HarnessCapability::FollowupQueue),
        };
        require_capability(
            &self.capabilities(),
            capability,
            CapabilityRequirement::Deterministic,
        )?;
        let command = prompt_command(command_name, input.message, input.images);
        let response = self
            .transport
            .request(Some(&run), command)
            .map_err(HarnessAdapterError::Transport)?;
        expect_success(response, command_name)?;
        Ok(())
    }

    fn abort(&mut self, run: RunRef) -> Result<(), HarnessAdapterError> {
        run.validate()?;
        require_capability(
            &self.capabilities(),
            HarnessCapability::NativeAbort,
            CapabilityRequirement::Native,
        )?;
        let response = self
            .transport
            .request(Some(&run), json!({"type": "abort"}))
            .map_err(HarnessAdapterError::Transport)?;
        expect_success(response, "abort")?;
        Ok(())
    }

    fn query_state(&mut self, run: RunRef) -> Result<HarnessState, HarnessAdapterError> {
        run.validate()?;
        let response = self
            .transport
            .request(Some(&run), json!({"type": "get_state"}))
            .map_err(HarnessAdapterError::Transport)?;
        let data = expect_success(response, "get_state")?;
        let is_streaming = data
            .get("isStreaming")
            .and_then(Value::as_bool)
            .ok_or_else(|| {
                HarnessAdapterError::InvalidResponse("isStreaming is required".into())
            })?;
        let pending_message_count = data
            .get("pendingMessageCount")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        let native_session_ref = data
            .get("sessionFile")
            .or_else(|| data.get("sessionId"))
            .and_then(Value::as_str)
            .map(str::to_owned);
        Ok(HarnessState {
            activity: if is_streaming {
                HarnessActivity::Working
            } else {
                HarnessActivity::Idle
            },
            is_streaming,
            pending_message_count,
            native_session_ref,
            raw: data,
        })
    }

    fn query_model(&mut self, run: RunRef) -> Result<ModelBinding, HarnessAdapterError> {
        run.validate()?;
        require_capability(
            &self.capabilities(),
            HarnessCapability::ModelObservation,
            CapabilityRequirement::Native,
        )?;
        let response = self
            .transport
            .request(Some(&run), json!({"type": "get_state"}))
            .map_err(HarnessAdapterError::Transport)?;
        let data = expect_success(response, "get_state")?;
        let model = data
            .get("model")
            .and_then(Value::as_object)
            .ok_or_else(|| HarnessAdapterError::InvalidResponse("model is required".into()))?;
        Ok(ModelBinding {
            provider: required_string(model, "provider")?,
            model: required_string(model, "id")?,
            thinking: data
                .get("thinkingLevel")
                .and_then(Value::as_str)
                .map(str::to_owned),
        })
    }

    fn resume_native_session(&mut self, native_ref: &str) -> Result<(), HarnessAdapterError> {
        require_capability(
            &self.capabilities(),
            HarnessCapability::NativeSessionResume,
            CapabilityRequirement::Native,
        )?;
        if native_ref.trim().is_empty() {
            return Err(HarnessAdapterError::InvalidConfig(
                "native session reference is required".into(),
            ));
        }
        let response = self
            .transport
            .request(
                None,
                json!({"type": "switch_session", "sessionPath": native_ref}),
            )
            .map_err(HarnessAdapterError::Transport)?;
        expect_success(response, "switch_session")?;
        Ok(())
    }
}

fn build_pi_manifest(
    descriptor: &HarnessAdapterDescriptor,
    config: &EffectiveConfig,
) -> Result<LaunchManifest, HarnessAdapterError> {
    validate_effective_config(descriptor, HarnessKind::Pi, config)?;
    if config.launch_manifest.reproducibility.model_binding
        != config.session.effective_config.model.requested
    {
        return Err(HarnessAdapterError::InvalidConfig(
            "launch model binding does not match effective config".into(),
        ));
    }
    match &config.launch_manifest.mission_delivery {
        MissionDelivery::Rpc { method } if method == "prompt" => {}
        _ => {
            return Err(HarnessAdapterError::InvalidConfig(
                "Pi RPC mission delivery must use the prompt method".into(),
            ));
        }
    }

    let mut manifest = config.launch_manifest.clone();
    require_or_append_option(&mut manifest.argv, "--mode", "rpc")?;
    require_or_append_option(
        &mut manifest.argv,
        "--provider",
        &config.session.effective_config.model.requested.provider,
    )?;
    require_or_append_option(
        &mut manifest.argv,
        "--model",
        &config.session.effective_config.model.requested.model,
    )?;
    if let Some(thinking) = &config.session.effective_config.model.requested.thinking {
        require_or_append_option(&mut manifest.argv, "--thinking", thinking)?;
    }
    manifest.validate()?;
    Ok(manifest)
}

fn require_or_append_option(
    argv: &mut Vec<String>,
    option: &str,
    expected: &str,
) -> Result<(), HarnessAdapterError> {
    let matches: Vec<_> = argv
        .iter()
        .enumerate()
        .filter(|(_, argument)| argument.as_str() == option)
        .collect();
    if matches.is_empty() {
        argv.push(option.into());
        argv.push(expected.into());
        return Ok(());
    }
    if matches.len() != 1 || argv.get(matches[0].0 + 1).map(String::as_str) != Some(expected) {
        return Err(HarnessAdapterError::InvalidConfig(format!(
            "{option} must select the exact effective value"
        )));
    }
    Ok(())
}

fn prompt_command(command_name: &str, message: String, images: Vec<ImagePayload>) -> Value {
    let mut command = Map::from_iter([
        ("type".into(), Value::String(command_name.into())),
        ("message".into(), Value::String(message)),
    ]);
    if !images.is_empty() {
        command.insert(
            "images".into(),
            Value::Array(
                images
                    .into_iter()
                    .map(|image| {
                        json!({
                            "type": "image",
                            "data": image.data_base64,
                            "mimeType": image.mime_type,
                        })
                    })
                    .collect(),
            ),
        );
    }
    Value::Object(command)
}

fn expect_success(response: Value, expected_command: &str) -> Result<Value, HarnessAdapterError> {
    if response.get("type").and_then(Value::as_str) != Some("response")
        || response.get("command").and_then(Value::as_str) != Some(expected_command)
    {
        return Err(HarnessAdapterError::InvalidResponse(format!(
            "expected response for {expected_command}"
        )));
    }
    if response.get("success").and_then(Value::as_bool) != Some(true) {
        return Err(HarnessAdapterError::InvalidResponse(
            response
                .get("error")
                .and_then(Value::as_str)
                .unwrap_or("command failed")
                .into(),
        ));
    }
    Ok(response.get("data").cloned().unwrap_or(Value::Null))
}

fn required_string(object: &Map<String, Value>, key: &str) -> Result<String, HarnessAdapterError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(str::to_owned)
        .ok_or_else(|| HarnessAdapterError::InvalidResponse(format!("{key} is required")))
}
