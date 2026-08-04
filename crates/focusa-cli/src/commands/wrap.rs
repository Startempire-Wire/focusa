//! Wrap harnesses with semantic events as the primary interactive Pi surface.
//!
//! Usage: focusa wrap -- <command> [args...]
//!
//! Interactive Pi preserves its native terminal and emits typed events through
//! the Focusa extension. Raw PTY capture is explicit, redacted, bounded to 8 MiB,
//! externalized to ECS, and never inlined as assistant output.

use crate::api_client::ApiClient;
use anyhow::{Context, Result};
use chrono::Utc;
use rand::random;
use serde_json::{Value, json};
use shlex::try_quote;
use std::fs;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

static WRAP_DEBUG_ENABLED: AtomicBool = AtomicBool::new(false);

fn wrap_debug_enabled() -> bool {
    WRAP_DEBUG_ENABLED.load(Ordering::Relaxed)
}

macro_rules! debugln {
    ($($arg:tt)*) => {
        if wrap_debug_enabled() {
            eprintln!($($arg)*);
        }
    };
}

/// Fire-and-forget POST - doesn't block on daemon response.
async fn fire_and_forget(client: &ApiClient, path: &str, body: Value) {
    let url = format!("{}{}", client.base_url(), path);
    let client = client.http_client().clone();
    let body = body.clone();
    let path = path.to_string();

    tokio::spawn(async move {
        if let Err(e) = client.post(&url).json(&body).send().await {
            debugln!("[DEBUG] POST {} failed: {}", path, e);
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

    debugln!("[DEBUG] Starting daemon: {:?}", daemon_path);

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
            if let Some(simple_content) = simple_marker_content {
                let simple_content = simple_content.trim_start();
                if !simple_content.is_empty() {
                    current_content.push_str(simple_content);
                }
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

/// Run harness with bounded session recording using `script` command.
///
/// The recorder is monitored while the harness runs. It is terminated when the
/// bounded capture budget is reached, preventing runaway PTY files and memory.
/// Arguments are properly shell-quoted using shlex to prevent injection.
const MAX_RECORDING_BYTES: u64 = 8 * 1024 * 1024;
const STALE_RECORDING_MIN_AGE_SECS: u64 = 60 * 60;

fn cleanup_stale_recordings(temp_dir: &std::path::Path) {
    let Ok(entries) = fs::read_dir(temp_dir) else {
        return;
    };
    let mode = std::env::var("FOCUSA_PTY_SCAVENGE_MODE").unwrap_or_else(|_| "apply".to_string());
    let mut actions = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        let Some(rest) = name.strip_prefix("focusa-session-") else {
            continue;
        };
        let Some(pid_text) = rest.split('-').next() else {
            continue;
        };
        let Ok(pid) = pid_text.parse::<u32>() else {
            continue;
        };
        if std::path::Path::new(&format!("/proc/{pid}")).exists() {
            continue;
        }
        let age_secs = fs::metadata(&path)
            .and_then(|metadata| metadata.modified())
            .ok()
            .and_then(|modified| modified.elapsed().ok())
            .map(|age| age.as_secs())
            .unwrap_or(0);
        if age_secs < STALE_RECORDING_MIN_AGE_SECS {
            continue;
        }
        let status = if mode == "dry-run" {
            "would_remove".to_string()
        } else {
            match fs::remove_file(&path) {
                Ok(()) => "removed".to_string(),
                Err(error) => {
                    eprintln!(
                        "[WARN] Failed to remove stale PTY recording {}: {}",
                        path.display(),
                        error
                    );
                    format!("remove_failed:{error}")
                }
            }
        };
        actions.push(json!({
            "path": path,
            "pid": pid,
            "age_secs": age_secs,
            "status": status,
        }));
    }
    if actions.is_empty() {
        return;
    }
    let data_root = std::env::var("FOCUSA_DATA_DIR")
        .map(PathBuf::from)
        .or_else(|_| std::env::var("HOME").map(|home| PathBuf::from(home).join(".focusa")))
        .unwrap_or_else(|_| PathBuf::from("/tmp/focusa"));
    let receipt_dir = data_root.join("receipts");
    if fs::create_dir_all(&receipt_dir).is_ok() {
        let timestamp = Utc::now().timestamp_millis();
        let receipt_path = receipt_dir.join(format!("pty-scavenge-{timestamp}.json"));
        let temporary = receipt_dir.join(format!(".pty-scavenge-{timestamp}.tmp"));
        let receipt = json!({
            "schema": "focusa.pty_scavenge_receipt.v1",
            "mode": mode,
            "recorded_at": Utc::now().to_rfc3339(),
            "actions": actions,
        });
        if fs::write(&temporary, format!("{}\n", receipt)).is_ok() {
            let _ = fs::rename(temporary, receipt_path);
        }
    }
}
fn redact_diagnostic_transcript(transcript: &str) -> String {
    let mut redacted = transcript.to_string();
    for (key, value) in std::env::vars() {
        let upper = key.to_ascii_uppercase();
        if value.len() >= 4
            && ["TOKEN", "SECRET", "PASSWORD", "API_KEY", "AUTH", "COOKIE"]
                .iter()
                .any(|marker| upper.contains(marker))
        {
            redacted = redacted.replace(&value, "[REDACTED]");
        }
    }
    redacted
        .lines()
        .map(|line| {
            let lower = line.to_ascii_lowercase();
            if [
                "authorization:",
                "api_key=",
                "apikey=",
                "password=",
                "secret=",
            ]
            .iter()
            .any(|marker| lower.contains(marker))
            {
                "[REDACTED LINE]"
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

async fn store_diagnostic_artifacts(client: &ApiClient, transcript: &str) -> Vec<String> {
    const ECS_CHUNK_BYTES: usize = 512 * 1024;
    let content = redact_diagnostic_transcript(transcript);
    let timestamp = Utc::now().timestamp_millis();
    let mut handles = Vec::new();
    for (index, chunk) in content
        .as_bytes()
        .chunks(ECS_CHUNK_BYTES)
        .take(16)
        .enumerate()
    {
        let body = json!({
            "kind": "text",
            "label": format!("bounded-pty-diagnostic-{timestamp}-part-{:02}", index + 1),
            "content": String::from_utf8_lossy(chunk),
        });
        match client.post("/v1/ecs/store", &body).await {
            Ok(response) => {
                if let Some(handle) = response.get("id").and_then(Value::as_str) {
                    handles.push(handle.to_string());
                }
            }
            Err(error) => eprintln!(
                "[WARN] ECS diagnostic chunk {} rejected: {}",
                index + 1,
                error
            ),
        }
    }
    handles
}

fn run_with_recording(
    harness_path: &str,
    args: &[String],
    env_vars: &[(&str, &str)],
) -> Result<(i32, String)> {
    // Create temp directory for recording
    let temp_dir = PathBuf::from("/tmp");
    cleanup_stale_recordings(&temp_dir);
    let timestamp = Utc::now().timestamp_millis();
    let session_file = temp_dir.join(format!(
        "focusa-session-{}-{}.txt",
        std::process::id(),
        timestamp
    ));

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

    let mut child: Child = Command::new("script")
        .args(["-q", "-c", &harness_cmd])
        .arg("-a")
        .arg(&session_file)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .envs(env_vars.iter().copied())
        .spawn()
        .context("Failed to run script command")?;

    let mut capped = false;
    loop {
        if child
            .try_wait()
            .context("Failed waiting for script command")?
            .is_some()
        {
            break;
        }
        if fs::metadata(&session_file)
            .map(|m| m.len() > MAX_RECORDING_BYTES)
            .unwrap_or(false)
        {
            capped = true;
            eprintln!(
                "[WARN] PTY recording exceeded {} bytes; stopping capture",
                MAX_RECORDING_BYTES
            );
            child
                .kill()
                .context("Failed stopping oversized script recording")?;
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }
    let status = child.wait().context("Failed to reap script command")?;

    let transcript = if session_file.exists() {
        let bytes = fs::read(&session_file)?;
        let limit = bytes.len().min(MAX_RECORDING_BYTES as usize);
        String::from_utf8_lossy(&bytes[..limit]).into_owned()
    } else {
        String::new()
    };

    if let Err(e) = fs::remove_file(&session_file) {
        debugln!("[DEBUG] Failed to remove session file: {}", e);
    }

    let exit_code = if capped {
        124
    } else {
        status.code().unwrap_or(1)
    };
    Ok((exit_code, transcript))
}

/// Run an interactive harness with its native terminal semantics.
/// No PTY transcript is collected; Pi's semantic extension/RPC surfaces own events.
fn run_interactive(harness_path: &str, args: &[String], env_vars: &[(&str, &str)]) -> Result<i32> {
    let status = Command::new(harness_path)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .envs(env_vars.iter().copied())
        .status()
        .context("Failed to run interactive harness")?;
    Ok(status.code().unwrap_or(1))
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

fn semantic_harness_failure(transcript: &str) -> Option<String> {
    let trimmed = transcript.trim();
    if trimmed.is_empty() {
        return Some("harness exited successfully without semantic output".to_string());
    }
    if trimmed
        .to_ascii_lowercase()
        .contains("no models match pattern")
    {
        return Some("harness model selector matched no available model".to_string());
    }
    None
}

/// Run the wrap command with full session capture.
pub async fn run(command: Vec<String>, verbose: bool) -> anyhow::Result<()> {
    WRAP_DEBUG_ENABLED.store(verbose, Ordering::Relaxed);
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

    debugln!("[DEBUG] Mode: {} (TUI={})", harness_name, is_tui);

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
    debugln!("[DEBUG] Turn ID: {}", turn_id);

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
            debugln!("[DEBUG] Using --print mode for pi");
        } else {
            // For other harnesses, try prompt assembly
            debugln!("[DEBUG] Assembling prompt for: {} chars", prompt.len());

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
                debugln!("[DEBUG] Prompt assembled: {} chars", assembled.len());
            }
        }
    }

    // 3. Run the harness with full session capture
    let env_vars = vec![("FOCUSA_MAGIC_DISABLE", "1"), ("FOCUSA_TURN_ID", &turn_id)];

    debugln!("[DEBUG] Running: {} {}", harness_path, final_args.join(" "));

    let (exit_code, transcript) = if is_tui && harness_name == "pi" {
        // Interactive Pi: preserve native TTY behavior; semantic events come from
        // the Focusa extension/RPC stream rather than ANSI screen scraping.
        match run_interactive(harness_path, &final_args, &env_vars) {
            Ok(code) => (code, String::new()),
            Err(e) => {
                eprintln!("[ERROR] Interactive Pi harness failed: {}", e);
                (1, String::new())
            }
        }
    } else if is_tui && std::env::var("FOCUSA_RAW_PTY_CAPTURE").as_deref() == Ok("1") {
        // Non-Pi forensic capture is explicit opt-in and bounded to 8 MiB.
        match run_with_recording(harness_path, &final_args, &env_vars) {
            Ok(result) => result,
            Err(e) => {
                debugln!("[DEBUG] Recording failed ({}), falling back to simple", e);
                match run_simple(harness_path, &final_args, &env_vars) {
                    Ok(result) => result,
                    Err(e) => {
                        eprintln!("[ERROR] Harness failed: {}", e);
                        (1, String::new())
                    }
                }
            }
        }
    } else if is_tui {
        // Native interactive mode: preserve the terminal and collect no raw PTY.
        match run_interactive(harness_path, &final_args, &env_vars) {
            Ok(code) => (code, String::new()),
            Err(e) => {
                eprintln!("[ERROR] Interactive harness failed: {}", e);
                (1, String::new())
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

    debugln!("[DEBUG] Harness exited with code: {}", exit_code);
    debugln!("[DEBUG] Transcript length: {} chars", transcript.len());
    let semantic_output_expected =
        !is_tui || std::env::var("FOCUSA_RAW_PTY_CAPTURE").as_deref() == Ok("1");
    let semantic_failure = (exit_code == 0 && semantic_output_expected)
        .then(|| semantic_harness_failure(&transcript))
        .flatten();
    let effective_exit_code = if semantic_failure.is_some() {
        65
    } else {
        exit_code
    };
    if let Some(failure) = semantic_failure.as_deref() {
        eprintln!("[ERROR] Semantic harness failure: {failure}");
    }

    // 4. Parse transcript to extract user/assistant content (for TUI observability)
    let (user_input, assistant_output) = if is_tui {
        let parsed = parse_transcript(&transcript);
        debugln!("[DEBUG] Extracted user_input: {} chars", parsed.0.len());
        debugln!(
            "[DEBUG] Extracted assistant_output: {} chars",
            parsed.1.len()
        );
        parsed
    } else {
        // CLI mode: use captured output directly (no speaker markers to parse)
        debugln!("[DEBUG] CLI mode: using captured output directly");
        (String::new(), transcript.clone())
    };

    // 5. Turn complete - send bounded semantic output/handles to daemon.
    let raw_pty_capture = is_tui && std::env::var("FOCUSA_RAW_PTY_CAPTURE").as_deref() == Ok("1");
    let diagnostic_handles = if raw_pty_capture {
        store_diagnostic_artifacts(&client, &transcript).await
    } else {
        Vec::new()
    };
    let errors = if let Some(failure) = semantic_failure {
        vec![failure]
    } else if exit_code != 0 {
        vec![format!("Harness exited with code {}", exit_code)]
    } else {
        vec![]
    };

    // Raw terminal bytes are never inlined. Explicit forensic capture is
    // redacted and externalized to ECS, leaving only a stable handle.
    let final_output = if raw_pty_capture {
        if diagnostic_handles.is_empty() {
            "Bounded PTY diagnostic was not persisted; raw content was discarded".to_string()
        } else {
            format!(
                "Bounded PTY diagnostic stored as {} ECS handle(s): {}",
                diagnostic_handles.len(),
                diagnostic_handles.join(",")
            )
        }
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
            "artifacts": diagnostic_handles.iter().map(|handle| json!({"handle_id": handle, "kind": "bounded_pty_diagnostic"})).collect::<Vec<_>>(),
            "errors": errors
        }),
        2,
    );

    // Exit with the effective code so empty/invalid-model output cannot become false success.
    std::process::exit(effective_exit_code);
}

#[cfg(test)]
mod tests {
    use super::semantic_harness_failure;

    #[test]
    fn semantic_harness_failure_rejects_empty_output() {
        assert!(semantic_harness_failure("  \n").is_some());
    }

    #[test]
    fn semantic_harness_failure_rejects_invalid_model_warning() {
        assert!(
            semantic_harness_failure(
                "Warning: No models match pattern \"ovh-ai-llama-cpp/ovh-local-coder\""
            )
            .is_some()
        );
    }

    #[test]
    fn semantic_harness_failure_accepts_valid_short_output() {
        assert!(semantic_harness_failure("WORKER_MODEL_OK").is_none());
    }
}
