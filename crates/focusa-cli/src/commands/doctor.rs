//! Agent-first doctor command — Spec92 §9.

use crate::api_client::ApiClient;
use serde_json::{Value, json};
use std::path::Path;

fn fs_check(name: &str, path: &str) -> Value {
    let exists = Path::new(path).exists();
    json!({
        "name": name,
        "status": if exists { "completed" } else { "blocked" },
        "path": path,
        "what_failed": if exists { Value::Null } else { json!("required file/path missing") },
        "safe_recovery": if exists { Value::Null } else { json!(format!("restore or generate {path}")) },
    })
}

async fn api_check(api: &ApiClient, name: &str, path: &str) -> Value {
    match api.get(path).await {
        Ok(resp) => json!({
            "name": name,
            "status": "completed",
            "path": path,
            "details": resp,
        }),
        Err(err) => json!({
            "name": name,
            "status": "blocked",
            "path": path,
            "what_failed": name,
            "likely_why": err.to_string(),
            "safe_recovery": "systemctl status focusa-daemon && journalctl -u focusa-daemon -n 80 --no-pager",
            "command": format!("curl -sS {}{} | jq .", api.base_url(), path),
            "fallback": "focusa start",
            "docs": ["docs/current/TROUBLESHOOTING_CURRENT.md"],
            "evidence_refs": [],
            "severity": "blocked",
        }),
    }
}

fn status_rank(status: &str) -> u8 {
    match status {
        "blocked" => 3,
        "degraded" => 2,
        "watch" => 1,
        _ => 0,
    }
}

pub async fn run(json_mode: bool) -> anyhow::Result<()> {
    let api = ApiClient::new();
    let mut checks = Vec::new();

    checks.push(api_check(&api, "daemon health", "/v1/health").await);
    checks.push(api_check(&api, "command-center doctor API", "/v1/doctor").await);
    checks.push(api_check(&api, "API route inventory surface", "/v1/agents").await);
    checks.push(api_check(&api, "Spec90 tool contracts", "/v1/ontology/tool-contracts").await);
    checks.push(api_check(&api, "Workpoint canonicality", "/v1/workpoint/current").await);
    checks.push(api_check(&api, "Work-loop writer state", "/v1/work-loop/status").await);
    checks.push(
        api_check(
            &api,
            "token telemetry status",
            "/v1/telemetry/token-budget/status?limit=20",
        )
        .await,
    );
    checks.push(
        api_check(
            &api,
            "cache metadata status",
            "/v1/telemetry/cache-metadata/status?limit=20",
        )
        .await,
    );

    checks.push(fs_check(
        "daemon exe path",
        "/home/wirebot/focusa/target/release/focusa-daemon",
    ));
    checks.push(fs_check(
        "Spec91 live proof harness",
        "scripts/prove-focusa-tool-contracts-live.mjs",
    ));
    checks.push(fs_check(
        "Spec90 contract validator",
        "scripts/validate-focusa-tool-contracts.mjs",
    ));
    checks.push(fs_check("Pi extension skills", "apps/pi-extension/skills"));
    checks.push(fs_check("root Pi skills", "/root/.pi/skills"));
    checks.push(fs_check("Mac app package", "apps/menubar/package.json"));
    checks.push(fs_check(
        "release command docs",
        "docs/current/PRODUCTION_RELEASE_COMMANDS.md",
    ));
    checks.push(fs_check("Guardian scanner", "/usr/local/bin/guardian"));

    let worst = checks
        .iter()
        .filter_map(|c| c.get("status").and_then(|v| v.as_str()))
        .max_by_key(|s| status_rank(s))
        .unwrap_or("completed");
    let blocked = checks
        .iter()
        .filter(|c| c.get("status").and_then(|v| v.as_str()) == Some("blocked"))
        .count();
    let status = if blocked > 0 { "blocked" } else { worst };
    let response = json!({
        "status": status,
        "summary": if blocked > 0 { format!("{blocked} doctor check(s) blocked") } else { "All required doctor checks completed".to_string() },
        "next_action": if blocked > 0 { "Run the recovery command for the first blocked check, then re-run focusa doctor" } else { "Continue with focusa continue or focusa release prove --tag <tag> when ready" },
        "why": "Spec92 requires one agent-first command center covering health, contracts, workpoint/work-loop, telemetry, cache, Pi skills, Mac app, release proof, Guardian, and cleanup readiness.",
        "commands": [
            "focusa doctor",
            "focusa telemetry token-budget",
            "focusa cache doctor",
            "node scripts/validate-focusa-tool-contracts.mjs",
            "node scripts/prove-focusa-tool-contracts-live.mjs --safe-fixtures"
        ],
        "recovery": ["focusa start", "systemctl status focusa-daemon", "journalctl -u focusa-daemon -n 80 --no-pager"],
        "evidence_refs": ["docs/current/EFFICIENCY_GUIDE.md", "docs/current/HOOK_COVERAGE.md", "docs/current/VALIDATION_AND_RELEASE_PROOF.md"],
        "docs": ["docs/92-agent-first-polish-hooks-efficiency-spec.md", "docs/current/DOCTOR_CONTINUE_RELEASE_PROVE.md"],
        "warnings": [],
        "details": { "checks": checks }
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
                .unwrap_or("doctor summary unavailable")
        );
        println!(
            "Next action: {}",
            response["next_action"]
                .as_str()
                .unwrap_or("re-run focusa doctor")
        );
        println!(
            "Why: {}",
            response["why"].as_str().unwrap_or("Spec92 doctor")
        );
        println!("Command: focusa doctor");
        println!("Recovery: focusa start && systemctl status focusa-daemon");
        println!("Evidence: docs/current/EFFICIENCY_GUIDE.md, docs/current/HOOK_COVERAGE.md");
        println!("Docs: docs/92-agent-first-polish-hooks-efficiency-spec.md");
    }
    Ok(())
}
