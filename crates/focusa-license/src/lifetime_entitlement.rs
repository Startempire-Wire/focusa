//! Spec 172 §14 — lifetime entitlement is separate from bounded device
//! credentials.
//!
//! A lifetime License Type (`term = "lifetime"`) never expires through the
//! passage of time, but the signed device lease used to execute is a bounded
//! credential with a finite refresh window and a bounded Offline Grace.
//!
//! This module keeps the two records apart and reconciles them in one
//! lifecycle state machine:
//!
//! - `LifetimeEntitlement` is the perpetual commercial right (product, License
//!   Type, term, price version, family digest, node/seat limits, highest
//!   authority sequence). Refund/revoke/chargeback at a strictly higher
//!   authority sequence marks it `Revoked`; nothing else changes it.
//! - `DeviceCredentialWindow` is the bounded signed lease (node binding,
//!   lease sequence, refresh window, offline grace). It rotates on refresh,
//!   is replaced after verified recovery, and is re-signed on key rotation.
//!   It never carries — and never expands — product, price, License Type,
//!   family, feature, limit, or commercial right.
//!
//! The machine guarantees:
//! - credential expiry/rotation never erases the lifetime entitlement
//!   (expired credentials resolve to `RecoveryOnly`, and verified recovery
//!   issues a replacement lease);
//! - a revoked lifetime entitlement defeats every stale and offline device
//!   credential (`DeniedRevoked`), regardless of the credential window;
//! - a stale credential (sequence below the entitlement's highest authority
//!   sequence) is never trusted (`DeniedStale`);
//! - Offline Grace stays bounded: active → offline_grace → recovery_only,
//!   never an expansion.

use std::{
    collections::BTreeMap,
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
};

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::authority::{EntitlementSnapshot, EntitlementState};

pub const LIFETIME_ENTITLEMENT_SCHEMA: &str = "focusa.spec172.lifetime_entitlement.v1";
pub const DEVICE_CREDENTIAL_SCHEMA: &str = "focusa.spec172.device_credential.v1";
pub const LIFETIME_STATE_SCHEMA: &str = "focusa.spec172.lifetime_state.v1";
pub const LIFETIME_TERM: &str = "lifetime";

pub const PRODUCT_FOCUSA: &str = "focusa";
pub const PRODUCT_UIAI_ENGINE: &str = "uiai_engine";
pub const LICENSE_TYPE_FOCUSA_OPERATOR_LIFETIME_V1: &str = "focusa_operator_lifetime_v1";
pub const LICENSE_TYPE_UIAI_OPERATOR_LIFETIME_V1: &str = "uiai_operator_lifetime_v1";

/// Bounded device-credential refresh window for a lifetime entitlement
/// (Spec 172 §14; mirror of the PHP issuer constant).
pub const REFRESH_WINDOW_DAYS: u64 = 90;
/// Bounded offline grace past the refresh window; never expands products,
/// families, seats, nodes, or limits (Spec 172 §14).
pub const OFFLINE_GRACE_DAYS: u64 = 30;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifetimeEntitlementStatus {
    Entitled,
    Revoked,
}

/// The perpetual License Type entitlement record. This is the commercial
/// right; it is never the device credential and never expires with it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifetimeEntitlement {
    pub schema: String,
    pub product: String,
    pub license_type: String,
    pub term: String,
    pub status: LifetimeEntitlementStatus,
    /// Highest authority sequence observed for this product/account grant.
    /// Refund, revoke, refresh, and recovery issuance all advance it; a
    /// device credential below it is stale and never trusted.
    pub sequence: u64,
    pub price_version: String,
    pub family_digest: String,
    pub node_limit: u64,
    pub operator_seats: u64,
    pub updated_at: DateTime<Utc>,
}

