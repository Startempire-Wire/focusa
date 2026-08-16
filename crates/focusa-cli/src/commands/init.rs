//! `focusa init` — minimal, low-risk project bootstrap.
//!
//! Writes a `.focusa-project.json` marker in the current directory (or the
//! path passed via `--project-root`), runs a daemon health check, and prints
//! the canonical project identity. Use `--quickstart` to skip the daemon
//! health probe and write the marker only; this is the right knob for the
//! 60-second quickstart in README.md.

use anyhow::{Context, Result};
use clap::Args;
use focusa_core::scope_safety::{ScopeSafety, classify_project_root};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

#[derive(Args)]
pub struct InitArgs {
    /// Project root where `.focusa-project.json` should be written.
    /// Defaults to the current directory.
    #[arg(long)]
    pub project_root: Option<String>,

    /// Skip the live daemon health probe and just write the marker.
    /// This is the 60-second quickstart path: cheap, hermetic, no network.
    #[arg(long)]
    pub quickstart: bool,

    /// Skip writing the marker file (dry-run). Useful for CI guards.
    #[arg(long)]
    pub dry_run: bool,

    /// Acknowledge that --project-root (or cwd) is a broad unsafe path
    /// (e.g. /, /root, /home). Required to write a marker at such a path.
    /// Without this flag, init refuses and prints a remediation pointing at
    /// `focusa project identity` + the canonical project_root path.
    #[arg(long)]
    pub allow_unsafe_root: bool,
}

