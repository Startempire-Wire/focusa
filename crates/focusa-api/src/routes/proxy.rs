//! Proxy routes — HTTP proxy for AI providers with full turn tracking.
//!
//! POST /proxy/v1/chat/completions — OpenAI-compatible
//! POST /proxy/v1/messages — Anthropic (Claude)
//!
//! Per spec G1-detail-04-proxy-adapter.md:
//! 1. TurnStart — record turn beginning
//! 2. PromptAssemble — enhance with Focusa context
//! 3. Forward to upstream
//! 4. TurnComplete — record result (success or failure)
//! 5. Emit signals to Focus Gate

use crate::server::AppState;
use async_stream::try_stream;
use axum::body::{Body, Bytes};
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode, header::CONTENT_TYPE};
use axum::response::{IntoResponse, Response};
use axum::{Json, Router, routing::post};
use chrono::Utc;
use focusa_core::adapters::anthropic::{self, MessagesRequest};
use focusa_core::adapters::openai::{
    self, ChatCompletionRequest, extract_assistant_output, extract_usage,
};
use focusa_core::adapters::passthrough;
use focusa_core::types::*;
use futures_core::Stream;
use reqwest::Client;
use serde_json::{Value, json};
use std::pin::Pin;
use std::sync::Arc;
use tokio_stream::StreamExt;

const DEFAULT_UPSTREAM: &str = "https://api.openai.com/v1/chat/completions";
const DEFAULT_MESSAGES_UPSTREAM: &str = "https://api.anthropic.com/v1/messages";
const DEFAULT_KIMI_UPSTREAM: &str = "https://api.kimi.com/coding/v1/messages";

static UPSTREAM_CLIENT: std::sync::OnceLock<Client> = std::sync::OnceLock::new();

fn get_client() -> &'static Client {
    UPSTREAM_CLIENT.get_or_init(|| {
        Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .expect("failed to build HTTP client")
    })
}

fn upstream_url() -> String {
    std::env::var("FOCUSA_UPSTREAM_URL").unwrap_or_else(|_| DEFAULT_UPSTREAM.into())
}

fn messages_upstream_url() -> String {
    std::env::var("FOCUSA_MESSAGES_UPSTREAM").or_else(|_| std::env::var("FOCUSA_ANTHROPIC_UPSTREAM")).unwrap_or_else(|_| DEFAULT_MESSAGES_UPSTREAM.into())
}

fn kimi_upstream_url() -> String {
    std::env::var("FOCUSA_KIMI_UPSTREAM").unwrap_or_else(|_| DEFAULT_KIMI_UPSTREAM.into())
}

fn is_kimi_model(model: &str) -> bool {
    let lower = model.to_lowercase();
    lower.starts_with("k2") || lower.contains("kimi")
}

fn resolve_messages_upstream(model: &str) -> String {
    if is_kimi_model(model) {
        return kimi_upstream_url();
    }
    messages_upstream_url()
}

