//! Agent-first doctor command — Spec92 §9.

use super::scope_resolver;
use crate::api_client::ApiClient;
use clap::Subcommand;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

#[derive(clap::Args, Debug, Default)]
pub struct DoctorArgs {
    /// What level of checks to run. Default = host. Per transcript gap:
    /// the previous single-mode doctor was too rigid for non-focusa hosts;
    /// it tried to run repo-specific checks on a Next.js project and
    /// generated dummy files. Scope modes let the operator pick the
    /// right level.
    #[arg(long, value_name = "MODE", default_value = "host")]
    pub scope: DoctorScope,

    /// Show all checks regardless of pass/fail (default hides passed).
    #[arg(long)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Option<DoctorCommand>,
}

#[derive(clap::ValueEnum, Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DoctorScope {
    /// Daemon + lifecycle + scope safety (works on any host that has
    /// focusa-daemon running; does NOT require focusa repo presence).
    #[default]
    Host,
    /// Adds project-shape checks (scripts/validate-focusa-tool-contracts.mjs,
    /// apps/pi-extension/skills, focusa-project.json marker). Requires
    /// running inside a focusa repo.
    Project,
    /// Adds repo-only checks (target/release/focusa-daemon binary, build
    /// artifacts, etc.). Requires a built focusa repo.
    Repo,
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
    let candidates = [PathBuf::from(path), repo_root().join(path)];
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
            "json_shape_path_guard": {
                "status": "ok",
                "max_depth": env_usize("FOCUSA_API_JSON_MAX_DEPTH", 64),
                "max_array_items": env_usize("FOCUSA_API_JSON_MAX_ARRAY_ITEMS", 2_048),
                "max_object_fields": env_usize("FOCUSA_API_JSON_MAX_OBJECT_FIELDS", 2_048),
                "path_traversal": "rejects ../ and encoded traversal in path-like JSON fields",
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
    // Resolution order:
    //   1. FOCUSA_DAEMON_BIN env var (operator override)
    //   2. ~/.focusa/bin/focusa-daemon (system install via bootstrapper)
    //   3. `which focusa-daemon` (PATH lookup)
    //   4. repo_root()/target/release/focusa-daemon (dev build)
    //   5. repo_root()/target/debug/focusa-daemon (dev build, debug)
    if let Some(p) = std::env::var_os("FOCUSA_DAEMON_BIN").map(PathBuf::from) {
        return p;
    }
    if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
        let p = home.join(".focusa/bin/focusa-daemon");
        if p.exists() {
            return p;
        }
    }
    if let Ok(out) = std::process::Command::new("which")
        .arg("focusa-daemon")
        .output()
    {
        if out.status.success() {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !s.is_empty() {
                return PathBuf::from(s);
            }
        }
    }
    let rr = repo_root();
    let release = rr.join("target/release/focusa-daemon");
    if release.exists() {
        return release;
    }
    rr.join("target/debug/focusa-daemon")
}

fn pi_skills_path() -> Option<PathBuf> {
    std::env::var_os("PI_SKILLS_DIR")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".pi/skills")))
}

fn path_string(path: PathBuf) -> String {
    path.to_string_lossy().to_string()
}

