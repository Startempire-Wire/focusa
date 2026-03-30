//! Mode A — Wrap harness CLI with full session capture.
//!
//! Usage: focusa wrap -- <command> [args...]
//!
//! This implementation uses the `script` command to record full terminal sessions,
//! then parses the recording for observability.
//!
//! Responsibilities:
//! 1. Ensures daemon is running (auto-start)
//! 2. Records harness session with `script`
//! 3. Parses recording to extract user input and assistant output
//! 4. Sends parsed data to daemon

use crate::api_client::ApiClient;
use anyhow::{Context, Result};
use chrono::Utc;
use rand::random;
use serde_json::{Value, json};
use shlex::try_quote;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Fire-and-forget POST - doesn't block on daemon response.
async fn fire_and_forget(client: &ApiClient, path: &str, body: Value) {
    let url = format!("{}{}", client.base_url(), path);
    let client = client.http_client().clone();
    let body = body.clone();
    let path = path.to_string();

    tokio::spawn(async move {
        if let Err(e) = client.post(&url).json(&body).send().await {
            eprintln!("[DEBUG] POST {} failed: {}", path, e);
        }
    });
}

/// Fire-and-forget POST using curl.
fn fire_blocking(client: &ApiClient, path: &str, body: Value, timeout_secs: u64) {
    client.post_blocking(path, &body, timeout_secs);
}

/// Check if daemon is running.
async fn is_daemon_running(client: &ApiClient) -> bool {
    client.get("/v1/health").await.is_ok()
}

/// Start daemon as background process using setsid.
async fn start_daemon() -> Result<()> {
    let daemon_path = which::which("focusa-daemon")
        .or_else(|_| {
            let exe = std::env::current_exe()?;
            let dir = exe.parent().unwrap_or(std::path::Path::new("."));
            let candidate = dir.join("focusa-daemon");
            if candidate.exists() {
                Ok(candidate)
            } else {
                Err(anyhow::anyhow!("not found"))
            }
        })
        .or_else(|_| {
            let candidate = std::path::PathBuf::from("/usr/local/bin/focusa-daemon");
            if candidate.exists() {
                Ok(candidate)
            } else {
                Err(anyhow::anyhow!("not found"))
            }
        })?;

    eprintln!("[DEBUG] Starting daemon: {:?}", daemon_path);

    // Use setsid to create new session - no need for exec or shell redirections
    std::process::Command::new("/usr/bin/setsid")
        .arg("-f")
        .arg(&daemon_path)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .context("Failed to spawn daemon")?;

    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    Ok(())
}

