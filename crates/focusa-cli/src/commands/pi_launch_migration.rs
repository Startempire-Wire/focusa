//! Bounded streaming migration for oversized Pi JSONL sessions.

use anyhow::{Context, Result, bail};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::VecDeque;
use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};

const RECOVERY_MAX_BYTES: usize = 8 * 1024 * 1024;
const ENTRY_MAX_BYTES: usize = 256 * 1024;

#[derive(Debug)]
pub struct MigrationResult {
    pub recovery_path: PathBuf,
    pub manifest_path: PathBuf,
    pub source_bytes: u64,
    pub recovery_bytes: u64,
}

struct ScanResult {
    source_sha256: String,
    source_bytes: u64,
    entry_count: u64,
    omitted_entries: u64,
    recovery_entries: Vec<Vec<u8>>,
    recovery_bytes: usize,
}

struct RecoveryCollector {
    entries: VecDeque<Vec<u8>>,
    total: usize,
    line: Vec<u8>,
    line_hash: Sha256,
    line_bytes: usize,
    line_oversized: bool,
    entry_count: u64,
    omitted_entries: u64,
}

impl RecoveryCollector {
    fn new() -> Self {
        Self {
            entries: VecDeque::new(),
            total: 0,
            line: Vec::new(),
            line_hash: Sha256::new(),
            line_bytes: 0,
            line_oversized: false,
            entry_count: 0,
            omitted_entries: 0,
        }
    }

    fn consume(&mut self, segment: &[u8]) {
        self.line_hash.update(segment);
        self.line_bytes = self.line_bytes.saturating_add(segment.len());
        if !self.line_oversized && self.line_bytes <= ENTRY_MAX_BYTES {
            self.line.extend_from_slice(segment);
        } else {
            self.line_oversized = true;
            self.line.clear();
        }
    }

    fn finish_line(&mut self) -> Result<()> {
        if self.line_bytes == 0 && !self.line_oversized {
            return Ok(());
        }
        self.entry_count += 1;
        let digest = format!("sha256:{:x}", self.line_hash.clone().finalize());
        let mut entry = if self.line_oversized {
            self.omitted_entries += 1;
            serde_json::to_vec(&json!({
                "type": "focusa-migration-omitted-entry",
                "schema": "focusa.native_session_omitted_entry.v1",
                "bytes": self.line_bytes,
                "sha256": digest,
                "reason": "entry_exceeds_migration_budget"
            }))?
        } else {
            std::mem::take(&mut self.line)
        };
        entry.push(b'\n');
        self.push(entry);
        self.line_hash = Sha256::new();
        self.line_bytes = 0;
        self.line_oversized = false;
        Ok(())
    }

    fn push(&mut self, entry: Vec<u8>) {
        if entry.len() > RECOVERY_MAX_BYTES {
            return;
        }
        while self.total + entry.len() > RECOVERY_MAX_BYTES {
            let Some(oldest) = self.entries.pop_front() else {
                break;
            };
            self.total -= oldest.len();
        }
        self.total += entry.len();
        self.entries.push_back(entry);
    }
}

