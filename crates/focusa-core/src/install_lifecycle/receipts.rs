use super::{
    AdapterEntitlementPosture, LifecycleAcceptanceReceipt, LifecycleEntitlementBinding,
    LifecycleEntitlementReceiptClass, LifecycleEntitlementState, LifecycleOperation,
    LifecycleState,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

const RECEIPT_GENESIS_HASH: &str =
    "sha256:0000000000000000000000000000000000000000000000000000000000000000";

fn lifecycle_receipt_schema_v1() -> String {
    "focusa.lifecycle_acceptance_receipt.v1".into()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifecycleReceiptV1 {
    #[serde(default = "lifecycle_receipt_schema_v1")]
    pub schema_version: String,
    pub receipt_id: String,
    pub transaction_id: String,
    pub operation: LifecycleOperation,
    pub final_state: LifecycleState,
    pub created_at: DateTime<Utc>,
    pub installer_artifact_digest: String,
    pub daemon_service_healthy: bool,
    pub entitlement_receipt_class: LifecycleEntitlementReceiptClass,
    pub lease_id: Option<String>,
    pub lease_sequence: Option<u64>,
    pub lease_payload_digest: Option<String>,
    pub product_grants_digest: Option<String>,
    pub feature_grants_digest: Option<String>,
    pub node_id: Option<String>,
    pub license_class: Option<String>,
    pub signature_verified: bool,
    pub offline_valid_until: Option<DateTime<Utc>>,
    pub entitlement_evidence_refs: Vec<String>,
    pub protected_component_set_digest: Option<String>,
    pub adapter_entitlement_postures: Vec<AdapterEntitlementPosture>,
    pub journal_head_hash: String,
    pub previous_receipt_hash: String,
    pub receipt_hash: String,
    /// Canonical Spec 152F simple-policy binding (policy digest, capability
    /// family, entitlement state, lease sequence, recovery posture, and the
    /// reconciled product-ready flag). Never records raw keys, tokens,
    /// credentials, or customer PII.
    pub policy_binding: LifecyclePolicyBinding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleReceiptAppendOutcome {
    Appended,
    IdempotentReplay,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum LifecycleReceiptError {
    #[error("receipt identity, digest, or evidence is incomplete")]
    Incomplete,
    #[error("product-ready receipt is not bound to a verified entitlement snapshot")]
    UnverifiedProductReady,
    #[error("adapter entitlement posture does not reconcile with the receipt")]
    AdapterMismatch,
    #[error("receipt hash or predecessor chain is invalid")]
    IntegrityFailure,
    #[error("receipt id was replayed with different content")]
    IdempotencyConflict,
    #[error(
        "lifecycle receipt policy binding does not reconcile with the canonical entitlement policy"
    )]
    PolicyReconciliation,
}

impl LifecycleReceiptV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn from_acceptance(
        receipt_id: impl Into<String>,
        acceptance: &LifecycleAcceptanceReceipt,
        created_at: DateTime<Utc>,
        installer_artifact_digest: impl Into<String>,
        protected_component_set_digest: Option<String>,
        adapter_entitlement_postures: Vec<AdapterEntitlementPosture>,
        previous_receipt_hash: Option<String>,
    ) -> Result<Self, LifecycleReceiptError> {
        let binding = acceptance.entitlement_binding.as_ref();
        let mut receipt = Self {
            schema_version: lifecycle_receipt_schema_v1(),
            receipt_id: receipt_id.into(),
            transaction_id: acceptance.transaction_id.clone(),
            operation: acceptance.operation,
            final_state: acceptance.final_state,
            created_at,
            installer_artifact_digest: installer_artifact_digest.into(),
            daemon_service_healthy: acceptance.daemon_service_healthy,
            entitlement_receipt_class: acceptance.entitlement_receipt_class,
            lease_id: binding.map(|value| value.lease_id.clone()),
            lease_sequence: binding.map(|value| value.lease_sequence),
            lease_payload_digest: binding.map(|value| value.lease_payload_digest.clone()),
            product_grants_digest: binding.map(|value| value.product_grants_digest.clone()),
            feature_grants_digest: binding.map(|value| value.feature_grants_digest.clone()),
            node_id: binding.map(|value| value.node_id.clone()),
            license_class: binding.map(|value| value.license_class.clone()),
            signature_verified: binding.is_some_and(|value| value.signature_verified),
            offline_valid_until: binding.map(|value| value.offline_valid_until),
            entitlement_evidence_refs: acceptance.entitlement_evidence_refs.clone(),
            protected_component_set_digest,
            adapter_entitlement_postures,
            journal_head_hash: acceptance.journal_head_hash.clone(),
            previous_receipt_hash: previous_receipt_hash
                .unwrap_or_else(|| RECEIPT_GENESIS_HASH.into()),
            receipt_hash: String::new(),
            policy_binding: LifecyclePolicyBinding::from_acceptance(
                acceptance.operation,
                acceptance.entitlement_receipt_class,
                binding,
                acceptance.final_state == LifecycleState::Accepted
                    && matches!(
                        acceptance.entitlement_receipt_class,
                        LifecycleEntitlementReceiptClass::LimitedAccessReady
                            | LifecycleEntitlementReceiptClass::PaidReady
                            | LifecycleEntitlementReceiptClass::DevelopmentReady
                    )
                    && binding.is_some_and(|value| value.signature_verified),
            ),
        };
        receipt.validate_content()?;
        receipt.receipt_hash = receipt.compute_hash();
        Ok(receipt)
    }

    pub fn product_ready(&self) -> bool {
        self.final_state == LifecycleState::Accepted
            && matches!(
                self.entitlement_receipt_class,
                LifecycleEntitlementReceiptClass::LimitedAccessReady
                    | LifecycleEntitlementReceiptClass::PaidReady
                    | LifecycleEntitlementReceiptClass::DevelopmentReady
            )
            && self.signature_verified
    }

    /// Presenter-safe lifecycle receipt posture (Spec 152E §21 surface
    /// consolidation). Lifecycle receipts expose the same frozen presenter
    /// state, next action, and allowed actions as the menubar, TUI, and
    /// daemon REST license routes for the same entitlement posture, without
    /// duplicating any business decision: the receipt class plus the
    /// verified-signature flag project onto the shared vocabulary and
    /// everything else fails closed.
    pub fn presenter_posture(&self) -> LifecycleReceiptPresenterPosture {
        if self.product_ready() {
            return self.posture(
                "activated",
                "activated",
                &["manage_nodes", "refresh_lease", "manage_account", "resume"],
            );
        }
        match self.entitlement_receipt_class {
            LifecycleEntitlementReceiptClass::RecoveryReady => self.posture(
                "recovery_only",
                "recovery",
                &[
                    "recovery",
                    "repair",
                    "export",
                    "uninstall",
                    "manage_nodes",
                    "manage_account",
                ],
            ),
            // Paid/LimitedAccess/Dev readiness claimed without a verified signature
            // fails closed to recovery_only; it never renders as activated.
            LifecycleEntitlementReceiptClass::LimitedAccessReady
            | LifecycleEntitlementReceiptClass::PaidReady
            | LifecycleEntitlementReceiptClass::DevelopmentReady => self.posture(
                "recovery_only",
                "recovery",
                &[
                    "recovery",
                    "repair",
                    "export",
                    "uninstall",
                    "manage_nodes",
                    "manage_account",
                ],
            ),
            LifecycleEntitlementReceiptClass::BlockedEntitlement => self.posture(
                "denied",
                "activate_or_manage_entitlement",
                &["activate_or_manage_entitlement", "recovery"],
            ),
        }
    }

    fn posture(
        &self,
        presenter_state: &'static str,
        next_action: &'static str,
        allowed_actions: &'static [&'static str],
    ) -> LifecycleReceiptPresenterPosture {
        LifecycleReceiptPresenterPosture {
            schema: "focusa.lifecycle_receipt_presenter_posture.v1".into(),
            receipt_id: self.receipt_id.clone(),
            receipt_class: self.entitlement_receipt_class.label().into(),
            presenter_state: presenter_state.into(),
            next_action: next_action.into(),
            terminal: true,
            product_ready: self.product_ready(),
            allowed_actions: allowed_actions
                .iter()
                .map(|action| (*action).to_string())
                .collect(),
        }
    }

    pub fn verify(&self, expected_previous_hash: &str) -> Result<(), LifecycleReceiptError> {
        self.validate_content()?;
        if self.previous_receipt_hash != expected_previous_hash
            || self.receipt_hash != self.compute_hash()
        {
            return Err(LifecycleReceiptError::IntegrityFailure);
        }
        Ok(())
    }

    fn validate_content(&self) -> Result<(), LifecycleReceiptError> {
        if self.schema_version != "focusa.lifecycle_acceptance_receipt.v1"
            || self.receipt_id.trim().is_empty()
            || self.transaction_id.trim().is_empty()
            || !valid_digest(&self.installer_artifact_digest)
            || !valid_digest(&self.journal_head_hash)
            || !valid_digest(&self.previous_receipt_hash)
        {
            return Err(LifecycleReceiptError::Incomplete);
        }
        let claims_product_ready = matches!(
            self.entitlement_receipt_class,
            LifecycleEntitlementReceiptClass::LimitedAccessReady
                | LifecycleEntitlementReceiptClass::PaidReady
                | LifecycleEntitlementReceiptClass::DevelopmentReady
        );
        if claims_product_ready
            && (!self.signature_verified
                || self.lease_id.as_deref().is_none_or(str::is_empty)
                || self.lease_sequence.is_none_or(|sequence| sequence == 0)
                || !self
                    .lease_payload_digest
                    .as_deref()
                    .is_some_and(valid_digest)
                || !self
                    .product_grants_digest
                    .as_deref()
                    .is_some_and(valid_digest)
                || !self
                    .feature_grants_digest
                    .as_deref()
                    .is_some_and(valid_digest)
                || self.node_id.as_deref().is_none_or(str::is_empty)
                || self.license_class.as_deref().is_none_or(str::is_empty)
                || self.offline_valid_until.is_none()
                || self.entitlement_evidence_refs.is_empty())
        {
            return Err(LifecycleReceiptError::UnverifiedProductReady);
        }
        if self
            .protected_component_set_digest
            .as_deref()
            .is_some_and(|digest| !valid_digest(digest))
        {
            return Err(LifecycleReceiptError::Incomplete);
        }
        for posture in &self.adapter_entitlement_postures {
            posture
                .validate()
                .map_err(|_| LifecycleReceiptError::AdapterMismatch)?;
            if self
                .lease_payload_digest
                .as_deref()
                .is_none_or(|digest| posture.parent_lease_digest != digest)
            {
                return Err(LifecycleReceiptError::AdapterMismatch);
            }
        }
        // Spec 152F simple-policy binding: the receipt must record the exact
        // canonical policy digest, capability family, entitlement state,
        // lease sequence, and recovery posture. The binding is derived from
        // the canonical policy and this receipt's own authority fields, never
        // from caller input, and never contains raw key material.
        if self.policy_binding.schema_version != "focusa.lifecycle_policy_binding.v1"
            || !valid_digest(&self.policy_binding.policy_digest)
            || !LIFECYCLE_CAPABILITY_FAMILIES
                .contains(&self.policy_binding.capability_family.as_str())
            || !LIFECYCLE_ENTITLEMENT_STATES
                .contains(&self.policy_binding.entitlement_state.as_str())
            || self.policy_binding.recovery_posture == self.policy_binding.product_ready
            || self.policy_binding.product_ready != self.product_ready()
            || self.policy_binding.lease_sequence != self.lease_sequence.unwrap_or_default()
        {
            return Err(LifecycleReceiptError::PolicyReconciliation);
        }
        Ok(())
    }

    /// Reconcile the recorded simple-policy binding with the canonical
    /// embedded entitlement policy (Spec 152F §1, P2, P5, P9): the recorded
    /// policy digest must equal the registry digest, and the capability
    /// family / entitlement state / lease sequence / recovery posture must
    /// match what the canonical policy recomputes from this receipt's own
    /// authority fields. Receipts never carry caller-supplied policy identity
    /// and never record raw keys.
    pub fn reconcile_policy(&self) -> Result<(), LifecycleReceiptError> {
        self.validate_content()?;
        let registry = focusa_license::embedded_entitlement_policy_registry()
            .map_err(|_| LifecycleReceiptError::PolicyReconciliation)?;
        if self.policy_binding.policy_digest != registry.digest() {
            return Err(LifecycleReceiptError::PolicyReconciliation);
        }
        let expected_family = lifecycle_policy_family(
            self.operation,
            self.entitlement_receipt_class,
            self.product_ready(),
        );
        if self.policy_binding.capability_family != expected_family {
            return Err(LifecycleReceiptError::PolicyReconciliation);
        }
        if !lifecycle_receipt_class_states(self.entitlement_receipt_class)
            .contains(&self.policy_binding.entitlement_state.as_str())
        {
            return Err(LifecycleReceiptError::PolicyReconciliation);
        }
        Ok(())
    }

    fn compute_hash(&self) -> String {
        let mut unsigned = self.clone();
        unsigned.receipt_hash.clear();
        let bytes = serde_json::to_vec(&unsigned).expect("receipt is serializable");
        format!("sha256:{:x}", Sha256::digest(bytes))
    }
}

