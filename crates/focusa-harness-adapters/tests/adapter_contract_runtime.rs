use focusa_core::silent_session::*;
use focusa_core::silent_session_config::{ConfigMutationClass, EffectiveSilentSessionConfig};
use focusa_core::silent_session_launch::*;
use focusa_core::silent_session_protocol::{
    CapabilityRequirement, CapabilitySupport, ProtocolVersionOffer,
};
use focusa_harness_adapters::*;
use serde_json::{Value, json};
use std::collections::{BTreeMap, VecDeque};
use std::path::PathBuf;

fn model() -> ModelBinding {
    ModelBinding {
        provider: "openai-codex".into(),
        model: "gpt-test".into(),
        thinking: Some("high".into()),
    }
}

fn session_config(kind: HarnessKind, adapter_version: &str) -> SilentSessionConfig {
    SilentSessionConfig {
        schema: SILENT_SESSION_CONFIG_SCHEMA.into(),
        identity: SilentSessionIdentityConfig {
            display_name: "adapter test".into(),
            project_root: PathBuf::from("/projects/focusa"),
            project_identity_ref: "project:focusa".into(),
            continuity_id: "adapter-contract-test".into(),
            work_item_ref: Some("focusa-test".into()),
            mission: "exercise adapter contract".into(),
            agent_identity_ref: "agent:test".into(),
            role_profile_ref: "role:test".into(),
        },
        harness: HarnessConfig {
            kind,
            adapter_version: adapter_version.into(),
            native_resume_policy: NativeResumePolicy::Prefer,
        },
        model: SilentSessionModelConfig {
            requested: model(),
            selection_policy: ModelSelectionPolicy::Exact,
            fallback_policy: ModelFallbackPolicy::Disabled,
            allowed_fallbacks: vec![],
            auth_profile_ref: "auth:test".into(),
            require_entitlement_preflight: false,
            require_runtime_model_confirmation: true,
        },
        workspace: WorkspaceConfig {
            strategy: WorkspaceStrategy::IsolatedWorktree,
            source_root: PathBuf::from("/projects/focusa"),
            worktree_root: Some(PathBuf::from("/projects/focusa-worktree")),
            base_ref: Some("main".into()),
            branch_name: Some("focusa/test".into()),
            integration_policy: IntegrationPolicy::Manual,
        },
        bootstrap_target_profile: "silent-session".into(),
        bootstrap_packet_mode: "session_start".into(),
        bootstrap_verification_required: true,
        supervision: SupervisionConfig {
            restart_policy: "never".into(),
            max_process_restarts: 0,
            max_transport_retries: 2,
            retry_backoff_ms: 100,
            retry_budgets: focusa_core::silent_session_retry::default_retry_budgets(),
            soft_pause_timeout_ms: 1000,
            graceful_stop_timeout_ms: 1000,
            checkpoint_interval_seconds: 60,
            checkpoint_event_interval: 100,
            waiting_input_timeout_seconds: 300,
            silent_output_warning_seconds: 120,
        },
        resources: ResourceLimits {
            priority: 0,
            max_wall_clock_seconds: Some(3600),
            max_cpu_percent: None,
            max_memory_bytes: None,
            max_pids: None,
            max_disk_bytes: None,
            max_output_bytes: Some(1_000_000),
            max_tokens: None,
            max_cost_usd: None,
            max_turns: Some(20),
        },
        output: OutputPolicy {
            persist_stdout: true,
            persist_stderr: true,
            persist_semantic_events: true,
            chunk_max_bytes: 1_000_000,
            chunk_max_seconds: 60,
            redaction_profile_ref: "redaction:test".into(),
            operator_projection_budget: 10_000,
            raw_retention_policy_ref: "retention:test".into(),
        },
        governance: GovernancePolicy {
            context_authority_required: true,
            risky_mutation_preflight_required: true,
            destructive_actions_allowed: false,
            writer_lease_required: true,
            completion_receipt_required: true,
            evidence_policy_ref: "evidence:test".into(),
            policy_locks: vec![],
        },
        notifications: NotificationPolicy {
            waiting_input: true,
            blocked: true,
            failed: true,
            completed: true,
            model_mismatch: true,
            budget_pressure: true,
            channels: vec![],
        },
        retention: RetentionConfig {
            policy_ref: "retention:test".into(),
            evidence_hold: false,
        },
    }
}

