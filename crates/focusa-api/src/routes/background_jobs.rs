//! Background job HTTP surface — create, lifecycle, complete (with SSE
//! broadcast), status, list, and long-poll wait. Mirrors the silent-session
//! completion pattern (#311): durable first, broadcast second.

use axum::Json;
use axum::Router;
use axum::extract::{Query, State};
use axum::routing::{get, post};
use focusa_core::background_jobs::{
    BACKGROUND_JOB_SCHEMA, BackgroundJobCompletionEvent, BackgroundJobFailureClass,
    BackgroundJobRecord, BackgroundJobStatus,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;
use std::time::Duration;

use crate::server::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/background-jobs", post(create_job).get(list_jobs))
        .route(
            "/v1/background-jobs/{job_id}",
            get(get_job).post(update_job),
        )
        .route("/v1/background-jobs/{job_id}/complete", post(complete_job))
        .route("/v1/background-jobs/wait", get(wait_job))
}

fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339()
}

#[derive(Deserialize)]
pub struct CreateJobBody {
    #[serde(default)]
    pub job_id: Option<String>,
    pub name: String,
    pub command: String,
    #[serde(default = "default_cwd")]
    pub cwd: String,
    #[serde(default)]
    pub log_path: Option<String>,
    #[serde(default)]
    pub attachment: Option<focusa_core::scoped_state::AttachmentKey>,
    /// Lifecycle-owning creator/monitor process. A queued row with a dead
    /// creator can be reconciled instead of remaining queued forever.
    #[serde(default)]
    pub pid: Option<u32>,
    /// Optional OS process-start identity. Older clients may omit it.
    #[serde(default)]
    pub process_start_token: Option<String>,
}

fn default_cwd() -> String {
    ".".to_string()
}

#[derive(Deserialize)]
pub struct UpdateJobBody {
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub pid: Option<u32>,
    /// Optional OS process-start identity. Older clients may omit it.
    #[serde(default)]
    pub process_start_token: Option<String>,
}

#[derive(Deserialize)]
pub struct CompleteJobBody {
    pub exit_code: i32,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub failure_class: Option<BackgroundJobFailureClass>,
    /// Monitor-captured tail. Required for cross-namespace delivery when the
    /// daemon cannot see the CLI monitor's host `/tmp` log.
    #[serde(default)]
    pub output_tail: String,
}

#[derive(Deserialize)]
pub struct WaitQuery {
    pub job_id: String,
    #[serde(default = "default_wait_timeout_ms")]
    pub timeout_ms: u64,
}

fn default_wait_timeout_ms() -> u64 {
    30_000
}

async fn create_job(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateJobBody>,
) -> Json<Value> {
    let path = crate::routes::events_sqlite::focusa_db_path(&state.config.data_dir);
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<Value> {
        let conn = rusqlite::Connection::open(path)?;
        focusa_core::background_job_store::ensure_schema(&conn)?;
        if let Some(attachment) = body.attachment.as_ref() {
            attachment.validate()?;
            anyhow::ensure!(
                attachment.workstream.root_scope.scope_kind
                    == focusa_core::scoped_state::ScopeKind::Project,
                "background job attachment must use a verified project scope"
            );
        }
        let job_id = body
            .job_id
            .unwrap_or_else(|| uuid::Uuid::now_v7().to_string());
        let record = BackgroundJobRecord {
            schema: BACKGROUND_JOB_SCHEMA.to_string(),
            job_id: job_id.clone(),
            name: body.name,
            command: body.command,
            cwd: body.cwd,
            attachment: body.attachment,
            status: BackgroundJobStatus::Queued,
            failure_class: None,
            exit_code: None,
            pid: body.pid,
            process_start_token: body.process_start_token,
            log_path: body
                .log_path
                .unwrap_or_else(|| format!("/tmp/focusa-bg-{job_id}.log")),
            started_at: now_iso(),
            completed_at: None,
            output_tail: String::new(),
        };
        focusa_core::background_job_store::upsert_job(&conn, &record)?;
        Ok(json!({ "status": "queued", "job": record }))
    })
    .await;
    match result {
        Ok(Ok(payload)) => Json(payload),
        Ok(Err(error)) => Json(focusa_core::error_envelope::internal_error(
            "route",
            &error.to_string(),
        )),
        Err(error) => Json(focusa_core::error_envelope::internal_error(
            "join",
            &format!("join error: {error}"),
        )),
    }
}

