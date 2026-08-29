//! Transport-neutral communications broker facade (Plan 180).
//! Connector credentials/profile state stay in the private broker. This route
//! forwards only versioned capability requests and redacted envelopes.

use axum::{
    Json, Router,
    extract::{Path, Query},
    http::StatusCode,
    routing::{get, post},
};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    net::IpAddr,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    sync::{Arc, OnceLock},
    time::Duration,
};

use crate::server::AppState;

fn broker_config() -> Result<(String, String), &'static str> {
    let base = std::env::var("FOCUSA_SMS_BROKER_URL").map_err(|_| "sms_broker_unconfigured")?;
    let parsed = reqwest::Url::parse(&base).map_err(|_| "sms_broker_url_invalid")?;
    if parsed.scheme() != "http" || parsed.username() != "" || parsed.password().is_some() {
        return Err("sms_broker_url_invalid");
    }
    let private_host = match parsed.host_str() {
        Some("localhost") => true,
        Some(host) => host
            .parse::<IpAddr>()
            .map(|ip| match ip {
                IpAddr::V4(value) => value.is_loopback() || value.is_private(),
                IpAddr::V6(value) => value.is_loopback() || value.is_unique_local(),
            })
            .unwrap_or(false),
        None => false,
    };
    if !private_host {
        return Err("sms_broker_url_not_private");
    }

    let token_path = PathBuf::from(
        std::env::var("FOCUSA_SMS_BROKER_TOKEN_FILE")
            .map_err(|_| "sms_broker_token_unconfigured")?,
    );
    let metadata =
        std::fs::symlink_metadata(&token_path).map_err(|_| "sms_broker_token_unavailable")?;
    if !metadata.is_file()
        || metadata.file_type().is_symlink()
        || metadata.permissions().mode() & 0o077 != 0
    {
        return Err("sms_broker_token_permissions_invalid");
    }
    let token = std::fs::read_to_string(token_path).map_err(|_| "sms_broker_token_unavailable")?;
    let token = token.trim().to_string();
    if token.len() < 32 {
        return Err("sms_broker_token_unavailable");
    }
    Ok((base.trim_end_matches('/').to_string(), token))
}

fn broker_client() -> Result<&'static reqwest::Client, &'static str> {
    static CLIENT: OnceLock<Result<reqwest::Client, &'static str>> = OnceLock::new();
    CLIENT
        .get_or_init(|| {
            reqwest::Client::builder()
                .connect_timeout(Duration::from_secs(3))
                .timeout(Duration::from_secs(15))
                .build()
                .map_err(|_| "sms_broker_client_unavailable")
        })
        .as_ref()
        .map_err(|failure| *failure)
}

fn unavailable(failure_class: &str) -> (StatusCode, Json<Value>) {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(json!({
            "schema": "focusa.tool_result_v1", "canonical": true, "ok": false,
            "status": "degraded", "failure_class": failure_class,
            "summary": "Private SMS broker is unavailable; connector state was not exposed."
        })),
    )
}