fn manifest(kind: HarnessKind, adapter_version: &str) -> LaunchManifest {
    let mission = b"exercise adapter contract";
    LaunchManifest {
        schema: LAUNCH_MANIFEST_SCHEMA.into(),
        executable: PathBuf::from("/usr/local/bin/pi"),
        argv: if kind == HarnessKind::Pi {
            vec!["-a".into()]
        } else {
            vec![]
        },
        cwd: PathBuf::from("/projects/focusa-worktree"),
        safe_env: BTreeMap::new(),
        secret_env_refs: vec![],
        mission_artifact: MissionArtifact {
            artifact_ref: "artifact:mission/test".into(),
            sha256: sha256_hex(mission),
            byte_len: mission.len() as u64,
        },
        mission_delivery: MissionDelivery::Rpc {
            method: "prompt".into(),
        },
        stdin_mode: StdinMode::Null,
        stdout_mode: OutputMode::Piped,
        stderr_mode: OutputMode::Piped,
        process_backend: ProcessBackendKind::PosixDirect,
        os_user: "alice".into(),
        resource_limits: LaunchResourceLimits {
            max_wall_clock_seconds: Some(3600),
            max_cpu_percent_basis_points: None,
            max_memory_bytes: None,
            max_pids: None,
            max_disk_bytes: None,
            max_output_bytes: Some(1_000_000),
        },
        resource_mode: ResourceModeRequest {
            mode: ResourceMode::Normal,
            requirement: ResourceModeRequirement::Advisory,
            reason: "test".into(),
            policy_ref: "resource:test".into(),
        },
        trust_policy: TrustPolicy {
            mode: TrustMode::ApprovedNonInteractive,
            operator_approval_ref: "approval:test".into(),
            context_authority_verdict_ref: "verdict:test".into(),
            project_identity_ref: "project:focusa".into(),
            workspace_ref: "workspace:test".into(),
            unexpected_prompt_policy: UnexpectedTrustPromptPolicy::Block,
        },
        harness_kind: kind,
        reproducibility: LaunchReproducibility {
            config_revision_ref: "config:test".into(),
            project_identity_ref: "project:focusa".into(),
            workspace_ref: "workspace:test".into(),
            bootstrap_packet_ref: "bootstrap:test".into(),
            bootstrap_packet_sha256: "b".repeat(64),
            model_binding: model(),
            thinking_level: Some("high".into()),
            adapter_version: adapter_version.into(),
            process_backend_version: "posix_direct.v1".into(),
            resource_policy_ref: "resource:test".into(),
        },
    }
}

fn effective_config(kind: HarnessKind, adapter_version: &str) -> EffectiveConfig {
    let config = session_config(kind.clone(), adapter_version);
    EffectiveConfig {
        session: EffectiveSilentSessionConfig {
            requested_config: config.clone(),
            effective_config: config,
            field_provenance: BTreeMap::new(),
            policy_locks: vec![],
            mutation_classes: BTreeMap::<String, ConfigMutationClass>::new(),
            warnings: vec![],
            validation: ConfigValidationResult {
                valid: true,
                errors: vec![],
                warnings: vec![],
            },
            redacted_config_hash: "c".repeat(64),
        },
        launch_manifest: manifest(kind, adapter_version),
        negotiation: HarnessNegotiationRequest {
            protocol_versions: ProtocolVersionOffer::new([HARNESS_ADAPTER_PROTOCOL_VERSION]),
            required_capabilities: BTreeMap::new(),
        },
    }
}

fn run_ref() -> RunRef {
    RunRef {
        run_id: SilentSessionRunId::new(),
        generation: 1,
    }
}

