//! Typed facade operations and errors (Spec 152E §9, §10, §20 and the frozen
//! `spec152e-activation-errors.v1.json` registry).
//!
//! Facades and presenters expose branded paths and proxy to the WPUIAI.com
//! authority kernel; they have zero identity, commerce, or entitlement
//! authority. This module gives every surface one typed operation and one
//! typed error code so unknown operations and unknown errors fail closed
//! instead of being stringly re-decided per presenter.

use serde::{Deserialize, Serialize};

/// The eleven public activation operations from the frozen call stack.
/// Clients and facades submit only these typed operations; the authority owns
/// all product, price, grant, feature, limit, and entitlement decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FacadeOperation {
    ActivationStart,
    ActivationVerify,
    ActivationOffers,
    ActivationSelectOffer,
    ActivationCheckout,
    ActivationExistingLicense,
    ActivationPoll,
    LeaseRefresh,
    NodesList,
    NodesDeactivate,
    AccountManageLink,
}

impl FacadeOperation {
    pub const ALL: [FacadeOperation; 11] = [
        Self::ActivationStart,
        Self::ActivationVerify,
        Self::ActivationOffers,
        Self::ActivationSelectOffer,
        Self::ActivationCheckout,
        Self::ActivationExistingLicense,
        Self::ActivationPoll,
        Self::LeaseRefresh,
        Self::NodesList,
        Self::NodesDeactivate,
        Self::AccountManageLink,
    ];

