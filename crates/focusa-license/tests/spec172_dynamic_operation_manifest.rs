//! Spec 172 Section 12: trusted manifests for dynamic and generated operations.
//!
//! Runtime intake of MCP tools, extensions, downloaded capsules, plugins,
//! generated UI, and private modules MUST fail closed unless the operation
//! resolves through trusted signed metadata. Unknown ownership, unknown
//! mutation, unknown side effect, or unregistered family quarantines before
//! execution; a tool cannot self-label as recovery; generated UI may render
//! only canonical registered actions; client metadata cannot select product,
//! price, License Type, family, feature, limit, node, or commercial right.

use std::{collections::HashSet, fs, path::PathBuf};

use focusa_license::{
    verify_dynamic_operation_manifest, verify_generated_ui_action, CanonicalManifestFacts,
    DynamicOperationManifest, ManifestQuarantineLedger, ManifestTrustDecision,
    REGISTERED_OPERATION_CLASSES, REGISTERED_PRODUCT_OWNERS, REGISTERED_SIDE_EFFECT_CLASSES,
};

fn contract_path(name: &str) -> PathBuf {
    [
        env!("CARGO_MANIFEST_DIR"),
        "../../docs/contracts/spec135/generated-contract-v1",
        name,
    ]
    .iter()
    .collect()
}

fn registry_operations() -> Vec<serde_json::Value> {
    let payload = fs::read_to_string(contract_path("operation-registry.json"))
        .expect("operation registry file should exist");
    let registry: serde_json::Value = serde_json::from_str(&payload).expect("valid JSON");
    registry
        .get("operations")
        .and_then(serde_json::Value::as_array)
        .expect("operations list")
        .clone()
}

fn ui_bindings() -> Vec<serde_json::Value> {
    let payload = fs::read_to_string(contract_path("ui-action-bindings.fixture.json"))
        .expect("ui bindings fixture should exist");
    let bindings: serde_json::Value = serde_json::from_str(&payload).expect("valid JSON");
    bindings
        .get("bindings")
        .and_then(serde_json::Value::as_array)
        .expect("bindings list")
        .clone()
}

/// Build the signed manifest the canonical registry would accept for one
/// operation: every claim matches the stamped trusted metadata exactly.
fn signed_manifest_from_registry(operation: &serde_json::Value) -> DynamicOperationManifest {
    DynamicOperationManifest::new(
        operation["operation_id"].as_str().unwrap_or("<missing>"),
        operation["product_owner"].as_str().unwrap_or(""),
        operation["operation_class"].as_str().unwrap_or(""),
        operation["capability_family"].as_str().unwrap_or(""),
        operation["side_effect_class"].as_str().unwrap_or(""),
    )
    .with_signature()
}

/// Factual canonical-registry lookup for one operation. Callers only report
/// registry facts; they never supply product, price, family, feature, limit,
/// node, or commercial right.
fn facts_from_registry(operation: &serde_json::Value) -> CanonicalManifestFacts {
    CanonicalManifestFacts {
        operation_registered: true,
        canonical_operation_class: operation["operation_class"].as_str().map(String::from),
        canonical_capability_family: operation["capability_family"].as_str().map(String::from),
        canonical_side_effect_class: operation["side_effect_class"].as_str().map(String::from),
        product_owner_registered: REGISTERED_PRODUCT_OWNERS
            .contains(&operation["product_owner"].as_str().unwrap_or("")),
        operation_class_registered: REGISTERED_OPERATION_CLASSES
            .contains(&operation["operation_class"].as_str().unwrap_or("")),
        side_effect_class_registered: REGISTERED_SIDE_EFFECT_CLASSES
            .contains(&operation["side_effect_class"].as_str().unwrap_or("")),
        capability_family_registered: true,
    }
}

fn operation_with_class<'a>(
    operations: &'a [serde_json::Value],
    class: &str,
) -> &'a serde_json::Value {
    operations
        .iter()
        .find(|operation| operation["operation_class"].as_str() == Some(class))
        .unwrap_or_else(|| panic!("no operation with class {class}"))
}