pub async fn run(args: InitArgs, json: bool) -> Result<()> {
    let cwd = std::env::current_dir().context("cwd unavailable")?;
    let project_root = match args.project_root.as_deref() {
        Some(value) if !value.trim().is_empty() => PathBuf::from(value),
        _ => cwd,
    };

    // SAFETY: reject broad unsafe roots unless --allow-unsafe-root is set.
    // Without this, `focusa init --quickstart` happily writes a marker to
    // `/root` (or any /home/<user>) which is exactly the case the auto-
    // bootstrap nag warns against. This was an MVP-launch blocker.
    //
    // This uses the shared ScopeSafety classifier so init, project identity,
    // trajectory, and workpoint all agree on what is too broad to bind.
    let project_root_str = project_root.to_string_lossy();
    let safety = classify_project_root(&project_root_str);
    if !safety.is_safe() && !args.allow_unsafe_root {
        if json {
            let blocked = json!({
                "schema": "focusa.init.v1",
                "status": "blocked",
                "failure_class": "scope_mismatch",
                "reason": safety.reason(),
                "project_root": project_root.display().to_string(),
                "next_step_hint": safety.next_step_hint(),
                "safe_next_commands": [
                    "cd /path/to/repo && focusa init --quickstart",
                    "focusa onboard --scope host"
                ]
            });
            println!("{}", serde_json::to_string_pretty(&blocked)?);
            return Ok(());
        }
        anyhow::bail!(
            "Scope blocked: {} is too broad to bind as a Focusa project.\n\
             \n\
             Why:\n\
               {} is a {}, not a focused project.\n\
             \n\
             Do this instead:\n\
               cd /path/to/your/repo\n\
               focusa init --quickstart\n\
             \n\
             For host setup:\n\
               focusa onboard --scope host\n\
             \n\
             Override:\n\
               focusa init --quickstart --allow-unsafe-root\n\
               Not recommended.",
            project_root.display(),
            project_root.display(),
            safety.human_kind()
        );
    }

    // ensure_dir_all so a fresh `--project-root /tmp/foo/bar` works on a
    // directory that has not been created yet (gap #4 from the dry-run).
    if !args.dry_run {
        std::fs::create_dir_all(&project_root)
            .with_context(|| format!("could not create {}", project_root.display()))?;
    }

    let marker_path = project_root.join(".focusa-project.json");
    let slug = project_slug(&project_root);
    let daemon_base =
        std::env::var("FOCUSA_DAEMON_URL").unwrap_or_else(|_| "http://127.0.0.1:8787".into());

    let health = if args.quickstart {
        json!({"checked": false, "ok": null, "reason": "quickstart skip"})
    } else {
        match probe_health(&daemon_base).await {
            Ok(payload) => payload,
            Err(err) => {
                json!({"checked": true, "ok": false, "error": err.to_string(), "url": daemon_base})
            }
        }
    };

    let marker = json!({
        "schema": "focusa.project.v1",
        "project_id": slug,
        "canonical_name": title_from_slug(&slug),
        "project_root": project_root.display().to_string(),
        "beads_prefix": slug,
        "workspace_kind": detect_workspace_kind(&project_root),
        "aliases": [],
        "created_at": chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
    });

    let mut payload = json!({
        "schema": "focusa.init.v1",
        "marker_path": marker_path.display().to_string(),
        "project_id": slug,
        "canonical_name": marker["canonical_name"],
        "daemon_health": health,
    });

    if args.dry_run {
        payload["mode"] = json!("dry_run");
        payload["marker_preview"] = marker;
    } else {
        // #243: canonical marker production through the shared core service.
        let marker_struct = focusa_core::project_marker::ProjectMarker {
            schema: focusa_core::project_marker::MARKER_SCHEMA.into(),
            project_id: slug.clone(),
            canonical_name: marker["canonical_name"].as_str().unwrap_or("").to_string(),
            project_root: project_root.display().to_string(),
            repo_remote: None,
            beads_prefix: marker["beads_prefix"].as_str().map(str::to_string),
            workspace_kind: marker["workspace_kind"].as_str().map(str::to_string),
            continuity_id: None,
            aliases: vec![],
            created_at: marker["created_at"].as_str().unwrap_or("").to_string(),
            updated_at: None,
        };
        let outcome = focusa_core::project_marker::write_marker(
            &project_root,
            &marker_struct,
            &focusa_core::project_marker::MarkerWriteOptions::default(),
        )?;
        if matches!(
            outcome,
            focusa_core::project_marker::MarkerWriteOutcome::BlockedPermission { .. }
        ) {
            anyhow::bail!(
                "project marker blocked: directory is owned by another user; rerun as that user"
            );
        }
        payload["mode"] = json!("written");
        payload["marker"] = marker;
        payload["marker_outcome"] = serde_json::to_value(outcome)?;
    }

    // Loud override: surface when a marker was written with unsafe-root override.
    if args.allow_unsafe_root && !safety.is_safe() {
        payload["scope_override"] = json!(true);
        payload["scope_warning"] = json!("unsafe project root override used");
    }

    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}

fn project_slug(project_root: &Path) -> String {
    let raw = project_root
        .file_name()
        .and_then(|os| os.to_str())
        .unwrap_or("focusa-project")
        .to_ascii_lowercase();
    raw.chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Returns true if `path` is a broad unsafe root that should not host a
/// project marker. These are directories that contain user homes or are
/// shared mutable roots; a project marker here is meaningless and pollutes
/// downstream scope (state.db, beads, workpoints, trajectory).
fn title_from_slug(slug: &str) -> String {
    slug.split('-')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn detect_workspace_kind(project_root: &Path) -> &'static str {
    if project_root.join("Cargo.toml").exists() {
        "rust-monorepo"
    } else if project_root.join("go.mod").exists() || project_root.join("go.work").exists() {
        "go-workspace"
    } else if project_root.join("package.json").exists() {
        "node-workspace"
    } else {
        "unknown"
    }
}

async fn probe_health(daemon_base: &str) -> Result<Value> {
    let url = format!("{}/v1/health", daemon_base.trim_end_matches('/'));
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()?;
    let resp = client.get(&url).send().await?;
    let status = resp.status().as_u16();
    let body: Value = resp
        .json()
        .await
        .unwrap_or_else(|_| json!({"raw_error": "decode_failed"}));
    Ok(json!({
        "checked": true,
        "ok": (200..300).contains(&status),
        "status": status,
        "url": url,
        "body": body,
    }))
}