async fn update_job(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(job_id): axum::extract::Path<String>,
    Json(body): Json<UpdateJobBody>,
) -> Json<Value> {
    let path = crate::routes::events_sqlite::focusa_db_path(&state.config.data_dir);
    let events_tx = state.events_tx.clone();
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<Value> {
        let conn = rusqlite::Connection::open(path)?;
        focusa_core::background_job_store::ensure_schema(&conn)?;
        let Some(mut record) = focusa_core::background_job_store::load_job(&conn, &job_id)? else {
            return Ok(json!({"status": "missing", "job_id": job_id}));
        };
        if let Some(status) = body.status {
            record.status = BackgroundJobStatus::parse(&status);
        }
        if let Some(pid) = body.pid {
            record.pid = Some(pid);
            // A PID change without a matching start token must not retain the
            // previous process identity. Legacy clients remain supported.
            record.process_start_token = body.process_start_token;
        } else if body.process_start_token.is_some() {
            record.process_start_token = body.process_start_token;
        }
        let became_running = record.status == BackgroundJobStatus::Running;
        let became_monitor_lost = record.status == BackgroundJobStatus::MonitorLost;
        if became_monitor_lost && record.completed_at.is_none() {
            record.completed_at = Some(now_iso());
        }
        focusa_core::background_job_store::upsert_job(&conn, &record)?;
        // docs/165 v2 §2 — durable first, then broadcast lifecycle
        // envelopes so surfaces see dispatch latency and monitor loss.
        let started_event = if became_running {
            serde_json::to_string(
                &focusa_core::background_jobs::BackgroundJobStartedEvent::from_record(&record),
            )
            .ok()
        } else {
            None
        };
        let completion_event = if became_monitor_lost {
            serde_json::to_string(&BackgroundJobCompletionEvent::from_record(&record)).ok()
        } else {
            None
        };
        Ok(json!({
            "status": "updated",
            "job": record,
            "started_event": started_event,
            "completion_event": completion_event,
        }))
    })
    .await;
    match result {
        Ok(Ok(payload)) => {
            if let Some(event) = payload.get("started_event").and_then(|v| v.as_str()) {
                let _ = events_tx.send(event.to_string());
            }
            if let Some(event) = payload.get("completion_event").and_then(|v| v.as_str()) {
                let _ = events_tx.send(event.to_string());
            }
            Json(json!({
                "status": payload.get("status").cloned().unwrap_or(Value::Null),
                "job": payload.get("job").cloned().unwrap_or(Value::Null),
            }))
        }
        Ok(Err(error)) => Json(focusa_core::error_envelope::internal_error(
            "route",
            &error.to_string(),
        )),
        Err(error) => Json(focusa_core::error_envelope::internal_error(
            "join",
            &format!("join error: {error}"),
        )),
    }
}