#[test]
fn spec172_dynamic_operation_manifest_registry_metadata_uses_registered_vocabulary() {
    // Every canonical operation must resolve through trusted metadata with the
    // exact Spec 172 Section 12 vocabulary (operation_id, product_owner,
    // operation_class, capability_family, side_effect_class).
    let operations = registry_operations();
    assert_eq!(operations.len(), 108, "canonical operation count");
    let mut owners = HashSet::new();
    let mut classes = HashSet::new();
    let mut side_effects = HashSet::new();
    for operation in &operations {
        assert!(operation["operation_id"].as_str().is_some());
        let owner = operation["product_owner"].as_str().expect("product_owner");
        let class = operation["operation_class"].as_str().expect("operation_class");
        let family = operation["capability_family"].as_str().expect("capability_family");
        let side_effect = operation["side_effect_class"].as_str().expect("side_effect_class");
        assert!(!owner.is_empty() && !family.is_empty(), "no empty identity fields");
        assert!(
            REGISTERED_PRODUCT_OWNERS.contains(&owner),
            "unregistered product owner {owner}"
        );
        assert!(
            REGISTERED_OPERATION_CLASSES.contains(&class),
            "unregistered operation class {class}"
        );
        assert!(
            REGISTERED_SIDE_EFFECT_CLASSES.contains(&side_effect),
            "unregistered side effect class {side_effect}"
        );
        owners.insert(owner);
        classes.insert(class);
        side_effects.insert(side_effect);
    }
    assert_eq!(owners, HashSet::from(["focusa"]), "all canonical operations are Focusa");
    assert_eq!(
        classes,
        HashSet::from(["read", "value_mutation", "recovery"]),
        "only registered classes appear"
    );
    assert!(
        side_effects
            .iter()
            .all(|value| REGISTERED_SIDE_EFFECT_CLASSES.contains(value))
    );
}

#[test]
fn spec172_dynamic_operation_manifest_all_registry_operations_trusted_when_signed() {
    // A signed manifest whose claims match the canonical registry exactly is
    // trusted for all 108 operations: trusted operations inherit canonical
    // policy and never become limited/paid by client metadata.
    let operations = registry_operations();
    for operation in &operations {
        let operation_id = operation["operation_id"].as_str().unwrap_or("<missing>");
        let manifest = signed_manifest_from_registry(operation);
        let facts = facts_from_registry(operation);
        let decision = verify_dynamic_operation_manifest(&manifest, &facts);
        assert!(
            decision.is_trusted(),
            "{operation_id} must be trusted with exact canonical claims, got {decision:?}"
        );
    }
}

#[test]
fn spec172_dynamic_operation_manifest_unsigned_manifest_quarantined() {
    // Unknown/unsigned manifests quarantine: no anonymous product capability.
    let operations = registry_operations();
    let operation = operation_with_class(&operations, "value_mutation");
    let mut manifest = signed_manifest_from_registry(operation);
    manifest.signed = false;
    assert_eq!(
        verify_dynamic_operation_manifest(&manifest, &facts_from_registry(operation)),
        ManifestTrustDecision::QuarantinedUnsigned
    );
    // A freshly constructed manifest is unsigned by default.
    let unsigned = DynamicOperationManifest::new("x", "focusa", "read", "base_focusa", "none");
    assert_eq!(
        verify_dynamic_operation_manifest(&unsigned, &facts_from_registry(operation)),
        ManifestTrustDecision::QuarantinedUnsigned
    );
}

#[test]
fn spec172_dynamic_operation_manifest_unknown_operation_quarantined() {
    // A dynamic operation not present in the canonical registry quarantines.
    let operations = registry_operations();
    let operation = operation_with_class(&operations, "read");
    let mut facts = facts_from_registry(operation);
    facts.operation_registered = false;
    let manifest = DynamicOperationManifest::new(
        "focusa.invented.dynamic_tool",
        "focusa",
        "read",
        "read_projection",
        "none",
    )
    .with_signature();
    assert_eq!(
        verify_dynamic_operation_manifest(&manifest, &facts),
        ManifestTrustDecision::QuarantinedUnknownOperation
    );
}

#[test]
fn spec172_dynamic_operation_manifest_unknown_owner_quarantined() {
    // Unknown ownership fails closed before execution; a caller cannot invent
    // a product owner to obtain a grant.
    let operations = registry_operations();
    let operation = operation_with_class(&operations, "value_mutation");
    let mut manifest = signed_manifest_from_registry(operation);
    manifest.product_owner = "caller_owned_product".to_string();
    assert_eq!(
        verify_dynamic_operation_manifest(&manifest, &facts_from_registry(operation)),
        ManifestTrustDecision::QuarantinedUnknownOwner
    );
    // A future product without operator-approved registration is not a grant.
    let mut facts = facts_from_registry(operation);
    facts.product_owner_registered = false;
    let mut manifest = signed_manifest_from_registry(operation);
    manifest.product_owner = "navigator".to_string();
    assert_eq!(
        verify_dynamic_operation_manifest(&manifest, &facts),
        ManifestTrustDecision::QuarantinedUnknownOwner
    );
}

