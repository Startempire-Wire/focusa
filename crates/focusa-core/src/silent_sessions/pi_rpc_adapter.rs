//! Governed Pi RPC protocol and AgentBootstrap mutation barrier.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::{
    HarnessRunRef, InputPayload, ObservedHarnessModel, PromptPayload, SilentSessionId,
    SilentSessionRunId,
};

use crate::license::{
    EntitlementExecutionContext, EntitlementExecutionDecision, EntitlementExecutionPolicy,
    evaluate_entitlement_execution,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentBootstrapBinding {
    pub project_root: String,
    pub project_identity_ref: String,
    pub continuity_id: String,
    pub trajectory_ref: String,
    pub workpoint_ref: String,
    pub context_packet_ref: String,
    pub context_packet_digest: String,
    pub writer_lease_ref: String,
    pub writer_lease_expires_at: DateTime<Utc>,
    pub authority_verified_at: DateTime<Utc>,
    pub authority_max_age_seconds: i64,
    pub requested_model: String,
    pub effective_model: String,
    pub observed_model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentBootstrapBarrier {
    pub binding: AgentBootstrapBinding,
    pub project_identity_verified: bool,
    pub trajectory_verified: bool,
    pub workpoint_verified: bool,
    pub context_packet_verified: bool,
    pub writer_lease_verified: bool,
    pub model_preflight_verified: bool,
}

impl AgentBootstrapBarrier {
    pub fn verify(&self, now: DateTime<Utc>) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.project_identity_verified,
            "ProjectIdentity barrier denied"
        );
        anyhow::ensure!(self.trajectory_verified, "Trajectory barrier denied");
        anyhow::ensure!(self.workpoint_verified, "Workpoint barrier denied");
        anyhow::ensure!(
            self.context_packet_verified,
            "Context packet barrier denied"
        );
        anyhow::ensure!(self.writer_lease_verified, "writer lease barrier denied");
        anyhow::ensure!(
            self.model_preflight_verified,
            "model preflight barrier denied"
        );
        anyhow::ensure!(
            self.binding.project_root.starts_with('/'),
            "project root is not absolute"
        );
        for value in [
            &self.binding.project_identity_ref,
            &self.binding.continuity_id,
            &self.binding.trajectory_ref,
            &self.binding.workpoint_ref,
            &self.binding.context_packet_ref,
            &self.binding.context_packet_digest,
            &self.binding.writer_lease_ref,
        ] {
            anyhow::ensure!(
                !value.trim().is_empty(),
                "bootstrap binding contains an empty authority reference"
            );
        }
        anyhow::ensure!(
            now < self.binding.writer_lease_expires_at,
            "writer lease expired"
        );
        anyhow::ensure!(
            now.signed_duration_since(self.binding.authority_verified_at)
                .num_seconds()
                <= self.binding.authority_max_age_seconds,
            "Context Authority is stale"
        );
        anyhow::ensure!(
            self.binding.requested_model == self.binding.effective_model,
            "requested and effective models differ before launch"
        );
        if let Some(observed) = &self.binding.observed_model {
            anyhow::ensure!(
                observed == &self.binding.effective_model,
                "observed model differs from effective model"
            );
        }
        Ok(())
    }

    pub fn authorize_project_mutation(&self, now: DateTime<Utc>) -> anyhow::Result<()> {
        self.verify(now)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "method", content = "params", rename_all = "snake_case")]
pub enum PiRpcRequest {
    Bootstrap {
        barrier: Box<AgentBootstrapBarrier>,
    },
    Prompt {
        run: HarnessRunRef,
        prompt: PromptPayload,
    },
    Input {
        run: HarnessRunRef,
        input: InputPayload,
    },
    Steer {
        run: HarnessRunRef,
        instruction: String,
    },
    Followup {
        run: HarnessRunRef,
        prompt: PromptPayload,
    },
    Abort {
        run: HarnessRunRef,
    },
    QueryState {
        run: HarnessRunRef,
    },
    QueryModel {
        run: HarnessRunRef,
    },
    QueryUsage {
        run: HarnessRunRef,
    },
    ResumeNativeSession {
        native_session_ref: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "event", content = "data", rename_all = "snake_case")]