    pub const fn id(self) -> &'static str {
        match self {
            Self::ActivationStart => "activation.start",
            Self::ActivationVerify => "activation.verify",
            Self::ActivationOffers => "activation.offers",
            Self::ActivationSelectOffer => "activation.select_offer",
            Self::ActivationCheckout => "activation.checkout",
            Self::ActivationExistingLicense => "activation.existing_license",
            Self::ActivationPoll => "activation.poll",
            Self::LeaseRefresh => "lease.refresh",
            Self::NodesList => "nodes.list",
            Self::NodesDeactivate => "nodes.deactivate",
            Self::AccountManageLink => "account.manage_link",
        }
    }

    pub const fn method(self) -> &'static str {
        match self {
            Self::ActivationOffers | Self::NodesList | Self::AccountManageLink => "GET",
            _ => "POST",
        }
    }

    pub const fn path(self) -> &'static str {
        match self {
            Self::ActivationStart => "/v1/activation/start",
            Self::ActivationVerify => "/v1/activation/verify",
            Self::ActivationOffers => "/v1/activation/offers",
            Self::ActivationSelectOffer => "/v1/activation/select-offer",
            Self::ActivationCheckout => "/v1/activation/checkout",
            Self::ActivationExistingLicense => "/v1/activation/existing-license",
            Self::ActivationPoll => "/v1/activation/poll",
            Self::LeaseRefresh => "/v1/lease/refresh",
            Self::NodesList => "/v1/nodes",
            Self::NodesDeactivate => "/v1/nodes/deactivate",
            Self::AccountManageLink => "/v1/account/manage-link",
        }
    }

    pub const fn is_mutation(self) -> bool {
        !matches!(
            self,
            Self::ActivationOffers | Self::NodesList | Self::AccountManageLink
        )
    }

    /// Presenter states this operation may legitimately settle to.
    pub const fn success_presenter_states(self) -> &'static [&'static str] {
        match self {
            Self::ActivationStart => &["email_verification_pending"],
            Self::ActivationVerify => &["email_verified", "selection_required"],
            Self::ActivationOffers => &["selection_required"],
            Self::ActivationSelectOffer => &["checkout_required", "selection_required"],
            Self::ActivationCheckout => &["checkout_required", "payment_pending"],
            Self::ActivationExistingLicense => &["license_delivery_ready", "activated"],
            Self::ActivationPoll => &[
                "email_verification_pending",
                "payment_pending",
                "license_delivery_ready",
                "activated",
                "denied",
                "recovery_only",
            ],
            Self::LeaseRefresh => &["activated", "recovery_only"],
            Self::NodesList => &["activated", "recovery_only"],
            Self::NodesDeactivate => &["activated", "recovery_only"],
            Self::AccountManageLink => &["activated", "recovery_only"],
        }
    }

    /// Stable failure codes this operation may return (frozen contract).
    pub const fn failure_codes(self) -> &'static [ActivationErrorCode] {
        match self {
            Self::ActivationStart => &[
                ActivationErrorCode::EmailRequired,
                ActivationErrorCode::EmailDeliveryFailed,
                ActivationErrorCode::FacadeOriginDenied,
                ActivationErrorCode::FacadeProductDenied,
                ActivationErrorCode::ProductMappingRequired,
                ActivationErrorCode::RequestIdRequired,
                ActivationErrorCode::IdempotencyKeyRequired,
                ActivationErrorCode::IdempotencyConflict,
                ActivationErrorCode::AuthorityUnavailable,
            ],
            Self::ActivationVerify => &[
                ActivationErrorCode::EmailVerificationRequired,
                ActivationErrorCode::EmailVerificationExpired,
                ActivationErrorCode::EmailVerificationFailed,
                ActivationErrorCode::EmailDeliveryFailed,
                ActivationErrorCode::AccountMergeReviewRequired,
                ActivationErrorCode::EddCustomerResolutionFailed,
                ActivationErrorCode::RequestIdRequired,
                ActivationErrorCode::IdempotencyKeyRequired,
                ActivationErrorCode::IdempotencyConflict,
                ActivationErrorCode::AuthorityUnavailable,
            ],
            Self::ActivationOffers => &[
                ActivationErrorCode::EmailVerificationRequired,
                ActivationErrorCode::FacadeProductDenied,
                ActivationErrorCode::ProductMappingRequired,
                ActivationErrorCode::RequestIdRequired,
                ActivationErrorCode::AuthorityUnavailable,
            ],
            Self::ActivationSelectOffer => &[
                ActivationErrorCode::EmailVerificationRequired,
                ActivationErrorCode::FacadeProductDenied,
                ActivationErrorCode::ProductMappingRequired,
                ActivationErrorCode::EvaluationNotEligible,
                ActivationErrorCode::RequestIdRequired,
                ActivationErrorCode::IdempotencyKeyRequired,
                ActivationErrorCode::IdempotencyConflict,
            ],
            Self::ActivationCheckout => &[
                ActivationErrorCode::EmailVerificationRequired,
                ActivationErrorCode::AccountEmailMismatch,
                ActivationErrorCode::ProductMappingRequired,
                ActivationErrorCode::EddCheckoutRequired,
                ActivationErrorCode::EddOrderUnverified,
                ActivationErrorCode::RequestIdRequired,
                ActivationErrorCode::IdempotencyKeyRequired,
                ActivationErrorCode::IdempotencyConflict,
                ActivationErrorCode::AuthorityUnavailable,
            ],
            Self::ActivationExistingLicense => &[
                ActivationErrorCode::EmailVerificationRequired,
                ActivationErrorCode::LicenseAccountMismatch,
                ActivationErrorCode::EddLicensePending,
                ActivationErrorCode::EddLicenseUnusable,
                ActivationErrorCode::NodeLimitExhausted,
                ActivationErrorCode::RequestIdRequired,
                ActivationErrorCode::IdempotencyKeyRequired,
                ActivationErrorCode::IdempotencyConflict,
                ActivationErrorCode::AuthorityUnavailable,
            ],
            Self::ActivationPoll => &[
                ActivationErrorCode::PollCredentialRequired,
                ActivationErrorCode::PollCredentialExpired,
                ActivationErrorCode::EddOrderPending,
                ActivationErrorCode::EddOrderUnverified,
                ActivationErrorCode::EddLicensePending,
                ActivationErrorCode::EddLicenseUnusable,
                ActivationErrorCode::LicenseDeliveryPending,
                ActivationErrorCode::LicenseDeliveryFailed,
                ActivationErrorCode::NodeLimitExhausted,
                ActivationErrorCode::RequestIdRequired,
                ActivationErrorCode::IdempotencyKeyRequired,
                ActivationErrorCode::IdempotencyConflict,
                ActivationErrorCode::RequestInProgress,
                ActivationErrorCode::AuthorityUnavailable,
            ],
            Self::LeaseRefresh => &[
                ActivationErrorCode::EddLicenseUnusable,
                ActivationErrorCode::LicenseAccountMismatch,
                ActivationErrorCode::Refunded,
                ActivationErrorCode::Revoked,
                ActivationErrorCode::EntitlementRequired,
                ActivationErrorCode::RequestIdRequired,
                ActivationErrorCode::IdempotencyKeyRequired,
                ActivationErrorCode::IdempotencyConflict,
                ActivationErrorCode::AuthorityUnavailable,
            ],
            Self::NodesList => &[
                ActivationErrorCode::EmailVerificationRequired,
                ActivationErrorCode::EntitlementRequired,
                ActivationErrorCode::RequestIdRequired,
                ActivationErrorCode::AuthorityUnavailable,
            ],
            Self::NodesDeactivate => &[
                ActivationErrorCode::EmailVerificationRequired,
                ActivationErrorCode::LicenseAccountMismatch,
                ActivationErrorCode::RequestIdRequired,
                ActivationErrorCode::IdempotencyKeyRequired,
                ActivationErrorCode::IdempotencyConflict,
                ActivationErrorCode::AuthorityUnavailable,
            ],
            Self::AccountManageLink => &[
                ActivationErrorCode::EmailVerificationRequired,
                ActivationErrorCode::AccountEmailMismatch,
                ActivationErrorCode::FacadeOriginDenied,
                ActivationErrorCode::RequestIdRequired,
                ActivationErrorCode::AuthorityUnavailable,
            ],
        }
    }
}

