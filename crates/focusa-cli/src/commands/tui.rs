//! Focusa CLI: `focusa tui` — proxy to the focusa-tui binary with headless self-test.

use anyhow::{Context, Result};
use clap::Args;
use std::process::Command;

#[derive(Args)]
pub struct TuiArgs {
    /// Override the Focusa API URL for the TUI (defaults to FOCUSA_API_URL then http://127.0.0.1:8787).
    #[arg(long)]
    pub api_url: Option<String>,
    /// Run the headless TUI self-test instead of launching the interactive TUI.
    /// Prints initial daemon, health, focus-stack, workpoint, and lineage snapshot.
    #[arg(long)]
    pub headless_self_test: bool,
}

pub async fn run(args: TuiArgs, _json: bool) -> Result<()> {
    let api = args
        .api_url
        .clone()
        .or_else(|| std::env::var("FOCUSA_API_URL").ok())
        .unwrap_or_else(|| "http://127.0.0.1:8787".into());

    if args.headless_self_test {
        return run_headless_self_test(&api).await;
    }

    let bin = locate_tui_binary().context("focusa-tui binary not found in PATH or target/*/build")?;
    let status = Command::new(&bin)
        .env("FOCUSA_API_URL", &api)
        .status()
        .with_context(|| format!("failed to launch {}", bin.display()))?;
    if !status.success() {
        anyhow::bail!("focusa-tui exited with status {:?}", status.code());
    }
    Ok(())
}

fn locate_tui_binary() -> Option<std::path::PathBuf> {
    if let Some(path) = std::env::var_os("FOCUSA_TUI_BIN") {
        let p = std::path::PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
    }
    for name in ["focusa-tui"] {
        if let Ok(found) = which(name) {
            return Some(found);
        }
    }
    let cwd = std::env::current_dir().ok()?;
    for profile in ["release", "debug"] {
        let candidate = cwd.join("target").join(profile).join("focusa-tui");
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

fn which(name: &str) -> Result<std::path::PathBuf, ()> {
    let path = std::env::var_os("PATH").ok_or(())?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    Err(())
}

async fn run_headless_self_test(api: &str) -> Result<()> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .context("reqwest client init failed")?;

    async fn fetch(client: &reqwest::Client, api: &str, path: &str) -> serde_json::Value {
        let url = format!("{}{}", api.trim_end_matches('/'), path);
        match client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => resp
                .json::<serde_json::Value>()
                .await
                .unwrap_or_else(|_| serde_json::json!({"raw_error": "decode_failed"})),
            Ok(resp) => serde_json::json!({"status": resp.status().as_u16(), "url": url}),
            Err(err) => serde_json::json!({"error": err.to_string(), "url": url}),
        }
    }

    let health = fetch(&client, api, "/v1/health").await;
    let identity = fetch(&client, api, "/v1/project/identity?project_root=/home/wirebot/focusa").await;
    let focus_stack = fetch(&client, api, "/v1/focus/stack").await;
    let workpoint = fetch(&client, api, "/v1/workpoint/resume").await;
    let telemetry = fetch(&client, api, "/v1/telemetry/snapshot").await;

    let payload = serde_json::json!({
        "schema": "focusa.tui_headless_self_test.v1",
        "api_url": api,
        "health": health,
        "project_identity": identity,
        "focus_stack": focus_stack,
        "workpoint": workpoint,
        "telemetry": telemetry,
        "tabs": [
            "1:FocusState", "2:FocusStack", "3:Gate", "4:Events", "5:Metrics",
            "6:Lineage", "w:WorkLoop", "7:Autonomy", "8:Constitution",
            "9:Telemetry", "0:Rfm", "p:Proposals", "s:Skills", "u:Uxp",
            "x:Training",
        ],
        "keybindings": {
            "quit": ["q", "Esc"],
            "refresh": ["r"],
            "next_tab": ["Tab"],
            "prev_tab": ["BackTab"],
            "scroll_down": ["Down", "j"],
            "scroll_up": ["Up", "k"],
        },
    });
    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}