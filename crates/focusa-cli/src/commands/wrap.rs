//! Mode A — Wrap harness CLI subprocess.
//!
//! Usage: focusa wrap -- <command> [args...]
//!
//! Per spec (docs/G1-detail-04-proxy-adapter.md):
//! 1. Parse command to find user prompt
//! 2. Call daemon to assemble enhanced prompt
//! 3. Run harness with enhanced prompt
//! 4. Emit signals to Focus Gate
//! 5. Provide transcript to ASCC
//! 6. Externalize large blobs to ECS

use crate::api_client::ApiClient;
use chrono::Utc;
use serde_json::{json, Value};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use uuid::Uuid;

/// Harness configuration — how each harness takes prompts.
struct HarnessConfig {
    name: &'static str,
    /// How to find the prompt in args.
    prompt_arg: PromptArg,
    /// Environment variables to set.
    env_vars: &'static [(&'static str, &'static str)],
    /// API provider (for HTTP proxy fallback).
    provider: Provider,
}

enum PromptArg {
    /// Prompt is a positional argument at given index.
    Positional(usize),
    /// Prompt follows a flag (e.g., -p "prompt").
    Flag(&'static str),
    /// Prompt is read from stdin.
    Stdin,
    /// Unknown — use HTTP proxy only.
    Unknown,
}

enum Provider {
    Anthropic,
    OpenAI,
    Unknown,
}

/// Detect harness type from command name.
fn detect_harness(cmd: &str) -> HarnessConfig {
    let base = std::path::Path::new(cmd)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(cmd);

    match base {
        "pi" => HarnessConfig {
            name: "pi",
            prompt_arg: PromptArg::Positional(0), // pi "prompt"
            env_vars: &[],
            provider: Provider::Anthropic,
        },
        "claude" => HarnessConfig {
            name: "claude",
            prompt_arg: PromptArg::Positional(0),
            env_vars: &[],
            provider: Provider::Anthropic,
        },
        "aider" => HarnessConfig {
            name: "aider",
            prompt_arg: PromptArg::Stdin, // aider reads from stdin
            env_vars: &[],
            provider: Provider::OpenAI,
        },
        "codex" => HarnessConfig {
            name: "codex",
            prompt_arg: PromptArg::Positional(0),
            env_vars: &[],
            provider: Provider::OpenAI,
        },
        "letta" => HarnessConfig {
            name: "letta",
            prompt_arg: PromptArg::Flag("-m"), // letta run -m "message"
            env_vars: &[("LETTA_FOCUSA_ENABLED", "1")],
            provider: Provider::OpenAI,
        },
        _ => HarnessConfig {
            name: "generic",
            prompt_arg: PromptArg::Unknown,
            env_vars: &[],
            provider: Provider::Unknown,
        },
    }
}

/// Extract prompt from command args based on harness config.
fn extract_prompt(args: &[String], config: &HarnessConfig) -> Option<(usize, String)> {
    match config.prompt_arg {
        PromptArg::Positional(idx) => {
            args.get(idx).map(|s| (idx, s.clone()))
        }
        PromptArg::Flag(flag) => {
            for (i, arg) in args.iter().enumerate() {
                if arg == flag && i + 1 < args.len() {
                    return Some((i + 1, args[i + 1].clone()));
                }
            }
            None
        }
        PromptArg::Stdin | PromptArg::Unknown => None,
    }
}

/// Run the wrap command.
pub async fn run(command: Vec<String>) -> anyhow::Result<()> {
    if command.is_empty() {
        anyhow::bail!("Usage: focusa wrap -- <command> [args...]");
    }

    let client = ApiClient::new();
    
    // Get Focusa API URL.
    let focusa_url = std::env::var("FOCUSA_API_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8787".into());

    // ═══════════════════════════════════════════════════════════════════════════
    // 0. ENSURE DAEMON IS RUNNING (magic auto-start)
    // ═══════════════════════════════════════════════════════════════════════════
    
    if !is_daemon_running(&client).await {
        eprintln!("🚀 Starting Focusa daemon...");
        start_daemon(&focusa_url).await?;
        
        // Wait for daemon to be ready (max 5s).
        for _ in 0..50 {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            if is_daemon_running(&client).await {
                break;
            }
        }
        
        if !is_daemon_running(&client).await {
            anyhow::bail!("Failed to start Focusa daemon");
        }
        eprintln!("✓ Daemon ready");
    }

    let adapter_id = format!("wrap-{}", &Uuid::now_v7().to_string()[..8]);
    let harness = detect_harness(&command[0]);
    let turn_id = Uuid::now_v7().to_string();

    // ═══════════════════════════════════════════════════════════════════════════
    // 1. TURN START — notify daemon
    // ═══════════════════════════════════════════════════════════════════════════
    
    let _ = client.post("/v1/turn/start", &json!({
        "turn_id": turn_id,
        "adapter_id": adapter_id,
        "harness_name": harness.name,
        "timestamp": Utc::now().to_rfc3339()
    })).await;

    // ═══════════════════════════════════════════════════════════════════════════
    // 2. EXTRACT & ENHANCE PROMPT (Mode A core)
    // ═══════════════════════════════════════════════════════════════════════════
    
    let args: Vec<String> = command[1..].to_vec();
    let mut final_args = args.clone();
    let mut user_input = String::new();

    if let Some((idx, prompt)) = extract_prompt(&args, &harness) {
        user_input = prompt.clone();

        // Emit signal: user input received.
        let _ = client.post("/v1/gate/signal", &json!({
            "kind": "user_input_received",
            "summary": format!("User input: {} chars", prompt.len()),
            "frame_context": null
        })).await;

        // Call daemon to assemble enhanced prompt.
        let assemble_resp = client.post("/v1/prompt/assemble", &json!({
            "turn_id": turn_id,
            "raw_user_input": prompt,
            "harness_context": null,
            "max_tokens_budget": null
        })).await;

        if let Ok(resp) = assemble_resp {
            // Extract assembled prompt.
            if let Some(assembled) = extract_assembled_prompt(&resp) {
                // Replace prompt in args with enhanced version.
                final_args[idx] = assembled;
                tracing::debug!("Prompt enhanced via Expression Engine");
            }
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // 3. SET UP HTTP PROXY (Mode B fallback)
    // ═══════════════════════════════════════════════════════════════════════════
    
    let mut env_vars: Vec<(String, String)> = harness.env_vars
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

    // Set proxy URLs based on provider.
    let proxy_base = format!("{}/proxy", focusa_url);
    match harness.provider {
        Provider::Anthropic => {
            env_vars.push(("ANTHROPIC_BASE_URL".into(), proxy_base));
        }
        Provider::OpenAI => {
            env_vars.push(("OPENAI_BASE_URL".into(), format!("{}/v1", proxy_base)));
        }
        Provider::Unknown => {
            // Set both.
            env_vars.push(("ANTHROPIC_BASE_URL".into(), proxy_base.clone()));
            env_vars.push(("OPENAI_BASE_URL".into(), format!("{}/v1", proxy_base)));
        }
    }

    // Pass through API keys.
    for key in ["ANTHROPIC_API_KEY", "OPENAI_API_KEY"] {
        if let Ok(val) = std::env::var(key) {
            env_vars.push((key.into(), val));
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // 4. RUN SUBPROCESS
    // ═══════════════════════════════════════════════════════════════════════════
    
    let mut child = Command::new(&command[0])
        .args(&final_args)
        .envs(env_vars)
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().expect("stdout");
    let stderr = child.stderr.take().expect("stderr");

    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    let mut output_buffer = String::new();
    let mut stdout_done = false;
    let mut stderr_done = false;

    // Stream output from both stdout and stderr until both close.
    loop {
        tokio::select! {
            line = stdout_reader.next_line(), if !stdout_done => {
                match line {
                    Ok(Some(text)) => {
                        println!("{}", text);
                        output_buffer.push_str(&text);
                        output_buffer.push('\n');

                        // Check for tool output patterns (best effort).
                        if text.contains("```") || text.starts_with("Tool:") {
                            let _ = client.post("/v1/gate/signal", &json!({
                                "kind": "tool_output_captured",
                                "summary": format!("Tool output detected: {} chars", text.len()),
                                "frame_context": null
                            })).await;
                        }
                    }
                    Ok(None) => stdout_done = true,
                    Err(e) => {
                        eprintln!("stdout error: {}", e);
                        stdout_done = true;
                    }
                }
            }
            line = stderr_reader.next_line(), if !stderr_done => {
                match line {
                    Ok(Some(text)) => {
                        eprintln!("{}", text);

                        // Emit error signal.
                        let _ = client.post("/v1/gate/signal", &json!({
                            "kind": "error_observed",
                            "summary": format!("stderr: {}", truncate(&text, 100)),
                            "frame_context": null
                        })).await;
                    }
                    Ok(None) => stderr_done = true,
                    Err(_) => stderr_done = true,
                }
            }
            else => break, // Both streams closed
        }
    }

    let status = child.wait().await?;

    // ═══════════════════════════════════════════════════════════════════════════
    // 5. EXTERNALIZE LARGE BLOBS TO ECS
    // ═══════════════════════════════════════════════════════════════════════════
    
    let mut artifacts: Vec<Value> = vec![];
    
    // If output exceeds 8KB, externalize to ECS.
    if output_buffer.len() > 8192 {
        let ecs_resp = client.post("/v1/ecs/store", &json!({
            "kind": "transcript",
            "label": format!("turn-{}-output", &turn_id[..8]),
            "content": output_buffer,
            "mime_type": "text/plain"
        })).await;

        if let Ok(resp) = ecs_resp
            && let Some(handle_id) = resp.get("handle_id").and_then(|v| v.as_str()) {
                artifacts.push(json!({"handle_id": handle_id, "kind": "transcript"}));
                tracing::debug!("Output externalized to ECS: {}", handle_id);
            }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // 6. TURN COMPLETE — send transcript to ASCC
    // ═══════════════════════════════════════════════════════════════════════════
    
    let errors: Vec<String> = if status.success() {
        vec![]
    } else {
        vec![format!("Exit code: {}", status.code().unwrap_or(-1))]
    };

    // Truncate output for ASCC if not externalized.
    let transcript = if output_buffer.len() <= 8192 {
        output_buffer.clone()
    } else {
        format!("[externalized to ECS — {} bytes]", output_buffer.len())
    };

    let _ = client.post("/v1/turn/complete", &json!({
        "turn_id": turn_id,
        "assistant_output": transcript,
        "artifacts": artifacts,
        "errors": errors
    })).await;

    // Update focus state with turn summary (ASCC).
    if !user_input.is_empty() {
        let _ = client.post("/v1/focus/update", &json!({
            "delta": {
                "recent_results": [truncate(&output_buffer, 500)]
            }
        })).await;
    }

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}

/// Extract assembled prompt from response.
///
/// The assembled prompt contains Focusa context (Focus State, rules, handles)
/// plus the original user input. For messages format, this is in the "system"
/// role. The "user" role just contains the raw input (no enhancement).
fn extract_assembled_prompt(resp: &Value) -> Option<String> {
    // Try plain string format first (preferred for Mode A).
    if let Some(plain) = resp.get("assembled_prompt").and_then(|v| v.as_str()) {
        return Some(plain.to_string());
    }

    // Try messages format — extract system message (contains full assembled prompt).
    if let Some(messages) = resp.get("assembled_prompt").and_then(|v| v.as_array()) {
        for msg in messages {
            if msg.get("role").and_then(|r| r.as_str()) == Some("system") {
                return msg.get("content").and_then(|c| c.as_str()).map(String::from);
            }
        }
    }

    None
}

/// Truncate string to max length (UTF-8 safe).
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        // Find a safe UTF-8 boundary at or before max.
        let boundary = s
            .char_indices()
            .take_while(|(i, _)| *i < max)
            .last()
            .map(|(i, c)| i + c.len_utf8())
            .unwrap_or(0);
        format!("{}…", &s[..boundary])
    }
}

/// Check if daemon is running by hitting health endpoint.
async fn is_daemon_running(client: &ApiClient) -> bool {
    client.get("/v1/health").await.is_ok()
}

/// Start daemon as background process.
async fn start_daemon(focusa_url: &str) -> anyhow::Result<()> {
    // Parse bind address from URL.
    let bind = focusa_url
        .strip_prefix("http://")
        .unwrap_or("127.0.0.1:8787");

    // Find daemon binary — check common locations.
    let daemon_path = find_daemon_binary()?;

    // Start daemon in background.
    let mut cmd = std::process::Command::new(&daemon_path);
    cmd.env("FOCUSA_BIND", bind);
    
    // Pass through API keys.
    for key in ["ANTHROPIC_API_KEY", "OPENAI_API_KEY", "FOCUSA_ANTHROPIC_KEY", "FOCUSA_API_KEY"] {
        if let Ok(val) = std::env::var(key) {
            cmd.env(key, val);
        }
    }

    // Redirect output to avoid cluttering terminal.
    cmd.stdout(std::process::Stdio::null());
    cmd.stderr(std::process::Stdio::null());

    cmd.spawn()?;
    Ok(())
}

/// Find the daemon binary.
fn find_daemon_binary() -> anyhow::Result<std::path::PathBuf> {
    // Check if focusa-daemon is in PATH.
    if let Ok(path) = which::which("focusa-daemon") {
        return Ok(path);
    }

    // Check relative to current executable.
    if let Ok(exe) = std::env::current_exe() {
        let dir = exe.parent().unwrap_or(std::path::Path::new("."));
        let candidate = dir.join("focusa-daemon");
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    // Check common dev locations.
    for path in [
        "./target/release/focusa-daemon",
        "./target/debug/focusa-daemon",
        "/usr/local/bin/focusa-daemon",
    ] {
        let p = std::path::PathBuf::from(path);
        if p.exists() {
            return Ok(p);
        }
    }

    anyhow::bail!("Could not find focusa-daemon binary. Install it or add to PATH.")
}
