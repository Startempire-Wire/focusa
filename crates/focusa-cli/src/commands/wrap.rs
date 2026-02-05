//! Mode A — Wrap harness CLI subprocess.
//!
//! Usage: focusa wrap -- <command> [args...]
//!
//! Focusa starts the harness as a subprocess, sets environment variables
//! to redirect API calls through Focusa's proxy, and mediates I/O.
//!
//! Source: docs/G1-detail-04-proxy-adapter.md

use crate::api_client::ApiClient;
use chrono::Utc;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use uuid::Uuid;

/// Run the wrap command.
pub async fn run(command: Vec<String>) -> anyhow::Result<()> {
    if command.is_empty() {
        anyhow::bail!("Usage: focusa wrap -- <command> [args...]");
    }

    let client = ApiClient::new();
    let adapter_id = format!("cli-wrap-{}", Uuid::now_v7());
    let harness_name = command[0].clone();

    println!("🔄 Focusa wrapping: {}", command.join(" "));

    // Detect harness type and set appropriate env vars.
    let mut env_vars = detect_harness_env(&harness_name);

    // Get Focusa API URL for proxy redirection.
    let focusa_url = std::env::var("FOCUSA_API_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8787".to_string());

    // Set proxy URLs based on harness type.
    let proxy_base = format!("{}/proxy", focusa_url);
    env_vars.push(("ANTHROPIC_BASE_URL".to_string(), proxy_base.clone()));
    env_vars.push(("OPENAI_BASE_URL".to_string(), format!("{}/v1", proxy_base)));

    // Pass through API keys from environment.
    if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
        env_vars.push(("ANTHROPIC_API_KEY".to_string(), key));
    }
    if let Ok(key) = std::env::var("OPENAI_API_KEY") {
        env_vars.push(("OPENAI_API_KEY".to_string(), key));
    }

    // Start subprocess.
    let mut child = Command::new(&command[0])
        .args(&command[1..])
        .envs(env_vars)
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let turn_id = Uuid::now_v7().to_string();

    // Notify daemon of turn start.
    let _ = client.post("/v1/turn/start", &serde_json::json!({
        "turn_id": turn_id,
        "adapter_id": adapter_id,
        "harness_name": harness_name,
        "timestamp": Utc::now().to_rfc3339()
    })).await;

    // Stream stdout.
    let stdout = child.stdout.take().expect("stdout");
    let stderr = child.stderr.take().expect("stderr");

    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    let mut output_buffer = String::new();

    // Process output streams.
    loop {
        tokio::select! {
            line = stdout_reader.next_line() => {
                match line {
                    Ok(Some(text)) => {
                        println!("{}", text);
                        output_buffer.push_str(&text);
                        output_buffer.push('\n');
                    }
                    Ok(None) => break,
                    Err(e) => {
                        eprintln!("stdout error: {}", e);
                        break;
                    }
                }
            }
            line = stderr_reader.next_line() => {
                match line {
                    Ok(Some(text)) => {
                        eprintln!("{}", text);
                    }
                    Ok(None) => {}
                    Err(e) => {
                        eprintln!("stderr error: {}", e);
                    }
                }
            }
        }
    }

    // Wait for process to exit.
    let status = child.wait().await?;

    // Notify daemon of turn completion.
    let errors: Vec<String> = if status.success() {
        vec![]
    } else {
        vec![format!("Process exited with: {}", status)]
    };

    let _ = client.post("/v1/turn/complete", &serde_json::json!({
        "turn_id": turn_id,
        "assistant_output": output_buffer,
        "artifacts": [],
        "errors": errors
    })).await;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}

/// Detect harness type and return appropriate environment variables.
fn detect_harness_env(harness: &str) -> Vec<(String, String)> {
    let mut vars = vec![];

    // Harness-specific configuration.
    match harness {
        "pi" | "claude" => {
            // Pi/Claude Code use Anthropic API.
            // Proxy is set via ANTHROPIC_BASE_URL.
        }
        "letta" => {
            // Letta may use various backends.
            vars.push(("LETTA_FOCUSA_ENABLED".to_string(), "1".to_string()));
        }
        "codex" => {
            // Codex CLI uses OpenAI.
            // Proxy is set via OPENAI_BASE_URL.
        }
        _ => {
            // Generic — set both proxy URLs.
        }
    }

    vars
}