#[test]
fn spec172_dynamic_operation_manifest_unknown_mutation_class_quarantined() {
    // Unknown mutation class quarantines; a caller cannot invent a class that
    // silently expands a grant.
    let operations = registry_operations();
    let operation = operation_with_class(&operations, "value_mutation");
    let mut manifest = signed_manifest_from_registry(operation);
    manifest.operation_class = "self_grant".to_string();
    assert_eq!(
        verify_dynamic_operation_manifest(&manifest, &facts_from_registry(operation)),
        ManifestTrustDecision::QuarantinedUnknownMutation
    );
    let mut facts = facts_from_registry(operation);
    facts.operation_class_registered = false;
    let mut manifest = signed_manifest_from_registry(operation);
    manifest.operation_class = "value_mutation".to_string();
    assert_eq!(
        verify_dynamic_operation_manifest(&manifest, &facts),
        ManifestTrustDecision::QuarantinedUnknownMutation
    );
}

#[test]
fn spec172_dynamic_operation_manifest_unknown_side_effect_quarantined() {
    // Unknown side effect fails closed; a caller cannot claim an unmetered
    // unlimited side effect to bypass resource controls.
    let operations = registry_operations();
    let operation = operation_with_class(&operations, "value_mutation");
    let mut manifest = signed_manifest_from_registry(operation);
    manifest.side_effect_class = "unmetered_unlimited".to_string();
    assert_eq!(
        verify_dynamic_operation_manifest(&manifest, &facts_from_registry(operation)),
        ManifestTrustDecision::QuarantinedUnknownSideEffect
    );
    let mut facts = facts_from_registry(operation);
    facts.side_effect_class_registered = false;
    let mut manifest = signed_manifest_from_registry(operation);
    manifest.side_effect_class = "local".to_string();
    assert_eq!(
        verify_dynamic_operation_manifest(&manifest, &facts),
        ManifestTrustDecision::QuarantinedUnknownSideEffect
    );
}

#[test]
fn spec172_dynamic_operation_manifest_unregistered_family_quarantined() {
    // Unregistered capability family quarantines; a new family never enters
    // any allowlist implicitly.
    let operations = registry_operations();
    let operation = operation_with_class(&operations, "value_mutation");
    let mut manifest = signed_manifest_from_registry(operation);
    manifest.capability_family = "unregistered_new_customer_outcome".to_string();
    let mut facts = facts_from_registry(operation);
    facts.capability_family_registered = false;
    assert_eq!(
        verify_dynamic_operation_manifest(&manifest, &facts),
        ManifestTrustDecision::QuarantinedUnregisteredFamily
    );
}

#[test]
fn spec172_dynamic_operation_manifest_self_labeled_recovery_quarantined() {
    // A tool cannot self-label as recovery to bypass licensing: a canonical
    // value_mutation operation that claims recovery quarantines.
    let operations = registry_operations();
    let operation = operation_with_class(&operations, "value_mutation");
    let mut manifest = signed_manifest_from_registry(operation);
    manifest.operation_class = "recovery".to_string();
    assert_eq!(
        verify_dynamic_operation_manifest(&manifest, &facts_from_registry(operation)),
        ManifestTrustDecision::QuarantinedSelfLabeledRecovery
    );
    // The genuine canonical recovery operations still verify as recovery.
    let recovery = operation_with_class(&operations, "recovery");
    assert_eq!(
        verify_dynamic_operation_manifest(
            &signed_manifest_from_registry(recovery),
            &facts_from_registry(recovery)
        ),
        ManifestTrustDecision::Trusted
    );
}

