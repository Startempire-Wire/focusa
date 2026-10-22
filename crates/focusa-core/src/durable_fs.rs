#[cfg(any(not(windows), test))]
use std::fs::{self, File};
use std::io;
use std::path::Path;

/// Atomically publish `replacement` at `destination`, replacing an existing
/// destination on every supported platform.
#[cfg(not(windows))]
pub fn atomic_replace(replacement: &Path, destination: &Path) -> io::Result<()> {
    fs::rename(replacement, destination)
}

/// Publish without clobbering an existing destination. The caller owns cleanup
/// of the temporary source, which may be retained or consumed by the platform.
#[cfg(not(windows))]
pub fn atomic_publish_new(replacement: &Path, destination: &Path) -> io::Result<()> {
    fs::hard_link(replacement, destination)
}

/// Windows `std::fs::rename` does not replace an existing destination. Use the
/// native write-through primitive so durable ledgers retain atomic publication.
#[cfg(windows)]
pub fn atomic_replace(replacement: &Path, destination: &Path) -> io::Result<()> {
    windows_publish(replacement, destination, true)
}

/// Write-through publication without permission to replace an existing file.
#[cfg(windows)]
pub fn atomic_publish_new(replacement: &Path, destination: &Path) -> io::Result<()> {
    windows_publish(replacement, destination, false)
}

#[cfg(windows)]
fn windows_publish(
    replacement: &Path,
    destination: &Path,
    replace_existing: bool,
) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;

    const MOVEFILE_REPLACE_EXISTING: u32 = 0x0000_0001;
    const MOVEFILE_WRITE_THROUGH: u32 = 0x0000_0008;

    #[link(name = "Kernel32")]
    unsafe extern "system" {
        fn MoveFileExW(
            existing_file_name: *const u16,
            new_file_name: *const u16,
            flags: u32,
        ) -> i32;
    }

    let from = replacement
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let to = destination
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let flags = MOVEFILE_WRITE_THROUGH
        | if replace_existing {
            MOVEFILE_REPLACE_EXISTING
        } else {
            0
        };
    // SAFETY: both path buffers are NUL-terminated and remain alive for the
    // duration of the synchronous Win32 call.
    let moved = unsafe { MoveFileExW(from.as_ptr(), to.as_ptr(), flags) };
    if moved == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

/// Persist directory metadata where the platform exposes directory handles.
/// Windows publication uses `MOVEFILE_WRITE_THROUGH`; opening a directory with
/// `File::open` returns access denied and is not a valid durability primitive.
#[cfg(not(windows))]
pub fn sync_directory(path: &Path) -> io::Result<()> {
    File::open(path)?.sync_all()
}

#[cfg(windows)]
pub fn sync_directory(_path: &Path) -> io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atomic_replace_overwrites_and_directory_sync_contract_succeeds() {
        let root = std::env::temp_dir().join(format!("focusa-durable-fs-{}", uuid::Uuid::now_v7()));
        fs::create_dir_all(&root).unwrap();
        let destination = root.join("ledger.jsonl");
        let replacement = root.join("ledger.tmp");
        fs::write(&destination, b"old").unwrap();
        fs::write(&replacement, b"new").unwrap();
        File::options()
            .write(true)
            .open(&replacement)
            .unwrap()
            .sync_all()
            .unwrap();

        atomic_replace(&replacement, &destination).unwrap();
        sync_directory(&root).unwrap();

        assert_eq!(fs::read(&destination).unwrap(), b"new");
        assert!(!replacement.exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn atomic_publish_new_preserves_existing_destination() {
        let root =
            std::env::temp_dir().join(format!("focusa-durable-new-{}", uuid::Uuid::now_v7()));
        fs::create_dir_all(&root).unwrap();
        let destination = root.join("receipt.json");
        let replacement = root.join("receipt.tmp");
        fs::write(&destination, b"existing").unwrap();
        fs::write(&replacement, b"candidate").unwrap();
        let error = atomic_publish_new(&replacement, &destination).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
        assert_eq!(fs::read(&destination).unwrap(), b"existing");
        assert_eq!(fs::read(&replacement).unwrap(), b"candidate");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn atomic_publish_new_creates_missing_destination() {
        let root =
            std::env::temp_dir().join(format!("focusa-durable-new-{}", uuid::Uuid::now_v7()));
        fs::create_dir_all(&root).unwrap();
        let destination = root.join("receipt.json");
        let replacement = root.join("receipt.tmp");
        fs::write(&replacement, b"candidate").unwrap();
        File::options()
            .write(true)
            .open(&replacement)
            .unwrap()
            .sync_all()
            .unwrap();
        atomic_publish_new(&replacement, &destination).unwrap();
        sync_directory(&root).unwrap();
        assert_eq!(fs::read(&destination).unwrap(), b"candidate");
        fs::remove_dir_all(root).unwrap();
    }
}
