pub mod checkpoint;
pub mod http;
pub mod journal;

use agent_stateful_cognitive_runtime::{
    ClientToolRequest, ClientToolResult, RuntimeBinding, RuntimeContractError, RuntimeMode,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeSet, HashMap},
    future::Future,
    pin::Pin,
    sync::Arc,
};
use thiserror::Error;
use tokio::sync::Mutex;
use uuid::Uuid;

pub type AdapterFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LettaOperation {
    SendTurn,
    ContinueClientToolThroughPi,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LettaCapabilityContract {
    pub schema: String,
    pub supported_operations: BTreeSet<LettaOperation>,
    pub authentication: String,
    pub identity_fields: Vec<String>,
    pub cognitive_loop_owner: String,
    pub client_tool_owner: String,
    pub forbidden_direct_capabilities: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LettaScopeBinding {
    pub schema: String,
    pub project_root: String,
    pub continuity_id: String,
    pub workpoint_id: String,
    pub provider_agent_id: String,
    pub provider_thread_id: String,
    pub epoch_id: Uuid,
    pub replay_key_prefix: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LettaResumeDecision {
    pub schema: String,
    pub status: String,
    pub binding: Option<LettaScopeBinding>,
    pub failure_class: Option<String>,
    pub quarantined_candidate_digest: Option<String>,
}

impl LettaScopeBinding {
    pub fn validate_against_runtime(
        &self,
        runtime: &RuntimeBinding,
    ) -> Result<(), LettaAdapterError> {
        runtime.validate()?;
        if self.schema != "focusa.letta_scope_binding.v1"
            || self.project_root.trim().is_empty()
            || self.continuity_id.trim().is_empty()
            || self.workpoint_id.trim().is_empty()
            || self.provider_agent_id.trim().is_empty()
            || self.provider_thread_id.trim().is_empty()
            || self.replay_key_prefix.trim().is_empty()
        {
            return Err(LettaAdapterError::IncompleteIdentity("letta_scope_binding"));
        }
        if runtime.mode != RuntimeMode::LettaManaged
            || self.project_root != runtime.epoch.project_root
            || self.continuity_id != runtime.epoch.continuity_id
            || self.provider_agent_id != runtime.provider_agent_id.as_deref().unwrap_or("")
            || self.epoch_id != runtime.epoch.epoch_id
        {
            return Err(LettaAdapterError::ScopeMismatch);
        }
        Ok(())
    }
}

pub fn evaluate_letta_resume(
    expected: &LettaScopeBinding,
    candidate: LettaScopeBinding,
    runtime: &RuntimeBinding,
) -> LettaResumeDecision {
    let exact = candidate.validate_against_runtime(runtime).is_ok() && expected == &candidate;
    if exact {
        LettaResumeDecision {
            schema: "focusa.letta_resume_decision.v1".into(),
            status: "resumed".into(),
            binding: Some(candidate),
            failure_class: None,
            quarantined_candidate_digest: None,
        }
    } else {
        let digest = serde_json::to_vec(&candidate)
            .map(|bytes| format!("sha256:{}", hex::encode(Sha256::digest(bytes))))
            .unwrap_or_else(|_| "sha256:unavailable".into());
        LettaResumeDecision {
            schema: "focusa.letta_resume_decision.v1".into(),
            status: "quarantined".into(),
            binding: None,
            failure_class: Some("foreign_scope_or_thread".into()),
            quarantined_candidate_digest: Some(digest),
        }
    }
}

pub fn canonical_letta_capability_contract() -> LettaCapabilityContract {
    LettaCapabilityContract {
        schema: "focusa.letta_capability_contract.v1".into(),
        supported_operations: BTreeSet::from([
            LettaOperation::SendTurn,
            LettaOperation::ContinueClientToolThroughPi,
        ]),
        authentication: "bearer_from_credential_provider".into(),
        identity_fields: [
            "project_root",
            "continuity_id",
            "agent_instance_id",
            "provider_agent_id",
            "epoch_id",
            "event_id",
            "request_id",
        ]
        .into_iter()
        .map(str::to_string)
        .collect(),
        cognitive_loop_owner: "mode_exact_pi_or_letta".into(),
        client_tool_owner: "pi_gateway_only".into(),
        forbidden_direct_capabilities: BTreeSet::from([
            "browser_cookie".into(),
            "raw_session_secret".into(),
            "unrestricted_browser".into(),
            "unrestricted_filesystem".into(),
            "unrestricted_terminal".into(),
            "wallet_key".into(),
        ]),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LettaTurnRequest {
    pub request_id: Uuid,
    pub event_id: String,
    pub provider_agent_id: String,
    pub epoch_id: Uuid,
    pub input: String,
    pub input_digest: String,
    pub continuation: Option<ClientToolResult>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum LettaTurnResponse {
    Completed {
        response_digest: String,
        evidence_refs: Vec<String>,
    },
    ClientToolRequested {
        request: ClientToolRequest,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LettaTurnReceipt {
    pub schema: String,
    pub request_id: Uuid,
    pub event_id: String,
    pub provider_agent_id: String,
    pub epoch_id: Uuid,
    pub response_digest: String,
    pub evidence_refs: Vec<String>,
    pub tool_continuations: u32,
}

pub trait LettaTransport: Send + Sync {
    fn send_turn<'a>(
        &'a self,
        request: &'a LettaTurnRequest,
    ) -> AdapterFuture<'a, Result<LettaTurnResponse, LettaAdapterError>>;
}

pub trait PiClientToolGateway: Send + Sync {
    fn execute<'a>(
        &'a self,
        binding: &'a RuntimeBinding,
        request: &'a ClientToolRequest,
    ) -> AdapterFuture<'a, Result<ClientToolResult, LettaAdapterError>>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LettaTurnIntent {
    pub event_id: String,
    pub request_id: Uuid,
    pub provider_agent_id: String,
    pub epoch_id: Uuid,
    pub input_digest: String,
}

impl From<&LettaTurnRequest> for LettaTurnIntent {
    fn from(request: &LettaTurnRequest) -> Self {
        Self {
            event_id: request.event_id.clone(),
            request_id: request.request_id,
            provider_agent_id: request.provider_agent_id.clone(),
            epoch_id: request.epoch_id,
            input_digest: request.input_digest.clone(),
        }
    }
}

pub trait LettaTurnJournal: Send + Sync {
    fn reserve<'a>(
        &'a self,
        request: &'a LettaTurnRequest,
    ) -> AdapterFuture<'a, Result<Uuid, LettaAdapterError>>;
    fn find<'a>(
        &'a self,
        event_id: &'a str,
    ) -> AdapterFuture<'a, Result<Option<LettaTurnReceipt>, LettaAdapterError>>;
    fn append<'a>(
        &'a self,
        receipt: &'a LettaTurnReceipt,
    ) -> AdapterFuture<'a, Result<(), LettaAdapterError>>;
}

#[derive(Debug, Error)]
pub enum LettaAdapterError {
    #[error("runtime contract rejected Letta operation: {0}")]
    Runtime(#[from] RuntimeContractError),
    #[error("runtime binding is not Letta-managed")]
    WrongRuntimeMode,
    #[error("provider agent does not match runtime binding")]
    AgentMismatch,
    #[error("Letta scope or thread does not match runtime binding")]
    ScopeMismatch,
    #[error("turn identity is incomplete: {0}")]
    IncompleteIdentity(&'static str),
    #[error("Letta transport failed: {0}")]
    Transport(String),
    #[error("Pi client-tool gateway failed: {0}")]
    Gateway(String),
    #[error("turn journal failed: {0}")]
    Journal(String),
    #[error("Letta exceeded the bounded client-tool continuation budget")]
    ToolContinuationBudgetExceeded,
    #[error("Letta response omitted a digest")]
    MissingResponseDigest,
}

pub struct LettaAdapter<T, G, J> {
    transport: Arc<T>,
    gateway: Arc<G>,
    journal: Arc<J>,
    agent_locks: Mutex<HashMap<String, Arc<Mutex<()>>>>,
    max_tool_continuations: u32,
}

impl<T, G, J> LettaAdapter<T, G, J>
where
    T: LettaTransport,
    G: PiClientToolGateway,
    J: LettaTurnJournal,
{
    pub fn new(transport: Arc<T>, gateway: Arc<G>, journal: Arc<J>) -> Self {
        Self {
            transport,
            gateway,
            journal,
            agent_locks: Mutex::new(HashMap::new()),
            max_tool_continuations: 8,
        }
    }

    pub async fn run_turn(
        &self,
        binding: &RuntimeBinding,
        mut request: LettaTurnRequest,
    ) -> Result<LettaTurnReceipt, LettaAdapterError> {
        binding.validate()?;
        if binding.mode != RuntimeMode::LettaManaged {
            return Err(LettaAdapterError::WrongRuntimeMode);
        }
        let expected_agent = binding
            .provider_agent_id
            .as_deref()
            .ok_or(LettaAdapterError::IncompleteIdentity("provider_agent_id"))?;
        if request.provider_agent_id != expected_agent || request.epoch_id != binding.epoch.epoch_id
        {
            return Err(LettaAdapterError::AgentMismatch);
        }
        if request.event_id.trim().is_empty() {
            return Err(LettaAdapterError::IncompleteIdentity("event_id"));
        }
        if request.input_digest.trim().is_empty() {
            return Err(LettaAdapterError::IncompleteIdentity("input_digest"));
        }

        let agent_lock = {
            let mut locks = self.agent_locks.lock().await;
            locks
                .entry(request.provider_agent_id.clone())
                .or_insert_with(|| Arc::new(Mutex::new(())))
                .clone()
        };
        let _turn_guard = agent_lock.lock().await;
        if let Some(receipt) = self.journal.find(&request.event_id).await? {
            return Ok(receipt);
        }
        // Persist intent before remote I/O. A retry after an uncertain outcome
        // reuses the first request id and therefore the same remote idempotency key.
        request.request_id = self.journal.reserve(&request).await?;

        let mut tool_continuations = 0;
        loop {
            match self.transport.send_turn(&request).await? {
                LettaTurnResponse::Completed {
                    response_digest,
                    evidence_refs,
                } => {
                    if response_digest.trim().is_empty() {
                        return Err(LettaAdapterError::MissingResponseDigest);
                    }
                    let receipt = LettaTurnReceipt {
                        schema: "focusa.letta_turn_receipt.v1".into(),
                        request_id: request.request_id,
                        event_id: request.event_id.clone(),
                        provider_agent_id: request.provider_agent_id.clone(),
                        epoch_id: request.epoch_id,
                        response_digest,
                        evidence_refs,
                        tool_continuations,
                    };
                    self.journal.append(&receipt).await?;
                    return Ok(receipt);
                }
                LettaTurnResponse::ClientToolRequested {
                    request: tool_request,
                } => {
                    if tool_continuations >= self.max_tool_continuations {
                        return Err(LettaAdapterError::ToolContinuationBudgetExceeded);
                    }
                    binding.authorize_tool_request(&tool_request)?;
                    request.continuation =
                        Some(self.gateway.execute(binding, &tool_request).await?);
                    request.input.clear();
                    tool_continuations += 1;
                }
            }
        }
    }
}

#[derive(Default)]
pub struct InMemoryTurnJournal {
    intents: Mutex<HashMap<String, LettaTurnIntent>>,
    receipts: Mutex<HashMap<String, LettaTurnReceipt>>,
}

impl LettaTurnJournal for InMemoryTurnJournal {
    fn reserve<'a>(
        &'a self,
        request: &'a LettaTurnRequest,
    ) -> AdapterFuture<'a, Result<Uuid, LettaAdapterError>> {
        Box::pin(async move {
            let candidate = LettaTurnIntent::from(request);
            let mut intents = self.intents.lock().await;
            match intents.get(&request.event_id) {
                Some(existing)
                    if existing.provider_agent_id != candidate.provider_agent_id
                        || existing.epoch_id != candidate.epoch_id
                        || existing.input_digest != candidate.input_digest =>
                {
                    Err(LettaAdapterError::Journal(
                        "event_id_content_conflict".into(),
                    ))
                }
                Some(existing) => Ok(existing.request_id),
                None => {
                    intents.insert(request.event_id.clone(), candidate);
                    Ok(request.request_id)
                }
            }
        })
    }

    fn find<'a>(
        &'a self,
        event_id: &'a str,
    ) -> AdapterFuture<'a, Result<Option<LettaTurnReceipt>, LettaAdapterError>> {
        Box::pin(async move { Ok(self.receipts.lock().await.get(event_id).cloned()) })
    }

    fn append<'a>(
        &'a self,
        receipt: &'a LettaTurnReceipt,
    ) -> AdapterFuture<'a, Result<(), LettaAdapterError>> {
        Box::pin(async move {
            let mut receipts = self.receipts.lock().await;
            receipts
                .entry(receipt.event_id.clone())
                .or_insert_with(|| receipt.clone());
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_stateful_cognitive_runtime::{CognitiveLoopOwner, RuntimeEpochIdentity};
    use std::{
        collections::BTreeSet,
        sync::atomic::{AtomicUsize, Ordering},
    };

    struct FixtureTransport {
        calls: AtomicUsize,
    }

    impl LettaTransport for FixtureTransport {
        fn send_turn<'a>(
            &'a self,
            request: &'a LettaTurnRequest,
        ) -> AdapterFuture<'a, Result<LettaTurnResponse, LettaAdapterError>> {
            Box::pin(async move {
                let call = self.calls.fetch_add(1, Ordering::SeqCst);
                if call == 0 {
                    Ok(LettaTurnResponse::ClientToolRequested {
                        request: ClientToolRequest {
                            request_id: Uuid::now_v7(),
                            epoch_id: request.epoch_id,
                            tool_name: "focusa_browser_read".into(),
                            operation: "read".into(),
                            requested_capabilities: BTreeSet::new(),
                            payload_digest: "sha256:tool".into(),
                        },
                    })
                } else {
                    assert!(request.input.is_empty());
                    assert!(request.continuation.is_some());
                    Ok(LettaTurnResponse::Completed {
                        response_digest: "sha256:response".into(),
                        evidence_refs: vec!["evidence:fixture".into()],
                    })
                }
            })
        }
    }

    struct FixtureGateway;

    impl PiClientToolGateway for FixtureGateway {
        fn execute<'a>(
            &'a self,
            _binding: &'a RuntimeBinding,
            request: &'a ClientToolRequest,
        ) -> AdapterFuture<'a, Result<ClientToolResult, LettaAdapterError>> {
            Box::pin(async move {
                Ok(ClientToolResult {
                    request_id: request.request_id,
                    status: agent_stateful_cognitive_runtime::ToolResultStatus::Completed,
                    evidence_refs: vec!["evidence:tool".into()],
                    receipt_ref: Some("receipt:tool".into()),
                    result_digest: Some("sha256:tool-result".into()),
                    failure_class: None,
                })
            })
        }
    }

    fn binding() -> RuntimeBinding {
        RuntimeBinding {
            schema: RuntimeBinding::SCHEMA.into(),
            mode: RuntimeMode::LettaManaged,
            owner: CognitiveLoopOwner::Letta,
            epoch: RuntimeEpochIdentity {
                epoch_id: Uuid::now_v7(),
                project_root: "/project".into(),
                continuity_id: "continuity".into(),
                agent_instance_id: "agent".into(),
                native_session_id: None,
            },
            provider_agent_id: Some("letta-agent".into()),
            admitted_client_tools: BTreeSet::from(["focusa_browser_read".into()]),
        }
    }

    fn request(binding: &RuntimeBinding) -> LettaTurnRequest {
        LettaTurnRequest {
            request_id: Uuid::now_v7(),
            event_id: "event-1".into(),
            provider_agent_id: "letta-agent".into(),
            epoch_id: binding.epoch.epoch_id,
            input: "Analyze bounded evidence".into(),
            input_digest: "sha256:input".into(),
            continuation: None,
        }
    }

    #[test]
    fn capability_contract_is_strict_and_matches_implemented_boundaries() {
        let contract = canonical_letta_capability_contract();
        assert_eq!(contract.schema, "focusa.letta_capability_contract.v1");
        assert_eq!(
            contract.supported_operations,
            BTreeSet::from([
                LettaOperation::SendTurn,
                LettaOperation::ContinueClientToolThroughPi
            ])
        );
        assert_eq!(contract.authentication, "bearer_from_credential_provider");
        assert_eq!(contract.client_tool_owner, "pi_gateway_only");
        assert!(
            contract
                .identity_fields
                .contains(&"provider_agent_id".into())
        );
        assert!(contract.identity_fields.contains(&"epoch_id".into()));
        assert!(
            contract
                .forbidden_direct_capabilities
                .contains("unrestricted_browser")
        );
        let json = serde_json::to_value(contract).unwrap();
        assert!(json.get("sdk_execute_arbitrary").is_none());
        assert!(json.get("direct_uiai").is_none());
    }

    #[test]
    fn exact_scope_resume_quarantines_foreign_project_and_thread_state() {
        let runtime = binding();
        let expected = LettaScopeBinding {
            schema: "focusa.letta_scope_binding.v1".into(),
            project_root: runtime.epoch.project_root.clone(),
            continuity_id: runtime.epoch.continuity_id.clone(),
            workpoint_id: "workpoint-1".into(),
            provider_agent_id: runtime.provider_agent_id.clone().unwrap(),
            provider_thread_id: "thread-1".into(),
            epoch_id: runtime.epoch.epoch_id,
            replay_key_prefix: "continuity/workpoint-1".into(),
        };
        let resumed = evaluate_letta_resume(&expected, expected.clone(), &runtime);
        assert_eq!(resumed.status, "resumed");
        assert_eq!(resumed.binding, Some(expected.clone()));

        for mutate in ["project", "thread", "workpoint", "continuity"] {
            let mut foreign = expected.clone();
            match mutate {
                "project" => foreign.project_root = "/foreign".into(),
                "thread" => foreign.provider_thread_id = "thread-foreign".into(),
                "workpoint" => foreign.workpoint_id = "workpoint-foreign".into(),
                "continuity" => foreign.continuity_id = "continuity-foreign".into(),
                _ => unreachable!(),
            }
            let decision = evaluate_letta_resume(&expected, foreign, &runtime);
            assert_eq!(decision.status, "quarantined");
            assert_eq!(decision.binding, None);
            assert_eq!(
                decision.failure_class.as_deref(),
                Some("foreign_scope_or_thread")
            );
            assert!(decision.quarantined_candidate_digest.is_some());
        }
    }

    #[tokio::test]
    async fn tool_requests_flow_through_gateway_and_receipt_is_idempotent() {
        let transport = Arc::new(FixtureTransport {
            calls: AtomicUsize::new(0),
        });
        let adapter = LettaAdapter::new(
            transport.clone(),
            Arc::new(FixtureGateway),
            Arc::new(InMemoryTurnJournal::default()),
        );
        let binding = binding();
        let first = adapter.run_turn(&binding, request(&binding)).await.unwrap();
        let second = adapter.run_turn(&binding, request(&binding)).await.unwrap();
        assert_eq!(first, second);
        assert_eq!(first.tool_continuations, 1);
        assert_eq!(transport.calls.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn pi_native_mode_cannot_be_claimed_by_letta() {
        let adapter = LettaAdapter::new(
            Arc::new(FixtureTransport {
                calls: AtomicUsize::new(0),
            }),
            Arc::new(FixtureGateway),
            Arc::new(InMemoryTurnJournal::default()),
        );
        let mut binding = binding();
        binding.mode = RuntimeMode::PiNative;
        binding.owner = CognitiveLoopOwner::Pi;
        binding.provider_agent_id = None;
        assert!(matches!(
            adapter.run_turn(&binding, request(&binding)).await,
            Err(LettaAdapterError::WrongRuntimeMode)
        ));
    }
}
