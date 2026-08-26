//! Browser Fleet bridge (spec 181 F1/F2) — bounded read-only proxy to the
//! paired UIAI engine. The engine token is held server-side and never
//! exposed to clients. Fleet status is JSON; events are SSE passthrough.

use crate::server::AppState;
use axum::Json;
use futures_util::StreamExt;
use std::sync::Arc;

use axum::body::Body;

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::IntoResponse;

pub fn engine_base() -> String {
    std::env::var("UIAI_ENGINE_URL").unwrap_or_else(|_| "http://127.0.0.1:7456".into())
}

pub fn engine_token() -> Option<String> {
    std::env::var("UIAI_ENGINE_TOKEN")
        .ok()
        .filter(|t| !t.trim().is_empty())
}

fn auth_headers(headers: &mut HeaderMap) {
    if let Some(tok) = engine_token() {
        if let Ok(v) = format!("Bearer {}", tok).parse() {
            headers.insert(header::AUTHORIZATION, v);
        }
    }
}

/// GET /v1/browser-fleet/status — bounded proxy of engine health/fleet JSON.
pub async fn fleet_status(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .map_err(|e| internal(&state, format!("client build: {e}")))?;
    let mut headers = HeaderMap::new();
    auth_headers(&mut headers);
    let resp = client
        .get(format!("{}/api/health/browser", engine_base()))
        .headers(headers)
        .send()
        .await
        .map_err(|e| internal(&state, format!("engine unreachable: {e}")))?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    // Bounded payload: reject >512 KiB upstream bodies.
    if text.len() > 512 * 1024 {
        return Err(internal(&state, "engine payload exceeds bound".into()));
    }
    let value: serde_json::Value = serde_json::from_str(&text).unwrap_or(serde_json::json!({
        "status": "degraded",
        "degraded_reason": "engine_non_json",
        "upstream_status": status.as_u16(),
    }));
    Ok(Json(value))
}

fn internal(state: &AppState, msg: String) -> (StatusCode, Json<serde_json::Value>) {
    let _ = state;
    (
        StatusCode::BAD_GATEWAY,
        Json(serde_json::json!({
            "error": "browser_fleet_bridge_failed",
            "message": msg,
        })),
    )
}

pub fn router() -> axum::Router<Arc<crate::server::AppState>> {
    use axum::routing::get;
    axum::Router::new()
        .route("/v1/browser-fleet/status", get(fleet_status))
        .route("/v1/browser-fleet/stream", get(fleet_events))
}

/// GET /v1/browser-fleet/stream — SSE passthrough of engine
/// `focusa.stream_event.v1` envelopes (spec 181 F2). Read-only.
pub async fn fleet_events(
    State(state): State<Arc<AppState>>,
    raw_query: axum::extract::RawQuery,
) -> Result<axum::response::Response, (StatusCode, Json<serde_json::Value>)> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3600))
        .build()
        .map_err(|e| internal(&state, format!("client build: {e}")))?;
    let mut url = format!("{}/api/focusa-events", engine_base());
    if let Some(q) = raw_query.0 {
        url.push('?');
        url.push_str(&q);
    }
    let mut headers = HeaderMap::new();
    auth_headers(&mut headers);
    headers.insert(
        header::ACCEPT,
        header::HeaderValue::from_static("text/event-stream"),
    );
    let upstream = client
        .get(url)
        .headers(headers)
        .send()
        .await
        .map_err(|e| internal(&state, format!("engine stream unreachable: {e}")))?;
    if !upstream.status().is_success() {
        return Err(internal(
            &state,
            format!("engine stream status: {}", upstream.status()),
        ));
    }
    let stream = upstream
        .bytes_stream()
        .map(|chunk| chunk.map_err(std::io::Error::other));
    let body = Body::from_stream(stream);
    Ok((
        [
            (header::CONTENT_TYPE, "text/event-stream"),
            (header::CACHE_CONTROL, "no-cache"),
            (header::CONNECTION, "keep-alive"),
        ],
        body,
    )
        .into_response())
}
