//! Versioned, mutually authenticated daemon/runner protocol primitives.
//!
//! Every frame is signed, short-lived, exactly addressed, and replay protected.
//! Runner heartbeats carry signed process-tree records that can be adopted only
//! after an exact scope and launch-manifest match.

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use chrono::{DateTime, Duration, Utc};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use focusa_core::silent_session::{
    SilentSessionId, SilentSessionLifecycleState, SilentSessionRunId,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use thiserror::Error;

pub const RUNNER_PROTOCOL_SCHEMA: &str = "focusa.daemon_runner_frame.v1";
pub const RUNNER_PROTOCOL_VERSION: u32 = 1;
pub const DEFAULT_MAX_FRAME_TTL_SECONDS: i64 = 60;
pub const DEFAULT_MAX_CLOCK_SKEW_SECONDS: i64 = 5;
pub const DEFAULT_REPLAY_CAPACITY: usize = 4_096;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtocolActorKind {
    Daemon,
    Runner,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolActor {
    pub kind: ProtocolActorKind,
    pub actor_id: String,
    pub os_user: String,
    pub uid: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunnerCapability {
    AuthenticatedFrames,
    Heartbeat,
    OrphanAdoption,
    ProcessTreeIdentity,
    PerUserExecution,
    EmbeddedSameUser,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunnerHello {
    pub runner_id: String,
    pub os_user: String,
    pub uid: u32,
    pub supported_protocol_versions: BTreeSet<u32>,
    pub capabilities: BTreeSet<RunnerCapability>,
    pub active_runs: Vec<ActiveRunRecord>,
    pub runner_challenge_nonce: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DaemonWelcome {
    pub daemon_id: String,
    pub runner_id: String,
    pub selected_protocol_version: u32,
    pub required_capabilities: BTreeSet<RunnerCapability>,
    pub runner_challenge_nonce: String,
    pub daemon_challenge_nonce: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunnerHeartbeat {
    pub runner_id: String,
    pub os_user: String,
    pub uid: u32,
    pub sequence: u64,
    pub observed_at: DateTime<Utc>,
    pub active_runs: Vec<ActiveRunRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessTreeIdentity {
    /// Random identity assigned once at spawn; prevents PID reuse from matching.
    pub process_instance_id: String,
    pub runner_id: String,
    pub session_id: SilentSessionId,
    pub run_id: SilentSessionRunId,
    pub generation: u64,
    pub pid: u32,
    pub process_group_id: i64,
    pub os_session_id: i64,
    pub execution_user: String,
    pub execution_uid: u32,
    pub executable_ref: String,
    pub spawned_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveRunRecord {
    pub runner_id: String,
    pub session_id: SilentSessionId,
    pub run_id: SilentSessionRunId,
    pub generation: u64,
    pub project_root: PathBuf,
    pub project_identity_ref: String,
    pub workspace_root: PathBuf,
    pub execution_user: String,
    pub execution_uid: u32,
    pub executable_ref: String,
    pub launch_manifest_sha256: String,
    pub process_tree: ProcessTreeIdentity,
    pub heartbeat_at: DateTime<Utc>,
}

impl ActiveRunRecord {
    pub fn signed_record_ref(&self) -> Result<String, ProtocolError> {
        let encoded = serde_json::to_vec(self).map_err(|_| ProtocolError::InvalidFrame)?;
        Ok(format!("sha256:{}", hex_digest(&encoded)))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdoptionExpectation {
    pub daemon_id: String,
    pub runner_id: String,
    pub session_id: SilentSessionId,
    pub run_id: SilentSessionRunId,
    pub generation: u64,
    pub project_root: PathBuf,
    pub project_identity_ref: String,
    pub workspace_root: PathBuf,
    pub execution_user: String,
    pub execution_uid: u32,
    pub executable_ref: String,
    pub launch_manifest_sha256: String,
    pub expected_process_instance_id: Option<String>,
    pub heartbeat_fresh_after: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdoptionRejection {
    RunnerMismatch,
    SessionMismatch,
    RunMismatch,
    GenerationMismatch,
    ProjectMismatch,
    WorkspaceMismatch,
    UserMismatch,
    ExecutableMismatch,
    ManifestMismatch,
    ProcessIdentityMismatch,
    ProcessNotOwned,
    StaleHeartbeat,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdoptionDecision {
    pub runner_id: String,
    pub session_id: SilentSessionId,
    pub run_id: SilentSessionRunId,
    pub generation: u64,
    pub accepted: bool,
    pub rejection: Option<AdoptionRejection>,
    pub signed_runner_record_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrphanReconciliationRequest {
    pub expectation: AdoptionExpectation,
    pub expected_stream_cursor: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrphanReconciliationStatus {
    AdoptedRecovering,
    RejectedOrphaned,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrphanReconciliationDecision {
    pub status: OrphanReconciliationStatus,
    pub lifecycle_state: SilentSessionLifecycleState,
    pub adoption: AdoptionDecision,
    pub restored_stream_cursor: Option<String>,
}

impl OrphanReconciliationRequest {
    pub fn reconcile(
        &self,
        adoption: AdoptionDecision,
    ) -> Result<OrphanReconciliationDecision, ProtocolError> {
        if self.expected_stream_cursor.trim().is_empty()
            || adoption.runner_id != self.expectation.runner_id
            || adoption.session_id != self.expectation.session_id
            || adoption.run_id != self.expectation.run_id
            || adoption.generation != self.expectation.generation
        {
            return Err(ProtocolError::InvalidFrame);
        }
        let accepted = adoption.accepted
            && adoption.rejection.is_none()
            && adoption.signed_runner_record_ref.is_some();
        Ok(OrphanReconciliationDecision {
            status: if accepted {
                OrphanReconciliationStatus::AdoptedRecovering
            } else {
                OrphanReconciliationStatus::RejectedOrphaned
            },
            lifecycle_state: if accepted {
                SilentSessionLifecycleState::Recovering
            } else {
                SilentSessionLifecycleState::Orphaned
            },
            adoption,
            restored_stream_cursor: if accepted {
                Some(self.expected_stream_cursor.clone())
            } else {
                None
            },
        })
    }
}

impl AdoptionExpectation {
    pub fn rejected(&self, rejection: AdoptionRejection) -> AdoptionDecision {
        AdoptionDecision {
            runner_id: self.runner_id.clone(),
            session_id: self.session_id,
            run_id: self.run_id,
            generation: self.generation,
            accepted: false,
            rejection: Some(rejection),
            signed_runner_record_ref: None,
        }
    }

    /// Compare every authority-bearing field required by Spec 133 orphan adoption.
    pub fn evaluate(&self, record: &ActiveRunRecord) -> Result<AdoptionDecision, ProtocolError> {
        let rejection = if self.runner_id != record.runner_id
            || record.process_tree.runner_id != record.runner_id
        {
            Some(AdoptionRejection::RunnerMismatch)
        } else if self.session_id != record.session_id
            || record.process_tree.session_id != record.session_id
        {
            Some(AdoptionRejection::SessionMismatch)
        } else if self.run_id != record.run_id || record.process_tree.run_id != record.run_id {
            Some(AdoptionRejection::RunMismatch)
        } else if self.generation != record.generation
            || record.process_tree.generation != record.generation
        {
            Some(AdoptionRejection::GenerationMismatch)
        } else if self.project_root != record.project_root
            || self.project_identity_ref != record.project_identity_ref
        {
            Some(AdoptionRejection::ProjectMismatch)
        } else if self.workspace_root != record.workspace_root {
            Some(AdoptionRejection::WorkspaceMismatch)
        } else if self.execution_user != record.execution_user
            || self.execution_uid != record.execution_uid
            || record.process_tree.execution_user != record.execution_user
            || record.process_tree.execution_uid != record.execution_uid
        {
            Some(AdoptionRejection::UserMismatch)
        } else if self.executable_ref != record.executable_ref
            || record.process_tree.executable_ref != record.executable_ref
        {
            Some(AdoptionRejection::ExecutableMismatch)
        } else if self.launch_manifest_sha256 != record.launch_manifest_sha256 {
            Some(AdoptionRejection::ManifestMismatch)
        } else if self
            .expected_process_instance_id
            .as_deref()
            .is_some_and(|expected| expected != record.process_tree.process_instance_id)
        {
            Some(AdoptionRejection::ProcessIdentityMismatch)
        } else if record.heartbeat_at < self.heartbeat_fresh_after {
            Some(AdoptionRejection::StaleHeartbeat)
        } else {
            None
        };

        Ok(AdoptionDecision {
            runner_id: record.runner_id.clone(),
            session_id: record.session_id,
            run_id: record.run_id,
            generation: record.generation,
            accepted: rejection.is_none(),
            signed_runner_record_ref: if rejection.is_none() {
                Some(record.signed_record_ref()?)
            } else {
                None
            },
            rejection,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "body", rename_all = "snake_case")]
pub enum RunnerProtocolMessage {
    RunnerHello(RunnerHello),
    DaemonWelcome(DaemonWelcome),
    Heartbeat(RunnerHeartbeat),
    AdoptionQuery(AdoptionExpectation),
    AdoptionDecision(AdoptionDecision),
    OrphanReconciliationQuery(OrphanReconciliationRequest),
    OrphanReconciliationDecision(OrphanReconciliationDecision),
}

impl RunnerProtocolMessage {
    fn validate_sender(&self, sender: &ProtocolActor, receiver_id: &str) -> bool {
        match self {
            Self::RunnerHello(hello) => {
                sender.kind == ProtocolActorKind::Runner
                    && hello.runner_id == sender.actor_id
                    && hello.os_user == sender.os_user
                    && hello.uid == sender.uid
                    && !hello.runner_challenge_nonce.is_empty()
            }
            Self::DaemonWelcome(welcome) => {
                sender.kind == ProtocolActorKind::Daemon
                    && welcome.daemon_id == sender.actor_id
                    && welcome.runner_id == receiver_id
                    && !welcome.runner_challenge_nonce.is_empty()
                    && !welcome.daemon_challenge_nonce.is_empty()
            }
            Self::Heartbeat(heartbeat) => {
                sender.kind == ProtocolActorKind::Runner
                    && heartbeat.runner_id == sender.actor_id
                    && heartbeat.os_user == sender.os_user
                    && heartbeat.uid == sender.uid
                    && heartbeat
                        .active_runs
                        .iter()
                        .all(|record| record.runner_id == sender.actor_id)
            }
            Self::AdoptionQuery(expectation) => {
                sender.kind == ProtocolActorKind::Daemon
                    && expectation.daemon_id == sender.actor_id
                    && expectation.runner_id == receiver_id
            }
            Self::AdoptionDecision(decision) => {
                sender.kind == ProtocolActorKind::Runner && decision.runner_id == sender.actor_id
            }
            Self::OrphanReconciliationQuery(request) => {
                sender.kind == ProtocolActorKind::Daemon
                    && request.expectation.daemon_id == sender.actor_id
                    && request.expectation.runner_id == receiver_id
            }
            Self::OrphanReconciliationDecision(decision) => {
                sender.kind == ProtocolActorKind::Runner
                    && decision.adoption.runner_id == sender.actor_id
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedRunnerFrame {
    pub schema: String,
    pub protocol_version: u32,
    pub sender: ProtocolActor,
    pub receiver_id: String,
    pub nonce: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub payload_sha256: String,
    pub payload: RunnerProtocolMessage,
    pub signature_base64: String,
}

#[derive(Serialize)]
struct UnsignedRunnerFrame<'a> {
    schema: &'a str,
    protocol_version: u32,
    sender: &'a ProtocolActor,
    receiver_id: &'a str,
    nonce: &'a str,
    issued_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    payload_sha256: &'a str,
    payload: &'a RunnerProtocolMessage,
}

impl SignedRunnerFrame {
    fn signing_bytes(&self) -> Result<Vec<u8>, ProtocolError> {
        serde_json::to_vec(&UnsignedRunnerFrame {
            schema: &self.schema,
            protocol_version: self.protocol_version,
            sender: &self.sender,
            receiver_id: &self.receiver_id,
            nonce: &self.nonce,
            issued_at: self.issued_at,
            expires_at: self.expires_at,
            payload_sha256: &self.payload_sha256,
            payload: &self.payload,
        })
        .map_err(|_| ProtocolError::InvalidFrame)
    }
}

pub struct ProtocolSigner {
    actor: ProtocolActor,
    signing_key: SigningKey,
}

impl ProtocolSigner {
    pub fn new(actor: ProtocolActor, signing_key: SigningKey) -> Self {
        Self { actor, signing_key }
    }

    pub fn verifying_key(&self) -> VerifyingKey {
        self.signing_key.verifying_key()
    }

    pub fn sign(
        &self,
        receiver_id: impl Into<String>,
        nonce: impl Into<String>,
        issued_at: DateTime<Utc>,
        expires_at: DateTime<Utc>,
        payload: RunnerProtocolMessage,
    ) -> Result<SignedRunnerFrame, ProtocolError> {
        let receiver_id = receiver_id.into();
        let nonce = nonce.into();
        if receiver_id.trim().is_empty()
            || nonce.trim().is_empty()
            || expires_at <= issued_at
            || !payload.validate_sender(&self.actor, &receiver_id)
        {
            return Err(ProtocolError::InvalidFrame);
        }
        let payload_bytes =
            serde_json::to_vec(&payload).map_err(|_| ProtocolError::InvalidFrame)?;
        let mut frame = SignedRunnerFrame {
            schema: RUNNER_PROTOCOL_SCHEMA.to_owned(),
            protocol_version: RUNNER_PROTOCOL_VERSION,
            sender: self.actor.clone(),
            receiver_id,
            nonce,
            issued_at,
            expires_at,
            payload_sha256: hex_digest(&payload_bytes),
            payload,
            signature_base64: String::new(),
        };
        frame.signature_base64 =
            BASE64.encode(self.signing_key.sign(&frame.signing_bytes()?).to_bytes());
        Ok(frame)
    }
}

#[derive(Debug)]
pub struct ProtocolVerifier {
    expected_sender: ProtocolActor,
    receiver_id: String,
    verifying_key: VerifyingKey,
    max_frame_ttl: Duration,
    max_clock_skew: Duration,
    seen_nonces: BTreeMap<(String, String), DateTime<Utc>>,
    replay_capacity: usize,
}

impl ProtocolVerifier {
    pub fn new(
        expected_sender: ProtocolActor,
        receiver_id: impl Into<String>,
        verifying_key: VerifyingKey,
    ) -> Self {
        Self {
            expected_sender,
            receiver_id: receiver_id.into(),
            verifying_key,
            max_frame_ttl: Duration::seconds(DEFAULT_MAX_FRAME_TTL_SECONDS),
            max_clock_skew: Duration::seconds(DEFAULT_MAX_CLOCK_SKEW_SECONDS),
            seen_nonces: BTreeMap::new(),
            replay_capacity: DEFAULT_REPLAY_CAPACITY,
        }
    }

    pub fn verify(
        &mut self,
        frame: &SignedRunnerFrame,
        now: DateTime<Utc>,
    ) -> Result<RunnerProtocolMessage, ProtocolError> {
        if frame.schema != RUNNER_PROTOCOL_SCHEMA
            || frame.protocol_version != RUNNER_PROTOCOL_VERSION
            || frame.sender != self.expected_sender
            || frame.receiver_id != self.receiver_id
            || frame.nonce.trim().is_empty()
            || frame.expires_at <= frame.issued_at
        {
            return Err(ProtocolError::InvalidFrame);
        }
        if frame.issued_at > now + self.max_clock_skew {
            return Err(ProtocolError::IssuedInFuture);
        }
        if frame.expires_at <= now {
            return Err(ProtocolError::Expired);
        }
        if frame.expires_at - frame.issued_at > self.max_frame_ttl {
            return Err(ProtocolError::TtlExceeded);
        }

        let payload_bytes =
            serde_json::to_vec(&frame.payload).map_err(|_| ProtocolError::InvalidFrame)?;
        if frame.payload_sha256 != hex_digest(&payload_bytes) {
            return Err(ProtocolError::PayloadDigestMismatch);
        }
        let raw_signature = BASE64
            .decode(&frame.signature_base64)
            .map_err(|_| ProtocolError::SignatureInvalid)?;
        let signature =
            Signature::from_slice(&raw_signature).map_err(|_| ProtocolError::SignatureInvalid)?;
        self.verifying_key
            .verify(&frame.signing_bytes()?, &signature)
            .map_err(|_| ProtocolError::SignatureInvalid)?;
        if !frame
            .payload
            .validate_sender(&frame.sender, &frame.receiver_id)
        {
            return Err(ProtocolError::ActorBindingMismatch);
        }

        self.seen_nonces.retain(|_, expiry| *expiry > now);
        let replay_key = (frame.sender.actor_id.clone(), frame.nonce.clone());
        if self.seen_nonces.contains_key(&replay_key) {
            return Err(ProtocolError::ReplayDetected);
        }
        if self.seen_nonces.len() >= self.replay_capacity {
            let oldest = self
                .seen_nonces
                .iter()
                .min_by_key(|(_, expiry)| **expiry)
                .map(|(key, _)| key.clone());
            if let Some(oldest) = oldest {
                self.seen_nonces.remove(&oldest);
            }
        }
        self.seen_nonces.insert(replay_key, frame.expires_at);
        Ok(frame.payload.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DaemonHandshakePolicy {
    pub daemon_id: String,
    pub supported_protocol_versions: BTreeSet<u32>,
    pub required_capabilities: BTreeSet<RunnerCapability>,
}

impl DaemonHandshakePolicy {
    pub fn negotiate(
        &self,
        hello: &RunnerHello,
        daemon_challenge_nonce: impl Into<String>,
    ) -> Result<DaemonWelcome, ProtocolError> {
        let selected_protocol_version = self
            .supported_protocol_versions
            .intersection(&hello.supported_protocol_versions)
            .max()
            .copied()
            .ok_or(ProtocolError::ProtocolIncompatible)?;
        if !self.required_capabilities.is_subset(&hello.capabilities) {
            return Err(ProtocolError::RequiredCapabilityMissing);
        }
        let daemon_challenge_nonce = daemon_challenge_nonce.into();
        if daemon_challenge_nonce.trim().is_empty()
            || hello.runner_challenge_nonce.trim().is_empty()
        {
            return Err(ProtocolError::InvalidFrame);
        }
        Ok(DaemonWelcome {
            daemon_id: self.daemon_id.clone(),
            runner_id: hello.runner_id.clone(),
            selected_protocol_version,
            required_capabilities: self.required_capabilities.clone(),
            runner_challenge_nonce: hello.runner_challenge_nonce.clone(),
            daemon_challenge_nonce,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ProtocolError {
    #[error("runner protocol frame is malformed or addressed incorrectly")]
    InvalidFrame,
    #[error("runner protocol frame was issued too far in the future")]
    IssuedInFuture,
    #[error("runner protocol frame has expired")]
    Expired,
    #[error("runner protocol frame lifetime exceeds policy")]
    TtlExceeded,
    #[error("runner protocol payload digest does not match")]
    PayloadDigestMismatch,
    #[error("runner protocol signature is invalid")]
    SignatureInvalid,
    #[error("runner protocol payload does not match the authenticated actor")]
    ActorBindingMismatch,
    #[error("runner protocol nonce was already consumed")]
    ReplayDetected,
    #[error("daemon and runner have no compatible protocol version")]
    ProtocolIncompatible,
    #[error("runner is missing a required protocol capability")]
    RequiredCapabilityMissing,
}

fn hex_digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn actor(kind: ProtocolActorKind, actor_id: &str, user: &str, uid: u32) -> ProtocolActor {
        ProtocolActor {
            kind,
            actor_id: actor_id.to_owned(),
            os_user: user.to_owned(),
            uid,
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

    fn active_record(now: DateTime<Utc>) -> ActiveRunRecord {
        let session_id = SilentSessionId::new();
        let run_id = SilentSessionRunId::new();
        ActiveRunRecord {
            runner_id: "runner:alice".into(),
            session_id,
            run_id,
            generation: 3,
            project_root: PathBuf::from("/projects/focusa"),
            project_identity_ref: "project:focusa".into(),
            workspace_root: PathBuf::from("/projects/focusa-worktree"),
            execution_user: "alice".into(),
            execution_uid: 501,
            executable_ref: "/usr/local/bin/pi".into(),
            launch_manifest_sha256: "manifest-sha256".into(),
            process_tree: ProcessTreeIdentity {
                process_instance_id: "process:019f".into(),
                runner_id: "runner:alice".into(),
                session_id,
                run_id,
                generation: 3,
                pid: 4120,
                process_group_id: 4120,
                os_session_id: 4120,
                execution_user: "alice".into(),
                execution_uid: 501,
                executable_ref: "/usr/local/bin/pi".into(),
                spawned_at: now - Duration::seconds(30),
            },
            heartbeat_at: now,
        }
    }

    #[test]
    fn signed_heartbeat_is_exactly_addressed_and_replay_protected() {
        let now = Utc::now();
        let runner = actor(ProtocolActorKind::Runner, "runner:alice", "alice", 501);
        let signer = ProtocolSigner::new(runner.clone(), SigningKey::from_bytes(&[7; 32]));
        let heartbeat = RunnerHeartbeat {
            runner_id: runner.actor_id.clone(),
            os_user: runner.os_user.clone(),
            uid: runner.uid,
            sequence: 9,
            observed_at: now,
            active_runs: vec![active_record(now)],
        };
        let frame = signer
            .sign(
                "daemon:local",
                "nonce:heartbeat:9",
                now,
                now + Duration::seconds(30),
                RunnerProtocolMessage::Heartbeat(heartbeat.clone()),
            )
            .expect("heartbeat should sign");
        let mut verifier = ProtocolVerifier::new(runner, "daemon:local", signer.verifying_key());

        assert_eq!(
            verifier.verify(&frame, now),
            Ok(RunnerProtocolMessage::Heartbeat(heartbeat))
        );
        assert_eq!(
            verifier.verify(&frame, now),
            Err(ProtocolError::ReplayDetected)
        );
    }

    #[test]
    fn tampered_expired_and_wrong_user_frames_fail_closed() {
        let now = Utc::now();
        let runner = actor(ProtocolActorKind::Runner, "runner:alice", "alice", 501);
        let signer = ProtocolSigner::new(runner.clone(), SigningKey::from_bytes(&[8; 32]));
        let hello = RunnerHello {
            runner_id: runner.actor_id.clone(),
            os_user: runner.os_user.clone(),
            uid: runner.uid,
            supported_protocol_versions: [RUNNER_PROTOCOL_VERSION].into_iter().collect(),
            capabilities: capabilities(),
            active_runs: vec![],
            runner_challenge_nonce: "challenge:runner".into(),
        };
        let frame = signer
            .sign(
                "daemon:local",
                "nonce:hello",
                now,
                now + Duration::seconds(30),
                RunnerProtocolMessage::RunnerHello(hello),
            )
            .expect("hello should sign");

        let mut tampered = frame.clone();
        tampered.receiver_id = "daemon:other".into();
        let mut verifier =
            ProtocolVerifier::new(runner.clone(), "daemon:other", signer.verifying_key());
        assert_eq!(
            verifier.verify(&tampered, now),
            Err(ProtocolError::SignatureInvalid)
        );

        let mut expired_verifier =
            ProtocolVerifier::new(runner.clone(), "daemon:local", signer.verifying_key());
        assert_eq!(
            expired_verifier.verify(&frame, now + Duration::seconds(31)),
            Err(ProtocolError::Expired)
        );

        let wrong_user = actor(ProtocolActorKind::Runner, "runner:alice", "bob", 502);
        let mut wrong_user_verifier =
            ProtocolVerifier::new(wrong_user, "daemon:local", signer.verifying_key());
        assert_eq!(
            wrong_user_verifier.verify(&frame, now),
            Err(ProtocolError::InvalidFrame)
        );
    }

    #[test]
    fn handshake_negotiates_highest_common_version_and_required_capabilities() {
        let hello = RunnerHello {
            runner_id: "runner:alice".into(),
            os_user: "alice".into(),
            uid: 501,
            supported_protocol_versions: [1, 2].into_iter().collect(),
            capabilities: capabilities(),
            active_runs: vec![],
            runner_challenge_nonce: "challenge:runner".into(),
        };
        let policy = DaemonHandshakePolicy {
            daemon_id: "daemon:local".into(),
            supported_protocol_versions: [1, 2, 3].into_iter().collect(),
            required_capabilities: [
                RunnerCapability::AuthenticatedFrames,
                RunnerCapability::Heartbeat,
                RunnerCapability::OrphanAdoption,
            ]
            .into_iter()
            .collect(),
        };
        let welcome = policy
            .negotiate(&hello, "challenge:daemon")
            .expect("compatible runner should negotiate");
        assert_eq!(welcome.selected_protocol_version, 2);
        assert_eq!(welcome.runner_challenge_nonce, "challenge:runner");
        assert_eq!(welcome.daemon_challenge_nonce, "challenge:daemon");

        let mut incompatible = hello;
        incompatible
            .capabilities
            .remove(&RunnerCapability::Heartbeat);
        assert_eq!(
            policy.negotiate(&incompatible, "challenge:other"),
            Err(ProtocolError::RequiredCapabilityMissing)
        );
    }

    #[test]
    fn adoption_requires_exact_signed_process_scope_and_fresh_heartbeat() {
        let now = Utc::now();
        let record = active_record(now);
        let expectation = AdoptionExpectation {
            daemon_id: "daemon:local".into(),
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
            heartbeat_fresh_after: now - Duration::seconds(5),
        };
        let accepted = expectation
            .evaluate(&record)
            .expect("record hashing should succeed");
        assert!(accepted.accepted);
        assert!(accepted.signed_runner_record_ref.is_some());
        let reconciliation = OrphanReconciliationRequest {
            expectation: expectation.clone(),
            expected_stream_cursor: "stream:42".into(),
        }
        .reconcile(accepted)
        .expect("accepted adoption should reconcile");
        assert_eq!(
            reconciliation.status,
            OrphanReconciliationStatus::AdoptedRecovering
        );
        assert_eq!(
            reconciliation.lifecycle_state,
            SilentSessionLifecycleState::Recovering
        );
        assert_eq!(
            reconciliation.restored_stream_cursor.as_deref(),
            Some("stream:42")
        );

        let mut wrong_workspace = record.clone();
        wrong_workspace.workspace_root = PathBuf::from("/projects/other");
        let rejected = expectation
            .evaluate(&wrong_workspace)
            .expect("rejection should be typed");
        assert_eq!(
            rejected.rejection,
            Some(AdoptionRejection::WorkspaceMismatch)
        );
        assert!(rejected.signed_runner_record_ref.is_none());
        let reconciliation = OrphanReconciliationRequest {
            expectation: expectation.clone(),
            expected_stream_cursor: "stream:42".into(),
        }
        .reconcile(rejected)
        .expect("rejected adoption should reconcile as orphaned");
        assert_eq!(
            reconciliation.status,
            OrphanReconciliationStatus::RejectedOrphaned
        );
        assert_eq!(
            reconciliation.lifecycle_state,
            SilentSessionLifecycleState::Orphaned
        );
        assert!(reconciliation.restored_stream_cursor.is_none());

        let mut reused_pid = record;
        reused_pid.process_tree.process_instance_id = "process:reused".into();
        let rejected = expectation
            .evaluate(&reused_pid)
            .expect("rejection should be typed");
        assert_eq!(
            rejected.rejection,
            Some(AdoptionRejection::ProcessIdentityMismatch)
        );
    }

    #[test]
    fn daemon_adoption_query_is_mutually_authenticated() {
        let now = Utc::now();
        let record = active_record(now);
        let daemon = actor(ProtocolActorKind::Daemon, "daemon:local", "root", 0);
        let signer = ProtocolSigner::new(daemon.clone(), SigningKey::from_bytes(&[9; 32]));
        let query = AdoptionExpectation {
            daemon_id: daemon.actor_id.clone(),
            runner_id: record.runner_id.clone(),
            session_id: record.session_id,
            run_id: record.run_id,
            generation: record.generation,
            project_root: record.project_root,
            project_identity_ref: record.project_identity_ref,
            workspace_root: record.workspace_root,
            execution_user: record.execution_user,
            execution_uid: record.execution_uid,
            executable_ref: record.executable_ref,
            launch_manifest_sha256: record.launch_manifest_sha256,
            expected_process_instance_id: Some(record.process_tree.process_instance_id),
            heartbeat_fresh_after: now - Duration::seconds(5),
        };
        let frame = signer
            .sign(
                "runner:alice",
                "nonce:adopt",
                now,
                now + Duration::seconds(30),
                RunnerProtocolMessage::AdoptionQuery(query.clone()),
            )
            .expect("daemon query should sign");
        let reconciliation_query = OrphanReconciliationRequest {
            expectation: query.clone(),
            expected_stream_cursor: "stream:reconnect:9".into(),
        };
        let reconciliation_frame = signer
            .sign(
                "runner:alice",
                "nonce:reconcile",
                now,
                now + Duration::seconds(30),
                RunnerProtocolMessage::OrphanReconciliationQuery(reconciliation_query.clone()),
            )
            .expect("daemon reconciliation query should sign");
        let mut verifier = ProtocolVerifier::new(daemon, "runner:alice", signer.verifying_key());
        assert_eq!(
            verifier.verify(&frame, now),
            Ok(RunnerProtocolMessage::AdoptionQuery(query))
        );
        assert_eq!(
            verifier.verify(&reconciliation_frame, now),
            Ok(RunnerProtocolMessage::OrphanReconciliationQuery(
                reconciliation_query
            ))
        );
    }
}