/// One immutable row of the frozen stable-failure registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivationErrorSpec {
    pub http_status: u16,
    pub retryable: bool,
    pub safe_next_action: &'static str,
}

/// The thirty-three stable failure codes from
/// `spec152e-activation-errors.v1.json`. Unknown codes have no typed variant
/// and therefore cannot be produced or consumed; they fail closed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ActivationErrorCode {
    AccountEmailMismatch,
    AccountMergeReviewRequired,
    AuthorityUnavailable,
    EddCheckoutRequired,
    EddCustomerResolutionFailed,
    EddLicensePending,
    EddLicenseUnusable,
    EddOrderPending,
    EddOrderUnverified,
    EmailDeliveryFailed,
    EmailRequired,
    EmailVerificationExpired,
    EmailVerificationFailed,
    EmailVerificationRequired,
    EntitlementFeatureRequired,
    EntitlementLimitExhausted,
    EntitlementRequired,
    EvaluationNotEligible,
    FacadeOriginDenied,
    FacadeProductDenied,
    IdempotencyConflict,
    IdempotencyKeyRequired,
    LicenseAccountMismatch,
    LicenseDeliveryFailed,
    LicenseDeliveryPending,
    NodeLimitExhausted,
    PollCredentialExpired,
    PollCredentialRequired,
    ProductMappingRequired,
    Refunded,
    RequestIdRequired,
    RequestInProgress,
    Revoked,
}

