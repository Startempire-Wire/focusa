use std::{
    collections::BTreeSet,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::unix::{
        fs::{FileTypeExt, MetadataExt, OpenOptionsExt, PermissionsExt},
        process::CommandExt,
    },
    path::{Path, PathBuf},
    process::Stdio,
};

use anyhow::{Context, Result, ensure};
use focusa_core::silent_sessions::{
    LaunchManifest, MissionDelivery, ProcessBackend, RunnerSignal, StdioMode,
    validate_environment_name,
};
use sha2::{Digest, Sha256};
use tokio::process::Command;

pub(crate) struct PreparedLaunch {
    pub command: Command,
    pub workspace: PathBuf,
    pub manifest_digest: String,
}

pub(crate) fn prepare_launch_manifest(
    manifest: &LaunchManifest,
    owner_uid: u32,
    secret_dir: &Path,
) -> Result<PreparedLaunch> {
    manifest.validate()?;
    let workspace = canonical_owned_directory(&manifest.cwd, owner_uid)?;
    let executable = fs::canonicalize(&manifest.executable)
        .with_context(|| format!("canonicalize executable {}", manifest.executable))?;
    ensure!(
        executable.is_file(),
        "launch executable is not a regular file"
    );
    ensure!(
        manifest.os_user == current_user()?.0,
        "launch manifest OS user mismatch"
    );
    ensure!(
        matches!(
            manifest.process_backend,
            ProcessBackend::UnixProcessGroup | ProcessBackend::EmbeddedSameUser
        ),
        "runner does not support the declared process backend"
    );
    verify_mission_delivery(&manifest.mission_delivery, &manifest.argv, owner_uid)?;

    let mut command = Command::new(executable);
    command
        .args(&manifest.argv)
        .current_dir(&workspace)
        .env_clear()
        .stdin(stdin_for_manifest(manifest, owner_uid)?)
        .stdout(stdio_for_mode(manifest.stdout_mode)?)
        .stderr(stdio_for_mode(manifest.stderr_mode)?);
    for (name, value) in &manifest.safe_env {
        validate_environment_name(name)?;
        command.env(name, value);
    }
    for secret in &manifest.secret_env_refs {
        let value = resolve_secret_reference(&secret.secret_ref, secret_dir, owner_uid)?;
        command.env(&secret.env_name, value);
    }
    command.as_std_mut().process_group(0);
    Ok(PreparedLaunch {
        command,
        workspace,
        manifest_digest: manifest.digest()?,
    })
}

fn stdin_for_manifest(manifest: &LaunchManifest, owner_uid: u32) -> Result<Stdio> {
    match manifest.stdin_mode {
        StdioMode::Null => Ok(Stdio::null()),
        StdioMode::Inherit => Ok(Stdio::inherit()),
        StdioMode::Pipe => Ok(Stdio::piped()),
        StdioMode::MissionArtifact => match &manifest.mission_delivery {
            MissionDelivery::Stdin { artifact_path, .. } => {
                Ok(Stdio::from(open_secure_artifact(artifact_path, owner_uid)?))
            }
            _ => bail_manifest("mission stdin mode lacks a stdin artifact"),
        },
    }
}

fn stdio_for_mode(mode: StdioMode) -> Result<Stdio> {
    match mode {
        StdioMode::Null | StdioMode::MissionArtifact => Ok(Stdio::null()),
        StdioMode::Inherit => Ok(Stdio::inherit()),
        StdioMode::Pipe => Ok(Stdio::piped()),
    }
}

fn verify_mission_delivery(
    delivery: &MissionDelivery,
    argv: &[String],
    owner_uid: u32,
) -> Result<()> {
    match delivery {
        MissionDelivery::Rpc { .. } => Ok(()),
        MissionDelivery::Stdin {
            artifact_path,
            sha256,
        }
        | MissionDelivery::SecureArtifact {
            artifact_path,
            sha256,
        } => {
            let mut artifact = open_secure_artifact(artifact_path, owner_uid)?;
            let mut bytes = Vec::new();
            artifact.read_to_end(&mut bytes)?;
            ensure!(
                hex::encode(Sha256::digest(bytes)) == *sha256,
                "mission artifact hash mismatch"
            );
            Ok(())
        }
        MissionDelivery::TypedArgument {
            argv_index,
            sha256,
            max_bytes,
        } => {
            let value = argv
                .get(*argv_index)
                .context("mission argv index is out of bounds")?;
            ensure!(
                value.len() <= *max_bytes,
                "typed mission argument exceeds its bound"
            );
            ensure!(
                hex::encode(Sha256::digest(value.as_bytes())) == *sha256,
                "typed mission argument hash mismatch"
            );
            Ok(())
        }
    }
}

fn open_secure_artifact(path: &str, owner_uid: u32) -> Result<File> {
    let path = Path::new(path);
    ensure!(path.is_absolute(), "mission artifact path must be absolute");
    let metadata = fs::symlink_metadata(path)?;
    ensure!(
        metadata.is_file() && !metadata.file_type().is_symlink(),
        "mission artifact is not a regular file"
    );
    ensure!(
        metadata.uid() == owner_uid,
        "mission artifact owner mismatch"
    );
    ensure!(
        metadata.permissions().mode() & 0o077 == 0,
        "mission artifact permissions are too broad"
    );
    File::open(path).map_err(Into::into)
}

fn resolve_secret_reference(reference: &str, secret_dir: &Path, owner_uid: u32) -> Result<String> {
    if let Some(name) = reference.strip_prefix("env://") {
        validate_environment_name(name)?;
        return std::env::var(name)
            .with_context(|| format!("resolve inherited secret reference {name}"));
    }
    let name = reference
        .strip_prefix("secret://")
        .context("unsupported secret reference scheme")?;
    ensure!(valid_secret_name(name), "invalid secret reference name");
    let path = secret_dir.join(name);
    let mut secret = open_secure_artifact(
        path.to_str().context("secret path is not UTF-8")?,
        owner_uid,
    )?;
    let mut value = String::new();
    secret.read_to_string(&mut value)?;
    while value.ends_with('\n') || value.ends_with('\r') {
        value.pop();
    }
    ensure!(!value.is_empty(), "resolved secret is empty");
    Ok(value)
}

fn valid_secret_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 200
        && name.bytes().all(|byte| {
            byte == b'-' || byte == b'_' || byte == b'.' || byte.is_ascii_alphanumeric()
        })
        && !name.starts_with('.')
        && !name.contains("..")
}

fn bail_manifest<T>(message: &str) -> Result<T> {
    Err(anyhow::anyhow!(message.to_string()))
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
