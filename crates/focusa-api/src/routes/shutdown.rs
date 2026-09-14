use crate::server::{AppState, DaemonRuntimeIdentity};
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use axum::{Json, Router};
use focusa_core::daemon_lifecycle::{
    DAEMON_SHUTDOWN_REQUEST_SCHEMA, DaemonLockRecord, DaemonShutdownRequest,
};
use serde_json::{Value, json};
use std::sync::Arc;

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.iter()
        .zip(right)
        .fold(0_u8, |difference, (a, b)| difference | (a ^ b))
        == 0
}

fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .filter(|value| !value.is_empty())
}

fn bearer_authorized(
    presented: Option<&str>,
    shutdown_token: &str,
    admin_token: Option<&str>,
) -> bool {
    let Some(presented) = presented else {
        return false;
    };
    constant_time_eq(presented.as_bytes(), shutdown_token.as_bytes())
        || admin_token.is_some_and(|admin| {
            !admin.is_empty() && constant_time_eq(presented.as_bytes(), admin.as_bytes())
        })
}

fn exact_identity_matches(
    runtime: &DaemonRuntimeIdentity,
    request: &DaemonShutdownRequest,
    lock: &DaemonLockRecord,
) -> bool {
    request.schema == DAEMON_SHUTDOWN_REQUEST_SCHEMA
        && request.pid == runtime.process.pid
        && request.start_token == runtime.process.start_token
        && lock.pid == runtime.process.pid
        && lock.start_token == runtime.process.start_token
        && constant_time_eq(
            lock.shutdown_token.as_bytes(),
            runtime.shutdown_token.as_bytes(),
        )
}

fn response(status: StatusCode, code: &str, message: &str) -> (StatusCode, Json<Value>) {
    (
        status,
        Json(json!({
            "ok": false,
            "error": {"code": code, "message": message},
        })),
    )
}

async fn shutdown(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<DaemonShutdownRequest>,
) -> (StatusCode, Json<Value>) {
    let admin_token = std::env::var("FOCUSA_AUTH_TOKEN")
        .ok()
        .filter(|token| !token.is_empty());
    if !bearer_authorized(
        bearer_token(&headers),
        &state.daemon_runtime_identity.shutdown_token,
        admin_token.as_deref(),
    ) {
        return response(
            StatusCode::UNAUTHORIZED,
            "DAEMON_SHUTDOWN_UNAUTHORIZED",
            "exact daemon shutdown authorization is required",
        );
    }

    let lock_content =
        match tokio::fs::read_to_string(&state.daemon_runtime_identity.process.lock_path).await {
            Ok(content) => content,
            Err(_) => {
                return response(
                    StatusCode::CONFLICT,
                    "DAEMON_SHUTDOWN_LOCK_UNAVAILABLE",
                    "daemon lock ownership cannot be verified",
                );
            }
        };
    let lock = match DaemonLockRecord::parse(&lock_content) {
        Ok(lock) => lock,
        Err(_) => {
            return response(
                StatusCode::CONFLICT,
                "DAEMON_SHUTDOWN_LOCK_INVALID",
                "daemon lock ownership cannot be verified",
            );
        }
    };
    if !exact_identity_matches(&state.daemon_runtime_identity, &request, &lock) {
        return response(
            StatusCode::CONFLICT,
            "DAEMON_SHUTDOWN_IDENTITY_MISMATCH",
            "shutdown request does not match the exact daemon instance",
        );
    }

    let mut accepted = state.shutdown_accepted.lock().await;
    if *accepted {
        return response(
            StatusCode::CONFLICT,
            "DAEMON_SHUTDOWN_ALREADY_ACCEPTED",
            "shutdown was already accepted for this daemon instance",
        );
    }
    if state.shutdown_tx.send(true).is_err() {
        return response(
            StatusCode::SERVICE_UNAVAILABLE,
            "DAEMON_SHUTDOWN_CHANNEL_CLOSED",
            "daemon shutdown channel is unavailable",
        );
    }
    *accepted = true;

    (
        StatusCode::ACCEPTED,
        Json(json!({
            "ok": true,
            "schema": "focusa.daemon_shutdown_response.v1",
            "status": "accepted",
            "pid": request.pid,
            "start_token": request.start_token,
        })),
    )
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/v1/shutdown", post(shutdown))
}

#[cfg(test)]
mod tests {
    use super::*;
    use focusa_core::daemon_lifecycle::DaemonProcessIdentity;

    fn runtime() -> DaemonRuntimeIdentity {
        DaemonRuntimeIdentity {
            process: DaemonProcessIdentity::new(42, "start-token", "/tmp/focusa.lock"),
            shutdown_token: "shutdown-token".into(),
        }
    }

    fn request() -> DaemonShutdownRequest {
        DaemonShutdownRequest::new(42, "start-token")
    }

    fn lock() -> DaemonLockRecord {
        DaemonLockRecord {
            pid: 42,
            bind: "127.0.0.1:8787".into(),
            started_at: "2026-08-31T00:00:00Z".into(),
            start_token: "start-token".into(),
            shutdown_token: "shutdown-token".into(),
        }
    }

    #[test]
    fn exact_lock_process_and_request_identity_match() {
        assert!(exact_identity_matches(&runtime(), &request(), &lock()));
    }

    #[test]
    fn pid_start_token_and_lock_credential_mismatches_fail_closed() {
        let mut foreign_request = request();
        foreign_request.pid = 43;
        assert!(!exact_identity_matches(
            &runtime(),
            &foreign_request,
            &lock()
        ));
        foreign_request = request();
        foreign_request.start_token = "other-start".into();
        assert!(!exact_identity_matches(
            &runtime(),
            &foreign_request,
            &lock()
        ));
        let mut foreign_lock = lock();
        foreign_lock.shutdown_token = "other-shutdown".into();
        assert!(!exact_identity_matches(
            &runtime(),
            &request(),
            &foreign_lock
        ));
    }

    #[test]
    fn only_per_start_or_admin_bearer_authorizes() {
        assert!(bearer_authorized(
            Some("shutdown-token"),
            "shutdown-token",
            None
        ));
        assert!(bearer_authorized(
            Some("admin-token"),
            "shutdown-token",
            Some("admin-token")
        ));
        assert!(!bearer_authorized(
            Some("device-token"),
            "shutdown-token",
            Some("admin-token")
        ));
        assert!(!bearer_authorized(None, "shutdown-token", None));
    }
}