impl ActivationErrorCode {
    pub const ALL: [ActivationErrorCode; 33] = [
        Self::AccountEmailMismatch,
        Self::AccountMergeReviewRequired,
        Self::AuthorityUnavailable,
        Self::EddCheckoutRequired,
        Self::EddCustomerResolutionFailed,
        Self::EddLicensePending,
        Self::EddLicenseUnusable,
        Self::EddOrderPending,
        Self::EddOrderUnverified,
        Self::EmailDeliveryFailed,
        Self::EmailRequired,
        Self::EmailVerificationExpired,
        Self::EmailVerificationFailed,
        Self::EmailVerificationRequired,
        Self::EntitlementFeatureRequired,
        Self::EntitlementLimitExhausted,
        Self::EntitlementRequired,
        Self::EvaluationNotEligible,
        Self::FacadeOriginDenied,
        Self::FacadeProductDenied,
        Self::IdempotencyConflict,
        Self::IdempotencyKeyRequired,
        Self::LicenseAccountMismatch,
        Self::LicenseDeliveryFailed,
        Self::LicenseDeliveryPending,
        Self::NodeLimitExhausted,
        Self::PollCredentialExpired,
        Self::PollCredentialRequired,
        Self::ProductMappingRequired,
        Self::Refunded,
        Self::RequestIdRequired,
        Self::RequestInProgress,
        Self::Revoked,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::AccountEmailMismatch => "ACCOUNT_EMAIL_MISMATCH",
            Self::AccountMergeReviewRequired => "ACCOUNT_MERGE_REVIEW_REQUIRED",
            Self::AuthorityUnavailable => "AUTHORITY_UNAVAILABLE",
            Self::EddCheckoutRequired => "EDD_CHECKOUT_REQUIRED",
            Self::EddCustomerResolutionFailed => "EDD_CUSTOMER_RESOLUTION_FAILED",
            Self::EddLicensePending => "EDD_LICENSE_PENDING",
            Self::EddLicenseUnusable => "EDD_LICENSE_UNUSABLE",
            Self::EddOrderPending => "EDD_ORDER_PENDING",
            Self::EddOrderUnverified => "EDD_ORDER_UNVERIFIED",
            Self::EmailDeliveryFailed => "EMAIL_DELIVERY_FAILED",
            Self::EmailRequired => "EMAIL_REQUIRED",
            Self::EmailVerificationExpired => "EMAIL_VERIFICATION_EXPIRED",
            Self::EmailVerificationFailed => "EMAIL_VERIFICATION_FAILED",
            Self::EmailVerificationRequired => "EMAIL_VERIFICATION_REQUIRED",
            Self::EntitlementFeatureRequired => "ENTITLEMENT_FEATURE_REQUIRED",
            Self::EntitlementLimitExhausted => "ENTITLEMENT_LIMIT_EXHAUSTED",
            Self::EntitlementRequired => "ENTITLEMENT_REQUIRED",
            Self::EvaluationNotEligible => "EVALUATION_NOT_ELIGIBLE",
            Self::FacadeOriginDenied => "FACADE_ORIGIN_DENIED",
            Self::FacadeProductDenied => "FACADE_PRODUCT_DENIED",
            Self::IdempotencyConflict => "IDEMPOTENCY_CONFLICT",
            Self::IdempotencyKeyRequired => "IDEMPOTENCY_KEY_REQUIRED",
            Self::LicenseAccountMismatch => "LICENSE_ACCOUNT_MISMATCH",
            Self::LicenseDeliveryFailed => "LICENSE_DELIVERY_FAILED",
            Self::LicenseDeliveryPending => "LICENSE_DELIVERY_PENDING",
            Self::NodeLimitExhausted => "NODE_LIMIT_EXHAUSTED",
            Self::PollCredentialExpired => "POLL_CREDENTIAL_EXPIRED",
            Self::PollCredentialRequired => "POLL_CREDENTIAL_REQUIRED",
            Self::ProductMappingRequired => "PRODUCT_MAPPING_REQUIRED",
            Self::Refunded => "REFUNDED",
            Self::RequestIdRequired => "REQUEST_ID_REQUIRED",
            Self::RequestInProgress => "REQUEST_IN_PROGRESS",
            Self::Revoked => "REVOKED",
        }
    }

    pub const fn spec(self) -> ActivationErrorSpec {
        match self {
            Self::AccountEmailMismatch => ActivationErrorSpec {
                http_status: 403,
                retryable: false,
                safe_next_action: "verify_account_email",
            },
            Self::AccountMergeReviewRequired => ActivationErrorSpec {
                http_status: 409,
                retryable: false,
                safe_next_action: "contact_support",
            },
            Self::AuthorityUnavailable => ActivationErrorSpec {
                http_status: 503,
                retryable: true,
                safe_next_action: "retry_or_use_recovery",
            },
            Self::EddCheckoutRequired => ActivationErrorSpec {
                http_status: 409,
                retryable: false,
                safe_next_action: "open_checkout",
            },
            Self::EddCustomerResolutionFailed => ActivationErrorSpec {
                http_status: 503,
                retryable: true,
                safe_next_action: "retry_or_use_recovery",
            },
            Self::EddLicensePending => ActivationErrorSpec {
                http_status: 202,
                retryable: true,
                safe_next_action: "poll_after_retry_after",
            },
            Self::EddLicenseUnusable => ActivationErrorSpec {
                http_status: 403,
                retryable: false,
                safe_next_action: "recovery_only",
            },
            Self::EddOrderPending => ActivationErrorSpec {
                http_status: 202,
                retryable: true,
                safe_next_action: "poll_after_retry_after",
            },
            Self::EddOrderUnverified => ActivationErrorSpec {
                http_status: 409,
                retryable: false,
                safe_next_action: "verify_checkout_identity",
            },
            Self::EmailDeliveryFailed => ActivationErrorSpec {
                http_status: 503,
                retryable: true,
                safe_next_action: "retry_or_use_recovery",
            },
            Self::EmailRequired => ActivationErrorSpec {
                http_status: 400,
                retryable: false,
                safe_next_action: "provide_email",
            },
            Self::EmailVerificationExpired => ActivationErrorSpec {
                http_status: 410,
                retryable: false,
                safe_next_action: "restart_verification",
            },
            Self::EmailVerificationFailed => ActivationErrorSpec {
                http_status: 403,
                retryable: false,
                safe_next_action: "retry_verification_within_budget",
            },
            Self::EmailVerificationRequired => ActivationErrorSpec {
                http_status: 403,
                retryable: false,
                safe_next_action: "verify_email",
            },
            Self::EntitlementFeatureRequired => ActivationErrorSpec {
                http_status: 403,
                retryable: false,
                safe_next_action: "manage_license",
            },
            Self::EntitlementLimitExhausted => ActivationErrorSpec {
                http_status: 429,
                retryable: false,
                safe_next_action: "manage_limit",
            },
            Self::EntitlementRequired => ActivationErrorSpec {
                http_status: 403,
                retryable: false,
                safe_next_action: "activate_or_manage_license",
            },
            Self::EvaluationNotEligible => ActivationErrorSpec {
                http_status: 403,
                retryable: false,
                safe_next_action: "select_paid_or_limited_access",
            },
            Self::FacadeOriginDenied => ActivationErrorSpec {
                http_status: 403,
                retryable: false,
                safe_next_action: "use_registered_facade",
            },
            Self::FacadeProductDenied => ActivationErrorSpec {
                http_status: 403,
                retryable: false,
                safe_next_action: "select_supported_product",
            },
            Self::IdempotencyConflict => ActivationErrorSpec {
                http_status: 409,
                retryable: false,
                safe_next_action: "use_original_request_or_new_key",
            },
            Self::IdempotencyKeyRequired => ActivationErrorSpec {
                http_status: 400,
                retryable: false,
                safe_next_action: "send_idempotency_key",
            },
            Self::LicenseAccountMismatch => ActivationErrorSpec {
                http_status: 403,
                retryable: false,
                safe_next_action: "verify_license_owner",
            },
            Self::LicenseDeliveryFailed => ActivationErrorSpec {
                http_status: 503,
                retryable: false,
                safe_next_action: "authenticated_recovery",
            },
            Self::LicenseDeliveryPending => ActivationErrorSpec {
                http_status: 202,
                retryable: true,
                safe_next_action: "poll_after_retry_after",
            },
            Self::NodeLimitExhausted => ActivationErrorSpec {
                http_status: 409,
                retryable: false,
                safe_next_action: "manage_nodes",
            },
            Self::PollCredentialExpired => ActivationErrorSpec {
                http_status: 401,
                retryable: false,
                safe_next_action: "restart_or_recover_activation",
            },
            Self::PollCredentialRequired => ActivationErrorSpec {
                http_status: 401,
                retryable: false,
                safe_next_action: "restart_or_recover_activation",
            },
            Self::ProductMappingRequired => ActivationErrorSpec {
                http_status: 409,
                retryable: false,
                safe_next_action: "wait_for_product_mapping",
            },
            Self::Refunded => ActivationErrorSpec {
                http_status: 403,
                retryable: false,
                safe_next_action: "recovery_only",
            },
            Self::RequestIdRequired => ActivationErrorSpec {
                http_status: 400,
                retryable: false,
                safe_next_action: "send_new_request_id",
            },
            Self::RequestInProgress => ActivationErrorSpec {
                http_status: 409,
                retryable: true,
                safe_next_action: "retry_same_idempotency_key",
            },
            Self::Revoked => ActivationErrorSpec {
                http_status: 403,
                retryable: false,
                safe_next_action: "recovery_only",
            },
        }
    }

    pub const fn http_status(self) -> u16 {
        self.spec().http_status
    }

    pub const fn retryable(self) -> bool {
        self.spec().retryable
    }

    pub const fn safe_next_action(self) -> &'static str {
        self.spec().safe_next_action
    }
}

