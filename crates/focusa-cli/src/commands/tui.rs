//! Focusa CLI: `focusa tui` — proxy to the focusa-tui binary with headless self-test.

use anyhow::{Context, Result};
use clap::Args;
use std::process::Command;

fn urlencode(s: &str) -> String {
    // Minimal RFC3986 percent-encoding for path-query values: encode any byte
    // outside the unreserved set (ALPHA / DIGIT / "-" / "." / "_" / "~") and
    // "/" so that arbitrary project_root paths survive in the URL.
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(b as char);
            }
            _ => {
                out.push('%');
                out.push_str(&format!("{:02X}", b));
            }
        }
    }
    out
}

#[derive(Args)]
pub struct TuiArgs {
    /// Override the Focusa API URL for the TUI (defaults to FOCUSA_API_URL then http://127.0.0.1:8787).
    #[arg(long)]
    pub api_url: Option<String>,
    /// Project root to scope API requests. Falls back to the daemon's
    /// `/v1/project/identity` (no-op if the daemon can't resolve one).
    #[arg(long)]
    pub project_root: Option<String>,
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

    // Resolve project_root: CLI flag > env > daemon's own identity.
    let project_root = match args
        .project_root
        .clone()
        .or_else(|| std::env::var("FOCUSA_PROJECT_ROOT").ok())
    {
        Some(r) => Some(r),
        None => match reqwest::get(format!("{api}/v1/project/identity")).await {
            Ok(resp) if resp.status().is_success() => {
                let body: serde_json::Value = resp.json().await.ok().unwrap_or_default();
                body.get("project_identity")
                    .and_then(|p| p.get("root"))
                    .and_then(|r| r.as_str())
                    .map(|s| s.to_string())
            }
            _ => None,
        },
    };

    if args.headless_self_test {
        return run_headless_self_test(&api, project_root.as_deref()).await;
    }

    let bin = locate_tui_binary().context(
        "focusa-tui binary not found; checked FOCUSA_TUI_BIN, the installed focusa CLI directory, ~/.focusa/bin, PATH, and target/{release,debug}. Run `focusa install --dry-run`, then reinstall Focusa or set FOCUSA_TUI_BIN.",
    )?;
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
    locate_tui_binary_from(
        std::env::var_os("FOCUSA_TUI_BIN").map(std::path::PathBuf::from),
        std::env::current_exe().ok(),
        std::env::var_os("HOME").map(std::path::PathBuf::from),
        std::env::var_os("PATH"),
        std::env::current_dir().ok(),
    )
}

fn locate_tui_binary_from(
    override_path: Option<std::path::PathBuf>,
    current_exe: Option<std::path::PathBuf>,
    home: Option<std::path::PathBuf>,
    path: Option<std::ffi::OsString>,
    cwd: Option<std::path::PathBuf>,
) -> Option<std::path::PathBuf> {
    let mut candidates = Vec::new();
    if let Some(candidate) = override_path {
        candidates.push(candidate);
    }
    if let Some(exe) = current_exe
        && let Some(parent) = exe.parent()
    {
        candidates.push(parent.join("focusa-tui"));
    }
    if let Some(home) = home {
        candidates.push(home.join(".focusa/bin/focusa-tui"));
    }
    if let Some(path) = path {
        candidates
            .extend(std::env::split_paths(&path).map(|directory| directory.join("focusa-tui")));
    }
    if let Some(cwd) = cwd {
        for profile in ["release", "debug"] {
            candidates.push(cwd.join("target").join(profile).join("focusa-tui"));
        }
    }
    candidates.into_iter().find(|candidate| candidate.is_file())
}

async fn run_headless_self_test(api: &str, project_root: Option<&str>) -> Result<()> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .context("reqwest client init failed")?;

    async fn fetch_get(client: &reqwest::Client, api: &str, path: &str) -> serde_json::Value {
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

    async fn fetch_post(
        client: &reqwest::Client,
        api: &str,
        path: &str,
        body: serde_json::Value,
    ) -> serde_json::Value {
        let url = format!("{}{}", api.trim_end_matches('/'), path);
        match client.post(&url).json(&body).send().await {
            Ok(resp) if resp.status().is_success() => resp
                .json::<serde_json::Value>()
                .await
                .unwrap_or_else(|_| serde_json::json!({"raw_error": "decode_failed"})),
            Ok(resp) => serde_json::json!({"status": resp.status().as_u16(), "url": url}),
            Err(err) => serde_json::json!({"error": err.to_string(), "url": url}),
        }
    }

    let health = fetch_get(&client, api, "/v1/health").await;
    let identity_path = project_root
        .map(|r| format!("/v1/project/identity?project_root={}", urlencode(r)))
        .unwrap_or_else(|| "/v1/project/identity".to_string());
    let identity = fetch_get(&client, api, &identity_path).await;
    let focus_stack = fetch_get(&client, api, "/v1/focus/stack").await;
    // /v1/workpoint/resume requires POST with a JSON body.
    let workpoint = fetch_post(
        &client,
        api,
        "/v1/workpoint/resume",
        serde_json::json!({
            "project_root": project_root,
            "continuity_id": null,
            "current_ask": "headless self-test"
        }),
    )
    .await;
    // /v1/telemetry/snapshot doesn't exist; record status only when non-404.
    let telemetry_url = format!("{}/v1/telemetry/snapshot", api.trim_end_matches('/'));
    let telemetry = match client.get(&telemetry_url).send().await {
        Ok(r) if r.status().as_u16() == 404 => {
            serde_json::json!({"status": "absent", "url": telemetry_url})
        }
        Ok(r) if r.status().is_success() => r
            .json::<serde_json::Value>()
            .await
            .unwrap_or_else(|_| serde_json::json!({"raw_error": "decode_failed"})),
        Ok(r) => serde_json::json!({"status": r.status().as_u16(), "url": telemetry_url}),
        Err(e) => serde_json::json!({"error": e.to_string(), "url": telemetry_url}),
    };

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

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> std::path::PathBuf {
        let path =
            std::env::temp_dir().join(format!("focusa-tui-locate-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("create fixture");
        path
    }

    #[test]
    fn locate_tui_finds_sibling_of_installed_cli() {
        let root = fixture("sibling");
        let cli = root.join("focusa");
        let tui = root.join("focusa-tui");
        std::fs::write(&cli, "cli").unwrap();
        std::fs::write(&tui, "tui").unwrap();
        let found = locate_tui_binary_from(None, Some(cli), None, None, None);
        assert_eq!(found.as_deref(), Some(tui.as_path()));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn locate_tui_honors_explicit_override_first() {
        let root = fixture("override");
        let override_path = root.join("custom-tui");
        let sibling = root.join("focusa-tui");
        std::fs::write(&override_path, "custom").unwrap();
        std::fs::write(&sibling, "sibling").unwrap();
        let found = locate_tui_binary_from(
            Some(override_path.clone()),
            Some(root.join("focusa")),
            None,
            None,
            None,
        );
        assert_eq!(found.as_deref(), Some(override_path.as_path()));
        let _ = std::fs::remove_dir_all(root);
    }
}
