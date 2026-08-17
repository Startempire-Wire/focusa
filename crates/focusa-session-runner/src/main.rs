//! Protected per-user process runner for daemon-native Silent Sessions.

#[cfg(unix)]
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::PathBuf,
};

#[cfg(unix)]
use anyhow::{Context, Result, ensure};
#[cfg(unix)]
use chrono::Utc;
#[cfg(unix)]
use clap::Parser;
#[cfg(unix)]
use focusa_core::silent_sessions::{
    ProcessTreeIdentity, RUNNER_PROTOCOL_SCHEMA, RunnerHeartbeat, RunnerIdentity, RunnerOperation,
    RunnerProcessProjection, RunnerProcessState, RunnerSignal, RunnerWireRequest,
    RunnerWireResponse, SilentSessionId, SilentSessionRunId,
};
#[cfg(unix)]
use serde_json::{Value, json};
#[cfg(unix)]
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::{UnixListener, UnixStream},
    process::Child,
};
#[cfg(unix)]
use tracing::{error, info};

#[cfg(unix)]
mod security;

#[cfg(unix)]
use security::{
    append_nonce, canonical_owned_directory, controlled_stop, current_user, load_nonces,
    prepare_launch_manifest, prepare_socket, process_group_exists, read_secret,
    send_process_group_signal,
};

#[cfg(unix)]
const MAX_REQUEST_BYTES: usize = 1024 * 1024;

#[cfg(unix)]
#[derive(Debug, Parser)]
#[command(name = "focusa-session-runner")]
struct Args {
    #[arg(long)]
    socket: PathBuf,
    #[arg(long)]
    key_file: PathBuf,
    #[arg(long)]
    nonce_ledger: PathBuf,
    #[arg(long)]
    secret_dir: PathBuf,
    #[arg(long)]
    principal_id: String,
    #[arg(long)]
    owner_os_user: String,
    #[arg(long)]
    socket_scope: String,
}

#[cfg(unix)]
struct ManagedProcess {
    identity: ProcessTreeIdentity,
    child: Option<Child>,
    state: RunnerProcessState,
    exit_code: Option<i32>,
}

#[cfg(unix)]
impl ManagedProcess {
    fn projection(&self) -> RunnerProcessProjection {
        RunnerProcessProjection {
            identity: self.identity.clone(),
            state: self.state.clone(),
            exit_code: self.exit_code,
        }
    }

    fn refresh(&mut self) -> Result<()> {
        if let Some(child) = &mut self.child {
            if let Some(status) = child.try_wait()? {
                self.state = RunnerProcessState::Exited;
                self.exit_code = status.code();
            }
        } else if !process_group_exists(self.identity.process_group_id)? {
            self.state = RunnerProcessState::Exited;
        }
        Ok(())
    }
}

#[cfg(unix)]
struct RunnerState {
    identity: RunnerIdentity,
    owner_uid: u32,
    key: Vec<u8>,
    nonce_ledger: PathBuf,
    secret_dir: PathBuf,
    consumed_nonces: BTreeSet<String>,
    processes: BTreeMap<String, ManagedProcess>,
}

#[cfg(unix)]
impl RunnerState {
    fn process_key(session_id: SilentSessionId, run_id: SilentSessionRunId) -> String {
        format!("{session_id}:{run_id}")
    }

    fn response(
        &self,
        request: &RunnerWireRequest,
        status: &str,
        process: Option<RunnerProcessProjection>,
        heartbeat: Option<RunnerHeartbeat>,
    ) -> RunnerWireResponse {
        RunnerWireResponse {
            schema: RUNNER_PROTOCOL_SCHEMA.into(),
            ok: true,
            status: status.into(),
            session_id: request.command.session_id,
            run_id: request.command.run_id,
            replayed: false,
            process,
            heartbeat,
            details: BTreeMap::new(),
        }
    }

    fn authenticate(&mut self, request: &RunnerWireRequest) -> Result<()> {
        let payload = request.validate_binding()?;
        let mut consumed_nonces = self.consumed_nonces.clone();
        request.command.authenticate_payload(
            &self.identity,
            Utc::now(),
            &self.key,
            &mut consumed_nonces,
            &payload,
        )?;
        append_nonce(&self.nonce_ledger, &request.command.nonce, self.owner_uid)?;
        self.consumed_nonces = consumed_nonces;
        Ok(())
    }

    async fn apply(&mut self, request: &RunnerWireRequest) -> Result<RunnerWireResponse> {
        self.authenticate(request)?;
        match &request.operation {
            RunnerOperation::Launch { spec } => self.launch(request, spec.as_ref()).await,
            RunnerOperation::Signal { signal } => self.signal(request, *signal).await,
            RunnerOperation::Query => self.query(request),
            RunnerOperation::Heartbeat => Ok(self.heartbeat(request)),
            RunnerOperation::Adopt { expected } => self.adopt(request, expected),
        }
    }

