//! Shared interactive activation presenter (Spec 152E §14.1, §21): one flow
//! renders email → verify → offer → checkout/poll → key/lease for both the
//! CLI (`focusa license activate-flow`) and the Rust installer. It drives the
//! shared [`ActivationSession`] from `focusa-license`; it never invents an
//! email, verification code, consent, payment confirmation, or license,
//! never accepts card data, and never self-issues. Existing key, Evaluation
//! (Spec 172 limited-access overlay), resume, cancel, timeout, and recovery
//! are all rendered here from the frozen presenter states and the shared
//! reducer.
//!
//! Presenter rendering only: every decision (identity, product, price,
//! Evaluation, license, node, lease, refund/revoke posture) comes from the
//! authority through the reducer. This module contains no transition table
//! and no entitlement logic.

use focusa_license::activation_agent::{AgentActivationEnvelope, AgentKeyReveal};
use focusa_license::activation_client::{
    ActivationAuthority, ActivationClientError, ActivationJourney, ActivationLedgerEvent,
    ActivationRegistration, ActivationSession, retry_policy_for_code,
};
use focusa_license::activation_facade::{
    ActivationError, ActivationErrorCode, ActivationRequestContext,
};
use focusa_license::activation_http::{
    ActivationHttpClient, ActivationHttpPolicy, LeaseDeliveryEnvelope,
};
use focusa_license::activation_reducer::{
    ActivationOutputEnvelope, ActivationState, RetryPosture, presenter_state,
};
use focusa_license::authority::{EntitlementSnapshot, LeaseVerificationContext};
use focusa_license::authority_client::SensitiveCredential;
use focusa_license::authority_credentials::{
    CredentialHandle, KeyringCredentialStore, NodeIdentity, ProtectedCredentialStore,
    load_or_create_node_identity,
};
use focusa_license::authority_store::{
    AUTHORITY_STATE_FILE, PersistedAuthorityState, embedded_production_trust_roots,
};
use serde::{Deserialize, Serialize};
use std::io::{IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use thiserror::Error;

/// Presenter identity for one flow invocation. Presenters may render prompts
/// and links; they may not reimplement identity, product, payment,
/// Evaluation, license, node, or lease decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActivationFlowConfig {
    pub facade_id: &'static str,
    pub presenter: &'static str,
    pub install_channel: &'static str,
    pub origin: &'static str,
}

/// CLI (`focusa license activate-flow`) presenter identity.
pub const CLI_FLOW: ActivationFlowConfig = ActivationFlowConfig {
    facade_id: "wpuiai_public_v1",
    presenter: "cli",
    install_channel: "source_build",
    origin: "https://wpuiai.com",
};

/// Rust installer presenter identity (`focusa install`).
pub const INSTALLER_FLOW: ActivationFlowConfig = ActivationFlowConfig {
    facade_id: "focusa_install_v1",
    presenter: "installer",
    install_channel: "official_installer",
    origin: "https://install.focusa.dev",
};

/// Fail-closed interactive-flow errors. Authority errors carry the typed
/// registry code; local input/invariant breaches never grant anything.
#[derive(Debug, Error)]
pub enum ActivationFlowError {
    #[error("activation flow requires an email")]
    EmailRequired,
    #[error("activation flow requires an interactive terminal")]
    InteractiveRequired,
    #[error("activation flow restart: verification expired; begin again")]
    RestartVerificationRequired,
    #[error("activation flow client error: {0}")]
    Client(ActivationClientError),
    #[error("activation flow input error: {0}")]
    Io(std::io::Error),
    #[error("activation flow render error: {0}")]
    Render(serde_json::Error),
    #[error("activation delivery persistence failed: {0}")]
    Delivery(String),
}

impl From<ActivationClientError> for ActivationFlowError {
    fn from(value: ActivationClientError) -> Self {
        Self::Client(value)
    }
}

impl From<serde_json::Error> for ActivationFlowError {
    fn from(value: serde_json::Error) -> Self {
        Self::Render(value)
    }
}

impl From<std::io::Error> for ActivationFlowError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

/// One presenter-visible flow step. The transcript is deterministic and
/// replayable: presenter state labels are the frozen values, and no raw
/// email, poll credential, or full key ever appears.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlowEvent {
    pub presenter_state: String,
    pub terminal: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub masked_email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safe_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_action: Option<String>,
}

/// Terminal outcome of one interactive flow: the presenter transcript, the
/// final registration, and the ledger. Only presenter-safe values are
/// exposed.
#[derive(Debug)]
pub struct ActivationFlowOutcome {
    pub presenter_state: String,
    pub terminal: bool,
    pub registration_id: String,
    pub masked_email: Option<String>,
    pub events: Vec<FlowEvent>,
    pub ledger: Vec<ActivationLedgerEvent>,
}

/// Prompt source abstraction so the flow is testable without a terminal.
pub trait ActivationFlowInput {
    fn prompt(&mut self, label: &str) -> std::io::Result<String>;
}

/// Terminal (stdin/stdout) prompt source.
pub struct StdinFlowInput;

impl ActivationFlowInput for StdinFlowInput {
    fn prompt(&mut self, label: &str) -> std::io::Result<String> {
        print!("{label} ");
        std::io::stdout().flush()?;
        let mut answer = String::new();
        std::io::stdin().read_line(&mut answer)?;
        Ok(answer.trim().to_string())
    }
}

/// Scripted prompt source for deterministic transcript replay.
pub struct ScriptedFlowInput {
    answers: std::collections::VecDeque<String>,
}

impl ScriptedFlowInput {
    pub fn new(answers: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            answers: answers.into_iter().map(Into::into).collect(),
        }
    }
}

impl ActivationFlowInput for ScriptedFlowInput {
    fn prompt(&mut self, _label: &str) -> std::io::Result<String> {
        Ok(self.answers.pop_front().unwrap_or_default())
    }
}

/// True when stdin and stdout are interactive terminals (the universal flow
/// renders prompts; noninteractive presenters must fail closed and use
/// device-code authorization instead).
pub fn interactive_available() -> bool {
    std::io::stdin().is_terminal() && std::io::stdout().is_terminal()
}

/// Presenter persistence hook: called after every successful non-terminal
/// step so the presenter can store the presenter-safe registration snapshot,
/// the expiring poll credential (protected store), and, at terminal
/// delivery, the verified signed lease — never raw keys or emails.
pub trait ActivationFlowPersist<A: ActivationAuthority> {
    fn persist(&self, session: &ActivationSession<A>) -> Result<(), ActivationFlowError>;
}

/// Default presenter persistence for the CLI and Rust installer: private
/// snapshot, protected-store poll credential, and verified lease/terminal
/// envelope persistence.
pub struct ActivationFlowSessionPersist {
    config_dir: PathBuf,
}

impl ActivationFlowSessionPersist {
    pub fn new(config_dir: &Path) -> Self {
        Self {
            config_dir: config_dir.to_path_buf(),
        }
    }

    fn persist_inner(
        &self,
        session: &ActivationSession<ActivationHttpClient>,
    ) -> Result<(), ActivationFlowError> {
        persist_agent_session_state(&self.config_dir, session, &KeyringCredentialStore)
    }

    pub fn config_dir(&self) -> &std::path::Path {
        &self.config_dir
    }
}

/// Persist the resumable agent-session state (registration snapshot + protected
/// poll credential; lease/key envelopes when terminal). Shared by the interactive
/// flow and the agent/JSON protocol so continuity never depends on which surface
/// ran the step (#370).
pub fn persist_agent_session_state<A: ActivationAuthority>(
    config_dir: &std::path::Path,
    session: &ActivationSession<A>,
    store: &dyn ProtectedCredentialStore,
) -> Result<(), ActivationFlowError> {
    persist_registration_snapshot(config_dir, session.registration())?;
    if let Some(credential) = session.poll_credential() {
        persist_poll_credential(store, session.registration_id(), credential)?;
    }
    if session.state().is_terminal() {
        let envelope = session.envelope(None)?;
        if let Some(lease) = envelope.lease_envelope.as_deref() {
            let identity = resolve_flow_node_identity(config_dir)?;
            persist_delivered_lease(config_dir, &identity, lease, chrono::Utc::now())?;
        }
        if let Some(key_envelope) = envelope.one_time_key_envelope.as_deref() {
            persist_key_envelope(config_dir, session.registration_id(), key_envelope)?;
        }
    }
    Ok(())
}

impl ActivationFlowPersist<ActivationHttpClient> for ActivationFlowSessionPersist {
    fn persist(
        &self,
        session: &ActivationSession<ActivationHttpClient>,
    ) -> Result<(), ActivationFlowError> {
        self.persist_inner(session)
    }
}

