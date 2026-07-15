//! Pre-deserialization Pi native-session launcher guard (Spec 130 §§44–48).

use anyhow::{Context, Result, bail};
use clap::{Args, Subcommand};
use serde::Serialize;
use std::env;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

#[path = "pi_launch_migration.rs"]
mod migration;

const DEFAULT_STARTUP_CAP_MIB: u64 = 256;

#[derive(Subcommand, Debug)]
pub enum PiCmd {
    /// Preflight a native session before Pi loads it, then launch only when safe.
    Launch(PiLaunchArgs),
}

#[derive(Args, Debug)]
pub struct PiLaunchArgs {
    /// Pi executable name or path.
    #[arg(long, default_value = "pi")]
    pi_bin: OsString,

    /// Startup migration threshold in MiB.
    #[arg(long, default_value_t = DEFAULT_STARTUP_CAP_MIB)]
    source_cap_mib: u64,

    /// Stream-migrate an oversized source before launch.
    #[arg(long)]
    migrate: bool,

    /// Private migration output root; defaults beside the source session.
    #[arg(long)]
    migration_dir: Option<PathBuf>,

    /// Pi arguments; place them after `--`.
    #[arg(last = true, allow_hyphen_values = true)]
    pi_args: Vec<OsString>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SourceKind {
    Session,
    Fork,
    Export,
    SessionId,
    Continue,
}

#[derive(Debug, Clone)]
struct SourceHint {
    kind: SourceKind,
    value: Option<OsString>,
}

#[derive(Debug, Serialize)]
struct PreflightReceipt {
    schema: &'static str,
    status: &'static str,
    source: &'static str,
    session_bytes: u64,
    startup_cap_bytes: u64,
    action: &'static str,
    next: Option<&'static str>,
}

#[derive(Debug)]
enum LaunchPlan {
    NewSession,
    Allowed {
        source: PathBuf,
        bytes: u64,
        cap: u64,
    },
    MigrationRequired {
        source: PathBuf,
        bytes: u64,
        cap: u64,
    },
}

pub fn run(command: PiCmd, json_mode: bool) -> Result<()> {
    match command {
        PiCmd::Launch(args) => run_launch(args, json_mode),
    }
}

fn run_launch(args: PiLaunchArgs, json_mode: bool) -> Result<()> {
    let cwd = env::current_dir().context("resolve launcher cwd")?;
    let plan = preflight(&args, &cwd)?;
    let mut launch_args = args.pi_args.clone();
    match plan {
        LaunchPlan::NewSession => emit_receipt(
            json_mode,
            &PreflightReceipt {
                schema: "focusa.pi_native_session_preflight.v1",
                status: "allowed",
                source: "none",
                session_bytes: 0,
                startup_cap_bytes: startup_cap_bytes(&args),
                action: "launch_new_session",
                next: None,
            },
        )?,
        LaunchPlan::Allowed { bytes, cap, .. } => emit_receipt(
            json_mode,
            &PreflightReceipt {
                schema: "focusa.pi_native_session_preflight.v1",
                status: "allowed",
                source: "redacted",
                session_bytes: bytes,
                startup_cap_bytes: cap,
                action: "launch_existing_session",
                next: None,
            },
        )?,
        LaunchPlan::MigrationRequired { source, bytes, cap } => {
            if !args.migrate {
                emit_receipt(
                    json_mode,
                    &PreflightReceipt {
                        schema: "focusa.pi_native_session_preflight.v1",
                        status: "blocked",
                        source: "redacted",
                        session_bytes: bytes,
                        startup_cap_bytes: cap,
                        action: "refuse_full_load",
                        next: Some("focusa pi launch --migrate -- --session <path-or-id>"),
                    },
                )?;
                bail!("oversized native session refused before Pi deserialization");
            }
            let migrated = migration::migrate(&source, args.migration_dir.as_deref(), &cwd)?;
            migration::rewrite_args_for_recovery(&mut launch_args, &migrated.recovery_path);
            emit_receipt(
                json_mode,
                &PreflightReceipt {
                    schema: "focusa.pi_native_session_preflight.v1",
                    status: "migrated",
                    source: "redacted",
                    session_bytes: migrated.recovery_bytes,
                    startup_cap_bytes: cap,
                    action: "launch_recovery_segment",
                    next: None,
                },
            )?;
            if !json_mode {
                eprintln!(
                    "Migration verified: source_bytes={} manifest={}",
                    migrated.source_bytes,
                    migrated.manifest_path.display()
                );
            }
        }
    }

    let status = Command::new(&args.pi_bin)
        .args(&launch_args)
        .status()
        .with_context(|| format!("launch Pi executable {}", args.pi_bin.to_string_lossy()))?;
    if !status.success() {
        bail!("Pi exited with status {status}");
    }
    Ok(())
}

fn emit_receipt(json_mode: bool, receipt: &PreflightReceipt) -> Result<()> {
    if json_mode {
        println!("{}", serde_json::to_string_pretty(receipt)?);
    } else {
        eprintln!(
            "Focusa Pi preflight: status={} source={} bytes={} cap={} action={}",
            receipt.status,
            receipt.source,
            receipt.session_bytes,
            receipt.startup_cap_bytes,
            receipt.action
        );
        if let Some(next) = receipt.next {
            eprintln!("Next: {next}");
        }
    }
    Ok(())
}

fn preflight(args: &PiLaunchArgs, cwd: &Path) -> Result<LaunchPlan> {
    let hint = parse_source_hint(&args.pi_args)?;
    let Some(hint) = hint else {
        return Ok(LaunchPlan::NewSession);
    };
    let roots = session_roots(&args.pi_args, cwd);
    let source = match hint.kind {
        SourceKind::Continue => latest_project_session(&roots, cwd),
        _ => hint
            .value
            .as_deref()
            .and_then(|value| resolve_source(value, &roots, cwd)),
    };
    let Some(source) = source else {
        if hint.kind == SourceKind::SessionId {
            return Ok(LaunchPlan::NewSession);
        }
        bail!("native session source could not be resolved without deserialization");
    };
    let metadata = fs::metadata(&source).context("stat native session before Pi launch")?;
    if !metadata.is_file() {
        bail!("native session source is not a file");
    }
    let bytes = metadata.len();
    let cap = startup_cap_bytes(args);
    if bytes >= cap {
        Ok(LaunchPlan::MigrationRequired { source, bytes, cap })
    } else {
        Ok(LaunchPlan::Allowed { source, bytes, cap })
    }
}

fn startup_cap_bytes(args: &PiLaunchArgs) -> u64 {
    let env_mib = env::var("FOCUSA_NATIVE_STARTUP_MIGRATION_MIB")
        .ok()
        .and_then(|value| value.parse::<u64>().ok());
    let mib = env_mib.unwrap_or(args.source_cap_mib).max(1);
    mib.saturating_mul(1024 * 1024)
}

fn parse_source_hint(args: &[OsString]) -> Result<Option<SourceHint>> {
    if args.iter().any(|arg| arg == "--no-session") {
        return Ok(None);
    }
    if args.iter().any(|arg| {
        let text = arg.to_string_lossy();
        text == "--resume" || text == "-r" || text.starts_with("--resume=")
    }) {
        bail!("interactive --resume cannot be preflighted; use `--session <path-or-id>`");
    }
    for (index, arg) in args.iter().enumerate() {
        let text = arg.to_string_lossy();
        if text == "--continue" || text == "-c" {
            return Ok(Some(SourceHint {
                kind: SourceKind::Continue,
                value: None,
            }));
        }
        for (flag, kind) in [
            ("--session", SourceKind::Session),
            ("--fork", SourceKind::Fork),
            ("--export", SourceKind::Export),
            ("--session-id", SourceKind::SessionId),
        ] {
            if text == flag {
                let value = args
                    .get(index + 1)
                    .cloned()
                    .context("session flag requires a value")?;
                return Ok(Some(SourceHint {
                    kind,
                    value: Some(value),
                }));
            }
            if let Some(value) = text.strip_prefix(&format!("{flag}=")) {
                return Ok(Some(SourceHint {
                    kind,
                    value: Some(OsString::from(value)),
                }));
            }
        }
    }
    Ok(None)
}

fn session_roots(args: &[OsString], cwd: &Path) -> Vec<PathBuf> {
    let explicit = option_value(args, "--session-dir").map(PathBuf::from);
    let configured = env::var_os("PI_CODING_AGENT_SESSION_DIR").map(PathBuf::from);
    let agent_dir = env::var_os("PI_CODING_AGENT_DIR")
        .map(PathBuf::from)
        .map(|path| path.join("sessions"));
    let home = env::var_os("HOME")
        .map(PathBuf::from)
        .map(|path| path.join(".pi/agent/sessions"));
    let root = explicit
        .or(configured)
        .or(agent_dir)
        .or(home)
        .unwrap_or_else(|| cwd.join(".pi/agent/sessions"));
    let project = root.join(encode_project_dir(cwd));
    if project.is_dir() {
        vec![project, root]
    } else {
        vec![root]
    }
}

fn option_value(args: &[OsString], flag: &str) -> Option<OsString> {
    for (index, arg) in args.iter().enumerate() {
        let text = arg.to_string_lossy();
        if text == flag {
            return args.get(index + 1).cloned();
        }
        if let Some(value) = text.strip_prefix(&format!("{flag}=")) {
            return Some(OsString::from(value));
        }
    }
    None
}

fn resolve_source(value: &OsStr, roots: &[PathBuf], cwd: &Path) -> Option<PathBuf> {
    let path = PathBuf::from(value);
    for candidate in [path.clone(), cwd.join(&path)] {
        if candidate.is_file() {
            return fs::canonicalize(candidate).ok();
        }
    }
    let token = value.to_string_lossy();
    session_files(roots).into_iter().find(|candidate| {
        candidate
            .file_name()
            .is_some_and(|name| name.to_string_lossy().contains(token.as_ref()))
    })
}

fn latest_project_session(roots: &[PathBuf], cwd: &Path) -> Option<PathBuf> {
    let project_name = encode_project_dir(cwd);
    let mut candidates = session_files(roots)
        .into_iter()
        .filter(|path| {
            path.parent()
                .is_some_and(|parent| parent.ends_with(&project_name))
        })
        .collect::<Vec<_>>();
    if candidates.is_empty() {
        candidates = session_files(roots);
    }
    candidates.into_iter().max_by_key(|path| {
        fs::metadata(path)
            .and_then(|metadata| metadata.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH)
    })
}

fn session_files(roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for root in roots {
        collect_jsonl(root, 0, &mut files);
    }
    files.sort();
    files.dedup();
    files
}

fn collect_jsonl(dir: &Path, depth: usize, files: &mut Vec<PathBuf>) {
    if depth > 2 {
        return;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_jsonl(&path, depth + 1, files);
        } else if path.extension().is_some_and(|ext| ext == "jsonl") {
            files.push(path);
        }
    }
}