    async fn launch(
        &mut self,
        request: &RunnerWireRequest,
        spec: &focusa_core::silent_sessions::RunnerLaunchSpec,
    ) -> Result<RunnerWireResponse> {
        ensure!(
            spec.manifest.os_user == self.identity.os_user,
            "launch owner mismatch"
        );
        let prepared = prepare_launch_manifest(&spec.manifest, self.owner_uid, &self.secret_dir)?;
        let workspace = prepared.workspace;
        let manifest_digest = prepared.manifest_digest;

        let key = Self::process_key(request.command.session_id, request.command.run_id);
        if let Some(existing) = self.processes.get_mut(&key) {
            existing.refresh()?;
            ensure!(
                existing.state == RunnerProcessState::Exited,
                "exact run already has an active process tree"
            );
        }

        let mut command = prepared.command;
        let mut child = command.spawn().context("spawn owned process tree")?;
        if let Some(mut stdout) = child.stdout.take() {
            tokio::spawn(async move {
                let _ = tokio::io::copy(&mut stdout, &mut tokio::io::stdout()).await;
            });
        }
        if let Some(mut stderr) = child.stderr.take() {
            tokio::spawn(async move {
                let _ = tokio::io::copy(&mut stderr, &mut tokio::io::stderr()).await;
            });
        }
        let pid = child.id().context("spawned process has no pid")?;
        let identity = ProcessTreeIdentity {
            session_id: request.command.session_id,
            run_id: request.command.run_id,
            pid,
            process_group_id: i32::try_from(pid).context("pid exceeds process-group range")?,
            owner_os_user: self.identity.os_user.clone(),
            workspace: workspace.to_string_lossy().into_owned(),
            manifest_digest,
            started_at: Utc::now(),
        };
        let process = ManagedProcess {
            identity,
            child: Some(child),
            state: RunnerProcessState::Running,
            exit_code: None,
        };
        let projection = process.projection();
        self.processes.insert(key, process);
        Ok(self.response(request, "launched", Some(projection), None))
    }

    async fn signal(
        &mut self,
        request: &RunnerWireRequest,
        signal: RunnerSignal,
    ) -> Result<RunnerWireResponse> {
        let key = Self::process_key(request.command.session_id, request.command.run_id);
        let process = self
            .processes
            .get_mut(&key)
            .context("exact process tree is not owned by this runner")?;
        process.refresh()?;
        ensure!(
            process.state != RunnerProcessState::Exited,
            "process tree already exited"
        );
        if signal == RunnerSignal::Cancel {
            let receipt = controlled_stop(
                process.identity.process_group_id,
                &focusa_core::silent_sessions::ControlledStopPolicy::default(),
            )
            .await?;
            receipt.verify_complete()?;
            process.state = RunnerProcessState::Exited;
        } else {
            send_process_group_signal(process.identity.process_group_id, signal).await?;
        }
        process.state = match signal {
            RunnerSignal::Pause => RunnerProcessState::Paused,
            RunnerSignal::Resume => RunnerProcessState::Running,
            RunnerSignal::Cancel => RunnerProcessState::Exited,
            RunnerSignal::Interrupt | RunnerSignal::ForceKill => RunnerProcessState::Running,
        };
        let projection = process.projection();
        Ok(self.response(request, "signal_delivered", Some(projection), None))
    }

    fn query(&mut self, request: &RunnerWireRequest) -> Result<RunnerWireResponse> {
        let key = Self::process_key(request.command.session_id, request.command.run_id);
        let process = self
            .processes
            .get_mut(&key)
            .context("exact process tree is unknown")?;
        process.refresh()?;
        let projection = process.projection();
        Ok(self.response(request, "process_projected", Some(projection), None))
    }

    fn heartbeat(&mut self, request: &RunnerWireRequest) -> RunnerWireResponse {
        let active_processes = self
            .processes
            .values()
            .filter(|process| process.state != RunnerProcessState::Exited)
            .count();
        let heartbeat = RunnerHeartbeat {
            runner_principal_id: self.identity.principal_id.clone(),
            owner_os_user: self.identity.os_user.clone(),
            socket_scope: self.identity.socket_scope.clone(),
            observed_at: Utc::now(),
            active_processes,
        };
        self.response(request, "heartbeat", None, Some(heartbeat))
    }

    fn adopt(
        &mut self,
        request: &RunnerWireRequest,
        expected: &ProcessTreeIdentity,
    ) -> Result<RunnerWireResponse> {
        ensure!(
            expected.session_id == request.command.session_id,
            "adoption session mismatch"
        );
        ensure!(
            expected.run_id == request.command.run_id,
            "adoption run mismatch"
        );
        ensure!(
            expected.owner_os_user == self.identity.os_user,
            "adoption owner mismatch"
        );
        let workspace = canonical_owned_directory(&expected.workspace, self.owner_uid)?;
        ensure!(
            workspace.as_os_str() == std::ffi::OsStr::new(&expected.workspace),
            "adoption workspace is not canonical"
        );
        ensure!(
            !expected.manifest_digest.trim().is_empty(),
            "adoption manifest digest is required"
        );
        ensure!(
            expected.pid > 1 && expected.process_group_id > 1,
            "unsafe adoption process identity"
        );
        ensure!(
            process_group_exists(expected.process_group_id)?,
            "adoption process group is not alive"
        );
        let key = Self::process_key(expected.session_id, expected.run_id);
        ensure!(
            !self.processes.contains_key(&key),
            "exact run is already tracked"
        );
        let process = ManagedProcess {
            identity: expected.clone(),
            child: None,
            state: RunnerProcessState::Running,
            exit_code: None,
        };
        let projection = process.projection();
        self.processes.insert(key, process);
        Ok(self.response(request, "adopted", Some(projection), None))
    }
}