/// Begin and drive the universal activation flow to a terminal state.
///
/// Journey selection mirrors Spec 152E §14.1: `1` existing license, `2`
/// purchase, `3` Evaluation (Spec 172 limited-access overlay; the authority
/// decides eligibility and never lets a client issue Evaluation).
///
/// `persist` (when provided) is called after every successful step so the
/// presenter can store the presenter-safe registration snapshot, the
/// expiring poll credential (protected store), and, at terminal delivery,
/// the verified signed lease — never raw keys or emails.
#[allow(clippy::too_many_arguments)]
pub fn run_activation_flow<A: ActivationAuthority>(
    authority: A,
    config: ActivationFlowConfig,
    input: &mut dyn ActivationFlowInput,
    email: Option<String>,
    device_public_key: Option<String>,
    journey: Option<ActivationJourney>,
    poll_timeout_seconds: Option<u64>,
    json_output: bool,
    persist: Option<&dyn ActivationFlowPersist<A>>,
) -> Result<ActivationFlowOutcome, ActivationFlowError> {
    let context = new_context(config);
    let email = match email {
        Some(email) => email,
        None => input.prompt("Email:")?,
    };
    if email.trim().is_empty() {
        return Err(ActivationFlowError::EmailRequired);
    }
    let mut session = ActivationSession::begin(
        authority,
        context,
        &email,
        "focusa",
        device_public_key.as_deref(),
    )?;
    let mut events = Vec::new();
    emit(&session, None, &mut events, json_output)?;
    try_persist(&session, persist)?;
    if session.state().is_terminal() {
        return finish(session, events, json_output);
    }

    // Mailbox control: the one-time verifier is never invented here.
    let code = input.prompt("Verification code:")?;
    match session.verify(code.trim()) {
        Ok(envelope) => {
            emit_envelope(&envelope, &mut events, json_output)?;
            try_persist(&session, persist)?;
            if envelope.terminal {
                return finish(session, events, json_output);
            }
        }
        Err(error) => return Err(handle_client_error(session, error, events, json_output)),
    }
    if session.state().is_terminal() {
        return finish(session, events, json_output);
    }

    // Offer/selection: server-owned offers are listed when present; the
    // §14.1 menu is the presenter rendering for the three journeys.
    let offers = match session.offers() {
        Ok(offers) => offers,
        Err(error) => return Err(handle_client_error(session, error, events, json_output)),
    };
    if json_output {
        for offer in &offers {
            println!(
                "{}",
                serde_json::to_string(&serde_json::json!({
                    "offer": {
                        "public_code": offer.public_code,
                        "display_name": offer.display_name,
                        "journey": offer.journey.label(),
                    }
                }))
                .unwrap_or_else(|_| "{}".into())
            );
        }
    } else {
        for (index, offer) in offers.iter().enumerate() {
            println!(
                "{}. {} ({})",
                index + 1,
                offer.display_name,
                offer.journey.label()
            );
        }
    }
    let choice = match journey {
        Some(ActivationJourney::Purchase) => "2".to_string(),
        Some(ActivationJourney::ExistingKey) => "1".to_string(),
        Some(ActivationJourney::LimitedAccess) => "3".to_string(),
        None => {
            input.prompt("1. Enter existing license  2. Purchase Focusa  3. Request Evaluation:")?
        }
    };
    match choice.trim() {
        "1" => {
            let key = input.prompt("License key:")?;
            if key.trim().is_empty() {
                return Err(ActivationFlowError::Client(
                    ActivationClientError::Authority(ActivationError::new(
                        ActivationErrorCode::EddLicenseUnusable,
                        session.registration_id().to_string(),
                    )),
                ));
            }
            match session.existing_license(key.trim(), device_public_key.as_deref()) {
                Ok(envelope) => {
                    emit_envelope(&envelope, &mut events, json_output)?;
                    try_persist(&session, persist)?;
                    if envelope.terminal {
                        return finish(session, events, json_output);
                    }
                }
                Err(error) => return Err(handle_client_error(session, error, events, json_output)),
            }
        }
        "2" => {
            match session.select_offer("focusa", ActivationJourney::Purchase) {
                Ok(envelope) => {
                    emit_envelope(&envelope, &mut events, json_output)?;
                    try_persist(&session, persist)?;
                    if envelope.terminal {
                        return finish(session, events, json_output);
                    }
                }
                Err(error) => return Err(handle_client_error(session, error, events, json_output)),
            }
            match session.checkout(None) {
                Ok(envelope) => {
                    emit_envelope(&envelope, &mut events, json_output)?;
                    try_persist(&session, persist)?;
                    if envelope.terminal {
                        return finish(session, events, json_output);
                    }
                }
                Err(error) => return Err(handle_client_error(session, error, events, json_output)),
            }
        }
        _ => {
            // Evaluation intent → Spec 172 limited-access overlay. The
            // authority settles `limited_access_chosen` or denies with
            // EVALUATION_NOT_ELIGIBLE; a client can never issue Evaluation.
            match session.select_offer("focusa", ActivationJourney::LimitedAccess) {
                Ok(envelope) => {
                    emit_envelope(&envelope, &mut events, json_output)?;
                    try_persist(&session, persist)?;
                    if envelope.terminal {
                        return finish(session, events, json_output);
                    }
                }
                Err(error) => return Err(handle_client_error(session, error, events, json_output)),
            }
        }
    }

    // Bounded poll: wall-clock timeout and registration poll budget both
    // settle fail-closed (cancel → recovery_only); terminal states stop.
    let deadline =
        poll_timeout_seconds.map(|seconds| Instant::now() + Duration::from_secs(seconds));
    while !session.state().is_terminal() {
        if let Some(deadline) = deadline {
            if Instant::now() >= deadline {
                let envelope = session.cancel().map_err(ActivationFlowError::Client)?;
                emit_envelope(&envelope, &mut events, json_output)?;
                return finish(session, events, json_output);
            }
        }
        match session.poll() {
            Ok(envelope) => {
                emit_envelope(&envelope, &mut events, json_output)?;
                try_persist(&session, persist)?;
                if envelope.terminal {
                    return finish(session, events, json_output);
                }
            }
            Err(ActivationClientError::Authority(error)) => {
                let retry = retry_policy_for_code(error.code);
                match retry.posture {
                    RetryPosture::SafeRetry | RetryPosture::RetrySameIdempotencyKey => {
                        emit_error(&error, &mut events, json_output)?;
                        let seconds = retry.retry_after_seconds.unwrap_or(3).min(30);
                        std::thread::sleep(Duration::from_secs(seconds as u64));
                    }
                    RetryPosture::Restart => {
                        return Err(ActivationFlowError::RestartVerificationRequired);
                    }
                    RetryPosture::RecoveryOnly | RetryPosture::None => {
                        emit_error(&error, &mut events, json_output)?;
                        // A recovery-only or non-retryable authority error
                        // settles the local registration fail-closed via
                        // cancel (denied -> recovery_only) and never grants.
                        let cancelled = session.cancel().map_err(ActivationFlowError::Client)?;
                        emit_envelope(&cancelled, &mut events, json_output)?;
                        return finish(session, events, json_output);
                    }
                }
            }
            Err(ActivationClientError::PollBudgetExhausted) => {
                let envelope = session.cancel().map_err(ActivationFlowError::Client)?;
                emit_envelope(&envelope, &mut events, json_output)?;
                return finish(session, events, json_output);
            }
            Err(error) => return Err(ActivationFlowError::Client(error)),
        }
    }
    finish(session, events, json_output)
}

/// Resume a persisted registration (bounded poll continuation). The poll
/// credential is re-supplied from the protected store; the snapshot never
/// contains it. Terminal registrations refuse every step in the shared
/// client before any presenter action.
/// One bounded authority reconciliation for `license status` (#342 field
/// evidence): when the local signed lease is absent but a resumable
/// registration snapshot + poll credential exist, poll the authority once and
/// persist any delivered terminal envelopes. This is how activation completed
/// manually on the authority website becomes visible locally.
/// Fail-closed: any transport/state error returns Ok(false) and the status
/// projection stays unactivated — never a fabricated licensed state.
pub fn reconcile_status_with_authority(config_dir: &Path) -> Result<bool, ActivationFlowError> {
    use focusa_license::activation_client::ActivationSession;

    let directory = config_dir.join("activation");
    let mut candidates: Vec<String> = std::fs::read_dir(&directory)
        .map(|entries| {
            entries
                .filter_map(|entry| entry.ok())
                .filter_map(|entry| {
                    let name = entry.file_name().to_string_lossy().to_string();
                    name.strip_suffix(".json").map(String::from)
                })
                .collect()
        })
        .unwrap_or_default();
    candidates.sort();
    let Some(registration_id) = candidates.pop() else {
        return Ok(false);
    };
    let registration = match load_registration_snapshot(config_dir, &registration_id) {
        Ok(registration) => registration,
        Err(_) => return Ok(false),
    };
    let credential = match load_poll_credential(&KeyringCredentialStore, &registration_id) {
        Ok(credential) => credential,
        Err(_) => return Ok(false),
    };
    let base_url = std::env::var("FOCUSA_AUTHORITY_ORIGIN")
        .unwrap_or_else(|_| "https://wpuiai.com/wp-json/wpuiai-ai-cloud/v1/".to_string());
    let policy = ActivationHttpPolicy {
        base_url: reqwest::Url::parse(&base_url)
            .map_err(|error| ActivationFlowError::Delivery(error.to_string()))?,
        timeout: std::time::Duration::from_secs(30),
        max_response_bytes: 1024 * 1024,
    };
    let http = ActivationHttpClient::new(policy)
        .map_err(|error| ActivationFlowError::Delivery(error.to_string()))?;
    let context = new_context(CLI_FLOW);
    let mut session = ActivationSession::resume(http, context, registration, credential)
        .map_err(ActivationFlowError::Client)?;
    if session.state().is_terminal() {
        return Ok(true);
    }
    if session.poll().is_err() {
        return Ok(false);
    }
    ActivationFlowSessionPersist::new(config_dir).persist_inner(&session)?;
    Ok(session.state().is_terminal())
}

