//! Spec 172 Section 12 — trusted manifests for dynamic and generated operations.
//!
//! Build-time scanning alone is insufficient for MCP tools, extensions,
//! downloaded capsules, plugins, generated UI, and private modules. Every
//! production operation MUST resolve through trusted metadata containing at
//! least:
//!
//! ```yaml
//! operation_id: stable.identifier
//! product_owner: focusa | uiai_engine | registered_future_product
//! operation_class: read | value_mutation | recovery | internal_maintenance
//! capability_family: registered_family
//! side_effect_class: none | local | remote | external
//! ```
//!
//! Dynamic operations require a trusted signed manifest. Unknown ownership,
//! unknown mutation, unknown side effect, or unregistered family MUST fail
//! closed before execution. A tool cannot self-label as recovery to bypass
//! licensing. Generated UI may render only canonical registered actions;
//! client-provided metadata cannot select products, prices, License Types,
//! grants, or commercial treatment.
//!
//! Callers supply factual canonical-registry lookup results; they NEVER supply
//! product, price, family, feature, limit, node, or commercial right.

use serde::{Deserialize, Serialize};

/// Spec 172 Section 21 stable error code for unknown/untrusted policy.
pub const ENTITLEMENT_POLICY_UNKNOWN: &str = "ENTITLEMENT_POLICY_UNKNOWN";

/// Registered product owners (Spec 172 Sections 12 and 15). Future products
/// are excluded until an operator-approved registration exists.
pub const REGISTERED_PRODUCT_OWNERS: [&str; 2] = ["focusa", "uiai_engine"];

/// Registered operation classes (Spec 172 Section 12).
pub const REGISTERED_OPERATION_CLASSES: [&str; 4] = [
    "read",
    "value_mutation",
    "recovery",
    "internal_maintenance",
];

/// Registered side-effect classes (Spec 172 Section 12).
pub const REGISTERED_SIDE_EFFECT_CLASSES: [&str; 4] = ["none", "local", "remote", "external"];

/// Client-supplied commercial policy fields a manifest must never carry.
/// A dynamic manifest may describe what it is; it may never select product,
/// price, License Type, family, feature, limit, node, or commercial right.
pub const FORBIDDEN_CLIENT_POLICY_FIELDS: [&str; 8] = [
    "product",
    "price",
    "license_type",
    "family",
    "feature",
    "limit",
    "node",
    "commercial_right",
];

/// A dynamic operation manifest presented at runtime intake by an MCP tool,
/// extension, capsule, A2UI surface, plugin, or private module.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DynamicOperationManifest {
    /// Stable canonical operation identifier.
    pub operation_id: String,
    /// Registered product owner (`focusa` | `uiai_engine` | registered future product).
    pub product_owner: String,
    /// Operation class (`read` | `value_mutation` | `recovery` | `internal_maintenance`).
    pub operation_class: String,
    /// Registered capability family.
    pub capability_family: String,
    /// Side-effect class (`none` | `local` | `remote` | `external`).
    pub side_effect_class: String,
    /// True only when the manifest carries a verifiable authority signature.
    pub signed: bool,
    /// Client-declared commercial policy fields. A non-empty list is a
    /// client-selected-policy attack and MUST quarantine the manifest.
    pub declared_policy_fields: Vec<String>,
}

impl DynamicOperationManifest {
    /// Construct a manifest with no signature and no declared policy fields.
    pub fn new(
        operation_id: impl Into<String>,
        product_owner: impl Into<String>,
        operation_class: impl Into<String>,
        capability_family: impl Into<String>,
        side_effect_class: impl Into<String>,
    ) -> Self {
        Self {
            operation_id: operation_id.into(),
            product_owner: product_owner.into(),
            operation_class: operation_class.into(),
            capability_family: capability_family.into(),
            side_effect_class: side_effect_class.into(),
            signed: false,
            declared_policy_fields: Vec::new(),
        }
    }

    /// Mark the manifest as carrying a verifiable authority signature.
    pub fn with_signature(mut self) -> Self {
        self.signed = true;
        self
    }