/// Presenter-safe lifecycle receipt posture. Renders the same shared
/// presenter state/actions as every other Spec 152E §21 surface for the same
/// entitlement posture; never contains raw email, license key, credential,
/// or card data by construction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifecycleReceiptPresenterPosture {
    pub schema: String,
    pub receipt_id: String,
    pub receipt_class: String,
    pub presenter_state: String,
    pub next_action: String,
    pub terminal: bool,
    pub product_ready: bool,
    pub allowed_actions: Vec<String>,
}

impl LifecycleEntitlementReceiptClass {
    pub const fn label(self) -> &'static str {
        match self {
            Self::RecoveryReady => "recovery_ready",
            Self::LimitedAccessReady => "limited_access_ready",
            Self::PaidReady => "paid_ready",
            Self::DevelopmentReady => "development_ready",
            Self::BlockedEntitlement => "blocked_entitlement",
        }
    }
}

pub fn append_lifecycle_receipt(
    receipts: &mut Vec<LifecycleReceiptV1>,
    candidate: LifecycleReceiptV1,
) -> Result<LifecycleReceiptAppendOutcome, LifecycleReceiptError> {
    if let Some(existing) = receipts
        .iter()
        .find(|receipt| receipt.receipt_id == candidate.receipt_id)
    {
        return if existing.receipt_hash == candidate.receipt_hash {
            Ok(LifecycleReceiptAppendOutcome::IdempotentReplay)
        } else {
            Err(LifecycleReceiptError::IdempotencyConflict)
        };
    }
    let expected_previous = receipts
        .last()
        .map(|receipt| receipt.receipt_hash.as_str())
        .unwrap_or(RECEIPT_GENESIS_HASH);
    candidate.verify(expected_previous)?;
    receipts.push(candidate);
    Ok(LifecycleReceiptAppendOutcome::Appended)
}