pub fn resume_activation_flow<A: ActivationAuthority>(
    authority: A,
    config: ActivationFlowConfig,
    registration: ActivationRegistration,
    poll_credential: SensitiveCredential,
    poll_timeout_seconds: Option<u64>,
    json_output: bool,
    persist: Option<&dyn ActivationFlowPersist<A>>,
) -> Result<ActivationFlowOutcome, ActivationFlowError> {
    let context = new_context(config);
    let mut session = ActivationSession::resume(authority, context, registration, poll_credential)?;
    let mut events = Vec::new();
    if session.state().is_terminal() {
        let envelope = session.envelope(None)?;
        emit_envelope(&envelope, &mut events, json_output)?;
        try_persist(&session, persist)?;
        return finish(session, events, json_output);
    }
    emit(&session, None, &mut events, json_output)?;
    try_persist(&session, persist)?;
    let deadline =
        poll_timeout_seconds.map(|seconds| Instant::now() + Duration::from_secs(seconds));
    while !session.state().is_terminal() {
        if let Some(deadline) = deadline {
            if Instant::now() >= deadline {
                let envelope = session.cancel().map_err(ActivationFlowError::Client)?;
                emit_envelope(&envelope, &mut events, json_output)?;
                return finish(session, events, json_output);
            }
        }
        match session.poll() {
            Ok(envelope) => {
                emit_envelope(&envelope, &mut events, json_output)?;
                try_persist(&session, persist)?;
                if envelope.terminal {
                    return finish(session, events, json_output);
                }
            }
            Err(ActivationClientError::Authority(error)) => {
                let retry = retry_policy_for_code(error.code);
                match retry.posture {
                    RetryPosture::SafeRetry | RetryPosture::RetrySameIdempotencyKey => {
                        emit_error(&error, &mut events, json_output)?;
                        let seconds = retry.retry_after_seconds.unwrap_or(3).min(30);
                        std::thread::sleep(Duration::from_secs(seconds as u64));
                    }
                    RetryPosture::Restart => {
                        return Err(ActivationFlowError::RestartVerificationRequired);
                    }
                    RetryPosture::RecoveryOnly | RetryPosture::None => {
                        emit_error(&error, &mut events, json_output)?;
                        // A recovery-only or non-retryable authority error
                        // settles the local registration fail-closed via
                        // cancel (denied -> recovery_only) and never grants.
                        let cancelled = session.cancel().map_err(ActivationFlowError::Client)?;
                        emit_envelope(&cancelled, &mut events, json_output)?;
                        return finish(session, events, json_output);
                    }
                }
            }
            Err(ActivationClientError::PollBudgetExhausted) => {
                let envelope = session.cancel().map_err(ActivationFlowError::Client)?;
                emit_envelope(&envelope, &mut events, json_output)?;
                return finish(session, events, json_output);
            }
            Err(error) => return Err(ActivationFlowError::Client(error)),
        }
    }
    finish(session, events, json_output)
}

fn new_context(config: ActivationFlowConfig) -> ActivationRequestContext {
    let request_id = uuid::Uuid::now_v7().to_string();
    let idempotency_key = uuid::Uuid::now_v7().to_string();
    ActivationRequestContext::new(
        request_id,
        config.facade_id,
        config.presenter,
        config.install_channel,
        config.origin,
        Some(idempotency_key),
    )
}

// ── Spec 152E §14.2 agent/JSON protocol ────────────────────────────────────

/// Terminal outcome of one agent step: the typed human-action envelope plus
/// the resumable registration handle. When `terminal` is false the agent must
/// hand the envelope to the human and resume later with `--resume <handle>`.
#[derive(Debug)]
pub struct AgentActivationOutcome {
    pub envelope: AgentActivationEnvelope,
    pub registration_id: String,
    pub terminal: bool,
}

/// One bounded agent step for a NEW registration: the email only creates a
/// pending attempt (Spec 152E §5/§6.1). The agent never invents a
/// verification code, so the envelope reports the typed human action
/// (`enter_verification_code`) and stops with the resumable handle. No prompt
/// is ever rendered and nothing is invented.
pub type AgentPersistHook<'a, A> =
    &'a dyn Fn(&ActivationSession<A>) -> Result<(), ActivationFlowError>;

pub fn run_agent_activation<A: ActivationAuthority>(
    authority: A,
    config: ActivationFlowConfig,
    email: Option<String>,
    device_public_key: Option<String>,
    reveal: AgentKeyReveal,
    persist: Option<AgentPersistHook<'_, A>>,
) -> Result<AgentActivationOutcome, ActivationFlowError> {
    let context = new_context(config);
    let email = email.ok_or(ActivationFlowError::EmailRequired)?;
    if email.trim().is_empty() {
        return Err(ActivationFlowError::EmailRequired);
    }
    let session = ActivationSession::begin(
        authority,
        context,
        &email,
        "focusa",
        device_public_key.as_deref(),
    )?;
    // The handle must survive process exit before the outcome is reported (#370).
    if let Some(persist) = persist {
        persist(&session)?;
    }
    let envelope = AgentActivationEnvelope::from_session(&session, None, reveal, None)?;
    Ok(AgentActivationOutcome {
        registration_id: session.registration_id().to_string(),
        terminal: session.state().is_terminal(),
        envelope,
    })
}

/// One bounded agent resume step: re-supplies the poll credential from the
/// protected store and polls within the registration budget (and optional
/// wall-clock timeout). Terminal settlements end the session; any other state
/// still requires a human action, so the agent receives the typed envelope
/// and the resumable handle instead of exhausting the budget. Authority
/// recovery/refund/revoke settles fail-closed to `recovery_only`.
pub fn resume_agent_activation<A: ActivationAuthority>(
    authority: A,
    config: ActivationFlowConfig,
    registration: ActivationRegistration,
    poll_credential: SensitiveCredential,
    poll_timeout_seconds: Option<u64>,
    reveal: AgentKeyReveal,
    persist: Option<AgentPersistHook<'_, A>>,
) -> Result<AgentActivationOutcome, ActivationFlowError> {
    let context = new_context(config);
    let mut session = ActivationSession::resume(authority, context, registration, poll_credential)?;
    if session.state().is_terminal() {
        if let Some(persist) = persist {
            persist(&session)?;
        }
        let envelope = AgentActivationEnvelope::from_session(&session, None, reveal, None)?;
        return Ok(AgentActivationOutcome {
            registration_id: session.registration_id().to_string(),
            terminal: true,
            envelope,
        });
    }
    let deadline =
        poll_timeout_seconds.map(|seconds| Instant::now() + Duration::from_secs(seconds));
    loop {
        if let Some(deadline) = deadline {
            if Instant::now() >= deadline {
                session.cancel().map_err(ActivationFlowError::Client)?;
                return finish_agent(session, reveal, persist);
            }
        }
        match session.poll() {
            Ok(_) => {
                // One bounded poll settled the step. Terminal states stop;
                // any other state still requires a human action, so return
                // the typed envelope + resumable handle. State is persisted
                // inside finish so a killed process never loses settlement.
                return finish_agent(session, reveal, persist);
            }
            Err(ActivationClientError::Authority(error)) => {
                let retry = retry_policy_for_code(error.code);
                match retry.posture {
                    RetryPosture::SafeRetry | RetryPosture::RetrySameIdempotencyKey => {
                        let seconds = retry.retry_after_seconds.unwrap_or(3).min(30);
                        std::thread::sleep(Duration::from_secs(seconds as u64));
                    }
                    RetryPosture::Restart => {
                        return Err(ActivationFlowError::RestartVerificationRequired);
                    }
                    RetryPosture::RecoveryOnly | RetryPosture::None => {
                        session.cancel().map_err(ActivationFlowError::Client)?;
                        return finish_agent_with_error(session, Some(&error), reveal, persist);
                    }
                }
            }
            Err(ActivationClientError::PollBudgetExhausted) => {
                session.cancel().map_err(ActivationFlowError::Client)?;
                return finish_agent(session, reveal, persist);
            }
            Err(error) => return Err(ActivationFlowError::Client(error)),
        }
    }
}

