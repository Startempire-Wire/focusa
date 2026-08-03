use crate::authority_client::SensitiveCredential;
use std::{collections::BTreeMap, sync::Mutex};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CredentialHandle {
    pub service: String,
    pub account: String,
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
    fn credential_handle_contains_identity_but_never_secret() {
        let handle = CredentialHandle::for_node("focusa", "node-1").unwrap();
        assert_eq!(handle.service, "focusa.focusa.license-authority");
        assert_eq!(handle.account, "node:node-1:refresh");
        assert!(!format!("{handle:?}").contains("secret"));
    }

    #[test]
    fn protected_store_round_trip_redacts_and_deletes() {
        let store = InMemoryCredentialStore::default();
        let handle = CredentialHandle::for_node("focusa", "node-1").unwrap();
        let credential = SensitiveCredential::new("refresh-secret".into()).unwrap();
        store.put(&handle, &credential).unwrap();
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