#[derive(Default)]
struct ScriptedTransport {
    requests: Vec<(Option<RunRef>, Value)>,
    responses: VecDeque<Result<Value, String>>,
}

impl ScriptedTransport {
    fn with_responses(responses: impl IntoIterator<Item = Value>) -> Self {
        Self {
            requests: vec![],
            responses: responses.into_iter().map(Ok).collect(),
        }
    }
}

impl PiRpcTransport for ScriptedTransport {
    fn request(&mut self, run: Option<&RunRef>, command: Value) -> Result<Value, String> {
        self.requests.push((run.cloned(), command));
        self.responses
            .pop_front()
            .unwrap_or_else(|| Err("scripted response exhausted".into()))
    }
}

#[test]
fn pi_rpc_negotiates_truthful_capabilities_and_builds_exact_launch() {
    let adapter = PiRpcAdapter::new(ScriptedTransport::default());
    let descriptor = adapter.descriptor();
    assert_eq!(
        descriptor.capabilities.explicit_entries().len(),
        ALL_HARNESS_CAPABILITIES.len()
    );
    assert_eq!(
        descriptor.capabilities.hard_pause,
        CapabilitySupport::Unsupported
    );
    assert_eq!(
        descriptor.capabilities.special_keys,
        CapabilitySupport::Unsupported
    );
    assert_eq!(
        descriptor.capabilities.subscription_entitlement_probe,
        CapabilitySupport::Unsupported
    );
    assert_eq!(
        descriptor.upstream_protocol.versioning,
        UpstreamProtocolVersioning::Undeclared
    );

    let config = effective_config(HarnessKind::Pi, PI_RPC_ADAPTER_VERSION);
    assert_eq!(adapter.preflight(&config).status, PreflightStatus::Passed);
    let launch = adapter
        .build_launch_manifest(&config)
        .expect("Pi launch should build");
    assert!(launch.argv.windows(2).any(|pair| pair == ["--mode", "rpc"]));
    assert!(
        launch
            .argv
            .windows(2)
            .any(|pair| pair == ["--provider", "openai-codex"])
    );
    assert!(
        launch
            .argv
            .windows(2)
            .any(|pair| pair == ["--model", "gpt-test"])
    );
    assert!(
        launch
            .argv
            .windows(2)
            .any(|pair| pair == ["--thinking", "high"])
    );

    let mut missing = config.clone();
    missing.negotiation.required_capabilities.insert(
        HarnessCapability::SubscriptionEntitlementProbe,
        CapabilityRequirement::Available,
    );
    let blocked = adapter.preflight(&missing);
    assert_eq!(blocked.status, PreflightStatus::Blocked);
    assert_eq!(blocked.failure_class.as_deref(), Some("capability_missing"));

    let mut incompatible = config;
    incompatible.negotiation.protocol_versions = ProtocolVersionOffer::new([99]);
    assert_eq!(
        adapter.preflight(&incompatible).status,
        PreflightStatus::Blocked
    );
}

#[test]
fn pi_rpc_strict_entitlement_policy_blocks_when_probe_is_unsupported() {
    let adapter = PiRpcAdapter::new(ScriptedTransport::default());
    let mut config = effective_config(HarnessKind::Pi, PI_RPC_ADAPTER_VERSION);
    config
        .session
        .effective_config
        .model
        .require_entitlement_preflight = true;

    let blocked = adapter.preflight(&config);
    assert_eq!(blocked.status, PreflightStatus::Blocked);
    assert_eq!(
        blocked.failure_class.as_deref(),
        Some("entitlement_unknown")
    );
    assert!(blocked.negotiated_contract.is_none());
}