fn finish_agent<A: ActivationAuthority>(
    session: ActivationSession<A>,
    reveal: AgentKeyReveal,
    persist: Option<AgentPersistHook<'_, A>>,
) -> Result<AgentActivationOutcome, ActivationFlowError> {
    finish_agent_with_error(session, None, reveal, persist)
}

fn finish_agent_with_error<A: ActivationAuthority>(
    session: ActivationSession<A>,
    error: Option<&ActivationError>,
    reveal: AgentKeyReveal,
    persist: Option<AgentPersistHook<'_, A>>,
) -> Result<AgentActivationOutcome, ActivationFlowError> {
    if let Some(persist) = persist {
        persist(&session)?;
    }
    let envelope = AgentActivationEnvelope::from_session(&session, error, reveal, None)?;
    Ok(AgentActivationOutcome {
        registration_id: session.registration_id().to_string(),
        terminal: session.state().is_terminal(),
        envelope,
    })
}

fn try_persist<A: ActivationAuthority>(
    session: &ActivationSession<A>,
    persist: Option<&dyn ActivationFlowPersist<A>>,
) -> Result<(), ActivationFlowError> {
    if let Some(persist) = persist {
        persist.persist(session)?;
    }
    Ok(())
}

fn handle_client_error<A: ActivationAuthority>(
    session: ActivationSession<A>,
    error: ActivationClientError,
    mut events: Vec<FlowEvent>,
    json_output: bool,
) -> ActivationFlowError {
    let next_action = match &error {
        ActivationClientError::Authority(authority) => {
            events.push(FlowEvent {
                presenter_state: presenter_state(session.state()).label().to_string(),
                terminal: session.state().is_terminal(),
                masked_email: session.registration().masked_email.clone(),
                safe_url: None,
                error_code: Some(authority.code.label().to_string()),
                next_action: Some(authority.code.safe_next_action().to_string()),
            });
            authority.code.safe_next_action().to_string()
        }
        ActivationClientError::PollBudgetExhausted => "restart_or_recover_activation".to_string(),
        _ => "recovery_only".to_string(),
    };
    if json_output {
        println!(
            "{}",
            serde_json::to_string(&serde_json::json!({
                "error": {
                    "code": client_error_code(&error),
                    "next_action": next_action,
                }
            }))
            .unwrap_or_else(|_| "{}".into())
        );
    } else {
        eprintln!(
            "Activation stopped: {} (next action: {next_action})",
            client_error_code(&error)
        );
    }
    ActivationFlowError::Client(error)
}

fn client_error_code(error: &ActivationClientError) -> &'static str {
    match error {
        ActivationClientError::Authority(authority) => authority.code.label(),
        ActivationClientError::IllegalTransition { .. } => "AUTHORITY_UNAVAILABLE",
        ActivationClientError::PollBudgetExhausted => "POLL_CREDENTIAL_EXPIRED",
        ActivationClientError::TerminalRegistration => "POLL_CREDENTIAL_REQUIRED",
        ActivationClientError::UnmaskableEmail => "EMAIL_REQUIRED",
    }
}

fn emit<A: ActivationAuthority>(
    session: &ActivationSession<A>,
    error: Option<ActivationError>,
    events: &mut Vec<FlowEvent>,
    json_output: bool,
) -> Result<(), ActivationFlowError> {
    let envelope = session.envelope(error)?;
    emit_envelope(&envelope, events, json_output)
}

fn emit_envelope(
    envelope: &ActivationOutputEnvelope,
    events: &mut Vec<FlowEvent>,
    json_output: bool,
) -> Result<(), ActivationFlowError> {
    let event = FlowEvent {
        presenter_state: envelope.state.clone(),
        terminal: envelope.terminal,
        masked_email: envelope.masked_email.clone(),
        safe_url: envelope.safe_url.clone(),
        error_code: envelope.error.as_ref().map(|error| error.code.clone()),
        next_action: envelope
            .error
            .as_ref()
            .map(|error| error.next_action.clone()),
    };
    events.push(event);
    if json_output {
        println!("{}", serde_json::to_string_pretty(envelope)?);
    } else {
        render_human(envelope);
    }
    Ok(())
}

fn emit_error(
    error: &ActivationError,
    events: &mut Vec<FlowEvent>,
    json_output: bool,
) -> Result<(), ActivationFlowError> {
    let event = FlowEvent {
        presenter_state: "denied".into(),
        terminal: false,
        masked_email: None,
        safe_url: None,
        error_code: Some(error.code.label().to_string()),
        next_action: Some(error.code.safe_next_action().to_string()),
    };
    events.push(event);
    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "error": {
                    "code": error.code.label(),
                    "next_action": error.code.safe_next_action(),
                }
            }))?
        );
    } else {
        eprintln!(
            "Authority: {} (next action: {})",
            error.code.label(),
            error.code.safe_next_action()
        );
    }
    Ok(())
}

/// Human rendering of the canonical presenter envelope (Spec 152E §14.1
/// style). Rendering only; the presenter state comes from the reducer.
fn render_human(envelope: &ActivationOutputEnvelope) {
    match envelope.state.as_str() {
        "email_required" => println!("Focusa requires activation."),
        "email_verification_pending" => {
            match envelope.verification_delivery_status.as_deref() {
                Some("queued") => println!(
                    "Verification code queued (mail delivery pending). Enter the code when it arrives."
                ),
                Some("failed") => println!(
                    "Verification code delivery failed. Use `focusa license resend` to retry."
                ),
                // Honest default: only claim sent when the authority reported it.
                Some(other) => println!(
                    "Verification code delivery: {other}. Enter the code when it arrives."
                ),
                None => println!("Enter the verification code when it arrives."),
            }
        }
        "email_verified" => println!("Email verified."),
        "selection_required" => println!("Select an option:"),
        "checkout_required" => println!("Starting checkout..."),
        "payment_pending" => {
            if let Some(url) = envelope.safe_url.as_deref() {
                println!("Complete payment:\n{url}");
            }
            println!("Waiting for payment...");
        }
        "license_delivery_ready" => {
            println!("License delivered.");
            if let Some(masked) = envelope.masked_email.as_deref() {
                println!("A copy was emailed to {masked}.");
            }
        }
        "activated" => println!("Device activated."),
        "denied" => {
            println!(
                "Activation denied. {}",
                envelope
                    .error
                    .as_ref()
                    .map(|error| error.next_action.as_str())
                    .unwrap_or("recovery, export, repair, and uninstall remain available")
            );
        }
        "recovery_only" => {
            println!("Recovery only: recovery, export, repair, and uninstall remain available.");
        }
        other => println!("State: {other}"),
    }
}

fn finish<A: ActivationAuthority>(
    session: ActivationSession<A>,
    events: Vec<FlowEvent>,
    json_output: bool,
) -> Result<ActivationFlowOutcome, ActivationFlowError> {
    let outcome = ActivationFlowOutcome {
        presenter_state: presenter_state(session.state()).label().to_string(),
        terminal: session.state().is_terminal(),
        registration_id: session.registration_id().to_string(),
        masked_email: session.registration().masked_email.clone(),
        events,
        ledger: session.ledger().to_vec(),
    };
    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "schema": "focusa.activation_flow_outcome.v1",
                "presenter_state": outcome.presenter_state,
                "terminal": outcome.terminal,
                "registration_id": outcome.registration_id,
                "masked_email": outcome.masked_email,
                "ledger": outcome.ledger,
            }))?
        );
    }
    Ok(outcome)
}

// ── Safe persistence: snapshots, poll credential, delivered lease ─────────

const ACTIVATION_SNAPSHOT_SCHEMA: &str = "focusa.activation_registration.v1";

/// Persist the presenter-safe registration snapshot (never the poll
/// credential) so the flow can resume after a pause.
pub fn persist_registration_snapshot(
    config_dir: &Path,
    registration: &ActivationRegistration,
) -> Result<PathBuf, ActivationFlowError> {
    if registration.schema != ACTIVATION_SNAPSHOT_SCHEMA {
        return Err(ActivationFlowError::Delivery(
            "registration snapshot has an unknown schema".into(),
        ));
    }
    let directory = config_dir.join("activation");
    std::fs::create_dir_all(&directory)
        .map_err(|error| ActivationFlowError::Delivery(error.to_string()))?;
    let path = directory.join(format!("{}.json", registration.registration_id));
    write_private(
        &path,
        &serde_json::to_vec_pretty(registration)
            .map_err(|error| ActivationFlowError::Delivery(error.to_string()))?,
    )?;
    Ok(path)
}

