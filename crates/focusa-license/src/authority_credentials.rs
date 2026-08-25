use crate::authority_client::SensitiveCredential;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CredentialHandle {
    pub service: String,
    pub account: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeIdentity {
    pub schema: String,
    pub node_id: String,
    pub product: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtectedStoreBackend {
    MacOsKeychain,
    LinuxSecretService,
    WindowsCredentialManager,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialRotationReceipt {
    pub schema: String,
    pub service: String,
    pub account: String,
    pub backend: ProtectedStoreBackend,
    pub rotated_at: DateTime<Utc>,
    pub secret_persisted_in_receipt: bool,
}

impl CredentialHandle {
    pub fn for_node(product: &str, node_id: &str) -> Result<Self, CredentialStoreError> {
        if product.trim().is_empty() {
            return Err(CredentialStoreError::MissingIdentity("product"));
        }
        if node_id.trim().is_empty() {
            return Err(CredentialStoreError::MissingIdentity("node_id"));
        }
        Ok(Self {
            service: format!("focusa.{product}.license-authority"),
            account: format!("node:{node_id}:refresh"),
        })
    }

    /// Registration-scoped handle for the expiring activation poll
    /// credential. The poll credential is never persisted raw in snapshots;
    /// resume re-supplies it from the protected store under this handle
    /// (Spec 152E §10: poll credentials are registration-specific, secret,
    /// expiring, and stored only as hashes server-side).
    pub fn for_registration(registration_id: &str) -> Result<Self, CredentialStoreError> {
        if registration_id.trim().is_empty() {
            return Err(CredentialStoreError::MissingIdentity("registration_id"));
        }
        Ok(Self {
            service: "focusa.activation.poll-credential".into(),
            account: format!("registration:{registration_id}"),
        })
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CredentialStoreError {
    #[error("credential identity is missing: {0}")]
    MissingIdentity(&'static str),
    #[error("protected credential store is unavailable")]
    StoreUnavailable,
    #[error("protected credential is missing")]
    CredentialMissing,
    #[error("protected credential write failed")]
    WriteFailed,
    #[error("protected credential delete failed")]
    DeleteFailed,
    #[error("node identity persistence failed")]
    NodeIdentityPersistenceFailed,
    #[error("node identity is invalid")]
    NodeIdentityInvalid,
}

pub fn protected_store_backend_for_os(
    os: &str,
) -> Result<ProtectedStoreBackend, CredentialStoreError> {
    match os {
        "macos" => Ok(ProtectedStoreBackend::MacOsKeychain),
        "linux" => Ok(ProtectedStoreBackend::LinuxSecretService),
        "windows" => Ok(ProtectedStoreBackend::WindowsCredentialManager),
        _ => Err(CredentialStoreError::StoreUnavailable),
    }
}

pub fn native_protected_store_backend() -> ProtectedStoreBackend {
    protected_store_backend_for_os(std::env::consts::OS)
        .unwrap_or(ProtectedStoreBackend::LinuxSecretService)
}

pub fn load_or_create_node_identity(
    config_dir: &Path,
    product: &str,
) -> Result<NodeIdentity, CredentialStoreError> {
    if product.trim().is_empty() {
        return Err(CredentialStoreError::MissingIdentity("product"));
    }
    let path = config_dir.join("node-identity.json");
    if path.exists() {
        let bytes =
            fs::read(&path).map_err(|_| CredentialStoreError::NodeIdentityPersistenceFailed)?;
        let identity: NodeIdentity = serde_json::from_slice(&bytes)
            .map_err(|_| CredentialStoreError::NodeIdentityInvalid)?;
        // Tolerate legacy identities that stored the node id with a `node-`
        // prefix (#342 field evidence): normalize instead of failing the whole
        // activation flow for pre-existing installs.
        let normalized_node_id = identity
            .node_id
            .strip_prefix("node-")
            .unwrap_or(&identity.node_id)
            .to_string();
        if identity.schema != "focusa.node_identity.v1"
            || identity.product != product
            || Uuid::parse_str(&normalized_node_id).is_err()
        {
            return Err(CredentialStoreError::NodeIdentityInvalid);
        }
        if normalized_node_id != identity.node_id {
            let mut fixed = identity;
            fixed.node_id = normalized_node_id;
            let bytes = serde_json::to_vec_pretty(&fixed)
                .map_err(|_| CredentialStoreError::NodeIdentityPersistenceFailed)?;
            fs::write(&path, bytes)
                .map_err(|_| CredentialStoreError::NodeIdentityPersistenceFailed)?;
            return Ok(fixed);
        }
        return Ok(identity);
    }
    fs::create_dir_all(config_dir)
        .map_err(|_| CredentialStoreError::NodeIdentityPersistenceFailed)?;
    let identity = NodeIdentity {
        schema: "focusa.node_identity.v1".into(),
        node_id: Uuid::now_v7().to_string(),
        product: product.into(),
        created_at: Utc::now(),
    };
    atomic_private_write(
        &path,
        &serde_json::to_vec_pretty(&identity)
            .map_err(|_| CredentialStoreError::NodeIdentityPersistenceFailed)?,
    )?;
    Ok(identity)
}

fn atomic_private_write(path: &Path, bytes: &[u8]) -> Result<(), CredentialStoreError> {
    let temporary = PathBuf::from(format!("{}.tmp-{}", path.display(), Uuid::now_v7()));
    fs::write(&temporary, bytes)
        .map_err(|_| CredentialStoreError::NodeIdentityPersistenceFailed)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&temporary, fs::Permissions::from_mode(0o600))
            .map_err(|_| CredentialStoreError::NodeIdentityPersistenceFailed)?;
    }
    fs::rename(temporary, path).map_err(|_| CredentialStoreError::NodeIdentityPersistenceFailed)
}

pub trait ProtectedCredentialStore: Send + Sync {
    fn put(
        &self,
        handle: &CredentialHandle,
        credential: &SensitiveCredential,
    ) -> Result<(), CredentialStoreError>;
    fn get(&self, handle: &CredentialHandle) -> Result<SensitiveCredential, CredentialStoreError>;
    fn delete(&self, handle: &CredentialHandle) -> Result<(), CredentialStoreError>;
}

#[derive(Debug, Default)]
pub struct KeyringCredentialStore;

impl KeyringCredentialStore {
    fn entry(handle: &CredentialHandle) -> Result<keyring::Entry, CredentialStoreError> {
        keyring::Entry::new(&handle.service, &handle.account)
            .map_err(|_| CredentialStoreError::StoreUnavailable)
    }
}

impl ProtectedCredentialStore for KeyringCredentialStore {
    fn put(
        &self,
        handle: &CredentialHandle,
        credential: &SensitiveCredential,
    ) -> Result<(), CredentialStoreError> {
        Self::entry(handle)?
            .set_password(credential.expose_for_protected_store())
            .map_err(|_| CredentialStoreError::WriteFailed)
    }

    fn get(&self, handle: &CredentialHandle) -> Result<SensitiveCredential, CredentialStoreError> {
        let value = Self::entry(handle)?
            .get_password()
            .map_err(|error| match error {
                keyring::Error::NoEntry => CredentialStoreError::CredentialMissing,
                _ => CredentialStoreError::StoreUnavailable,
            })?;
        SensitiveCredential::new(value).map_err(|_| CredentialStoreError::CredentialMissing)
    }

    fn delete(&self, handle: &CredentialHandle) -> Result<(), CredentialStoreError> {
        Self::entry(handle)?
            .delete_credential()
            .map_err(|error| match error {
                keyring::Error::NoEntry => CredentialStoreError::CredentialMissing,
                _ => CredentialStoreError::DeleteFailed,
            })
    }
}

pub fn rotate_refresh_credential(
    store: &dyn ProtectedCredentialStore,
    handle: &CredentialHandle,
    credential: &SensitiveCredential,
    now: DateTime<Utc>,
) -> Result<CredentialRotationReceipt, CredentialStoreError> {
    store.put(handle, credential)?;
    let loaded = store.get(handle)?;
    if loaded.expose_for_protected_store() != credential.expose_for_protected_store() {
        return Err(CredentialStoreError::WriteFailed);
    }
    Ok(CredentialRotationReceipt {
        schema: "focusa.credential_rotation_receipt.v1".into(),
        service: handle.service.clone(),
        account: handle.account.clone(),
        backend: native_protected_store_backend(),
        rotated_at: now,
        secret_persisted_in_receipt: false,
    })
}

#[derive(Debug, Default)]
pub struct InMemoryCredentialStore {
    values: Mutex<BTreeMap<(String, String), String>>,
}

impl ProtectedCredentialStore for InMemoryCredentialStore {
    fn put(
        &self,
        handle: &CredentialHandle,
        credential: &SensitiveCredential,
    ) -> Result<(), CredentialStoreError> {
        self.values
            .lock()
            .map_err(|_| CredentialStoreError::StoreUnavailable)?
            .insert(
                (handle.service.clone(), handle.account.clone()),
                credential.expose_for_protected_store().into(),
            );
        Ok(())
    }

    fn get(&self, handle: &CredentialHandle) -> Result<SensitiveCredential, CredentialStoreError> {
        let value = self
            .values
            .lock()
            .map_err(|_| CredentialStoreError::StoreUnavailable)?
            .get(&(handle.service.clone(), handle.account.clone()))
            .cloned()
            .ok_or(CredentialStoreError::CredentialMissing)?;
        SensitiveCredential::new(value).map_err(|_| CredentialStoreError::CredentialMissing)
    }

    fn delete(&self, handle: &CredentialHandle) -> Result<(), CredentialStoreError> {
        self.values
            .lock()
            .map_err(|_| CredentialStoreError::StoreUnavailable)?
            .remove(&(handle.service.clone(), handle.account.clone()))
            .map(|_| ())
            .ok_or(CredentialStoreError::CredentialMissing)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_platforms_map_to_os_protected_backends() {
        assert_eq!(
            protected_store_backend_for_os("macos").unwrap(),
            ProtectedStoreBackend::MacOsKeychain
        );
        assert_eq!(
            protected_store_backend_for_os("linux").unwrap(),
            ProtectedStoreBackend::LinuxSecretService
        );
        assert_eq!(
            protected_store_backend_for_os("windows").unwrap(),
            ProtectedStoreBackend::WindowsCredentialManager
        );
        assert_eq!(
            protected_store_backend_for_os("other"),
            Err(CredentialStoreError::StoreUnavailable)
        );
    }

    #[test]
    fn credential_handle_contains_identity_but_never_secret() {
        let handle = CredentialHandle::for_node("focusa", "node-1").unwrap();
        assert_eq!(handle.service, "focusa.focusa.license-authority");
        assert_eq!(handle.account, "node:node-1:refresh");
        assert!(!format!("{handle:?}").contains("secret"));
    }

    #[test]
    fn node_identity_is_durable_private_and_fail_closed() {
        let directory = std::env::temp_dir().join(format!("focusa-node-{}", Uuid::now_v7()));
        let first = load_or_create_node_identity(&directory, "focusa").unwrap();
        let second = load_or_create_node_identity(&directory, "focusa").unwrap();
        assert_eq!(first, second);
        assert!(Uuid::parse_str(&first.node_id).is_ok());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(directory.join("node-identity.json"))
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o777,
                0o600
            );
        }
        fs::write(directory.join("node-identity.json"), b"not-json").unwrap();
        assert_eq!(
            load_or_create_node_identity(&directory, "focusa"),
            Err(CredentialStoreError::NodeIdentityInvalid)
        );
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn protected_store_round_trip_redacts_and_deletes() {
        let store = InMemoryCredentialStore::default();
        let handle = CredentialHandle::for_node("focusa", "node-1").unwrap();
        let credential = SensitiveCredential::new("refresh-secret".into()).unwrap();
        let receipt = rotate_refresh_credential(&store, &handle, &credential, Utc::now()).unwrap();
        let receipt_json = serde_json::to_string(&receipt).unwrap();
        assert!(!receipt_json.contains("refresh-secret"));
        assert!(!receipt.secret_persisted_in_receipt);
        let loaded = store.get(&handle).unwrap();
        assert_eq!(loaded.expose_for_protected_store(), "refresh-secret");
        assert_eq!(format!("{loaded:?}"), "SensitiveCredential([REDACTED])");
        store.delete(&handle).unwrap();
        assert_eq!(
            store.get(&handle),
            Err(CredentialStoreError::CredentialMissing)
        );
    }
}