/// Parse a terminal recording to extract user input and assistant output.
///
/// Looks for speaker markers like "You:", "Human:", "User:", or "> " to distinguish
/// user input from assistant responses. Content on the same line as a marker
/// (after the colon or "> ") is included in that speaker's content.
fn parse_transcript(transcript: &str) -> (String, String) {
    let mut user_input = String::new();
    let mut assistant_output = String::new();

    let lines: Vec<&str> = transcript.lines().collect();
    let mut current_speaker = String::new();
    let mut current_content = String::new();

    for line in &lines {
        let trimmed = line.trim();

        // Skip empty lines
        if trimmed.is_empty() {
            continue;
        }

        // Extract marker (before colon) and content (after colon)
        let (marker_part, content_after_marker) = if let Some(idx) = trimmed.find(':') {
            (&trimmed[..=idx], Some(&trimmed[idx + 1..]))
        } else {
            ("", None)
        };

        // Check for simple marker first (no colon needed)
        let simple_marker_content = if trimmed.starts_with("> ") && trimmed.len() > 2 {
            Some(&trimmed[2..])
        } else {
            None
        };

        // Detect marker types - order matters for disambiguation
        let is_user_marker = matches!(marker_part, "You:" | "Human:" | "User:");
        let is_assistant_marker =
            marker_part.ends_with(':') && !is_user_marker && !marker_part.is_empty();
        let has_simple_marker = simple_marker_content.is_some();

        if is_user_marker {
            // Flush previous content if any (content before any marker goes to assistant)
            if !current_content.is_empty() {
                // Content is user input only if we had a previous user speaker
                let prev_is_user = !current_speaker.is_empty()
                    && (current_speaker == "You:"
                        || current_speaker == "Human:"
                        || current_speaker == "User:"
                        || current_speaker.starts_with("You:")
                        || current_speaker.starts_with("Human:")
                        || current_speaker.starts_with("User:"));

                if prev_is_user {
                    if !user_input.is_empty() {
                        user_input.push('\n');
                    }
                    user_input.push_str(&current_content);
                } else {
                    if !assistant_output.is_empty() {
                        assistant_output.push('\n');
                    }
                    assistant_output.push_str(&current_content);
                }
            }

            // Start new speaker
            current_speaker = marker_part.to_string();
            current_content = String::new();

            // Add content from same line after marker
            if let Some(content) = content_after_marker {
                let trimmed_content = content.trim_start();
                if !trimmed_content.is_empty() {
                    current_content.push_str(trimmed_content);
                }
            }
        } else if is_assistant_marker {
            // Flush previous content if any (content before any marker goes to assistant)
            if !current_content.is_empty() {
                // Content is user input only if we had a previous user speaker
                let prev_is_user = !current_speaker.is_empty()
                    && (current_speaker == "You:"
                        || current_speaker == "Human:"
                        || current_speaker == "User:"
                        || current_speaker == "> "
                        || current_speaker.starts_with("You:")
                        || current_speaker.starts_with("Human:")
                        || current_speaker.starts_with("User:"));

                if prev_is_user {
                    if !user_input.is_empty() {
                        user_input.push('\n');
                    }
                    user_input.push_str(&current_content);
                } else {
                    if !assistant_output.is_empty() {
                        assistant_output.push('\n');
                    }
                    assistant_output.push_str(&current_content);
                }
            }

            // Start new assistant speaker
            current_speaker = marker_part.to_string();
            current_content = String::new();

            // Add content from same line after marker
            if let Some(content) = content_after_marker {
                let trimmed_content = content.trim_start();
                if !trimmed_content.is_empty() {
                    current_content.push_str(trimmed_content);
                }
            }
        } else if has_simple_marker {
            // Flush previous content if any (content before any marker goes to assistant)
            if !current_content.is_empty() {
                // Content is user input only if we had a previous user speaker
                let prev_is_user = !current_speaker.is_empty()
                    && (current_speaker == "You:"
                        || current_speaker == "Human:"
                        || current_speaker == "User:"
                        || current_speaker.starts_with("You:")
                        || current_speaker.starts_with("Human:")
                        || current_speaker.starts_with("User:"));

                if prev_is_user {
                    if !user_input.is_empty() {
                        user_input.push('\n');
                    }
                    user_input.push_str(&current_content);
                } else {
                    if !assistant_output.is_empty() {
                        assistant_output.push('\n');
                    }
                    assistant_output.push_str(&current_content);
                }
            }

            current_speaker = "> ".to_string();
            current_content = String::new();

            // Add content after "> "
            let simple_content = simple_marker_content.unwrap().trim_start();
            if !simple_content.is_empty() {
                current_content.push_str(simple_content);
            }
        } else {
            // Regular content line - add to current speaker or default to assistant
            if !current_content.is_empty() {
                current_content.push('\n');
            }
            current_content.push_str(trimmed);
        }
    }

    // Handle last speaker - or default to assistant if no markers at all
    if !current_speaker.is_empty() {
        // Simple marker "> " counts as user input
        let is_user = current_speaker.starts_with("You:")
            || current_speaker.starts_with("Human:")
            || current_speaker.starts_with("User:")
            || current_speaker == "> ";

        if is_user {
            if !user_input.is_empty() {
                user_input.push('\n');
            }
            user_input.push_str(&current_content);
        } else {
            if !assistant_output.is_empty() {
                assistant_output.push('\n');
            }
            assistant_output.push_str(&current_content);
        }
    } else if !current_content.is_empty() {
        // No markers at all - default to assistant output
        if !assistant_output.is_empty() {
            assistant_output.push('\n');
        }
        assistant_output.push_str(&current_content);
    }

    (user_input, assistant_output)
}