pub fn verify_lifecycle_receipt_chain(
    receipts: &[LifecycleReceiptV1],
) -> Result<(), LifecycleReceiptError> {
    let mut previous = RECEIPT_GENESIS_HASH;
    for receipt in receipts {
        receipt.verify(previous)?;
        previous = &receipt.receipt_hash;
    }
    Ok(())
}

fn valid_digest(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..].bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn lifecycle_policy_binding_schema_v1() -> String {
    "focusa.lifecycle_policy_binding.v1".into()
}

/// Canonical Spec 152F capability-family labels a lifecycle receipt may
/// record. The family is derived from the canonical policy (base product for
/// value-producing lifecycle mutations, recovery/read allowances for
/// customer-control and inspection flows, `premium_updates` for update
/// operations); receipts never invent a family.
const LIFECYCLE_CAPABILITY_FAMILIES: [&str; 9] = [
    "account_recovery",
    "read_projection",
    "base_focusa",
    "automation",
    "team_remote",
    "release_proof",
    "premium_updates",
    "customer_data_export",
    "internal_maintenance",
];

/// Lifecycle entitlement-state labels a receipt may record (the serialized
/// `LifecycleEntitlementState` vocabulary plus `none` when no binding exists).
const LIFECYCLE_ENTITLEMENT_STATES: [&str; 10] = [
    "unactivated",
    "pending_identity",
    "pending_device_code",
    "active_verified_limited",
    "active_paid",
    "offline_grace",
    "expired",
    "revoked",
    "invalid",
    "none",
];