pub enum PiRpcEvent {
    Turn(Value),
    Message(Value),
    ToolCall(Value),
    ToolResult(Value),
    Model(Value),
    Usage(Value),
    State(Value),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PiRpcResponse {
    pub protocol: String,
    pub ok: bool,
    pub result: Value,
    #[serde(default)]
    pub events: Vec<PiRpcEvent>,
    pub native_session_ref: Option<String>,
}

fn entitlement_policy_for_request(request: &PiRpcRequest) -> EntitlementExecutionPolicy {
    let operation_id = match request {
        PiRpcRequest::Prompt { .. } => "focusa.agent_runtime.prompt",
        PiRpcRequest::Input { .. } => "focusa.agent_runtime.input",
        PiRpcRequest::Steer { .. } => "focusa.agent_runtime.steer",
        PiRpcRequest::Followup { .. } => "focusa.agent_runtime.followup",
        PiRpcRequest::Abort { .. } => "focusa.agent_runtime.abort",
        PiRpcRequest::ResumeNativeSession { .. } => "focusa.agent_runtime.resume_native_session",
        PiRpcRequest::QueryState { .. } => "focusa.agent_runtime.query_state",
        PiRpcRequest::QueryModel { .. } => "focusa.agent_runtime.query_model",
        PiRpcRequest::QueryUsage { .. } => "focusa.agent_runtime.query_usage",
        PiRpcRequest::Bootstrap { .. } => "focusa.agent_runtime.bootstrap",
    };

    match request {
        PiRpcRequest::QueryState { .. }
        | PiRpcRequest::QueryModel { .. }
        | PiRpcRequest::QueryUsage { .. } => EntitlementExecutionPolicy::new(
            operation_id,
            focusa_license::OperationClass::Read,
            focusa_license::CapabilityFamily::ReadProjection,
            None,
            None,
            focusa_license::RecoveryAllowance::None,
        ),
        _ => EntitlementExecutionPolicy::new(
            operation_id,
            focusa_license::OperationClass::ValueMutation,
            focusa_license::CapabilityFamily::BaseFocusa,
            None,
            None,
            focusa_license::RecoveryAllowance::None,
        ),
    }
}

fn evaluate_pi_rpc_request_entitlement(
    guard: &focusa_license::LicenseGuard,
    request: &PiRpcRequest,
    now: DateTime<Utc>,
) -> anyhow::Result<EntitlementExecutionDecision> {
    let policy = entitlement_policy_for_request(request);
    evaluate_entitlement_execution(
        guard,
        &policy,
        EntitlementExecutionContext {
            now,
            ..Default::default()
        },
    )
    .map_err(|error| anyhow::anyhow!("{}: {}", error.code, error.message))
}

pub trait PiRpcTransport {
    fn call(&mut self, request: PiRpcRequest) -> anyhow::Result<PiRpcResponse>;
}

pub struct GovernedPiRpcAdapter<T> {
    pub transport: T,
    pub barrier: AgentBootstrapBarrier,
    pub bootstrapped: bool,
}

impl<T: PiRpcTransport> GovernedPiRpcAdapter<T> {
    pub fn bootstrap(&mut self, now: DateTime<Utc>) -> anyhow::Result<PiRpcResponse> {
        self.barrier.verify(now)?;
        let request = PiRpcRequest::Bootstrap {
            barrier: Box::new(self.barrier.clone()),
        };
        let guard = focusa_license::resolve_license_guard();
        evaluate_pi_rpc_request_entitlement(&guard, &request, now)?;
        let response = self.transport.call(request)?;
        anyhow::ensure!(response.ok, "Pi rejected AgentBootstrap packet");
        self.bootstrapped = true;
        Ok(response)
    }

    pub fn mutate(
        &mut self,
        now: DateTime<Utc>,
        request: PiRpcRequest,
    ) -> anyhow::Result<PiRpcResponse> {
        anyhow::ensure!(
            self.bootstrapped,
            "project mutation blocked before AgentBootstrap"
        );
        self.barrier.authorize_project_mutation(now)?;
        anyhow::ensure!(
            matches!(
                request,
                PiRpcRequest::Prompt { .. }
                    | PiRpcRequest::Input { .. }
                    | PiRpcRequest::Steer { .. }
                    | PiRpcRequest::Followup { .. }
                    | PiRpcRequest::Abort { .. }
                    | PiRpcRequest::ResumeNativeSession { .. }
            ),
            "non-mutation RPC must use query"
        );
        let guard = focusa_license::resolve_license_guard();
        evaluate_pi_rpc_request_entitlement(&guard, &request, now)?;
        let response = self.transport.call(request)?;
        anyhow::ensure!(response.ok, "Pi mutation RPC failed");
        Ok(response)
    }