    /// Attach client-declared commercial policy fields (attack input).
    pub fn with_declared_policy_fields(mut self, fields: &[&str]) -> Self {
        self.declared_policy_fields = fields.iter().map(|field| (*field).to_string()).collect();
        self
    }
}

/// Fail-closed trust decision for one dynamic manifest or generated-UI action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ManifestTrustDecision {
    /// Signed manifest whose claims match the canonical registry exactly.
    Trusted,
    /// No verifiable authority signature.
    QuarantinedUnsigned,
    /// `operation_id` is not in the canonical operation registry.
    QuarantinedUnknownOperation,
    /// `product_owner` is not a registered product owner.
    QuarantinedUnknownOwner,
    /// `operation_class` is not a registered operation class.
    QuarantinedUnknownMutation,
    /// `side_effect_class` is not a registered side-effect class.
    QuarantinedUnknownSideEffect,
    /// `capability_family` is not a registered family.
    QuarantinedUnregisteredFamily,
    /// The tool self-labeled as `recovery` for an operation the canonical
    /// registry classifies otherwise (licensing bypass attempt).
    QuarantinedSelfLabeledRecovery,
    /// The manifest selects product, price, License Type, family, feature,
    /// limit, node, commercial right, or any policy field that differs from
    /// the canonical registry (client-selected policy).
    QuarantinedClientSelectedPolicy,
    /// A generated-UI binding references an action outside the canonical
    /// registered action set (grant expansion).
    QuarantinedGeneratedUiGrantExpansion,
}

impl ManifestTrustDecision {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Trusted => "trusted",
            Self::QuarantinedUnsigned => "quarantined_unsigned",
            Self::QuarantinedUnknownOperation => "quarantined_unknown_operation",
            Self::QuarantinedUnknownOwner => "quarantined_unknown_owner",
            Self::QuarantinedUnknownMutation => "quarantined_unknown_mutation",
            Self::QuarantinedUnknownSideEffect => "quarantined_unknown_side_effect",
            Self::QuarantinedUnregisteredFamily => "quarantined_unregistered_family",
            Self::QuarantinedSelfLabeledRecovery => "quarantined_self_labeled_recovery",
            Self::QuarantinedClientSelectedPolicy => "quarantined_client_selected_policy",
            Self::QuarantinedGeneratedUiGrantExpansion => "quarantined_generated_ui_grant_expansion",
        }
    }

    pub const fn is_trusted(self) -> bool {
        matches!(self, Self::Trusted)
    }

    pub const fn is_quarantined(self) -> bool {
        !matches!(self, Self::Trusted)
    }

    /// Stable Section 21 error code. Quarantined manifests never execute and
    /// never become limited/paid by client metadata; they surface the stable
    /// `ENTITLEMENT_POLICY_UNKNOWN` recovery/upgrade error.
    pub const fn stable_error(self) -> &'static str {
        match self {
            Self::Trusted => "",
            _ => ENTITLEMENT_POLICY_UNKNOWN,
        }
    }
}

/// Factual canonical-registry lookup results supplied by the runtime intake.
///
/// Callers report what the signed canonical registry contains; they NEVER
/// supply product, price, License Type, family, feature, limit, node, or
/// commercial right.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CanonicalManifestFacts {
    /// The canonical operation registry contains `operation_id`.
    pub operation_registered: bool,
    /// Canonical operation class for the operation, if registered.
    pub canonical_operation_class: Option<String>,
    /// Canonical capability family for the operation, if registered.
    pub canonical_capability_family: Option<String>,
    /// Canonical side-effect class for the operation, if registered.
    pub canonical_side_effect_class: Option<String>,
    /// `product_owner` is a registered product owner.
    pub product_owner_registered: bool,
    /// `operation_class` is a registered operation class.
    pub operation_class_registered: bool,
    /// `side_effect_class` is a registered side-effect class.
    pub side_effect_class_registered: bool,
    /// `capability_family` is a registered family.
    pub capability_family_registered: bool,
}

