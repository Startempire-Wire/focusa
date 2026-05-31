//! Agent-first doctor command — Spec92 §9.

use crate::api_client::ApiClient;
use clap::Subcommand;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

#[derive(clap::Args, Debug, Default)]
pub struct DoctorArgs {
    #[command(subcommand)]
    pub command: Option<DoctorCommand>,
}

#[derive(Subcommand, Debug)]
pub enum DoctorCommand {
    /// Show API/resource security posture: auth, body limits, mutation rate limits, and JSON shape guard.
    Security,
}

fn env_usize(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|value| value.trim().parse::<usize>().ok())
        .unwrap_or(default)
}

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .unwrap_or(default)
}

fn env_u32(name: &str, default: u32) -> u32 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.trim().parse::<u32>().ok())
        .unwrap_or(default)
}

fn configured_bind() -> String {
    std::env::var("FOCUSA_BIND").unwrap_or_else(|_| "127.0.0.1:8787".to_string())
}

fn bind_looks_loopback(bind: &str) -> bool {
    bind.starts_with("127.")
        || bind.starts_with("localhost:")
        || bind.starts_with("[::1]")
        || bind.starts_with("::1")
}

fn auth_token_configured() -> bool {
    std::env::var("FOCUSA_AUTH_TOKEN")
        .map(|token| !token.trim().is_empty())
        .unwrap_or(false)
}

fn doc_contains(path: &str, needle: &str) -> bool {
    let candidates = [
        PathBuf::from(path),
        repo_root().join(path),
        PathBuf::from("/home/wirebot/focusa").join(path),
    ];
    candidates.iter().any(|candidate| {
        std::fs::read_to_string(candidate)
            .map(|content| content.contains(needle))
            .unwrap_or(false)
    })
}

fn security_posture_payload() -> Value {
    let bind = configured_bind();
    let bind_loopback = bind_looks_loopback(&bind);
    let auth_configured = auth_token_configured();
    let mutation_rate_limit = env_u32("FOCUSA_API_MUTATION_RATE_LIMIT_PER_WINDOW", 120);
    let rate_limit_enabled = mutation_rate_limit > 0;
    let reverse_proxy_doc_present = doc_contains(
        "docs/current/API_RESOURCE_LIMITS.md",
        "Reverse-proxy rate-limit guidance",
    );
    let status = if !bind_loopback && !auth_configured {
        "blocked"
    } else if !rate_limit_enabled || !reverse_proxy_doc_present {
        "degraded"
    } else {
        "ok"
    };
    json!({
        "status": status,
        "summary": if status == "ok" { "Security posture checks passed" } else { "Security posture needs attention" },
        "checks": {
            "bind_auth_boundary": {
                "status": if bind_loopback || auth_configured { "ok" } else { "blocked" },
                "bind": bind,
                "loopback": bind_loopback,
                "auth_token_configured": auth_configured,
                "requirement": "non-loopback bind requires FOCUSA_AUTH_TOKEN",
            },
            "request_body_limit": {
                "status": "ok",
                "env": "FOCUSA_API_MAX_BODY_BYTES",
                "bytes": env_usize("FOCUSA_API_MAX_BODY_BYTES", 1_048_576),
            },
            "mutation_rate_limit": {
                "status": if rate_limit_enabled { "ok" } else { "degraded" },
                "per_window": mutation_rate_limit,
                "window_ms": env_u64("FOCUSA_API_MUTATION_RATE_LIMIT_WINDOW_MS", 1_000),
                "env": ["FOCUSA_API_MUTATION_RATE_LIMIT_PER_WINDOW", "FOCUSA_API_MUTATION_RATE_LIMIT_WINDOW_MS"],
            },
            "json_shape_guard": {
                "status": "ok",
                "max_depth": env_usize("FOCUSA_API_JSON_MAX_DEPTH", 64),
                "max_array_items": env_usize("FOCUSA_API_JSON_MAX_ARRAY_ITEMS", 2_048),
                "max_object_fields": env_usize("FOCUSA_API_JSON_MAX_OBJECT_FIELDS", 2_048),
                "env": ["FOCUSA_API_JSON_MAX_DEPTH", "FOCUSA_API_JSON_MAX_ARRAY_ITEMS", "FOCUSA_API_JSON_MAX_OBJECT_FIELDS"],
            },
            "reverse_proxy_guidance": {
                "status": if reverse_proxy_doc_present { "ok" } else { "degraded" },
                "doc": "docs/current/API_RESOURCE_LIMITS.md#reverse-proxy-rate-limit-guidance",
            }
        },
        "next_action": if status == "blocked" { "bind to loopback or set FOCUSA_AUTH_TOKEN before non-loopback exposure" } else if status == "degraded" { "enable mutation rate limits and keep reverse-proxy guidance current" } else { "continue normally; keep Focusa loopback/Tailscale-first unless intentionally exposing it" },
        "docs": ["docs/current/API_RESOURCE_LIMITS.md", "docs/current/DAEMON_RESILIENCE.md", "docs/current/DYNAMIC_API_SECURITY_SMOKE.md"],
        "commands": ["focusa doctor security", "focusa --json doctor security", "focusa doctor"],
    })
}