#[test]
fn pi_rpc_control_query_and_event_translation_follow_published_jsonl_contract() {
    let state_response = json!({
        "type":"response",
        "command":"get_state",
        "success":true,
        "data": {
            "model":{"provider":"openai-codex","id":"gpt-test"},
            "thinkingLevel":"high",
            "isStreaming":true,
            "pendingMessageCount":1,
            "sessionFile":"/sessions/test.jsonl",
            "sessionId":"native-session-id"
        }
    });
    let responses = [
        json!({"type":"response","command":"prompt","success":true}),
        json!({"type":"response","command":"steer","success":true}),
        json!({"type":"response","command":"follow_up","success":true}),
        state_response.clone(),
        state_response,
        json!({
            "type":"response",
            "command":"get_session_stats",
            "success":true,
            "data": {
                "tokens": {
                    "input":50,"output":10,"cacheRead":40,"cacheWrite":5,"total":105
                },
                "cost":0.45,
                "contextUsage":{"tokens":60,"contextWindow":200,"percent":30.0}
            }
        }),
        json!({"type":"response","command":"switch_session","success":true,"data":{"cancelled":false}}),
        json!({"type":"response","command":"abort","success":true}),
    ];
    let mut adapter = PiRpcAdapter::new(ScriptedTransport::with_responses(responses));
    let run = run_ref();
    adapter
        .send_prompt(
            run.clone(),
            PromptPayload {
                message: "do the bounded slice".into(),
                images: vec![],
            },
        )
        .expect("prompt should be accepted");
    adapter
        .send_input(
            run.clone(),
            InputPayload {
                kind: InputKind::Steering,
                message: "use the focused test".into(),
                images: vec![],
            },
        )
        .expect("steering should be accepted");
    adapter
        .send_input(
            run.clone(),
            InputPayload {
                kind: InputKind::Followup,
                message: "queue the verification summary".into(),
                images: vec![],
            },
        )
        .expect("follow-up should be accepted");
    let state = adapter
        .query_state(run.clone())
        .expect("state and native session ref should parse");
    assert_eq!(state.activity, HarnessActivity::Working);
    assert_eq!(
        state.native_session_ref.as_deref(),
        Some("/sessions/test.jsonl")
    );
    assert_eq!(
        adapter
            .query_model(run.clone())
            .expect("model should parse"),
        model()
    );
    let usage = adapter
        .query_usage(run.clone())
        .expect("token, cost, and context usage should parse");
    assert_eq!(usage.input_tokens, 50);
    assert_eq!(usage.output_tokens, 10);
    assert_eq!(usage.cache_read_tokens, 40);
    assert_eq!(usage.cache_write_tokens, 5);
    assert_eq!(usage.total_tokens, 105);
    assert_eq!(usage.cost_usd, 0.45);
    assert_eq!(usage.context_tokens, Some(60));
    assert_eq!(usage.context_window, Some(200));
    assert_eq!(usage.context_percent, Some(30.0));
    adapter
        .resume_native_session("/sessions/test.jsonl")
        .expect("native session should switch");
    adapter
        .abort(run.clone())
        .expect("abort should be accepted");

    let requests = &adapter.transport().requests;
    assert_eq!(requests.len(), 8);
    assert_eq!(requests[0].0.as_ref(), Some(&run));
    assert_eq!(requests[0].1["type"], "prompt");
    assert_eq!(requests[1].0.as_ref(), Some(&run));
    assert_eq!(requests[1].1["type"], "steer");
    assert_eq!(requests[2].0.as_ref(), Some(&run));
    assert_eq!(requests[2].1["type"], "follow_up");
    assert_eq!(requests[3].1["type"], "get_state");
    assert_eq!(requests[4].1["type"], "get_state");
    assert_eq!(requests[5].1["type"], "get_session_stats");
    assert!(requests[6].0.is_none());
    assert_eq!(requests[6].1["type"], "switch_session");
    assert_eq!(requests[7].1["type"], "abort");

    let text = adapter
        .parse_event(
            br#"{"type":"message_update","assistantMessageEvent":{"type":"text_delta","delta":"hello"}}"#,
        )
        .expect("text event should parse");
    assert_eq!(text[0].kind, "assistant.text_delta");
    let failed_tool = adapter
        .parse_event(br#"{"type":"tool_execution_end","toolCallId":"call-1","isError":true}"#)
        .expect("tool event should parse");
    assert_eq!(failed_tool[0].kind, "tool.failed");
    let unknown = adapter
        .parse_event(br#"{"type":"future_pi_event","value":7}"#)
        .expect("unknown additive event should be preserved");
    assert_eq!(unknown[0].kind, "harness.unknown");
    assert_eq!(unknown[0].payload["type"], "future_pi_event");

    let mut invalid = PiRpcAdapter::new(ScriptedTransport::default());
    assert!(matches!(
        invalid.send_input(
            RunRef {
                run_id: run.run_id,
                generation: 0,
            },
            InputPayload {
                kind: InputKind::Steering,
                message: "stale control".into(),
                images: vec![],
            },
        ),
        Err(HarnessAdapterError::InvalidRunRef)
    ));
    assert!(invalid.transport().requests.is_empty());

    let mut cancelled = PiRpcAdapter::new(ScriptedTransport::with_responses([json!({
        "type":"response","command":"switch_session","success":true,"data":{"cancelled":true}
    })]));
    assert!(matches!(
        cancelled.resume_native_session("/sessions/cancelled.jsonl"),
        Err(HarnessAdapterError::InvalidResponse(_))
    ));
}

#[test]
fn pi_rpc_normalizes_structured_turn_message_tool_model_and_usage_events() {
    let adapter = PiRpcAdapter::new(ScriptedTransport::default());

    let turn_start = adapter
        .parse_event(br#"{"type":"turn_start"}"#)
        .expect("turn start should parse");
    assert_eq!(turn_start[0].kind, "agent.turn_started");
    let turn_end = adapter
        .parse_event(br#"{"type":"turn_end","message":{"role":"assistant"},"toolResults":[]}"#)
        .expect("turn end should parse");
    assert_eq!(turn_end[0].kind, "agent.turn_ended");
    assert!(turn_end[0].payload["toolResults"].is_array());

    let delta_kinds = [
        ("text_start", "assistant.text_started"),
        ("text_delta", "assistant.text_delta"),
        ("text_end", "assistant.text_completed"),
        ("thinking_start", "assistant.thinking_started"),
        ("thinking_delta", "assistant.thinking_delta"),
        ("thinking_end", "assistant.thinking_completed"),
        ("toolcall_start", "agent.tool_call_started"),
        ("toolcall_delta", "agent.tool_call_delta"),
        ("toolcall_end", "agent.tool_call_completed"),
        ("done", "assistant.message_completed"),
    ];
    for (pi_kind, focusa_kind) in delta_kinds {
        let frame = serde_json::to_vec(&json!({
            "type":"message_update",
            "assistantMessageEvent":{"type":pi_kind}
        }))
        .unwrap();
        let parsed = adapter.parse_event(&frame).expect("delta should parse");
        assert_eq!(parsed[0].kind, focusa_kind);
    }

    let message_end = adapter
        .parse_event(
            br#"{
                "type":"message_end",
                "message":{
                    "role":"assistant",
                    "api":"openai-responses",
                    "provider":"openai-codex",
                    "model":"gpt-test",
                    "usage":{
                        "input":12,"output":4,"cacheRead":5,"cacheWrite":1,
                        "totalTokens":22,
                        "cost":{"input":0.1,"output":0.2,"cacheRead":0.01,"cacheWrite":0.02,"total":0.33}
                    },
                    "stopReason":"stop"
                }
            }"#,
        )
        .expect("assistant completion should produce semantic observations");
    assert_eq!(
        message_end
            .iter()
            .map(|event| event.kind.as_str())
            .collect::<Vec<_>>(),
        vec!["agent.message_ended", "model.observed", "usage.observed"]
    );
    assert_eq!(message_end[1].payload["provider"], "openai-codex");
    assert_eq!(message_end[1].payload["model"], "gpt-test");
    assert_eq!(message_end[2].payload["input_tokens"], 12);
    assert_eq!(message_end[2].payload["total_tokens"], 22);
    assert_eq!(message_end[2].payload["cost_usd"], 0.33);

    for (frame, expected) in [
        (
            br#"{"type":"tool_execution_start","toolCallId":"call-1","toolName":"bash","args":{"command":"pwd"}}"#.as_slice(),
            "tool.started",
        ),
        (
            br#"{"type":"tool_execution_update","toolCallId":"call-1","toolName":"bash","partialResult":{"content":[]}}"#.as_slice(),
            "tool.output",
        ),
        (
            br#"{"type":"tool_execution_end","toolCallId":"call-1","toolName":"bash","result":{"content":[]},"isError":false}"#.as_slice(),
            "tool.completed",
        ),
        (
            br#"{"type":"tool_execution_end","toolCallId":"call-2","toolName":"bash","result":{"content":[]},"isError":true}"#.as_slice(),
            "tool.failed",
        ),
    ] {
        let parsed = adapter.parse_event(frame).expect("tool event should parse");
        assert_eq!(parsed[0].kind, expected);
        assert!(parsed[0].payload["toolCallId"].is_string());
    }
}

#[test]
fn deterministic_fake_adapter_replays_identical_frames_and_controls() {
    let mut adapter = DeterministicFakeAdapter::new(model());
    let config = effective_config(HarnessKind::GenericRpc, DETERMINISTIC_FAKE_ADAPTER_VERSION);
    assert_eq!(adapter.preflight(&config).status, PreflightStatus::Passed);
    adapter
        .build_launch_manifest(&config)
        .expect("fake launch should validate");

    let frame = br#"{"events":[{"kind":"agent.started","payload":{"step":1}},{"kind":"tool.completed","payload":{"step":2}}]}"#;
    assert_eq!(
        adapter.parse_event(frame).expect("first parse"),
        adapter.parse_event(frame).expect("second parse")
    );

    let run = run_ref();
    adapter
        .send_prompt(
            run.clone(),
            PromptPayload {
                message: "deterministic prompt".into(),
                images: vec![],
            },
        )
        .expect("fake prompt should run");
    adapter
        .send_input(
            run.clone(),
            InputPayload {
                kind: InputKind::Followup,
                message: "deterministic followup".into(),
                images: vec![],
            },
        )
        .expect("fake followup should queue");
    let state = adapter
        .query_state(run.clone())
        .expect("fake state should be deterministic");
    assert_eq!(state.activity, HarnessActivity::Working);
    assert_eq!(state.pending_message_count, 1);
    assert_eq!(adapter.query_model(run.clone()).unwrap(), model());
    assert_eq!(adapter.query_usage(run.clone()).unwrap().total_tokens, 0);
    adapter.abort(run).expect("fake abort should run");
    adapter
        .resume_native_session("fake-native-ref")
        .expect("fake resume should run");
    assert_eq!(adapter.control_log().len(), 4);
}

#[test]
fn generic_rpc_and_pty_declarations_never_inflate_unknown_capabilities() {
    let rpc = generic_rpc_descriptor();
    assert!(
        rpc.capabilities
            .explicit_entries()
            .values()
            .all(|support| *support == CapabilitySupport::Unsupported)
    );

    let pty = generic_pty_descriptor();
    assert_eq!(
        pty.capabilities.structured_events,
        CapabilitySupport::Unsupported
    );
    assert_eq!(
        pty.capabilities.stdout_stderr_split,
        CapabilitySupport::Unsupported
    );
    assert_eq!(
        pty.capabilities.semantic_agent_state,
        CapabilitySupport::Heuristic
    );
    assert_eq!(
        pty.capabilities.prompt_delivery,
        CapabilitySupport::Emulated
    );
    let request = HarnessNegotiationRequest {
        protocol_versions: ProtocolVersionOffer::new([HARNESS_ADAPTER_PROTOCOL_VERSION]),
        required_capabilities: BTreeMap::from([(
            HarnessCapability::SemanticAgentState,
            CapabilityRequirement::Deterministic,
        )]),
    };
    assert!(matches!(
        pty.negotiate(&request),
        Err(HarnessNegotiationError::RequiredCapabilityMissing {
            actual: CapabilitySupport::Heuristic,
            ..
        })
    ));
}
