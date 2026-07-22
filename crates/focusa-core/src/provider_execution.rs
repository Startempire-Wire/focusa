//! Provider-neutral governance contracts (Spec 135 P1).
//!
//! Providers execute behind their native owners. Focusa authorizes only requests
//! that preserve exact scope, permission, idempotency, Operation Registry, and
//! Receipt requirements.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderClass {
    FocusaOperation,
    WorkItem,
    Model,
    Browser,
    AgentTransport,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderGovernanceContract {
    pub provider_id: String,
    pub provider_class: ProviderClass,
    pub implementation_owner: String,
    pub execution_owner: String,
    pub operation_prefixes: Vec<String>,
    pub exact_scope_required: bool,
    pub permission_required: bool,
    pub idempotency_required: bool,
    pub receipt_required: bool,
    pub operation_registry_required: bool,
    pub canonical_state_owner: String,
    pub direct_canonical_mutation_allowed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderExecutionScope {
    pub project_root: String,
    pub continuity_id: String,
    pub attachment_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderExecutionRequest {
    pub provider_id: String,
    pub operation_id: String,
    pub scope: ProviderExecutionScope,
    pub permission_grant_ref: String,
    pub idempotency_key: String,
    pub receipt_required: bool,
    pub payload_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderConformanceResult {
    pub schema: String,
    pub conformant: bool,
    pub provider_id: String,
    pub operation_id: String,
    pub checks: Vec<String>,
    pub violations: Vec<String>,
    pub receipt_ref: String,
}

pub fn supported_provider_contracts() -> Vec<ProviderGovernanceContract> {
    let contract = |provider_id: &str,
                    provider_class,
                    implementation_owner: &str,
                    execution_owner: &str,
                    prefixes: &[&str]| ProviderGovernanceContract {
        provider_id: provider_id.into(),
        provider_class,
        implementation_owner: implementation_owner.into(),
        execution_owner: execution_owner.into(),
        operation_prefixes: std::iter::once("focusa.provider.".into())
            .chain(prefixes.iter().map(|value| (*value).into()))
            .collect(),
        exact_scope_required: true,
        permission_required: true,
        idempotency_required: true,
        receipt_required: true,
        operation_registry_required: true,
        canonical_state_owner: "focusa_core_reducer".into(),
        direct_canonical_mutation_allowed: false,
    };
    vec![
        contract(
            "focusa.operation",
            ProviderClass::FocusaOperation,
            "focusa_core",
            "focusa_core",
            &["focusa."],
        ),
        contract(
            "work_item.bd",
            ProviderClass::WorkItem,
            "focusa_core",
            "bd_adapter",
            &["focusa.work_item."],
        ),
        contract(
            "work_item.none",
            ProviderClass::WorkItem,
            "focusa_core",
            "focusa_core",
            &["focusa.work_item."],
        ),
        contract(
            "model.openai_compatible",
            ProviderClass::Model,
            "focusa_core",
            "openai_compatible_runtime",
            &["focusa.model.", "focusa.agent."],
        ),
        contract(
            "model.anthropic",
            ProviderClass::Model,
            "focusa_core",
            "anthropic_runtime",
            &["focusa.model.", "focusa.agent."],
        ),
        contract(
            "browser.uiai_engine",
            ProviderClass::Browser,
            "uiai_engine",
            "uiai_engine",
            &["focusa.browser.", "focusa.workspace_artifact."],
        ),
        contract(
            "agent.pi",
            ProviderClass::AgentTransport,
            "pi",
            "pi",
            &["focusa.agent.", "focusa.turn."],
        ),
    ]
}

pub fn evaluate_provider_request(
    request: &ProviderExecutionRequest,
    registered_operations: &BTreeSet<String>,
) -> ProviderConformanceResult {
    let contract = supported_provider_contracts()
        .into_iter()
        .find(|candidate| candidate.provider_id == request.provider_id);
    let mut checks = Vec::new();
    let mut violations = Vec::new();
    let present = |value: &str| !value.trim().is_empty();
    if let Some(contract) = contract {
        checks.push("supported_provider".into());
        if present(&request.scope.project_root)
            && present(&request.scope.continuity_id)
            && present(&request.scope.attachment_id)
        {
            checks.push("exact_scope".into());
        } else {
            violations
                .push("exact project_root, continuity_id, and attachment_id are required".into());
        }
        if present(&request.permission_grant_ref) {
            checks.push("permission_grant".into());
        } else {
            violations.push("permission_grant_ref is required".into());
        }
        if present(&request.idempotency_key) {
            checks.push("idempotency".into());
        } else {
            violations.push("idempotency_key is required".into());
        }
        if request.receipt_required {
            checks.push("receipt_required".into());
        } else {
            violations.push("Receipt cannot be disabled".into());
        }
        if registered_operations.contains(&request.operation_id) {
            checks.push("operation_registry".into());
        } else {
            violations.push("operation_id is absent from the generated Operation Registry".into());
        }
        if contract
            .operation_prefixes
            .iter()
            .any(|prefix| request.operation_id.starts_with(prefix))
        {
            checks.push("provider_operation_binding".into());
        } else {
            violations.push("operation_id is not bound to this provider contract".into());
        }
        if present(&request.payload_ref) {
            checks.push("bounded_payload_ref".into());
        } else {
            violations
                .push("payload_ref is required; unbounded inline execution is prohibited".into());
        }
        checks.push("canonical_state_is_reducer_owned".into());
    } else {
        violations.push("unsupported provider_id".into());
    }
    let stable = format!(
        "{}:{}:{}",
        request.provider_id, request.operation_id, request.idempotency_key
    );
    ProviderConformanceResult {
        schema: "focusa.provider_conformance_result.v1".into(),
        conformant: violations.is_empty(),
        provider_id: request.provider_id.clone(),
        operation_id: request.operation_id.clone(),
        checks,
        violations,
        receipt_ref: format!(
            "receipt:provider-conformance:{:016x}",
            fnv1a(stable.as_bytes())
        ),
    }
}

fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(provider_id: &str, operation_id: &str) -> ProviderExecutionRequest {
        ProviderExecutionRequest {
            provider_id: provider_id.into(),
            operation_id: operation_id.into(),
            scope: ProviderExecutionScope {
                project_root: "/project".into(),
                continuity_id: "continuity".into(),
                attachment_id: "attachment".into(),
            },
            permission_grant_ref: "permission:grant".into(),
            idempotency_key: "idem-1".into(),
            receipt_required: true,
            payload_ref: "artifact:payload".into(),
        }
    }

    #[test]
    fn every_supported_provider_has_identical_governance_gates() {
        let contracts = supported_provider_contracts();
        assert_eq!(contracts.len(), 7);
        assert!(contracts.iter().all(|c| c.exact_scope_required
            && c.permission_required
            && c.idempotency_required
            && c.receipt_required
            && c.operation_registry_required
            && !c.direct_canonical_mutation_allowed));
    }

    #[test]
    fn provider_cannot_bypass_governance_envelope() {
        let operation = "focusa.work_item.closure.submit";
        let registered = BTreeSet::from([operation.into()]);
        let mut candidate = request("work_item.bd", operation);
        assert!(evaluate_provider_request(&candidate, &registered).conformant);
        candidate.scope.attachment_id.clear();
        candidate.permission_grant_ref.clear();
        candidate.idempotency_key.clear();
        candidate.receipt_required = false;
        candidate.payload_ref.clear();
        let result = evaluate_provider_request(&candidate, &registered);
        assert!(!result.conformant);
        assert_eq!(result.violations.len(), 5);
    }

    #[test]
    fn provider_cannot_invent_or_cross_bind_operation() {
        let result = evaluate_provider_request(
            &request("browser.uiai_engine", "focusa.work_item.closure.submit"),
            &BTreeSet::new(),
        );
        assert!(!result.conformant);
        assert_eq!(result.violations.len(), 2);
    }
}
