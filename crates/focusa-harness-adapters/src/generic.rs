//! Truthful declarations for generic harness classes.
//!
//! Generic RPC is intentionally deny-by-default because JSONL framing alone
//! says nothing about commands or semantic events. Generic PTY exposes only the
//! terminal semantics inherent in that class; prompt/blocker state remains
//! heuristic and every structured/model operation is explicitly unsupported.

use crate::contract::*;
use focusa_core::silent_session_protocol::{CapabilitySupport, ProtocolVersionOffer};

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