fn print_security_posture(response: &Value) {
    println!(
        "Status: {}",
        response["status"].as_str().unwrap_or("blocked")
    );
    println!(
        "Summary: {}",
        response["summary"]
            .as_str()
            .unwrap_or("Security posture unavailable")
    );
    println!(
        "Next action: {}",
        response["next_action"]
            .as_str()
            .unwrap_or("Re-run focusa doctor security")
    );
    println!(
        "Checks: body-size, mutation rate limit, JSON shape guard, non-loopback auth, reverse-proxy guidance"
    );
    println!("Docs: docs/current/API_RESOURCE_LIMITS.md, docs/current/DAEMON_RESILIENCE.md");
}

fn repo_root() -> PathBuf {
    std::env::var_os("FOCUSA_PROJECT_ROOT")
        .map(PathBuf::from)
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."))
}

fn daemon_exe_path() -> PathBuf {
    std::env::var_os("FOCUSA_DAEMON_BIN")
        .map(PathBuf::from)
        .unwrap_or_else(|| repo_root().join("target/release/focusa-daemon"))
}

fn pi_skills_path() -> Option<PathBuf> {
    std::env::var_os("PI_SKILLS_DIR")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".pi/skills")))
}

fn path_string(path: PathBuf) -> String {
    path.to_string_lossy().to_string()
}

fn bin_check(name: &str, bin: &str) -> Value {
    let exists = std::env::var_os("PATH")
        .map(|paths| std::env::split_paths(&paths).any(|dir| dir.join(bin).exists()))
        .unwrap_or(false);
    json!({
        "name": name,
        "status": if exists { "completed" } else { "blocked" },
        "path": bin,
        "what_failed": if exists { Value::Null } else { json!("required executable missing from PATH") },
        "safe_recovery": if exists { Value::Null } else { json!(format!("install {bin} or add it to PATH")) },
    })
}

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
            "safe_recovery": "focusa start || focusa-daemon; journalctl is optional for Linux service installs",
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

pub async fn run(json_mode: bool, args: DoctorArgs) -> anyhow::Result<()> {
    if matches!(args.command, Some(DoctorCommand::Security)) {
        let response = security_posture_payload();
        if json_mode {
            println!("{}", serde_json::to_string_pretty(&response)?);
        } else {
            print_security_posture(&response);
        }
        return Ok(());
    }

    let api = ApiClient::new();
    let mut checks = Vec::new();

    checks.push(api_check(&api, "daemon health", "/v1/health").await);
    checks.push(api_check(&api, "command-center doctor API", "/v1/doctor").await);
    checks.push(api_check(&api, "API route inventory surface", "/v1/agents").await);
    checks.push(api_check(&api, "Spec90 tool contracts", "/v1/ontology/tool-contracts").await);
    checks.push(api_check(&api, "Workpoint canonicality", "/v1/workpoint/current").await);
    checks.push(
        api_check(
            &api,
            "Work-loop writer state",
            "/v1/work-loop/status?summary_only=true",
        )
        .await,
    );
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

    checks.push(fs_check("daemon exe path", &path_string(daemon_exe_path())));
    checks.push(fs_check(
        "Spec91 live proof harness",
        "scripts/prove-focusa-tool-contracts-live.mjs",
    ));
    checks.push(fs_check(
        "Spec90 contract validator",
        "scripts/validate-focusa-tool-contracts.mjs",
    ));
    checks.push(fs_check("Pi extension skills", "apps/pi-extension/skills"));
    if let Some(path) = pi_skills_path() {
        checks.push(fs_check("Pi user skills", &path_string(path)));
    }
    checks.push(fs_check("Mac app package", "apps/menubar/package.json"));
    checks.push(fs_check(
        "release command docs",
        "docs/current/PRODUCTION_RELEASE_COMMANDS.md",
    ));
    checks.push(bin_check("Guardian scanner", "guardian"));

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
        "recovery": ["focusa start", "focusa-daemon", "journalctl -u focusa-daemon -n 80 --no-pager (Linux service installs)"],
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
        println!("Recovery: focusa start || focusa-daemon");
        println!("Evidence: docs/current/EFFICIENCY_GUIDE.md, docs/current/HOOK_COVERAGE.md");
        println!("Docs: docs/92-agent-first-polish-hooks-efficiency-spec.md");
    }
    Ok(())
}
