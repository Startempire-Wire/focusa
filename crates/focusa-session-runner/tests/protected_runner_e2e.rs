#![cfg(unix)]

use chrono::{Duration, Utc};
use ed25519_dalek::SigningKey;
use focusa_core::silent_session::{SilentSessionId, SilentSessionRunId};
use focusa_session_runner::identity::{
    ExecutionIdentityRequest, ExecutionMode, OsIdentity, VerifiedExecutionContext,
};
use focusa_session_runner::process_posix::{PosixProcessSupervisor, PosixSpawnRequest};
use focusa_session_runner::protocol::{
    ActiveRunRecord, AdoptionExpectation, DaemonHandshakePolicy, ProtocolActor, ProtocolActorKind,
    ProtocolSigner, ProtocolVerifier, RUNNER_PROTOCOL_VERSION, RunnerCapability, RunnerHello,
    RunnerProtocolMessage,
};
use focusa_session_runner::transport::{AuthenticatedLocalStream, LocalSocketListener};
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::time::{Duration as TokioDuration, sleep};

static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(1);

struct RunnerFixture {
    root: PathBuf,
    workspace: PathBuf,
    socket_root: PathBuf,
    socket: PathBuf,
}

impl RunnerFixture {
    fn new() -> Self {
        let sequence = NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "focusa-protected-runner-e2e-{}-{sequence}",
            std::process::id()
        ));
        let workspace = root.join("worktree");
        // macOS Unix-domain socket paths are very short; keep the protected
        // endpoint independent from the intentionally descriptive project root.
        let socket_root =
            PathBuf::from("/tmp").join(format!("focusa-re2e-{}-{sequence}", std::process::id()));
        let socket = socket_root.join("r.sock");
        fs::create_dir_all(&workspace).expect("test workspace should be created");
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700))
            .expect("test root should be private");
        fs::set_permissions(&workspace, fs::Permissions::from_mode(0o700))
            .expect("test workspace should be private");
        Self {
            root,
            workspace,
            socket_root,
            socket,
        }
    }

    fn context(&self, current: &OsIdentity) -> VerifiedExecutionContext {
        VerifiedExecutionContext::verify(&ExecutionIdentityRequest {
            daemon_uid: current.uid,
            execution_user: current.user_name.clone(),
            execution_uid: current.uid,
            project_root: self.root.clone(),
            project_identity_ref: "project:protected-runner-e2e".into(),
            workspace_root: self.workspace.clone(),
        })
        .expect("same-user project context should verify")
    }
}

impl Drop for RunnerFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
        let _ = fs::remove_dir_all(&self.socket_root);
    }
}

fn actor(kind: ProtocolActorKind, actor_id: &str, current: &OsIdentity) -> ProtocolActor {
    ProtocolActor {
        kind,
        actor_id: actor_id.into(),
        os_user: current.user_name.clone(),
        uid: current.uid,
    }
}

fn capabilities() -> BTreeSet<RunnerCapability> {
    [
        RunnerCapability::AuthenticatedFrames,
        RunnerCapability::Heartbeat,
        RunnerCapability::OrphanAdoption,
        RunnerCapability::ProcessTreeIdentity,
        RunnerCapability::PerUserExecution,
        RunnerCapability::EmbeddedSameUser,
    ]
    .into_iter()
    .collect()
}

fn spawn_request() -> PosixSpawnRequest {
    PosixSpawnRequest {
        session_id: SilentSessionId::new(),
        run_id: SilentSessionRunId::new(),
        generation: 1,
        executable: PathBuf::from("/bin/sh"),
        argv: vec![
            OsString::from("-c"),
            OsString::from(
                "printf '%s\\n%s\\n' \"$(/usr/bin/id -u)\" \"$PWD\" > runner-owner-proof; /bin/sleep 30 & wait",
            ),
        ],
        env: BTreeMap::new(),
        launch_manifest_sha256: "b".repeat(64),
    }
}

fn adoption_expectation(record: &ActiveRunRecord) -> AdoptionExpectation {
    AdoptionExpectation {
        daemon_id: "daemon:e2e".into(),
        runner_id: record.runner_id.clone(),
        session_id: record.session_id,
        run_id: record.run_id,
        generation: record.generation,
        project_root: record.project_root.clone(),
        project_identity_ref: record.project_identity_ref.clone(),
        workspace_root: record.workspace_root.clone(),
        execution_user: record.execution_user.clone(),
        execution_uid: record.execution_uid,
        executable_ref: record.executable_ref.clone(),
        launch_manifest_sha256: record.launch_manifest_sha256.clone(),
        expected_process_instance_id: Some(record.process_tree.process_instance_id.clone()),
        heartbeat_fresh_after: Utc::now() - Duration::seconds(5),
    }
}