fn is_streaming(extra: &Value) -> bool {
    extra
        .get("stream")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

fn proxy_compat_mode_enabled() -> bool {
    std::env::var("FOCUSA_PROXY_COMPAT_MODE")
        .map(|v| {
            matches!(
                v.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

async fn ensure_session(state: &Arc<AppState>) {
    let needs_session = {
        let focusa = state.focusa.read().await;
        focusa.session.is_none()
    };
    if !needs_session {
        return;
    }
    let _ = state
        .command_tx
        .send(Action::InstanceConnect {
            kind: InstanceKind::Background,
        })
        .await;
    let _ = state
        .command_tx
        .send(Action::StartSession {
            adapter_id: Some("proxy".into()),
            workspace_id: None,
            instance_id: None,
        })
        .await;
}

fn strip_tool_markup_text(input: &str) -> String {
    anthropic::strip_tool_markup_text(input)
}

fn count_tool_markup_markers(input: &str) -> usize {
    [
        "<tool>",
        "</tool>",
        "<parameter>",
        "</parameter>",
        "[tool_use:",
        "[tool_result:",
        "<function_calls>",
        "</function_calls>",
        "<invoke",
        "</invoke>",
    ]
    .iter()
    .map(|m| input.matches(m).count())
    .sum()
}

fn sanitize_messages_response_for_discord(value: &mut Value) -> usize {
    let mut removed_markers = 0usize;
    if let Some(content) = value.get_mut("content").and_then(|v| v.as_array_mut()) {
        for block in content {
            if block.get("type").and_then(|v| v.as_str()) == Some("text") {
                if let Some(text) = block.get("text").and_then(|v| v.as_str()) {
                    let before = count_tool_markup_markers(text);
                    let cleaned = strip_tool_markup_text(text);
                    let after = count_tool_markup_markers(&cleaned);
                    removed_markers += before.saturating_sub(after);
                    block["text"] = Value::String(cleaned);
                }
            }
        }
    }
    removed_markers
}

fn extract_messages_stream_text(body: &str) -> (String, usize) {
    let mut out = String::new();
    for line in body.lines() {
        let line = line.trim();
        if !line.starts_with("data:") {
            continue;
        }
        let data = line.trim_start_matches("data:").trim();
        if data == "[DONE]" {
            break;
        }
        if let Ok(value) = serde_json::from_str::<Value>(data) {
            if value.get("type").and_then(|v| v.as_str()) == Some("content_block_delta") {
                if let Some(text) = value
                    .get("delta")
                    .and_then(|d| d.get("text"))
                    .and_then(|t| t.as_str())
                {
                    out.push_str(text);
                }
            }
        }
    }
    let removed = count_tool_markup_markers(&out);
    (strip_tool_markup_text(&out), removed)
}

fn messages_auth(headers: &HeaderMap) -> Option<anthropic::AnthropicAuth> {
    if let Ok(key) = std::env::var("FOCUSA_MESSAGES_API_KEY").or_else(|_| std::env::var("FOCUSA_ANTHROPIC_KEY")) {
        if !key.is_empty() {
            return Some(anthropic::AnthropicAuth::ApiKey(key));
        }
    }
    if let Ok(token) = std::env::var("FOCUSA_MESSAGES_BEARER").or_else(|_| std::env::var("FOCUSA_ANTHROPIC_BEARER")) {
        if !token.is_empty() {
            return Some(anthropic::AnthropicAuth::Bearer(token));
        }
    }
    if let Some(key) = headers.get("x-api-key").and_then(|v| v.to_str().ok()) {
        return Some(anthropic::AnthropicAuth::ApiKey(key.to_string()));
    }
    if let Some(token) = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
    {
        return Some(anthropic::AnthropicAuth::Bearer(token.to_string()));
    }
    None
}

fn api_key(headers: &HeaderMap) -> Option<String> {
    if let Ok(key) = std::env::var("FOCUSA_API_KEY") {
        return Some(key);
    }
    headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(String::from)
}

/// Create a signal for the Focus Gate.
fn create_signal(kind: SignalKind, summary: impl Into<String>) -> Signal {
    Signal {
        id: uuid::Uuid::now_v7(),
        ts: Utc::now(),
        origin: SignalOrigin::Adapter,
        kind,
        frame_context: None,
        summary: summary.into(),
        payload_ref: None,
        tags: vec![],
    }
}

async fn stream_messages_response(
    state: Arc<AppState>,
    auth: anthropic::AnthropicAuth,
    url: String,
    request: MessagesRequest,
    turn_id: String,
    user_input: String,
) -> Result<Response, (StatusCode, Json<Value>)> {
    let client = get_client();

    let mut req = client
        .post(url)
        .header("anthropic-version", "2023-06-01")
        .header("Content-Type", "application/json");

    match auth {
        anthropic::AnthropicAuth::ApiKey(key) => {
            req = req.header("x-api-key", key);
        }
        anthropic::AnthropicAuth::Bearer(token) => {
            req = req.header("Authorization", format!("Bearer {}", token));
        }
    }

    let resp = req.json(&request).send().await.map_err(|e| {
        (
            StatusCode::BAD_GATEWAY,
            Json(json!({"error": e.to_string()})),
        )
    })?;

    let status = resp.status();
    let headers = resp.headers().clone();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err((
            StatusCode::BAD_GATEWAY,
            Json(json!({"error": format!("Anthropic returned HTTP {}: {}", status, body)})),
        ));
    }

    let mut stream = resp.bytes_stream();
    let body_stream = try_stream! {
        let mut buf: Vec<u8> = Vec::new();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            buf.extend_from_slice(&chunk);
            yield chunk;
        }

        let body = String::from_utf8_lossy(&buf);
        let (assistant_output, removed_markers) = extract_messages_stream_text(&body);

        if removed_markers > 0 {
            tracing::warn!(
                turn_id = %turn_id,
                removed_markers,
                "Sanitized leaked tool markup from Anthropic stream response"
            );
        }

        if !assistant_output.is_empty() {
            let summary: String = assistant_output.chars().take(200).collect();
            let signal = create_signal(SignalKind::AssistantOutput, summary);
            let _ = state.command_tx.send(Action::IngestSignal { signal }).await;
        }

        let event = FocusaEvent::TurnCompleted {
            turn_id: turn_id.clone(),
            harness_name: "messages".into(),
            raw_user_input: Some(user_input.clone()),
            assistant_output: if assistant_output.is_empty() { None } else { Some(assistant_output) },
            artifacts_used: vec![],
            errors: vec![],
            prompt_tokens: None,
            completion_tokens: None,
        };
        let _ = state.command_tx.send(Action::EmitEvent { event }).await;

        let mut focusa = state.focusa.write().await;
        if let Some(ref turn) = focusa.active_turn
            && turn.turn_id == turn_id
        {
            focusa.active_turn.take();
        }
    };

    let body_stream: Pin<Box<dyn Stream<Item = Result<Bytes, std::io::Error>> + Send>> =
        Box::pin(body_stream);

    let mut response = Response::builder().status(status);
    if let Some(ct) = headers.get(CONTENT_TYPE) {
        response = response.header(CONTENT_TYPE, ct.clone());
    }

    Ok(response.body(Body::from_stream(body_stream)).unwrap())
}

/// POST /proxy/v1/chat/completions — OpenAI proxy with turn tracking.
async fn chat_completions(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<ChatCompletionRequest>,
) -> Result<Response, (StatusCode, Json<Value>)> {
    let key = api_key(&headers).ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "No API key — set FOCUSA_API_KEY or pass Authorization header"})),
        )
    })?;

    ensure_session(&state).await;

    let turn_id = uuid::Uuid::now_v7().to_string();

    // 1. TURN START — Record in state.
    let user_input = request
        .messages
        .iter()
        .find(|m| m.role == "user")
        .map(|m| m.content.clone())
        .unwrap_or_default();

    {
        let mut focusa = state.focusa.write().await;
        focusa.active_turn = Some(ActiveTurn {
            turn_id: turn_id.clone(),
            adapter_id: "openai-proxy".to_string(),
            harness_name: "openai".to_string(),
            started_at: Utc::now(),
            raw_user_input: Some(user_input.clone()),
            assembled_prompt: None,
        });
    }
    tracing::info!(turn_id = %turn_id, harness = "openai", "Turn started");

    let start_event = FocusaEvent::TurnStarted {
        turn_id: turn_id.clone(),
        harness_name: "openai".into(),
        adapter_id: "openai-proxy".into(),
        raw_user_input: Some(user_input.clone()),
    };
    let _ = state
        .command_tx
        .send(Action::EmitEvent { event: start_event })
        .await;

    // Emit user_input signal to Focus Gate (summary max 200 chars).
    if !user_input.is_empty() {
        let summary: String = user_input.chars().take(200).collect();
        let signal = create_signal(SignalKind::UserInput, summary);
        let _ = state.command_tx.send(Action::IngestSignal { signal }).await;
    }

    let url = upstream_url();
    let client = get_client();

    // 1b. CONTEXT CORE + PAIRING ENGINE — Background enrichment (§9.5, §10B.1).
    // Spawned as async task so they DON'T BLOCK the hot path.
    // Results land in Focus State via command_tx → available for NEXT turn's assembly.
    // Current turn uses whatever state exists from previous enrichment.
    {
        let enrichment_state = state.clone();
        let enrichment_turn_id = turn_id.clone();
        let enrichment_client = client.clone();
        tokio::spawn(async move {
            // Context Core → constraints
            if let Ok(Ok(resp)) = tokio::time::timeout(
                std::time::Duration::from_secs(2),
                enrichment_client.get("http://127.0.0.1:7400/me").send(),
            ).await {
                if let Ok(ctx_json) = resp.json::<serde_json::Value>().await {
                    let mut constraints = Vec::new();
                    if let Some(interruptibility) = ctx_json.pointer("/state/interruptibility").and_then(|v| v.as_str()) {
                        if interruptibility == "very_low" {
                            constraints.push("Operator interruptibility: very low — do not ask questions, queue them".to_string());
                        } else if interruptibility == "low" {
                            constraints.push("Operator interruptibility: low — minimize questions".to_string());
                        }
                    }
                    if ctx_json.pointer("/timely/is_quiet_hours").and_then(|v| v.as_bool()).unwrap_or(false) {
                        constraints.push("Quiet hours — be concise, avoid churn".to_string());
                    }
                    if let Some(agent_should) = ctx_json.pointer("/policy/agent_should").and_then(|v| v.as_str()) {
                        if agent_should == "queue_questions" {
                            constraints.push("Policy: queue questions, do not interrupt operator".to_string());
                        }
                    }
                    if !constraints.is_empty() {
                        let focusa_read = enrichment_state.focusa.read().await;
                        if let Some(frame_id) = focusa_read.focus_stack.active_id {
                            let existing: Vec<String> = focusa_read.focus_stack.frames.iter()
                                .find(|f| f.id == frame_id)
                                .map(|f| f.focus_state.constraints.clone())
                                .unwrap_or_default();
                            drop(focusa_read);
                            let new_c: Vec<String> = constraints.into_iter().filter(|c| !existing.contains(c)).collect();
                            if !new_c.is_empty() {
                                let _ = enrichment_state.command_tx.send(Action::UpdateCheckpointDelta {
                                    frame_id,
                                    turn_id: enrichment_turn_id.clone(),
                                    delta: FocusStateDelta { constraints: Some(new_c), ..Default::default() },
                                }).await;
                            }
                        }
                    }
                }
            }

            // Pairing Engine → Focus Gate signals
            if let Some(sb_token) = std::env::var("SCOREBOARD_TOKEN").ok().or_else(|| std::env::var("GATEWAY_TOKEN").ok()) {
                if let Ok(Ok(resp)) = tokio::time::timeout(
                    std::time::Duration::from_secs(2),
                    enrichment_client.get("http://127.0.0.1:8100/v1/score")
                        .header("Authorization", format!("Bearer {}", sb_token))
                        .send(),
                ).await {
                    if let Ok(score_json) = resp.json::<serde_json::Value>().await {
                        let mut signal_summaries = Vec::new();
                        if let Some(drift_score) = score_json.pointer("/drift/score").and_then(|v| v.as_f64()) {
                            if drift_score < 30.0 {
                                signal_summaries.push(format!("Operator-agent drift score: {} (disconnected)", drift_score));
                            }
                        }
                        if score_json.pointer("/drift/rabbit/active").and_then(|v| v.as_bool()).unwrap_or(false) {
                            let rt = score_json.pointer("/drift/rabbit/type").and_then(|v| v.as_str()).unwrap_or("unknown");
                            signal_summaries.push(format!("R.A.B.I.T. detected: {}", rt));
                        }
                        if score_json.pointer("/drift/handshake_streak").and_then(|v| v.as_i64()).unwrap_or(1) == 0 {
                            signal_summaries.push("Neural handshake streak broken".to_string());
                        }
                        for summary in signal_summaries {
                            let signal = Signal {
                                id: uuid::Uuid::now_v7(),
                                ts: Utc::now(),
                                origin: SignalOrigin::Adapter,
                                kind: SignalKind::Warning,
                                frame_context: None,
                                summary,
                                payload_ref: None,
                                tags: vec![],
                            };
                            let _ = enrichment_state.command_tx.send(Action::IngestSignal { signal }).await;
                        }
                    }
                }
            }
        });
    }

    // 1c. PRE-TURN ENRICHMENT — Fast Mem0 + Wiki query (§11.7).
    // Extract keywords from user input, query Mem0 AND Wiki in parallel,
    // inject results into state BEFORE assembly for same-turn availability.
    {
        if !user_input.is_empty() {
            // Keyword extraction: split on whitespace, filter stopwords, take top 5
            let stopwords = ["the","a","an","is","are","was","were","be","been","being",
                "have","has","had","do","does","did","will","would","could","should",
                "may","might","can","shall","to","of","in","for","on","with","at","by",
                "from","as","into","about","like","through","after","over","between",
                "out","up","it","its","this","that","these","those","i","me","my","we",
                "you","your","he","she","they","them","and","or","but","not","so","if"];
            let keywords: Vec<&str> = user_input.split_whitespace()
                .filter(|w| w.len() > 2 && !stopwords.contains(&w.to_lowercase().as_str()))
                .take(5)
                .collect();
            let keyword_query = keywords.join(" ");
            let query_text = if keyword_query.is_empty() {
                if user_input.len() > 100 { &user_input[..100] } else { &user_input }
            } else {
                &keyword_query
            };

            // Parallel: Mem0 search + Wiki search (50ms each)
            let mem0_client = client.clone();
            let mem0_query = query_text.to_string();
            let wiki_query = query_text.to_string();

            let (mem0_result, wiki_result) = tokio::join!(
                async {
                    tokio::time::timeout(
                        std::time::Duration::from_millis(50),
                        mem0_client.post("http://127.0.0.1:8200/v1/search")
                            .json(&serde_json::json!({
                                "query": mem0_query,
                                "namespace": "wirebot_verious",
                                "limit": 5
                            }))
                            .send(),
                    ).await
                },
                async {
                    tokio::time::timeout(
                        std::time::Duration::from_millis(50),
                        tokio::process::Command::new("wb")
                            .args(["wiki", "search", &wiki_query, "--format", "json", "--limit", "3"])
                            .output(),
                    ).await
                },
            );

            let mut focusa = state.focusa.write().await;

            // Inject Mem0 results
            if let Ok(Ok(resp)) = mem0_result {
                if let Ok(data) = resp.json::<serde_json::Value>().await {
                    if let Some(results) = data.get("results").and_then(|v| v.as_array()) {
                        for (i, mem) in results.iter().enumerate().take(3) {
                            if let Some(text) = mem.get("memory").and_then(|v| v.as_str()) {
                                focusa_core::memory::semantic::upsert(
                                    &mut focusa.memory,
                                    format!("mem0.preturn.{}", i),
                                    text.to_string(),
                                    focusa_core::types::MemorySource::Mem0,
                                );
                            }
                        }
                        if !results.is_empty() {
                            tracing::debug!(count = results.len().min(3), "Pre-turn: Mem0 memories injected");
                        }
                    }
                }
            }

            // Inject Wiki results
            if let Ok(Ok(output)) = wiki_result {
                if output.status.success() {
                    if let Ok(wiki_json) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
                        if let Some(pages) = wiki_json.as_array().or_else(|| wiki_json.get("pages").and_then(|v| v.as_array())) {
                            for (i, page) in pages.iter().enumerate().take(2) {
                                let title = page.get("title").and_then(|v| v.as_str()).unwrap_or("");
                                let path = page.get("path").and_then(|v| v.as_str()).unwrap_or("");
                                if !title.is_empty() {
                                    focusa_core::memory::semantic::upsert(
                                        &mut focusa.memory,
                                        format!("wiki.preturn.{}", i),
                                        format!("Wiki: {} ({})", title, path),
                                        focusa_core::types::MemorySource::Worker,
                                    );
                                }
                            }
                            if !pages.is_empty() {
                                tracing::debug!(count = pages.len().min(2), "Pre-turn: Wiki results injected");
                            }
                        }
                    }
                }
            }

            drop(focusa);
        }
    }

    // 2. PROMPT ASSEMBLY — Enhance with Focusa context.
    let focusa_state = state.focusa.read().await;
    let result = openai::process_request(request.clone(), &focusa_state, &state.config);
    drop(focusa_state);

    // Update active turn with assembled prompt if enhancement occurred.
    if let Some(ref proxy_result) = result {
        let mut focusa = state.focusa.write().await;
        if let Some(ref mut turn) = focusa.active_turn {
            if turn.turn_id == turn_id {
                turn.assembled_prompt = Some(proxy_result.assembly.content.clone());
            }
        }
        drop(focusa);

        // Emit PromptAssembled event per G1-detail-11 §Events.
        let prompt_event = proxy_result.assembly.to_event(Some(turn_id.clone().into()));
        let _ = state.command_tx.send(Action::EmitEvent { event: prompt_event }).await;
    }

    // 3. FORWARD TO UPSTREAM.
    let (response, error_str) = match result {
        Some(proxy_result) => {
            match openai::forward_request(client, &url, &key, &proxy_result.request).await {
                Ok(resp) => (resp, None),
                Err(e) => {
                    tracing::error!("Upstream failed: {}", e);
                    (json!({"error": e.to_string()}), Some(e.to_string()))
                }
            }
        }
        None => match passthrough::passthrough(client, &url, &key, &request).await {
            Ok(resp) => (resp, None),
            Err(e) => (json!({"error": e.to_string()}), Some(e.to_string())),
        },
    };

    // 4. TURN COMPLETE — Record result.
    let assistant_output = extract_assistant_output(&response).unwrap_or_default();
    let (prompt_tokens, completion_tokens) = extract_usage(&response);

    let errors = error_str
        .as_ref()
        .map(|e| vec![e.clone()])
        .unwrap_or_default();
    let complete_event = FocusaEvent::TurnCompleted {
        turn_id: turn_id.clone(),
        harness_name: "openai".into(),
        raw_user_input: Some(user_input.clone()),
        assistant_output: if assistant_output.is_empty() {
            None
        } else {
            Some(assistant_output.clone())
        },
        artifacts_used: vec![],
        errors: errors.clone(),
        prompt_tokens: Some(prompt_tokens),
        completion_tokens: Some(completion_tokens),
    };
    let _ = state
        .command_tx
        .send(Action::EmitEvent {
            event: complete_event,
        })
        .await;

    // Emit error signal if failed (summary max 200 chars).
    if let Some(ref err) = error_str {
        let summary: String = err.chars().take(200).collect();
        let signal = create_signal(SignalKind::Error, summary);
        let _ = state.command_tx.send(Action::IngestSignal { signal }).await;
    }

    // Emit assistant_output signal if success (summary max 200 chars).
    if error_str.is_none() && !assistant_output.is_empty() {
        let summary: String = assistant_output.chars().take(200).collect();
        let signal = create_signal(SignalKind::AssistantOutput, summary);
        let _ = state.command_tx.send(Action::IngestSignal { signal }).await;
    }

    tracing::info!(
        turn_id = %turn_id,
        success = error_str.is_none(),
        output_len = assistant_output.len(),
        "Turn completed"
    );

    // Clear active_turn to prevent stale data.
    {
        let mut focusa = state.focusa.write().await;
        if let Some(ref turn) = focusa.active_turn
            && turn.turn_id == turn_id
        {
            focusa.active_turn.take();
        }
    }

    // 5. UPDATE FRAME CHECKPOINT.
    {
        let frame_id = {
            let focusa = state.focusa.read().await;
            focusa.focus_stack.active_id
        };

        if let Some(fid) = frame_id {
            // Provide meaningful current_state + recent_results per G1-07.
            // Workers (ExtractAsccDelta) will add decisions/constraints/etc.
            let (current_state, recent_results, failures) = if error_str.is_some() {
                let err_msg = error_str.as_deref().unwrap_or("unknown error");
                (
                    format!("Turn {} failed: {}", turn_id, &err_msg[..err_msg.len().min(150)]),
                    None,
                    Some(vec![format!("Turn {} error: {}", turn_id, &err_msg[..err_msg.len().min(150)])]),
                )
            } else {
                let summary: String = assistant_output.chars().take(150).collect();
                (
                    format!("Turn {} ({}+{} tokens): {}", turn_id, prompt_tokens, completion_tokens, summary),
                    Some(vec![format!("Turn {}: {} tokens used", turn_id, prompt_tokens + completion_tokens)]),
                    None,
                )
            };

            let delta = FocusStateDelta {
                current_state: Some(current_state),
                recent_results,
                failures,
                ..Default::default()
            };
            let _ = state
                .command_tx
                .send(Action::UpdateCheckpointDelta {
                    frame_id: fid,
                    turn_id: turn_id.clone(),
                    delta,
                })
                .await;
        }
    }

    if let Some(err) = error_str {
        return Err((StatusCode::BAD_GATEWAY, Json(json!({"error": err}))));
    }

    Ok(Json(response).into_response())
}

