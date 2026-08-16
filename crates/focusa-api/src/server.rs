//! HTTP server setup.
//!
//! The API server is a thin read/write facade:
//!   - Reads: snapshot current state via Arc<RwLock<FocusaState>>
//!   - Writes: dispatch Actions via mpsc::Sender<Action> to the daemon event loop
//!
//! The daemon owns the state; the API borrows a read handle and a command channel.

use crate::middleware;
use crate::routes;
use crate::routes::bounded::{observe_resource_mode_transition, resource_mode_status};
use crate::routes::sse::EventBroadcaster;
use crate::scoped_store::ScopedCrdtLedger;
use axum::middleware as axum_mw;
use axum::{Router, extract::DefaultBodyLimit};
use focusa_core::prediction::PredictionValue;
use focusa_core::runtime::persistence_actor::PersistenceActor;
use focusa_core::runtime::persistence_sqlite::SqlitePersistence;
use focusa_core::scoped_state::WorkstreamKey;
use tower_http::services::ServeDir;

/// Vendored static files directory (e.g. jsQR for offline PWA /scan pages).
const STATIC_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/static");
use focusa_core::types::{
    Action, EventLogEntry, FocusStackState, FocusaConfig, FocusaState, WorkLoopPolicy,
    WorkLoopPolicyOverrides, WorkLoopPreset, WorkLoopStatus,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::process::{Child, ChildStdin};
use tokio::sync::RwLock as TokioRwLock;
use tokio::sync::{Mutex, RwLock, broadcast, mpsc};
use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;

#[cfg(all(target_os = "linux", target_env = "gnu"))]
unsafe extern "C" {
    fn malloc_trim(pad: usize) -> std::os::raw::c_int;
}

fn allocator_trim_interval_secs() -> u64 {
    std::env::var("FOCUSA_ALLOCATOR_TRIM_INTERVAL_SECS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(30)
}

fn trim_allocator_once() -> bool {
    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    {
        // SAFETY: malloc_trim is a process-local glibc allocator maintenance call.
        // It does not touch Rust-owned references; glibc serializes allocator internals.
        unsafe { malloc_trim(0) != 0 }
    }
    #[cfg(not(all(target_os = "linux", target_env = "gnu")))]
    {
        false
    }
}

async fn allocator_trim_loop() {
    let interval_secs = allocator_trim_interval_secs();
    if interval_secs == 0 {
        tracing::info!("allocator trim loop disabled");
        return;
    }
    let mut interval = tokio::time::interval(Duration::from_secs(interval_secs.max(5)));
    loop {
        interval.tick().await;
        let trimmed = tokio::task::spawn_blocking(trim_allocator_once)
            .await
            .unwrap_or(false);
        tracing::debug!(trimmed, "allocator trim tick");
    }
}

fn resource_mode_monitor_interval_secs() -> u64 {
    std::env::var("FOCUSA_RESOURCE_MODE_MONITOR_INTERVAL_SECS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(15)
}

fn drain_oldest<T>(items: &mut Vec<T>, limit: usize) -> usize {
    if items.len() <= limit {
        return 0;
    }
    let overflow = items.len() - limit;
    items.drain(0..overflow);
    overflow
}

async fn prune_pressure_sensitive_state(state: &Arc<AppState>, trace_limit: usize) -> usize {
    let trace_limit = trace_limit.max(1);
    let tool_call_limit = trace_limit.saturating_mul(2).max(50);
    let ledger_limit = trace_limit.max(50);
    let token_limit = trace_limit.max(100);
    let mut focusa = state.focusa.write().await;
    let pruned = drain_oldest(&mut focusa.telemetry.trace_events, trace_limit)
        + drain_oldest(&mut focusa.telemetry.tool_calls, tool_call_limit)
        + drain_oldest(&mut focusa.telemetry.secondary_loop_ledger, ledger_limit)
        + drain_oldest(&mut focusa.telemetry.tokens_per_task, token_limit)
        + drain_oldest(&mut focusa.anticipated_context, 8);
    // Spec98 §13.11/13.12: pressure pruning only trims telemetry/advisory/runtime buffers.
    // It must not advance canonical cognition freshness (`FocusaState.version`).
    drop(focusa);
    if pruned > 0 {
        state.mark_external_mutation();
    }
    pruned
}

async fn resource_mode_monitor_loop(state: Arc<AppState>) {
    let interval_secs = resource_mode_monitor_interval_secs();
    if interval_secs == 0 {
        tracing::info!("resource mode monitor disabled");
        return;
    }
    let mut interval = tokio::time::interval(Duration::from_secs(interval_secs.max(5)));
    loop {
        interval.tick().await;
        let active_session_id = {
            let focusa = state.focusa.read().await;
            focusa
                .session
                .as_ref()
                .map(|session| session.session_id.to_string())
        };
        let status =
            observe_resource_mode_transition("background_resource_monitor", active_session_id);
        if matches!(status.mode, "lowmem" | "emergency") {
            let mode = status.mode;
            let reason = status.reason;
            let pruned =
                prune_pressure_sensitive_state(&state, status.retention_policy.trace_event_limit)
                    .await;
            let trimmed = tokio::task::spawn_blocking(trim_allocator_once)
                .await
                .unwrap_or(false);
            tracing::debug!(
                mode = %mode,
                reason = %reason,
                pruned,
                trimmed,
                "resource pressure triggered bounded-state prune and allocator trim"
            );
        }
        if let Some(transition) = status.latest_transition.as_ref() {
            tracing::debug!(
                transition_id = %transition.transition_id,
                mode = %status.mode,
                reason = %status.reason,
                durability = %transition.durability,
                "resource mode monitor observed transition"
            );
        }
    }
}

fn lowmem_background_throttle() -> Option<(String, String)> {
    let status = resource_mode_status();
    if matches!(status.mode, "lowmem" | "emergency") && status.budget.background_concurrency == 0 {
        Some((status.mode.to_string(), status.reason.to_string()))
    } else {
        None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandExecutionStatus {
    Accepted,
    Dispatched,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandLogEntry {
    pub ts: chrono::DateTime<chrono::Utc>,
    pub level: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandRecord {
    pub command_id: String,
    pub command: String,
    pub status: CommandExecutionStatus,
    pub submitted_at: chrono::DateTime<chrono::Utc>,
    pub dispatched_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub error: Option<String>,
    pub logs: Vec<CommandLogEntry>,
}

pub type CommandStore = Arc<RwLock<HashMap<String, CommandRecord>>>;

pub struct PiRpcSession {
    pub child: Child,
    pub process_group_id: u32,
    pub stdin: ChildStdin,
    pub session_id: String,
    pub cwd: Option<String>,
    pub idempotency_key: String,
    pub started_at: Instant,
}

#[derive(Default)]
pub struct SupervisorPerfCounters {
    pub ticks_total: AtomicU64,
    pub driver_start_attempts: AtomicU64,
    pub driver_stop_attempts: AtomicU64,
    pub dispatch_attempts: AtomicU64,
    pub dispatch_skipped_disallowed: AtomicU64,
    pub dispatch_recovery_restarts: AtomicU64,
    pub background_throttled_ticks: AtomicU64,
}

/// Shared state between API server and daemon.
#[derive(Clone, Debug, Serialize)]
pub struct WriterLease {
    pub writer_id: String,
    pub fencing_token: u64,
    pub acquired_at: chrono::DateTime<chrono::Utc>,
    pub renewed_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

pub struct AppState {
    /// Read-only snapshot of cognitive state (daemon writes, API reads).
    pub focusa: Arc<RwLock<FocusaState>>,
    /// Workstream-partitioned states (#125): additive migration foundation —
    /// routes opt in per workstream; the global state stays canonical until
    /// migration completes.
    pub workstream_states: crate::workstream_store::WorkstreamStateStore,
    /// Command channel to the daemon event loop.
    pub command_tx: mpsc::Sender<Action>,
    /// Event broadcast channel (SSE clients subscribe).
    pub events_tx: broadcast::Sender<String>,
    /// SSE event broadcaster for real-time TUI updates.
    #[allow(dead_code)]
    pub event_broadcaster: EventBroadcaster,
    pub config: FocusaConfig,
    /// Direct persistence access for sync routes.
    pub persistence: SqlitePersistence,
    /// Process-wide bounded single-writer for state snapshots and checkpoint acks.
    pub persistence_actor: Option<PersistenceActor>,
    /// Serializes canonical state writers across daemon actions and sync API routes.
    pub write_serial_lock: Arc<Mutex<()>>,
    /// In-memory command write-model state for /v1/commands/* endpoints.
    pub command_store: CommandStore,
    /// Token store for capability permissions (docs/25-26).
    pub token_store: Arc<RwLock<focusa_core::permissions::TokenStore>>,
    /// Scoped writer claims for continuous work-loop mutations, keyed by ProjectRootKey + WorkstreamKey + WorkItemKey.
    pub writer_claims: Arc<TokioRwLock<HashMap<String, WriterLease>>>,
    /// Process-monotonic source for writer fencing tokens. Zero is never issued.
    pub next_writer_fencing_token: Arc<AtomicU64>,
    /// FocusStackState by scope key — FS-01: Focus State reducer scope enforcement.
    pub focus_stack_by_scope: Arc<TokioRwLock<HashMap<String, FocusStackState>>>,
    /// Typed ProjectRootKey + WorkstreamKey scoped prediction CRDT ledger.
    pub prediction_store: Arc<ScopedCrdtLedger<PredictionValue>>,
    /// Request-local turn completion idempotency, keyed by typed WorkstreamKey.
    pub(crate) recent_completed_turns_by_scope:
        Arc<TokioRwLock<HashMap<WorkstreamKey, VecDeque<String>>>>,
    /// Focus snapshots keyed first by typed WorkstreamKey, never daemon-global.
    pub(crate) snapshots_by_scope: Arc<
        TokioRwLock<HashMap<WorkstreamKey, HashMap<String, routes::snapshots::SnapshotRecord>>>,
    >,
    /// Metacognition read model keyed first by typed WorkstreamKey, never daemon-global.
    pub(crate) metacog_by_scope:
        Arc<std::sync::Mutex<HashMap<WorkstreamKey, routes::metacognition::MetaStore>>>,
    /// Process start time for uptime reporting.
    pub started_at: Instant,
    /// Optional daemon-owned Pi RPC transport session for continuous work.
    pub pi_rpc_session: Arc<Mutex<Option<PiRpcSession>>>,
    /// Lightweight performance/backpressure counters for supervisor loop.
    pub supervisor_perf: Arc<SupervisorPerfCounters>,
    /// Monotonic signal for API routes that mutate shared state outside the daemon reducer.
    pub external_mutation_epoch: Arc<AtomicU64>,
}

impl AppState {
    pub fn mark_external_mutation(&self) -> u64 {
        self.external_mutation_epoch.fetch_add(1, Ordering::AcqRel) + 1
    }

    pub async fn persist_checkpoint(&self, state: FocusaState) -> anyhow::Result<()> {
        self.persist_events_checkpoint(Vec::new(), state).await
    }

    pub async fn persist_events_checkpoint(
        &self,
        events: Vec<EventLogEntry>,
        state: FocusaState,
    ) -> anyhow::Result<()> {
        if let Some(actor) = &self.persistence_actor {
            actor.persist_checkpoint(events, state).await
        } else {
            let persistence = self.persistence.clone();
            tokio::task::spawn_blocking(move || {
                for event in &events {
                    persistence.append_event(event)?;
                }
                persistence.save_state(&state)
            })
            .await
            .map_err(|error| anyhow::anyhow!("persistence worker join failed: {error}"))?
        }
    }

    pub async fn append_events_checkpoint(&self, events: Vec<EventLogEntry>) -> anyhow::Result<()> {
        if let Some(actor) = &self.persistence_actor {
            actor.append_events_checkpoint(events).await
        } else {
            let persistence = self.persistence.clone();
            tokio::task::spawn_blocking(move || {
                for event in &events {
                    persistence.append_event(event)?;
                }
                Ok(())
            })
            .await
            .map_err(|error| anyhow::anyhow!("persistence worker join failed: {error}"))?
        }
    }
}

/// CORS layer for the macOS Tauri menubar app (focusa-ui0y).
///
/// Allows:
///   - Tauri 2 origins: `tauri://localhost`, `http://tauri.localhost`,
///     `https://tauri.localhost`
///   - Vite dev server: `http://localhost:1420`, `http://127.0.0.1:1420`
///   - Anything explicitly listed in `FOCUSA_CORS_ALLOWED_ORIGINS` (comma-separated)
///
/// This is intentionally permissive for menubar client development; production
/// auth tokens are still required for write endpoints via `middleware::auth`.
pub fn menubar_cors_layer() -> CorsLayer {
    use axum::http::HeaderValue;
    use tower_http::cors::AllowOrigin;

    // Extract origin (scheme + host + port) from a URL string without
    // pulling in the `url` crate. Handles http://, https://, and URLs with
    // optional port.
    fn origin_from_url(s: &str) -> Option<String> {
        let s = s.trim().trim_end_matches('/');
        let (scheme, rest) = if let Some(r) = s.strip_prefix("https://") {
            ("https", r)
        } else {
            let r = s.strip_prefix("http://")?;
            ("http", r)
        };
        // rest is host[:port][/path]
        let host_port = rest.split('/').next().unwrap_or(rest);
        if host_port.is_empty() {
            return None;
        }
        Some(format!("{scheme}://{host_port}"))
    }

    let mut allowed: Vec<HeaderValue> = vec![
        HeaderValue::from_static("tauri://localhost"),
        HeaderValue::from_static("http://tauri.localhost"),
        HeaderValue::from_static("https://tauri.localhost"),
        HeaderValue::from_static("http://localhost:1420"),
        HeaderValue::from_static("http://127.0.0.1:1420"),
    ];
    // V2: The phone PWA at /connect/room/<id>/scan is loaded in the phone's
    // browser, NOT in a Tauri webview. The browser origin is whatever the
    // operator's daemon public URL is (e.g. https://focusa-vps.tail-net.ts.net).
    // Auto-allow that origin so the PWA's fetch(/join, /approve) succeeds
    // without manual CORS configuration.
    for var in &["FOCUSA_PAIRING_URL", "FOCUSA_PUBLIC_URL"] {
        if let Ok(public_url) = std::env::var(var) {
            if let Some(origin) = origin_from_url(&public_url) {
                if let Ok(hv) = HeaderValue::from_str(&origin) {
                    allowed.push(hv);
                }
            }
        }
    }
    if let Ok(extra) = std::env::var("FOCUSA_CORS_ALLOWED_ORIGINS") {
        for raw in extra.split(',') {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Ok(hv) = HeaderValue::from_str(trimmed) {
                allowed.push(hv);
            }
        }
    }
    CorsLayer::new()
        .allow_origin(AllowOrigin::list(allowed))
        .allow_methods(Any)
        .allow_headers(Any)
        .max_age(std::time::Duration::from_secs(86_400))
}

/// Build the axum Router with all routes.
/// V2: Global access to AppState for auth middleware token lookup.
static APP_STATE: std::sync::OnceLock<Arc<AppState>> = std::sync::OnceLock::new();

/// V2: Rehydrate PairingStore in-memory maps (connect_sessions, tokens) from
/// the SQLite ledger on daemon startup. Without this, the first /join or
/// /approve after a daemon restart would 404 (in-memory miss). The auth
/// middleware also rehydrates tokens on demand; this is the eager path.
pub async fn rehydrate_pairing_state_from_ledger(
    state: &Arc<AppState>,
) -> anyhow::Result<(usize, usize)> {
    use crate::routes::device_pairing::shared_state;
    let persistence = &state.persistence;
    let expired_pairing_rooms_cleaned = persistence.cleanup_expired_pairing_rooms().unwrap_or_else(|err| {
        tracing::warn!(error = %err, "V2: expired pairing room cleanup failed during startup rehydrate");
        0
    });
    if expired_pairing_rooms_cleaned > 0 {
        tracing::info!(
            expired_pairing_rooms_cleaned = expired_pairing_rooms_cleaned,
            "V2: expired pairing rooms cleaned during startup rehydrate"
        );
    }
    let shared = shared_state();
    let mut s = shared.write().await;
    // Load all non-expired connect_sessions.
    let now = chrono::Utc::now();
    let now_str = now.to_rfc3339();
    // Use the list helper to enumerate rooms.
    if let Ok(persisted) = persistence.list_connect_sessions() {
        for (connect_id, server_url, _expires_at, status) in persisted {
            // We need expires_at as DateTime<Utc> for the in-memory shape.
            // list_connect_sessions already filters expired.
            // Pull the full row to get expires_at + device_id + mac_name etc.
            let detail = persistence
                .get_connect_session_full(&connect_id)
                .ok()
                .flatten();
            let (
                device_id,
                mac_name,
                mac_nonce,
                mac_pubkey,
                mac_callback,
                expires_at,
                scopes,
                room_claim_secret,
            ) = if let Some(d) = detail {
                (
                    d.device_id,
                    d.mac_name,
                    d.mac_nonce,
                    d.mac_pubkey,
                    d.mac_callback,
                    d.expires_at,
                    d.scopes,
                    d.room_claim_secret,
                )
            } else {
                (
                    String::new(),
                    String::new(),
                    String::new(),
                    None,
                    None,
                    now + chrono::Duration::seconds(300),
                    vec!["read".to_string(), "write".to_string()],
                    String::new(),
                )
            };
            if status == "completed" {
                // completed rooms: don't rehydrate in-memory (they're done).
                continue;
            }
            s.connect_sessions.insert(
                connect_id.clone(),
                crate::routes::device_pairing::ConnectSession {
                    connect_id: connect_id.clone(),
                    device_id: device_id.clone(),
                    mac_name,
                    mac_nonce,
                    mac_pubkey,
                    mac_callback,
                    server_url,
                    scopes,
                    created_at: now,
                    expires_at,
                    status,
                    token: None,
                    token_delivered: false,
                    delivered_at: None,
                    // V2 P0 round 2: rehydrate the room_claim_secret
                    // from the persisted full row so the /join auth
                    // check survives daemon restart.
                    room_claim_secret: room_claim_secret.clone(),
                },
            );
        }
    }
    // Load all non-expired device_tokens.
    let rehydrated_tokens = if let Ok(rows) = persistence.list_device_tokens() {
        let mut count = 0;
        for (token, device_id, scopes_json, issued_at, expires_at, issued_to) in rows {
            if s.tokens.contains_key(&token) {
                continue;
            }
            let scopes: Vec<String> = scopes_json
                .as_deref()
                .and_then(|j| serde_json::from_str(j).ok())
                .unwrap_or_else(|| vec!["read".to_string(), "write".to_string()]);
            let issued_at_dt = chrono::DateTime::parse_from_rfc3339(&issued_at)
                .map(|t| t.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| now);
            let expires_at_dt = chrono::DateTime::parse_from_rfc3339(&expires_at)
                .map(|t| t.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| now + chrono::Duration::seconds(86400 * 30));
            s.tokens.insert(
                token.clone(),
                focusa_core::types::DeviceToken {
                    token,
                    device_id,
                    scopes,
                    issued_at: issued_at_dt,
                    expires_at: expires_at_dt,
                    last_used_at: None,
                    issued_to: issued_to.unwrap_or_else(|| "ledger".to_string()),
                },
            );
            count += 1;
        }
        count
    } else {
        0
    };
    let rehydrated_rooms = s.connect_sessions.len();
    drop(s);
    Ok((rehydrated_rooms, rehydrated_tokens))
}

pub fn set_app_state(state: Arc<AppState>) {
    let _ = APP_STATE.set(state);
}

pub fn app_state_for_token_lookup() -> Option<Arc<AppState>> {
    APP_STATE.get().cloned()
}

pub fn build_router(state: Arc<AppState>) -> Router {
    set_app_state(state.clone());

    Router::new()
        .merge(routes::agent_capabilities::routes())
        .merge(routes::health::router())
        .merge(routes::info::router())
        .merge(routes::llms_txt::router())
        .merge(routes::env::router())
        .merge(routes::commands::router())
        .merge(routes::compaction::router())
        .merge(routes::capabilities::router())
        .merge(routes::capabilities_extra::router())
        .merge(routes::instances::router())
        .merge(routes::attachments::router())
        .merge(routes::sync::router())
        .merge(routes::bloatgaurd::router())
        .merge(routes::focus::router())
        .merge(routes::work_items::router())
        .merge(routes::gate::router())
        .merge(routes::ecs::router())
        .merge(routes::memory::router())
        .merge(routes::mcp::router())
        .merge(routes::browser_interop::router())
        .merge(routes::metacognition::router())
        .merge(routes::ontology::router())
        .merge(routes::events_sqlite::router())
        .merge(routes::events_retention::router())
        .merge(routes::silent_sessions_wait::router())
        .merge(routes::remote_workspaces::router())
        .merge(routes::compaction_controller::router())
        .merge(routes::callgraph::router())
        .merge(routes::background_jobs::router())
        .merge(routes::runtime_constitution::router())
        .merge(routes::adapters::router())
        .merge(routes::direction::router())
        .merge(routes::session_fanout::router())
        .merge(routes::worksets::router())
        .merge(routes::cockpit::router())
        .merge(routes::research_packet::router())
        .merge(routes::completion_claims::router())
        .merge(routes::session::router())
        .merge(routes::silent_sessions::router())
        .merge(routes::proxy::router())
        .merge(routes::license::router())
        .merge(routes::clt::router())
        .merge(routes::uxp::router())
        .merge(routes::autonomy::router())
        .merge(routes::constitution::router())
        .merge(routes::telemetry::router())
        .merge(routes::trust::router())
        .merge(routes::threads::router())
        .merge(routes::proposals::router())
        .merge(routes::project::router())
        .merge(routes::predictions::router())
        .merge(routes::rfm::router())
        .merge(routes::resource::router())
        .merge(routes::reflection::router())
        .merge(routes::reflex::router())
        .merge(routes::release::router())
        .merge(routes::update::router())
        .merge(routes::skills::router())
        .merge(routes::snapshots::router())
        .merge(routes::subagent::router())
        .merge(routes::training::router())
        .merge(routes::trajectory::router())
        .merge(routes::call_stack::router())
        .merge(routes::context_cognition::router())
        .merge(routes::context_sources::router())
        .merge(routes::context_claims::router())
        .merge(routes::role_profiles::router())
        .merge(routes::interview_sessions::router())
        .merge(routes::interview_strategy::router())
        .merge(routes::spec_workbench::router())
        .merge(routes::provider_execution::router())
        .merge(routes::task_plans::router())
        .merge(routes::work_rail::router())
        .merge(routes::mission_canvas_surfaces::router())
        .merge(routes::workspace_artifacts::router())
        .merge(routes::device_pairing::router())
        .merge(routes::deck::router())
        .merge(routes::preload::router())
        .merge(routes::bloatgaurd_optical::router())
        .merge(routes::turn_recent::router())
        .nest_service("/static", ServeDir::new(STATIC_DIR))
        .merge(routes::dxux::router())
        .merge(routes::utility::router())
        .merge(routes::traverse::router())
        .merge(routes::visual_workflow::router())
        .merge(routes::work_loop::router())
        .merge(routes::workpoint::router())
        .merge(routes::turn::router())
        .merge(routes::ascc::router())
        .merge(routes::awareness::router())
        .merge(routes::tokens::router())
        .merge(routes::sse::router())
        .merge(routes::agent_reminder::router())
        .layer(axum_mw::from_fn(
            routes::agent_reminder::agent_prompt_response_header_mw,
        ))
        .layer(menubar_cors_layer())
        .layer(DefaultBodyLimit::max(routes::bounded::env_limit(
            "FOCUSA_API_MAX_BODY_BYTES",
            1_048_576,
        )))
        .layer(axum_mw::from_fn(
            middleware::json_guard::mutation_json_guard_layer,
        ))
        .layer(axum_mw::from_fn(
            middleware::rate_limit::mutation_rate_limit_layer,
        ))
        .layer(axum_mw::from_fn(middleware::route_scope::route_scope_layer))
        .layer(axum_mw::from_fn(middleware::auth::auth_layer))
        .layer(axum_mw::from_fn(
            middleware::error_envelope::error_envelope_layer,
        ))
        .with_state(state)
}

fn scheduler_base_url(bind_addr: &str) -> String {
    let port = bind_addr.rsplit(':').next().unwrap_or("8787");
    format!("http://127.0.0.1:{}", port)
}

fn should_auto_reenable_continuous(
    enabled: bool,
    status: WorkLoopStatus,
    last_continue_reason: Option<&str>,
) -> bool {
    if enabled || status != WorkLoopStatus::Idle {
        return false;
    }
    !was_explicit_operator_stop(last_continue_reason)
}

fn was_explicit_operator_stop(last_continue_reason: Option<&str>) -> bool {
    let Some(reason) = last_continue_reason else {
        return false;
    };
    let normalized = reason.to_ascii_lowercase();
    normalized.contains("operator stop")
        || normalized.contains("stop working")
        || normalized.trim() == "stop"
}

fn dispatch_error_suggests_transport_recovery(error: &str) -> bool {
    let normalized = error.to_ascii_lowercase();
    normalized.contains("pi rpc driver not active")
        || normalized.contains("failed writing prompt")
        || normalized.contains("broken pipe")
        || normalized.contains("stream closed")
}

fn supervisor_allows_pi_driver(enabled: bool, status: WorkLoopStatus) -> bool {
    enabled
        && matches!(
            status,
            WorkLoopStatus::SelectingReadyWork
                | WorkLoopStatus::PreparingTurn
                | WorkLoopStatus::AwaitingHarnessTurn
                | WorkLoopStatus::EvaluatingOutcome
                | WorkLoopStatus::AdvancingTask
                | WorkLoopStatus::Idle
        )
}

fn supervisor_should_start_pi_driver(
    enabled: bool,
    status: WorkLoopStatus,
    has_current_task: bool,
) -> bool {
    supervisor_allows_pi_driver(enabled, status)
        && (has_current_task || status != WorkLoopStatus::Idle)
}

async fn reflection_scheduler_loop(base_url: String) {
    let client = reqwest::Client::new();

    loop {
        let delay_secs = {
            let scheduler_url = format!("{}/v1/reflect/scheduler", base_url);
            match client.get(&scheduler_url).send().await {
                Ok(resp) => match resp.json::<serde_json::Value>().await {
                    Ok(body) => {
                        let enabled = body
                            .get("enabled")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);
                        let interval = body
                            .get("interval_seconds")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(3600)
                            .max(1);

                        if enabled {
                            if let Some((mode, reason)) = lowmem_background_throttle() {
                                tracing::debug!(
                                    mode,
                                    reason,
                                    "reflection scheduler tick throttled by LowMem background policy"
                                );
                                interval.max(60)
                            } else {
                                let tick_url = format!("{}/v1/reflect/scheduler/tick", base_url);
                                let _ = client
                                    .post(&tick_url)
                                    .json(&serde_json::json!({}))
                                    .send()
                                    .await
                                    .map(|r| {
                                        tracing::debug!(status = %r.status(), "reflection scheduler tick executed");
                                    });
                                interval
                            }
                        } else {
                            30
                        }
                    }
                    Err(_) => 30,
                },
                Err(_) => 30,
            }
        };

        tokio::time::sleep(Duration::from_secs(delay_secs)).await;
    }
}

async fn continuous_work_supervisor_loop(state: Arc<AppState>, base_url: String) {
    let client = reqwest::Client::new();
    let mut attached_stuck_ticks: u32 = 0;
    let mut last_transport_event_seq: Option<u64> = None;

    loop {
        state
            .supervisor_perf
            .ticks_total
            .fetch_add(1, Ordering::Relaxed);

        let (
            enabled,
            status,
            session_state,
            last_event_kind,
            last_event_seq,
            status_heartbeat_ms,
            last_continue_reason,
            current_task_id,
            execution_scope,
            execution_work_item_id,
            execution_workpoint_id,
        ) = {
            let s = state.focusa.read().await;
            (
                s.work_loop.enabled,
                s.work_loop.status,
                s.work_loop.transport_session_state.clone(),
                s.work_loop.last_transport_event_kind.clone(),
                Some(s.work_loop.last_transport_event_sequence),
                s.work_loop.policy.status_heartbeat_ms,
                s.work_loop.last_continue_reason.clone(),
                s.work_loop
                    .current_task
                    .as_ref()
                    .map(|task| task.work_item_id.clone()),
                s.work_loop.execution_scope.clone(),
                s.work_loop.execution_work_item_id.clone(),
                s.work_loop.execution_workpoint_id,
            )
        };

        let mut delay_ms = status_heartbeat_ms.clamp(500, 5_000);

        if !enabled {
            delay_ms = delay_ms.min(2_000);
        }

        if let Some((mode, reason)) = lowmem_background_throttle() {
            state
                .supervisor_perf
                .background_throttled_ticks
                .fetch_add(1, Ordering::Relaxed);
            tracing::debug!(
                mode,
                reason,
                "continuous work supervisor tick throttled by LowMem background policy"
            );
            tokio::time::sleep(Duration::from_millis(delay_ms.max(5_000))).await;
            continue;
        }

        if should_auto_reenable_continuous(enabled, status, last_continue_reason.as_deref())
            && let (Some(scope), Some(work_item_id), Some(workpoint_id)) = (
                execution_scope.clone(),
                execution_work_item_id.clone(),
                execution_workpoint_id,
            )
        {
            let policy = WorkLoopPolicy::with_overrides(
                WorkLoopPreset::Push,
                WorkLoopPolicyOverrides {
                    max_turns: Some(100_000),
                    max_wall_clock_ms: Some(2_592_000_000),
                    max_retries: Some(1_000),
                    max_consecutive_low_productivity_turns: Some(1_000),
                    max_consecutive_failures: Some(1_000),
                    max_same_subproblem_retries: Some(1_000),
                    ..WorkLoopPolicyOverrides::default()
                },
            );
            let _ = state
                .command_tx
                .send(Action::EnableContinuousWork {
                    project_run_id: Uuid::now_v7(),
                    policy,
                    scope,
                    work_item_id,
                    workpoint_id,
                })
                .await;
        }

        if enabled {
            // Exhausted budgets remain paused until an explicitly approved
            // resume request renews the epoch; the supervisor never silently
            // inflates policy limits or resets counters.
            let (Some(scope), Some(work_item_id)) =
                (execution_scope.as_ref(), execution_work_item_id.as_deref())
            else {
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                continue;
            };
            let project_root = scope.root_scope.root_path.to_string_lossy().to_string();
            let continuity_id = scope.continuity_id.clone();
            let claim_key = format!(
                "project:{}|workstream:{}|work_item:{}",
                project_root.replace('|', "_"),
                continuity_id.replace('|', "_"),
                work_item_id.replace('|', "_")
            );
            let lease = {
                let claims = state.writer_claims.read().await;
                claims
                    .get(&claim_key)
                    .filter(|lease| lease.expires_at > chrono::Utc::now())
                    .cloned()
            };
            let Some(lease) = lease else {
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                continue;
            };

            let allows_driver = supervisor_allows_pi_driver(enabled, status);
            let should_start_driver =
                supervisor_should_start_pi_driver(enabled, status, current_task_id.is_some());

            let mut has_session = {
                let mut guard = state.pi_rpc_session.lock().await;
                if let Some(session) = guard.as_mut() {
                    match session.child.try_wait() {
                        Ok(Some(_)) => {
                            *guard = None;
                            false
                        }
                        Ok(None) => true,
                        Err(_) => {
                            *guard = None;
                            false
                        }
                    }
                } else {
                    false
                }
            };

            let attached_waiting = has_session
                && status == WorkLoopStatus::AwaitingHarnessTurn
                && session_state.as_deref() == Some("attached")
                && last_event_kind.as_deref() == Some("session_attached");

            if attached_waiting {
                if last_event_seq == last_transport_event_seq {
                    attached_stuck_ticks = attached_stuck_ticks.saturating_add(1);
                } else {
                    attached_stuck_ticks = 1;
                }
            } else {
                attached_stuck_ticks = 0;
            }
            last_transport_event_seq = last_event_seq;

            if attached_stuck_ticks >= 3 {
                state
                    .supervisor_perf
                    .driver_stop_attempts
                    .fetch_add(1, Ordering::Relaxed);
                let stop_url = format!("{}/v1/work-loop/driver/stop", base_url);
                let _ = client
                    .post(&stop_url)
                    .header("x-focusa-writer-id", &lease.writer_id)
                    .header("x-focusa-fencing-token", lease.fencing_token)
                    .header("x-focusa-project-root", &project_root)
                    .header("x-focusa-continuity-id", &continuity_id)
                    .json(&serde_json::json!({}))
                    .send()
                    .await;
                has_session = false;
                attached_stuck_ticks = 0;
            }

            if !allows_driver && has_session {
                state
                    .supervisor_perf
                    .driver_stop_attempts
                    .fetch_add(1, Ordering::Relaxed);
                let stop_url = format!("{}/v1/work-loop/driver/stop", base_url);
                let _ = client
                    .post(&stop_url)
                    .header("x-focusa-writer-id", &lease.writer_id)
                    .header("x-focusa-fencing-token", lease.fencing_token)
                    .header("x-focusa-project-root", &project_root)
                    .header("x-focusa-continuity-id", &continuity_id)
                    .json(&serde_json::json!({}))
                    .send()
                    .await;
                has_session = false;
            }

            if should_start_driver && !has_session {
                state
                    .supervisor_perf
                    .driver_start_attempts
                    .fetch_add(1, Ordering::Relaxed);
                let driver_url = format!("{}/v1/work-loop/driver/start", base_url);
                let _ = client
                    .post(&driver_url)
                    .header("x-focusa-writer-id", &lease.writer_id)
                    .header("x-focusa-fencing-token", lease.fencing_token)
                    .header("x-focusa-project-root", &project_root)
                    .header("x-focusa-continuity-id", &continuity_id)
                    .json(&serde_json::json!({"cwd": project_root}))
                    .send()
                    .await;
            }

            if !allows_driver {
                state
                    .supervisor_perf
                    .dispatch_skipped_disallowed
                    .fetch_add(1, Ordering::Relaxed);
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                continue;
            }

            state
                .supervisor_perf
                .dispatch_attempts
                .fetch_add(1, Ordering::Relaxed);
            let dispatch_result = crate::routes::work_loop::maybe_dispatch_continuous_turn_prompt(
                &state,
                "daemon heartbeat supervisor tick",
            )
            .await;

            if let Err((_status_code, body)) = dispatch_result {
                let error_message = body
                    .0
                    .get("error")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                if dispatch_error_suggests_transport_recovery(&error_message) && allows_driver {
                    state
                        .supervisor_perf
                        .dispatch_recovery_restarts
                        .fetch_add(1, Ordering::Relaxed);

                    state
                        .supervisor_perf
                        .driver_stop_attempts
                        .fetch_add(1, Ordering::Relaxed);
                    let stop_url = format!("{}/v1/work-loop/driver/stop", base_url);
                    let _ = client
                        .post(&stop_url)
                        .header("x-focusa-writer-id", &lease.writer_id)
                        .header("x-focusa-fencing-token", lease.fencing_token)
                        .header("x-focusa-project-root", &project_root)
                        .header("x-focusa-continuity-id", &continuity_id)
                        .json(&serde_json::json!({}))
                        .send()
                        .await;

                    state
                        .supervisor_perf
                        .driver_start_attempts
                        .fetch_add(1, Ordering::Relaxed);
                    let driver_url = format!("{}/v1/work-loop/driver/start", base_url);
                    let _ = client
                        .post(&driver_url)
                        .header("x-focusa-writer-id", &lease.writer_id)
                        .header("x-focusa-fencing-token", lease.fencing_token)
                        .header("x-focusa-project-root", &project_root)
                        .header("x-focusa-continuity-id", &continuity_id)
                        .json(&serde_json::json!({"cwd": project_root}))
                        .send()
                        .await;

                    state
                        .supervisor_perf
                        .dispatch_attempts
                        .fetch_add(1, Ordering::Relaxed);
                    let _ = crate::routes::work_loop::maybe_dispatch_continuous_turn_prompt(
                        &state,
                        "daemon heartbeat supervisor tick (transport recovery retry)",
                    )
                    .await;
                }
            }

            delay_ms = status_heartbeat_ms.clamp(500, 5_000);
        }

        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
    }
}

/// Start the API server on the configured bind address.
pub async fn run(
    focusa: Arc<RwLock<FocusaState>>,
    command_tx: mpsc::Sender<Action>,
    events_tx: broadcast::Sender<String>,
    config: FocusaConfig,
    persistence_runtime: (SqlitePersistence, PersistenceActor),
    write_serial_lock: Arc<Mutex<()>>,
    external_mutation_epoch: Arc<AtomicU64>,
) -> anyhow::Result<()> {
    let bind_addr = config.api_bind.clone();
    let (persistence, persistence_actor) = persistence_runtime;

    let broadcaster = EventBroadcaster::new();
    let prediction_store = Arc::new(ScopedCrdtLedger::new(
        &config.data_dir,
        "predictions",
        format!("daemon:{}", std::process::id()),
    ));

    let state = Arc::new(AppState {
        focusa,
        workstream_states: crate::workstream_store::WorkstreamStateStore::default(),
        command_tx,
        events_tx,
        event_broadcaster: broadcaster,
        config,
        persistence,
        persistence_actor: Some(persistence_actor),
        write_serial_lock,
        command_store: Arc::new(RwLock::new(HashMap::new())),
        token_store: Arc::new(RwLock::new(focusa_core::permissions::TokenStore::new())),
        writer_claims: Arc::new(TokioRwLock::new(HashMap::new())),
        next_writer_fencing_token: Arc::new(AtomicU64::new(
            (chrono::Utc::now().timestamp_millis().max(1) as u64).saturating_mul(1_000),
        )),
        focus_stack_by_scope: Arc::new(TokioRwLock::new(HashMap::new())),
        prediction_store,
        recent_completed_turns_by_scope: Arc::new(TokioRwLock::new(HashMap::new())),
        snapshots_by_scope: Arc::new(TokioRwLock::new(HashMap::new())),
        metacog_by_scope: Arc::new(std::sync::Mutex::new(HashMap::new())),
        started_at: Instant::now(),
        pi_rpc_session: Arc::new(Mutex::new(None)),
        supervisor_perf: Arc::new(SupervisorPerfCounters::default()),
        external_mutation_epoch,
    });

    let app = build_router(state.clone());

    // Bind readiness before eager pairing-ledger rehydration. The ledger shares
    // SQLite with the persistence actor and may wait behind recovery backlog;
    // health/readiness must never wait on that optional cache warm-up. Pairing
    // token auth retains its on-demand SQLite recovery path during this window.
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    let pairing_rehydrate_state = state.clone();
    tokio::spawn(async move {
        match rehydrate_pairing_state_from_ledger(&pairing_rehydrate_state).await {
            Ok((rooms, tokens)) => {
                tracing::info!(
                    rooms = rooms,
                    tokens = tokens,
                    "V2: PairingStore rehydrated from ledger after API readiness"
                );
            }
            Err(e) => {
                tracing::warn!(error = %e, "V2: PairingStore rehydrate after readiness failed");
            }
        }
    });

    let scheduler_url = scheduler_base_url(&bind_addr);
    tokio::spawn(async move {
        // Delay one cycle to allow server readiness, then run continuously.
        reflection_scheduler_loop(scheduler_url).await;
    });

    let supervisor_url = scheduler_base_url(&bind_addr);
    let supervisor_state = state.clone();
    tokio::spawn(async move {
        continuous_work_supervisor_loop(supervisor_state, supervisor_url).await;
    });

    tokio::spawn(async move {
        allocator_trim_loop().await;
    });

    let resource_monitor_state = state.clone();
    tokio::spawn(async move {
        resource_mode_monitor_loop(resource_monitor_state).await;
    });

    tracing::info!("Listening on {}", bind_addr);
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        dispatch_error_suggests_transport_recovery, scheduler_base_url,
        should_auto_reenable_continuous, supervisor_allows_pi_driver,
        supervisor_should_start_pi_driver, was_explicit_operator_stop,
    };
    use focusa_core::types::WorkLoopStatus;

    #[test]
    fn scheduler_base_url_uses_localhost_port() {
        assert_eq!(
            scheduler_base_url("127.0.0.1:8787"),
            "http://127.0.0.1:8787"
        );
        assert_eq!(scheduler_base_url("0.0.0.0:9999"), "http://127.0.0.1:9999");
    }

    #[test]
    fn explicit_stop_detection_is_conservative() {
        assert!(was_explicit_operator_stop(Some(
            "Operator requested: stop working"
        )));
        assert!(was_explicit_operator_stop(Some("stop")));
        assert!(!was_explicit_operator_stop(Some(
            "operator steering detected"
        )));
        assert!(!was_explicit_operator_stop(None));
    }

    #[test]
    fn supervisor_reenables_idle_loop_unless_explicitly_stopped() {
        assert!(should_auto_reenable_continuous(
            false,
            WorkLoopStatus::Idle,
            Some("operator steering detected"),
        ));
        assert!(!should_auto_reenable_continuous(
            false,
            WorkLoopStatus::Idle,
            Some("Operator requested: stop working"),
        ));
        assert!(!should_auto_reenable_continuous(
            false,
            WorkLoopStatus::Paused,
            Some("operator steering detected"),
        ));
        assert!(!should_auto_reenable_continuous(
            true,
            WorkLoopStatus::Idle,
            Some("operator steering detected"),
        ));
    }

    #[test]
    fn supervisor_driver_gate_respects_loop_status() {
        assert!(supervisor_allows_pi_driver(true, WorkLoopStatus::Idle));
        assert!(supervisor_allows_pi_driver(
            true,
            WorkLoopStatus::AwaitingHarnessTurn
        ));
        assert!(!supervisor_allows_pi_driver(
            false,
            WorkLoopStatus::AwaitingHarnessTurn
        ));
        assert!(!supervisor_allows_pi_driver(true, WorkLoopStatus::Paused));
        assert!(!supervisor_allows_pi_driver(true, WorkLoopStatus::Blocked));
        assert!(!supervisor_allows_pi_driver(
            true,
            WorkLoopStatus::TransportDegraded
        ));
    }

    #[test]
    fn supervisor_does_not_start_pi_driver_for_empty_idle_loop() {
        assert!(!supervisor_should_start_pi_driver(
            true,
            WorkLoopStatus::Idle,
            false
        ));
        assert!(supervisor_should_start_pi_driver(
            true,
            WorkLoopStatus::Idle,
            true
        ));
        assert!(supervisor_should_start_pi_driver(
            true,
            WorkLoopStatus::AwaitingHarnessTurn,
            false
        ));
    }

    #[test]
    fn dispatch_error_transport_recovery_detection_is_specific() {
        assert!(dispatch_error_suggests_transport_recovery(
            "pi rpc driver not active"
        ));
        assert!(dispatch_error_suggests_transport_recovery(
            "failed writing prompt: Broken pipe"
        ));
        assert!(dispatch_error_suggests_transport_recovery(
            "pi rpc stdout stream closed; restart required"
        ));
        assert!(!dispatch_error_suggests_transport_recovery(
            "required verification not yet satisfied"
        ));
    }
}