/// Run harness with full session recording using `script` command.
///
/// Uses `script -q -c "command"` to record full terminal session including TUI.
/// Arguments are properly shell-quoted using shlex to prevent injection.
fn run_with_recording(
    harness_path: &str,
    args: &[String],
    env_vars: &[(&str, &str)],
) -> Result<(i32, String)> {
    // Create temp directory for recording
    let temp_dir = PathBuf::from("/tmp");
    let timestamp = Utc::now().timestamp_millis();
    let session_file = temp_dir.join(format!("focusa-session-{}.txt", timestamp));

    // Properly quote each argument to prevent shell injection
    let harness_args: Vec<String> = args
        .iter()
        .map(|a| {
            try_quote(a)
                .map(|q| q.to_string())
                .unwrap_or_else(|_| a.clone())
        })
        .collect();
    let harness_cmd = format!("{} {}", harness_path, harness_args.join(" "));

    let status = Command::new("script")
        .args(["-q", "-c", &harness_cmd])
        .arg("-a")
        .arg(&session_file)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .envs(env_vars.iter().copied())
        .status()
        .context("Failed to run script command")?;

    // Read the recording (script writes to session_file, not stdout)
    let transcript = if session_file.exists() {
        fs::read_to_string(&session_file)?
    } else {
        String::new()
    };

    // Clean up temp file (log error if fails)
    if let Err(e) = fs::remove_file(&session_file) {
        eprintln!("[DEBUG] Failed to remove session file: {}", e);
    }

    let exit_code = status.code().unwrap_or(1);

    Ok((exit_code, transcript))
}

/// Simple harness runner (non-PTY) for command-line only mode.
/// Returns exit code and combined stdout+stderr.
fn run_simple(
    harness_path: &str,
    args: &[String],
    env_vars: &[(&str, &str)],
) -> Result<(i32, String)> {
    let output = Command::new(harness_path)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .envs(env_vars.iter().copied())
        .output()
        .context("Failed to run harness")?;

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    let combined = if stderr.is_empty() {
        stdout
    } else {
        format!("{}\n{}", stdout, stderr)
    };

    Ok((output.status.code().unwrap_or(1), combined))
}

/// Detect if we're in TUI mode (no prompt argument) or CLI mode (has prompt).
fn detect_mode(args: &[String]) -> (&str, &[String], bool) {
    if args.is_empty() {
        return ("", args, true); // TUI mode
    }

    let first = &args[0];
    if first.starts_with('-') || first.contains(' ') || first.len() > 50 {
        return (first, &args[1..], false); // CLI mode with prompt
    }

    ("", args, true) // Default to TUI
}