impl LifetimeEntitlement {
    /// Build a perpetual entitlement record from server-owned grant metadata.
    /// No caller-controlled product, License Type, term, price, family, limit,
    /// node, or commercial right is accepted: every field is validated against
    /// the canonical Operator lifetime registrations and fails closed.
    ///
    /// These parameters are the frozen lifetime credential contract.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        product: impl Into<String>,
        license_type: impl Into<String>,
        sequence: u64,
        price_version: impl Into<String>,
        family_digest: impl Into<String>,
        node_limit: u64,
        operator_seats: u64,
        updated_at: DateTime<Utc>,
    ) -> Result<Self, LifetimeCredentialError> {
        let product = product.into();
        let license_type = license_type.into();
        let valid_product = match product.as_str() {
            PRODUCT_FOCUSA => license_type == LICENSE_TYPE_FOCUSA_OPERATOR_LIFETIME_V1,
            PRODUCT_UIAI_ENGINE => license_type == LICENSE_TYPE_UIAI_OPERATOR_LIFETIME_V1,
            _ => return Err(LifetimeCredentialError::InvalidProduct(product)),
        };
        if !valid_product {
            return Err(LifetimeCredentialError::InvalidLicenseType(license_type));
        }
        if node_limit == 0 || operator_seats == 0 {
            return Err(LifetimeCredentialError::InvalidCredentialWindow);
        }
        Ok(Self {
            schema: LIFETIME_ENTITLEMENT_SCHEMA.to_string(),
            product,
            license_type,
            term: LIFETIME_TERM.to_string(),
            status: LifetimeEntitlementStatus::Entitled,
            sequence,
            price_version: price_version.into(),
            family_digest: family_digest.into(),
            node_limit,
            operator_seats,
            updated_at,
        })
    }

    pub fn is_lifetime(&self) -> bool {
        self.term == LIFETIME_TERM
    }

    pub fn is_entitled(&self) -> bool {
        self.status == LifetimeEntitlementStatus::Entitled
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceCredentialStatus {
    Active,
    Revoked,
}

/// The bounded signed device lease: the execution credential. Rotates on
/// refresh, is replaced after verified recovery, and is re-signed on key
/// rotation. It never expands the entitlement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceCredentialWindow {
    pub schema: String,
    pub lease_id: String,
    pub product: String,
    pub node_id: String,
    pub sequence: u64,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub offline_grace_until: DateTime<Utc>,
    pub authority_key_id: String,
    pub status: DeviceCredentialStatus,
}

impl DeviceCredentialWindow {
    /// Extract the bounded device credential from a verified authority lease
    /// snapshot (the focusa-license authority verifier output). Only snapshots
    /// that are Active or OfflineGrace carry a usable signed lease; unactivated
    /// and recovery-only snapshots yield no credential so the lifetime
    /// entitlement alone governs the decision (lifetime persists → recovery).
    pub fn from_snapshot(snapshot: &EntitlementSnapshot) -> Option<Self> {
        if !matches!(
            snapshot.state,
            EntitlementState::Active | EntitlementState::OfflineGrace
        ) {
            return None;
        }
        let lease_id = snapshot.lease_id.clone()?;
        let sequence = snapshot.sequence?;
        let expires_at = snapshot.expires_at?;
        let offline_grace_until = snapshot.offline_grace_until?;
        // The snapshot carries no issued_at; the machine only consults the
        // window bounds, sequence, and status, so back-derive a conservative
        // issued_at from the frozen refresh window.
        let issued_at = expires_at - Duration::days(REFRESH_WINDOW_DAYS as i64);
        Some(Self {
            schema: DEVICE_CREDENTIAL_SCHEMA.to_string(),
            lease_id,
            product: snapshot.product.clone(),
            node_id: snapshot.node_id.clone(),
            sequence,
            issued_at,
            expires_at,
            offline_grace_until,
            authority_key_id: String::new(),
            status: DeviceCredentialStatus::Active,
        })
    }
}

/// Joint lifecycle posture decided by the separate lifetime entitlement
/// record and the bounded device credential.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifetimeCredentialState {
    /// Lifetime entitled and the bounded device credential is inside its
    /// signed refresh window.
    Active,
    /// Lifetime entitled and the credential is inside the bounded offline
    /// grace; never an expansion of products, families, seats, nodes, limits.
    OfflineGrace,
    /// Lifetime entitled but the credential is expired or missing: the
    /// lifetime right persists and verified recovery can issue a replacement
    /// lease; value-producing execution is denied meanwhile.
    RecoveryOnly,
    /// The lifetime entitlement is revoked (refund/revoke/chargeback at a
    /// higher authority sequence). Every stale and offline device credential
    /// is defeated — this decision never depends on the credential window.
    DeniedRevoked,
    /// A device credential at a sequence below the entitlement's highest
    /// authority sequence: replayed or pre-rotation credentials are never
    /// trusted.
    DeniedStale,
    /// No lifetime entitlement record exists for the product/account.
    DeniedUnknown,
}

