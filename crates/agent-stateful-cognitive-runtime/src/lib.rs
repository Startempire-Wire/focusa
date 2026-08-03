use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeMode {
    PiNative,
    LettaManaged,
    PiWithLettaMemorySidecar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CognitiveLoopOwner {
    Pi,
    Letta,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeEpochIdentity {
    pub epoch_id: Uuid,
    pub project_root: String,
    pub continuity_id: String,
    pub agent_instance_id: String,
    pub native_session_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeBinding {
    pub schema: String,
    pub mode: RuntimeMode,
    pub owner: CognitiveLoopOwner,
    pub epoch: RuntimeEpochIdentity,
    pub provider_agent_id: Option<String>,
    #[serde(default)]
    pub admitted_client_tools: BTreeSet<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryAuthority {
    ReadOnlyProjection,
    AgentWritable,
    CanonicalForbidden,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryNamespace {
    pub name: String,
    pub authority: MemoryAuthority,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientToolRequest {
    pub request_id: Uuid,
    pub epoch_id: Uuid,
    pub tool_name: String,
    pub operation: String,
    #[serde(default)]
    pub requested_capabilities: BTreeSet<String>,
    pub payload_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientToolResult {
    pub request_id: Uuid,
    pub status: ToolResultStatus,
    pub evidence_refs: Vec<String>,
    pub receipt_ref: Option<String>,
    pub result_digest: Option<String>,
    pub failure_class: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolResultStatus {
    Completed,
    Denied,
    Failed,
    UnknownOutcome,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RuntimeContractError {
    #[error("runtime binding schema is unsupported")]
    UnsupportedSchema,
    #[error("runtime epoch identity is incomplete: {0}")]
    IncompleteIdentity(&'static str),
    #[error("runtime mode and cognitive-loop owner conflict")]
    OwnerConflict,
    #[error("letta-managed mode requires a provider agent id")]
    MissingProviderAgent,
    #[error("provider agent id is forbidden in pi-native mode")]
    UnexpectedProviderAgent,
    #[error("client tool is not admitted: {0}")]
    ToolNotAdmitted(String),
    #[error("direct provider capability is forbidden: {0}")]
    ForbiddenCapability(String),
    #[error("tool request epoch does not match the runtime binding")]
    EpochMismatch,
    #[error("payload digest is missing")]
    MissingPayloadDigest,
}

impl RuntimeBinding {
    pub const SCHEMA: &'static str = "focusa.stateful_cognitive_runtime_binding.v1";

    pub fn validate(&self) -> Result<(), RuntimeContractError> {
        if self.schema != Self::SCHEMA {
            return Err(RuntimeContractError::UnsupportedSchema);
        }
        for (value, field) in [
            (&self.epoch.project_root, "project_root"),
            (&self.epoch.continuity_id, "continuity_id"),
            (&self.epoch.agent_instance_id, "agent_instance_id"),
        ] {
            if value.trim().is_empty() {
                return Err(RuntimeContractError::IncompleteIdentity(field));
            }
        }
        match (self.mode, self.owner) {
            (
                RuntimeMode::PiNative | RuntimeMode::PiWithLettaMemorySidecar,
                CognitiveLoopOwner::Pi,
            )
            | (RuntimeMode::LettaManaged, CognitiveLoopOwner::Letta) => {}
            _ => return Err(RuntimeContractError::OwnerConflict),
        }
        match self.mode {
            RuntimeMode::LettaManaged | RuntimeMode::PiWithLettaMemorySidecar
                if self
                    .provider_agent_id
                    .as_deref()
                    .is_none_or(|value| value.trim().is_empty()) =>
            {
                return Err(RuntimeContractError::MissingProviderAgent);
            }
            RuntimeMode::PiNative if self.provider_agent_id.is_some() => {
                return Err(RuntimeContractError::UnexpectedProviderAgent);
            }
            _ => {}
        }
        Ok(())
    }

    pub fn authorize_tool_request(
        &self,
        request: &ClientToolRequest,
    ) -> Result<(), RuntimeContractError> {
        self.validate()?;
        if request.epoch_id != self.epoch.epoch_id {
            return Err(RuntimeContractError::EpochMismatch);
        }
        if request.payload_digest.trim().is_empty() {
            return Err(RuntimeContractError::MissingPayloadDigest);
        }
        if !self.admitted_client_tools.contains(&request.tool_name) {
            return Err(RuntimeContractError::ToolNotAdmitted(
                request.tool_name.clone(),
            ));
        }
        const FORBIDDEN: &[&str] = &[
            "browser_cookie",
            "wallet_key",
            "broker_credential",
            "raw_session_secret",
            "unrestricted_browser",
            "unrestricted_terminal",
            "unrestricted_filesystem",
            "direct_financial_tool",
        ];
        if let Some(capability) = request
            .requested_capabilities
            .iter()
            .find(|candidate| FORBIDDEN.contains(&candidate.as_str()))
        {
            return Err(RuntimeContractError::ForbiddenCapability(
                capability.clone(),
            ));
        }
        Ok(())
    }
}

pub fn canonical_memory_namespaces() -> Vec<MemoryNamespace> {
    [
        ("identity_lineage", MemoryAuthority::ReadOnlyProjection),
        ("runtime_constitution", MemoryAuthority::ReadOnlyProjection),
        ("operating_manifest", MemoryAuthority::ReadOnlyProjection),
        ("current_workpoint", MemoryAuthority::ReadOnlyProjection),
        ("working_memory", MemoryAuthority::AgentWritable),
        ("beliefs", MemoryAuthority::AgentWritable),
        ("research_questions", MemoryAuthority::AgentWritable),
        ("lessons", MemoryAuthority::AgentWritable),
        ("predictions", MemoryAuthority::CanonicalForbidden),
        ("journals", MemoryAuthority::CanonicalForbidden),
        ("balances", MemoryAuthority::CanonicalForbidden),
        ("life_death_state", MemoryAuthority::CanonicalForbidden),
        ("rewards", MemoryAuthority::CanonicalForbidden),
        ("owner_truth", MemoryAuthority::CanonicalForbidden),
    ]
    .into_iter()
    .map(|(name, authority)| MemoryNamespace {
        name: name.into(),
        authority,
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn binding(mode: RuntimeMode, owner: CognitiveLoopOwner) -> RuntimeBinding {
        RuntimeBinding {
            schema: RuntimeBinding::SCHEMA.into(),
            mode,
            owner,
            epoch: RuntimeEpochIdentity {
                epoch_id: Uuid::now_v7(),
                project_root: "/project".into(),
                continuity_id: "continuity".into(),
                agent_instance_id: "agent-1".into(),
                native_session_id: Some("pi-session".into()),
            },
            provider_agent_id: (mode != RuntimeMode::PiNative).then(|| "letta-agent-1".into()),
            admitted_client_tools: BTreeSet::from(["focusa_browser_read".into()]),
        }
    }

    #[test]
    fn exactly_one_loop_owner_matches_mode() {
        assert!(
            binding(RuntimeMode::PiNative, CognitiveLoopOwner::Pi)
                .validate()
                .is_ok()
        );
        assert!(
            binding(RuntimeMode::LettaManaged, CognitiveLoopOwner::Letta)
                .validate()
                .is_ok()
        );
        assert_eq!(
            binding(RuntimeMode::LettaManaged, CognitiveLoopOwner::Pi).validate(),
            Err(RuntimeContractError::OwnerConflict)
        );
    }

    #[test]
    fn tool_bridge_denies_direct_secrets_and_unadmitted_tools() {
        let binding = binding(RuntimeMode::LettaManaged, CognitiveLoopOwner::Letta);
        let mut request = ClientToolRequest {
            request_id: Uuid::now_v7(),
            epoch_id: binding.epoch.epoch_id,
            tool_name: "focusa_browser_read".into(),
            operation: "read".into(),
            requested_capabilities: BTreeSet::from(["browser_cookie".into()]),
            payload_digest: "sha256:fixture".into(),
        };
        assert_eq!(
            binding.authorize_tool_request(&request),
            Err(RuntimeContractError::ForbiddenCapability(
                "browser_cookie".into()
            ))
        );
        request.requested_capabilities.clear();
        request.tool_name = "letta_builtin_shell".into();
        assert_eq!(
            binding.authorize_tool_request(&request),
            Err(RuntimeContractError::ToolNotAdmitted(
                "letta_builtin_shell".into()
            ))
        );
    }

    #[test]
    fn canonical_state_is_never_agent_writable() {
        let namespaces = canonical_memory_namespaces();
        assert_eq!(
            namespaces
                .iter()
                .find(|namespace| namespace.name == "predictions")
                .unwrap()
                .authority,
            MemoryAuthority::CanonicalForbidden
        );
        assert_eq!(
            namespaces
                .iter()
                .find(|namespace| namespace.name == "working_memory")
                .unwrap()
                .authority,
            MemoryAuthority::AgentWritable
        );
    }
}
