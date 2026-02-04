//! Passthrough adapter — fail-safe mode.
//!
//! When Focusa processing fails (state unavailable, assembly error, etc.),
//! the adapter passes through the raw request unchanged.
//!
//! Emits a failure event but does NOT block the harness.
//! The user's request always reaches the model — Focusa is never a SPOF.

use crate::adapters::openai::{ChatCompletionRequest, forward_request};
use reqwest::Client;
use serde_json::Value;

/// Forward request unchanged — Focusa is transparent.
pub async fn passthrough(
    client: &Client,
    upstream_url: &str,
    api_key: &str,
    request: &ChatCompletionRequest,
) -> anyhow::Result<Value> {
    tracing::warn!("Focusa passthrough: forwarding request without cognitive enhancement");
    forward_request(client, upstream_url, api_key, request).await
}
