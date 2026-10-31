//! Spec133 §13.1 runner material provisioning (daemon side).
//!
//! The runner authenticates every command with an HMAC over a shared key and
//! rejects replays through its own nonce ledger. Both inputs are files the
//! runner reads at startup (`--key_file`, `--nonce_ledger`), so the daemon must
//! create them with owner-only permissions before it can launch a runner.
//! No nonce issuer is required: the daemon's `consume_runner_nonce` is the
//! replay guard, and the runner appends to its own ledger on accept.

use anyhow::{Context, bail};
use std::path::{Path, PathBuf};

use super::RunnerIdentity;

/// Length of the shared runner HMAC key.
pub const RUNNER_KEY_BYTES: usize = 32;
/// POSIX socket paths are limited to 108 bytes including the terminator.
const MAX_SOCKET_PATH_BYTES: usize = 104;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunnerPaths {
    pub root: PathBuf,
    pub key_file: PathBuf,
    pub secret_dir: PathBuf,
    pub nonce_ledger: PathBuf,
    pub socket: PathBuf,
}

fn valid_segment(label: &str, value: &str) -> anyhow::Result<()> {
    let ok = !value.is_empty()
        && value.len() <= 64
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '@'));
    if !ok {
        bail!("invalid {label} for runner path: {value:?}");
    }
    Ok(())
}

impl RunnerPaths {
    /// Deterministic material location for one runner identity.
    pub fn resolve(
        state_root: &Path,
        principal_id: &str,
        socket_scope: &str,
    ) -> anyhow::Result<Self> {
        valid_segment("principal_id", principal_id)?;
        valid_segment("socket_scope", socket_scope)?;
        let root = state_root
            .join("runners")
            .join(principal_id)
            .join(socket_scope);
        let paths = Self {
            key_file: root.join("runner.key"),
            secret_dir: root.join("secrets"),
            nonce_ledger: root.join("nonces.log"),
            socket: root.join("runner.sock"),
            root,
        };
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStrExt;
            let bytes = paths.socket.as_os_str().as_bytes().len();
            if bytes > MAX_SOCKET_PATH_BYTES {
                bail!(
                    "runner socket path is {bytes} bytes, exceeds {MAX_SOCKET_PATH_BYTES}: {}",
                    paths.socket.display()
                );
            }
        }
        Ok(paths)
    }

    /// Create directories, key and nonce ledger if absent; return the key bytes.
    /// Idempotent: existing material is reused, never rotated implicitly.
    pub fn ensure_material(&self, owner_uid: u32) -> anyhow::Result<Vec<u8>> {
        std::fs::create_dir_all(&self.root)
            .with_context(|| format!("create runner root {}", self.root.display()))?;
        std::fs::create_dir_all(&self.secret_dir)
            .with_context(|| format!("create secret dir {}", self.secret_dir.display()))?;
        restrict(&self.root, 0o700)?;
        restrict(&self.secret_dir, 0o700)?;

        let key = match std::fs::read(&self.key_file) {
            Ok(bytes) => {
                if bytes.len() != RUNNER_KEY_BYTES {
                    bail!(
                        "existing runner key has {} bytes, expected {RUNNER_KEY_BYTES}",
                        bytes.len()
                    );
                }
                bytes
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let bytes = generate_key();
                write_private(&self.key_file, &bytes)?;
                bytes
            }
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("read runner key {}", self.key_file.display()));
            }
        };

        if !self.nonce_ledger.exists() {
            write_private(&self.nonce_ledger, b"")?;
        }
        verify_owner(&self.key_file, owner_uid)?;
        Ok(key)
    }

    pub fn identity(os_user: &str, principal_id: &str, socket_scope: &str) -> RunnerIdentity {
        RunnerIdentity {
            principal_id: principal_id.to_string(),
            os_user: os_user.to_string(),
            socket_scope: socket_scope.to_string(),
        }
    }
}

fn generate_key() -> Vec<u8> {
    // Two v4 UUIDs give 32 uniform bytes without adding a dependency.
    let mut bytes = Vec::with_capacity(RUNNER_KEY_BYTES);
    bytes.extend_from_slice(uuid::Uuid::new_v4().as_bytes());
    bytes.extend_from_slice(uuid::Uuid::new_v4().as_bytes());
    bytes
}

fn write_private(path: &Path, bytes: &[u8]) -> anyhow::Result<()> {
    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(path)
            .with_context(|| format!("create private file {}", path.display()))?;
        file.write_all(bytes)?;
        file.sync_all()?;
        return Ok(());
    }
    #[cfg(not(unix))]
    {
        std::fs::write(path, bytes)
            .with_context(|| format!("create private file {}", path.display()))
    }
}

fn restrict(path: &Path, mode: u32) -> anyhow::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))
            .with_context(|| format!("set mode {mode:o} on {}", path.display()))?;
    }
    #[cfg(not(unix))]
    {
        let _ = (path, mode);
    }
    Ok(())
}

fn verify_owner(path: &Path, expected_uid: u32) -> anyhow::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let meta = std::fs::metadata(path)
            .with_context(|| format!("stat {}", path.display()))?;
        if meta.uid() != expected_uid {
            bail!(
                "runner key {} is owned by uid {}, expected {expected_uid}",
                path.display(),
                meta.uid()
            );
        }
    }
    #[cfg(not(unix))]
    {
        let _ = (path, expected_uid);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("focusa-runner-prov-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn material_is_idempotent_and_private() {
        let root = scratch();
        let paths = RunnerPaths::resolve(&root, "principal-1", "scope-1").unwrap();
        let uid = current_uid();
        let first = paths.ensure_material(uid).unwrap();
        let second = paths.ensure_material(uid).unwrap();
        assert_eq!(first.len(), RUNNER_KEY_BYTES);
        assert_eq!(first, second, "key must not rotate on re-provision");
        assert!(paths.nonce_ledger.exists());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&paths.key_file).unwrap().permissions().mode();
            assert_eq!(mode & 0o777, 0o600);
            let dir_mode = std::fs::metadata(&paths.secret_dir).unwrap().permissions().mode();
            assert_eq!(dir_mode & 0o777, 0o700);
        }
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn rejects_path_traversal_and_overlong_socket() {
        let root = scratch();
        assert!(RunnerPaths::resolve(&root, "../escape", "scope").is_err());
        assert!(RunnerPaths::resolve(&root, "principal", "scope/../x").is_err());
        let deep = format!("{}/{}", root.display(), "a".repeat(80));
        assert!(RunnerPaths::resolve(Path::new(&deep), "p", "s").is_err());
        std::fs::remove_dir_all(&root).ok();
    }

    #[cfg(unix)]
    fn current_uid() -> u32 {
        use std::os::unix::fs::MetadataExt;
        std::fs::metadata(std::env::temp_dir()).unwrap().uid()
    }
    #[cfg(not(unix))]
    fn current_uid() -> u32 {
        0
    }
}
