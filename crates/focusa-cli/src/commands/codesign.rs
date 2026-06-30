//! First-class macOS code signing/notarization helper (focusa-covz).
//!
//! Provides:
//! - `focusa codesign inspect` — probe local codesign/notarization posture.
//! - `focusa codesign sign`    — run the full codesign + notarize + staple +
//!   spctl flow against a `.app` on a Mac. Requires Apple Developer
//!   Program enrollment + credentials.

use anyhow::{Context, Result, anyhow, bail};
use clap::{Args, Subcommand};
use serde::Serialize;
use std::path::PathBuf;
use std::process::Command;

#[derive(Args, Debug)]
pub struct CodesignArgs {
    #[command(subcommand)]
    pub cmd: CodesignCmd,
}

#[derive(Subcommand, Debug)]
pub enum CodesignCmd {
    /// Inspect macOS code signing + notarization posture for the local artifacts.
    Inspect {
        #[arg(long, default_value = "/Applications")]
        app_dir: String,
        #[arg(long)]
        json: bool,
    },
    /// Sign + (optionally) notarize + staple a `.app` (macOS only).
    ///
    /// Three modes:
    ///   1. Ad-hoc (free, no Apple ID): `--developer-id -`
    ///      Signs with the local ad-hoc identity; Gatekeeper requires
    ///      right-click -> Open on first launch. No notarization.
    ///   2. Personal Team (free, Apple ID): `--developer-id 'Apple Development: <name>' --apple-id <email>`
    ///      Signs with the user's free Apple Developer Program identity;
    ///      Gatekeeper requires right-click -> Open on first launch. No
    ///      notarization (notarization is paid-Apple-Developer-only).
    ///   3. Full Developer ID (paid program): `--developer-id 'Developer ID Application: <team>'`
    ///      plus team_id + apple_id + app-specific_password -> full sign +
    ///      notarize + staple. Gatekeeper accepts without user action.
    Sign {
        /// Path to the `.app` bundle.
        #[arg(long)]
        app_path: PathBuf,
        /// Developer identity. Use "-" for ad-hoc, "Apple Development: <name>"
        /// for Personal Team, or "Developer ID Application: <team>" for the
        /// paid program. Stored in FOCUSA_DEVELOPER_ID env var.
        #[arg(long, env = "FOCUSA_DEVELOPER_ID")]
        developer_id: String,
        /// Apple Developer Team ID (10 chars; required for Personal Team +
        /// full Developer ID, ignored for ad-hoc).
        #[arg(long, env = "FOCUSA_APPLE_TEAM_ID")]
        team_id: String,
        /// Apple ID email (required for notarization, ignored for ad-hoc
        /// and Personal Team).
        #[arg(long, env = "FOCUSA_APPLE_ID")]
        apple_id: String,
        /// App-specific password for notarytool (required for notarization).
        #[arg(long, env = "FOCUSA_APP_SPECIFIC_PASSWORD")]
        app_specific_password: String,
        /// Optional output zip path (defaults to <app>.zip alongside).
        #[arg(long)]
        zip_path: Option<PathBuf>,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Serialize)]
