use std::{
    collections::BTreeSet,
    fs::{self, OpenOptions},
    io::Write,
    os::unix::fs::{FileTypeExt, MetadataExt, OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, ensure};
use focusa_core::silent_sessions::RunnerSignal;
use tokio::process::Command;

pub(crate) fn valid_environment_name(name: &str) -> bool {
    !name.is_empty()
        && !name.as_bytes()[0].is_ascii_digit()
        && name.len() <= 128
        && name
            .bytes()
            .all(|byte| byte == b'_' || byte.is_ascii_alphanumeric())
}

fn command_output(program: &str, args: &[&str]) -> Result<String> {
    let output = std::process::Command::new(program).args(args).output()?;
    ensure!(
        output.status.success(),
        "{program} returned a failure status"
    );
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

pub(crate) fn current_user() -> Result<(String, u32)> {
    let user = command_output("/usr/bin/id", &["-un"])?;
    let uid = command_output("/usr/bin/id", &["-u"])?.parse()?;
    Ok((user, uid))
}

pub(crate) fn canonical_owned_directory(path: &str, owner_uid: u32) -> Result<PathBuf> {
    let canonical =
        fs::canonicalize(path).with_context(|| format!("canonicalize workspace {path}"))?;
    let metadata = fs::metadata(&canonical)?;
    ensure!(metadata.is_dir(), "workspace is not a directory");
    ensure!(
        metadata.uid() == owner_uid,
        "workspace is not owned by the verified runner user"
    );
    Ok(canonical)
}

pub(crate) fn read_secret(path: &Path, owner_uid: u32) -> Result<Vec<u8>> {
    let metadata =
        fs::metadata(path).with_context(|| format!("inspect key file {}", path.display()))?;
    ensure!(metadata.is_file(), "runner key is not a regular file");
    ensure!(metadata.uid() == owner_uid, "runner key owner mismatch");
    ensure!(
        metadata.permissions().mode() & 0o077 == 0,
        "runner key permissions must exclude group and other users"
    );
    let secret = fs::read(path)?;
    ensure!(
        secret.len() >= 32,
        "runner key must contain at least 32 bytes"
    );
    Ok(secret)
}

pub(crate) fn load_nonces(path: &Path, owner_uid: u32) -> Result<BTreeSet<String>> {
    if !path.exists() {
        return Ok(BTreeSet::new());
    }
    let metadata = fs::metadata(path)?;
    ensure!(metadata.uid() == owner_uid, "nonce ledger owner mismatch");
    ensure!(
        metadata.permissions().mode() & 0o077 == 0,
        "nonce ledger permissions are too broad"
    );
    Ok(fs::read_to_string(path)?
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(str::to_string)
        .collect())
}

pub(crate) fn append_nonce(path: &Path, nonce: &str, owner_uid: u32) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
        fs::set_permissions(parent, fs::Permissions::from_mode(0o700))?;
    }
    if path.exists() {
        let metadata = fs::symlink_metadata(path)?;
        ensure!(
            metadata.is_file() && !metadata.file_type().is_symlink(),
            "nonce ledger is not a regular file"
        );
        ensure!(metadata.uid() == owner_uid, "nonce ledger owner mismatch");
        ensure!(
            metadata.permissions().mode() & 0o077 == 0,
            "nonce ledger permissions are too broad"
        );
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .mode(0o600)
        .open(path)?;
    writeln!(file, "{nonce}")?;
    file.sync_data()?;
    Ok(())
}

pub(crate) fn prepare_socket(path: &Path, owner_uid: u32) -> Result<()> {
    let parent = path
        .parent()
        .context("runner socket requires a parent directory")?;
    fs::create_dir_all(parent)?;
    fs::set_permissions(parent, fs::Permissions::from_mode(0o700))?;
    ensure!(
        fs::metadata(parent)?.uid() == owner_uid,
        "runner socket directory owner mismatch"
    );
    if path.exists() {
        let metadata = fs::symlink_metadata(path)?;
        ensure!(
            metadata.file_type().is_socket(),
            "refusing to replace non-socket path"
        );
        ensure!(
            metadata.uid() == owner_uid,
            "refusing to replace a socket owned by another user"
        );
        ensure!(
            std::os::unix::net::UnixStream::connect(path).is_err(),
            "refusing to replace an active runner socket"
        );
        fs::remove_file(path)?;
    }
    Ok(())
}

pub(crate) fn process_group_exists(process_group_id: i32) -> Result<bool> {
    let status = std::process::Command::new("/bin/kill")
        .args(["-0", "--", &format!("-{process_group_id}")])
        .status()?;
    Ok(status.success())
}

pub(crate) async fn send_process_group_signal(
    process_group_id: i32,
    signal: RunnerSignal,
) -> Result<()> {
    let signal = match signal {
        RunnerSignal::Pause => "STOP",
        RunnerSignal::Resume => "CONT",
        RunnerSignal::Interrupt => "INT",
        RunnerSignal::Cancel => "TERM",
        RunnerSignal::ForceKill => "KILL",
    };
    let status = Command::new("/bin/kill")
        .args(["-s", signal, "--", &format!("-{process_group_id}")])
        .status()
        .await?;
    ensure!(status.success(), "process-group signal delivery failed");
    Ok(())
}
