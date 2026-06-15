use crate::api_client::ApiClient;
use clap::Subcommand;
use serde_json::Value;
use std::process::Command;

#[derive(Subcommand)]
pub enum DxuxCmd {
    /// Read Spec105 DX/UX implementation report.
    Report,
    /// Read one DXUX requirement by id, e.g. DXUX-004.
    Requirement { id: String },
    /// Read compact continuation/doability digest.
    Digest,
}

pub async fn handle(client: &mut ApiClient, cmd: DxuxCmd) -> anyhow::Result<()> {
    match cmd {
        DxuxCmd::Report => print_report(&client.get("/v1/dxux/report").await?),
        DxuxCmd::Requirement { id } => {
            print_requirement(&client.get(&format!("/v1/dxux/requirement/{id}")).await?)
        }
        DxuxCmd::Digest => println!(
            "{}",
            serde_json::to_string_pretty(&client.get("/v1/dxux/digest").await?)?
        ),
    }
    Ok(())
}

pub async fn explain(failure: String) -> anyhow::Result<()> {
    let client = ApiClient::new();
    let encoded = failure.replace('/', "%2F");
    let payload = client.get(&format!("/v1/dxux/explain/{encoded}")).await?;
    let summary = payload
        .get("root_cause_summary")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let confidence = payload
        .get("confidence")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    println!("explain completed | confidence={confidence}");
    println!("root_cause: {summary}");
    if let Some(commands) = payload.get("recovery_commands").and_then(Value::as_array) {
        for command in commands {
            if let Some(text) = command.as_str() {
                println!("  recover: {text}");
            }
        }
    }
    Ok(())
}

/// CLI usage: `focusa preflight`.
pub async fn preflight() -> anyhow::Result<()> {
    let commands = [
        "cargo test --workspace",
        "cargo clippy --workspace -- -D warnings",
        "node scripts/validate-focusa-tool-contracts.mjs",
        "python3 tests/spec101_bloatgaurd_budgets_static_test.py",
        "scripts/enforce_bd_closure_evidence.sh",
    ];
    println!("preflight started | commands={}", commands.len());
    for command in commands {
        println!("preflight running: {command}");
        let status = Command::new("bash").arg("-lc").arg(command).status()?;
        if !status.success() {
            anyhow::bail!("preflight failed: {command} exit={status}");
        }
    }
    println!("preflight completed | status=ok");
    Ok(())
}

fn print_report(payload: &Value) {
    let status = payload
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let count = payload
        .get("requirements")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);
    println!("dxux report {status} | requirements={count}");
    if let Some(requirements) = payload.get("requirements").and_then(Value::as_array) {
        for req in requirements.iter().take(12) {
            let id = req.get("id").and_then(Value::as_str).unwrap_or("unknown");
            let title = req
                .get("title")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            println!("  {id}: {title}");
        }
    }
}

fn print_requirement(payload: &Value) {
    let status = payload
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    if let Some(req) = payload.get("requirement") {
        let id = req.get("id").and_then(Value::as_str).unwrap_or("unknown");
        let title = req
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        println!("dxux requirement {status} | {id}: {title}");
    } else {
        println!("dxux requirement {status}");
    }
}
