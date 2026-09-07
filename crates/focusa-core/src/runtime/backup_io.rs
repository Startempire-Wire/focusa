//! Shared, fail-closed I/O primitives for the canonical backup runtime.

use super::backup_contracts::*;
use anyhow::{Context, Result, bail};
use chrono::Utc;
use rusqlite::{Connection, OptionalExtension, backup::Backup};
use serde::Serialize;
use sha2::{Digest, Sha256};
#[cfg(unix)]
use std::ffi::CString;
use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;
use uuid::Uuid;

pub(super) fn online_backup(source: &Path, destination: &Path) -> Result<()> {
    let source = Connection::open_with_flags(source, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    source.busy_timeout(Duration::from_secs(30))?;
    let journal_mode: String = source.query_row("PRAGMA journal_mode", [], |row| row.get(0))?;
    if !journal_mode.eq_ignore_ascii_case("wal") {
        bail!("source database must use WAL journal mode")
    }
    let mut destination = Connection::open(destination)?;
    let backup = Backup::new(&source, &mut destination)?;
    let started = std::time::Instant::now();
    let mut best_remaining = i32::MAX;
    let mut regressions = 0_u8;
    loop {
        let result = backup.step(4_096)?;
        let progress = backup.progress();
        if progress.remaining < best_remaining {
            best_remaining = progress.remaining;
        } else if progress.remaining > best_remaining.saturating_add(4_096) {
            regressions = regressions.saturating_add(1);
        }
        if matches!(result, rusqlite::backup::StepResult::Done) {
            break;
        }
        if regressions >= 3 || started.elapsed() >= Duration::from_secs(120) {
            tracing::warn!(
                remaining_pages = progress.remaining,
                page_count = progress.pagecount,
                regressions,
                elapsed_seconds = started.elapsed().as_secs(),
                "online backup did not converge; taking bounded final source lock"
            );
            loop {
                match backup.step(-1)? {
                    rusqlite::backup::StepResult::Done => break,
                    rusqlite::backup::StepResult::Busy
                    | rusqlite::backup::StepResult::Locked
                    | rusqlite::backup::StepResult::More => {
                        std::thread::sleep(Duration::from_millis(25));
                    }
                    _ => bail!("unsupported SQLite backup step result"),
                }
            }
            break;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    drop(backup);
    destination.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
    Ok(())
}

pub(super) fn append_progress_receipt(
    path: &Path,
    base: &BackupReceipt,
    phase: &str,
    bytes: u64,
    quick_check: Option<String>,
) -> Result<()> {
    let mut receipt = base.clone();
    receipt.phase = phase.to_string();
    receipt.status = "in_progress".to_string();
    receipt.timestamp = Utc::now();
    receipt.bytes = bytes;
    receipt.quick_check = quick_check;
    append_receipt(path, &receipt)
}

pub(super) fn append_receipt(path: &Path, receipt: &BackupReceipt) -> Result<()> {
    fs::create_dir_all(path.parent().context("receipt parent missing")?)?;
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    serde_json::to_writer(&mut file, receipt)?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    Ok(())
}

pub(super) fn prepare_root(root: &Path, data_dir: &Path) -> Result<()> {
    reject_symlink_components(root)?;
    fs::create_dir_all(root)?;
    let canonical_root = root.canonicalize()?;
    let canonical_data = data_dir.canonicalize()?;
    if canonical_root.starts_with(&canonical_data) {
        bail!("backup root resolves inside live data directory")
    }
    for subdir in ["locks", "staging", "generations", "receipts"] {
        create_private_dir(&root.join(subdir))?;
    }
    Ok(())
}

pub(super) fn reject_symlink_components(path: &Path) -> Result<()> {
    let mut current = PathBuf::new();
    for component in path.components() {
        current.push(component);
        if let Ok(metadata) = fs::symlink_metadata(&current) {
            if metadata.file_type().is_symlink() {
                bail!("backup path contains symlink: {}", current.display())
            }
        }
    }
    Ok(())
}

pub(super) fn canonical_regular_file(path: &Path) -> Result<PathBuf> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        bail!("source database must be a real regular file")
    }
    Ok(path.canonicalize()?)
}

pub(super) struct FailureReceiptGuard {
    path: PathBuf,
    receipt: BackupReceipt,
    pub(super) settled: bool,
}

impl FailureReceiptGuard {
    pub(super) fn new(path: PathBuf, receipt: BackupReceipt) -> Self {
        Self {
            path,
            receipt,
            settled: false,
        }
    }
}

impl Drop for FailureReceiptGuard {
    fn drop(&mut self) {
        if self.settled {
            return;
        }
        self.receipt.phase = "failed".to_string();
        self.receipt.status = "failed".to_string();
        self.receipt.timestamp = Utc::now();
        self.receipt.error_code = Some("operation_aborted".to_string());
        self.receipt.error = Some("backup operation exited before verified settlement".to_string());
        if let Err(error) = append_receipt(&self.path, &self.receipt) {
            tracing::error!(error = %error, "failed to append backup failure receipt");
        }
    }
}

pub(super) struct MaintenanceLock(PathBuf);
impl MaintenanceLock {
    pub(super) fn acquire(root: &Path) -> Result<Self> {
        let path = root.join("locks/maintenance.lock");
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .context("backup maintenance operation already active")?;
        writeln!(
            file,
            "pid={}\nstarted_at={}",
            std::process::id(),
            Utc::now().to_rfc3339()
        )?;
        file.sync_all()?;
        Ok(Self(path))
    }
}
impl Drop for MaintenanceLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

pub(super) fn compress_file(input: &Path, output: &Path, level: i32) -> Result<()> {
    let mut source = File::open(input)?;
    let destination = File::create(output)?;
    let mut encoder = zstd::stream::write::Encoder::new(BufWriter::new(destination), level)?;
    std::io::copy(&mut source, &mut encoder)?;
    let mut writer = encoder.finish()?;
    writer.flush()?;
    writer.get_ref().sync_all()?;
    Ok(())
}

pub(super) fn inventory_digest(root: &Path) -> Result<String> {
    if !root.is_dir() {
        return Ok(sha256_bytes(b"missing"));
    }
    let mut files = Vec::new();
    collect_files(root, root, &mut files)?;
    files.sort();
    let mut hasher = Sha256::new();
    for relative in files {
        let path = root.join(&relative);
        hasher.update(relative.to_string_lossy().as_bytes());
        hasher.update([0]);
        hasher.update(fs::metadata(&path)?.len().to_le_bytes());
        hasher.update(sha256_file(&path)?.as_bytes());
    }
    Ok(hex::encode(hasher.finalize()))
}

pub(super) fn collect_files(root: &Path, current: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let metadata = entry.file_type()?;
        if metadata.is_symlink() {
            bail!("inventory contains symlink")
        }
        if metadata.is_dir() {
            collect_files(root, &entry.path(), files)?;
        } else if metadata.is_file() {
            files.push(entry.path().strip_prefix(root)?.to_path_buf());
        }
    }
    Ok(())
}