async fn complete_job(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(job_id): axum::extract::Path<String>,
    Json(body): Json<CompleteJobBody>,
) -> Json<Value> {
    let path = crate::routes::events_sqlite::focusa_db_path(&state.config.data_dir);
    let events_tx = state.events_tx.clone();
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<Value> {
        let conn = rusqlite::Connection::open(path)?;
        focusa_core::background_job_store::ensure_schema(&conn)?;
        let Some(mut record) = focusa_core::background_job_store::load_job(&conn, &job_id)? else {
            return Ok(json!({"status": "missing", "job_id": job_id}));
        };
        // Idempotent settlement: a completed/failed/monitor-lost job can
        // never be re-settled (forged completions rejected).
        if record.completed_at.is_some() {
            return Ok(json!({
                "status": "already_settled",
                "job": record,
            }));
        }
        let status = match body.status.as_deref() {
            Some(value) => BackgroundJobStatus::parse(value),
            None if body.exit_code == 0 => BackgroundJobStatus::Completed,
            None => BackgroundJobStatus::Failed,
        };
        anyhow::ensure!(
            body.failure_class.is_none() || status == BackgroundJobStatus::Failed,
            "failure_class is valid only for failed background jobs"
        );
        if let Some(failure_class) = body.failure_class {
            anyhow::ensure!(
                body.exit_code == failure_class.exit_code(),
                "{} requires exit_code {}",
                failure_class.as_str(),
                failure_class.exit_code()
            );
        }
        record.status = status;
        record.failure_class = body.failure_class;
        record.exit_code = Some(body.exit_code);
        record.completed_at = Some(now_iso());
        record.output_tail =
            focusa_core::background_jobs::bounded_output_tail(&body.output_tail, 4096);
        if record.output_tail.is_empty() {
            record.output_tail =
                focusa_core::background_jobs::resolved_background_job_output_tail(&record);
        }
        focusa_core::background_job_store::upsert_job(&conn, &record)?;
        let envelope = BackgroundJobCompletionEvent::from_record(&record);
        // Duration stats feed the ETA for the next same-name job.
        if let (Some(started), Some(completed)) = (
            &record
                .started_at
                .parse::<chrono::DateTime<chrono::Utc>>()
                .ok(),
            &record
                .completed_at
                .as_ref()
                .and_then(|t| t.parse::<chrono::DateTime<chrono::Utc>>().ok()),
        ) {
            let duration_ms = (*completed - *started).num_milliseconds();
            if duration_ms > 0 {
                let _ = focusa_core::background_job_store::record_job_duration(
                    &conn,
                    &record.name,
                    duration_ms,
                );
            }
        }
        Ok(json!({
            "status": "completed",
            "job": record,
            "completion_event": envelope,
        }))
    })
    .await;
    match result {
        Ok(Ok(payload)) => {
            // Durable first: broadcast the completion envelope over SSE so
            // every surface (Pi uiCtx.notify, TUI, waiters) observes it.
            if let Some(event) = payload.get("completion_event") {
                if let Ok(serialized) = serde_json::to_string(event) {
                    let _ = events_tx.send(serialized);
                }
            }
            Json(payload)
        }
        Ok(Err(error)) => Json(focusa_core::error_envelope::internal_error(
            "route",
            &error.to_string(),
        )),
        Err(error) => Json(focusa_core::error_envelope::internal_error(
            "join",
            &format!("join error: {error}"),
        )),
    }
}

async fn get_job(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(job_id): axum::extract::Path<String>,
) -> Json<Value> {
    let path = crate::routes::events_sqlite::focusa_db_path(&state.config.data_dir);
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<Value> {
        let conn = rusqlite::Connection::open_with_flags(
            path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )?;
        let Some(record) = focusa_core::background_job_store::load_job(&conn, &job_id)? else {
            return Ok(json!({"status": "missing", "job_id": job_id}));
        };
        let elapsed_ms = record
            .started_at
            .parse::<chrono::DateTime<chrono::Utc>>()
            .ok()
            .map(|started| (chrono::Utc::now() - started).num_milliseconds().max(0));
        let eta_ms = focusa_core::background_job_store::eta_ms_for(&conn, &record.name)?;
        Ok(json!({
            "status": "ok",
            "job": record,
            "elapsed_ms": elapsed_ms,
            "eta_ms": eta_ms,
        }))
    })
    .await;
    match result {
        Ok(Ok(payload)) => Json(payload),
        Ok(Err(error)) => Json(focusa_core::error_envelope::internal_error(
            "route",
            &error.to_string(),
        )),
        Err(error) => Json(focusa_core::error_envelope::internal_error(
            "join",
            &format!("join error: {error}"),
        )),
    }
}