/// Verify one dynamic operation manifest at operation registry runtime intake.
///
/// Fail-closed gates, in order:
/// 1. Unsigned manifests quarantine.
/// 2. Unregistered `operation_id` quarantines.
/// 3. Unknown `product_owner` quarantines.
/// 4. Unknown `operation_class` (mutation class) quarantines.
/// 5. Unknown `side_effect_class` quarantines.
/// 6. Unregistered `capability_family` quarantines.
/// 7. Self-labeled `recovery` for a canonical non-recovery operation quarantines
///    (a tool cannot self-label as recovery to bypass licensing).
/// 8. Any claim that differs from the canonical registry, or any declared
///    client policy field, quarantines as client-selected policy.
///
/// Trusted operations inherit canonical policy; they cannot become
/// limited/paid by client metadata.
pub fn verify_dynamic_operation_manifest(
    manifest: &DynamicOperationManifest,
    facts: &CanonicalManifestFacts,
) -> ManifestTrustDecision {
    if !manifest.signed {
        return ManifestTrustDecision::QuarantinedUnsigned;
    }
    if !facts.operation_registered {
        return ManifestTrustDecision::QuarantinedUnknownOperation;
    }
    if !facts.product_owner_registered
        || !REGISTERED_PRODUCT_OWNERS.contains(&manifest.product_owner.as_str())
    {
        return ManifestTrustDecision::QuarantinedUnknownOwner;
    }
    if !facts.operation_class_registered
        || !REGISTERED_OPERATION_CLASSES.contains(&manifest.operation_class.as_str())
    {
        return ManifestTrustDecision::QuarantinedUnknownMutation;
    }
    if !facts.side_effect_class_registered
        || !REGISTERED_SIDE_EFFECT_CLASSES.contains(&manifest.side_effect_class.as_str())
    {
        return ManifestTrustDecision::QuarantinedUnknownSideEffect;
    }
    if !facts.capability_family_registered {
        return ManifestTrustDecision::QuarantinedUnregisteredFamily;
    }
    if manifest.operation_class == "recovery"
        && facts.canonical_operation_class.as_deref() != Some("recovery")
    {
        return ManifestTrustDecision::QuarantinedSelfLabeledRecovery;
    }
    if facts.canonical_operation_class.as_deref() != Some(manifest.operation_class.as_str())
        || facts.canonical_capability_family.as_deref() != Some(manifest.capability_family.as_str())
        || facts.canonical_side_effect_class.as_deref()
            != Some(manifest.side_effect_class.as_str())
        || !manifest.declared_policy_fields.is_empty()
    {
        return ManifestTrustDecision::QuarantinedClientSelectedPolicy;
    }
    ManifestTrustDecision::Trusted
}

/// Verify one generated-UI binding.
///
/// Generated UI may render only canonical registered actions. A binding that
/// is unsigned or references an action outside the canonical registered action
/// set fails closed as grant expansion; it can never be rendered as a
/// limited/paid surface.
pub fn verify_generated_ui_action(
    action_id: &str,
    canonical_registered_actions: &[&str],
    signed: bool,
) -> ManifestTrustDecision {
    if !signed {
        return ManifestTrustDecision::QuarantinedUnsigned;
    }
    if !canonical_registered_actions.contains(&action_id) {
        return ManifestTrustDecision::QuarantinedGeneratedUiGrantExpansion;
    }
    ManifestTrustDecision::Trusted
}

/// One quarantine record for an untrusted dynamic manifest or UI binding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuarantinedManifestRecord {
    /// Monotonic quarantine sequence (intake order).
    pub sequence: u64,
    /// The operation/action identifier that was rejected.
    pub operation_id: String,
    /// Stable fail-closed reason label.
    pub reason: String,
    /// Spec 172 Section 21 stable error code.
    pub stable_error: String,
}

/// Runtime quarantine ledger: rejected dynamic manifests and generated-UI
/// bindings are recorded here, cannot execute, and cannot become limited/paid
/// by client metadata.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestQuarantineLedger {
    records: Vec<QuarantinedManifestRecord>,
    next_sequence: u64,
}