#[test]
fn spec172_dynamic_operation_manifest_client_selected_policy_quarantined() {
    // Client-selected policy is forbidden: declaring a License Type, product,
    // price, family, feature, limit, node, or commercial right quarantines.
    let operations = registry_operations();
    let operation = operation_with_class(&operations, "value_mutation");
    for field in [
        "product", "price", "license_type", "family", "feature", "limit", "node",
        "commercial_right",
    ] {
        let manifest = signed_manifest_from_registry(operation)
            .with_declared_policy_fields(&[field]);
        assert_eq!(
            verify_dynamic_operation_manifest(&manifest, &facts_from_registry(operation)),
            ManifestTrustDecision::QuarantinedClientSelectedPolicy,
            "declared {field} must quarantine as client-selected policy"
        );
    }
    // Any claim differing from the canonical registry is client-selected
    // policy: a caller cannot reclassify a mutation as a read to obtain a
    // cheaper treatment.
    let mut manifest = signed_manifest_from_registry(operation);
    manifest.operation_class = "read".to_string();
    assert_eq!(
        verify_dynamic_operation_manifest(&manifest, &facts_from_registry(operation)),
        ManifestTrustDecision::QuarantinedClientSelectedPolicy
    );
    let mut manifest = signed_manifest_from_registry(operation);
    manifest.capability_family = "read_projection".to_string();
    assert_eq!(
        verify_dynamic_operation_manifest(&manifest, &facts_from_registry(operation)),
        ManifestTrustDecision::QuarantinedClientSelectedPolicy
    );
    let mut manifest = signed_manifest_from_registry(operation);
    manifest.side_effect_class = "none".to_string();
    assert_eq!(
        verify_dynamic_operation_manifest(&manifest, &facts_from_registry(operation)),
        ManifestTrustDecision::QuarantinedClientSelectedPolicy
    );
}

#[test]
fn spec172_dynamic_operation_manifest_generated_ui_only_canonical_registered_actions() {
    // Generated UI may render only canonical registered actions. Every binding
    // in the generated UI bindings fixture must be a canonical registered
    // operation that is allowed in generated UI.
    let operations = registry_operations();
    let operations_by_id: std::collections::HashMap<&str, &serde_json::Value> = operations
        .iter()
        .map(|operation| (operation["operation_id"].as_str().unwrap(), operation))
        .collect();
    let bindings = ui_bindings();
    assert_eq!(bindings.len(), 108, "generated UI binding count");
    let canonical_actions: Vec<&str> = bindings
        .iter()
        .map(|binding| binding["action_id"].as_str().unwrap())
        .collect();

    for binding in &bindings {
        let action_id = binding["action_id"].as_str().unwrap();
        let operation = operations_by_id
            .get(action_id)
            .unwrap_or_else(|| panic!("binding {action_id} is not a canonical operation"));
        assert!(
            operation["ui"]["allowed_in_generated_ui"].as_bool() == Some(true),
            "{action_id} must be allowed in generated UI"
        );
        // A signed canonical binding is trusted.
        assert_eq!(
            verify_generated_ui_action(action_id, &canonical_actions, true),
            ManifestTrustDecision::Trusted
        );
        // An unsigned binding quarantines.
        assert_eq!(
            verify_generated_ui_action(action_id, &canonical_actions, false),
            ManifestTrustDecision::QuarantinedUnsigned
        );
    }

    // A generated-UI action outside the canonical registered action set is
    // grant expansion and quarantines — it cannot become a limited/paid
    // surface from client metadata.
    assert_eq!(
        verify_generated_ui_action("focusa.invented.auto_grant", &canonical_actions, true),
        ManifestTrustDecision::QuarantinedGeneratedUiGrantExpansion
    );
    assert_eq!(
        verify_generated_ui_action(
            "focusa.paid.upgrade.button",
            &canonical_actions,
            true
        ),
        ManifestTrustDecision::QuarantinedGeneratedUiGrantExpansion
    );
}

#[test]
fn spec172_dynamic_operation_manifest_quarantine_prevents_execution() {
    // Quarantined manifests are recorded and can never execute: an unknown
    // dynamic operation cannot become limited/paid by client metadata.
    let operations = registry_operations();
    let operation = operation_with_class(&operations, "value_mutation");
    let operation_id = operation["operation_id"].as_str().unwrap();
    let mut manifest = signed_manifest_from_registry(operation);
    manifest.declared_policy_fields = vec!["license_type".to_string()];
    let decision = verify_dynamic_operation_manifest(&manifest, &facts_from_registry(operation));
    assert_eq!(decision, ManifestTrustDecision::QuarantinedClientSelectedPolicy);

    let mut ledger = ManifestQuarantineLedger::default();
    let sequence = ledger.quarantine(operation_id, decision.label());
    assert_eq!(sequence, 0);
    assert!(ledger.is_quarantined(operation_id));

    // Even a canonical signed manifest for a quarantined operation must not
    // execute: the quarantine ledger check precedes execution.
    let canonical = signed_manifest_from_registry(operation);
    assert!(verify_dynamic_operation_manifest(&canonical, &facts_from_registry(operation)).is_trusted());
    assert!(ledger.is_quarantined(operation_id));
    let record = &ledger.records()[0];
    assert_eq!(record.operation_id, operation_id);
    assert_eq!(record.reason, "quarantined_client_selected_policy");
    assert_eq!(record.stable_error, "ENTITLEMENT_POLICY_UNKNOWN");
}