/// Load a persisted registration snapshot for resume.
pub fn load_registration_snapshot(
    config_dir: &Path,
    registration_id: &str,
) -> Result<ActivationRegistration, ActivationFlowError> {
    let path = config_dir
        .join("activation")
        .join(format!("{registration_id}.json"));
    let raw = std::fs::read_to_string(&path)
        .map_err(|error| ActivationFlowError::Delivery(error.to_string()))?;
    let registration: ActivationRegistration = serde_json::from_str(&raw)
        .map_err(|error| ActivationFlowError::Delivery(error.to_string()))?;
    if registration.schema != ACTIVATION_SNAPSHOT_SCHEMA {
        return Err(ActivationFlowError::Delivery(
            "registration snapshot has an unknown schema".into(),
        ));
    }
    Ok(registration)
}

/// Store the expiring poll credential in the protected store under the
/// registration-scoped handle (never in the snapshot).
pub fn persist_poll_credential(
    store: &dyn ProtectedCredentialStore,
    registration_id: &str,
    credential: &SensitiveCredential,
) -> Result<(), ActivationFlowError> {
    let handle = CredentialHandle::for_registration(registration_id)
        .map_err(|error| ActivationFlowError::Delivery(format!("{error}")))?;
    store
        .put(&handle, credential)
        .map_err(|error| ActivationFlowError::Delivery(format!("{error}")))
}

/// Re-supply the poll credential from the protected store for resume.
pub fn load_poll_credential(
    store: &dyn ProtectedCredentialStore,
    registration_id: &str,
) -> Result<SensitiveCredential, ActivationFlowError> {
    let handle = CredentialHandle::for_registration(registration_id)
        .map_err(|error| ActivationFlowError::Delivery(format!("{error}")))?;
    store
        .get(&handle)
        .map_err(|error| ActivationFlowError::Delivery(format!("{error}")))
}

/// Persist the delivered signed lease through the canonical authority store:
/// parse the authority-owned delivery bundle, verify both envelopes against
/// the embedded production trust roots, and atomically write the state file.
/// No raw key, email, or credential is ever written here. Without embedded
/// production trust roots this fails closed.
pub fn persist_delivered_lease(
    config_dir: &Path,
    identity: &NodeIdentity,
    lease_envelope_raw: &str,
    now: chrono::DateTime<chrono::Utc>,
) -> Result<EntitlementSnapshot, ActivationFlowError> {
    let delivery = LeaseDeliveryEnvelope::parse(lease_envelope_raw).map_err(|_| {
        ActivationFlowError::Delivery("lease delivery envelope is malformed".into())
    })?;
    let context = LeaseVerificationContext {
        expected_product: "focusa".into(),
        expected_node_id: identity.node_id.clone(),
        now,
        minimum_sequence: None,
        expected_previous_digest: None,
    };
    let roots = embedded_production_trust_roots().map_err(|error| {
        ActivationFlowError::Delivery(format!("production trust roots unavailable: {error}"))
    })?;
    let (state, snapshot) = PersistedAuthorityState::from_verified_envelopes(
        delivery.key_set,
        delivery.lease,
        &roots,
        &context,
    )
    .map_err(|error| {
        ActivationFlowError::Delivery(format!("lease verification failed: {error}"))
    })?;
    state
        .write_atomic(&config_dir.join(AUTHORITY_STATE_FILE))
        .map_err(|error| {
            ActivationFlowError::Delivery(format!("authority state write failed: {error}"))
        })?;
    Ok(snapshot)
}

/// Persist the encrypted one-time terminal key envelope (never the plaintext
/// key) to a private file so the EDD key can be re-presented by authenticated
/// recovery; the human key is also delivered by EDD transactional email.
pub fn persist_key_envelope(
    config_dir: &Path,
    registration_id: &str,
    one_time_key_envelope: &str,
) -> Result<PathBuf, ActivationFlowError> {
    let directory = config_dir.join("activation");
    std::fs::create_dir_all(&directory)
        .map_err(|error| ActivationFlowError::Delivery(error.to_string()))?;
    let path = directory.join(format!("{registration_id}.terminal-key-envelope"));
    write_private(&path, one_time_key_envelope.as_bytes())?;
    Ok(path)
}