pub(super) fn sha256_file(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 1024 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hex::encode(hasher.finalize()))
}
pub(super) fn sha256_bytes(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}
pub(super) fn deterministic_generation_id(slot: &str, identity: &str, policy: &str) -> String {
    deterministic_kind_id("full", slot, identity, policy)
}
pub(super) fn deterministic_incremental_id(slot: &str, identity: &str, policy: &str) -> String {
    deterministic_kind_id("delta", slot, identity, policy)
}
fn deterministic_kind_id(kind: &str, slot: &str, identity: &str, policy: &str) -> String {
    format!(
        "{kind}-{}",
        &sha256_bytes(format!("{kind}\n{slot}\n{identity}\n{policy}").as_bytes())[..24]
    )
}
pub(super) fn file_identity(path: &Path) -> Result<String> {
    let metadata = fs::metadata(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        Ok(format!(
            "dev:{}:ino:{}:len:{}",
            metadata.dev(),
            metadata.ino(),
            metadata.len()
        ))
    }
    #[cfg(not(unix))]
    {
        Ok(format!("len:{}", metadata.len()))
    }
}
pub(super) fn pragma_u64(conn: &Connection, name: &str) -> Result<u64> {
    Ok(conn
        .query_row(&format!("PRAGMA {name}"), [], |row| row.get::<_, i64>(0))?
        .max(0) as u64)
}
pub(super) fn query_u64_or_zero(conn: &Connection, sql: &str) -> Result<u64> {
    Ok(conn.query_row(sql, [], |row| row.get::<_, i64>(0))?.max(0) as u64)
}
pub(super) fn query_meta(conn: &Connection, key: &str) -> Result<Option<String>> {
    Ok(conn
        .query_row("SELECT value FROM meta WHERE key=?1", [key], |row| {
            row.get(0)
        })
        .optional()?)
}
pub(super) fn write_json_atomic<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let temp = path.with_extension(format!("tmp-{}", Uuid::now_v7()));
    let mut file = File::create(&temp)?;
    serde_json::to_writer_pretty(&mut file, value)?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    fs::rename(temp, path)?;
    Ok(())
}
pub(super) fn sync_dir(path: &Path) -> Result<()> {
    File::open(path)?.sync_all()?;
    Ok(())
}
pub(super) fn create_private_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}
#[cfg(unix)]
pub(super) fn filesystem_space(path: &Path) -> Result<(u64, u64)> {
    let c_path = CString::new(path.as_os_str().as_encoded_bytes())?;
    let mut stat: libc::statvfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::statvfs(c_path.as_ptr(), &mut stat) } != 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    Ok((
        stat.f_bavail.saturating_mul(stat.f_frsize),
        stat.f_blocks.saturating_mul(stat.f_frsize),
    ))
}
#[cfg(windows)]
pub(super) fn filesystem_space(path: &Path) -> Result<(u64, u64)> {
    use std::os::windows::ffi::OsStrExt;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetDiskFreeSpaceExW(
            directory: *const u16,
            available: *mut u64,
            capacity: *mut u64,
            free: *mut u64,
        ) -> i32;
    }

    let mut directory: Vec<u16> = path.as_os_str().encode_wide().collect();
    if directory.contains(&0) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "filesystem path contains a null character",
        )
        .into());
    }
    directory.push(0);
    let mut available = 0_u64;
    let mut capacity = 0_u64;
    // SAFETY: directory is null-terminated UTF-16; both output pointers are
    // valid for the call. The optional total-free output is deliberately null.
    if unsafe {
        GetDiskFreeSpaceExW(
            directory.as_ptr(),
            &mut available,
            &mut capacity,
            std::ptr::null_mut(),
        )
    } == 0
    {
        return Err(std::io::Error::last_os_error().into());
    }
    Ok((available, capacity))
}

pub(super) fn available_bytes(path: &Path) -> Result<u64> {
    Ok(filesystem_space(path)?.0)
}
