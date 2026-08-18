//! Release proof orchestration — Spec92 §9.

#[path = "release_master.rs"]
mod release_master;

use clap::{Subcommand, ValueEnum};
use focusa_core::license::require_release_proof;
use focusa_core::release_cycle::ReleaseTopology;
use focusa_core::release_intelligence::ReleaseIntelligencePacket;
use focusa_core::release_orchestrator::ReleaseInvocationSurface;
use focusa_core::types::default_focusa_data_dir;
use serde_json::{Value, json};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Subcommand, Debug)]
pub enum ReleaseCmd {
    /// Operate the provider-neutral canonical release cycle.
    Cycle {
        #[command(subcommand)]
        action: ReleaseCycleCmd,
    },
    /// Backward-compatible alias for the canonical release workflow's page renderer.
    RenderIntelligence {
        /// Release intelligence packet JSON path.
        #[arg(long)]
        packet: PathBuf,
        /// Markdown output path.
        #[arg(long)]
        output: PathBuf,
        /// Require every check and artifact proof needed for publication.
        #[arg(long)]
        publishable: bool,
    },
    /// Prove the current checkout/release tag with the standard safe gate set.
    Prove {
        /// Release tag to verify, for example v0.9.10-dev.
        #[arg(long)]
        tag: String,

        /// Include GitHub release lookup via gh release view.
        #[arg(long)]
        github: bool,

        /// Skip slower cargo clippy/test gates.
        #[arg(long)]
        fast: bool,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum ReleaseSurfaceArg {
    Canvas,
    Terminal,
    Headless,
}

impl From<ReleaseSurfaceArg> for ReleaseInvocationSurface {
    fn from(value: ReleaseSurfaceArg) -> Self {
        match value {
            ReleaseSurfaceArg::Canvas => Self::Canvas,
            ReleaseSurfaceArg::Terminal => Self::Terminal,
            ReleaseSurfaceArg::Headless => Self::Headless,
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum ReleaseCycleCmd {
    /// Validate a release topology before candidate lock/tagging.
    ValidateTopology {
        /// Release topology JSON path.
        #[arg(long)]
        path: PathBuf,
    },
    /// Validate a pluggable adapter manifest against its topology.
    ValidateAdapter {
        #[arg(long)]
        manifest: PathBuf,
        #[arg(long)]
        topology: PathBuf,
    },
    /// Render one canonical plan for Canvas, terminal, or headless execution.
    Plan {
        #[arg(long)]
        manifest: PathBuf,
        #[arg(long)]
        topology: PathBuf,
        #[arg(long)]
        candidate: PathBuf,
        #[arg(long)]
        tuning: Option<PathBuf>,
        #[arg(long, value_enum, default_value = "terminal")]
        surface: ReleaseSurfaceArg,
    },
    /// Execute through an external typed JSON plugin; no shell interpolation.
    Execute {
        #[arg(long)]
        manifest: PathBuf,
        #[arg(long)]
        topology: PathBuf,
        #[arg(long)]
        candidate: PathBuf,
        #[arg(long)]
        tuning: Option<PathBuf>,
        #[arg(long)]
        plugin: PathBuf,
        /// Absolute append-only checkpoint ledger; existing exact candidate resumes.
        #[arg(long)]
        ledger: PathBuf,
        #[arg(long, value_enum, default_value = "headless")]
        surface: ReleaseSurfaceArg,
        #[arg(long)]
        yes: bool,
        #[arg(long)]
        allow_mutations: bool,
        #[arg(long = "approval-ref", required = true)]
        approval_refs: Vec<String>,
    },
    /// Append a benchmark and produce evidence-backed tuning for the next cycle.
    Calibrate {
        #[arg(long)]
        ledger: PathBuf,
        #[arg(long)]
        observation: PathBuf,
        #[arg(long)]
        active_tuning: Option<PathBuf>,
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Render a deterministic evidence-backed release page from a typed packet.
    RenderIntelligence {
        /// Release intelligence packet JSON path.
        #[arg(long)]
        packet: PathBuf,
        /// Markdown output path.
        #[arg(long)]
        output: PathBuf,
        /// Require every check and artifact proof needed for publication.
        #[arg(long)]
        publishable: bool,
    },
}

fn release_proof_dir() -> PathBuf {
    let configured = std::env::var("FOCUSA_DATA_DIR").unwrap_or_else(|_| default_focusa_data_dir());
    let expanded = if configured == "~" {
        std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(configured))
    } else if let Some(rest) = configured.strip_prefix("~/") {
        std::env::var("HOME")
            .map(|home| PathBuf::from(home).join(rest))
            .unwrap_or_else(|_| PathBuf::from(configured))
    } else {
        PathBuf::from(configured)
    };
    expanded.join("release-proof")
}

fn release_proof_file_stem(tag: &str) -> String {
    tag.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_') {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn persist_release_proof(tag: &str, response: &mut Value) -> anyhow::Result<(String, String)> {
    let dir = release_proof_dir();
    fs::create_dir_all(&dir)?;
    let tag_path = dir.join(format!("{}.json", release_proof_file_stem(tag)));
    let latest_path = dir.join("latest.json");
    let tag_path_display = tag_path.display().to_string();
    let latest_path_display = latest_path.display().to_string();

    response["proof_artifact"] = json!({
        "tag_path": tag_path_display,
        "latest_path": latest_path_display,
    });

    let body = serde_json::to_string_pretty(response)?;
    fs::write(&tag_path, &body)?;
    fs::write(&latest_path, body)?;
    Ok((
        tag_path.display().to_string(),
        latest_path.display().to_string(),
    ))
}

fn run_gate(name: &str, command: &str) -> Value {
    let output = Command::new("bash").arg("-lc").arg(command).output();
    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            let mut combined = format!("{}{}", stdout, stderr);
            if combined.len() > 4000 {
                combined = combined[combined.len().saturating_sub(4000)..].to_string();
            }
            json!({
                "name": name,
                "command": command,
                "status": if out.status.success() { "completed" } else { "blocked" },
                "exit_code": out.status.code(),
                "output_tail": combined,
            })
        }
        Err(err) => json!({
            "name": name,
            "command": command,
            "status": "blocked",
            "what_failed": "failed to spawn proof command",
            "likely_why": err.to_string(),
            "safe_recovery": "run the command manually from the Focusa repository root",
            "severity": "blocked",
        }),
    }
}

fn render_intelligence(
    packet: PathBuf,
    output: PathBuf,
    publishable: bool,
    json_mode: bool,
) -> anyhow::Result<()> {
    let body = fs::read_to_string(&packet)?;
    let intelligence: ReleaseIntelligencePacket = serde_json::from_str(&body)?;
    intelligence.validate(publishable)?;
    let markdown = intelligence.render_markdown()?;
    anyhow::ensure!(
        !output.exists(),
        "release intelligence output already exists; immutable release pages are never overwritten"
    );
    let parent = output.parent().unwrap_or_else(|| std::path::Path::new("."));
    anyhow::ensure!(
        parent.is_dir(),
        "release intelligence output parent does not exist"
    );
    let staged = parent.join(format!(".release-intelligence-{}.tmp", std::process::id()));
    fs::write(&staged, markdown)?;
    fs::rename(&staged, &output)?;
    let result = json!({
        "schema": "focusa.release_intelligence_render.v1",
        "status": "completed",
        "release_id": intelligence.release_id,
        "version": intelligence.version,
        "exact_sha": intelligence.exact_sha,
        "publishable": publishable,
        "output": output,
        "artifact_count": intelligence.artifacts.len(),
        "proof_count": intelligence.exact_proofs.len()
    });
    if json_mode {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("release intelligence rendered: {}", result["output"]);
    }
    Ok(())
}

pub async fn run(cmd: ReleaseCmd, json_mode: bool) -> anyhow::Result<()> {
    match cmd {
        ReleaseCmd::Cycle { action } => match action {
            ReleaseCycleCmd::ValidateTopology { path } => {
                let body = fs::read_to_string(&path)?;
                let topology: ReleaseTopology = serde_json::from_str(&body)?;
                topology.validate()?;
                let output = json!({
                    "schema": "focusa.release_topology_validation.v1",
                    "status": "completed",
                    "valid": true,
                    "path": path,
                    "project_id": &topology.project_id,
                    "profile": &topology.profile,
                    "provider": &topology.provider,
                    "surface_count": topology.surfaces.len(),
                    "surface_ids": topology.surfaces.iter().map(|surface| &surface.surface_id).collect::<Vec<_>>(),
                    "global_gates": &topology.global_gates,
                    "next_action": "Create and lock an exact-SHA ReleaseCandidate before provider execution"
                });
                if json_mode {
                    println!("{}", serde_json::to_string_pretty(&output)?);
                } else {
                    println!(
                        "release topology valid: {} surfaces",
                        topology.surfaces.len()
                    );
                    println!("project: {}", topology.project_id);
                    println!("provider: {}", topology.provider);
                }
            }
            ReleaseCycleCmd::ValidateAdapter { manifest, topology } => {
                release_master::validate_adapter(manifest, topology)?;
            }
            ReleaseCycleCmd::Plan {
                manifest,
                topology,
                candidate,
                tuning,
                surface,
            } => {
                release_master::plan(manifest, topology, candidate, tuning, surface)?;
            }
            ReleaseCycleCmd::Execute {
                manifest,
                topology,
                candidate,
                tuning,
                plugin,
                ledger,
                surface,
                yes,
                allow_mutations,
                approval_refs,
            } => {
                release_master::execute(
                    manifest,
                    topology,
                    candidate,
                    tuning,
                    plugin,
                    ledger,
                    surface,
                    yes,
                    allow_mutations,
                    approval_refs,
                )
                .await?;
            }
            ReleaseCycleCmd::Calibrate {
                ledger,
                observation,
                active_tuning,
                output,
            } => {
                release_master::calibrate(ledger, observation, active_tuning, output)?;
            }
            ReleaseCycleCmd::RenderIntelligence {
                packet,
                output,
                publishable,
            } => render_intelligence(packet, output, publishable, json_mode)?,
        },
        ReleaseCmd::RenderIntelligence {
            packet,
            output,
            publishable,
        } => render_intelligence(packet, output, publishable, json_mode)?,
        ReleaseCmd::Prove { tag, github, fast } => {
            // Spec 152F §3, §4, §6: release proof orchestration requires the
            // release_proof premium family grant. Safe status reads remain
            // available through the ReadProjection family.
            if let Err(error) = require_release_proof() {
                anyhow::bail!("{error}");
            }
            let mut gates = vec![
                ("git status", "git status --short".to_string()),
                ("Spec90 contract validation", "node scripts/validate-focusa-tool-contracts.mjs".to_string()),
                ("Spec91 live safe fixtures", "node scripts/prove-focusa-tool-contracts-live.mjs --safe-fixtures".to_string()),
                ("work-loop auto-continue wiring", "./tests/work_loop_autocontinue_wiring_test.sh".to_string()),
                ("daemon health", "curl -fsS http://127.0.0.1:8787/v1/health | jq .ok".to_string()),
                ("Guardian docs scan", "guardian scan docs/current && guardian scan README.md && guardian scan CHANGELOG.md".to_string()),
            ];
            if !fast {
                gates.push(("cargo check", "cargo check --workspace".to_string()));
                gates.push((
                    "cargo clippy",
                    "cargo clippy --workspace -- -D warnings".to_string(),
                ));
                gates.push(("cargo test", "cargo test --workspace".to_string()));
            }
            if github {
                gates.push(("GitHub release", format!("gh release view {tag} --json name,tagName,isDraft,isPrerelease,url,assets | jq '{{tagName,name,isDraft,isPrerelease,url,assets:[.assets[].name]}}'")));
            }

            let results: Vec<Value> = gates
                .iter()
                .map(|(name, command)| run_gate(name, command))
                .collect();
            let blocked = results
                .iter()
                .filter(|r| r.get("status").and_then(|v| v.as_str()) == Some("blocked"))
                .count();
            let mut response = json!({
                "status": if blocked == 0 { "completed" } else { "blocked" },
                "summary": if blocked == 0 { format!("Release proof passed for {tag}") } else { format!("Release proof blocked for {tag}: {blocked} gate(s) failed") },
                "next_action": if blocked == 0 { format!("If publishing, create/push tag {tag} and verify GitHub release assets") } else { "Fix the first blocked gate, then rerun focusa release prove --tag <tag>".to_string() },
                "why": "Spec92 requires one command that orchestrates validation, live safe proof, Guardian scan, and release evidence before publication.",
                "commands": ["focusa release prove --tag <tag>", "focusa release prove --tag <tag> --github", "gh release view <tag> --json name,tagName,isDraft,isPrerelease,url,assets"],
                "recovery": ["focusa doctor", "focusa start", "node scripts/prove-focusa-tool-contracts-live.mjs --safe-fixtures", "journalctl -u focusa-daemon -n 80 --no-pager (Linux service installs)"],
                "evidence_refs": ["docs/current/VALIDATION_AND_RELEASE_PROOF.md", "docs/current/PRODUCTION_RELEASE_COMMANDS.md"],
                "docs": ["docs/92-agent-first-polish-hooks-efficiency-spec.md", "docs/current/DOCTOR_CONTINUE_RELEASE_PROVE.md"],
                "warnings": if fast { vec!["fast mode skipped cargo check/clippy/test"] } else { Vec::<&str>::new() },
                "details": { "tag": tag, "gates": results },
            });

            match persist_release_proof(&tag, &mut response) {
                Ok((_tag_path, _latest_path)) => {}
                Err(err) => {
                    if let Some(warnings) = response["warnings"].as_array_mut() {
                        warnings.push(json!(format!(
                            "failed to persist release proof artifact: {err}"
                        )));
                    }
                }
            }

            if json_mode {
                println!("{}", serde_json::to_string_pretty(&response)?);
            } else {
                println!(
                    "Status: {}",
                    response["status"].as_str().unwrap_or("blocked")
                );
                println!(
                    "Summary: {}",
                    response["summary"]
                        .as_str()
                        .unwrap_or("release proof complete")
                );
                println!(
                    "Next action: {}",
                    response["next_action"]
                        .as_str()
                        .unwrap_or("rerun focusa release prove")
                );
                println!(
                    "Why: {}",
                    response["why"].as_str().unwrap_or("Spec92 release proof")
                );
                println!("Command: focusa release prove --tag <tag> --github");
                println!(
                    "Recovery: focusa doctor && node scripts/prove-focusa-tool-contracts-live.mjs --safe-fixtures"
                );
                println!(
                    "Evidence: docs/current/VALIDATION_AND_RELEASE_PROOF.md, docs/current/PRODUCTION_RELEASE_COMMANDS.md"
                );
                if let Some(path) = response
                    .get("proof_artifact")
                    .and_then(|artifact| artifact.get("latest_path"))
                    .and_then(|v| v.as_str())
                {
                    println!("Proof artifact: {path}");
                }
                println!("Docs: docs/current/DOCTOR_CONTINUE_RELEASE_PROVE.md");
            }
        }
    }
    Ok(())
}