async fn wait_for_file(path: &Path) {
    for _ in 0..100 {
        if path.exists() {
            return;
        }
        sleep(TokioDuration::from_millis(10)).await;
    }
    panic!("runner process did not create {}", path.display());
}

#[tokio::test]
async fn protected_protocol_executes_and_adopts_same_user_owner_process() {
    let fixture = RunnerFixture::new();
    let current = OsIdentity::current().expect("current Unix identity should resolve");
    let context = fixture.context(&current);
    assert_eq!(context.mode(), ExecutionMode::EmbeddedSameUser);

    let mut supervisor = PosixProcessSupervisor::for_current_user("runner:e2e")
        .expect("same-user runner should initialize");
    let request = spawn_request();
    let run_id = request.run_id;
    let record = supervisor
        .spawn(&context, request, Utc::now())
        .expect("verified project owner should spawn");
    assert_eq!(
        record.process_tree.pid,
        record.process_tree.process_group_id as u32
    );
    assert_eq!(record.execution_uid, current.uid);

    let owner_proof = context
        .authorize_mutation_path("runner-owner-proof")
        .expect("owner proof should remain workspace-scoped");
    wait_for_file(owner_proof.as_path()).await;
    let proof = fs::read_to_string(owner_proof.as_path()).expect("owner proof should be readable");
    let mut lines = proof.lines();
    assert_eq!(lines.next(), Some(current.uid.to_string().as_str()));
    assert_eq!(lines.next(), context.workspace_root().to_str());
    assert_eq!(
        fs::metadata(owner_proof.as_path())
            .expect("owner proof metadata should exist")
            .uid(),
        current.uid
    );

    let listener =
        LocalSocketListener::bind(&fixture.socket, current.uid, BTreeSet::from([current.uid]))
            .await
            .expect("private runner socket should bind");
    let socket_path = listener.socket_path().to_path_buf();

    let daemon_actor = actor(ProtocolActorKind::Daemon, "daemon:e2e", &current);
    let runner_actor = actor(ProtocolActorKind::Runner, "runner:e2e", &current);
    let daemon_signer =
        ProtocolSigner::new(daemon_actor.clone(), SigningKey::from_bytes(&[41; 32]));
    let runner_signer =
        ProtocolSigner::new(runner_actor.clone(), SigningKey::from_bytes(&[42; 32]));
    let daemon_verifying_key = daemon_signer.verifying_key();
    let runner_verifying_key = runner_signer.verifying_key();
    let expected_record = record.clone();
    let expected_adoption = adoption_expectation(&record);

    let daemon = tokio::spawn(async move {
        let mut connection = listener
            .accept()
            .await
            .expect("runner peer UID should pass");
        let mut runner_verifier =
            ProtocolVerifier::new(runner_actor, "daemon:e2e", runner_verifying_key);

        let hello = connection
            .receive_authenticated(&mut runner_verifier, Utc::now())
            .await
            .expect("runner hello should authenticate");
        let RunnerProtocolMessage::RunnerHello(hello) = hello else {
            panic!("first runner frame must be hello");
        };
        assert_eq!(hello.active_runs, vec![expected_record]);
        let policy = DaemonHandshakePolicy {
            daemon_id: "daemon:e2e".into(),
            supported_protocol_versions: BTreeSet::from([RUNNER_PROTOCOL_VERSION]),
            required_capabilities: capabilities(),
        };
        let welcome = policy
            .negotiate(&hello, "challenge:daemon:e2e")
            .expect("runner capabilities should negotiate");
        let now = Utc::now();
        let welcome_frame = daemon_signer
            .sign(
                "runner:e2e",
                "nonce:daemon:welcome",
                now,
                now + Duration::seconds(30),
                RunnerProtocolMessage::DaemonWelcome(welcome),
            )
            .expect("daemon welcome should sign");
        connection
            .send_frame(&welcome_frame)
            .await
            .expect("daemon welcome should send");

        let heartbeat = connection
            .receive_authenticated(&mut runner_verifier, Utc::now())
            .await
            .expect("runner heartbeat should authenticate");
        let RunnerProtocolMessage::Heartbeat(heartbeat) = heartbeat else {
            panic!("second runner frame must be heartbeat");
        };
        assert_eq!(heartbeat.active_runs.len(), 1);

        let now = Utc::now();
        let adoption_frame = daemon_signer
            .sign(
                "runner:e2e",
                "nonce:daemon:adoption",
                now,
                now + Duration::seconds(30),
                RunnerProtocolMessage::AdoptionQuery(expected_adoption),
            )
            .expect("adoption query should sign");
        connection
            .send_frame(&adoption_frame)
            .await
            .expect("adoption query should send");

        let decision = connection
            .receive_authenticated(&mut runner_verifier, Utc::now())
            .await
            .expect("adoption decision should authenticate");
        let RunnerProtocolMessage::AdoptionDecision(decision) = decision else {
            panic!("third runner frame must be adoption decision");
        };
        decision
    });

    let mut runner_connection =
        AuthenticatedLocalStream::connect(socket_path, BTreeSet::from([current.uid]))
            .await
            .expect("daemon socket UID should pass");
    let hello = RunnerHello {
        runner_id: "runner:e2e".into(),
        os_user: current.user_name.clone(),
        uid: current.uid,
        supported_protocol_versions: BTreeSet::from([RUNNER_PROTOCOL_VERSION]),
        capabilities: capabilities(),
        active_runs: vec![record.clone()],
        runner_challenge_nonce: "challenge:runner:e2e".into(),
    };
    let now = Utc::now();
    let hello_frame = runner_signer
        .sign(
            "daemon:e2e",
            "nonce:runner:hello",
            now,
            now + Duration::seconds(30),
            RunnerProtocolMessage::RunnerHello(hello),
        )
        .expect("runner hello should sign");
    runner_connection
        .send_frame(&hello_frame)
        .await
        .expect("runner hello should send");

    let mut daemon_verifier =
        ProtocolVerifier::new(daemon_actor, "runner:e2e", daemon_verifying_key);
    let welcome = runner_connection
        .receive_authenticated(&mut daemon_verifier, Utc::now())
        .await
        .expect("daemon welcome should authenticate");
    let RunnerProtocolMessage::DaemonWelcome(welcome) = welcome else {
        panic!("daemon must answer hello with welcome");
    };
    assert_eq!(welcome.runner_challenge_nonce, "challenge:runner:e2e");
    assert_eq!(welcome.daemon_challenge_nonce, "challenge:daemon:e2e");

    let snapshot = supervisor
        .heartbeat(Utc::now())
        .expect("owned process should produce heartbeat");
    assert_eq!(snapshot.heartbeat.active_runs.len(), 1);
    let now = Utc::now();
    let heartbeat_frame = runner_signer
        .sign(
            "daemon:e2e",
            "nonce:runner:heartbeat",
            now,
            now + Duration::seconds(30),
            RunnerProtocolMessage::Heartbeat(snapshot.heartbeat),
        )
        .expect("runner heartbeat should sign");
    runner_connection
        .send_frame(&heartbeat_frame)
        .await
        .expect("runner heartbeat should send");

    let query = runner_connection
        .receive_authenticated(&mut daemon_verifier, Utc::now())
        .await
        .expect("daemon adoption query should authenticate");
    let RunnerProtocolMessage::AdoptionQuery(query) = query else {
        panic!("daemon must send adoption query");
    };
    let decision = supervisor
        .evaluate_adoption(&query, Utc::now())
        .expect("live process adoption should evaluate");
    assert!(decision.accepted);
    let now = Utc::now();
    let decision_frame = runner_signer
        .sign(
            "daemon:e2e",
            "nonce:runner:adoption-decision",
            now,
            now + Duration::seconds(30),
            RunnerProtocolMessage::AdoptionDecision(decision.clone()),
        )
        .expect("adoption decision should sign");
    runner_connection
        .send_frame(&decision_frame)
        .await
        .expect("adoption decision should send");

    let daemon_decision = daemon.await.expect("daemon task should finish");
    assert_eq!(daemon_decision, decision);
    assert!(daemon_decision.signed_runner_record_ref.is_some());

    supervisor
        .force_terminate(run_id, Utc::now())
        .await
        .expect("owned process tree should terminate");
    assert_eq!(supervisor.active_run_count(), 0);
}
