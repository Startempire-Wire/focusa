//! Governed Pi RPC protocol and AgentBootstrap mutation barrier.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::{HarnessRunRef, InputPayload, ObservedHarnessModel, PromptPayload};

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
        let response = self.transport.call(PiRpcRequest::Bootstrap {
            barrier: Box::new(self.barrier.clone()),
        })?;
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