async fn forward(
    method: reqwest::Method,
    path: &str,
    query: HashMap<String, String>,
    body: Option<Value>,
) -> (StatusCode, Json<Value>) {
    let (base, token) = match broker_config() {
        Ok(value) => value,
        Err(error) => return unavailable(error),
    };
    let url = format!("{base}{path}");
    let client = match broker_client() {
        Ok(value) => value,
        Err(error) => return unavailable(error),
    };
    let mut request = client.request(method, url).bearer_auth(token).query(&query);
    if let Some(value) = body {
        request = request.json(&value);
    }
    let response = match request.send().await {
        Ok(value) => value,
        Err(_) => return unavailable("sms_broker_connect_failed"),
    };
    let status =
        StatusCode::from_u16(response.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
    if response
        .content_length()
        .is_some_and(|size| size > 1_048_576)
    {
        return unavailable("sms_broker_response_too_large");
    }
    let value = match response.bytes().await {
        Ok(bytes) if bytes.len() <= 1_048_576 => serde_json::from_slice::<Value>(&bytes).unwrap_or_else(|_| {
            json!({"schema":"focusa.tool_result_v1","canonical":true,"ok":false,"status":"blocked","failure_class":"sms_broker_invalid_envelope","summary":"Private SMS broker returned an invalid envelope."})
        }),
        _ => return unavailable("sms_broker_response_too_large"),
    };
    (status, Json(value))
}

async fn health() -> (StatusCode, Json<Value>) {
    forward(reqwest::Method::GET, "/v1/sms/health", HashMap::new(), None).await
}
async fn enrollment() -> (StatusCode, Json<Value>) {
    forward(
        reqwest::Method::GET,
        "/v1/sms/enrollment",
        HashMap::new(),
        None,
    )
    .await
}
async fn threads(Query(query): Query<HashMap<String, String>>) -> (StatusCode, Json<Value>) {
    forward(reqwest::Method::GET, "/v1/sms/threads", query, None).await
}
async fn messages(
    Path(thread): Path<String>,
    Query(query): Query<HashMap<String, String>>,
) -> (StatusCode, Json<Value>) {
    if thread.is_empty()
        || !thread
            .chars()
            .all(|value| value.is_ascii_alphanumeric() || matches!(value, '-' | '_' | '.' | ':'))
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(
                json!({"schema":"focusa.tool_result_v1","canonical":true,"ok":false,"status":"blocked","failure_class":"invalid_thread_handle"}),
            ),
        );
    }
    let path = format!("/v1/sms/threads/{thread}/messages");
    forward(reqwest::Method::GET, &path, query, None).await
}
async fn search(Query(query): Query<HashMap<String, String>>) -> (StatusCode, Json<Value>) {
    forward(reqwest::Method::GET, "/v1/sms/search", query, None).await
}
async fn events(Query(query): Query<HashMap<String, String>>) -> (StatusCode, Json<Value>) {
    forward(reqwest::Method::GET, "/v1/sms/events", query, None).await
}
async fn challenge(Json(body): Json<Value>) -> (StatusCode, Json<Value>) {
    forward(
        reqwest::Method::POST,
        "/v1/sms/otp/challenges",
        HashMap::new(),
        Some(body),
    )
    .await
}
async fn inject(Json(body): Json<Value>) -> (StatusCode, Json<Value>) {
    forward(
        reqwest::Method::POST,
        "/v1/sms/otp/inject",
        HashMap::new(),
        Some(body),
    )
    .await
}
async fn send(Json(body): Json<Value>) -> (StatusCode, Json<Value>) {
    forward(
        reqwest::Method::POST,
        "/v1/sms/send",
        HashMap::new(),
        Some(body),
    )
    .await
}
async fn checkpoint(Json(body): Json<Value>) -> (StatusCode, Json<Value>) {
    forward(
        reqwest::Method::POST,
        "/v1/sms/checkpoint",
        HashMap::new(),
        Some(body),
    )
    .await
}
async fn revoke(Json(body): Json<Value>) -> (StatusCode, Json<Value>) {
    forward(
        reqwest::Method::POST,
        "/v1/sms/revoke",
        HashMap::new(),
        Some(body),
    )
    .await
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/sms/health", get(health))
        .route("/v1/sms/enrollment", get(enrollment))
        .route("/v1/sms/threads", get(threads))
        .route("/v1/sms/threads/{thread}/messages", get(messages))
        .route("/v1/sms/search", get(search))
        .route("/v1/sms/events", get(events))
        .route("/v1/sms/otp/challenges", post(challenge))
        .route("/v1/sms/otp/inject", post(inject))
        .route("/v1/sms/send", post(send))
        .route("/v1/sms/checkpoint", post(checkpoint))
        .route("/v1/sms/revoke", post(revoke))
}
