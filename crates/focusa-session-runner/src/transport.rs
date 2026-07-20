//! Protected Unix-domain transport for the signed runner protocol.
//!
//! The socket endpoint is owner-scoped (`0700` directory, `0600` socket),
//! refuses symlinks and occupied paths, authenticates kernel peer credentials,
//! and then verifies the signed, replay-protected protocol frame.

use crate::protocol::{ProtocolError, ProtocolVerifier, RunnerProtocolMessage, SignedRunnerFrame};
use chrono::{DateTime, Utc};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};

pub const MAX_RUNNER_FRAME_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PeerIdentity {
    pub uid: u32,
    pub gid: u32,
    pub pid: Option<i32>,
}

pub struct LocalSocketListener {
    listener: UnixListener,
    socket_path: PathBuf,
    socket_device: u64,
    socket_inode: u64,
    allowed_peer_uids: BTreeSet<u32>,
}

impl LocalSocketListener {
    pub async fn bind(
        requested_path: impl AsRef<Path>,
        owner_uid: u32,
        allowed_peer_uids: BTreeSet<u32>,
    ) -> Result<Self, TransportError> {
        if allowed_peer_uids.is_empty() {
            return Err(TransportError::NoAllowedPeers);
        }
        let socket_path = prepare_socket_path(requested_path.as_ref(), owner_uid)?;
        if fs::symlink_metadata(&socket_path).is_ok() {
            return Err(TransportError::SocketPathOccupied(socket_path));
        }

        let listener = UnixListener::bind(&socket_path).map_err(io_error)?;
        if let Err(error) = set_private_socket_permissions(&socket_path) {
            let _ = fs::remove_file(&socket_path);
            return Err(error);
        }
        let metadata = validate_socket_endpoint(&socket_path, &BTreeSet::from([owner_uid]))?;
        use std::os::unix::fs::MetadataExt;
        Ok(Self {
            listener,
            socket_path,
            socket_device: metadata.dev(),
            socket_inode: metadata.ino(),
            allowed_peer_uids,
        })
    }

    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    pub async fn accept(&self) -> Result<AuthenticatedLocalStream, TransportError> {
        let (stream, _) = self.listener.accept().await.map_err(io_error)?;
        AuthenticatedLocalStream::from_stream(stream, &self.allowed_peer_uids)
    }
}

impl Drop for LocalSocketListener {
    fn drop(&mut self) {
        use std::os::unix::fs::MetadataExt;

        let Ok(metadata) = fs::symlink_metadata(&self.socket_path) else {
            return;
        };
        if metadata.file_type().is_symlink()
            || metadata.dev() != self.socket_device
            || metadata.ino() != self.socket_inode
        {
            return;
        }
        let _ = fs::remove_file(&self.socket_path);
    }
}

pub struct AuthenticatedLocalStream {
    stream: UnixStream,
    peer: PeerIdentity,
}

impl AuthenticatedLocalStream {
    pub async fn connect(
        socket_path: impl AsRef<Path>,
        allowed_server_uids: BTreeSet<u32>,
    ) -> Result<Self, TransportError> {
        if allowed_server_uids.is_empty() {
            return Err(TransportError::NoAllowedPeers);
        }
        validate_socket_endpoint(socket_path.as_ref(), &allowed_server_uids)?;
        let stream = UnixStream::connect(socket_path.as_ref())
            .await
            .map_err(io_error)?;
        Self::from_stream(stream, &allowed_server_uids)
    }

    fn from_stream(
        stream: UnixStream,
        allowed_peer_uids: &BTreeSet<u32>,
    ) -> Result<Self, TransportError> {
        let credentials = stream.peer_cred().map_err(io_error)?;
        let peer = PeerIdentity {
            uid: credentials.uid(),
            gid: credentials.gid(),
            pid: credentials.pid(),
        };
        if !allowed_peer_uids.contains(&peer.uid) {
            return Err(TransportError::PeerUidDenied(peer.uid));
        }
        Ok(Self { stream, peer })
    }

    pub fn peer_identity(&self) -> PeerIdentity {
        self.peer
    }

