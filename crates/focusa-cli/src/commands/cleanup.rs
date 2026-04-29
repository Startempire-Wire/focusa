//! Safe cleanup command — Spec92 §9.

use clap::Args;
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Args, Debug)]
pub struct CleanupArgs {
    /// Required safe mode; cleanup refuses to run unless set.
    #[arg(long)]
    pub safe: bool,

    /// Preview actions without moving files.
    #[arg(long)]
    pub dry_run: bool,
}

const PRESERVE: &[&str] = &[".beads", "data", "target"];
const CLEAN_PATHS: &[&str] = &[
    ".tmp",
    "apps/menubar/.svelte-kit",
    "apps/menubar/build",
    "apps/menubar/node_modules",
    "apps/pi-extension/node_modules",
];
const TMP_GLOBS: &[&str] = &[
    "/tmp/specgates*",
    "/tmp/commit-*",
    "/tmp/*focusa*.json",
    "/tmp/*focusa*.log",
    "/tmp/*guardian*",
];

fn trash_root() -> PathBuf {
    let stamp = chrono::Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    PathBuf::from(format!("/home/wirebot/.trash/focusa-clean-{stamp}"))
}

fn safe_target(path: &Path, root: &Path) -> PathBuf {
    let rel = path.strip_prefix("/").unwrap_or(path);
    root.join(rel)
}

fn move_recoverable(path: &Path, root: &Path, dry_run: bool) -> Value {
    let path_s = path.display().to_string();
    if PRESERVE
        .iter()
        .any(|p| path_s == *p || path_s.ends_with(&format!("/{p}")))
    {
        return json!({"path": path_s, "status": "skipped", "why": "preserved runtime-critical path"});
    }
    if !path.exists() {
        return json!({"path": path_s, "status": "skipped", "why": "not present"});
    }
    let target = safe_target(path, root);
    if dry_run {
        return json!({"path": path_s, "status": "would_move", "target": target});
    }
    if let Some(parent) = target.parent()
        && let Err(err) = fs::create_dir_all(parent)
    {
        return json!({"path": path_s, "status": "blocked", "what_failed": "create trash parent", "likely_why": err.to_string(), "safe_recovery": "check trash path permissions"});
    }
    match fs::rename(path, &target) {
        Ok(_) => json!({"path": path_s, "status": "completed", "target": target}),
        Err(err) => {
            json!({"path": path_s, "status": "blocked", "what_failed": "recoverable move failed", "likely_why": err.to_string(), "safe_recovery": format!("manually inspect {path_s}")})
        }
    }
}

fn expand_glob(pattern: &str) -> Vec<PathBuf> {
    let cmd = format!("compgen -G '{}'", pattern.replace('\'', "'\\''"));
    let output = Command::new("bash").arg("-lc").arg(cmd).output();
    output
        .ok()
        .map(|out| {
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(PathBuf::from)
                .collect()
        })
        .unwrap_or_default()
}

pub async fn run(args: CleanupArgs, json_mode: bool) -> anyhow::Result<()> {
    if !args.safe {
        anyhow::bail!(
            "[CLI_INPUT_ERROR] cleanup requires --safe; destructive cleanup is not supported"
        );
    }
    let root = trash_root();
    let mut actions = Vec::new();
    for p in CLEAN_PATHS {
        actions.push(move_recoverable(Path::new(p), &root, args.dry_run));
    }
    for pattern in TMP_GLOBS {
        for p in expand_glob(pattern) {
            actions.push(move_recoverable(&p, &root, args.dry_run));
        }
    }
    let blocked = actions
        .iter()
        .filter(|a| a.get("status").and_then(|v| v.as_str()) == Some("blocked"))
        .count();
    let moved = actions
        .iter()
        .filter(|a| a.get("status").and_then(|v| v.as_str()) == Some("completed"))
        .count();
    let would_move = actions
        .iter()
        .filter(|a| a.get("status").and_then(|v| v.as_str()) == Some("would_move"))
        .count();
    let response = json!({
        "status": if blocked == 0 { "completed" } else { "blocked" },
        "summary": if args.dry_run { format!("Safe cleanup preview: {would_move} item(s) would move") } else { format!("Safe cleanup moved {moved} item(s) recoverably") },
        "next_action": if blocked == 0 { "Run focusa doctor or continue release proof" } else { "Inspect blocked cleanup action and rerun focusa cleanup --safe" },
        "why": "Spec92 cleanup must be recoverable and must preserve runtime-critical Focusa state.",
        "commands": ["focusa cleanup --safe --dry-run", "focusa cleanup --safe"],
        "recovery": ["restore files from /home/wirebot/.trash/focusa-clean-*", "focusa doctor"],
        "evidence_refs": ["docs/current/PRODUCTION_RELEASE_COMMANDS.md", "docs/current/DAEMON_RESILIENCE.md"],
        "docs": ["docs/92-agent-first-polish-hooks-efficiency-spec.md"],
        "warnings": ["preserves .beads, data, and target"],
        "details": {"trash_root": root, "actions": actions},
    });
    if json_mode {
        println!("{}", serde_json::to_string_pretty(&response)?);
    } else {
        println!(
            "Status: {}",
            response["status"].as_str().unwrap_or("blocked")
        );
        println!(
            "Summary: {}",
            response["summary"].as_str().unwrap_or("cleanup complete")
        );
        println!(
            "Next action: {}",
            response["next_action"].as_str().unwrap_or("focusa doctor")
        );
        println!(
            "Why: {}",
            response["why"].as_str().unwrap_or("safe cleanup")
        );
        println!("Command: focusa cleanup --safe --dry-run");
        println!("Recovery: restore files from /home/wirebot/.trash/focusa-clean-*");
        println!("Evidence: docs/current/PRODUCTION_RELEASE_COMMANDS.md");
        println!("Docs: docs/92-agent-first-polish-hooks-efficiency-spec.md");
    }
    Ok(())
}