/// A typed, correlated activation error. The error carries only the public
/// code and opaque request identity — never authority, email, payment, or
/// credential data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivationError {
    pub code: ActivationErrorCode,
    pub request_id: String,
}

impl ActivationError {
    pub fn new(code: ActivationErrorCode, request_id: String) -> Self {
        Self { code, request_id }
    }
}

impl std::fmt::Display for ActivationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.code.label())
    }
}

/// Typed request context required by the frozen call stack. `required_all`
/// fields are always present; `idempotency_key` is required for mutations.
/// Callers cannot inject any forbidden field (`email_verified`, `account_id`,
/// `edd_customer_id`, `edd_download_id`, `edd_price_id`, `order_id`,
/// `license_id`, `price`, `tier`, `products`, `grants`, `features`, `limits`,
/// `node_limit`, `commercial_rights`, `entitlement_sequence`, `lease`,
/// `refund_status`): the struct has no field for any of them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivationRequestContext {
    pub request_id: String,
    pub facade_id: String,
    pub presenter: String,
    pub install_channel: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
}

impl ActivationRequestContext {
    /// Frozen forbidden caller fields; kept as a named constant so static
    /// contract scans can bind it.
    pub const FORBIDDEN_CALLER_FIELDS: [&'static str; 18] = [
        "email_verified",
        "account_id",
        "edd_customer_id",
        "edd_download_id",
        "edd_price_id",
        "order_id",
        "license_id",
        "price",
        "tier",
        "products",
        "grants",
        "features",
        "limits",
        "node_limit",
        "commercial_rights",
        "entitlement_sequence",
        "lease",
        "refund_status",
    ];

