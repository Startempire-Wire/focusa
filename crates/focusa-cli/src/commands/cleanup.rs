//! Safe cleanup command — Spec92 §9.

use crate::commands::{scope::ensure_project_root_scope_safe, scope_resolver};
use clap::Args;
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Args, Debug)]
pub struct CleanupArgs {
    /// Required safe mode; cleanup refuses to run unless set.
    #[arg(long)]
    pub safe: bool,

    /// Safe project folder/container whose generated residue may be moved.
    #[arg(long)]
    pub project_root: Option<String>,

    /// Preview actions without moving files.
    #[arg(long)]
    pub dry_run: bool,

    /// Also include bounded global /tmp Focusa residue patterns.
    /// Off by default because /tmp may contain other sessions' evidence.
    #[arg(long)]
    pub include_global_tmp: bool,
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
    let stamp = format!(
        "{}-{}",
        chrono::Utc::now().format("%Y%m%dT%H%M%S%3fZ"),
        uuid::Uuid::now_v7()
    );
    let base = std::env::var_os("FOCUSA_TRASH_DIR")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("XDG_DATA_HOME").map(|p| PathBuf::from(p).join("Trash")))
        .or_else(|| std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".trash")))
        .unwrap_or_else(std::env::temp_dir);
    base.join(format!("focusa-clean-{stamp}"))
}

fn safe_target(path: &Path, root: &Path) -> PathBuf {
    let rel = path.strip_prefix("/").unwrap_or(path);
    root.join(rel)
}

fn copy_path_no_follow(source: &Path, target: &Path) -> std::io::Result<()> {
    let metadata = fs::symlink_metadata(source)?;
    if metadata.file_type().is_symlink() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "cleanup refuses to follow or copy symlinks",
        ));
    }
    if metadata.is_file() {
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(source, target)?;
        fs::set_permissions(target, metadata.permissions())?;
        return Ok(());
    }
    if metadata.is_dir() {
        fs::create_dir_all(target)?;
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            copy_path_no_follow(&entry.path(), &target.join(entry.file_name()))?;
        }
        fs::set_permissions(target, metadata.permissions())?;
        return Ok(());
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "unsupported cleanup source type",
    ))
}

fn remove_source_after_copy(path: &Path) -> std::io::Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
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
    if target.exists() {
        return json!({"path": path_s, "status": "blocked", "what_failed": "trash target collision", "likely_why": target.display().to_string(), "safe_recovery": "retry to allocate a fresh trash session"});
    }
    if let Some(parent) = target.parent()
        && let Err(err) = fs::create_dir_all(parent)
    {
        return json!({"path": path_s, "status": "blocked", "what_failed": "create trash parent", "likely_why": err.to_string(), "safe_recovery": "check trash path permissions"});
    }
    match fs::rename(path, &target) {
        Ok(_) => {
            json!({"path": path_s, "status": "completed", "target": target, "move_mode": "rename"})
        }
        Err(rename_error) if rename_error.kind() == std::io::ErrorKind::CrossesDevices => {
            match copy_path_no_follow(path, &target) {
                Ok(()) => match remove_source_after_copy(path) {
                    Ok(()) => {
                        json!({"path": path_s, "status": "completed", "target": target, "move_mode": "copy_then_remove"})
                    }
                    Err(remove_error) => {
                        json!({"path": path_s, "status": "blocked", "what_failed": "source removal after verified copy failed", "likely_why": remove_error.to_string(), "target": target, "safe_recovery": "source remains; inspect both source and trash copy before retrying"})
                    }
                },
                Err(copy_error) => {
                    json!({"path": path_s, "status": "blocked", "what_failed": "cross-filesystem recoverable copy failed", "likely_why": copy_error.to_string(), "target": target, "safe_recovery": "source remains unchanged; inspect partial trash target before retrying"})
                }
            }
        }
        Err(err) => {
            json!({"path": path_s, "status": "blocked", "what_failed": "recoverable move failed", "likely_why": err.to_string(), "safe_recovery": format!("source remains; inspect {path_s} and retry")})
        }
    }
}

fn simple_tmp_glob_match(name: &str, pattern: &str) -> bool {
    if name.contains('/') || name.contains('\\') {
        return false;
    }
    let Some(rest) = pattern.strip_prefix("/tmp/") else {
        return false;
    };
    if !rest.contains('*') {
        return name == rest;
    }
    let mut cursor = 0usize;
    let anchored_start = !rest.starts_with('*');
    let anchored_end = !rest.ends_with('*');
    let parts: Vec<&str> = rest.split('*').filter(|part| !part.is_empty()).collect();
    if parts.is_empty() {
        return true;
    }
    if anchored_start && !name.starts_with(parts[0]) {
        return false;
    }
    for part in &parts {
        let Some(pos) = name[cursor..].find(part) else {
            return false;
        };
        cursor += pos + part.len();
    }
    !anchored_end || name.ends_with(parts[parts.len() - 1])
}

fn expand_glob(pattern: &str) -> Vec<PathBuf> {
    if !pattern.starts_with("/tmp/") || !pattern.contains('*') {
        let path = PathBuf::from(pattern);
        return path.exists().then_some(path).into_iter().collect();
    }
    fs::read_dir("/tmp")
        .ok()
        .into_iter()
        .flat_map(|entries| entries.filter_map(Result::ok))
        .filter_map(|entry| {
            let name = entry.file_name();
            let name = name.to_str()?;
            simple_tmp_glob_match(name, pattern).then(|| entry.path())
        })
        .collect()
}

