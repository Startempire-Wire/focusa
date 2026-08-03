use agent_stateful_cognitive_runtime::{
    ClientToolRequest, ClientToolResult, RuntimeBinding, ToolResultStatus,
};
use letta_adapter::{AdapterFuture, LettaAdapterError, PiClientToolGateway};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolManifest {
    pub tool_name: String,
    pub operation: String,
    #[serde(default)]
    pub admitted_capabilities: BTreeSet<String>,
    pub mutation: bool,
    pub max_result_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConstitutionDecision {
    pub permitted: bool,
    pub decision_ref: String,
    pub reason_code: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationOutcome {
    pub status: ToolResultStatus,
    pub result_digest: Option<String>,
    pub evidence_refs: Vec<String>,
    pub failure_class: Option<String>,
}

pub trait RuntimeConstitutionGuard: Send + Sync {
    fn authorize<'a>(
        &'a self,
        binding: &'a RuntimeBinding,
        request: &'a ClientToolRequest,
        manifest: &'a ToolManifest,
    ) -> AdapterFuture<'a, Result<ConstitutionDecision, GatewayError>>;
}

pub trait FocusaOperationExecutor: Send + Sync {
    fn execute<'a>(
        &'a self,
        binding: &'a RuntimeBinding,
        request: &'a ClientToolRequest,
        manifest: &'a ToolManifest,
    ) -> AdapterFuture<'a, Result<OperationOutcome, GatewayError>>;
}

#[derive(Debug, Error)]
pub enum GatewayError {
    #[error("runtime contract rejected request: {0}")]
    Runtime(String),
    #[error("tool manifest is missing: {0}")]
    ManifestMissing(String),
    #[error("operation does not match manifest")]
    OperationMismatch,
    #[error("requested capability is not admitted: {0}")]
    CapabilityNotAdmitted(String),
    #[error("Runtime Constitution denied operation: {0}")]
    ConstitutionDenied(String),
    #[error("operation executor failed: {0}")]
    Execution(String),
    #[error("completed mutation omitted evidence")]
    MutationEvidenceMissing,
}

pub struct Gateway<G, E> {
    manifests: BTreeMap<String, ToolManifest>,
    guard: Arc<G>,
    executor: Arc<E>,
}

impl<G, E> Gateway<G, E>
where
    G: RuntimeConstitutionGuard,
    E: FocusaOperationExecutor,
{
    pub fn new(
        manifests: impl IntoIterator<Item = ToolManifest>,
        guard: Arc<G>,
        executor: Arc<E>,
    ) -> Self {
        Self {
            manifests: manifests
                .into_iter()
                .map(|manifest| (manifest.tool_name.clone(), manifest))
                .collect(),
            guard,
            executor,
        }
    }

    pub async fn execute_governed(
        &self,
        binding: &RuntimeBinding,
        request: &ClientToolRequest,
    ) -> Result<ClientToolResult, GatewayError> {
        binding
            .authorize_tool_request(request)
            .map_err(|error| GatewayError::Runtime(error.to_string()))?;
        let manifest = self
            .manifests
            .get(&request.tool_name)
            .ok_or_else(|| GatewayError::ManifestMissing(request.tool_name.clone()))?;
        if manifest.operation != request.operation {
            return Err(GatewayError::OperationMismatch);
        }
        if let Some(capability) = request
            .requested_capabilities
            .iter()
            .find(|capability| !manifest.admitted_capabilities.contains(*capability))
        {
            return Err(GatewayError::CapabilityNotAdmitted(capability.clone()));
        }
        let decision = self.guard.authorize(binding, request, manifest).await?;
        if !decision.permitted {
            return Err(GatewayError::ConstitutionDenied(decision.reason_code));
        }
        let outcome = self.executor.execute(binding, request, manifest).await?;
        if manifest.mutation
            && outcome.status == ToolResultStatus::Completed
            && outcome.evidence_refs.is_empty()
        {
            return Err(GatewayError::MutationEvidenceMissing);
        }
        let receipt_ref = receipt_ref(request, manifest, &decision, &outcome);
        Ok(ClientToolResult {
            request_id: request.request_id,
            status: outcome.status,
            evidence_refs: outcome.evidence_refs,
            receipt_ref: Some(receipt_ref),
            result_digest: outcome.result_digest,
            failure_class: outcome.failure_class,
        })
    }
}

impl<G, E> PiClientToolGateway for Gateway<G, E>
where
    G: RuntimeConstitutionGuard + 'static,
    E: FocusaOperationExecutor + 'static,
{
    fn execute<'a>(
        &'a self,
        binding: &'a RuntimeBinding,
        request: &'a ClientToolRequest,
    ) -> AdapterFuture<'a, Result<ClientToolResult, LettaAdapterError>> {
        Box::pin(async move {
            self.execute_governed(binding, request)
                .await
                .map_err(|error| LettaAdapterError::Gateway(error.to_string()))
        })
    }
}

