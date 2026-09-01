use std::fs::{self, File};
use std::io;
use std::path::Path;

/// Atomically publish `replacement` at `destination`, replacing an existing
/// destination on every supported platform.
#[cfg(not(windows))]
pub fn atomic_replace(replacement: &Path, destination: &Path) -> io::Result<()> {
    fs::rename(replacement, destination)
}

/// Windows `std::fs::rename` does not replace an existing destination. Use the
/// native write-through primitive so durable ledgers retain atomic publication.
#[cfg(windows)]
pub fn atomic_replace(replacement: &Path, destination: &Path) -> io::Result<()> {
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
    // SAFETY: both path buffers are NUL-terminated and remain alive for the
    // duration of the synchronous Win32 call.
    let moved = unsafe {
        MoveFileExW(
            from.as_ptr(),
            to.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };
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
        File::open(&replacement).unwrap().sync_all().unwrap();

        atomic_replace(&replacement, &destination).unwrap();
        sync_directory(&root).unwrap();

        assert_eq!(fs::read(&destination).unwrap(), b"new");
        assert!(!replacement.exists());
        fs::remove_dir_all(root).unwrap();
    }
}