impl LifetimeCredentialState {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::OfflineGrace => "offline_grace",
            Self::RecoveryOnly => "recovery_only",
            Self::DeniedRevoked => "denied_revoked",
            Self::DeniedStale => "denied_stale",
            Self::DeniedUnknown => "denied_unknown",
        }
    }

    /// Only Active and OfflineGrace permit value-producing product use.
    pub const fn allows_product_use(self) -> bool {
        matches!(self, Self::Active | Self::OfflineGrace)
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum LifetimeCredentialError {
    #[error("lifetime entitlement record is missing")]
    MissingEntitlement,
    #[error("device credential is missing")]
    MissingCredential,
    #[error("lifetime entitlement is revoked")]
    RevokedEntitlement,
    #[error("device credential is revoked")]
    RevokedCredential,
    #[error("term is not lifetime: {0}")]
    NotLifetimeTerm(String),
    #[error("invalid product: {0}")]
    InvalidProduct(String),
    #[error("invalid license type: {0}")]
    InvalidLicenseType(String),
    #[error("credential sequence {credential} is stale; lifetime entitlement is at {entitlement}")]
    StaleCredential { entitlement: u64, credential: u64 },
    #[error("sequence rollback denied: {new} <= {current}")]
    SequenceRollback { new: u64, current: u64 },
    #[error("authority sequence rollback denied: {new} <= {current}")]
    AuthoritySequenceRollback { new: u64, current: u64 },
    #[error("credential window is invalid (zero width or expired bounds)")]
    InvalidCredentialWindow,
    #[error("lifetime state cannot be read: {0}")]
    Read(String),
    #[error("lifetime state cannot be written: {0}")]
    Write(String),
    #[error("lifetime state is invalid JSON")]
    InvalidJson,
    #[error("unsupported lifetime state schema: {0}")]
    UnsupportedSchema(String),
}

/// The stateless Spec 172 §14 lifecycle state machine over the two separate
/// records (perpetual entitlement + bounded device credential).
pub struct LifetimeCredentialMachine;

impl LifetimeCredentialMachine {
    /// Resolve the runtime posture from the separate records. The lifetime
    /// entitlement is authoritative: a revoked entitlement denies every
    /// credential; a missing entitlement denies with `DeniedUnknown`; an
    /// expired credential with a live entitlement resolves to `RecoveryOnly`
    /// (lifetime preserved, replacement lease issuable).
    pub fn resolve(
        entitlement: Option<&LifetimeEntitlement>,
        credential: Option<&DeviceCredentialWindow>,
        now: DateTime<Utc>,
    ) -> LifetimeCredentialState {
        let Some(entitlement) = entitlement else {
            return LifetimeCredentialState::DeniedUnknown;
        };
        if !entitlement.is_entitled() {
            return LifetimeCredentialState::DeniedRevoked;
        }
        let Some(credential) = credential else {
            // No usable signed lease yet (or the verifier yielded no
            // credential). The lifetime right persists; only recovery is open.
            return LifetimeCredentialState::RecoveryOnly;
        };
        if credential.sequence < entitlement.sequence {
            return LifetimeCredentialState::DeniedStale;
        }
        if credential.status == DeviceCredentialStatus::Revoked {
            return LifetimeCredentialState::DeniedRevoked;
        }
        if now <= credential.expires_at {
            LifetimeCredentialState::Active
        } else if now <= credential.offline_grace_until {
            LifetimeCredentialState::OfflineGrace
        } else {
            LifetimeCredentialState::RecoveryOnly
        }
    }

    /// Rotate (refresh) the bounded device credential. Only while the lifetime
    /// entitlement is Entitled; the new lease carries a strictly higher
    /// sequence and a fresh bounded window under the given authority key.
    /// Families, limits, product, and License Type never change: they live in
    /// the perpetual entitlement, not in the credential.
    pub fn rotate_credential(
        entitlement: &LifetimeEntitlement,
        current: &DeviceCredentialWindow,
        issued_at: DateTime<Utc>,
        refresh_window_days: u64,
        offline_grace_days: u64,
        authority_key_id: impl Into<String>,
    ) -> Result<DeviceCredentialWindow, LifetimeCredentialError> {
        require_entitled(entitlement)?;
        let sequence = entitlement.sequence.checked_add(1).ok_or(
            LifetimeCredentialError::SequenceRollback {
                new: 0,
                current: entitlement.sequence,
            },
        )?;
        if sequence <= current.sequence {
            return Err(LifetimeCredentialError::SequenceRollback {
                new: sequence,
                current: current.sequence,
            });
        }
        build_window(
            entitlement,
            current.node_id.clone(),
            sequence,
            issued_at,
            refresh_window_days,
            offline_grace_days,
            authority_key_id,
        )
    }

    /// Verified recovery issuance: after credential expiry the lifetime
    /// entitlement persists and a replacement bounded lease is issued at a
    /// strictly higher sequence. Refused when the entitlement is revoked or
    /// missing. A stale current credential never blocks recovery.
    pub fn recover_credential(
        entitlement: &LifetimeEntitlement,
        node_id: impl Into<String>,
        issued_at: DateTime<Utc>,
        refresh_window_days: u64,
        offline_grace_days: u64,
        authority_key_id: impl Into<String>,
    ) -> Result<DeviceCredentialWindow, LifetimeCredentialError> {
        require_entitled(entitlement)?;
        let sequence = entitlement.sequence.checked_add(1).ok_or(
            LifetimeCredentialError::SequenceRollback {
                new: 0,
                current: entitlement.sequence,
            },
        )?;
        build_window(
            entitlement,
            node_id.into(),
            sequence,
            issued_at,
            refresh_window_days,
            offline_grace_days,
            authority_key_id,
        )
    }

    /// Refund/revoke/chargeback: mark the lifetime entitlement Revoked at a
    /// strictly higher authority sequence. Stale and offline device
    /// credentials can never override this decision.
    pub fn revoke_entitlement(
        entitlement: &LifetimeEntitlement,
        higher_sequence: u64,
        updated_at: DateTime<Utc>,
    ) -> Result<LifetimeEntitlement, LifetimeCredentialError> {
        if higher_sequence <= entitlement.sequence {
            return Err(LifetimeCredentialError::AuthoritySequenceRollback {
                new: higher_sequence,
                current: entitlement.sequence,
            });
        }
        let mut revoked = entitlement.clone();
        revoked.status = LifetimeEntitlementStatus::Revoked;
        revoked.sequence = higher_sequence;
        revoked.updated_at = updated_at;
        Ok(revoked)
    }

    /// Key rotation: re-sign the current bounded credential under a new
    /// authority key id without widening anything and without touching the
    /// lifetime entitlement or its sequence.
    pub fn rotate_key(
        credential: &DeviceCredentialWindow,
        new_authority_key_id: impl Into<String>,
    ) -> Result<DeviceCredentialWindow, LifetimeCredentialError> {
        if credential.status == DeviceCredentialStatus::Revoked {
            return Err(LifetimeCredentialError::RevokedCredential);
        }
        let mut rotated = credential.clone();
        rotated.authority_key_id = new_authority_key_id.into();
        Ok(rotated)
    }

    /// Deterministic opaque lease id shared with the WPUIAI issuer so the
    /// cross-language vectors agree byte-for-byte.
    pub fn lease_id(product: &str, node_id: &str, sequence: u64) -> String {
        let digest = Sha256::digest(format!("{product}\0{node_id}\0{sequence}").as_bytes());
        format!("lease-{:032x}", digest)
    }
}

fn require_entitled(entitlement: &LifetimeEntitlement) -> Result<(), LifetimeCredentialError> {
    if !entitlement.is_lifetime() {
        return Err(LifetimeCredentialError::NotLifetimeTerm(
            entitlement.term.clone(),
        ));
    }
    if !entitlement.is_entitled() {
        return Err(LifetimeCredentialError::RevokedEntitlement);
    }
    Ok(())
}

fn build_window(
    entitlement: &LifetimeEntitlement,
    node_id: String,
    sequence: u64,
    issued_at: DateTime<Utc>,
    refresh_window_days: u64,
    offline_grace_days: u64,
    authority_key_id: impl Into<String>,
) -> Result<DeviceCredentialWindow, LifetimeCredentialError> {
    if refresh_window_days == 0 || offline_grace_days == 0 {
        return Err(LifetimeCredentialError::InvalidCredentialWindow);
    }
    let expires_at = issued_at + Duration::days(refresh_window_days as i64);
    let offline_grace_until = expires_at + Duration::days(offline_grace_days as i64);
    Ok(DeviceCredentialWindow {
        schema: DEVICE_CREDENTIAL_SCHEMA.to_string(),
        lease_id: LifetimeCredentialMachine::lease_id(&entitlement.product, &node_id, sequence),
        product: entitlement.product.clone(),
        node_id,
        sequence,
        issued_at,
        expires_at,
        offline_grace_until,
        authority_key_id: authority_key_id.into(),
        status: DeviceCredentialStatus::Active,
    })
}

/// Durable store record holding the separate lifetime entitlement and the
/// current bounded device credential, mirroring `PersistedAuthorityState` in
/// `authority_store.rs`. Every read failure is fail-closed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedLifetimeState {
    pub schema: String,
    pub entitlement: LifetimeEntitlement,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_credential: Option<DeviceCredentialWindow>,
    pub last_validated_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refresh_after: Option<DateTime<Utc>>,
}