/// Simple-policy binding recorded on every lifecycle acceptance receipt
/// (Spec 152F §1, P2, P5, P9): the digest of the single embedded entitlement
/// policy, the canonical capability family, the entitlement state, the lease
/// sequence, the recovery posture, and the reconciled product-ready flag.
/// The binding is derived only from the canonical registry and the receipt's
/// own authority fields; receipts never record raw keys, tokens, credentials,
/// or customer PII.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifecyclePolicyBinding {
    #[serde(default = "lifecycle_policy_binding_schema_v1")]
    pub schema_version: String,
    pub policy_digest: String,
    pub capability_family: String,
    pub entitlement_state: String,
    pub lease_sequence: u64,
    pub recovery_posture: bool,
    pub product_ready: bool,
}

impl LifecyclePolicyBinding {
    /// Derive the binding from the canonical embedded policy and the
    /// acceptance's own authority fields. No caller-controlled product,
    /// price, grant, family, or policy identity is accepted.
    pub fn from_acceptance(
        operation: LifecycleOperation,
        class: LifecycleEntitlementReceiptClass,
        binding: Option<&LifecycleEntitlementBinding>,
        product_ready: bool,
    ) -> Self {
        let (entitlement_state, lease_sequence) = binding
            .map(|value| {
                (
                    lifecycle_entitlement_state_label(value.state),
                    value.lease_sequence,
                )
            })
            .unwrap_or_else(|| ("none".to_string(), 0));
        Self {
            schema_version: lifecycle_policy_binding_schema_v1(),
            policy_digest: focusa_license::embedded_entitlement_policy_registry()
                .expect("embedded entitlement policy registry")
                .digest()
                .to_string(),
            capability_family: lifecycle_policy_family(operation, class, product_ready),
            entitlement_state,
            lease_sequence,
            recovery_posture: !product_ready,
            product_ready,
        }
    }
}