    pub fn new(
        request_id: impl Into<String>,
        facade_id: impl Into<String>,
        presenter: impl Into<String>,
        install_channel: impl Into<String>,
        idempotency_key: Option<String>,
    ) -> Self {
        Self {
            request_id: request_id.into(),
            facade_id: facade_id.into(),
            presenter: presenter.into(),
            install_channel: install_channel.into(),
            idempotency_key,
        }
    }

    /// Fail-closed context validation. Missing identity fields and missing
    /// idempotency keys on mutations return the frozen typed codes.
    pub fn validate(&self, operation: FacadeOperation) -> Result<(), ActivationError> {
        let request_id = self.request_id.clone();
        if self.request_id.trim().is_empty()
            || self.facade_id.trim().is_empty()
            || self.presenter.trim().is_empty()
            || self.install_channel.trim().is_empty()
        {
            return Err(ActivationError::new(
                ActivationErrorCode::RequestIdRequired,
                request_id,
            ));
        }
        if operation.is_mutation()
            && self
                .idempotency_key
                .as_deref()
                .is_none_or(|value| value.trim().is_empty())
        {
            return Err(ActivationError::new(
                ActivationErrorCode::IdempotencyKeyRequired,
                request_id,
            ));
        }
        Ok(())
    }
}

/// Mask an email for presenter output: the frozen pattern is
/// `^[^@]*\*[^@]*@[^@]+$` (e.g. `c***@example.com`). Unmaskable or invalid
/// inputs return `None` so presenters never fall back to a raw address.
pub fn mask_email(email: &str) -> Option<String> {
    let email = email.trim();
    let (local, domain) = email.split_once('@')?;
    if local.is_empty()
        || local.chars().any(char::is_whitespace)
        || domain.is_empty()
        || domain.contains('@')
        || domain.contains(' ')
        || !domain.contains('.')
    {
        return None;
    }
    let head: String = local.chars().take(1).collect();
    if head.is_empty() {
        return None;
    }
    Some(format!("{head}***@{domain}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operation_ids_paths_methods_match_the_frozen_call_stack() {
        let expected: &[(&str, &str, &str, bool)] = &[
            ("activation.start", "POST", "/v1/activation/start", true),
            ("activation.verify", "POST", "/v1/activation/verify", true),
            ("activation.offers", "GET", "/v1/activation/offers", false),
            (
                "activation.select_offer",
                "POST",
                "/v1/activation/select-offer",
                true,
            ),
            (
                "activation.checkout",
                "POST",
                "/v1/activation/checkout",
                true,
            ),
            (
                "activation.existing_license",
                "POST",
                "/v1/activation/existing-license",
                true,
            ),
            ("activation.poll", "POST", "/v1/activation/poll", true),
            ("lease.refresh", "POST", "/v1/lease/refresh", true),
            ("nodes.list", "GET", "/v1/nodes", false),
            ("nodes.deactivate", "POST", "/v1/nodes/deactivate", true),
            (
                "account.manage_link",
                "GET",
                "/v1/account/manage-link",
                false,
            ),
        ];
        assert_eq!(FacadeOperation::ALL.len(), expected.len());
        for (operation, (id, method, path, mutation)) in FacadeOperation::ALL.iter().zip(expected) {
            assert_eq!(operation.id(), *id);
            assert_eq!(operation.method(), *method);
            assert_eq!(operation.path(), *path);
            assert_eq!(operation.is_mutation(), *mutation);
            assert!(!operation.failure_codes().is_empty());
            assert!(!operation.success_presenter_states().is_empty());
        }
        let ids = FacadeOperation::ALL
            .iter()
            .map(|operation| operation.id())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(ids.len(), 11);
    }

    #[test]
    fn all_33_error_codes_have_unique_labels_and_registry_values() {
        let labels = ActivationErrorCode::ALL
            .iter()
            .map(|code| code.label())
            .collect::<Vec<_>>();
        assert_eq!(labels.len(), 33);
        let unique = labels.iter().collect::<std::collections::BTreeSet<_>>();
        assert_eq!(unique.len(), 33);
        for code in ActivationErrorCode::ALL {
            let spec = code.spec();
            assert!(
                [202, 400, 401, 403, 409, 410, 429, 503].contains(&spec.http_status),
                "{} has a frozen http_status",
                code.label()
            );
            assert!(!spec.safe_next_action.is_empty());
            if code.retryable() {
                assert!(
                    [
                        "poll_after_retry_after",
                        "retry_or_use_recovery",
                        "retry_same_idempotency_key",
                    ]
                    .contains(&spec.safe_next_action)
                );
            }
        }
    }

    #[test]
    fn request_context_validates_fail_closed_and_has_no_forbidden_fields() {
        let mutation = ActivationRequestContext::new(
            "request-0001",
            "install.focusa.dev",
            "cli",
            "official_installer",
            None,
        );
        assert_eq!(
            mutation.validate(FacadeOperation::ActivationStart),
            Err(ActivationError::new(
                ActivationErrorCode::IdempotencyKeyRequired,
                "request-0001".to_string()
            ))
        );

        let read = ActivationRequestContext::new(
            "request-0001",
            "install.focusa.dev",
            "cli",
            "official_installer",
            None,
        );
        assert!(read.validate(FacadeOperation::ActivationOffers).is_ok());

        let missing_request = ActivationRequestContext::new(
            "",
            "install.focusa.dev",
            "cli",
            "official_installer",
            Some("idem-0001".into()),
        );
        assert_eq!(
            missing_request.validate(FacadeOperation::ActivationStart),
            Err(ActivationError::new(
                ActivationErrorCode::RequestIdRequired,
                String::new()
            ))
        );
        assert_eq!(ActivationRequestContext::FORBIDDEN_CALLER_FIELDS.len(), 18);
    }

    #[test]
    fn mask_email_never_returns_a_raw_address() {
        assert_eq!(
            mask_email("customer@example.com").as_deref(),
            Some("c***@example.com")
        );
        assert_eq!(
            mask_email("a@example.com").as_deref(),
            Some("a***@example.com")
        );
        for invalid in ["", "not-an-email", "a@b", "a b@example.com", "@example.com"] {
            assert_eq!(
                mask_email(invalid),
                None,
                "must fail closed for {invalid:?}"
            );
        }
    }

    #[test]
    fn error_serde_round_trip_uses_canonical_labels() {
        for code in ActivationErrorCode::ALL {
            let json = serde_json::to_string(&code).unwrap();
            assert_eq!(json, format!("\"{}\"", code.label()));
            let back: ActivationErrorCode = serde_json::from_str(&json).unwrap();
            assert_eq!(back, code);
        }
    }
}
