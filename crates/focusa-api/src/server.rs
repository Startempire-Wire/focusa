//! HTTP server setup.
//!
//! The API server is a thin read/write facade:
//!   - Reads: snapshot current state via Arc<RwLock<FocusaState>>
//!   - Writes: dispatch Actions via mpsc::Sender<Action> to the daemon event loop
//!
//! The daemon owns the state; the API borrows a read handle and a command channel.

use crate::middleware;
use crate::routes;
use crate::routes::sse::EventBroadcaster;
use axum::Router;
use axum::middleware as axum_mw;
use focusa_core::runtime::persistence_sqlite::SqlitePersistence;
use focusa_core::types::{
    Action, FocusaConfig, FocusaState, WorkLoopPolicy, WorkLoopPolicyOverrides, WorkLoopPreset,
    WorkLoopStatus,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::process::{Child, ChildStdin};
use tokio::sync::RwLock as TokioRwLock;
use tokio::sync::{Mutex, RwLock, broadcast, mpsc};
use uuid::Uuid;

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
    pub stdin: ChildStdin,
    pub session_id: String,
    pub cwd: Option<String>,
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
}

/// Shared state between API server and daemon.
pub struct AppState {
    /// Read-only snapshot of cognitive state (daemon writes, API reads).
    pub focusa: Arc<RwLock<FocusaState>>,
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
    /// Serializes canonical state writers across daemon actions and sync API routes.
    pub write_serial_lock: Arc<Mutex<()>>,
    /// In-memory command write-model state for /v1/commands/* endpoints.
    pub command_store: CommandStore,
    /// Token store for capability permissions (docs/25-26).
    pub token_store: Arc<RwLock<focusa_core::permissions::TokenStore>>,
    /// Claimed writer authority for continuous work-loop mutations.
    pub active_writer: Arc<TokioRwLock<Option<String>>>,
    /// Process start time for uptime reporting.
    pub started_at: Instant,
    /// Optional daemon-owned Pi RPC transport session for continuous work.
    pub pi_rpc_session: Arc<Mutex<Option<PiRpcSession>>>,
    /// Lightweight performance/backpressure counters for supervisor loop.
    pub supervisor_perf: Arc<SupervisorPerfCounters>,
}

/// Build the axum Router with all routes.
pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .merge(routes::health::router())
        .merge(routes::info::router())
        .merge(routes::env::router())
        .merge(routes::commands::router())
        .merge(routes::capabilities::router())
        .merge(routes::capabilities_extra::router())
        .merge(routes::instances::router())
        .merge(routes::attachments::router())
        .merge(routes::sync::router())
        .merge(routes::focus::router())
        .merge(routes::gate::router())
        .merge(routes::ecs::router())
        .merge(routes::memory::router())
        .merge(routes::metacognition::router())
        .merge(routes::ontology::router())
        .merge(routes::events_sqlite::router())
        .merge(routes::session::router())
        .merge(routes::proxy::router())
        .merge(routes::clt::router())
        .merge(routes::uxp::router())
        .merge(routes::autonomy::router())
        .merge(routes::constitution::router())
        .merge(routes::telemetry::router())
        .merge(routes::trust::router())
        .merge(routes::threads::router())
        .merge(routes::proposals::router())
        .merge(routes::predictions::router())
        .merge(routes::rfm::router())
        .merge(routes::reflection::router())
        .merge(routes::skills::router())
        .merge(routes::snapshots::router())
        .merge(routes::training::router())
        .merge(routes::visual_workflow::router())
        .merge(routes::work_loop::router())
        .merge(routes::workpoint::router())
        .merge(routes::turn::router())
        .merge(routes::ascc::router())
        .merge(routes::awareness::router())
        .merge(routes::tokens::router())
        .merge(routes::sse::router())
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
            last_blocker_reason,
            last_continue_reason,
        ) = {
            let s = state.focusa.read().await;
            (
                s.work_loop.enabled,
                s.work_loop.status,
                s.work_loop.transport_session_state.clone(),
                s.work_loop.last_transport_event_kind.clone(),
                Some(s.work_loop.last_transport_event_sequence),
                s.work_loop.policy.status_heartbeat_ms,
                s.work_loop.last_blocker_reason.clone(),
                s.work_loop.last_continue_reason.clone(),
            )
        };

        let mut delay_ms = status_heartbeat_ms.clamp(500, 5_000);

        if !enabled {
            delay_ms = delay_ms.min(2_000);
        }

        if should_auto_reenable_continuous(enabled, status, last_continue_reason.as_deref()) {
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
                })
                .await;
        }

        if enabled {
            let budget_exhausted =
                matches!(status, WorkLoopStatus::Paused | WorkLoopStatus::Blocked)
                    && last_blocker_reason
                        .as_deref()
                        .map(|reason| {
                            reason.contains("max_turns budget exhausted")
                                || reason.contains("max_wall_clock_ms budget exhausted")
                        })
                        .unwrap_or(false);

            if budget_exhausted {
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
                    })
                    .await;
            }

            let writer = {
                let mut claim = state.active_writer.write().await;
                if claim.is_none() {
                    *claim = Some("daemon-supervisor".to_string());
                }
                claim
                    .clone()
                    .unwrap_or_else(|| "daemon-supervisor".to_string())
            };

            let allows_driver = supervisor_allows_pi_driver(enabled, status);

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
                    .header("x-focusa-writer-id", &writer)
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
                    .header("x-focusa-writer-id", &writer)
                    .json(&serde_json::json!({}))
                    .send()
                    .await;
                has_session = false;
            }

            if allows_driver && !has_session {
                state
                    .supervisor_perf
                    .driver_start_attempts
                    .fetch_add(1, Ordering::Relaxed);
                let driver_url = format!("{}/v1/work-loop/driver/start", base_url);
                let _ = client
                    .post(&driver_url)
                    .header("x-focusa-writer-id", &writer)
                    .json(&serde_json::json!({"cwd":"/home/wirebot/focusa"}))
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
                        .header("x-focusa-writer-id", &writer)
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
                        .header("x-focusa-writer-id", &writer)
                        .json(&serde_json::json!({"cwd":"/home/wirebot/focusa"}))
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
    persistence: SqlitePersistence,
    write_serial_lock: Arc<Mutex<()>>,
) -> anyhow::Result<()> {
    let bind_addr = config.api_bind.clone();

    let broadcaster = EventBroadcaster::new();

    let state = Arc::new(AppState {
        focusa,
        command_tx,
        events_tx,
        event_broadcaster: broadcaster,
        config,
        persistence,
        write_serial_lock,
        command_store: Arc::new(RwLock::new(HashMap::new())),
        token_store: Arc::new(RwLock::new(focusa_core::permissions::TokenStore::new())),
        active_writer: Arc::new(TokioRwLock::new(None)),
        started_at: Instant::now(),
        pi_rpc_session: Arc::new(Mutex::new(None)),
        supervisor_perf: Arc::new(SupervisorPerfCounters::default()),
    });

    let app = build_router(state.clone());
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;

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

    tracing::info!("Listening on {}", bind_addr);
    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        dispatch_error_suggests_transport_recovery, scheduler_base_url,
        should_auto_reenable_continuous, supervisor_allows_pi_driver, was_explicit_operator_stop,
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
