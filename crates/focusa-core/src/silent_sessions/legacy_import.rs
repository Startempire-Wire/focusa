use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use super::{
    SilentSessionId, SilentSessionRunId, StreamStorageError,
    secure_fs::{
        atomic_publish, create_secure_descendants, create_secure_root, relative_ref, secure_read,
    },
};

const REGISTRY_LIMIT: u64 = 1024 * 1024;
const LOG_LIMIT: u64 = 64 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ImportedLegacySession {
    pub session_id: SilentSessionId,
    pub run_id: SilentSessionRunId,
    pub aliases: Vec<String>,
    pub copied_log_ref: Option<String>,
    pub source_registry_hash: String,
    pub legacy_unverified: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct LegacyImportMap {
    records: BTreeMap<String, LegacyIdentity>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct LegacyIdentity {
    session_id: SilentSessionId,
    run_id: SilentSessionRunId,
}

pub struct LegacySilentSessionImporter {
    destination_root: PathBuf,
    allowed_log_roots: Vec<PathBuf>,
    expected_uid: u32,
}

impl LegacySilentSessionImporter {
    pub fn new(
        destination_root: impl Into<PathBuf>,
        allowed_log_roots: Vec<PathBuf>,
        expected_uid: u32,
    ) -> Result<Self, StreamStorageError> {
        let destination_root = destination_root.into();
        if !destination_root.is_absolute()
            || allowed_log_roots.iter().any(|root| !root.is_absolute())
        {
            return Err(StreamStorageError::RootNotAbsolute);
        }
        create_secure_root(&destination_root)?;
        Ok(Self {
            destination_root,
            allowed_log_roots,
            expected_uid,
        })
    }

    /// Imports metadata and logs only. The legacy `command` field is
    /// deliberately neither returned nor passed to any execution surface.
    pub fn import_registry(
        &self,
        registry_path: &Path,
    ) -> Result<Vec<ImportedLegacySession>, StreamStorageError> {
        validate_owned_regular_file(registry_path, self.expected_uid, REGISTRY_LIMIT)?;
        let registry_bytes = secure_read(
            registry_path,
            fs::metadata(registry_path)
                .map_err(anyhow::Error::from)?
                .len(),
        )?;
        let registry_hash = hex::encode(Sha256::digest(&registry_bytes));
        let value: Value = serde_json::from_slice(&registry_bytes).map_err(anyhow::Error::from)?;
        let sessions = value
            .get("sessions")
            .unwrap_or(&value)
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("legacy registry sessions must be an object"))?;
        let mut identity_map = self.read_identity_map()?;
        let mut imported = Vec::with_capacity(sessions.len());
        for (alias, raw) in sessions {
            validate_alias(alias)?;
            let identity = *identity_map
                .records
                .entry(alias.clone())
                .or_insert_with(|| LegacyIdentity {
                    session_id: SilentSessionId::new(),
                    run_id: SilentSessionRunId::new(),
                });
            let copied_log_ref = raw
                .get("log_path")
                .and_then(Value::as_str)
                .map(|path| self.copy_log(identity, Path::new(path)))
                .transpose()?;
            imported.push(ImportedLegacySession {
                session_id: identity.session_id,
                run_id: identity.run_id,
                aliases: vec![alias.clone()],
                copied_log_ref,
                source_registry_hash: registry_hash.clone(),
                legacy_unverified: true,
            });
        }
        self.write_identity_map(&identity_map)?;
        Ok(imported)
    }

    fn copy_log(
        &self,
        identity: LegacyIdentity,
        source: &Path,
    ) -> Result<String, StreamStorageError> {
        validate_owned_regular_file(source, self.expected_uid, LOG_LIMIT)?;
        let canonical = source.canonicalize().map_err(anyhow::Error::from)?;
        let allowed = self.allowed_log_roots.iter().any(|root| {
            root.canonicalize()
                .is_ok_and(|allowed| canonical.starts_with(allowed))
        });
        if !allowed {
            return Err(StreamStorageError::PathOutsideRoot);
        }
        let bytes = secure_read(
            &canonical,
            fs::metadata(&canonical).map_err(anyhow::Error::from)?.len(),
        )?;
        let directory = self
            .destination_root
            .join(identity.session_id.to_string())
            .join(identity.run_id.to_string())
            .join("legacy");
        create_secure_descendants(&self.destination_root, &directory)?;
        let destination = directory.join("legacy.log");
        atomic_publish(&directory, &destination, &bytes)?;
        relative_ref(&self.destination_root, &destination)
    }

    fn map_path(&self) -> PathBuf {
        self.destination_root.join("legacy-import-map.json")
    }

    fn read_identity_map(&self) -> Result<LegacyImportMap, StreamStorageError> {
        let path = self.map_path();
        if !path.exists() {
            return Ok(LegacyImportMap::default());
        }
        let metadata = fs::metadata(&path).map_err(anyhow::Error::from)?;
        let bytes = secure_read(&path, metadata.len())?;
        serde_json::from_slice(&bytes)
            .map_err(anyhow::Error::from)
            .map_err(Into::into)
    }

    fn write_identity_map(&self, map: &LegacyImportMap) -> Result<(), StreamStorageError> {
        let bytes = serde_json::to_vec_pretty(map).map_err(anyhow::Error::from)?;
        atomic_publish(&self.destination_root, &self.map_path(), &bytes)
    }
}

fn validate_alias(alias: &str) -> Result<(), StreamStorageError> {
    if alias.is_empty()
        || alias.len() > 100
        || !alias
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._:-".contains(&byte))
    {
        return Err(anyhow::anyhow!("unsafe legacy alias").into());
    }
    Ok(())
}

fn validate_owned_regular_file(
    path: &Path,
    expected_uid: u32,
    max_bytes: u64,
) -> Result<(), StreamStorageError> {
    if !path.is_absolute() {
        return Err(StreamStorageError::PathOutsideRoot);
    }
    let metadata = fs::symlink_metadata(path).map_err(anyhow::Error::from)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() > max_bytes {
        return Err(StreamStorageError::UnsafePath(path.display().to_string()));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if metadata.uid() != expected_uid {
            return Err(StreamStorageError::UnsafePath(path.display().to_string()));
        }
    }
    #[cfg(not(unix))]
    let _ = expected_uid;
    Ok(())
}