fn encode_project_dir(path: &Path) -> String {
    let body = path
        .to_string_lossy()
        .trim_matches(['/', '\\'])
        .replace(['/', '\\'], "-");
    format!("--{body}--")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::thread::sleep;
    use std::time::Duration;

    fn temp_dir(label: &str) -> PathBuf {
        let path = env::temp_dir().join(format!("focusa-pi-launch-{label}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn args(pi_args: &[&str], cap_mib: u64) -> PiLaunchArgs {
        PiLaunchArgs {
            pi_bin: OsString::from("pi"),
            source_cap_mib: cap_mib,
            migrate: false,
            migration_dir: None,
            pi_args: pi_args.iter().map(OsString::from).collect(),
        }
    }

    #[test]
    fn encodes_project_session_directory() {
        assert_eq!(
            encode_project_dir(Path::new("/home/wirebot/focusa")),
            "--home-wirebot-focusa--"
        );
        assert_eq!(encode_project_dir(Path::new("/root")), "--root--");
    }

    #[test]
    fn rejects_interactive_resume() {
        let error = parse_source_hint(&[OsString::from("--resume")]).unwrap_err();
        assert!(error.to_string().contains("--session <path-or-id>"));
        let mixed = parse_source_hint(&[
            OsString::from("--session"),
            OsString::from("known.jsonl"),
            OsString::from("--resume"),
        ])
        .unwrap_err();
        assert!(mixed.to_string().contains("cannot be preflighted"));
    }

    #[test]
    fn allows_new_and_bounded_explicit_sessions() {
        let root = temp_dir("allowed");
        let source = root.join("session.jsonl");
        File::create(&source).unwrap();
        assert!(matches!(
            preflight(&args(&[], 1), &root).unwrap(),
            LaunchPlan::NewSession
        ));
        let launch = args(&["--session", source.to_str().unwrap()], 1);
        match preflight(&launch, &root).unwrap() {
            LaunchPlan::Allowed {
                source: resolved,
                bytes: 0,
                ..
            } => assert_eq!(resolved, source),
            other => panic!("unexpected plan: {other:?}"),
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn refuses_oversized_before_launch() {
        let root = temp_dir("oversized");
        let source = root.join("large.jsonl");
        let file = File::create(&source).unwrap();
        file.set_len(2 * 1024 * 1024).unwrap();
        let launch = args(&["--session", source.to_str().unwrap()], 1);
        assert!(matches!(
            preflight(&launch, &root).unwrap(),
            LaunchPlan::MigrationRequired { bytes, cap, .. } if bytes == 2 * 1024 * 1024 && cap == 1024 * 1024
        ));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn continue_selects_latest_project_session() {
        let root = temp_dir("continue");
        let cwd = root.join("project");
        fs::create_dir_all(&cwd).unwrap();
        let project_sessions = root.join("sessions").join(encode_project_dir(&cwd));
        fs::create_dir_all(&project_sessions).unwrap();
        File::create(project_sessions.join("old.jsonl")).unwrap();
        sleep(Duration::from_millis(20));
        let latest = project_sessions.join("latest.jsonl");
        File::create(&latest).unwrap();
        let found = latest_project_session(&[root.join("sessions")], &cwd).unwrap();
        assert_eq!(found, latest);
        fs::remove_dir_all(root).unwrap();
    }
}
