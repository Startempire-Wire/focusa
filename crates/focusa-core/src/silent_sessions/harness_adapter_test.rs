use crate::silent_sessions::*;

#[test]
fn capability_negotiation_is_versioned_and_unsupported_is_explicit() {
    let adapter = PiRpcAdapter {
        endpoint: "unix:///run/user/1000/pi-rpc.sock".into(),
        adapter_version: "1".into(),
    };
    let capabilities = adapter.capabilities();
    capabilities
        .negotiate(
            HARNESS_ADAPTER_PROTOCOL_MAJOR,
            HARNESS_ADAPTER_PROTOCOL_MINOR,
        )
        .unwrap();
    assert_eq!(
        capabilities.support(HarnessCapability::StructuredEvents),
        CapabilitySupport::Supported
    );
    assert_eq!(
        capabilities.support(HarnessCapability::NativeSessionResume),
        CapabilitySupport::Unsupported
    );
    assert!(
        capabilities
            .negotiate(HARNESS_ADAPTER_PROTOCOL_MAJOR + 1, 0)
            .is_err()
    );
}

#[test]
fn direct_backend_never_claims_rpc_or_pty() {
    let backend = DirectProcessBackend;
    let capabilities = backend.capabilities();
    assert_eq!(capabilities.rpc, CapabilitySupport::Unsupported);
    assert_eq!(capabilities.pty, CapabilitySupport::Unsupported);
    assert_eq!(capabilities.process_tree_kill, CapabilitySupport::Supported);
}
