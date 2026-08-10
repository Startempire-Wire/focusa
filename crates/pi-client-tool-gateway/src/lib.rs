use agent_stateful_cognitive_runtime::{
    ClientToolRequest, ClientToolResult, RuntimeBinding, ToolResultStatus,
};
use chrono::{Duration, Utc};
use focusa_core::license::{
    evaluate_entitlement_execution,
    EntitlementExecutionContext,
    EntitlementExecutionPolicy,
};
use focusa_license::LicenseGuard;
use letta_adapter::{AdapterFuture, LettaAdapterError, PiClientToolGateway};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
use thiserror::Error;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolEntitlementPolicy {
    #[serde(default)]
    pub operation_class: Option<String>,
    #[serde(default)]
    pub capability_family: Option<String>,
    #[serde(default)]
    pub required_feature: Option<String>,
    #[serde(default)]
    pub limit_bucket: Option<String>,
}


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolManifest {
    pub tool_name: String,
    pub operation: String,
    #[serde(default)]
    pub admitted_capabilities: BTreeSet<String>,
    pub mutation: bool,
    pub max_result_bytes: usize,
    #[serde(default)]
    pub entitlement_policy: Option<ToolEntitlementPolicy>,
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

fn parse_operation_class(value: &str) -> Option<focusa_license::OperationClass> {
    match value {
        "read" => Some(focusa_license::OperationClass::Read),
        "value_mutation" | "value-mutation" | "mutation" => {
            Some(focusa_license::OperationClass::ValueMutation)
        }
        "recovery" => Some(focusa_license::OperationClass::Recovery),
        "internal_maintenance" | "internal-maintenance" => {
            Some(focusa_license::OperationClass::InternalMaintenance)
        }
        _ => None,
    }
}

fn parse_capability_family(value: &str) -> Option<focusa_license::CapabilityFamily> {
    match value {
        "read_projection" | "read-projection" => {
            Some(focusa_license::CapabilityFamily::ReadProjection)
        }
        "base_focusa" | "base-focusa" => Some(focusa_license::CapabilityFamily::BaseFocusa),
        "automation" => Some(focusa_license::CapabilityFamily::Automation),
        "team_remote" | "team-remote" => Some(focusa_license::CapabilityFamily::TeamRemote),
        "account_recovery" | "account-recovery" => {
            Some(focusa_license::CapabilityFamily::AccountRecovery)
        }
        _ => None,
    }
}

fn entitlement_policy_for_manifest(
    manifest: &ToolManifest,
) -> Result<EntitlementExecutionPolicy, GatewayError> {
    let lowered = manifest.operation.to_lowercase();
    let operation_id = manifest.operation.clone();
    let is_read_fallback = !manifest.mutation;
    let is_automation = lowered.contains("silent") || lowered.contains("parallel") || lowered.contains("agent_loop");

    let explicit = manifest.entitlement_policy.as_ref();
    let operation_class = explicit
        .and_then(|policy| {
            policy
                .operation_class
                .as_deref()
                .and_then(parse_operation_class)
        })
        .unwrap_or(if is_read_fallback {
            focusa_license::OperationClass::Read
        } else {
            focusa_license::OperationClass::ValueMutation
        });
    let capability_family = explicit
        .and_then(|policy| {
            policy
                .capability_family
                .as_deref()
                .and_then(parse_capability_family)
        })
        .unwrap_or(if is_automation {
            focusa_license::CapabilityFamily::Automation
        } else if is_read_fallback {
            focusa_license::CapabilityFamily::ReadProjection
        } else {
            focusa_license::CapabilityFamily::BaseFocusa
        });
    let recovery_allowance = if operation_class == focusa_license::OperationClass::Read {
        focusa_license::RecoveryAllowance::ReadProjection
    } else {
        focusa_license::RecoveryAllowance::None
    };

    Ok(EntitlementExecutionPolicy::new(
        &operation_id,
        operation_class,
        capability_family,
        explicit.and_then(|policy| policy.required_feature.as_deref()),
        explicit.and_then(|policy| policy.limit_bucket.as_deref()),
        recovery_allowance,
    ))
}

fn evaluate_tool_entitlement(
    manifest: &ToolManifest,
    license_guard: &LicenseGuard,
) -> Result<(), GatewayError> {
    evaluate_entitlement_execution(
        license_guard,
        &entitlement_policy_for_manifest(manifest)?,
        EntitlementExecutionContext::default(),
    )
    .map(|_| ())
    .map_err(|failure| {
        GatewayError::ConstitutionDenied(format!("{}: {}", failure.code, failure.message))
    })
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
    entitlement_guard: Option<LicenseGuard>,
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
            entitlement_guard: None,
        }
    }

    pub fn with_entitlement_guard(
        mut self,
        entitlement_guard: LicenseGuard,
    ) -> Self {
        self.entitlement_guard = Some(entitlement_guard);
        self
    }

    pub fn with_resolved_entitlement_guard(self) -> Self {
        self.with_entitlement_guard(focusa_license::resolve_license_guard())
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
        let entitlement_guard = self
            .entitlement_guard
            .clone()
            .unwrap_or_else(focusa_license::resolve_license_guard);
        evaluate_tool_entitlement(manifest, &entitlement_guard)?;
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
    use focusa_license::authority::{EntitlementSnapshot, EntitlementState};
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
                entitlement_policy: Some(ToolEntitlementPolicy {
                    operation_class: Some("value_mutation".into()),
                    capability_family: Some("base_focusa".into()),
                    required_feature: None,
                    limit_bucket: None,
                }),
            },
        )
    }

    fn signed_base_snapshot() -> LicenseGuard {
        let now = Utc::now();
        let mut snapshot = EntitlementSnapshot::unactivated("focusa", "tool-gateway");
        snapshot.state = EntitlementState::Active;
        snapshot.sequence = Some(7);
        snapshot.lease_id = Some("lease-tool-gateway".into());
        snapshot.lease_digest = Some("sha256:tool-gateway".into());
        snapshot.expires_at = Some(now + Duration::hours(1));
        snapshot.offline_grace_until = Some(now + Duration::hours(1));
        LicenseGuard::from_entitlement(snapshot)
    }

    #[tokio::test]
    async fn admitted_mutation_requires_constitution_and_evidence() {
        let (binding, request, manifest) = fixture();
        let gateway = Gateway::new(
            [manifest],
            Arc::new(Guard(true)),
            Arc::new(Executor { evidence: true }),
        )
        .with_entitlement_guard(signed_base_snapshot());
        let result = gateway.execute_governed(&binding, &request).await.unwrap();
        assert_eq!(result.status, ToolResultStatus::Completed);
        assert!(result.receipt_ref.unwrap().starts_with("sha256:"));
    }

    #[tokio::test]
    async fn denied_mutation_without_signed_base_entitlement() {
        let (binding, request, manifest) = fixture();
        let gateway = Gateway::new(
            [manifest],
            Arc::new(Guard(true)),
            Arc::new(Executor { evidence: true }),
        )
        .with_entitlement_guard(LicenseGuard::eval(7));
        assert!(matches!(
            gateway.execute_governed(&binding, &request).await,
            Err(GatewayError::ConstitutionDenied(message)) if message.contains("ENTITLEMENT_BASE_REQUIRED")
        ));
    }

    #[tokio::test]
    async fn denied_or_evidence_free_mutations_fail_before_success_receipt() {
        let (binding, request, manifest) = fixture();
        let denied = Gateway::new(
            [manifest.clone()],
            Arc::new(Guard(false)),
            Arc::new(Executor { evidence: true }),
        )
        .with_entitlement_guard(signed_base_snapshot());
        assert!(matches!(
            denied.execute_governed(&binding, &request).await,
            Err(GatewayError::ConstitutionDenied(_))
        ));
        let no_evidence = Gateway::new(
            [manifest],
            Arc::new(Guard(true)),
            Arc::new(Executor { evidence: false }),
        )
        .with_entitlement_guard(signed_base_snapshot());
        assert!(matches!(
            no_evidence.execute_governed(&binding, &request).await,
            Err(GatewayError::MutationEvidenceMissing)
        ));
    }
}