async fn list_jobs(State(state): State<Arc<AppState>>) -> Json<Value> {
    let path = crate::routes::events_sqlite::focusa_db_path(&state.config.data_dir);
    let events_tx = state.events_tx.clone();
    let may_reconcile_jobs =
        focusa_license::base_product_projection(state.license_guard.entitlement.as_ref())
            .is_ok_and(|projection| projection.permits_base_mutations);
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<Value> {
        let conn = if may_reconcile_jobs {
            rusqlite::Connection::open(path)?
        } else {
            rusqlite::Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?
        };
        let reconciled = if may_reconcile_jobs {
            focusa_core::background_job_store::reconcile_stale_jobs(&conn, chrono::Utc::now())?
        } else {
            Vec::new()
        };
        let completion_events = reconciled
            .iter()
            .map(BackgroundJobCompletionEvent::from_record)
            .collect::<Vec<_>>();
        let jobs = focusa_core::background_job_store::list_jobs(&conn)?;
        Ok(json!({
            "status": "ok",
            "jobs": jobs,
            "reconciled_count": completion_events.len(),
            "reconciliation_skipped": !may_reconcile_jobs,
            "reconciliation_events": completion_events,
        }))
    })
    .await;
    match result {
        Ok(Ok(mut payload)) => {
            if let Some(events) = payload
                .get("reconciliation_events")
                .and_then(Value::as_array)
            {
                for event in events {
                    if let Ok(serialized) = serde_json::to_string(event) {
                        let _ = events_tx.send(serialized);
                    }
                }
            }
            if let Some(object) = payload.as_object_mut() {
                object.remove("reconciliation_events");
            }
            Json(payload)
        }
        Ok(Err(error)) => Json(focusa_core::error_envelope::internal_error(
            "route",
            &error.to_string(),
        )),
        Err(error) => Json(focusa_core::error_envelope::internal_error(
            "join",
            &format!("join error: {error}"),
        )),
    }
}

/// Long-poll wait: returns immediately on completion, otherwise polls the
/// ledger every 500ms up to the timeout (silent-session wait pattern).
async fn wait_job(
    State(state): State<Arc<AppState>>,
    Query(query): Query<WaitQuery>,
) -> Json<Value> {
    let path = crate::routes::events_sqlite::focusa_db_path(&state.config.data_dir);
    let deadline = std::time::Instant::now() + Duration::from_millis(query.timeout_ms);
    // One connection reused for the whole wait — no per-poll open cost.
    let job_id = query.job_id.clone();
    let wait_future =
        tokio::task::spawn_blocking(move || -> anyhow::Result<Option<BackgroundJobRecord>> {
            let conn = match rusqlite::Connection::open_with_flags(
                &path,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
            ) {
                Ok(conn) => conn,
                Err(error) => return Err(anyhow::anyhow!("connection open failed: {error}")),
            };
            let deadline = std::time::Instant::now() + Duration::from_millis(query.timeout_ms);
            loop {
                let record = focusa_core::background_job_store::load_job(&conn, &job_id)?;
                if let Some(record) = record {
                    if matches!(
                        record.status,
                        BackgroundJobStatus::Completed
                            | BackgroundJobStatus::Failed
                            | BackgroundJobStatus::MonitorLost
                    ) {
                        return Ok(Some(record));
                    }
                }
                if std::time::Instant::now() >= deadline {
                    return Ok(None);
                }
                std::thread::sleep(Duration::from_millis(500));
            }
        });
    let record = wait_future.await;
    match record {
        Ok(Ok(Some(record))) => Json(json!({
            "status": "done",
            "job": record,
            "completion_event": BackgroundJobCompletionEvent::from_record(&record),
        })),
        Ok(Ok(None)) => Json(json!({"status": "timeout", "job_id": query.job_id})),
        Ok(Err(error)) => Json(json!({"status": "failed", "error": error.to_string()})),
        Err(error) => Json(json!({"status": "failed", "error": format!("join error: {error}")})),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_create_and_update_payloads_remain_valid() {
        let create: CreateJobBody = serde_json::from_value(json!({
            "name": "legacy",
            "command": "true",
            "cwd": ".",
            "pid": 42
        }))
        .expect("legacy create payload");
        assert_eq!(create.pid, Some(42));
        assert_eq!(create.process_start_token, None);

        let update: UpdateJobBody = serde_json::from_value(json!({
            "status": "running",
            "pid": 42
        }))
        .expect("legacy update payload");
        assert_eq!(update.pid, Some(42));
        assert_eq!(update.process_start_token, None);
    }
}