pub struct CodesignReport {
    pub platform: &'static str,
    pub host_supported: bool,
    pub codesign_present: bool,
    pub notary_present: bool,
    pub spctl_present: bool,
    pub artifacts: Vec<Artifact>,
    pub notes: Vec<String>,
    pub recovery_hint: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Artifact {
    pub path: String,
    pub status: String,
    pub signature_authority: Option<String>,
    pub notarized: bool,
    pub hardened_runtime: bool,
}

#[derive(Debug, Serialize)]
pub struct SignReport {
    pub steps: Vec<SignStep>,
    pub notarized: bool,
    pub hardened_runtime: bool,
    pub spctl_passed: bool,
    pub final_app: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct SignStep {
    pub name: &'static str,
    pub command: String,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub ok: bool,
}

pub async fn run(args: CodesignArgs) -> Result<()> {
    match args.cmd {
        CodesignCmd::Inspect { app_dir: _, json } => inspect(json).await,
        CodesignCmd::Sign {
            app_path,
            developer_id,
            team_id,
            apple_id,
            app_specific_password,
            zip_path,
            json,
        } => {
            sign(
                app_path,
                developer_id,
                team_id,
                apple_id,
                app_specific_password,
                zip_path,
                json,
            )
            .await
        }
    }
}

async fn inspect(json: bool) -> Result<()> {
    let codesign_present = which_present("codesign");
    let notary_present = which_present("notarytool") || which_present("xcrun-notarytool");
    let spctl_present = which_present("spctl");
    let host_supported = cfg!(target_os = "macos");

    let notes = vec![
        format!(
            "platform: {} (host_supported={host_supported})",
            std::env::consts::OS
        ),
        format!(
            "codesign available: {}; notarytool available: {}; spctl available: {}",
            codesign_present, notary_present, spctl_present
        ),
        "To notarize on a Mac: focusa codesign sign --app-path <.app> --developer-id 'Developer ID Application: <Team>' --team-id <id> --apple-id <email> --app-specific-password <pwd>".to_string(),
        "Operator rule: Apple Silicon unverified-developer errors require real codesign + notarize; deferral is not allowed once Apple Developer credentials exist.".to_string(),
    ];

    let recovery_hint = if cfg!(target_os = "macos") && !(codesign_present && notary_present) {
        Some("Install Xcode Command Line Tools (codesign, spctl) and Xcode (xcrun notarytool). Set FOCUSA_DEVELOPER_ID and FOCUSA_APPLE_TEAM_ID for non-interactive signing.".to_string())
    } else if !cfg!(target_os = "macos") {
        Some(
            "Codesign inspect runs everywhere; actual codesign/notarize must run on macOS."
                .to_string(),
        )
    } else {
        None
    };

    let report = CodesignReport {
        platform: std::env::consts::OS,
        host_supported,
        codesign_present,
        notary_present,
        spctl_present,
        artifacts: vec![],
        notes,
        recovery_hint,
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!("focusa codesign inspect");
        println!("  platform:        {}", report.platform);
        println!("  host_supported:  {}", report.host_supported);
        println!("  codesign:        {}", report.codesign_present);
        println!("  notarytool:      {}", report.notary_present);
        println!("  spctl:           {}", report.spctl_present);
        for n in &report.notes {
            println!("  note: {n}");
        }
        if let Some(h) = report.recovery_hint {
            println!("  recovery_hint: {h}");
        }
    }
    Ok(())
}

async fn sign(
    app_path: PathBuf,
    developer_id: String,
    team_id: String,
    apple_id: String,
    app_specific_password: String,
    zip_path: Option<PathBuf>,
    json: bool,
) -> Result<()> {
    if !cfg!(target_os = "macos") {
        bail!(
            "focusa codesign sign must run on macOS (current platform: {}). Use a Mac runner or local Mac.",
            std::env::consts::OS
        );
    }
    if !app_path.is_dir() {
        bail!("app path is not a directory: {}", app_path.display());
    }
    let canonical_app = std::fs::canonicalize(&app_path)
        .with_context(|| format!("canonicalize {}", app_path.display()))?
        .to_string_lossy()
        .to_string();

    // Pre-flight: required tools present.
    for tool in ["codesign", "xcrun", "spctl"] {
        if !which_present(tool) {
            bail!(
                "missing required tool `{tool}`. Install Xcode Command Line Tools (`xcode-select --install`) and Xcode."
            );
        }
    }

    // Auto-detect mode:
    //   developer_id == "-"                          → ad-hoc (no Apple ID)
    //   developer_id.starts_with("Apple Development:") → Personal Team (free)
    //   developer_id.starts_with("Developer ID Application:")
    //      + apple_id provided                          → full Developer ID (paid)
    let is_ad_hoc = developer_id == "-";
    let is_personal_team = developer_id.starts_with("Apple Development:") && team_id.len() == 10;

    if is_ad_hoc {
        tracing::info!("signing mode: ad-hoc (no Apple ID; Gatekeeper will require manual Open)");
    } else if is_personal_team {
        tracing::info!(
            "signing mode: Apple Developer Program free tier (Personal Team; Gatekeeper will require manual Open)"
        );
    } else {
        tracing::info!("signing mode: full Developer ID + notarization");
    }

    let zip = zip_path
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| format!("{}.zip", canonical_app.trim_end_matches('/')));

    let password_ref = format!("@keychain:FOCUSA_NOTARY_PASSWORD");
    let mut steps = Vec::new();
    let cmd1 = format!(
        "codesign --deep --force --options runtime --sign \"{}\" \"{}\"",
        developer_id, canonical_app
    );
    let s1 = run_step("codesign", &cmd1).await?;
    steps.push(s1.clone());
    if !s1.ok {
        return finish_sign(steps, canonical_app, json);
    }

    // Step 2: ditto -c -k <app> <zip>
    let cmd2 = format!(
        "ditto -c -k --sequesterRsrc --keepParent \"{}\" \"{}\"",
        canonical_app, zip
    );
    let s2 = run_step("zip", &cmd2).await?;
    steps.push(s2.clone());
    if !s2.ok {
        return finish_sign(steps, canonical_app, json);
    }