pub fn migrate(
    source: &Path,
    requested_root: Option<&Path>,
    project_root: &Path,
) -> Result<MigrationResult> {
    let before = fs::metadata(source).context("stat oversized native session")?;
    if !before.is_file() {
        bail!("native session migration source is not a file");
    }
    let scan = scan_jsonl(source)?;
    let digest_id = &scan.source_sha256[7..23];
    let base_root = requested_root.map(PathBuf::from).unwrap_or_else(|| {
        source
            .parent()
            .unwrap_or(project_root)
            .join("focusa-preflight-migrations")
    });
    let migration_dir = base_root.join(format!("native-session-{digest_id}"));
    create_private_dir(&migration_dir)?;

    let archive = migration_dir.join("source.immutable.jsonl");
    let recovery = migration_dir.join("recovery.jsonl");
    let manifest = migration_dir.join("manifest.json");
    if archive.exists() || recovery.exists() || manifest.exists() {
        bail!(
            "migration target already exists; inspect or remove the incomplete target before retry"
        );
    }

    let result = (|| -> Result<MigrationResult> {
        fs::copy(source, &archive).context("stream-copy immutable native session archive")?;
        set_file_mode(&archive, 0o400, true)?;
        let archive_hash = hash_file(&archive)?;
        if archive_hash != scan.source_sha256 || fs::metadata(&archive)?.len() != scan.source_bytes
        {
            bail!("native session archive integrity mismatch");
        }

        write_entries_atomic(&recovery, &scan.recovery_entries)?;
        let recovery_bytes = fs::metadata(&recovery)?.len();
        if recovery_bytes > RECOVERY_MAX_BYTES as u64 {
            bail!("recovery segment exceeded bounded migration budget");
        }
        let recovery_hash = hash_file(&recovery)?;
        let after = fs::metadata(source)?;
        let source_hash_after = hash_file(source)?;
        let source_unchanged = before.len() == after.len()
            && before.modified().ok() == after.modified().ok()
            && source_hash_after == scan.source_sha256;
        if !source_unchanged {
            bail!("native session source changed during migration");
        }

        let manifest_value = json!({
            "schema": "focusa.native_session_migration_manifest.v1",
            "migration_id": format!("native-session-{digest_id}"),
            "mode": "execute",
            "scope": {
                "status": "preload_unverified",
                "project_root": project_root,
                "continuity_id": null
            },
            "source": {
                "path": source,
                "bytes": scan.source_bytes,
                "sha256": scan.source_sha256
            },
            "archive": {
                "path": archive,
                "bytes": scan.source_bytes,
                "sha256": archive_hash,
                "immutable": true
            },
            "recovery_segment": {
                "path": recovery,
                "bytes": recovery_bytes,
                "sha256": recovery_hash,
                "entry_count": scan.entry_count,
                "omitted_oversized_entries": scan.omitted_entries
            },
            "integrity": {
                "source_unchanged": true,
                "archive_matches_source": true,
                "recovery_within_budget": true
            },
            "rollback": {
                "action": "resume_immutable_source",
                "source_path": source,
                "source_sha256": scan.source_sha256
            }
        });
        write_atomic(
            &manifest,
            &serde_json::to_vec_pretty(&manifest_value)?,
            0o600,
        )?;
        Ok(MigrationResult {
            recovery_path: recovery.clone(),
            manifest_path: manifest.clone(),
            source_bytes: scan.source_bytes,
            recovery_bytes,
        })
    })();

    if result.is_err() {
        let _ = set_file_mode(&archive, 0o600, false);
        let _ = fs::remove_file(&archive);
        let _ = fs::remove_file(&recovery);
        let _ = fs::remove_file(&manifest);
    }
    result
}

fn scan_jsonl(path: &Path) -> Result<ScanResult> {
    let file = File::open(path).context("open native session for bounded scan")?;
    let mut reader = BufReader::with_capacity(64 * 1024, file);
    let mut source_hash = Sha256::new();
    let mut source_bytes = 0_u64;
    let mut collector = RecoveryCollector::new();
    loop {
        let buffer = reader.fill_buf()?;
        if buffer.is_empty() {
            break;
        }
        source_hash.update(buffer);
        source_bytes = source_bytes.saturating_add(buffer.len() as u64);
        let mut start = 0;
        for (index, byte) in buffer.iter().enumerate() {
            if *byte == b'\n' {
                collector.consume(&buffer[start..index]);
                collector.finish_line()?;
                start = index + 1;
            }
        }
        collector.consume(&buffer[start..]);
        let consumed = buffer.len();
        reader.consume(consumed);
    }
    if collector.line_bytes > 0 || collector.line_oversized {
        collector.finish_line()?;
    }
    Ok(ScanResult {
        source_sha256: format!("sha256:{:x}", source_hash.finalize()),
        source_bytes,
        entry_count: collector.entry_count,
        omitted_entries: collector.omitted_entries,
        recovery_bytes: collector.total,
        recovery_entries: collector.entries.into_iter().collect(),
    })
}

fn hash_file(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hash = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hash.update(&buffer[..read]);
    }
    Ok(format!("sha256:{:x}", hash.finalize()))
}

fn create_private_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

fn write_entries_atomic(path: &Path, entries: &[Vec<u8>]) -> Result<()> {
    let temporary = path.with_extension(format!("tmp-{}", std::process::id()));
    let mut file = create_private_file(&temporary)?;
    for entry in entries {
        file.write_all(entry)?;
    }
    file.sync_all()?;
    fs::rename(&temporary, path)?;
    Ok(())
}