    pub async fn send_frame(&mut self, frame: &SignedRunnerFrame) -> Result<(), TransportError> {
        let encoded = serde_json::to_vec(frame).map_err(|_| TransportError::InvalidJsonFrame)?;
        if encoded.len() > MAX_RUNNER_FRAME_BYTES {
            return Err(TransportError::FrameTooLarge(encoded.len()));
        }
        let length = u32::try_from(encoded.len())
            .map_err(|_| TransportError::FrameTooLarge(encoded.len()))?;
        self.stream
            .write_all(&length.to_be_bytes())
            .await
            .map_err(io_error)?;
        self.stream.write_all(&encoded).await.map_err(io_error)?;
        self.stream.flush().await.map_err(io_error)
    }

    pub async fn receive_frame(&mut self) -> Result<SignedRunnerFrame, TransportError> {
        let mut header = [0_u8; 4];
        self.stream
            .read_exact(&mut header)
            .await
            .map_err(io_error)?;
        let length = u32::from_be_bytes(header) as usize;
        if length == 0 || length > MAX_RUNNER_FRAME_BYTES {
            return Err(TransportError::FrameTooLarge(length));
        }
        let mut encoded = vec![0_u8; length];
        self.stream
            .read_exact(&mut encoded)
            .await
            .map_err(io_error)?;
        serde_json::from_slice(&encoded).map_err(|_| TransportError::InvalidJsonFrame)
    }

    pub async fn receive_authenticated(
        &mut self,
        verifier: &mut ProtocolVerifier,
        now: DateTime<Utc>,
    ) -> Result<RunnerProtocolMessage, TransportError> {
        let frame = self.receive_frame().await?;
        if frame.sender.uid != self.peer.uid {
            return Err(TransportError::PeerActorUidMismatch {
                peer_uid: self.peer.uid,
                actor_uid: frame.sender.uid,
            });
        }
        verifier
            .verify(&frame, now)
            .map_err(TransportError::ProtocolAuthentication)
    }
}

fn prepare_socket_path(path: &Path, owner_uid: u32) -> Result<PathBuf, TransportError> {
    use std::os::unix::fs::{DirBuilderExt, FileTypeExt, MetadataExt, PermissionsExt};

    let parent = path.parent().ok_or(TransportError::InvalidSocketPath)?;
    let file_name = path.file_name().ok_or(TransportError::InvalidSocketPath)?;
    if file_name.is_empty() {
        return Err(TransportError::InvalidSocketPath);
    }

    match fs::symlink_metadata(parent) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                return Err(TransportError::UnsafeSocketDirectory(parent.to_path_buf()));
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let mut builder = fs::DirBuilder::new();
            builder.mode(0o700);
            builder.create(parent).map_err(io_error)?;
        }
        Err(error) => return Err(io_error(error)),
    }

    let parent_metadata = fs::symlink_metadata(parent).map_err(io_error)?;
    if !parent_metadata.file_type().is_dir()
        || parent_metadata.file_type().is_symlink()
        || parent_metadata.uid() != owner_uid
        || parent_metadata.mode() & 0o077 != 0
    {
        return Err(TransportError::UnsafeSocketDirectory(parent.to_path_buf()));
    }

    let canonical_parent = fs::canonicalize(parent).map_err(io_error)?;
    let socket_path = canonical_parent.join(file_name);
    if let Ok(metadata) = fs::symlink_metadata(&socket_path) {
        if metadata.file_type().is_symlink() {
            return Err(TransportError::SocketSymlinkRejected(socket_path));
        }
        if metadata.file_type().is_socket() {
            return Err(TransportError::SocketPathOccupied(socket_path));
        }
        return Err(TransportError::SocketPathOccupied(socket_path));
    }
    Ok(socket_path)
}

fn set_private_socket_permissions(path: &Path) -> Result<(), TransportError> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).map_err(io_error)
}

fn validate_socket_endpoint(
    path: &Path,
    allowed_owner_uids: &BTreeSet<u32>,
) -> Result<fs::Metadata, TransportError> {
    use std::os::unix::fs::{FileTypeExt, MetadataExt};

    if allowed_owner_uids.is_empty() {
        return Err(TransportError::NoAllowedPeers);
    }
    let metadata = fs::symlink_metadata(path).map_err(io_error)?;
    if metadata.file_type().is_symlink() {
        return Err(TransportError::SocketSymlinkRejected(path.to_path_buf()));
    }
    if !metadata.file_type().is_socket()
        || !allowed_owner_uids.contains(&metadata.uid())
        || metadata.mode() & 0o077 != 0
    {
        return Err(TransportError::UnsafeSocketEndpoint(path.to_path_buf()));
    }
    Ok(metadata)
}