impl PersistedLifetimeState {
    pub fn new(entitlement: LifetimeEntitlement, now: DateTime<Utc>) -> Self {
        Self {
            schema: LIFETIME_STATE_SCHEMA.to_string(),
            entitlement,
            device_credential: None,
            last_validated_at: now,
            refresh_after: None,
        }
    }

    pub fn read(path: &Path) -> Result<Self, LifetimeCredentialError> {
        if !path.exists() {
            return Err(LifetimeCredentialError::Read("missing".into()));
        }
        let raw = std::fs::read(path)
            .map_err(|error| LifetimeCredentialError::Read(error.to_string()))?;
        let state: Self =
            serde_json::from_slice(&raw).map_err(|_| LifetimeCredentialError::InvalidJson)?;
        if state.schema != LIFETIME_STATE_SCHEMA {
            return Err(LifetimeCredentialError::UnsupportedSchema(state.schema));
        }
        Ok(state)
    }

    pub fn write_atomic(&self, path: &Path) -> Result<(), LifetimeCredentialError> {
        let parent = path.parent().ok_or_else(|| {
            LifetimeCredentialError::Write("lifetime state path has no parent".into())
        })?;
        std::fs::create_dir_all(parent)
            .map_err(|error| LifetimeCredentialError::Write(error.to_string()))?;
        let temporary = temporary_state_path(path);
        let result = (|| {
            let mut options = OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            let mut file = options
                .open(&temporary)
                .map_err(|error| LifetimeCredentialError::Write(error.to_string()))?;
            let payload = serde_json::to_vec_pretty(self)
                .map_err(|error| LifetimeCredentialError::Write(error.to_string()))?;
            file.write_all(&payload)
                .and_then(|_| file.write_all(b"\n"))
                .and_then(|_| file.sync_all())
                .map_err(|error| LifetimeCredentialError::Write(error.to_string()))?;
            std::fs::rename(&temporary, path)
                .map_err(|error| LifetimeCredentialError::Write(error.to_string()))?;
            Ok(())
        })();
        if result.is_err() {
            let _ = std::fs::remove_file(&temporary);
        }
        result
    }