    pub fn query(&mut self, request: PiRpcRequest) -> anyhow::Result<PiRpcResponse> {
        anyhow::ensure!(
            matches!(
                request,
                PiRpcRequest::QueryState { .. }
                    | PiRpcRequest::QueryModel { .. }
                    | PiRpcRequest::QueryUsage { .. }
            ),
            "mutation RPC must pass the AgentBootstrap barrier"
        );
        let now = Utc::now();
        let guard = focusa_license::resolve_license_guard();
        evaluate_pi_rpc_request_entitlement(&guard, &request, now)?;
        self.transport.call(request)
    }
}

#[derive(Debug, Default)]
pub struct DeterministicPiRpcTransport {
    pub requests: Vec<PiRpcRequest>,
    pub model: String,
    pub native_session_ref: Option<String>,
}

impl PiRpcTransport for DeterministicPiRpcTransport {
    fn call(&mut self, request: PiRpcRequest) -> anyhow::Result<PiRpcResponse> {
        let result = match &request {
            PiRpcRequest::QueryModel { .. } => json!({"provider":"test","model":self.model}),
            PiRpcRequest::QueryUsage { .. } => json!({"input_tokens":0,"output_tokens":0}),
            PiRpcRequest::QueryState { .. } => json!({"state":"running"}),
            _ => json!({"accepted":true}),
        };
        self.requests.push(request);
        Ok(PiRpcResponse {
            protocol: "focusa.pi_rpc.v1".into(),
            ok: true,
            result,
            events: Vec::new(),
            native_session_ref: self.native_session_ref.clone(),
        })
    }
}

pub fn observed_model_from_response(
    response: &PiRpcResponse,
) -> anyhow::Result<ObservedHarnessModel> {
    Ok(ObservedHarnessModel {
        provider: response
            .result
            .get("provider")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .into(),
        model: response
            .result
            .get("model")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .into(),
        source: "pi_rpc".into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use focusa_license::authority::{EntitlementSnapshot, EntitlementState};
    use std::collections::BTreeMap;

    fn sample_barrier(now: DateTime<Utc>) -> AgentBootstrapBarrier {
        AgentBootstrapBarrier {
            binding: AgentBootstrapBinding {
                project_root: "/tmp/focusa-core-pi-rpc-tests".into(),
                project_identity_ref: "identity-ref".into(),
                continuity_id: "continuity-id".into(),
                trajectory_ref: "trajectory-ref".into(),
                workpoint_ref: "workpoint-ref".into(),
                context_packet_ref: "context-ref".into(),
                context_packet_digest: "context-digest".into(),
                writer_lease_ref: "lease-ref".into(),
                writer_lease_expires_at: now + chrono::Duration::minutes(10),
                authority_verified_at: now - chrono::Duration::minutes(1),
                authority_max_age_seconds: 3600,
                requested_model: "test-model".into(),
                effective_model: "test-model".into(),
                observed_model: None,
            },
            project_identity_verified: true,
            trajectory_verified: true,
            workpoint_verified: true,
            context_packet_verified: true,
            writer_lease_verified: true,
            model_preflight_verified: true,
        }
    }

    fn sample_run_ref() -> HarnessRunRef {
        HarnessRunRef {
            session_id: SilentSessionId::new(),
            run_id: SilentSessionRunId::new(),
        }
    }

    fn sample_focusa_license_guard(now: DateTime<Utc>) -> focusa_license::LicenseGuard {
        focusa_license::LicenseGuard::from_entitlement(EntitlementSnapshot {
            state: EntitlementState::Active,
            product: "focusa".into(),
            node_id: "test-node".into(),
            subject_id: None,
            lease_id: Some("lease-id".into()),
            sequence: Some(7),
            lease_digest: Some("lease-digest".into()),
            expires_at: Some(now + chrono::Duration::hours(1)),
            offline_grace_until: None,
            features: BTreeMap::new(),
            limits: BTreeMap::new(),
            recovery_reason: None,
        })
    }

    #[test]
    fn pi_rpc_adapter_rejects_mutation_when_base_entitlement_is_missing() {
        let now = Utc::now();
        let request = PiRpcRequest::Prompt {
            run: sample_run_ref(),
            prompt: PromptPayload {
                artifact_ref: "artifact".into(),
                sha256: "sha256".into(),
            },
        };
        let guard = focusa_license::LicenseGuard::eval(7);
        let error = evaluate_pi_rpc_request_entitlement(&guard, &request, now).unwrap_err();
        assert!(error.to_string().contains("ENTITLEMENT_BASE_REQUIRED"));
    }

    #[test]
    fn pi_rpc_adapter_allows_mutation_with_signed_base_entitlement_projection() {
        let now = Utc::now();
        let request = PiRpcRequest::Followup {
            run: sample_run_ref(),
            prompt: PromptPayload {
                artifact_ref: "artifact".into(),
                sha256: "sha256".into(),
            },
        };
        let guard = sample_focusa_license_guard(now);
        let decision = evaluate_pi_rpc_request_entitlement(&guard, &request, now).unwrap();
        assert_eq!(decision.code, "ENTITLEMENT_ALLOWED");
    }

    #[test]
    fn pi_rpc_adapter_uses_read_projection_for_query_requests() {
        let now = Utc::now();
        let request = PiRpcRequest::QueryUsage {
            run: sample_run_ref(),
        };
        let guard = sample_focusa_license_guard(now);
        assert!(
            evaluate_pi_rpc_request_entitlement(&guard, &request, now).is_ok(),
            "read-path should use a read policy when using query operations"
        );
        let _adapter = GovernedPiRpcAdapter {
            transport: DeterministicPiRpcTransport::default(),
            barrier: sample_barrier(now),
            bootstrapped: true,
        };
    }
}
