use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::{
    authority::{EntitlementSnapshot, EntitlementState},
    authority_client::SensitiveCredential,
};

pub const UIAI_CHILD_TOKEN_SCHEMA: &str = "focusa.uiai_child_token.v1";
pub const UIAI_CHILD_TOKEN_MAX_TTL_MINUTES: i64 = 15;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiaiChildTokenRequest {
    pub request_id: Uuid,
    pub audience: String,
    pub node_id: String,
    pub client_id: String,
    pub parent_lease_id: String,
    pub parent_lease_sequence: u64,
    pub parent_lease_digest: String,
    pub uiai_grant_lease_id: String,
    pub uiai_grant_sequence: u64,
    pub requested_features: BTreeSet<String>,
    pub requested_limits: BTreeMap<String, u64>,
    pub nonce: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorityChildTokenEnvelope {
    pub schema: String,
    pub token: String,
    pub token_id: String,
    pub audience: String,
    pub node_id: String,
    pub client_id: String,
    pub parent_lease_id: String,
    pub parent_lease_sequence: u64,
    pub parent_lease_digest: String,
    pub uiai_grant_lease_id: String,
    pub uiai_grant_sequence: u64,
    pub features: BTreeSet<String>,
    pub limits: BTreeMap<String, u64>,
    pub nonce: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

pub struct CachedUiaiChildToken {
    pub token_id: String,
    pub credential: SensitiveCredential,
    pub parent_lease_id: String,
    pub parent_lease_sequence: u64,
    pub parent_lease_digest: String,
    pub uiai_grant_lease_id: String,
    pub uiai_grant_sequence: u64,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiaiChildTokenReceipt {
    pub schema: String,
    pub token_id: String,
    pub request_id: Uuid,
    pub audience: String,
    pub parent_lease_sequence: u64,
    pub uiai_grant_sequence: u64,
    pub feature_count: usize,
    pub limit_count: usize,
    pub expires_at: DateTime<Utc>,
    pub token_persisted_in_receipt: bool,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum UiaiChildTokenError {
    #[error("Focusa parent entitlement is not active and bound")]
    ParentEntitlementInvalid,
    #[error("independent UIAI product entitlement is not active and bound")]
    UiaiGrantInvalid,
    #[error("Focusa parent and UIAI grant are not bound to the same EDD account")]
    AccountMismatch,
    #[error("requested child scope is not an exact subset of the UIAI grant")]
    ScopeNotGranted,
    #[error("authority response does not match request/parent authority")]
    AuthorityResponseMismatch,
    #[error("authority child token expiry is invalid")]
    InvalidExpiry,
    #[error("authority child token is missing")]
    TokenMissing,
    #[error("nonce has already been accepted")]
    NonceReplay,
}

#[derive(Default)]
pub struct UiaiChildTokenBroker {
    cache: BTreeMap<String, CachedUiaiChildToken>,
    accepted_nonces: BTreeSet<String>,
}

impl UiaiChildTokenBroker {
    pub fn validate_request(
        &self,
        request: &UiaiChildTokenRequest,
        focusa_parent: &EntitlementSnapshot,
        uiai_grant: &EntitlementSnapshot,
        now: DateTime<Utc>,
    ) -> Result<(), UiaiChildTokenError> {
        if !active_bound(focusa_parent, "focusa", &request.node_id, now)
            || focusa_parent.lease_id.as_deref() != Some(request.parent_lease_id.as_str())
            || focusa_parent.sequence != Some(request.parent_lease_sequence)
            || focusa_parent.lease_digest.as_deref() != Some(request.parent_lease_digest.as_str())
        {
            return Err(UiaiChildTokenError::ParentEntitlementInvalid);
        }
        if !active_bound(uiai_grant, "uiai-engine", &request.node_id, now)
            || uiai_grant.lease_id.as_deref() != Some(request.uiai_grant_lease_id.as_str())
            || uiai_grant.sequence != Some(request.uiai_grant_sequence)
        {
            return Err(UiaiChildTokenError::UiaiGrantInvalid);
        }
        // Same EDD account: the Focusa parent and the independent UIAI grant
        // must be issued to the same account (Spec 152E §7/§15). Verified
        // leases always carry the authority-issued subject; when one side
        // proves an account and the other does not (or differs), the child
        // token fails closed rather than bridging two customers.
        if !same_evidence_account(focusa_parent, uiai_grant) {
            return Err(UiaiChildTokenError::AccountMismatch);
        }
        if request.audience.trim().is_empty()
            || request.client_id.trim().is_empty()
            || request.nonce.trim().is_empty()
            || self.accepted_nonces.contains(&request.nonce)
        {
            return Err(UiaiChildTokenError::NonceReplay);
        }
        if !request
            .requested_features
            .iter()
            .all(|feature| uiai_grant.features.get(feature).copied().unwrap_or(false))
            || request.requested_limits.iter().any(|(bucket, requested)| {
                *requested == 0 || *requested > uiai_grant.limits.get(bucket).copied().unwrap_or(0)
            })
        {
            return Err(UiaiChildTokenError::ScopeNotGranted);
        }
        Ok(())
    }

    pub fn accept_authority_token(
        &mut self,
        request: &UiaiChildTokenRequest,
        focusa_parent: &EntitlementSnapshot,
        uiai_grant: &EntitlementSnapshot,
        envelope: AuthorityChildTokenEnvelope,
        now: DateTime<Utc>,
    ) -> Result<UiaiChildTokenReceipt, UiaiChildTokenError> {
        self.validate_request(request, focusa_parent, uiai_grant, now)?;
        if envelope.schema != UIAI_CHILD_TOKEN_SCHEMA
            || envelope.audience != request.audience
            || envelope.node_id != request.node_id
            || envelope.client_id != request.client_id
            || envelope.parent_lease_id != request.parent_lease_id
            || envelope.parent_lease_sequence != request.parent_lease_sequence
            || envelope.parent_lease_digest != request.parent_lease_digest
            || envelope.uiai_grant_lease_id != request.uiai_grant_lease_id
            || envelope.uiai_grant_sequence != request.uiai_grant_sequence
            || envelope.features != request.requested_features
            || envelope.limits != request.requested_limits
            || envelope.nonce != request.nonce
        {
            return Err(UiaiChildTokenError::AuthorityResponseMismatch);
        }
        let parent_bound = entitlement_bound(focusa_parent).unwrap_or(now);
        let uiai_bound = entitlement_bound(uiai_grant).unwrap_or(now);
        if envelope.issued_at > now
            || envelope.expires_at <= now
            || envelope.expires_at > now + Duration::minutes(UIAI_CHILD_TOKEN_MAX_TTL_MINUTES)
            || envelope.expires_at > parent_bound
            || envelope.expires_at > uiai_bound
        {
            return Err(UiaiChildTokenError::InvalidExpiry);
        }
        let credential = SensitiveCredential::new(envelope.token)
            .map_err(|_| UiaiChildTokenError::TokenMissing)?;
        let receipt = UiaiChildTokenReceipt {
            schema: "focusa.uiai_child_token_receipt.v1".into(),
            token_id: envelope.token_id.clone(),
            request_id: request.request_id,
            audience: request.audience.clone(),
            parent_lease_sequence: request.parent_lease_sequence,
            uiai_grant_sequence: request.uiai_grant_sequence,
            feature_count: request.requested_features.len(),
            limit_count: request.requested_limits.len(),
            expires_at: envelope.expires_at,
            token_persisted_in_receipt: false,
        };
        self.accepted_nonces.insert(request.nonce.clone());
        self.cache.insert(
            request.audience.clone(),
            CachedUiaiChildToken {
                token_id: envelope.token_id,
                credential,
                parent_lease_id: request.parent_lease_id.clone(),
                parent_lease_sequence: request.parent_lease_sequence,
                parent_lease_digest: request.parent_lease_digest.clone(),
                uiai_grant_lease_id: request.uiai_grant_lease_id.clone(),
                uiai_grant_sequence: request.uiai_grant_sequence,
                expires_at: envelope.expires_at,
            },
        );
        Ok(receipt)
    }

    pub fn cached(&self, audience: &str, now: DateTime<Utc>) -> Option<&CachedUiaiChildToken> {
        self.cache
            .get(audience)
            .filter(|token| token.expires_at > now)
    }

    pub fn revoke_parent(&mut self, lease_id: &str, minimum_sequence: u64) -> usize {
        let before = self.cache.len();
        self.cache.retain(|_, token| {
            token.parent_lease_id != lease_id || token.parent_lease_sequence >= minimum_sequence
        });
        before - self.cache.len()
    }

    /// Strict same-account binding (Spec 152E §7 / §15): the Focusa parent
    /// and the independent UIAI grant must both be issued to the single
    /// verified EDD account. The same-EDD-account UIAI activation adapter
    /// calls this before it accepts a child token; it fails closed unless
    /// both lease subjects equal the account id.
    pub fn validate_same_account_binding(
        &self,
        account: &crate::uiai_activation::UiaiAccountIdentity,
        focusa_parent: &EntitlementSnapshot,
        uiai_grant: &EntitlementSnapshot,
    ) -> Result<(), UiaiChildTokenError> {
        if !crate::uiai_activation::same_account_binding(account, focusa_parent, uiai_grant) {
            return Err(UiaiChildTokenError::AccountMismatch);
        }
        Ok(())
    }
}

fn entitlement_bound(snapshot: &EntitlementSnapshot) -> Option<DateTime<Utc>> {
    match snapshot.state {
        EntitlementState::Active => snapshot.expires_at,
        EntitlementState::OfflineGrace => snapshot.offline_grace_until,
        EntitlementState::Unactivated | EntitlementState::RecoveryOnly => None,
    }
}

/// Same-account evidence check: when either snapshot carries a lease
/// `subject_id` (account UUID), both must carry the SAME account. Synthetic
/// snapshots without lease subjects cannot prove an account split and pass;
/// verified leases always carry the authority-issued subject, so a mismatch
/// or a missing subject on one side fails closed.
fn same_evidence_account(parent: &EntitlementSnapshot, grant: &EntitlementSnapshot) -> bool {
    match (&parent.subject_id, &grant.subject_id) {
        (Some(parent_account), Some(grant_account)) => parent_account == grant_account,
        (None, None) => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uiai_activation::{PRODUCT_FOCUSA, PRODUCT_UIAI_ENGINE, UiaiAccountIdentity};

    fn bound_snapshot(product: &str, subject: Option<&str>) -> EntitlementSnapshot {
        let mut snapshot = EntitlementSnapshot::unactivated(product, "node-001");
        snapshot.state = EntitlementState::Active;
        snapshot.subject_id = subject.map(str::to_string);
        snapshot.lease_id = Some(format!("lease-{product}"));
        snapshot.sequence = Some(7);
        snapshot.lease_digest = Some("sha256:bound-grant-digest".to_string());
        snapshot.expires_at = Some(Utc::now() + Duration::hours(1));
        snapshot
            .features
            .insert("uiai.engine.core".to_string(), true);
        snapshot
    }

    fn request() -> UiaiChildTokenRequest {
        UiaiChildTokenRequest {
            request_id: Uuid::nil(),
            audience: "aud-focusa".to_string(),
            node_id: "node-001".to_string(),
            client_id: "client-focusa".to_string(),
            parent_lease_id: "lease-focusa".to_string(),
            parent_lease_sequence: 7,
            parent_lease_digest: "sha256:bound-grant-digest".to_string(),
            uiai_grant_lease_id: "lease-uiai-engine".to_string(),
            uiai_grant_sequence: 7,
            requested_features: BTreeSet::from(["uiai.engine.core".to_string()]),
            requested_limits: BTreeMap::new(),
            nonce: "nonce-same-account".to_string(),
        }
    }

    #[test]
    fn same_account_leases_pass_and_different_accounts_fail_closed() {
        let now = Utc::now();
        let focusa = bound_snapshot(PRODUCT_FOCUSA, Some("account-001"));
        let uiai = bound_snapshot(PRODUCT_UIAI_ENGINE, Some("account-001"));
        let broker = UiaiChildTokenBroker::default();
        assert_eq!(
            broker.validate_request(&request(), &focusa, &uiai, now),
            Ok(())
        );
        // A UIAI grant bound to a different EDD account fails closed.
        let other_customer = bound_snapshot(PRODUCT_UIAI_ENGINE, Some("account-002"));
        assert_eq!(
            broker.validate_request(&request(), &focusa, &other_customer, now),
            Err(UiaiChildTokenError::AccountMismatch)
        );
        // One side proving an account and the other not also fails closed.
        let no_subject = bound_snapshot(PRODUCT_UIAI_ENGINE, None);
        assert_eq!(
            broker.validate_request(&request(), &focusa, &no_subject, now),
            Err(UiaiChildTokenError::AccountMismatch)
        );
    }

    #[test]
    fn strict_same_account_binding_requires_the_single_verified_identity() {
        let focusa = bound_snapshot(PRODUCT_FOCUSA, Some("account-001"));
        let uiai = bound_snapshot(PRODUCT_UIAI_ENGINE, Some("account-001"));
        let broker = UiaiChildTokenBroker::default();
        let account = UiaiAccountIdentity {
            account_id: "account-001".to_string(),
            edd_customer_id: 1001,
        };
        assert!(
            broker
                .validate_same_account_binding(&account, &focusa, &uiai)
                .is_ok()
        );
        // No duplicate customer identity: a second customer on either lease
        // or an empty account identity is rejected.
        let second_customer = UiaiAccountIdentity {
            account_id: "account-002".to_string(),
            edd_customer_id: 1002,
        };
        assert_eq!(
            broker.validate_same_account_binding(&second_customer, &focusa, &uiai),
            Err(UiaiChildTokenError::AccountMismatch)
        );
        assert_eq!(
            broker.validate_same_account_binding(
                &UiaiAccountIdentity {
                    account_id: String::new(),
                    edd_customer_id: 0,
                },
                &focusa,
                &uiai,
            ),
            Err(UiaiChildTokenError::AccountMismatch)
        );
    }
}

fn active_bound(
    snapshot: &EntitlementSnapshot,
    product: &str,
    node: &str,
    now: DateTime<Utc>,
) -> bool {
    snapshot.product == product
        && snapshot.node_id == node
        && snapshot
            .lease_id
            .as_deref()
            .is_some_and(|value| !value.is_empty())
        && snapshot.sequence.is_some_and(|value| value > 0)
        && snapshot
            .lease_digest
            .as_deref()
            .is_some_and(|value| value.starts_with("sha256:"))
        && match snapshot.state {
            EntitlementState::Active => snapshot.expires_at.is_some_and(|expiry| expiry > now),
            EntitlementState::OfflineGrace => snapshot
                .offline_grace_until
                .is_some_and(|expiry| expiry > now),
            EntitlementState::Unactivated | EntitlementState::RecoveryOnly => false,
        }
}