impl ManifestQuarantineLedger {
    /// Record one quarantined operation and return its sequence.
    pub fn quarantine(&mut self, operation_id: &str, reason: &str) -> u64 {
        let sequence = self.next_sequence;
        self.next_sequence += 1;
        self.records.push(QuarantinedManifestRecord {
            sequence,
            operation_id: operation_id.to_string(),
            reason: reason.to_string(),
            stable_error: ENTITLEMENT_POLICY_UNKNOWN.to_string(),
        });
        sequence
    }

    /// True when the operation is quarantined and must not execute.
    pub fn is_quarantined(&self, operation_id: &str) -> bool {
        self.records
            .iter()
            .any(|record| record.operation_id == operation_id)
    }

    pub fn records(&self) -> &[QuarantinedManifestRecord] {
        &self.records
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec172_dynamic_operation_manifest_decision_labels_are_stable() {
        let variants = [
            ManifestTrustDecision::Trusted,
            ManifestTrustDecision::QuarantinedUnsigned,
            ManifestTrustDecision::QuarantinedUnknownOperation,
            ManifestTrustDecision::QuarantinedUnknownOwner,
            ManifestTrustDecision::QuarantinedUnknownMutation,
            ManifestTrustDecision::QuarantinedUnknownSideEffect,
            ManifestTrustDecision::QuarantinedUnregisteredFamily,
            ManifestTrustDecision::QuarantinedSelfLabeledRecovery,
            ManifestTrustDecision::QuarantinedClientSelectedPolicy,
            ManifestTrustDecision::QuarantinedGeneratedUiGrantExpansion,
        ];
        let mut seen = std::collections::HashSet::new();
        for variant in variants {
            let label = variant.label();
            assert!(!label.is_empty(), "empty label for {variant:?}");
            assert!(seen.insert(label), "duplicate label {label}");
        }
        assert!(ManifestTrustDecision::Trusted.is_trusted());
        for variant in variants[1..].iter() {
            assert!(variant.is_quarantined());
            assert_eq!(variant.stable_error(), ENTITLEMENT_POLICY_UNKNOWN);
        }
        assert_eq!(ManifestTrustDecision::Trusted.stable_error(), "");
    }

    #[test]
    fn spec172_dynamic_operation_manifest_registered_vocabularies_are_exact() {
        assert_eq!(REGISTERED_PRODUCT_OWNERS, ["focusa", "uiai_engine"]);
        assert_eq!(
            REGISTERED_OPERATION_CLASSES,
            ["read", "value_mutation", "recovery", "internal_maintenance"]
        );
        assert_eq!(
            REGISTERED_SIDE_EFFECT_CLASSES,
            ["none", "local", "remote", "external"]
        );
        assert_eq!(
            FORBIDDEN_CLIENT_POLICY_FIELDS,
            [
                "product",
                "price",
                "license_type",
                "family",
                "feature",
                "limit",
                "node",
                "commercial_right"
            ]
        );
    }

    #[test]
    fn spec172_dynamic_operation_manifest_quarantine_ledger_blocks_execution() {
        let mut ledger = ManifestQuarantineLedger::default();
        assert!(ledger.is_empty());
        let first = ledger.quarantine("focusa.unknown.tool", "quarantined_unknown_operation");
        let second = ledger.quarantine(
            "focusa.self_labeled.recovery",
            "quarantined_self_labeled_recovery",
        );
        assert_eq!(first, 0);
        assert_eq!(second, 1);
        assert_eq!(ledger.len(), 2);
        assert!(ledger.is_quarantined("focusa.unknown.tool"));
        assert!(ledger.is_quarantined("focusa.self_labeled.recovery"));
        assert!(!ledger.is_quarantined("focusa.license.validate"));
        for record in ledger.records() {
            assert_eq!(record.stable_error, ENTITLEMENT_POLICY_UNKNOWN);
            assert!(record.sequence < ledger.len() as u64);
        }
    }
}