fn write_atomic(path: &Path, bytes: &[u8], mode: u32) -> Result<()> {
    let temporary = path.with_extension(format!("tmp-{}", std::process::id()));
    let mut file = create_file_with_mode(&temporary, mode)?;
    file.write_all(bytes)?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    fs::rename(&temporary, path)?;
    Ok(())
}

fn create_private_file(path: &Path) -> Result<File> {
    create_file_with_mode(path, 0o600)
}

fn create_file_with_mode(path: &Path, mode: u32) -> Result<File> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(mode);
    }
    Ok(options.open(path)?)
}

fn set_file_mode(path: &Path, mode: u32, readonly: bool) -> Result<()> {
    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_readonly(readonly);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        permissions.set_mode(mode);
    }
    fs::set_permissions(path, permissions)?;
    Ok(())
}

pub fn rewrite_args_for_recovery(args: &mut Vec<OsString>, recovery: &Path) {
    let recovery_value = recovery.as_os_str().to_os_string();
    let mut index = 0;
    while index < args.len() {
        let text = args[index].to_string_lossy();
        if text == "--continue" || text == "-c" {
            args.splice(
                index..=index,
                [OsString::from("--session"), recovery_value.clone()],
            );
            return;
        }
        for flag in ["--session", "--fork", "--session-id"] {
            if text == flag {
                let end = usize::min(index + 1, args.len().saturating_sub(1));
                args.splice(
                    index..=end,
                    [OsString::from("--session"), recovery_value.clone()],
                );
                return;
            }
            if text.starts_with(&format!("{flag}=")) {
                args.splice(
                    index..=index,
                    [OsString::from("--session"), recovery_value.clone()],
                );
                return;
            }
        }
        if text == "--export" {
            if index + 1 < args.len() {
                args[index + 1] = recovery_value;
            }
            return;
        }
        if text.starts_with("--export=") {
            args.splice(index..=index, [OsString::from("--export"), recovery_value]);
            return;
        }
        index += 1;
    }
    args.extend([OsString::from("--session"), recovery_value]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn fixture_dir(label: &str) -> PathBuf {
        let path =
            std::env::temp_dir().join(format!("focusa-migration-{label}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn bounded_scan_replaces_oversized_entry() {
        let root = fixture_dir("scan");
        let source = root.join("source.jsonl");
        let mut file = File::create(&source).unwrap();
        writeln!(file, "{}", json!({"type":"session","id":"one"})).unwrap();
        file.write_all(&vec![b'x'; ENTRY_MAX_BYTES + 1]).unwrap();
        file.write_all(b"\n").unwrap();
        let scan = scan_jsonl(&source).unwrap();
        assert_eq!(scan.entry_count, 2);
        assert_eq!(scan.omitted_entries, 1);
        assert!(scan.recovery_bytes <= RECOVERY_MAX_BYTES);
        assert!(
            scan.recovery_entries
                .last()
                .unwrap()
                .windows(38)
                .any(|window| window == b"focusa.native_session_omitted_entry.v1")
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn migration_preserves_source_and_writes_verified_outputs() {
        let root = fixture_dir("integrity");
        let source = root.join("source.jsonl");
        fs::write(
            &source,
            b"{\"type\":\"session\",\"id\":\"one\"}\n{\"type\":\"message\"}\n",
        )
        .unwrap();
        let before = hash_file(&source).unwrap();
        let result = migrate(&source, Some(&root.join("out")), &root).unwrap();
        assert_eq!(hash_file(&source).unwrap(), before);
        assert!(result.recovery_bytes <= RECOVERY_MAX_BYTES as u64);
        assert!(result.recovery_path.is_file());
        assert!(result.manifest_path.is_file());
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(&result.manifest_path).unwrap()).unwrap();
        assert_eq!(manifest["integrity"]["source_unchanged"], true);
        assert_eq!(manifest["integrity"]["archive_matches_source"], true);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rewrites_continue_to_bounded_recovery_session() {
        let recovery = Path::new("/safe/recovery.jsonl");
        let mut args = vec![OsString::from("--continue"), OsString::from("hello")];
        rewrite_args_for_recovery(&mut args, recovery);
        assert_eq!(args[0], "--session");
        assert_eq!(args[1], recovery.as_os_str());
        assert_eq!(args[2], "hello");
    }
}