/// Run the wrap command with full session capture.
pub async fn run(command: Vec<String>) -> anyhow::Result<()> {
    if command.is_empty() {
        anyhow::bail!("Usage: focusa wrap -- <command> [args...]");
    }

    let client = ApiClient::new();
    let harness_path = &command[0];
    let args: Vec<String> = command[1..].to_vec();

    // Detect mode and extract prompt/args
    let (prompt, remaining_args, is_tui) = detect_mode(&args);
    let harness_name = std::path::Path::new(harness_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    eprintln!("[DEBUG] Mode: {} (TUI={})", harness_name, is_tui);

    // 0. Ensure daemon is running
    if !is_daemon_running(&client).await {
        eprintln!("🚀 Starting Focusa daemon...");
        start_daemon().await?;

        for _ in 0..50 {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            if is_daemon_running(&client).await {
                break;
            }
        }
        eprintln!("✓ Daemon ready");
    }

    // 1. Turn start (fire-and-forget)
    let timestamp_ms = Utc::now().timestamp_millis();
    let random_suffix: u32 = random();
    let turn_id = format!("{:x}{:08x}", timestamp_ms, random_suffix);
    eprintln!("[DEBUG] Turn ID: {}", turn_id);

    fire_and_forget(
        &client,
        "/v1/turn/start",
        json!({
            "turn_id": turn_id,
            "adapter_id": format!("wrap-{}", &turn_id[..8]),
            "harness_name": harness_name,
            "timestamp": Utc::now().to_rfc3339()
        }),
    )
    .await;

    // 2. For CLI mode: try prompt assembly
    let mut final_args = remaining_args.to_vec();

    if !is_tui && !prompt.is_empty() {
        // For CLI mode with pi harness, use --print mode with raw prompt
        // This bypasses Focusa prompt assembly which is meant for TUI mode
        if harness_name == "pi" {
            final_args = vec!["--print".to_string(), prompt.to_string()];
            eprintln!("[DEBUG] Using --print mode for pi");
        } else {
            // For other harnesses, try prompt assembly
            eprintln!("[DEBUG] Assembling prompt for: {} chars", prompt.len());

            if let Ok(resp) = client
                .post(
                    "/v1/prompt/assemble",
                    &json!({
                        "turn_id": turn_id,
                        "raw_user_input": prompt,
                        "format": "string",
                        "budget": null
                    }),
                )
                .await
                && let Some(assembled) = resp.get("assembled_prompt").and_then(|v| v.as_str())
            {
                final_args = vec![assembled.to_string()];
                eprintln!("[DEBUG] Prompt assembled: {} chars", assembled.len());
            }
        }
    }

    // 3. Run the harness with full session capture
    let env_vars = vec![("FOCUSA_MAGIC_DISABLE", "1"), ("FOCUSA_TURN_ID", &turn_id)];

    eprintln!("[DEBUG] Running: {} {}", harness_path, final_args.join(" "));

    let (exit_code, transcript) = if is_tui {
        // TUI mode: use script to record full session
        match run_with_recording(harness_path, &final_args, &env_vars) {
            Ok(result) => result,
            Err(e) => {
                eprintln!("[DEBUG] Recording failed ({}), falling back to simple", e);
                match run_simple(harness_path, &final_args, &env_vars) {
                    Ok(result) => result,
                    Err(e) => {
                        eprintln!("[ERROR] Harness failed: {}", e);
                        (1, String::new())
                    }
                }
            }
        }
    } else {
        // CLI mode: capture stdout/stderr directly
        match run_simple(harness_path, &final_args, &env_vars) {
            Ok(result) => result,
            Err(e) => {
                eprintln!("[ERROR] Harness failed: {}", e);
                (1, String::new())
            }
        }
    };

    eprintln!("[DEBUG] Harness exited with code: {}", exit_code);
    eprintln!("[DEBUG] Transcript length: {} chars", transcript.len());

    // 4. Parse transcript to extract user/assistant content (for TUI observability)
    let (user_input, assistant_output) = if is_tui {
        let parsed = parse_transcript(&transcript);
        eprintln!("[DEBUG] Extracted user_input: {} chars", parsed.0.len());
        eprintln!(
            "[DEBUG] Extracted assistant_output: {} chars",
            parsed.1.len()
        );
        parsed
    } else {
        // CLI mode: use captured output directly (no speaker markers to parse)
        eprintln!("[DEBUG] CLI mode: using captured output directly");
        (String::new(), transcript.clone())
    };

    // 5. Turn complete - send everything to daemon
    let errors = if exit_code != 0 {
        vec![format!("Harness exited with code {}", exit_code)]
    } else {
        vec![]
    };

    // For TUI: use full transcript for assistant_output
    // For CLI: use captured output
    let final_output = if is_tui {
        transcript
    } else {
        assistant_output.clone()
    };

    // raw_user_input: CLI mode uses the prompt, TUI mode uses parsed input (may be empty)
    let final_user_input = if is_tui {
        user_input
    } else {
        prompt.to_string()
    };

    fire_blocking(
        &client,
        "/v1/turn/complete",
        json!({
            "turn_id": turn_id,
            "raw_user_input": if final_user_input.is_empty() { None } else { Some(final_user_input) },
            "assistant_output": final_output,
            "artifacts": [],
            "errors": errors
        }),
        2,
    );

    // Exit with harness exit code
    std::process::exit(exit_code);
}