pub async fn run(args: CleanupArgs, json_mode: bool) -> anyhow::Result<()> {
    if !args.safe {
        anyhow::bail!(
            "[CLI_INPUT_ERROR] cleanup requires --safe; destructive cleanup is not supported"
        );
    }
    let resolved = scope_resolver::resolve_project_scope(
        args.project_root.as_deref(),
        None,
        std::env::current_dir()
            .ok()
            .and_then(|path| path.to_str().map(str::to_string))
            .as_deref(),
    )?;
    ensure_project_root_scope_safe(
        Some(resolved.project_root.as_str()),
        "cleanup: project_root",
    )?;
    let project_root = PathBuf::from(&resolved.project_root);
    let root = trash_root();
    let mut actions = Vec::new();
    for p in CLEAN_PATHS {
        actions.push(move_recoverable(&project_root.join(p), &root, args.dry_run));
    }
    if args.include_global_tmp {
        for pattern in TMP_GLOBS {
            for p in expand_glob(pattern) {
                actions.push(move_recoverable(&p, &root, args.dry_run));
            }
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
        "commands": if args.include_global_tmp {
            vec!["focusa cleanup --safe --dry-run --include-global-tmp", "focusa cleanup --safe --include-global-tmp"]
        } else {
            vec!["focusa cleanup --safe --dry-run", "focusa cleanup --safe"]
        },
        "recovery": ["restore files from the reported trash_root", "focusa doctor"],
        "evidence_refs": ["docs/current/PRODUCTION_RELEASE_COMMANDS.md", "docs/current/DAEMON_RESILIENCE.md"],
        "docs": ["docs/92-agent-first-polish-hooks-efficiency-spec.md"],
        "warnings": [
            "preserves .beads, data, and target",
            if args.include_global_tmp { "global /tmp cleanup explicitly authorized" } else { "global /tmp cleanup excluded; pass --include-global-tmp explicitly" }
        ],
        "details": {
            "trash_root": root,
            "project_root": resolved.project_root,
            "scope_source": format!("{:?}", resolved.scope_source),
            "actions": actions
        },
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
        println!(
            "Project root: {}",
            response["details"]["project_root"]
                .as_str()
                .unwrap_or("unknown")
        );
        println!("Command: focusa cleanup --safe --dry-run");
        println!("Recovery: restore files from the reported trash_root");
        println!("Evidence: docs/current/PRODUCTION_RELEASE_COMMANDS.md");
        println!("Docs: docs/92-agent-first-polish-hooks-efficiency-spec.md");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tmp_glob_match_is_prefix_suffix_only_under_tmp() {
        assert!(simple_tmp_glob_match(
            "focusa-audit.json",
            "/tmp/*focusa*.json"
        ));
        assert!(simple_tmp_glob_match("specgates-123", "/tmp/specgates*"));
        assert!(!simple_tmp_glob_match(
            "../focusa-audit.json",
            "/tmp/*focusa*.json"
        ));
        assert!(!simple_tmp_glob_match(
            "focusa-audit.log",
            "/tmp/*focusa*.json"
        ));
        assert!(!simple_tmp_glob_match(
            "focusa-audit.json",
            "../tmp/*focusa*.json"
        ));
    }

    #[test]
    fn safe_target_keeps_absolute_paths_inside_trash_root() {
        let root = Path::new("/tmp/focusa-trash-root");
        let target = safe_target(Path::new("/tmp/focusa-audit.json"), root);
        assert!(target.starts_with(root));
        assert_eq!(target, root.join("tmp/focusa-audit.json"));
    }

    #[test]
    fn trash_roots_are_unique_per_invocation() {
        assert_ne!(trash_root(), trash_root());
    }

    #[test]
    fn recursive_copy_preserves_source_until_explicit_remove() {
        let fixture =
            std::env::temp_dir().join(format!("focusa-cleanup-copy-{}", uuid::Uuid::now_v7()));
        let source = fixture.join("source");
        let target = fixture.join("target");
        fs::create_dir_all(source.join("nested")).unwrap();
        fs::write(source.join("nested/proof.txt"), "proof").unwrap();
        copy_path_no_follow(&source, &target).unwrap();
        assert_eq!(
            fs::read_to_string(target.join("nested/proof.txt")).unwrap(),
            "proof"
        );
        assert!(source.join("nested/proof.txt").is_file());
        remove_source_after_copy(&source).unwrap();
        assert!(!source.exists());
        let _ = fs::remove_dir_all(fixture);
    }

    #[cfg(unix)]
    #[test]
    fn recursive_copy_refuses_symlinks() {
        use std::os::unix::fs::symlink;
        let fixture =
            std::env::temp_dir().join(format!("focusa-cleanup-symlink-{}", uuid::Uuid::now_v7()));
        fs::create_dir_all(&fixture).unwrap();
        fs::write(fixture.join("real"), "data").unwrap();
        symlink(fixture.join("real"), fixture.join("link")).unwrap();
        let error = copy_path_no_follow(&fixture.join("link"), &fixture.join("target"))
            .expect_err("symlink must be refused");
        assert_eq!(error.kind(), std::io::ErrorKind::Unsupported);
        let _ = fs::remove_dir_all(fixture);
    }
}
