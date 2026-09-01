use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::Path,
};

use uuid::Uuid;

use super::StreamStorageError;

pub(super) fn create_secure_root(root: &Path) -> Result<(), StreamStorageError> {
    reject_symlink_ancestors(root)?;
    if root.exists() {
        reject_symlink(root)?;
        if !root.is_dir() {
            return Err(StreamStorageError::UnsafePath(root.display().to_string()));
        }
    } else {
        fs::create_dir_all(root).map_err(anyhow::Error::from)?;
    }
    secure_directory_permissions(root)?;
    Ok(())
}

pub(super) fn create_secure_descendants(
    root: &Path,
    target: &Path,
) -> Result<(), StreamStorageError> {
    let relative = target
        .strip_prefix(root)
        .map_err(|_| StreamStorageError::PathOutsideRoot)?;
    let mut current = root.to_path_buf();
    for component in relative.components() {
        current.push(component);
        if current.exists() {
            reject_symlink(&current)?;
            if !current.is_dir() {
                return Err(StreamStorageError::UnsafePath(
                    current.display().to_string(),
                ));
            }
        } else {
            fs::create_dir(&current).map_err(anyhow::Error::from)?;
        }
        secure_directory_permissions(&current)?;
    }
    Ok(())
}

fn reject_symlink_ancestors(path: &Path) -> Result<(), StreamStorageError> {
    for ancestor in path.ancestors() {
        match fs::symlink_metadata(ancestor) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(StreamStorageError::UnsafePath(
                    ancestor.display().to_string(),
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(anyhow::Error::from(error).into()),
        }
    }
    Ok(())
}

pub(super) fn reject_symlink(path: &Path) -> Result<(), StreamStorageError> {
    let metadata = fs::symlink_metadata(path).map_err(anyhow::Error::from)?;
    if metadata.file_type().is_symlink() {
        return Err(StreamStorageError::UnsafePath(path.display().to_string()));
    }
    Ok(())
}

pub(super) fn atomic_publish(
    directory: &Path,
    final_path: &Path,
    bytes: &[u8],
) -> Result<(), StreamStorageError> {
    let temporary = directory.join(format!(".chunk-{}.tmp", Uuid::now_v7()));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    secure_open_options(&mut options);
    let mut file = options.open(&temporary).map_err(anyhow::Error::from)?;
    file.write_all(bytes).map_err(anyhow::Error::from)?;
    file.sync_all().map_err(anyhow::Error::from)?;
    drop(file);
    crate::durable_fs::atomic_replace(&temporary, final_path).map_err(anyhow::Error::from)?;
    crate::durable_fs::sync_directory(directory).map_err(anyhow::Error::from)?;
    Ok(())
}

pub(super) fn secure_read(path: &Path, expected_bytes: u64) -> Result<Vec<u8>, StreamStorageError> {
    reject_symlink(path)?;
    if expected_bytes > 64 * 1024 * 1024 {
        return Err(StreamStorageError::ChecksumMismatch);
    }
    let mut options = OpenOptions::new();
    options.read(true);
    secure_open_options(&mut options);
    let file = options.open(path).map_err(anyhow::Error::from)?;
    if file.metadata().map_err(anyhow::Error::from)?.len() != expected_bytes {
        return Err(StreamStorageError::ChecksumMismatch);
    }
    let mut bytes = Vec::with_capacity(expected_bytes as usize);
    file.take(expected_bytes + 1)
        .read_to_end(&mut bytes)
        .map_err(anyhow::Error::from)?;
    if bytes.len() as u64 != expected_bytes {
        return Err(StreamStorageError::ChecksumMismatch);
    }
    Ok(bytes)
}

#[cfg(unix)]
fn secure_open_options(options: &mut OpenOptions) {
    use std::os::unix::fs::OpenOptionsExt;
    options.mode(0o600).custom_flags(no_follow_flag());
}

#[cfg(target_os = "linux")]
const fn no_follow_flag() -> i32 {
    0o400000
}

#[cfg(all(unix, not(target_os = "linux")))]
const fn no_follow_flag() -> i32 {
    0x100
}

#[cfg(windows)]
fn secure_open_options(options: &mut OpenOptions) {
    use std::os::windows::fs::OpenOptionsExt;
    const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
    options.custom_flags(FILE_FLAG_OPEN_REPARSE_POINT);
}

#[cfg(unix)]
fn secure_directory_permissions(path: &Path) -> Result<(), StreamStorageError> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).map_err(anyhow::Error::from)?;
    Ok(())
}

#[cfg(windows)]
fn secure_directory_permissions(_path: &Path) -> Result<(), StreamStorageError> {
    Ok(())
}

pub(super) fn relative_ref(root: &Path, path: &Path) -> Result<String, StreamStorageError> {
    path.strip_prefix(root)
        .map(|relative| relative.to_string_lossy().into_owned())
        .map_err(|_| StreamStorageError::PathOutsideRoot)
}