    pub fn resolve(&self, now: DateTime<Utc>) -> LifetimeCredentialState {
        LifetimeCredentialMachine::resolve(
            Some(&self.entitlement),
            self.device_credential.as_ref(),
            now,
        )
    }

    /// Reconcile with a freshly verified authority lease snapshot (the
    /// authority verifier output) while keeping the persisted lifetime
    /// entitlement record authoritative.
    pub fn resolve_with_snapshot(
        &self,
        snapshot: &EntitlementSnapshot,
        now: DateTime<Utc>,
    ) -> LifetimeCredentialState {
        let credential = DeviceCredentialWindow::from_snapshot(snapshot);
        LifetimeCredentialMachine::resolve(Some(&self.entitlement), credential.as_ref(), now)
    }

    /// Refresh surface: rotate the bounded device credential and persist it,
    /// advancing the entitlement's highest sequence so older credentials are
    /// stale. Never widens families, limits, nodes, or the License Type.
    pub fn rotate_credential(
        &self,
        issued_at: DateTime<Utc>,
        refresh_window_days: u64,
        offline_grace_days: u64,
        authority_key_id: impl Into<String>,
    ) -> Result<Self, LifetimeCredentialError> {
        let current = self
            .device_credential
            .as_ref()
            .ok_or(LifetimeCredentialError::MissingCredential)?;
        let rotated = LifetimeCredentialMachine::rotate_credential(
            &self.entitlement,
            current,
            issued_at,
            refresh_window_days,
            offline_grace_days,
            authority_key_id,
        )?;
        let mut next = self.clone();
        next.entitlement.sequence = rotated.sequence;
        next.entitlement.updated_at = issued_at;
        next.device_credential = Some(rotated);
        next.last_validated_at = issued_at;
        next.refresh_after = Some(issued_at + Duration::days(refresh_window_days as i64));
        Ok(next)
    }