#[cfg(unix)]
fn failure_response(request: Option<&RunnerWireRequest>, error: &anyhow::Error) -> Value {
    json!({
        "schema": RUNNER_PROTOCOL_SCHEMA,
        "ok": false,
        "status": "rejected",
        "session_id": request.map(|request| request.command.session_id),
        "run_id": request.map(|request| request.command.run_id),
        "failure_class": "runner_request_rejected",
        "recovery_hint": "Refresh exact runner identity, run generation, nonce and signed payload before retrying.",
        "detail": error.to_string(),
    })
}

#[cfg(unix)]
async fn handle_connection(stream: UnixStream, state: &mut RunnerState) -> Result<()> {
    let (reader, mut writer) = stream.into_split();
    let reader = BufReader::new(reader);
    let mut limited = reader.take((MAX_REQUEST_BYTES + 1) as u64);
    let mut bytes = Vec::new();
    let read = limited.read_until(b'\n', &mut bytes).await?;
    ensure!(read > 0, "empty runner request");
    ensure!(
        bytes.len() <= MAX_REQUEST_BYTES,
        "runner request exceeds one MiB"
    );
    let parsed = serde_json::from_slice::<RunnerWireRequest>(&bytes);
    let response = match parsed {
        Ok(request) => match state.apply(&request).await {
            Ok(response) => serde_json::to_value(response)?,
            Err(error) => failure_response(Some(&request), &error),
        },
        Err(error) => failure_response(None, &error.into()),
    };
    writer.write_all(&serde_json::to_vec(&response)?).await?;
    writer.write_all(b"\n").await?;
    writer.shutdown().await?;
    Ok(())
}

#[cfg(not(unix))]
fn main() {
    eprintln!(
        "focusa-session-runner: protected runner socket transport is unsupported on non-Unix platforms"
    );
    std::process::exit(78);
}

#[cfg(unix)]
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .json()
        .init();
    let args = Args::parse();
    let (actual_user, owner_uid) = current_user()?;
    ensure!(
        actual_user == args.owner_os_user,
        "runner must execute as the declared project owner"
    );
    ensure!(
        !args.principal_id.trim().is_empty(),
        "runner principal id is required"
    );
    ensure!(
        !args.socket_scope.trim().is_empty(),
        "runner socket scope is required"
    );
    prepare_socket(&args.socket, owner_uid)?;
    let socket_parent = fs::canonicalize(
        args.socket
            .parent()
            .context("runner socket requires a parent directory")?,
    )?;
    let nonce_parent = args
        .nonce_ledger
        .parent()
        .context("nonce ledger requires a parent directory")?;
    fs::create_dir_all(nonce_parent)?;
    ensure!(
        fs::canonicalize(nonce_parent)? == socket_parent,
        "nonce ledger must share the protected runner socket directory"
    );
    let secret_dir = canonical_owned_directory(
        args.secret_dir
            .to_str()
            .context("secret directory path is not UTF-8")?,
        owner_uid,
    )?;
    ensure!(
        fs::metadata(&secret_dir)?.permissions().mode() & 0o077 == 0,
        "secret directory permissions are too broad"
    );
    let key = read_secret(&args.key_file, owner_uid)?;
    let consumed_nonces = load_nonces(&args.nonce_ledger, owner_uid)?;
    let listener = UnixListener::bind(&args.socket)
        .with_context(|| format!("bind runner socket {}", args.socket.display()))?;
    fs::set_permissions(&args.socket, fs::Permissions::from_mode(0o600))?;
    ensure!(
        fs::metadata(&args.socket)?.uid() == owner_uid,
        "runner socket owner mismatch"
    );

    let mut state = RunnerState {
        identity: RunnerIdentity {
            principal_id: args.principal_id,
            os_user: args.owner_os_user,
            socket_scope: args.socket_scope,
        },
        owner_uid,
        key,
        nonce_ledger: args.nonce_ledger,
        secret_dir,
        consumed_nonces,
        processes: BTreeMap::new(),
    };
    info!(socket = %args.socket.display(), user = %state.identity.os_user, "session runner ready");
    loop {
        let (stream, _) = listener.accept().await?;
        if let Err(error) = handle_connection(stream, &mut state).await {
            error!(%error, "runner connection failed");
        }
    }
}
