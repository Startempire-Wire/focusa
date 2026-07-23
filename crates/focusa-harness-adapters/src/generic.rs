//! Truthful declarations for generic harness classes.
//!
//! Generic RPC is intentionally deny-by-default because JSONL framing alone
//! says nothing about commands or semantic events. Generic PTY exposes only the
//! terminal semantics inherent in that class; prompt/blocker state remains
//! heuristic and every structured/model operation is explicitly unsupported.

use crate::contract::*;
use focusa_core::silent_session::{ObservationProvenance, SilentSessionSemanticActivity};
use focusa_core::silent_session_protocol::{CapabilitySupport, ProtocolVersionOffer};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub const GENERIC_RPC_ADAPTER_ID: &str = "generic_rpc";
pub const GENERIC_PTY_ADAPTER_ID: &str = "generic_pty";
pub const GENERIC_ADAPTER_VERSION: &str = "generic.v1";

pub fn generic_rpc_descriptor() -> HarnessAdapterDescriptor {
    HarnessAdapterDescriptor {
        schema: HARNESS_ADAPTER_PROTOCOL_SCHEMA.into(),
        adapter_id: GENERIC_RPC_ADAPTER_ID.into(),
        adapter_version: GENERIC_ADAPTER_VERSION.into(),
        protocol_versions: ProtocolVersionOffer::new([HARNESS_ADAPTER_PROTOCOL_VERSION]),
        upstream_protocol: UpstreamProtocolDescriptor {
            protocol_id: "generic_jsonl_rpc".into(),
            versioning: UpstreamProtocolVersioning::Undeclared,
            observed_version: None,
        },
        capabilities: HarnessCapabilities::all(CapabilitySupport::Unsupported),
        limitations: vec![
            "no generic command or event semantics are assumed from JSONL framing".into(),
            "a concrete reviewed protocol profile must opt into each capability".into(),
        ],
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenericRpcProtocolProfile {
    pub profile_ref: String,
    pub event_kind_field: String,
    pub output_field: Option<String>,
    pub verified_semantic_kinds: BTreeSet<String>,
    pub control_methods: BTreeMap<String, String>,
    pub capabilities: HarnessCapabilities,
}

pub fn generic_rpc_profile_descriptor(
    profile: &GenericRpcProtocolProfile,
) -> Result<HarnessAdapterDescriptor, HarnessAdapterError> {
    if profile.profile_ref.trim().is_empty() || profile.event_kind_field.trim().is_empty() {
        return Err(HarnessAdapterError::InvalidConfig(
            "generic RPC profile identity/event field is empty".into(),
        ));
    }
    Ok(HarnessAdapterDescriptor {
        schema: HARNESS_ADAPTER_PROTOCOL_SCHEMA.into(),
        adapter_id: format!("{GENERIC_RPC_ADAPTER_ID}:{}", profile.profile_ref),
        adapter_version: GENERIC_ADAPTER_VERSION.into(),
        protocol_versions: ProtocolVersionOffer::new([HARNESS_ADAPTER_PROTOCOL_VERSION]),
        upstream_protocol: UpstreamProtocolDescriptor {
            protocol_id: profile.profile_ref.clone(),
            versioning: UpstreamProtocolVersioning::Undeclared,
            observed_version: None,
        },
        capabilities: profile.capabilities.clone(),
        limitations: vec![
            "only profile-declared methods and verified event kinds are structured".into(),
            "undeclared frames remain generic raw observations".into(),
        ],
    })
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenericParsedFrame {
    pub event: HarnessEvent,
    pub output: Option<String>,
    pub semantic: Option<GenericSemanticLabel>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenericSemanticLabel {
    pub activity: SilentSessionSemanticActivity,
    pub blocker_detected: bool,
    pub confidence: f64,
    pub provenance: ObservationProvenance,
    pub verified: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenericControlKind {
    Prompt,
    Steering,
    Followup,
    SpecialKey,
}

pub fn parse_generic_rpc_frame(
    profile: &GenericRpcProtocolProfile,
    line: &str,
) -> Result<GenericParsedFrame, HarnessAdapterError> {
    if profile.profile_ref.trim().is_empty()
        || profile.event_kind_field.trim().is_empty()
        || line.trim().is_empty()
    {
        return Err(HarnessAdapterError::InvalidFrame(
            "generic RPC profile or frame is empty".into(),
        ));
    }
    let raw: Value = serde_json::from_str(line)
        .map_err(|error| HarnessAdapterError::InvalidFrame(error.to_string()))?;
    let declared_kind = raw
        .get(&profile.event_kind_field)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty());
    let verified = profile.capabilities.structured_events == CapabilitySupport::Native
        && profile.capabilities.semantic_agent_state == CapabilitySupport::Native
        && declared_kind.is_some_and(|kind| profile.verified_semantic_kinds.contains(kind));
    let output = profile
        .output_field
        .as_ref()
        .and_then(|field| raw.get(field))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    let semantic = if verified {
        Some(GenericSemanticLabel {
            activity: semantic_activity(declared_kind.unwrap_or_default()),
            blocker_detected: declared_kind
                .unwrap_or_default()
                .to_ascii_lowercase()
                .contains("block"),
            confidence: 1.0,
            provenance: ObservationProvenance::VerificationConfirmed,
            verified: true,
        })
    } else {
        None
    };
    Ok(GenericParsedFrame {
        event: HarnessEvent {
            kind: declared_kind
                .filter(|_| verified)
                .unwrap_or("generic_rpc.frame")
                .into(),
            source: profile.profile_ref.clone(),
            provenance: if verified {
                ObservationProvenance::VerificationConfirmed
            } else {
                ObservationProvenance::RuntimeObserved
            },
            payload: raw,
        },
        output,
        semantic,
    })
}

pub fn encode_generic_rpc_control(
    profile: &GenericRpcProtocolProfile,
    kind: GenericControlKind,
    text: &str,
) -> Result<Value, HarnessAdapterError> {
    let key = control_key(kind);
    let support = match kind {
        GenericControlKind::Prompt => profile.capabilities.prompt_delivery,
        GenericControlKind::Steering => profile.capabilities.steering,
        GenericControlKind::Followup => profile.capabilities.followup_queue,
        GenericControlKind::SpecialKey => profile.capabilities.special_keys,
    };
    if support == CapabilitySupport::Unsupported {
        return Err(HarnessAdapterError::InvalidResponse(format!(
            "generic RPC profile declares {key} unsupported"
        )));
    }
    let method = profile.control_methods.get(key).ok_or_else(|| {
        HarnessAdapterError::InvalidResponse(format!(
            "generic RPC profile does not declare {key} control"
        ))
    })?;
    if method.trim().is_empty() || text.trim().is_empty() {
        return Err(HarnessAdapterError::InvalidResponse(
            "declared generic RPC control is empty".into(),
        ));
    }
    Ok(serde_json::json!({"method": method, "params": {"text": text}}))
}

pub fn parse_generic_pty_chunk(bytes: &[u8]) -> Result<GenericParsedFrame, HarnessAdapterError> {
    if bytes.is_empty() {
        return Err(HarnessAdapterError::InvalidFrame(
            "generic PTY frame is empty".into(),
        ));
    }
    let text = String::from_utf8_lossy(bytes).into_owned();
    let normalized = text.to_ascii_lowercase();
    let prompt = normalized.contains("input required")
        || normalized.contains("continue?")
        || normalized.ends_with("> ");
    let blocker = normalized.contains("blocked")
        || normalized.contains("cannot continue")
        || normalized.contains("permission denied");
    let semantic = (prompt || blocker).then_some(GenericSemanticLabel {
        activity: if prompt {
            SilentSessionSemanticActivity::WaitingForOperator
        } else {
            SilentSessionSemanticActivity::WaitingForDependency
        },
        blocker_detected: blocker,
        confidence: if blocker { 0.45 } else { 0.35 },
        provenance: ObservationProvenance::TerminalInferred,
        verified: false,
    });
    Ok(GenericParsedFrame {
        event: HarnessEvent {
            kind: "terminal.output".into(),
            source: GENERIC_PTY_ADAPTER_ID.into(),
            provenance: ObservationProvenance::RuntimeObserved,
            payload: serde_json::json!({
                "text": text,
                "stdout_stderr_merged": true,
                "semantic_claim_verified": false,
            }),
        },
        output: Some(text),
        semantic,
    })
}

pub fn encode_generic_pty_control(
    kind: GenericControlKind,
    text: &str,
) -> Result<Vec<u8>, HarnessAdapterError> {
    if text.is_empty() {
        return Err(HarnessAdapterError::InvalidResponse(
            "generic PTY control text is empty".into(),
        ));
    }
    let bytes = match kind {
        GenericControlKind::Prompt
        | GenericControlKind::Steering
        | GenericControlKind::Followup => format!("{text}\n").into_bytes(),
        GenericControlKind::SpecialKey => match text {
            "ENTER" => b"\r".to_vec(),
            "ESCAPE" => vec![0x1b],
            "CTRL_C" => vec![0x03],
            _ => {
                return Err(HarnessAdapterError::InvalidResponse(
                    "unsupported generic PTY special key".into(),
                ));
            }
        },
    };
    Ok(bytes)
}

fn semantic_activity(kind: &str) -> SilentSessionSemanticActivity {
    let kind = kind.to_ascii_lowercase();
    if kind.contains("wait") || kind.contains("input") || kind.contains("prompt") {
        SilentSessionSemanticActivity::WaitingForOperator
    } else if kind.contains("block") {
        SilentSessionSemanticActivity::WaitingForDependency
    } else if kind.contains("complete") || kind.contains("idle") {
        SilentSessionSemanticActivity::IdleBetweenTurns
    } else {
        SilentSessionSemanticActivity::Working
    }
}

const fn control_key(kind: GenericControlKind) -> &'static str {
    match kind {
        GenericControlKind::Prompt => "prompt",
        GenericControlKind::Steering => "steering",
        GenericControlKind::Followup => "followup",
        GenericControlKind::SpecialKey => "special_key",
    }
}

pub fn generic_pty_descriptor() -> HarnessAdapterDescriptor {
    let mut capabilities = HarnessCapabilities::all(CapabilitySupport::Unsupported);
    capabilities.semantic_agent_state = CapabilitySupport::Heuristic;
    capabilities.prompt_delivery = CapabilitySupport::Emulated;
    capabilities.steering = CapabilitySupport::Heuristic;
    capabilities.special_keys = CapabilitySupport::Emulated;
    HarnessAdapterDescriptor {
        schema: HARNESS_ADAPTER_PROTOCOL_SCHEMA.into(),
        adapter_id: GENERIC_PTY_ADAPTER_ID.into(),
        adapter_version: GENERIC_ADAPTER_VERSION.into(),
        protocol_versions: ProtocolVersionOffer::new([HARNESS_ADAPTER_PROTOCOL_VERSION]),
        upstream_protocol: UpstreamProtocolDescriptor {
            protocol_id: "generic_pty_bytes".into(),
            versioning: UpstreamProtocolVersioning::Undeclared,
            observed_version: None,
        },
        capabilities,
        limitations: vec![
            "PTY stdout and stderr are merged".into(),
            "prompt, blocker, and steering observations are heuristic".into(),
            "terminal delivery requires a separately negotiated PTY process backend".into(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile() -> GenericRpcProtocolProfile {
        GenericRpcProtocolProfile {
            profile_ref: "rpc-profile:reviewed".into(),
            event_kind_field: "type".into(),
            output_field: Some("text".into()),
            verified_semantic_kinds: BTreeSet::from(["input_required".into()]),
            control_methods: BTreeMap::from([("prompt".into(), "agent.prompt".into())]),
            capabilities: {
                let mut capabilities = HarnessCapabilities::all(CapabilitySupport::Unsupported);
                capabilities.structured_events = CapabilitySupport::Native;
                capabilities.semantic_agent_state = CapabilitySupport::Native;
                capabilities.prompt_delivery = CapabilitySupport::Native;
                capabilities
            },
        }
    }

    #[test]
    fn generic_rpc_keeps_undeclared_semantics_raw_and_only_verified_profile_kinds_structured() {
        let raw =
            parse_generic_rpc_frame(&profile(), r#"{"type":"mystery_blocked","text":"blocked"}"#)
                .unwrap();
        assert_eq!(raw.event.kind, "generic_rpc.frame");
        assert_eq!(raw.event.provenance, ObservationProvenance::RuntimeObserved);
        assert!(raw.semantic.is_none());

        let verified =
            parse_generic_rpc_frame(&profile(), r#"{"type":"input_required","text":"choose"}"#)
                .unwrap();
        let semantic = verified.semantic.unwrap();
        assert!(semantic.verified);
        assert_eq!(semantic.confidence, 1.0);
        assert_eq!(
            semantic.provenance,
            ObservationProvenance::VerificationConfirmed
        );
    }

    #[test]
    fn rpc_controls_require_declared_method_mapping() {
        assert_eq!(
            encode_generic_rpc_control(&profile(), GenericControlKind::Prompt, "continue").unwrap()
                ["method"],
            "agent.prompt"
        );
        assert!(
            encode_generic_rpc_control(&profile(), GenericControlKind::Steering, "change").is_err()
        );
    }

    #[test]
    fn pty_output_is_merged_and_prompt_blocker_semantics_remain_low_confidence_heuristics() {
        let frame = parse_generic_pty_chunk(b"BLOCKED: permission denied\n> ").unwrap();
        assert_eq!(frame.event.kind, "terminal.output");
        assert_eq!(
            frame.event.provenance,
            ObservationProvenance::RuntimeObserved
        );
        assert_eq!(frame.event.payload["stdout_stderr_merged"], true);
        assert_eq!(frame.event.payload["semantic_claim_verified"], false);
        let semantic = frame.semantic.unwrap();
        assert!(!semantic.verified);
        assert!(semantic.confidence < 0.5);
        assert_eq!(semantic.provenance, ObservationProvenance::TerminalInferred);
        assert!(semantic.blocker_detected);
    }

    #[test]
    fn pty_controls_are_bounded_to_text_and_declared_special_keys() {
        assert_eq!(
            encode_generic_pty_control(GenericControlKind::Prompt, "hello").unwrap(),
            b"hello\n"
        );
        assert_eq!(
            encode_generic_pty_control(GenericControlKind::SpecialKey, "CTRL_C").unwrap(),
            vec![0x03]
        );
        assert!(encode_generic_pty_control(GenericControlKind::SpecialKey, "UNSAFE_KEY").is_err());
    }
}