/// POST /proxy/v1/messages — Anthropic proxy with turn tracking.
async fn messages_proxy(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<MessagesRequest>,
) -> Result<Response, (StatusCode, Json<Value>)> {
    let auth = messages_auth(&headers).ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "No API key — set FOCUSA_MESSAGES_API_KEY (or FOCUSA_ANTHROPIC_KEY) or pass x-api-key/Authorization header"})),
        )
    })?;

    ensure_session(&state).await;

    let turn_id = uuid::Uuid::now_v7().to_string();

    // 1. TURN START.
    let user_input = anthropic::extract_user_input(&request.messages);

    {
        let mut focusa = state.focusa.write().await;
        focusa.active_turn = Some(ActiveTurn {
            turn_id: turn_id.clone(),
            adapter_id: "messages-proxy".to_string(),
            harness_name: "messages".to_string(),
            started_at: Utc::now(),
            raw_user_input: Some(user_input.clone()),
            assembled_prompt: None,
        });
    }
    tracing::info!(turn_id = %turn_id, harness = "messages", "Turn started");

    let start_event = FocusaEvent::TurnStarted {
        turn_id: turn_id.clone(),
        harness_name: "messages".into(),
        adapter_id: "messages-proxy".into(),
        raw_user_input: Some(user_input.clone()),
    };
    let _ = state
        .command_tx
        .send(Action::EmitEvent { event: start_event })
        .await;

    // Emit user_input signal (summary max 200 chars).
    if !user_input.is_empty() {
        let summary: String = user_input.chars().take(200).collect();
        let signal = create_signal(SignalKind::UserInput, summary);
        let _ = state.command_tx.send(Action::IngestSignal { signal }).await;
    }

    let url = resolve_messages_upstream(&request.model);
    let client = get_client();

    // 1c. PRE-TURN ENRICHMENT — Fast Mem0 + Wiki (50ms each, parallel, §11.7).
    {
        if !user_input.is_empty() {
            let stopwords = ["the","a","an","is","are","was","were","be","to","of","in",
                "for","on","with","at","by","from","as","it","this","that","i","me",
                "we","you","and","or","but","not","so","if"];
            let keywords: Vec<&str> = user_input.split_whitespace()
                .filter(|w| w.len() > 2 && !stopwords.contains(&w.to_lowercase().as_str()))
                .take(5).collect();
            let kw = keywords.join(" ");
            let query_text = if kw.is_empty() {
                if user_input.len() > 100 { &user_input[..100] } else { &user_input }
            } else { &kw };

            let mc = client.clone();
            let mq = query_text.to_string();
            let wq = query_text.to_string();
            let (mem0_r, wiki_r) = tokio::join!(
                async { tokio::time::timeout(std::time::Duration::from_millis(50),
                    mc.post("http://127.0.0.1:8200/v1/search")
                        .json(&serde_json::json!({"query": mq, "namespace": "wirebot_verious", "limit": 5}))
                        .send()).await },
                async { tokio::time::timeout(std::time::Duration::from_millis(50),
                    tokio::process::Command::new("wb")
                        .args(["wiki", "search", &wq, "--format", "json", "--limit", "3"])
                        .output()).await },
            );
            let mut focusa = state.focusa.write().await;
            if let Ok(Ok(resp)) = mem0_r {
                if let Ok(data) = resp.json::<serde_json::Value>().await {
                    if let Some(results) = data.get("results").and_then(|v| v.as_array()) {
                        for (i, mem) in results.iter().enumerate().take(3) {
                            if let Some(text) = mem.get("memory").and_then(|v| v.as_str()) {
                                focusa_core::memory::semantic::upsert(&mut focusa.memory,
                                    format!("mem0.preturn.{}", i), text.to_string(),
                                    focusa_core::types::MemorySource::Mem0);
                            }
                        }
                    }
                }
            }
            if let Ok(Ok(output)) = wiki_r {
                if output.status.success() {
                    if let Ok(wj) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
                        if let Some(pages) = wj.as_array().or_else(|| wj.get("pages").and_then(|v| v.as_array())) {
                            for (i, page) in pages.iter().enumerate().take(2) {
                                let title = page.get("title").and_then(|v| v.as_str()).unwrap_or("");
                                let path = page.get("path").and_then(|v| v.as_str()).unwrap_or("");
                                if !title.is_empty() {
                                    focusa_core::memory::semantic::upsert(&mut focusa.memory,
                                        format!("wiki.preturn.{}", i),
                                        format!("Wiki: {} ({})", title, path),
                                        focusa_core::types::MemorySource::Worker);
                                }
                            }
                        }
                    }
                }
            }
            drop(focusa);
        }
    }

    // 2. PROMPT ASSEMBLY.
    let focusa_state = state.focusa.read().await;
    let result = anthropic::process_request(request.clone(), &focusa_state, &state.config);
    drop(focusa_state);

    // Update active turn with assembled prompt if enhancement occurred.
    if let Some(ref proxy_result) = result {
        let mut focusa = state.focusa.write().await;
        if let Some(ref mut turn) = focusa.active_turn {
            if turn.turn_id == turn_id {
                turn.assembled_prompt = Some(proxy_result.assembly.content.clone());
            }
        }
        drop(focusa);

        // Emit PromptAssembled event per G1-detail-11 §Events.
        let prompt_event = proxy_result.assembly.to_event(Some(turn_id.clone().into()));
        let _ = state.command_tx.send(Action::EmitEvent { event: prompt_event }).await;
    }

    // 3. FORWARD.
    let mut upstream_request = result
        .as_ref()
        .map(|r| r.request.clone())
        .unwrap_or_else(|| request.clone());

    let kimi_mode = is_kimi_model(&upstream_request.model);
    let compat_mode = kimi_mode && proxy_compat_mode_enabled();
    if compat_mode {
        anthropic::sanitize_for_compat(&mut upstream_request);
        tracing::debug!(turn_id = %turn_id, "Applied Kimi Anthropic compatibility sanitizer");
    } else if kimi_mode {
        tracing::debug!(turn_id = %turn_id, "Kimi model in transparent mode; compat sanitizer disabled");
    }

    if is_streaming(&upstream_request.extra) {
        return stream_messages_response(
            state.clone(),
            auth.clone(),
            url,
            upstream_request,
            turn_id,
            user_input,
        )
        .await;
    }

    let (response, error_str) =
        match anthropic::forward_request(get_client(), &url, &auth, &upstream_request).await {
            Ok(resp) => (resp, None),
            Err(e) => {
                let e_str = e.to_string();
                if compat_mode
                    && (e_str.contains("HTTP 400")
                        || e_str.contains("invalid_request")
                        || e_str.contains("tool_use")
                        || e_str.contains("tool_result")
                        || e_str.contains("tool_call_id"))
                {
                    let mut retry_req = upstream_request.clone();
                    anthropic::sanitize_for_compat(&mut retry_req);
                    tracing::warn!(
                        "Kimi anthropic compatibility retry after upstream failure: {}",
                        e_str
                    );
                    match anthropic::forward_request(get_client(), &url, &auth, &retry_req).await {
                        Ok(resp) => (resp, None),
                        Err(e2) => {
                            tracing::error!("Anthropic upstream failed after compat retry: {}", e2);
                            (json!({"error": e2.to_string()}), Some(e2.to_string()))
                        }
                    }
                } else {
                    tracing::error!("Anthropic upstream failed: {}", e_str);
                    (json!({"error": e_str}), Some(e_str))
                }
            }
        };

    // 4. TURN COMPLETE.
    let mut response = response;
    if compat_mode {
        let removed_markers = sanitize_messages_response_for_discord(&mut response);
        if removed_markers > 0 {
            tracing::warn!(
                turn_id = %turn_id,
                removed_markers,
                "Sanitized leaked tool markup from Anthropic JSON response"
            );
        }
    }

    let mut assistant_output = anthropic::extract_assistant_output(&response).unwrap_or_default();

    // Kimi occasionally returns this transient transport sentence as assistant text.
    // Retry once to avoid leaking it to Discord.
    if compat_mode
        && error_str.is_none()
        && assistant_output
            .to_ascii_lowercase()
            .contains("request ended without sending any chunks")
    {
        tracing::warn!("Kimi transient chunk sentence detected; retrying once");
        if let Ok(mut retry_response) =
            anthropic::forward_request(get_client(), &url, &auth, &upstream_request).await
        {
            let removed_markers = sanitize_messages_response_for_discord(&mut retry_response);
            if removed_markers > 0 {
                tracing::warn!(
                    turn_id = %turn_id,
                    removed_markers,
                    "Sanitized leaked tool markup from retry Anthropic JSON response"
                );
            }
            response = retry_response;
            assistant_output = anthropic::extract_assistant_output(&response).unwrap_or_default();
        }
    }

    let (input_tokens, output_tokens) = anthropic::extract_usage(&response);

    let errors = error_str
        .as_ref()
        .map(|e| vec![e.clone()])
        .unwrap_or_default();
    let complete_event = FocusaEvent::TurnCompleted {
        turn_id: turn_id.clone(),
        harness_name: "messages".into(),
        raw_user_input: Some(user_input.clone()),
        assistant_output: if assistant_output.is_empty() {
            None
        } else {
            Some(assistant_output.clone())
        },
        artifacts_used: vec![],
        errors: errors.clone(),
        prompt_tokens: Some(input_tokens),
        completion_tokens: Some(output_tokens),
    };
    let _ = state
        .command_tx
        .send(Action::EmitEvent {
            event: complete_event,
        })
        .await;

    // Emit error or assistant signal (summary max 200 chars).
    if let Some(ref err) = error_str {
        let summary: String = err.chars().take(200).collect();
        let signal = create_signal(SignalKind::Error, summary);
        let _ = state.command_tx.send(Action::IngestSignal { signal }).await;
    } else if !assistant_output.is_empty() {
        let summary: String = assistant_output.chars().take(200).collect();
        let signal = create_signal(SignalKind::AssistantOutput, summary);
        let _ = state.command_tx.send(Action::IngestSignal { signal }).await;
    }

    tracing::info!(
        turn_id = %turn_id,
        success = error_str.is_none(),
        output_len = assistant_output.len(),
        "Turn completed"
    );

    // Clear active_turn to prevent stale data.
    {
        let mut focusa = state.focusa.write().await;
        if let Some(ref turn) = focusa.active_turn
            && turn.turn_id == turn_id
        {
            focusa.active_turn.take();
        }
    }

    // 5. UPDATE FRAME.
    {
        let frame_id = {
            let focusa = state.focusa.read().await;
            focusa.focus_stack.active_id
        };

        if let Some(fid) = frame_id {
            let (current_state, recent_results, failures) = if error_str.is_some() {
                let err_msg = error_str.as_deref().unwrap_or("unknown error");
                (
                    format!("Turn {} failed: {}", turn_id, &err_msg[..err_msg.len().min(150)]),
                    None,
                    Some(vec![format!("Turn {} error: {}", turn_id, &err_msg[..err_msg.len().min(150)])]),
                )
            } else {
                let summary: String = assistant_output.chars().take(150).collect();
                (
                    format!("Turn {} ({}+{} tokens): {}", turn_id, input_tokens, output_tokens, summary),
                    Some(vec![format!("Turn {}: {} tokens used", turn_id, input_tokens + output_tokens)]),
                    None,
                )
            };

            let delta = FocusStateDelta {
                current_state: Some(current_state),
                recent_results,
                failures,
                ..Default::default()
            };
            let _ = state
                .command_tx
                .send(Action::UpdateCheckpointDelta {
                    frame_id: fid,
                    turn_id: turn_id.clone(),
                    delta,
                })
                .await;
        }
    }

    if let Some(err) = error_str {
        return Err((StatusCode::BAD_GATEWAY, Json(json!({"error": err}))));
    }

    Ok(Json(response).into_response())
}

