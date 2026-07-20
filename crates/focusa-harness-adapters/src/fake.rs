//! Deterministic in-memory adapter used by runtime and fault-injection tests.

use crate::contract::*;
use focusa_core::silent_session::{HarnessKind, ModelBinding, ObservationProvenance};
use focusa_core::silent_session_protocol::CapabilitySupport;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub const DETERMINISTIC_FAKE_ADAPTER_ID: &str = "deterministic_fake";
pub const DETERMINISTIC_FAKE_ADAPTER_VERSION: &str = "deterministic_fake.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FakeControl {
    Prompt { run: RunRef, prompt: PromptPayload },
    Input { run: RunRef, input: InputPayload },
    Abort { run: RunRef },
    ResumeNativeSession { native_ref: String },
}

pub struct DeterministicFakeAdapter {
    model: ModelBinding,
    controls: Vec<FakeControl>,
    states: BTreeMap<RunRef, HarnessState>,
}

impl DeterministicFakeAdapter {
    pub fn new(model: ModelBinding) -> Self {
        Self {
            model,
            controls: vec![],
            states: BTreeMap::new(),
        }
    }

    pub fn control_log(&self) -> &[FakeControl] {
        &self.controls
    }

    fn idle_state(run: &RunRef) -> HarnessState {
        HarnessState {
            activity: HarnessActivity::Idle,
            is_streaming: false,
            pending_message_count: 0,
            native_session_ref: Some(format!("fake-session:{}", run.run_id)),
            raw: json!({"deterministic": true}),
        }
    }
}

impl HarnessAdapter for DeterministicFakeAdapter {
    fn descriptor(&self) -> HarnessAdapterDescriptor {
        HarnessAdapterDescriptor {
            schema: HARNESS_ADAPTER_PROTOCOL_SCHEMA.into(),
            adapter_id: DETERMINISTIC_FAKE_ADAPTER_ID.into(),
            adapter_version: DETERMINISTIC_FAKE_ADAPTER_VERSION.into(),
            protocol_versions: focusa_core::silent_session_protocol::ProtocolVersionOffer::new([
                HARNESS_ADAPTER_PROTOCOL_VERSION,
            ]),
            upstream_protocol: UpstreamProtocolDescriptor {
                protocol_id: "focusa.deterministic_fake_json.v1".into(),
                versioning: UpstreamProtocolVersioning::Declared,
                observed_version: Some("1".into()),
            },
            capabilities: HarnessCapabilities::all(CapabilitySupport::Native),
            limitations: vec!["test-only adapter; it performs no provider or process I/O".into()],
        }
    }

    fn preflight(&self, config: &EffectiveConfig) -> PreflightResult {
        match validate_effective_config(&self.descriptor(), HarnessKind::GenericRpc, config) {
            Ok(contract) => PreflightResult::passed(contract),
            Err(error) => PreflightResult::blocked("harness_preflight_failed", error.to_string()),
        }
    }

    fn build_launch_manifest(
        &self,
        config: &EffectiveConfig,
    ) -> Result<focusa_core::silent_session_launch::LaunchManifest, HarnessAdapterError> {
        validate_effective_config(&self.descriptor(), HarnessKind::GenericRpc, config)?;
        Ok(config.launch_manifest.clone())
    }

    fn parse_event(&self, frame: &[u8]) -> Result<Vec<HarnessEvent>, HarnessAdapterError> {
        let value: Value = serde_json::from_slice(frame)
            .map_err(|error| HarnessAdapterError::InvalidFrame(error.to_string()))?;
        let events = value
            .get("events")
            .and_then(Value::as_array)
            .ok_or_else(|| HarnessAdapterError::InvalidFrame("events array is required".into()))?;
        events
            .iter()
            .map(|event| {
                let kind = event
                    .get("kind")
                    .and_then(Value::as_str)
                    .filter(|kind| !kind.trim().is_empty())
                    .ok_or_else(|| {
                        HarnessAdapterError::InvalidFrame("event kind is required".into())
                    })?;
                Ok(HarnessEvent {
                    kind: kind.into(),
                    source: "deterministic_fake".into(),
                    provenance: ObservationProvenance::RuntimeObserved,
                    payload: event.get("payload").cloned().unwrap_or(Value::Null),
                })
            })
            .collect()
    }

    fn send_prompt(
        &mut self,
        run: RunRef,
        prompt: PromptPayload,
    ) -> Result<(), HarnessAdapterError> {
        run.validate()?;
        if prompt.message.is_empty() {
            return Err(HarnessAdapterError::InvalidConfig(
                "prompt message must not be empty".into(),
            ));
        }
        self.states.insert(
            run.clone(),
            HarnessState {
                activity: HarnessActivity::Working,
                is_streaming: true,
                pending_message_count: 0,
                native_session_ref: Some(format!("fake-session:{}", run.run_id)),
                raw: json!({"deterministic": true}),
            },
        );
        self.controls.push(FakeControl::Prompt { run, prompt });
        Ok(())
    }

    fn send_input(&mut self, run: RunRef, input: InputPayload) -> Result<(), HarnessAdapterError> {
        run.validate()?;
        if input.message.is_empty() {
            return Err(HarnessAdapterError::InvalidConfig(
                "input message must not be empty".into(),
            ));
        }
        let state = self
            .states
            .entry(run.clone())
            .or_insert_with(|| Self::idle_state(&run));
        state.pending_message_count = state.pending_message_count.saturating_add(1);
        self.controls.push(FakeControl::Input { run, input });
        Ok(())
    }

    fn abort(&mut self, run: RunRef) -> Result<(), HarnessAdapterError> {
        run.validate()?;
        self.states.insert(
            run.clone(),
            HarnessState {
                activity: HarnessActivity::Aborted,
                is_streaming: false,
                pending_message_count: 0,
                native_session_ref: Some(format!("fake-session:{}", run.run_id)),
                raw: json!({"deterministic": true}),
            },
        );
        self.controls.push(FakeControl::Abort { run });
        Ok(())
    }

    fn query_state(&mut self, run: RunRef) -> Result<HarnessState, HarnessAdapterError> {
        run.validate()?;
        Ok(self
            .states
            .get(&run)
            .cloned()
            .unwrap_or_else(|| Self::idle_state(&run)))
    }

    fn query_model(&mut self, run: RunRef) -> Result<ModelBinding, HarnessAdapterError> {
        run.validate()?;
        Ok(self.model.clone())
    }

    fn query_usage(&mut self, run: RunRef) -> Result<HarnessUsage, HarnessAdapterError> {
        run.validate()?;
        Ok(HarnessUsage {
            input_tokens: 0,
            output_tokens: 0,
            cache_read_tokens: 0,
            cache_write_tokens: 0,
            total_tokens: 0,
            cost_usd: 0.0,
            context_tokens: Some(0),
            context_window: Some(1),
            context_percent: Some(0.0),
            raw: json!({"deterministic": true}),
        })
    }

    fn resume_native_session(&mut self, native_ref: &str) -> Result<(), HarnessAdapterError> {
        if native_ref.trim().is_empty() {
            return Err(HarnessAdapterError::InvalidConfig(
                "native session reference is required".into(),
            ));
        }
        self.controls.push(FakeControl::ResumeNativeSession {
            native_ref: native_ref.into(),
        });
        Ok(())
    }
}