fn daemon_exe_check() -> Value {
    let path = daemon_exe_path();
    let exists = path.exists();
    json!({
        "name": "daemon exe path",
        "status": if exists { "completed" } else { "degraded" },
        "path": path_string(path),
        "what_failed": if exists { Value::Null } else { json!("focusa-daemon executable was not found at the resolved helper path") },
        "safe_recovery": if exists { Value::Null } else { json!("Run focusa start, install focusa-daemon, or set FOCUSA_DAEMON_BIN to an explicit daemon binary") },
        "why": "Doctor host readiness should not be blocked by a missing dev/release helper path when daemon/API checks are reported separately."
    })
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
    let p = Path::new(path);
    let exists = p.exists();
    let (is_file, is_dir, executable, size_bytes) = if exists {
        let md = std::fs::metadata(p).ok();
        (
            md.as_ref().map(|m| m.is_file()).unwrap_or(false),
            md.as_ref().map(|m| m.is_dir()).unwrap_or(false),
            md.as_ref()
                .map(|m| {
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        m.permissions().mode() & 0o111 != 0
                    }
                    #[cfg(not(unix))]
                    {
                        let _ = m;
                        false
                    }
                })
                .unwrap_or(false),
            md.as_ref().map(|m| m.len()).unwrap_or(0),
        )
    } else {
        (false, false, false, 0)
    };
    json!({
        "name": name,
        "status": if exists { "completed" } else { "blocked" },
        "path": path,
        "exists": exists,
        "is_file": is_file,
        "is_dir": is_dir,
        "executable": executable,
        "size_bytes": size_bytes,
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

async fn canonical_orchestration_check(
    api: &ApiClient,
    name: &str,
    path: &str,
    nonempty_array_pointer: Option<&str>,
) -> Value {
    match api.get(path).await {
        Ok(response) => {
            let envelope_ready = response.get("ok").and_then(Value::as_bool) == Some(true)
                && response.get("canonical").and_then(Value::as_bool) == Some(true)
                && response.get("degraded").and_then(Value::as_bool) == Some(false);
            let catalog_ready = nonempty_array_pointer.is_none_or(|pointer| {
                response
                    .pointer(pointer)
                    .and_then(Value::as_array)
                    .is_some_and(|items| !items.is_empty())
            });
            if envelope_ready && catalog_ready {
                json!({"name": name, "status": "completed", "path": path, "details": response})
            } else {
                json!({
                    "name": name,
                    "status": "blocked",
                    "path": path,
                    "what_failed": name,
                    "likely_why": "route exists but reports degraded, noncanonical, failed, or empty required catalog state",
                    "safe_recovery": "run focusa silent doctor --json and repair the reported harness/provider/config capability",
                    "command": "focusa silent doctor --json",
                    "fallback": Value::Null,
                    "severity": "blocked",
                    "details": response,
                })
            }
        }
        Err(error) => json!({
            "name": name,
            "status": "blocked",
            "path": path,
            "what_failed": name,
            "likely_why": error.to_string(),
            "safe_recovery": "verify CLI/daemon version parity and the canonical /v1 Silent Sessions routes",
            "command": "focusa silent doctor --json",
            "fallback": Value::Null,
            "severity": "blocked",
        }),
    }
}

fn work_loop_not_configured(path: &str, reason: impl ToString) -> Value {
    json!({
        "name": "Work-loop writer state",
        "status": "not_configured",
        "path": path,
        "likely_why": reason.to_string(),
        "safe_recovery": "run focusa project or resume a project Workpoint to establish project_root + continuity_id",
        "command": "focusa project && focusa workpoint current",
        "fallback": Value::Null,
        "severity": "info",
        "daemon_restart_required": false,
    })
}

async fn scoped_work_loop_check(api: &ApiClient, path: &str) -> Value {
    let cwd = std::env::current_dir()
        .ok()
        .map(|path| path.to_string_lossy().into_owned());
    let scope = match scope_resolver::resolve_active_workstream_scope(cwd.as_deref()) {
        Ok(scope) => scope,
        Err(error) => return work_loop_not_configured(path, error),
    };
    let continuity_id = scope
        .continuity_id
        .as_deref()
        .expect("active workstream resolver requires continuity_id");
    match api
        .get_scoped(path, &scope.project_root, continuity_id)
        .await
    {
        Ok(resp) => json!({
            "name": "Work-loop writer state",
            "status": "completed",
            "path": path,
            "scope": {"project_root": scope.project_root, "continuity_id": continuity_id},
            "details": resp,
        }),
        Err(error) => {
            let reason = error.to_string();
            if reason.contains("scope_required")
                || reason.contains("scope_mismatch")
                || reason.contains("status=400")
                || reason.contains("status=409")
            {
                work_loop_not_configured(path, reason)
            } else {
                json!({
                    "name": "Work-loop writer state",
                    "status": "blocked",
                    "path": path,
                    "what_failed": "Work-loop writer state",
                    "likely_why": reason,
                    "safe_recovery": "focusa doctor --verbose; inspect daemon logs only if daemon health also fails",
                    "fallback": Value::Null,
                    "severity": "blocked",
                    "daemon_restart_required": false,
                })
            }
        }
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

fn service_authority_result(
    active_state: &str,
    service_pid: Option<u32>,
    daemon_pid: Option<u32>,
    daemon_version: Option<&str>,
) -> Value {
    let pid_matches = service_pid.is_some() && service_pid == daemon_pid;
    let version_matches = daemon_version == Some(env!("CARGO_PKG_VERSION"));
    let ready = active_state == "active" && pid_matches && version_matches;
    json!({
        "name": "installed daemon service authority",
        "status": if ready { "completed" } else { "blocked" },
        "path": "focusa-daemon.service",
        "details": {
            "active_state": active_state,
            "service_pid": service_pid,
            "daemon_pid": daemon_pid,
            "cli_version": env!("CARGO_PKG_VERSION"),
            "daemon_version": daemon_version,
            "pid_matches": pid_matches,
            "version_matches": version_matches,
        },
        "what_failed": if ready { Value::Null } else { json!("the installed service, daemon process, and CLI do not agree") },
        "safe_recovery": if ready { Value::Null } else { json!("repair the installed service and version through the canonical lifecycle") },
    })
}

#[cfg(target_os = "linux")]
fn installed_service_authority_check(health_check: &Value) -> Value {
    let output = std::process::Command::new("systemctl")
        .args([
            "show",
            "focusa-daemon.service",
            "-p",
            "LoadState",
            "-p",
            "ActiveState",
            "-p",
            "MainPID",
            "--no-pager",
        ])
        .output();
    let Ok(output) = output else {
        return json!({
            "name": "installed daemon service authority",
            "status": "not_checked",
            "reason": "systemctl is unavailable",
        });
    };
    let output_text = String::from_utf8_lossy(&output.stdout);
    let fields: std::collections::HashMap<_, _> = output_text
        .lines()
        .filter_map(|line| line.split_once('='))
        .collect();
    if fields.get("LoadState") == Some(&"not-found") {
        return json!({
            "name": "installed daemon service authority",
            "status": "not_checked",
            "reason": "no systemd service is installed on this host",
        });
    }
    let service_pid = fields
        .get("MainPID")
        .and_then(|value| value.parse::<u32>().ok())
        .filter(|pid| *pid > 0);
    let daemon_pid = health_check
        .pointer("/details/daemon/pid")
        .and_then(Value::as_u64)
        .and_then(|pid| u32::try_from(pid).ok());
    let daemon_version = health_check
        .pointer("/details/version")
        .and_then(Value::as_str);
    service_authority_result(
        fields.get("ActiveState").copied().unwrap_or("unknown"),
        service_pid,
        daemon_pid,
        daemon_version,
    )
}

#[cfg(not(target_os = "linux"))]
fn installed_service_authority_check(_health_check: &Value) -> Value {
    json!({
        "name": "installed daemon service authority",
        "status": "not_checked",
        "reason": "service ownership is verified by the platform lifecycle adapter",
    })
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

    let daemon_health = api_check(&api, "daemon health", "/v1/health").await;
    let service_authority = installed_service_authority_check(&daemon_health);
    checks.push(daemon_health);
    checks.push(service_authority);
    checks.push(api_check(&api, "command-center doctor API", "/v1/doctor").await);
    checks.push(api_check(&api, "API route inventory surface", "/v1/agents").await);
    checks.push(api_check(&api, "Spec90 tool contracts", "/v1/ontology/tool-contracts").await);
    checks.push(api_check(&api, "Workpoint canonicality", "/v1/workpoint/current").await);
    checks.push(scoped_work_loop_check(&api, "/v1/work-loop/status?summary_only=true").await);
    checks.push(
        canonical_orchestration_check(
            &api,
            "Silent Sessions API",
            "/v1/silent-sessions?limit=1",
            None,
        )
        .await,
    );
    checks.push(
        canonical_orchestration_check(
            &api,
            "Silent Sessions profile catalog",
            "/v1/silent-sessions/profiles",
            Some("/data/profiles"),
        )
        .await,
    );
    checks.push(
        canonical_orchestration_check(
            &api,
            "Silent Sessions preset catalog",
            "/v1/silent-sessions/presets",
            Some("/data/presets"),
        )
        .await,
    );
    checks.push(
        canonical_orchestration_check(
            &api,
            "Silent Sessions capability catalog",
            "/v1/silent-sessions/capabilities",
            Some("/data/harnesses"),
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

    checks.push(daemon_exe_check());

    // Transcript gap fix: only run project-shape + repo-shape checks when
    // --scope=project or --scope=repo is requested. Default --scope=host
    // runs only the daemon + lifecycle + scope-safety checks above so
    // focusa doctor works on a Next.js / non-focusa host without
    // generating dummy files.
    if matches!(args.scope, DoctorScope::Project | DoctorScope::Repo) {
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
    }
    if matches!(args.scope, DoctorScope::Repo) {
        checks.push(bin_check("Guardian scanner", "guardian"));
    }

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
    let api_readiness = checks
        .iter()
        .find(|check| check.get("path").and_then(Value::as_str) == Some("/v1/doctor"))
        .and_then(|check| check.pointer("/details/readiness_categories"))
        .cloned()
        .unwrap_or_else(|| json!({}));
    let source_blocked = checks.iter().any(|check| {
        check.get("status").and_then(Value::as_str) == Some("blocked")
            && check
                .get("name")
                .and_then(Value::as_str)
                .is_some_and(|name| {
                    name.contains("Spec90")
                        || name.contains("proof")
                        || name.contains("Pi")
                        || name.contains("Mac")
                })
    });
    let release_blocked = checks.iter().any(|check| {
        check.get("status").and_then(Value::as_str) == Some("blocked")
            && check
                .get("name")
                .and_then(Value::as_str)
                .is_some_and(|name| {
                    name.contains("release") || name.contains("Guardian") || name.contains("Mac")
                })
    });
    let service_authority_blocked = checks.iter().any(|check| {
        check.get("name").and_then(Value::as_str) == Some("installed daemon service authority")
            && check.get("status").and_then(Value::as_str) == Some("blocked")
    });
    let runtime_readiness = if service_authority_blocked {
        json!({
            "status": "blocked",
            "reason": "the installed service, daemon process, or CLI version does not agree",
        })
    } else {
        api_readiness
            .get("runtime_readiness")
            .cloned()
            .unwrap_or_else(|| {
                json!({
                    "status": "blocked",
                    "reason": "the daemon did not provide a runtime readiness decision",
                })
            })
    };
    let readiness_categories = json!({
        "runtime_readiness": runtime_readiness,
        "project_scope_readiness": api_readiness.get("project_scope_readiness").cloned().unwrap_or_else(|| json!({"status":"not_checked", "reason":"project scope is checked by project identity/verify routes"})),
        "workpoint_readiness": api_readiness.get("workpoint_readiness").cloned().unwrap_or_else(|| json!({"status":"not_checked", "reason":"workpoint current check is advisory in CLI doctor"})),
        "trajectory_readiness": api_readiness.get("trajectory_readiness").cloned().unwrap_or_else(|| json!({"status":"not_checked", "reason":"trajectory view is not required for CLI runtime doctor"})),
        "focus_state_readiness": api_readiness.get("focus_state_readiness").cloned().unwrap_or_else(|| json!({"status":"not_checked", "reason":"Focus frame is not required for CLI runtime doctor"})),
        "source_build_readiness": api_readiness.get("source_build_readiness").cloned().unwrap_or_else(|| json!({"status": if source_blocked { "blocked" } else { "ready" }, "reason":"source-build checks are separate from daemon runtime readiness"})),
        "release_readiness": json!({"status": if release_blocked { "blocked" } else if blocked > 0 { "partial" } else { "ready" }, "reason":"release/helper checks are separate from daemon runtime readiness"}),
        "telemetry_readiness": api_readiness.get("telemetry_readiness").cloned().unwrap_or_else(|| json!({"status":"not_checked", "reason":"telemetry detail is available through token/cache doctor"})),
        "ui_browser_readiness": api_readiness.get("ui_browser_readiness").cloned().unwrap_or_else(|| json!({"status":"not_checked", "reason":"UIAI browser plane is checked by Pi/UIAI tools"})),
    });
    let response = json!({
        "status": status,
        "scope": args.scope,
        "summary": if blocked > 0 { format!("{blocked} doctor check(s) blocked; readiness categories split runtime/source/release planes") } else { "All required doctor checks completed".to_string() },
        "readiness_categories": readiness_categories,
        "next_action": if blocked > 0 { "Run the recovery command for the first blocked check, then re-run focusa doctor" } else { "Continue with focusa continue or focusa release prove --tag <tag> when ready" },
        "why": "Spec92 requires one agent-first command center covering health, contracts, workpoint/work-loop, telemetry, cache, Pi skills, Mac app, release proof, Guardian, and cleanup readiness.",
        "commands": [
            "focusa doctor",
            "focusa telemetry token-budget",
            "focusa cache doctor",
            "node scripts/validate-focusa-tool-contracts.mjs",
            "node scripts/prove-focusa-tool-contracts-live.mjs --safe-fixtures"
        ],
        "recovery": if blocked > 0 {
            vec!["focusa start", "focusa-daemon", "journalctl -u focusa-daemon -n 80 --no-pager (Linux service installs)"]
        } else {
            Vec::<&str>::new()
        },
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
        if response["status"].as_str() == Some("blocked") {
            println!("Recovery: focusa start || focusa-daemon");
        }
        println!("Evidence: docs/current/EFFICIENCY_GUIDE.md, docs/current/HOOK_COVERAGE.md");
        println!("Docs: docs/92-agent-first-polish-hooks-efficiency-spec.md");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_authority_requires_active_matching_process_and_version() {
        let ready = service_authority_result(
            "active",
            Some(42),
            Some(42),
            Some(env!("CARGO_PKG_VERSION")),
        );
        assert_eq!(ready["status"], "completed");

        for blocked in [
            service_authority_result(
                "inactive",
                Some(42),
                Some(42),
                Some(env!("CARGO_PKG_VERSION")),
            ),
            service_authority_result("active", Some(42), Some(7), Some(env!("CARGO_PKG_VERSION"))),
            service_authority_result("active", Some(42), Some(42), Some("older")),
        ] {
            assert_eq!(blocked["status"], "blocked");
        }
    }

    #[test]
    fn unconfigured_work_loop_scope_is_not_daemon_blockage() {
        let check = work_loop_not_configured(
            "/v1/work-loop/status?summary_only=true",
            "continuity scope unavailable",
        );
        assert_eq!(check["status"], "not_configured");
        assert_eq!(check["severity"], "info");
        assert_eq!(check["daemon_restart_required"], false);
        assert!(check["fallback"].is_null());
        assert!(
            !check["safe_recovery"]
                .as_str()
                .unwrap_or_default()
                .contains("focusa start")
        );
        assert_eq!(status_rank("not_configured"), 0);
    }
}