/// POST /proxy/acp — ACP JSON-RPC proxy. — ACP JSON-RPC proxy.
async fn acp_proxy(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    use focusa_core::adapters::acp;

    let bytes = serde_json::to_vec(&body).unwrap_or_default();
    let mut msg = acp::parse_message(&bytes)
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(json!({ "error": e }))))?;

    let s = state.focusa.read().await;
    let session_id = s
        .session
        .as_ref()
        .map(|s| s.session_id.to_string())
        .unwrap_or_default();
    let telemetry = acp::observe_message(&session_id, &msg, acp::AcpDirection::ClientToAgent);
    tracing::debug!(method = ?telemetry.method, direction = ?telemetry.direction, "ACP message observed");

    if let Some(active_id) = s.focus_stack.active_id
        && let Some(frame_record) = s.focus_stack.frames.iter().find(|f| f.id == active_id)
    {
        acp::apply_cognition(&mut msg, &frame_record.focus_state, &s.focus_gate);
    }
    drop(s);

    let upstream =
        std::env::var("FOCUSA_ACP_UPSTREAM").unwrap_or_else(|_| "http://127.0.0.1:4000".into());
    let client = get_client();
    let resp = client
        .post(&upstream)
        .json(&msg)
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_GATEWAY,
                Json(json!({ "error": format!("ACP upstream error: {}", e) })),
            )
        })?;

    let response: Value = resp.json().await.map_err(|e| {
        (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "error": format!("ACP response parse error: {}", e) })),
        )
    })?;

    Ok(Json(response))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/proxy/v1/chat/completions", post(chat_completions))
        .route("/proxy/v1/messages", post(messages_proxy))
        .route("/proxy/acp", post(acp_proxy))
}