fn receipt_ref(
    request: &ClientToolRequest,
    manifest: &ToolManifest,
    decision: &ConstitutionDecision,
    outcome: &OperationOutcome,
) -> String {
    let bytes = serde_json::to_vec(&(
        request.request_id,
        request.epoch_id,
        &request.tool_name,
        &request.operation,
        &request.payload_digest,
        &manifest.admitted_capabilities,
        manifest.mutation,
        &decision.decision_ref,
        outcome.status,
        &outcome.result_digest,
        &outcome.evidence_refs,
    ))
    .expect("gateway receipt tuple must serialize");
    format!("sha256:{}", hex::encode(Sha256::digest(bytes)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_stateful_cognitive_runtime::{CognitiveLoopOwner, RuntimeEpochIdentity, RuntimeMode};
    use uuid::Uuid;

    struct Guard(bool);

    impl RuntimeConstitutionGuard for Guard {
        fn authorize<'a>(
            &'a self,
            _binding: &'a RuntimeBinding,
            _request: &'a ClientToolRequest,
            _manifest: &'a ToolManifest,
        ) -> AdapterFuture<'a, Result<ConstitutionDecision, GatewayError>> {
            Box::pin(async move {
                Ok(ConstitutionDecision {
                    permitted: self.0,
                    decision_ref: "constitution:1".into(),
                    reason_code: if self.0 { "permitted" } else { "denied" }.into(),
                })
            })
        }
    }

    struct Executor {
        evidence: bool,
    }

    impl FocusaOperationExecutor for Executor {
        fn execute<'a>(
            &'a self,
            _binding: &'a RuntimeBinding,
            _request: &'a ClientToolRequest,
            _manifest: &'a ToolManifest,
        ) -> AdapterFuture<'a, Result<OperationOutcome, GatewayError>> {
            Box::pin(async move {
                Ok(OperationOutcome {
                    status: ToolResultStatus::Completed,
                    result_digest: Some("sha256:result".into()),
                    evidence_refs: self
                        .evidence
                        .then(|| "evidence:1".into())
                        .into_iter()
                        .collect(),
                    failure_class: None,
                })
            })
        }
    }

    fn fixture() -> (RuntimeBinding, ClientToolRequest, ToolManifest) {
        let epoch_id = Uuid::now_v7();
        (
            RuntimeBinding {
                schema: RuntimeBinding::SCHEMA.into(),
                mode: RuntimeMode::LettaManaged,
                owner: CognitiveLoopOwner::Letta,
                epoch: RuntimeEpochIdentity {
                    epoch_id,
                    project_root: "/project".into(),
                    continuity_id: "continuity".into(),
                    agent_instance_id: "agent".into(),
                    native_session_id: None,
                },
                provider_agent_id: Some("letta-agent".into()),
                admitted_client_tools: BTreeSet::from(["focusa_mutate".into()]),
            },
            ClientToolRequest {
                request_id: Uuid::now_v7(),
                epoch_id,
                tool_name: "focusa_mutate".into(),
                operation: "mutation".into(),
                requested_capabilities: BTreeSet::from(["focusa_write".into()]),
                payload_digest: "sha256:payload".into(),
            },
            ToolManifest {
                tool_name: "focusa_mutate".into(),
                operation: "mutation".into(),
                admitted_capabilities: BTreeSet::from(["focusa_write".into()]),
                mutation: true,
                max_result_bytes: 4096,
            },
        )
    }

    #[tokio::test]
    async fn admitted_mutation_requires_constitution_and_evidence() {
        let (binding, request, manifest) = fixture();
        let gateway = Gateway::new(
            [manifest],
            Arc::new(Guard(true)),
            Arc::new(Executor { evidence: true }),
        );
        let result = gateway.execute_governed(&binding, &request).await.unwrap();
        assert_eq!(result.status, ToolResultStatus::Completed);
        assert!(result.receipt_ref.unwrap().starts_with("sha256:"));
    }

    #[tokio::test]
    async fn denied_or_evidence_free_mutations_fail_before_success_receipt() {
        let (binding, request, manifest) = fixture();
        let denied = Gateway::new(
            [manifest.clone()],
            Arc::new(Guard(false)),
            Arc::new(Executor { evidence: true }),
        );
        assert!(matches!(
            denied.execute_governed(&binding, &request).await,
            Err(GatewayError::ConstitutionDenied(_))
        ));
        let no_evidence = Gateway::new(
            [manifest],
            Arc::new(Guard(true)),
            Arc::new(Executor { evidence: false }),
        );
        assert!(matches!(
            no_evidence.execute_governed(&binding, &request).await,
            Err(GatewayError::MutationEvidenceMissing)
        ));
    }
}