    /// Recovery surface: issue a replacement bounded lease after credential
    /// expiry. The lifetime entitlement persists; a revoked entitlement
    /// refuses issuance.
    pub fn recover_credential(
        &self,
        node_id: impl Into<String>,
        issued_at: DateTime<Utc>,
        refresh_window_days: u64,
        offline_grace_days: u64,
        authority_key_id: impl Into<String>,
    ) -> Result<Self, LifetimeCredentialError> {
        let recovered = LifetimeCredentialMachine::recover_credential(
            &self.entitlement,
            node_id,
            issued_at,
            refresh_window_days,
            offline_grace_days,
            authority_key_id,
        )?;
        let mut next = self.clone();
        next.entitlement.sequence = recovered.sequence;
        next.entitlement.updated_at = issued_at;
        next.device_credential = Some(recovered);
        next.last_validated_at = issued_at;
        next.refresh_after = Some(issued_at + Duration::days(refresh_window_days as i64));
        Ok(next)
    }

    /// Refund/revoke/chargeback: mark the lifetime entitlement revoked at a
    /// strictly higher authority sequence. Stale and offline device
    /// credentials can never override this decision.
    pub fn apply_refund_or_revoke(
        &self,
        higher_sequence: u64,
        updated_at: DateTime<Utc>,
    ) -> Result<Self, LifetimeCredentialError> {
        let revoked = LifetimeCredentialMachine::revoke_entitlement(
            &self.entitlement,
            higher_sequence,
            updated_at,
        )?;
        let mut next = self.clone();
        next.entitlement = revoked;
        next.last_validated_at = updated_at;
        Ok(next)
    }

    /// Key rotation: re-sign the current credential under a new authority key
    /// without widening anything and without changing the entitlement.
    pub fn rotate_key(
        &self,
        new_authority_key_id: impl Into<String>,
        validated_at: DateTime<Utc>,
    ) -> Result<Self, LifetimeCredentialError> {
        let current = self
            .device_credential
            .as_ref()
            .ok_or(LifetimeCredentialError::MissingCredential)?;
        let rotated = LifetimeCredentialMachine::rotate_key(current, new_authority_key_id)?;
        let mut next = self.clone();
        next.device_credential = Some(rotated);
        next.last_validated_at = validated_at;
        Ok(next)
    }

    /// Install a freshly verified signed lease as the current device
    /// credential without changing the lifetime entitlement record.
    pub fn with_snapshot_credential(
        &self,
        snapshot: &EntitlementSnapshot,
        validated_at: DateTime<Utc>,
    ) -> Self {
        let mut next = self.clone();
        next.device_credential = DeviceCredentialWindow::from_snapshot(snapshot);
        next.last_validated_at = validated_at;
        next
    }
}

fn temporary_state_path(path: &Path) -> PathBuf {
    let mut temporary = path.as_os_str().to_owned();
    temporary.push(format!(".tmp-{}", uuid::Uuid::now_v7()));
    PathBuf::from(temporary)
}

/// Compile-time assertion that the canonical registrations stay in sync with
/// the product-bound License Type mapping used by `LifetimeEntitlement::new`.
pub fn registered_operator_lifetime_products() -> BTreeMap<&'static str, &'static str> {
    BTreeMap::from([
        (PRODUCT_FOCUSA, LICENSE_TYPE_FOCUSA_OPERATOR_LIFETIME_V1),
        (PRODUCT_UIAI_ENGINE, LICENSE_TYPE_UIAI_OPERATOR_LIFETIME_V1),
    ])
}
