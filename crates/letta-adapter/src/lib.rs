pub mod checkpoint;
pub mod http;
pub mod journal;

use agent_stateful_cognitive_runtime::{
    ClientToolRequest, ClientToolResult, RuntimeBinding, RuntimeContractError, RuntimeMode,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};
use thiserror::Error;
use tokio::sync::Mutex;
use uuid::Uuid;

pub type AdapterFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

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