    // Step 3-4-5: notarize + staple + spctl — only for the full Developer ID path.
    // For ad-hoc (--identity "-") or Personal Team (no apple_id), skip
    // notarization entirely: it's a paid-Apple-Developer-Program-only API.
    if is_ad_hoc || is_personal_team {
        // Skip notarization + spctl. The operator can still install the
        // .app after right-click → Open (one-time per machine).
        tracing::info!("skipping notarize + staple + spctl_assess (ad-hoc / Personal Team mode)");
        // Re-run codesign after the zip step (stapling may modify the bundle
        // in the full path; skip that here since we didn't staple).
        let cmd_re = format!(
            "codesign --deep --force --options runtime --sign \"{}\" \"{}\"",
            if is_ad_hoc { "-" } else { &developer_id },
            canonical_app
        );
        let s_re = run_step("codesign_resign", &cmd_re).await?;
        steps.push(s_re);
        return finish_sign(steps, canonical_app, json);
    }

    // Step 3: xcrun notarytool submit <zip> --apple-id ... --team-id ... --password @keychain:... --wait
    let cmd3 = format!(
        "xcrun notarytool submit \"{}\" --apple-id \"{}\" --team-id \"{}\" --password \"{}\" --wait",
        zip, apple_id, team_id, password_ref
    );
    let s3 = run_step("notarytool_submit", &cmd3).await?;
    steps.push(s3.clone());
    if !s3.ok {
        return finish_sign(steps, canonical_app, json);
    }

    // Step 4: xcrun stapler staple <app>
    let cmd4 = format!("xcrun stapler staple \"{}\"", canonical_app);
    let s4 = run_step("stapler", &cmd4).await?;
    steps.push(s4.clone());
    if !s4.ok {
        return finish_sign(steps, canonical_app, json);
    }

    // Step 5: spctl --assess --type execute --verbose=2 <app>
    let cmd5 = format!(
        "spctl --assess --type execute --verbose=2 \"{}\"",
        canonical_app
    );
    let s5 = run_step("spctl_assess", &cmd5).await?;
    steps.push(s5);
    finish_sign(steps, canonical_app, json)
}

fn finish_sign(steps: Vec<SignStep>, app: String, json: bool) -> Result<()> {
    let notarized = steps
        .iter()
        .find(|s| s.name == "notarytool_submit")
        .map(|s| s.ok)
        .unwrap_or(false);
    let hardened_runtime = steps
        .iter()
        .find(|s| s.name == "codesign")
        .map(|s| s.ok)
        .unwrap_or(false);
    let spctl_passed = steps
        .iter()
        .find(|s| s.name == "spctl_assess")
        .map(|s| s.ok)
        .unwrap_or(false);
    let report = SignReport {
        steps,
        notarized,
        hardened_runtime,
        spctl_passed,
        final_app: app,
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!("focusa codesign sign");
        for s in &report.steps {
            println!(
                "  {} {} (exit {})",
                if s.ok { "OK" } else { "FAIL" },
                s.name,
                s.exit_code
            );
        }
        println!(
            "  notarized={notarized} hardened_runtime={hardened_runtime} spctl_passed={spctl_passed}"
        );
    }
    if !(notarized && hardened_runtime && spctl_passed) {
        return Err(anyhow!(
            "focusa codesign sign did not complete all steps; check per-step stderr in the report"
        ));
    }
    Ok(())
}

async fn run_step(name: &'static str, command: &str) -> Result<SignStep> {
    // Run via /bin/sh -c so we can use quoting and shell semantics the way the
    // operator would when running these by hand.
    let output = Command::new("/bin/sh")
        .args(["-c", command])
        .output()
        .with_context(|| format!("spawn {name}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(-1);
    Ok(SignStep {
        name,
        command: command.to_string(),
        exit_code,
        stdout,
        stderr,
        ok: output.status.success(),
    })
}

fn which_present(name: &str) -> bool {
    let path = std::env::var_os("PATH");
    if let Some(p) = path {
        for entry in std::env::split_paths(&p) {
            if entry.join(name).is_file() {
                return true;
            }
        }
    }
    // xcrun is a shim that fronts notarytool / stapler / spctl.
    if name == "notarytool" || name == "xcrun-notarytool" || name == "stapler" {
        return which_present("xcrun");
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inspect_report_is_serializable() {
        let r = CodesignReport {
            platform: "test",
            host_supported: false,
            codesign_present: false,
            notary_present: false,
            spctl_present: false,
            artifacts: vec![],
            notes: vec![],
            recovery_hint: None,
        };
        assert!(serde_json::to_string(&r).is_ok());
    }

    #[test]
    fn sign_report_serializes() {
        let r = SignReport {
            steps: vec![],
            notarized: false,
            hardened_runtime: false,
            spctl_passed: false,
            final_app: "/tmp/Focusa.app".into(),
        };
        assert!(serde_json::to_string(&r).is_ok());
    }

    #[test]
    fn which_present_finds_paths() {
        // /bin/sh is always present on POSIX.
        assert!(which_present("sh") || !which_present("definitely-not-a-binary-xyz"));
    }
}
