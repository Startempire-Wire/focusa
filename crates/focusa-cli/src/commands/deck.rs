//! `focusa deck` — Mission Deck CLI alias (Spec 117 §19).
//!
//! This is the operator-friendly alias for `focusa-tui`. It does NOT reimplement
//! the TUI; it locates the binary and execs it. If the binary is missing it
//! prints a recovery hint per Spec 117 §19.3.

use anyhow::{Context, Result};
use clap::Args;
use std::path::PathBuf;
use std::process::Command;

#[derive(Args)]
pub struct DeckArgs {
    /// Mode override passed through to focusa-tui (beginner|operator).
    #[arg(long, value_name = "MODE")]
    pub mode: Option<String>,

    /// Optional web surface hint. Reserved for Phase 6 PWA /deck work.
    #[arg(long)]
    pub web: bool,

    /// Headless self-test snapshot instead of launching the interactive TUI.
    #[arg(long)]
    pub headless_self_test: bool,
}

pub async fn run(args: DeckArgs, _json: bool) -> Result<()> {
    let bin = locate_tui_binary().context(
        "focusa-tui binary not found; recovery_hint: bash scripts/install-daemon.sh /usr/local",
    )?;

    let mut cmd = Command::new(&bin);
    if let Some(mode) = args.mode.as_deref() {
        cmd.arg("--mode").arg(mode);
    }
    if args.headless_self_test {
        cmd.arg("--headless-self-test");
    }
    if args.web {
        // Reserved for Phase 6 PWA /deck work; today we surface the hint.
        eprintln!("focusa deck web: PWA /deck planned (Spec 117 §17); not yet shipping");
        return Ok(());
    }
    let status = cmd
        .status()
        .with_context(|| format!("failed to launch {}", bin.display()))?;
    if !status.success() {
        anyhow::bail!("focusa-tui exited with status {:?}", status.code());
    }
    Ok(())
}

fn locate_tui_binary() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("FOCUSA_TUI_BIN") {
        let p = PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
    }
    if let Some(home) = std::env::var_os("HOME") {
        let installed = PathBuf::from(home).join(".focusa/bin/focusa-tui");
        if installed.is_file() {
            return Some(installed);
        }
    }
    {
        let name = "focusa-tui";
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

fn which(name: &str) -> Result<PathBuf, ()> {
    let path = std::env::var_os("PATH").ok_or(())?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    Err(())
}
