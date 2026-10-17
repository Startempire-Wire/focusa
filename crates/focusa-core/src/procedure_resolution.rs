//! Exact semantic dependency checks against the existing native operation owner.
//! This resolves contracts, never permissions, handler availability or execution.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

const REGISTRY: &str =
    include_str!("../../../docs/contracts/spec135/generated-contract-v1/operation-registry.json");
const OPENAPI: &str =
    include_str!("../../../docs/contracts/spec135/generated-contract-v1/openapi-3.0.3.json");

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoleRequirement {
    pub role_ref: String,
    pub operation_ref: String,
    pub operation_version: String,
    pub input_schema_ref: String,
    pub output_schema_ref: String,
    pub required_policy_refs: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DependencyRequest {
    pub requirements: Vec<RoleRequirement>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct RoleResolution {
    pub requirement: RoleRequirement,
    pub native_effect_class: Option<crate::callgraph::SideEffectClass>,
    pub native_confirmation_required: bool,
    pub contract_compatible: bool,
    pub reasons: Vec<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct DependencyResolution {
    pub schema: String,
    pub owner_contract_digest: String,
    pub contract_compatible: bool,
    pub roles: Vec<RoleResolution>,
    pub canonical: bool,
    pub permission_verified: bool,
    pub execution_authorized: bool,
}

pub fn resolve_dependencies(request: &DependencyRequest) -> Result<DependencyResolution, String> {
    if request.requirements.is_empty() || request.requirements.len() > 128 {
        return Err("between one and 128 explicit role requirements are required".into());
    }
    let registry: Value =
        serde_json::from_str(REGISTRY).map_err(|_| "native operation contract unreadable")?;
    let openapi: Value =
        serde_json::from_str(OPENAPI).map_err(|_| "native schema contract unreadable")?;
    let operations = registry["operations"]
        .as_array()
        .ok_or("native operation collection missing")?;
    if registry["operation_count"].as_u64() != Some(operations.len() as u64)
        || operations.is_empty()
    {
        return Err("native operation collection incomplete".into());
    }
    let mut identities = BTreeSet::new();
    for operation in operations {
        let id = operation["operation_id"]
            .as_str()
            .ok_or("native operation identity missing")?;
        if id.is_empty() || !identities.insert(id) {
            return Err("native operation identities invalid".into());
        }
    }
    // Pin the complete existing owner catalogue, including referenced schemas;
    // root-schema-only hashes miss changes behind $ref. This is integrity only.
    let mut digest = Sha256::new();
    digest.update(b"focusa.procedure_owner_contracts.digest.v1\0");
    // Exact generated artifact bytes avoid serializer-feature-dependent object
    // ordering; explicit lengths prevent concatenation ambiguity.
    for artifact in [REGISTRY.as_bytes(), OPENAPI.as_bytes()] {
        digest.update((artifact.len() as u64).to_be_bytes());
        digest.update(artifact);
    }
    let owner_contract_digest = format!("sha256:{:x}", digest.finalize());
    let mut roles = Vec::new();
    let mut names = BTreeSet::new();
    for requirement in &request.requirements {
        let fields = [
            &requirement.role_ref,
            &requirement.operation_ref,
            &requirement.operation_version,
            &requirement.input_schema_ref,
            &requirement.output_schema_ref,
        ];
        if fields.iter().any(|s| s.trim().is_empty() || s.len() > 256)
            || !names.insert(&requirement.role_ref)
        {
            return Err(
                "role identities and exact version/schema references must be nonblank and unique"
                    .into(),
            );
        }
        let mut policies = BTreeSet::new();
        if requirement.required_policy_refs.len() > 128
            || requirement
                .required_policy_refs
                .iter()
                .any(|p| p.trim().is_empty() || p.len() > 256 || !policies.insert(p))
        {
            return Err("required policy references must be nonblank and unique".into());
        }
        let mut reasons = Vec::new();
        let mut native_effect_class = None;
        let mut native_confirmation_required = false;
        if let Some(operation) = operations
            .iter()
            .find(|op| op["operation_id"] == requirement.operation_ref)
        {
            if operation["canonical"] != true
                || !operation["alias_of"].is_null()
                || !operation["deprecation"].is_null()
                || operation["policy_activation"] != "active"
            {
                reasons.push("operation_not_canonical_or_active".into());
            }
            match serde_json::from_value(operation["side_effect_class"].clone()) {
                Ok(effect) => native_effect_class = Some(effect),
                Err(_) => reasons.push("native_effect_class_not_supported".into()),
            }
            match operation["control"]["confirmation"].as_str() {
                Some("none") => {}
                Some("simple" | "consequential") => native_confirmation_required = true,
                _ => reasons.push("confirmation_policy_not_supported".into()),
            }
            if operation["operation_version"] != requirement.operation_version {
                reasons.push("operation_version_mismatch".into());
            }
            for (schema_ref, field, contract_field, reason) in [
                (
                    &requirement.input_schema_ref,
                    "request_schema_ref",
                    "input_schema_ref",
                    "input_schema_mismatch",
                ),
                (
                    &requirement.output_schema_ref,
                    "response_schema_ref",
                    "output_schema_ref",
                    "output_schema_mismatch",
                ),
            ] {
                if operation[field] != *schema_ref
                    || operation["contracts"][contract_field] != *schema_ref
                {
                    reasons.push(reason.into());
                    continue;
                }
                let component = &openapi["components"]["schemas"][schema_ref.replace('.', "_")];
                if component.is_null()
                    || component["title"] != *schema_ref
                    || (component["type"] == "object" && component["additionalProperties"] != false)
                {
                    reasons.push("schema_not_explicitly_qualified".into());
                }
            }
            let supported: BTreeSet<&str> = ["capability_refs", "permission_scopes"]
                .iter()
                .flat_map(|key| {
                    operation["control"][key]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(Value::as_str)
                })
                .collect();
            if requirement
                .required_policy_refs
                .iter()
                .any(|p| !supported.contains(p.as_str()))
            {
                reasons.push("policy_reference_not_supported".into());
            }
        } else {
            reasons.push("operation_not_registered".into());
        }
        roles.push(RoleResolution {
            requirement: requirement.clone(),
            native_effect_class,
            native_confirmation_required,
            contract_compatible: reasons.is_empty(),
            reasons,
        });
    }
    Ok(DependencyResolution {
        schema: "focusa.procedure_dependencies.v1".into(),
        owner_contract_digest,
        contract_compatible: roles.iter().all(|r| r.contract_compatible),
        roles,
        canonical: false,
        permission_verified: false,
        execution_authorized: false,
    })
}

/// Bind optional dependency-bearing definitions to the exact owner snapshot.
/// Legacy previews remain advisory; a resolved definition cannot hide another
/// operation in command text, weaken native effects or drop a required role.
pub(crate) fn validate_procedure_dependencies(
    procedure: &crate::procedure_compiler::Procedure,
) -> Result<(), String> {
    use crate::procedure_compiler::StepKind;
    let (request, expected_digest) =
        match (&procedure.dependencies, &procedure.owner_contract_digest) {
            (None, None) => return Ok(()),
            (Some(request), Some(digest)) => (request, digest),
            _ => {
                return Err(
                    "dependencies and exact owner contract digest must be supplied together".into(),
                );
            }
        };
    let resolution = resolve_dependencies(request)?;
    if !resolution.contract_compatible || resolution.owner_contract_digest != *expected_digest {
        return Err("procedure dependencies or owner contract snapshot are unresolved".into());
    }
    let mut bound = 0;
    for step in &procedure.steps {
        if matches!(step.kind, StepKind::HumanHandoff | StepKind::Approval) {
            continue;
        }
        if step.kind != StepKind::Api {
            return Err(
                "this native API resolver cannot qualify a shell/model provider binding".into(),
            );
        }
        let role = resolution
            .roles
            .iter()
            .find(|role| role.requirement.role_ref == step.step_id)
            .ok_or("every native step requires its exact resolved role")?;
        if role.requirement.operation_ref != step.command
            || role.native_effect_class != Some(step.effect_class)
        {
            return Err(
                "native operation or effect differs from the resolved role contract".into(),
            );
        }
        if role.native_confirmation_required
            && !(step.require_confirmation || step.effect_class.requires_confirmation())
        {
            return Err(
                "native operation confirmation policy cannot be weakened by a procedure".into(),
            );
        }
        bound += 1;
    }
    if bound != resolution.roles.len() {
        return Err("unattached dependency roles are not a valid compiled binding".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn requirement() -> RoleRequirement {
        RoleRequirement {
            role_ref: "integrity_preview".into(),
            operation_ref: "focusa.procedure.preview".into(),
            operation_version: "1.0.0".into(),
            input_schema_ref: "focusa.procedure.v1".into(),
            output_schema_ref: "focusa.procedure_preview.v1".into(),
            required_policy_refs: vec![],
        }
    }
    #[test]
    fn native_owner_contracts_resolve_without_minting_authority() {
        let result = resolve_dependencies(&DependencyRequest {
            requirements: vec![requirement()],
        })
        .unwrap();
        assert!(result.contract_compatible);
        assert!(result.owner_contract_digest.starts_with("sha256:"));
        assert_eq!(result.owner_contract_digest.len(), 71);
        assert_eq!(result.roles[0].requirement.operation_version, "1.0.0");
        assert!(!result.canonical && !result.permission_verified && !result.execution_authorized);
    }
    #[test]
    fn compiled_dependencies_are_pinned_complete_and_material_to_identity() {
        use crate::procedure_compiler::{
            EffectClass, PROCEDURE_SCHEMA, Procedure, Step, StepKind, compile_plan,
        };
        let request = DependencyRequest {
            requirements: vec![requirement()],
        };
        let snapshot = resolve_dependencies(&request).unwrap();
        let mut procedure = Procedure {
            schema: PROCEDURE_SCHEMA.into(),
            procedure_id: "resolved-fixture".into(),
            version: 1,
            steps: vec![Step {
                step_id: "integrity_preview".into(),
                kind: StepKind::Api,
                effect_class: EffectClass::None,
                require_confirmation: false,
                rollback_step: None,
                command: "focusa.procedure.preview".into(),
            }],
            dependencies: Some(request),
            owner_contract_digest: Some(snapshot.owner_contract_digest),
        };
        let original = compile_plan(&procedure).unwrap();
        procedure.dependencies.as_mut().unwrap().requirements[0]
            .required_policy_refs
            .push("read:*".into());
        assert_ne!(original.digest, compile_plan(&procedure).unwrap().digest);
        procedure.steps[0].effect_class = EffectClass::Financial;
        assert!(compile_plan(&procedure).is_err());
        procedure.steps[0].effect_class = EffectClass::None;
        procedure.steps[0].command = "unrelated.operation".into();
        assert!(compile_plan(&procedure).is_err());
        procedure.steps[0].command = "focusa.procedure.preview".into();
        procedure.owner_contract_digest = Some("sha256:stale".into());
        assert!(compile_plan(&procedure).is_err());
        procedure.owner_contract_digest = None;
        assert!(compile_plan(&procedure).is_err());
    }
    #[test]
    fn native_operation_confirmation_cannot_be_lowered_by_local_effect() {
        use crate::procedure_compiler::{
            EffectClass, PROCEDURE_SCHEMA, Procedure, Step, StepKind, compile_plan,
        };
        let registry: Value = serde_json::from_str(REGISTRY).unwrap();
        let operation = registry["operations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|op| op["operation_id"] == "focusa.role_profile.review")
            .unwrap();
        let role = RoleRequirement {
            role_ref: "review_fixture".into(),
            operation_ref: operation["operation_id"].as_str().unwrap().into(),
            operation_version: operation["operation_version"].as_str().unwrap().into(),
            input_schema_ref: operation["request_schema_ref"].as_str().unwrap().into(),
            output_schema_ref: operation["response_schema_ref"].as_str().unwrap().into(),
            required_policy_refs: vec![],
        };
        let request = DependencyRequest {
            requirements: vec![role],
        };
        let snapshot = resolve_dependencies(&request).unwrap();
        assert!(snapshot.contract_compatible);
        assert!(snapshot.roles[0].native_confirmation_required);
        let mut procedure = Procedure {
            schema: PROCEDURE_SCHEMA.into(),
            procedure_id: "confirmation-fixture".into(),
            version: 1,
            steps: vec![Step {
                step_id: "review_fixture".into(),
                kind: StepKind::Api,
                effect_class: EffectClass::Local,
                require_confirmation: false,
                rollback_step: None,
                command: "focusa.role_profile.review".into(),
            }],
            dependencies: Some(request),
            owner_contract_digest: Some(snapshot.owner_contract_digest),
        };
        assert!(compile_plan(&procedure).is_err());
        procedure.steps[0].require_confirmation = true;
        assert!(compile_plan(&procedure).unwrap().plan_steps[0].confirmation_required);
    }
    #[test]
    fn choice_or_preview_cannot_satisfy_drafting_schema() {
        let mut role = requirement();
        role.role_ref = "drafting".into();
        role.output_schema_ref = "operator.routine_blueprint.v1".into();
        let result = resolve_dependencies(&DependencyRequest {
            requirements: vec![role],
        })
        .unwrap();
        assert!(!result.contract_compatible);
        assert!(
            result.roles[0]
                .reasons
                .contains(&"output_schema_mismatch".into())
        );
    }
    #[test]
    fn unsupported_versions_policies_and_operations_fail_closed() {
        for kind in 0..3 {
            let mut role = requirement();
            match kind {
                0 => role.operation_version = "unsupported".into(),
                1 => role.required_policy_refs.push("unsupported-policy".into()),
                _ => role.operation_ref = "unsupported-operation".into(),
            }
            assert!(
                !resolve_dependencies(&DependencyRequest {
                    requirements: vec![role]
                })
                .unwrap()
                .contract_compatible
            );
        }
    }
    #[test]
    fn missing_and_duplicate_roles_are_not_an_empty_success() {
        assert!(
            resolve_dependencies(&DependencyRequest {
                requirements: vec![]
            })
            .is_err()
        );
        assert!(
            resolve_dependencies(&DependencyRequest {
                requirements: vec![requirement(), requirement()]
            })
            .is_err()
        );
    }
}
