//! Release proof orchestration — Spec92 §9.

use clap::Subcommand;
use serde_json::{Value, json};
use std::process::Command;

#[derive(Subcommand, Debug)]
pub enum ReleaseCmd {
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
            "safe_recovery": "run the command manually from /home/wirebot/focusa",
            "severity": "blocked",
        }),
    }
}

pub async fn run(cmd: ReleaseCmd, json_mode: bool) -> anyhow::Result<()> {
    match cmd {
        ReleaseCmd::Prove { tag, github, fast } => {
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
            let response = json!({
                "status": if blocked == 0 { "completed" } else { "blocked" },
                "summary": if blocked == 0 { format!("Release proof passed for {tag}") } else { format!("Release proof blocked for {tag}: {blocked} gate(s) failed") },
                "next_action": if blocked == 0 { format!("If publishing, create/push tag {tag} and verify GitHub release assets") } else { "Fix the first blocked gate, then rerun focusa release prove --tag <tag>".to_string() },
                "why": "Spec92 requires one command that orchestrates validation, live safe proof, Guardian scan, and release evidence before publication.",
                "commands": ["focusa release prove --tag <tag>", "focusa release prove --tag <tag> --github", "gh release view <tag> --json name,tagName,isDraft,isPrerelease,url,assets"],
                "recovery": ["focusa doctor", "node scripts/prove-focusa-tool-contracts-live.mjs --safe-fixtures", "journalctl -u focusa-daemon -n 80 --no-pager"],
                "evidence_refs": ["docs/current/VALIDATION_AND_RELEASE_PROOF.md", "docs/current/PRODUCTION_RELEASE_COMMANDS.md"],
                "docs": ["docs/92-agent-first-polish-hooks-efficiency-spec.md", "docs/current/DOCTOR_CONTINUE_RELEASE_PROVE.md"],
                "warnings": if fast { vec!["fast mode skipped cargo check/clippy/test"] } else { Vec::<&str>::new() },
                "details": { "tag": tag, "gates": results },
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
                println!("Docs: docs/current/DOCTOR_CONTINUE_RELEASE_PROVE.md");
            }
        }
    }
    Ok(())
}