fn write_private(path: &Path, bytes: &[u8]) -> Result<(), ActivationFlowError> {
    std::fs::write(path, bytes)
        .map_err(|error| ActivationFlowError::Delivery(error.to_string()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
            .map_err(|error| ActivationFlowError::Delivery(error.to_string()))?;
    }
    Ok(())
}

/// Resolve (or create) the node-bound identity used as the device public-key
/// anchor for activation and lease verification.
pub fn resolve_flow_node_identity(config_dir: &Path) -> Result<NodeIdentity, ActivationFlowError> {
    load_or_create_node_identity(config_dir, "focusa")
        .map_err(|error| ActivationFlowError::Delivery(format!("{error}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use focusa_license::activation_client::{
        ActivationStartReply, CheckoutOutcome, PollOutcome, PublicOffer,
    };
    use focusa_license::activation_facade::{ActivationError, ActivationErrorCode};
    use focusa_license::activation_reducer::ActivationTransition;
    use std::collections::{BTreeMap, VecDeque};

    enum AuthorityReply {
        Steps(Vec<ActivationTransition>),
        Start(ActivationStartReply),
        Checkout(CheckoutOutcome),
        Poll(PollOutcome),
        Offers(Vec<PublicOffer>),
        Nodes(Vec<String>),
        Link(String),
    }

    struct ScriptedAuthority {
        script: std::sync::Mutex<
            BTreeMap<&'static str, VecDeque<Result<AuthorityReply, ActivationErrorCode>>>,
        >,
    }

    impl ScriptedAuthority {
        fn new() -> Self {
            Self {
                script: std::sync::Mutex::new(BTreeMap::new()),
            }
        }

        fn push(
            &self,
            operation: &'static str,
            reply: Result<AuthorityReply, ActivationErrorCode>,
        ) -> &Self {
            self.script
                .lock()
                .unwrap()
                .entry(operation)
                .or_default()
                .push_back(reply);
            self
        }

        fn pop(&self, operation: &'static str) -> Result<AuthorityReply, ActivationError> {
            let mut script = self.script.lock().unwrap();
            script
                .get_mut(operation)
                .and_then(VecDeque::pop_front)
                .map(|reply| {
                    reply.map_err(|code| ActivationError::new(code, "request-0001".into()))
                })
                .unwrap_or_else(|| {
                    Err(ActivationError::new(
                        ActivationErrorCode::AuthorityUnavailable,
                        "request-0001".into(),
                    ))
                })
        }
    }

    impl ActivationAuthority for ScriptedAuthority {
        fn start(
            &self,
            _context: &ActivationRequestContext,
            _email: &str,
            _public_product_code: &str,
            _device_public_key: Option<&str>,
        ) -> Result<ActivationStartReply, ActivationError> {
            match self.pop("activation.start")? {
                AuthorityReply::Start(reply) => Ok(reply),
                _ => panic!("scripted authority: start expected Start reply"),
            }
        }
        fn verify(
            &self,
            _context: &ActivationRequestContext,
            _registration_id: &str,
            _one_time_verifier: &str,
        ) -> Result<Vec<ActivationTransition>, ActivationError> {
            match self.pop("activation.verify")? {
                AuthorityReply::Steps(steps) => Ok(steps),
                _ => panic!("scripted authority: verify expected Steps reply"),
            }
        }
        fn offers(
            &self,
            _context: &ActivationRequestContext,
            _registration_id: &str,
        ) -> Result<Vec<PublicOffer>, ActivationError> {
            match self.pop("activation.offers")? {
                AuthorityReply::Offers(offers) => Ok(offers),
                _ => panic!("scripted authority: offers expected Offers reply"),
            }
        }
        fn select_offer(
            &self,
            _context: &ActivationRequestContext,
            _registration_id: &str,
            _public_product_code: &str,
            _journey: ActivationJourney,
        ) -> Result<Vec<ActivationTransition>, ActivationError> {
            match self.pop("activation.select_offer")? {
                AuthorityReply::Steps(steps) => Ok(steps),
                _ => panic!("scripted authority: select_offer expected Steps reply"),
            }
        }
        fn checkout(
            &self,
            _context: &ActivationRequestContext,
            _registration_id: &str,
            _safe_redirect_handle: Option<&str>,
        ) -> Result<CheckoutOutcome, ActivationError> {
            match self.pop("activation.checkout")? {
                AuthorityReply::Checkout(outcome) => Ok(outcome),
                _ => panic!("scripted authority: checkout expected Checkout reply"),
            }
        }
        fn existing_license(
            &self,
            _context: &ActivationRequestContext,
            _registration_id: &str,
            _human_license_key: &str,
            _device_public_key: Option<&str>,
        ) -> Result<Vec<ActivationTransition>, ActivationError> {
            match self.pop("activation.existing_license")? {
                AuthorityReply::Steps(steps) => Ok(steps),
                _ => panic!("scripted authority: existing_license expected Steps reply"),
            }
        }
        fn poll(
            &self,
            _context: &ActivationRequestContext,
            _registration_id: &str,
            _poll_credential: &SensitiveCredential,
            _device_public_key: Option<&str>,
        ) -> Result<PollOutcome, ActivationError> {
            match self.pop("activation.poll")? {
                AuthorityReply::Poll(outcome) => Ok(outcome),
                _ => panic!("scripted authority: poll expected Poll reply"),
            }
        }
        fn refresh(
            &self,
            _context: &ActivationRequestContext,
            _node_id: &str,
            _refresh_credential: &SensitiveCredential,
            _current_sequence: u64,
        ) -> Result<Vec<ActivationTransition>, ActivationError> {
            match self.pop("lease.refresh")? {
                AuthorityReply::Steps(steps) => Ok(steps),
                _ => panic!("scripted authority: refresh expected Steps reply"),
            }
        }
        fn nodes(
            &self,
            _context: &ActivationRequestContext,
        ) -> Result<Vec<String>, ActivationError> {
            match self.pop("nodes.list")? {
                AuthorityReply::Nodes(nodes) => Ok(nodes),
                _ => panic!("scripted authority: nodes expected Nodes reply"),
            }
        }
        fn deactivate_node(
            &self,
            _context: &ActivationRequestContext,
            _node_id: &str,
        ) -> Result<Vec<ActivationTransition>, ActivationError> {
            match self.pop("nodes.deactivate")? {
                AuthorityReply::Steps(steps) => Ok(steps),
                _ => panic!("scripted authority: deactivate expected Steps reply"),
            }
        }
        fn manage_link(
            &self,
            _context: &ActivationRequestContext,
            _safe_redirect_handle: Option<&str>,
        ) -> Result<String, ActivationError> {
            match self.pop("account.manage_link")? {
                AuthorityReply::Link(link) => Ok(link),
                _ => panic!("scripted authority: manage_link expected Link reply"),
            }
        }
    }

    fn scripted_paid_start() -> ScriptedAuthority {
        let authority = ScriptedAuthority::new();
        authority.push(
            "activation.start",
            Ok(AuthorityReply::Start(ActivationStartReply {
                transitions: vec![ActivationTransition::ChallengeDelivered],
                registration_id: None,
                poll_credential: Some("poll-secret".into()),
            })),
        );
        authority.push("activation.offers", Ok(AuthorityReply::Offers(Vec::new())));
        authority
    }

    fn scripted_verified() -> ScriptedAuthority {
        let authority = scripted_paid_start();
        authority.push(
            "activation.verify",
            Ok(AuthorityReply::Steps(vec![
                ActivationTransition::EmailVerified,
                ActivationTransition::AccountPromoted,
            ])),
        );
        authority
    }

    #[test]
    fn paid_terminal_flow_renders_frozen_presenter_states_and_redacts() {
        unsafe {
            std::env::set_var("FOCUSA_ACTIVATION_BYPASS_DISABLE", "1");
        }
        let authority = scripted_verified();
        authority.push(
            "activation.select_offer",
            Ok(AuthorityReply::Steps(vec![
                ActivationTransition::OfferSelected,
            ])),
        );
        authority.push(
            "activation.checkout",
            Ok(AuthorityReply::Checkout(CheckoutOutcome {
                transitions: vec![ActivationTransition::CheckoutStarted],
                safe_url: Some("https://install.focusa.dev/pay/opaque-token".into()),
            })),
        );
        authority.push(
            "activation.poll",
            Ok(AuthorityReply::Poll(PollOutcome {
                transitions: vec![ActivationTransition::EntitlementIssued],
                one_time_key_envelope: Some("base64:key-envelope".into()),
                node_id: None,
                lease_envelope: None,
            })),
        );
        authority.push(
            "activation.poll",
            Ok(AuthorityReply::Poll(PollOutcome {
                transitions: vec![
                    ActivationTransition::TerminalDeliveryReady,
                    ActivationTransition::DeviceRegistered,
                    ActivationTransition::LeaseIssued,
                    ActivationTransition::Delivered,
                ],
                one_time_key_envelope: None,
                node_id: Some("node-0001".into()),
                lease_envelope: Some(
                    r#"{"schema":"focusa.lease_delivery_envelope.v1","key_set":{"schema":"focusa.signed_envelope.v1","signer_key_id":"key-001","payload_b64":"cA","signature_b64":"cw"},"lease":{"schema":"focusa.signed_envelope.v1","signer_key_id":"key-001","payload_b64":"cA","signature_b64":"cw"}}"#
                        .into(),
                ),
            })),
        );

        let mut input = ScriptedFlowInput::new(["customer@example.com", "483921", "2"]);
        let outcome = run_activation_flow(
            authority,
            CLI_FLOW,
            &mut input,
            None,
            Some("device-pub-key".into()),
            None,
            None,
            false,
            None,
        )
        .expect("paid terminal flow");

        assert_eq!(outcome.presenter_state, "activated");
        assert!(outcome.terminal);
        assert_eq!(outcome.masked_email.as_deref(), Some("c***@example.com"));
        let states: Vec<&str> = outcome
            .events
            .iter()
            .map(|event| event.presenter_state.as_str())
            .collect();
        assert_eq!(
            states,
            vec![
                "email_verification_pending",
                "selection_required",
                "checkout_required",
                "payment_pending",
                "license_delivery_ready",
                "activated",
            ]
        );
        assert_eq!(outcome.ledger.len(), 10);
        let serialized = serde_json::to_string(&outcome.events).unwrap();
        assert!(!serialized.contains("customer@example.com"));
        assert!(!serialized.contains("poll-secret"));
        assert!(!serialized.contains("483921"));
    }

    #[test]
    fn existing_key_flow_settles_activated_without_checkout() {
        unsafe {
            std::env::set_var("FOCUSA_ACTIVATION_BYPASS_DISABLE", "1");
        }
        let authority = scripted_verified();
        authority.push(
            "activation.existing_license",
            Ok(AuthorityReply::Steps(vec![
                ActivationTransition::ExistingKeyChosen,
                ActivationTransition::EntitlementIssued,
                ActivationTransition::TerminalDeliveryReady,
                ActivationTransition::DeviceRegistered,
                ActivationTransition::LeaseIssued,
                ActivationTransition::Delivered,
            ])),
        );
        let mut input = ScriptedFlowInput::new(["owner@example.com", "000000", "1", "FOCUSA-XXXX"]);
        let outcome = run_activation_flow(
            authority, CLI_FLOW, &mut input, None, None, None, None, false, None,
        )
        .expect("existing key flow");
        assert_eq!(outcome.presenter_state, "activated");
        assert!(outcome.terminal);
        let states: Vec<&str> = outcome
            .events
            .iter()
            .map(|event| event.presenter_state.as_str())
            .collect();
        assert_eq!(
            states,
            vec![
                "email_verification_pending",
                "selection_required",
                "activated"
            ]
        );
        assert_eq!(outcome.ledger.len(), 9);
    }

    #[test]
    fn limited_access_spec172_overlay_settles_activated_without_checkout() {
        unsafe {
            std::env::set_var("FOCUSA_ACTIVATION_BYPASS_DISABLE", "1");
        }
        let authority = scripted_verified();
        authority.push(
            "activation.select_offer",
            Ok(AuthorityReply::Steps(vec![
                ActivationTransition::LimitedAccessChosen,
            ])),
        );
        authority.push(
            "activation.poll",
            Ok(AuthorityReply::Poll(PollOutcome {
                transitions: vec![
                    ActivationTransition::DeviceRegistered,
                    ActivationTransition::LeaseIssued,
                    ActivationTransition::Delivered,
                ],
                one_time_key_envelope: None,
                node_id: Some("node-0001".into()),
                lease_envelope: None,
            })),
        );
        let mut input = ScriptedFlowInput::new(["eval@example.com", "000000", "3"]);
        let outcome = run_activation_flow(
            authority, CLI_FLOW, &mut input, None, None, None, None, false, None,
        )
        .expect("limited access flow");
        assert_eq!(outcome.presenter_state, "activated");
        assert!(outcome.terminal);
        let states: Vec<&str> = outcome
            .events
            .iter()
            .map(|event| event.presenter_state.as_str())
            .collect();
        assert_eq!(
            states,
            vec![
                "email_verification_pending",
                "selection_required",
                "selection_required",
                "activated",
            ]
        );
        assert_eq!(outcome.ledger.len(), 7);
    }

    #[test]
    fn verification_expiry_settles_recovery_only() {
        unsafe {
            std::env::set_var("FOCUSA_ACTIVATION_BYPASS_DISABLE", "1");
        }
        let authority = scripted_paid_start();
        authority.push(
            "activation.verify",
            Ok(AuthorityReply::Steps(vec![
                ActivationTransition::Expired,
                ActivationTransition::RecoveryOnly,
            ])),
        );
        let mut input = ScriptedFlowInput::new(["customer@example.com", "483921"]);
        let outcome = run_activation_flow(
            authority, CLI_FLOW, &mut input, None, None, None, None, false, None,
        )
        .expect("verification expiry");
        assert_eq!(outcome.presenter_state, "recovery_only");
        assert!(outcome.terminal);
    }

    #[test]
    fn checkout_timeout_cancels_fail_closed_to_recovery_only() {
        unsafe {
            std::env::set_var("FOCUSA_ACTIVATION_BYPASS_DISABLE", "1");
        }
        let authority = scripted_verified();
        authority.push(
            "activation.select_offer",
            Ok(AuthorityReply::Steps(vec![
                ActivationTransition::OfferSelected,
            ])),
        );
        authority.push(
            "activation.checkout",
            Ok(AuthorityReply::Checkout(CheckoutOutcome {
                transitions: vec![ActivationTransition::CheckoutStarted],
                safe_url: Some("https://install.focusa.dev/pay/opaque-token".into()),
            })),
        );
        let mut input = ScriptedFlowInput::new(["customer@example.com", "483921", "2"]);
        let outcome = run_activation_flow(
            authority,
            CLI_FLOW,
            &mut input,
            None,
            None,
            None,
            Some(0),
            false,
            None,
        )
        .expect("timeout cancels");
        assert_eq!(outcome.presenter_state, "recovery_only");
        assert!(outcome.terminal);
        let states: Vec<&str> = outcome
            .events
            .iter()
            .map(|event| event.presenter_state.as_str())
            .collect();
        assert_eq!(states[3], "payment_pending");
        assert_eq!(states[4], "recovery_only");
    }

    #[test]
    fn poll_error_recovery_only_renders_typed_code_and_next_action() {
        unsafe {
            std::env::set_var("FOCUSA_ACTIVATION_BYPASS_DISABLE", "1");
        }
        let authority = scripted_verified();
        authority.push(
            "activation.select_offer",
            Ok(AuthorityReply::Steps(vec![
                ActivationTransition::OfferSelected,
            ])),
        );
        authority.push(
            "activation.checkout",
            Ok(AuthorityReply::Checkout(CheckoutOutcome {
                transitions: vec![ActivationTransition::CheckoutStarted],
                safe_url: None,
            })),
        );
        authority.push("activation.poll", Err(ActivationErrorCode::Refunded));
        let mut input = ScriptedFlowInput::new(["customer@example.com", "483921", "2"]);
        let outcome = run_activation_flow(
            authority, CLI_FLOW, &mut input, None, None, None, None, false, None,
        )
        .expect("refund settles recovery");
        assert_eq!(outcome.presenter_state, "recovery_only");
        assert!(outcome.terminal);
        assert!(outcome.events.iter().any(|event| {
            event.error_code.as_deref() == Some("REFUNDED")
                && event.next_action.as_deref() == Some("recovery_only")
        }));
    }

    #[test]
    fn resume_flow_continues_bounded_poll_to_delivery() {
        let authority = ScriptedAuthority::new();
        authority.push(
            "activation.poll",
            Ok(AuthorityReply::Poll(PollOutcome {
                transitions: vec![
                    ActivationTransition::EntitlementIssued,
                    ActivationTransition::TerminalDeliveryReady,
                    ActivationTransition::DeviceRegistered,
                    ActivationTransition::LeaseIssued,
                    ActivationTransition::Delivered,
                ],
                one_time_key_envelope: None,
                node_id: Some("node-0001".into()),
                lease_envelope: None,
            })),
        );
        let registration = ActivationRegistration {
            schema: "focusa.activation_registration.v1".into(),
            registration_id: "registration-0001".into(),
            facade_id: "focusa-cli".into(),
            presenter: "cli".into(),
            install_channel: "source_build".into(),
            state: ActivationState::CheckoutPending,
            masked_email: Some("c***@example.com".into()),
            poll_count: 0,
            max_polls: 40,
        };
        let credential = SensitiveCredential::new("poll-secret".into()).unwrap();
        let outcome = resume_activation_flow(
            authority,
            CLI_FLOW,
            registration,
            credential,
            None,
            false,
            None,
        )
        .expect("resumed poll");
        assert_eq!(outcome.presenter_state, "activated");
        assert!(outcome.terminal);
        let states: Vec<&str> = outcome
            .events
            .iter()
            .map(|event| event.presenter_state.as_str())
            .collect();
        assert_eq!(states, vec!["payment_pending", "activated"]);
    }

    #[test]
    fn recovery_only_resume_never_regrants() {
        let authority = ScriptedAuthority::new();
        let registration = ActivationRegistration {
            schema: "focusa.activation_registration.v1".into(),
            registration_id: "registration-0001".into(),
            facade_id: "focusa-cli".into(),
            presenter: "cli".into(),
            install_channel: "source_build".into(),
            state: ActivationState::RecoveryOnly,
            masked_email: Some("c***@example.com".into()),
            poll_count: 0,
            max_polls: 40,
        };
        let credential = SensitiveCredential::new("poll-secret".into()).unwrap();
        let outcome = resume_activation_flow(
            authority,
            CLI_FLOW,
            registration,
            credential,
            None,
            false,
            None,
        )
        .expect("recovery resume");
        assert_eq!(outcome.presenter_state, "recovery_only");
        assert!(outcome.terminal);
        assert!(outcome.ledger.is_empty());
    }

    #[test]
    fn empty_email_fails_closed_without_authority_call() {
        let authority = ScriptedAuthority::new();
        let mut input = ScriptedFlowInput::new([""]);
        let error = run_activation_flow(
            authority, CLI_FLOW, &mut input, None, None, None, None, false, None,
        )
        .unwrap_err();
        assert!(matches!(error, ActivationFlowError::EmailRequired));
    }

    #[test]
    fn unmaskable_email_fails_closed_before_authority_call() {
        let authority = ScriptedAuthority::new();
        let mut input = ScriptedFlowInput::new(["not-an-email", "000000"]);
        let error = run_activation_flow(
            authority, CLI_FLOW, &mut input, None, None, None, None, false, None,
        )
        .unwrap_err();
        assert!(matches!(
            error,
            ActivationFlowError::Client(ActivationClientError::UnmaskableEmail)
        ));
    }

    #[test]
    fn journey_hint_selects_limited_access_without_prompt() {
        let authority = scripted_verified();
        authority.push(
            "activation.select_offer",
            Ok(AuthorityReply::Steps(vec![
                ActivationTransition::LimitedAccessChosen,
            ])),
        );
        authority.push(
            "activation.poll",
            Ok(AuthorityReply::Poll(PollOutcome {
                transitions: vec![
                    ActivationTransition::DeviceRegistered,
                    ActivationTransition::LeaseIssued,
                    ActivationTransition::Delivered,
                ],
                one_time_key_envelope: None,
                node_id: Some("node-0001".into()),
                lease_envelope: None,
            })),
        );
        // Only email + code prompts; no journey menu prompt.
        let mut input = ScriptedFlowInput::new(["eval@example.com", "000000"]);
        let outcome = run_activation_flow(
            authority,
            CLI_FLOW,
            &mut input,
            None,
            None,
            Some(ActivationJourney::LimitedAccess),
            None,
            false,
            None,
        )
        .expect("limited access hint");
        assert_eq!(outcome.presenter_state, "activated");
        assert!(outcome.terminal);
    }

    #[test]
    fn snapshot_and_credential_persistence_are_private_and_resumable() {
        let directory = std::env::temp_dir().join(format!("focusa-flow-{}", uuid::Uuid::now_v7()));
        let registration = ActivationRegistration {
            schema: "focusa.activation_registration.v1".into(),
            registration_id: "registration-0001".into(),
            facade_id: "focusa-cli".into(),
            presenter: "cli".into(),
            install_channel: "source_build".into(),
            state: ActivationState::CheckoutPending,
            masked_email: Some("c***@example.com".into()),
            poll_count: 1,
            max_polls: 40,
        };
        let path = persist_registration_snapshot(&directory, &registration).unwrap();
        assert!(path.exists());
        let store = focusa_license::authority_credentials::InMemoryCredentialStore::default();
        let credential = SensitiveCredential::new("poll-secret".into()).unwrap();
        persist_poll_credential(&store, "registration-0001", &credential).unwrap();
        let loaded = load_poll_credential(&store, "registration-0001").unwrap();
        assert_eq!(loaded.expose_for_protected_store(), "poll-secret");
        let round_trip = load_registration_snapshot(&directory, "registration-0001").unwrap();
        assert_eq!(round_trip, registration);
        // Snapshot JSON never contains the poll credential.
        let raw = std::fs::read_to_string(&path).unwrap();
        assert!(!raw.contains("poll-secret"));
        std::fs::remove_dir_all(&directory).unwrap();
    }

    // ── Spec 152E §14.2 agent/JSON protocol ───────────────────────────────

    #[test]
    fn agent_begin_returns_typed_human_action_envelope_and_handle_without_prompt() {
        unsafe {
            std::env::set_var("FOCUSA_ACTIVATION_BYPASS_DISABLE", "1");
        }
        let authority = scripted_paid_start();
        let outcome = run_agent_activation(
            authority,
            CLI_FLOW,
            Some("customer@example.com".into()),
            Some("device-pub-key".into()),
            AgentKeyReveal::denied(),
            None,
        )
        .expect("agent begin");
        assert!(!outcome.terminal);
        assert_eq!(
            outcome.envelope.schema,
            "focusa.agent_activation_envelope.v1"
        );
        assert_eq!(outcome.envelope.state, "email_verification_pending");
        assert!(outcome.envelope.human_action_required);
        assert_eq!(
            outcome.envelope.human_action.as_deref(),
            Some("enter_verification_code")
        );
        assert_eq!(
            outcome.envelope.masked_email.as_deref(),
            Some("c***@example.com")
        );
        assert_eq!(outcome.registration_id, outcome.envelope.registration_id);
        let body = serde_json::to_string(&outcome.envelope).unwrap();
        assert!(!body.contains("customer@example.com"));
        assert!(!body.contains("poll-secret"));
    }

    #[test]
    fn agent_start_persists_snapshot_and_poll_credential_for_resume() {
        unsafe {
            std::env::set_var("FOCUSA_ACTIVATION_BYPASS_DISABLE", "1");
        }
        let directory = std::env::temp_dir().join(format!("focusa-agent-{}", uuid::Uuid::now_v7()));
        let store = focusa_license::authority_credentials::InMemoryCredentialStore::default();
        let persist =
            |session: &ActivationSession<ScriptedAuthority>| -> Result<(), ActivationFlowError> {
                persist_agent_session_state(&directory, session, &store)
            };
        let outcome = run_agent_activation(
            scripted_paid_start(),
            CLI_FLOW,
            Some("customer@example.com".into()),
            Some("device-pub-key".into()),
            AgentKeyReveal::denied(),
            Some(&persist),
        )
        .expect("agent begin with persistence");
        // Snapshot exists on disk and keyring entry is loadable by registration id.
        let round_trip = load_registration_snapshot(&directory, &outcome.registration_id).unwrap();
        assert_eq!(round_trip.registration_id, outcome.registration_id);
        let credential = load_poll_credential(&store, &outcome.registration_id).unwrap();
        assert!(!credential.expose_for_protected_store().is_empty());
        std::fs::remove_dir_all(&directory).unwrap();
    }

    #[test]
    fn agent_start_creates_missing_activation_directory() {
        unsafe {
            std::env::set_var("FOCUSA_ACTIVATION_BYPASS_DISABLE", "1");
        }
        let directory =
            std::env::temp_dir().join(format!("focusa-agent-missing-{}", uuid::Uuid::now_v7()));
        assert!(!directory.join("activation").exists());
        let store = focusa_license::authority_credentials::InMemoryCredentialStore::default();
        let persist =
            |session: &ActivationSession<ScriptedAuthority>| -> Result<(), ActivationFlowError> {
                persist_agent_session_state(&directory, session, &store)
            };
        run_agent_activation(
            scripted_paid_start(),
            CLI_FLOW,
            Some("customer@example.com".into()),
            Some("device-pub-key".into()),
            AgentKeyReveal::denied(),
            Some(&persist),
        )
        .expect("begin with fresh directory");
        assert!(directory.join("activation").exists() || directory.exists());
        let _ = std::fs::remove_dir_all(&directory);
    }

    #[test]
    fn agent_begin_without_email_fails_closed_without_authority_call() {
        let authority = ScriptedAuthority::new();
        let error = run_agent_activation(
            authority,
            CLI_FLOW,
            None,
            None,
            AgentKeyReveal::denied(),
            None,
        )
        .unwrap_err();
        assert!(matches!(error, ActivationFlowError::EmailRequired));
    }

    #[test]
    fn agent_resume_polls_boundedly_and_returns_human_action_payment_envelope() {
        let authority = ScriptedAuthority::new();
        authority.push(
            "activation.poll",
            Ok(AuthorityReply::Poll(PollOutcome {
                transitions: vec![ActivationTransition::CheckoutStarted],
                one_time_key_envelope: None,
                node_id: None,
                lease_envelope: None,
            })),
        );
        let registration = ActivationRegistration {
            schema: "focusa.activation_registration.v1".into(),
            registration_id: "registration-0001".into(),
            facade_id: "focusa-cli".into(),
            presenter: "cli".into(),
            install_channel: "source_build".into(),
            state: ActivationState::OfferSelected,
            masked_email: Some("c***@example.com".into()),
            poll_count: 0,
            max_polls: 40,
        };
        let credential = SensitiveCredential::new("poll-secret".into()).unwrap();
        let outcome = resume_agent_activation(
            authority,
            CLI_FLOW,
            registration,
            credential,
            None,
            AgentKeyReveal::denied(),
            None,
        )
        .expect("agent resume");
        assert!(!outcome.terminal);
        assert_eq!(outcome.envelope.state, "payment_pending");
        assert!(outcome.envelope.human_action_required);
        assert_eq!(
            outcome.envelope.human_action.as_deref(),
            Some("complete_payment_then_poll")
        );
        assert_eq!(outcome.envelope.next_action, "complete_payment_then_poll");
        assert_eq!(outcome.envelope.poll_count, 1);
    }

    #[test]
    fn agent_resume_settles_terminal_delivery_with_key_masked_by_default() {
        let authority = ScriptedAuthority::new();
        authority.push(
            "activation.poll",
            Ok(AuthorityReply::Poll(PollOutcome {
                transitions: vec![
                    ActivationTransition::EntitlementIssued,
                    ActivationTransition::TerminalDeliveryReady,
                    ActivationTransition::DeviceRegistered,
                    ActivationTransition::LeaseIssued,
                    ActivationTransition::Delivered,
                ],
                one_time_key_envelope: Some("base64:key-envelope".into()),
                node_id: Some("node-0001".into()),
                lease_envelope: None,
            })),
        );
        let registration = ActivationRegistration {
            schema: "focusa.activation_registration.v1".into(),
            registration_id: "registration-0001".into(),
            facade_id: "focusa-cli".into(),
            presenter: "cli".into(),
            install_channel: "source_build".into(),
            state: ActivationState::CheckoutPending,
            masked_email: Some("c***@example.com".into()),
            poll_count: 0,
            max_polls: 40,
        };
        let credential = SensitiveCredential::new("poll-secret".into()).unwrap();
        let outcome = resume_agent_activation(
            authority,
            CLI_FLOW,
            registration,
            credential,
            None,
            AgentKeyReveal::denied(),
            None,
        )
        .expect("agent resume to delivery");
        assert!(outcome.terminal);
        assert_eq!(outcome.envelope.state, "activated");
        assert!(outcome.envelope.key_present);
        assert!(
            !outcome.envelope.key_visible,
            "key masked by default for agents"
        );
        let body = serde_json::to_string(&outcome.envelope).unwrap();
        assert!(!body.contains("key-envelope"));
        assert!(!body.contains("full-key-envelope"));
    }

    #[test]
    fn agent_resume_recovery_only_never_regrants_and_carries_typed_error() {
        let authority = ScriptedAuthority::new();
        authority.push("activation.poll", Err(ActivationErrorCode::Refunded));
        let registration = ActivationRegistration {
            schema: "focusa.activation_registration.v1".into(),
            registration_id: "registration-0001".into(),
            facade_id: "focusa-cli".into(),
            presenter: "cli".into(),
            install_channel: "source_build".into(),
            state: ActivationState::CheckoutPending,
            masked_email: Some("c***@example.com".into()),
            poll_count: 0,
            max_polls: 40,
        };
        let credential = SensitiveCredential::new("poll-secret".into()).unwrap();
        let outcome = resume_agent_activation(
            authority,
            CLI_FLOW,
            registration,
            credential,
            None,
            AgentKeyReveal::denied(),
            None,
        )
        .expect("refund settles recovery");
        assert!(outcome.terminal);
        assert_eq!(outcome.envelope.state, "recovery_only");
        assert_eq!(outcome.envelope.next_action, "recovery_only");
        assert_eq!(outcome.envelope.error.as_ref().unwrap().code, "REFUNDED");
        assert_eq!(
            outcome.envelope.error.as_ref().unwrap().next_action,
            "recovery_only"
        );
    }

    #[test]
    fn agent_timeout_cancels_fail_closed_to_recovery_only() {
        let authority = ScriptedAuthority::new();
        let registration = ActivationRegistration {
            schema: "focusa.activation_registration.v1".into(),
            registration_id: "registration-0001".into(),
            facade_id: "focusa-cli".into(),
            presenter: "cli".into(),
            install_channel: "source_build".into(),
            state: ActivationState::CheckoutPending,
            masked_email: Some("c***@example.com".into()),
            poll_count: 0,
            max_polls: 40,
        };
        let credential = SensitiveCredential::new("poll-secret".into()).unwrap();
        let outcome = resume_agent_activation(
            authority,
            CLI_FLOW,
            registration,
            credential,
            Some(0),
            AgentKeyReveal::denied(),
            None,
        )
        .expect("timeout cancels");
        assert!(outcome.terminal);
        assert_eq!(outcome.envelope.state, "recovery_only");
    }
}