fn io_error(error: std::io::Error) -> TransportError {
    TransportError::Io(error.to_string())
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TransportError {
    #[error("runner socket path is invalid")]
    InvalidSocketPath,
    #[error("runner socket requires at least one allowed peer UID")]
    NoAllowedPeers,
    #[error("runner socket directory is not private and owner-scoped: {0}")]
    UnsafeSocketDirectory(PathBuf),
    #[error("runner socket endpoint is not private and owner-scoped: {0}")]
    UnsafeSocketEndpoint(PathBuf),
    #[error("runner socket path is already occupied: {0}")]
    SocketPathOccupied(PathBuf),
    #[error("runner socket symlink is forbidden: {0}")]
    SocketSymlinkRejected(PathBuf),
    #[error("runner socket peer UID is not authorized: {0}")]
    PeerUidDenied(u32),
    #[error("authenticated runner actor UID {actor_uid} does not match kernel peer UID {peer_uid}")]
    PeerActorUidMismatch { peer_uid: u32, actor_uid: u32 },
    #[error("runner protocol frame is too large: {0} bytes")]
    FrameTooLarge(usize),
    #[error("runner protocol frame is not valid JSON")]
    InvalidJsonFrame,
    #[error("runner protocol authentication failed: {0}")]
    ProtocolAuthentication(ProtocolError),
    #[error("runner socket I/O failed: {0}")]
    Io(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::OsIdentity;
    use crate::protocol::{
        ProcessTreeIdentity, ProtocolActor, ProtocolActorKind, ProtocolSigner, RunnerHeartbeat,
        RunnerProtocolMessage,
    };
    use chrono::Duration;
    use ed25519_dalek::SigningKey;
    use focusa_core::silent_session::{SilentSessionId, SilentSessionRunId};
    use std::os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt, symlink};
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_SOCKET_ID: AtomicU64 = AtomicU64::new(1);

    struct SocketFixture {
        root: PathBuf,
        path: PathBuf,
    }

    impl SocketFixture {
        fn new() -> Self {
            let sequence = NEXT_SOCKET_ID.fetch_add(1, Ordering::Relaxed);
            let root =
                std::env::temp_dir().join(format!("focusa-sock-{}-{sequence}", std::process::id()));
            let path = root.join("r.sock");
            Self { root, path }
        }
    }

    impl Drop for SocketFixture {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.path);
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn runner_actor(current: &OsIdentity) -> ProtocolActor {
        ProtocolActor {
            kind: ProtocolActorKind::Runner,
            actor_id: "runner:test".into(),
            os_user: current.user_name.clone(),
            uid: current.uid,
        }
    }

    fn signed_heartbeat(
        signer: &ProtocolSigner,
        current: &OsIdentity,
        now: DateTime<Utc>,
    ) -> SignedRunnerFrame {
        let session_id = SilentSessionId::new();
        let run_id = SilentSessionRunId::new();
        let heartbeat = RunnerHeartbeat {
            runner_id: "runner:test".into(),
            os_user: current.user_name.clone(),
            uid: current.uid,
            sequence: 1,
            observed_at: now,
            active_runs: vec![crate::protocol::ActiveRunRecord {
                runner_id: "runner:test".into(),
                session_id,
                run_id,
                generation: 1,
                project_root: PathBuf::from("/project"),
                project_identity_ref: "project:test".into(),
                workspace_root: PathBuf::from("/workspace"),
                execution_user: current.user_name.clone(),
                execution_uid: current.uid,
                executable_ref: "/bin/test-agent".into(),
                launch_manifest_sha256: "manifest".into(),
                process_tree: ProcessTreeIdentity {
                    process_instance_id: "process:test".into(),
                    runner_id: "runner:test".into(),
                    session_id,
                    run_id,
                    generation: 1,
                    pid: std::process::id(),
                    process_group_id: i64::from(std::process::id()),
                    os_session_id: i64::from(std::process::id()),
                    execution_user: current.user_name.clone(),
                    execution_uid: current.uid,
                    executable_ref: "/bin/test-agent".into(),
                    spawned_at: now,
                },
                heartbeat_at: now,
            }],
        };
        signer
            .sign(
                "daemon:test",
                "nonce:transport",
                now,
                now + Duration::seconds(30),
                RunnerProtocolMessage::Heartbeat(heartbeat),
            )
            .expect("heartbeat frame should sign")
    }

    #[tokio::test]
    async fn socket_endpoint_is_private_and_guarded_by_inode() {
        let fixture = SocketFixture::new();
        let current = OsIdentity::current().expect("current user should resolve");
        let canonical_socket_path;
        {
            let listener = LocalSocketListener::bind(
                &fixture.path,
                current.uid,
                BTreeSet::from([current.uid]),
            )
            .await
            .expect("private socket should bind");
            canonical_socket_path = listener.socket_path().to_path_buf();
            let parent = fs::metadata(
                canonical_socket_path
                    .parent()
                    .expect("socket should have parent"),
            )
            .expect("socket parent should exist");
            let endpoint =
                fs::symlink_metadata(&canonical_socket_path).expect("socket endpoint should exist");
            assert_eq!(parent.mode() & 0o777, 0o700);
            assert_eq!(endpoint.mode() & 0o777, 0o600);
            assert_eq!(endpoint.uid(), current.uid);
            assert!(endpoint.file_type().is_socket());
        }
        assert!(!canonical_socket_path.exists());
    }

    #[tokio::test]
    async fn signed_frame_crosses_real_socket_and_authenticates() {
        let fixture = SocketFixture::new();
        let current = OsIdentity::current().expect("current user should resolve");
        let listener =
            LocalSocketListener::bind(&fixture.path, current.uid, BTreeSet::from([current.uid]))
                .await
                .expect("private socket should bind");
        let socket_path = listener.socket_path().to_path_buf();
        let actor = runner_actor(&current);
        let signer = ProtocolSigner::new(actor.clone(), SigningKey::from_bytes(&[31; 32]));
        let frame = signed_heartbeat(&signer, &current, Utc::now());
        let verifying_key = signer.verifying_key();
        let server = tokio::spawn(async move {
            let mut connection = listener.accept().await.expect("peer UID should pass");
            let mut verifier = ProtocolVerifier::new(actor, "daemon:test", verifying_key);
            connection
                .receive_authenticated(&mut verifier, Utc::now())
                .await
        });

        let mut client =
            AuthenticatedLocalStream::connect(socket_path, BTreeSet::from([current.uid]))
                .await
                .expect("server UID should pass");
        assert_eq!(client.peer_identity().uid, current.uid);
        client
            .send_frame(&frame)
            .await
            .expect("signed frame should send");
        assert!(matches!(
            server.await.expect("server task should finish"),
            Ok(RunnerProtocolMessage::Heartbeat(_))
        ));
    }

    #[tokio::test]
    async fn tampered_frame_is_rejected_after_kernel_peer_check() {
        let fixture = SocketFixture::new();
        let current = OsIdentity::current().expect("current user should resolve");
        let listener =
            LocalSocketListener::bind(&fixture.path, current.uid, BTreeSet::from([current.uid]))
                .await
                .expect("private socket should bind");
        let socket_path = listener.socket_path().to_path_buf();
        let actor = runner_actor(&current);
        let signer = ProtocolSigner::new(actor.clone(), SigningKey::from_bytes(&[32; 32]));
        let mut frame = signed_heartbeat(&signer, &current, Utc::now());
        frame.payload_sha256 = "tampered".into();
        let verifying_key = signer.verifying_key();
        let server = tokio::spawn(async move {
            let mut connection = listener.accept().await.expect("peer UID should pass");
            let mut verifier = ProtocolVerifier::new(actor, "daemon:test", verifying_key);
            connection
                .receive_authenticated(&mut verifier, Utc::now())
                .await
        });

        let mut client =
            AuthenticatedLocalStream::connect(socket_path, BTreeSet::from([current.uid]))
                .await
                .expect("server UID should pass");
        client
            .send_frame(&frame)
            .await
            .expect("tampered frame should reach verifier");
        assert_eq!(
            server.await.expect("server task should finish"),
            Err(TransportError::ProtocolAuthentication(
                ProtocolError::PayloadDigestMismatch
            ))
        );
    }

    #[tokio::test]
    async fn signed_actor_uid_must_match_kernel_peer_uid() {
        let fixture = SocketFixture::new();
        let current = OsIdentity::current().expect("current user should resolve");
        let listener =
            LocalSocketListener::bind(&fixture.path, current.uid, BTreeSet::from([current.uid]))
                .await
                .expect("private socket should bind");
        let socket_path = listener.socket_path().to_path_buf();
        let mut forged_actor = runner_actor(&current);
        forged_actor.uid = current.uid.wrapping_add(1);
        let signer = ProtocolSigner::new(forged_actor.clone(), SigningKey::from_bytes(&[33; 32]));
        let frame = signer
            .sign(
                "daemon:test",
                "nonce:wrong-peer-uid",
                Utc::now(),
                Utc::now() + Duration::seconds(30),
                RunnerProtocolMessage::Heartbeat(RunnerHeartbeat {
                    runner_id: forged_actor.actor_id.clone(),
                    os_user: forged_actor.os_user.clone(),
                    uid: forged_actor.uid,
                    sequence: 1,
                    observed_at: Utc::now(),
                    active_runs: vec![],
                }),
            )
            .expect("internally consistent forged actor should sign");
        let verifying_key = signer.verifying_key();
        let server = tokio::spawn(async move {
            let mut connection = listener.accept().await.expect("peer UID should pass");
            let mut verifier = ProtocolVerifier::new(forged_actor, "daemon:test", verifying_key);
            connection
                .receive_authenticated(&mut verifier, Utc::now())
                .await
        });

        let mut client =
            AuthenticatedLocalStream::connect(socket_path, BTreeSet::from([current.uid]))
                .await
                .expect("server UID should pass");
        client
            .send_frame(&frame)
            .await
            .expect("forged frame should reach transport binding check");
        assert_eq!(
            server.await.expect("server task should finish"),
            Err(TransportError::PeerActorUidMismatch {
                peer_uid: current.uid,
                actor_uid: current.uid.wrapping_add(1),
            })
        );
    }

    #[tokio::test]
    async fn kernel_peer_uid_allowlist_fails_closed() {
        let fixture = SocketFixture::new();
        let current = OsIdentity::current().expect("current user should resolve");
        let listener = LocalSocketListener::bind(
            &fixture.path,
            current.uid,
            BTreeSet::from([current.uid.wrapping_add(1)]),
        )
        .await
        .expect("listener policy should bind");
        let socket_path = listener.socket_path().to_path_buf();
        let server = tokio::spawn(async move { listener.accept().await.map(|_| ()) });
        let _client = UnixStream::connect(socket_path)
            .await
            .expect("kernel should connect local peer");
        assert_eq!(
            server.await.expect("server task should finish"),
            Err(TransportError::PeerUidDenied(current.uid))
        );
    }

    #[tokio::test]
    async fn oversized_frame_is_rejected_before_allocation() {
        let fixture = SocketFixture::new();
        let current = OsIdentity::current().expect("current user should resolve");
        let listener =
            LocalSocketListener::bind(&fixture.path, current.uid, BTreeSet::from([current.uid]))
                .await
                .expect("private socket should bind");
        let socket_path = listener.socket_path().to_path_buf();
        let server = tokio::spawn(async move {
            let mut connection = listener.accept().await.expect("peer UID should pass");
            connection.receive_frame().await
        });
        let mut client = UnixStream::connect(socket_path)
            .await
            .expect("client should connect");
        let oversized =
            u32::try_from(MAX_RUNNER_FRAME_BYTES + 1).expect("test frame limit should fit u32");
        client
            .write_all(&oversized.to_be_bytes())
            .await
            .expect("header should send");
        assert_eq!(
            server.await.expect("server task should finish"),
            Err(TransportError::FrameTooLarge(MAX_RUNNER_FRAME_BYTES + 1))
        );
    }

    #[tokio::test]
    async fn socket_symlink_and_shared_directory_are_rejected() {
        let fixture = SocketFixture::new();
        let current = OsIdentity::current().expect("current user should resolve");
        fs::create_dir(&fixture.root).expect("fixture root should be created");
        fs::set_permissions(&fixture.root, fs::Permissions::from_mode(0o777))
            .expect("fixture should become shared writable");
        assert!(matches!(
            LocalSocketListener::bind(&fixture.path, current.uid, BTreeSet::from([current.uid]))
                .await,
            Err(TransportError::UnsafeSocketDirectory(_))
        ));

        fs::set_permissions(&fixture.root, fs::Permissions::from_mode(0o700))
            .expect("fixture should become private");
        let target = fixture.root.join("target");
        fs::write(&target, b"not a socket").expect("target should be created");
        symlink(&target, &fixture.path).expect("socket symlink should be created");
        assert!(matches!(
            LocalSocketListener::bind(&fixture.path, current.uid, BTreeSet::from([current.uid]))
                .await,
            Err(TransportError::SocketSymlinkRejected(_))
        ));
    }
}
