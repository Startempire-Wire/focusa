//! Background job HTTP surface — create, lifecycle, complete (with SSE
//! broadcast), status, list, and long-poll wait. Mirrors the silent-session
//! completion pattern (#311): durable first, broadcast second.

use axum::extract::{Query, State};
use axum::routing::{get, post};
use axum::Json;
use axum::Router;
use focusa_core::background_jobs::{
    BackgroundJobCompletionEvent, BackgroundJobRecord, BackgroundJobStatus,
    BACKGROUND_JOB_SCHEMA,
};
use serde::Deserialize;
use serde_json::{json, Value};
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
        .route(
            "/v1/background-jobs/{job_id}/complete",
            post(complete_job),
        )
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
}

#[derive(Deserialize)]
pub struct CompleteJobBody {
    pub exit_code: i32,
    #[serde(default)]
    pub status: Option<String>,
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
        let job_id = body.job_id.unwrap_or_else(|| uuid::Uuid::now_v7().to_string());
        let record = BackgroundJobRecord {
            schema: BACKGROUND_JOB_SCHEMA.to_string(),
            job_id: job_id.clone(),
            name: body.name,
            command: body.command,
            cwd: body.cwd,
            status: BackgroundJobStatus::Queued,
            exit_code: None,
            pid: None,
            log_path: body
                .log_path
                .unwrap_or_else(|| format!("/tmp/focusa-bg-{job_id}.log")),
            started_at: now_iso(),
            completed_at: None,
        };
        focusa_core::background_job_store::upsert_job(&conn, &record)?;
        Ok(json!({ "status": "queued", "job": record }))
    })
    .await;
    match result {
        Ok(Ok(payload)) => Json(payload),
        Ok(Err(error)) => Json(focusa_core::error_envelope::internal_error("route", &error.to_string())),
        Err(error) => Json(focusa_core::error_envelope::internal_error("join", &format!("join error: {error}"))),
    }
}

async fn update_job(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(job_id): axum::extract::Path<String>,
    Json(body): Json<UpdateJobBody>,
) -> Json<Value> {
    let path = crate::routes::events_sqlite::focusa_db_path(&state.config.data_dir);
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<Value> {
        let conn = rusqlite::Connection::open(path)?;
        focusa_core::background_job_store::ensure_schema(&conn)?;
        let Some(mut record) =
            focusa_core::background_job_store::load_job(&conn, &job_id)?
        else {
            return Ok(json!({"status": "missing", "job_id": job_id}));
        };
        if let Some(status) = body.status {
            record.status = BackgroundJobStatus::parse(&status);
        }
        record.pid = body.pid.or(record.pid);
        focusa_core::background_job_store::upsert_job(&conn, &record)?;
        Ok(json!({"status": "updated", "job": record}))
    })
    .await;
    match result {
        Ok(Ok(payload)) => Json(payload),
        Ok(Err(error)) => Json(focusa_core::error_envelope::internal_error("route", &error.to_string())),
        Err(error) => Json(focusa_core::error_envelope::internal_error("join", &format!("join error: {error}"))),
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
        let Some(mut record) =
            focusa_core::background_job_store::load_job(&conn, &job_id)?
        else {
            return Ok(json!({"status": "missing", "job_id": job_id}));
        };
        let status = match body.status.as_deref() {
            Some(value) => BackgroundJobStatus::parse(value),
            None if body.exit_code == 0 => BackgroundJobStatus::Completed,
            None => BackgroundJobStatus::Failed,
        };
        record.status = status;
        record.exit_code = Some(body.exit_code);
        record.completed_at = Some(now_iso());
        focusa_core::background_job_store::upsert_job(&conn, &record)?;
        let envelope = BackgroundJobCompletionEvent::from_record(&record);
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
        Ok(Err(error)) => Json(focusa_core::error_envelope::internal_error("route", &error.to_string())),
        Err(error) => Json(focusa_core::error_envelope::internal_error("join", &format!("join error: {error}"))),
    }
}

async fn get_job(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(job_id): axum::extract::Path<String>,
) -> Json<Value> {
    let path = crate::routes::events_sqlite::focusa_db_path(&state.config.data_dir);
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<Value> {
        let conn = rusqlite::Connection::open(path)?;
        focusa_core::background_job_store::ensure_schema(&conn)?;
        let Some(record) = focusa_core::background_job_store::load_job(&conn, &job_id)? else {
            return Ok(json!({"status": "missing", "job_id": job_id}));
        };
        Ok(json!({"status": "ok", "job": record}))
    })
    .await;
    match result {
        Ok(Ok(payload)) => Json(payload),
        Ok(Err(error)) => Json(focusa_core::error_envelope::internal_error("route", &error.to_string())),
        Err(error) => Json(focusa_core::error_envelope::internal_error("join", &format!("join error: {error}"))),
    }
}

async fn list_jobs(State(state): State<Arc<AppState>>) -> Json<Value> {
    let path = crate::routes::events_sqlite::focusa_db_path(&state.config.data_dir);
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<Value> {
        let conn = rusqlite::Connection::open(path)?;
        focusa_core::background_job_store::ensure_schema(&conn)?;
        let jobs = focusa_core::background_job_store::list_jobs(&conn)?;
        Ok(json!({"status": "ok", "jobs": jobs}))
    })
    .await;
    match result {
        Ok(Ok(payload)) => Json(payload),
        Ok(Err(error)) => Json(focusa_core::error_envelope::internal_error("route", &error.to_string())),
        Err(error) => Json(focusa_core::error_envelope::internal_error("join", &format!("join error: {error}"))),
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
    loop {
        let path = path.clone();
        let job_id = query.job_id.clone();
        let record = tokio::task::spawn_blocking(move || -> anyhow::Result<Option<BackgroundJobRecord>> {
            let conn = rusqlite::Connection::open(path)?;
            focusa_core::background_job_store::ensure_schema(&conn)?;
            focusa_core::background_job_store::load_job(&conn, &job_id)
        })
        .await;
        match record {
            Ok(Ok(Some(record)))
                if matches!(
                    record.status,
                    BackgroundJobStatus::Completed
                        | BackgroundJobStatus::Failed
                        | BackgroundJobStatus::MonitorLost
                ) =>
            {
                return Json(json!({
                    "status": "done",
                    "job": record,
                    "completion_event": BackgroundJobCompletionEvent::from_record(&record),
                }));
            }
            Ok(Ok(None)) => {
                return Json(json!({"status": "missing", "job_id": query.job_id}));
            }
            Ok(Ok(_)) => {}
            _ => {
                return Json(json!({"status": "failed", "error": "ledger read failed"}));
            }
        }
        if std::time::Instant::now() >= deadline {
            return Json(json!({"status": "timeout", "job_id": query.job_id}));
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}