/// Canonical family for a lifecycle operation under the simple policy:
/// value-producing lifecycle mutations record the base product family when
/// the decision is product-ready; update receipts record the premium update
/// family; inspection records the read allowance; and uninstall/purge/
/// rollback plus every recovery or denied posture record the recovery
/// allowance family.
fn lifecycle_policy_family(
    operation: LifecycleOperation,
    class: LifecycleEntitlementReceiptClass,
    product_ready: bool,
) -> String {
    if !product_ready || matches!(class, LifecycleEntitlementReceiptClass::RecoveryReady) {
        return focusa_license::CapabilityFamily::AccountRecovery
            .label()
            .to_string();
    }
    match operation {
        LifecycleOperation::Update => focusa_license::CapabilityFamily::PremiumUpdates
            .label()
            .to_string(),
        LifecycleOperation::Inspect => focusa_license::CapabilityFamily::ReadProjection
            .label()
            .to_string(),
        LifecycleOperation::Uninstall
        | LifecycleOperation::Purge
        | LifecycleOperation::Rollback => focusa_license::CapabilityFamily::AccountRecovery
            .label()
            .to_string(),
        LifecycleOperation::Install | LifecycleOperation::Repair | LifecycleOperation::Rerun => {
            focusa_license::CapabilityFamily::BaseFocusa
                .label()
                .to_string()
        }
    }
}

fn lifecycle_entitlement_state_label(state: LifecycleEntitlementState) -> String {
    serde_json::to_value(state)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_else(|| "unknown".into())
}

/// Entitlement-state labels consistent with each receipt class. The recorded
/// state must be one of these so reconciliation cannot drift into an invented
/// or contradicting state claim.
fn lifecycle_receipt_class_states(
    class: LifecycleEntitlementReceiptClass,
) -> &'static [&'static str] {
    match class {
        LifecycleEntitlementReceiptClass::LimitedAccessReady => &["active_verified_limited"],
        LifecycleEntitlementReceiptClass::PaidReady => &["active_paid"],
        LifecycleEntitlementReceiptClass::DevelopmentReady => &["offline_grace"],
        LifecycleEntitlementReceiptClass::RecoveryReady => &["offline_grace", "none"],
        LifecycleEntitlementReceiptClass::BlockedEntitlement => &[
            "unactivated",
            "pending_identity",
            "pending_device_code",
            "expired",
            "revoked",
            "invalid",
            "none",
        ],
    }
}

#[cfg(test)]
#[path = "receipt_tests.rs"]
mod tests;
