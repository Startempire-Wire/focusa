//! `focusa bg` — first-class background execution with durable completion
//! notification. The CLI is the lifecycle monitor: it creates the job
//! record, executes the command as a detached child, and reports the
//! completion back through the daemon, which records it durably and
//! broadcasts the envelope over SSE. Not a shell wrapper — a typed,
//! ledger-backed monitor.

use clap::{Args, Subcommand};
use serde_json::{Value, json};

#[derive(Args, Debug)]
pub struct BgArgs {
    #[command(subcommand)]
    pub cmd: BgCmd,
}

#[derive(Subcommand, Debug)]
pub enum BgCmd {
    /// Run a command in the background with a durable completion record.
    Run(RunArgs),
    /// Read one job's ledger row.
    Status(StatusArgs),
    /// Block until a job completes (long-poll through the daemon).
    Wait(WaitArgs),
    /// List recent jobs.
    List,
}

#[derive(Args, Debug)]
pub struct RunArgs {
    /// Human name for the job (appears in the completion notification).
    #[arg(long)]
    pub name: String,
    /// Working directory (defaults to the current directory).
    #[arg(long)]
    pub cwd: Option<String>,
    /// Return immediately after dispatch; a detached self-monitor waits
    /// and records completion (docs/165 v2 AC1).
    #[arg(long)]
    pub detach: bool,
    /// Internal: the re-executed detached monitor sets this to skip the
    /// detach branch and run the blocking wait/complete flow.
    #[arg(long, hide = true)]
    pub internal_monitor: bool,
    /// Internal: durable job id created by the `--detach` parent.
    #[arg(long, hide = true, requires = "internal_monitor")]
    pub internal_job_id: Option<String>,
    /// Internal: durable log path created by the `--detach` parent.
    #[arg(long, hide = true, requires = "internal_monitor")]
    pub internal_log_path: Option<String>,
    /// The command to run. Everything after `--` is the command.
    #[arg(last = true, required = true)]
    pub command: Vec<String>,
}

#[derive(Args, Debug)]
pub struct StatusArgs {
    /// Job id.
    #[arg(long)]
    pub job: String,
}

#[derive(Args, Debug)]
pub struct WaitArgs {
    /// Job id.
    #[arg(long)]
    pub job: String,
    /// Poll timeout in milliseconds.
    #[arg(long, default_value_t = 30_000)]
    pub timeout_ms: u64,
}

fn internal_job_binding(args: &RunArgs) -> anyhow::Result<Option<(String, String)>> {
    if !args.internal_monitor {
        return Ok(None);
    }
    let job_id = args
        .internal_job_id
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| anyhow::anyhow!("internal monitor job_id missing"))?;
    let log_path = args
        .internal_log_path
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| anyhow::anyhow!("internal monitor log_path missing"))?;
    Ok(Some((job_id.to_string(), log_path.to_string())))
}

fn print_bg(result: Value, json_mode: bool) {
    if json_mode {
        println!(
            "{}",
            serde_json::to_string_pretty(&result).unwrap_or_default()
        );
    } else {
        println!("{}", serde_json::to_string(&result).unwrap_or_default());
    }
}

#[cfg(unix)]
fn pid_alive(pid: u32) -> bool {
    std::path::Path::new(&format!("/proc/{pid}")).exists()
}

#[cfg(not(unix))]
fn pid_alive(_pid: u32) -> bool {
    true
}

pub async fn run(cmd: BgCmd, json_mode: bool) -> anyhow::Result<()> {
    let api = crate::api_client::ApiClient::new();
    match cmd {
        BgCmd::List => {
            let result: Value = api.get("/v1/background-jobs").await?;
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                for job in result
                    .get("jobs")
                    .and_then(|j| j.as_array())
                    .into_iter()
                    .flatten()
                {
                    let id = job.get("job_id").and_then(|v| v.as_str()).unwrap_or("?");
                    let name = job.get("name").and_then(|v| v.as_str()).unwrap_or("?");
                    let status = job.get("status").and_then(|v| v.as_str()).unwrap_or("?");
                    println!("{id}\t{status}\t{name}");
                }
            }
            Ok(())
        }
        BgCmd::Status(args) => {
            let result: Value = api
                .get(&format!("/v1/background-jobs/{}", args.job))
                .await?;
            // Monitor-lost reaping: a running job whose pid is dead means the
            // monitor died without reporting. Mark it durably.
            if let Some(job) = result.get("job") {
                let status = job.get("status").and_then(|v| v.as_str()).unwrap_or("");
                if status == "running" {
                    if let Some(pid) = job.get("pid").and_then(|v| v.as_u64()) {
                        if !pid_alive(pid as u32) {
                            let _: Value = api
                                .post(
                                    &format!("/v1/background-jobs/{}", args.job),
                                    &json!({ "status": "monitor_lost" }),
                                )
                                .await?;
                            let result: Value = api
                                .get(&format!("/v1/background-jobs/{}", args.job))
                                .await?;
                            print_bg(result, json_mode);
                            return Ok(());
                        }
                    }
                }
            }
            print_bg(result, json_mode);
            Ok(())
        }
        BgCmd::Wait(args) => {
            let url = format!(
                "/v1/background-jobs/wait?job_id={}&timeout_ms={}",
                args.job, args.timeout_ms
            );
            let result: Value = api.get(&url).await?;
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                let status = result.get("status").and_then(|v| v.as_str()).unwrap_or("?");
                let job = &result["job"];
                let job_status = job.get("status").and_then(|v| v.as_str()).unwrap_or("?");
                let exit = job
                    .get("exit_code")
                    .and_then(|v| v.as_i64())
                    .map(|code| code.to_string())
                    .unwrap_or_else(|| "n/a".to_string());
                println!("{status}: {job_status} (exit {exit})");
            }
            Ok(())
        }
        BgCmd::Run(args) => {
            if args.command.is_empty() {
                anyhow::bail!("provide a command after --");
            }
            let cwd = args.cwd.clone().unwrap_or_else(|| {
                std::env::current_dir()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|_| ".".to_string())
            });
            let (job_id, log_path) = if let Some(binding) = internal_job_binding(&args)? {
                binding
            } else {
                let command = args.command.join(" ");
                let created: Value = api
                    .post(
                        "/v1/background-jobs",
                        &json!({
                            "name": args.name,
                            "command": command,
                            "cwd": cwd,
                        }),
                    )
                    .await?;
                let job = created
                    .get("job")
                    .ok_or_else(|| anyhow::anyhow!("daemon did not return a job record"))?;
                let job_id = job
                    .get("job_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("job_id missing"))?
                    .to_string();
                let log_path = job
                    .get("log_path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("log_path missing"))?
                    .to_string();
                (job_id, log_path)
            };

            // docs/165 v2: --detach re-execs this binary as a detached
            // process-group-0 monitor and returns immediately; the parent
            // terminal never blocks and no shell wrapper is required.
            if args.detach && !args.internal_monitor {
                let exe = std::env::current_exe()?;
                let mut monitor = std::process::Command::new(exe);
                monitor
                    .args([
                        "bg",
                        "run",
                        "--name",
                        &args.name,
                        "--internal-monitor",
                        "--internal-job-id",
                        &job_id,
                        "--internal-log-path",
                        &log_path,
                    ])
                    .arg("--")
                    .args(&args.command)
                    .stdin(std::process::Stdio::null())
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null());
                configure_detached_monitor(&mut monitor);
                if let Some(dir) = args.cwd.as_deref() {
                    monitor.current_dir(dir);
                }
                if let Err(error) = monitor.spawn() {
                    let _ = api
                        .post(
                            &format!("/v1/background-jobs/{job_id}"),
                            &json!({ "status": "monitor_lost" }),
                        )
                        .await;
                    return Err(error.into());
                }
                let dispatched = json!({
                    "schema": focusa_core::background_jobs::BACKGROUND_JOB_DISPATCH_SCHEMA,
                    "status": "dispatched",
                    "job_id": job_id,
                    "name": args.name,
                    "log_path": log_path,
                });
                if json_mode {
                    println!("{}", serde_json::to_string_pretty(&dispatched)?);
                } else {
                    println!("job {job_id} ({}) dispatched log {log_path}", args.name);
                }
                return Ok(());
            }

            // Detach the child from the terminal signal group, then wait on
            // it — this CLI is the monitor.
            let mut child = build_child(&args.command, &log_path)?;
            let pid = child.id();
            let _: Value = api
                .post(
                    &format!("/v1/background-jobs/{job_id}"),
                    &json!({ "status": "running", "pid": pid }),
                )
                .await?;

            let status = child.wait()?;
            let exit_code = status.code().unwrap_or(-1);
            let result: Value = api
                .post(
                    &format!("/v1/background-jobs/{job_id}/complete"),
                    &json!({ "exit_code": exit_code }),
                )
                .await?;

            if json_mode {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                let final_status = result
                    .get("job")
                    .and_then(|j| j.get("status"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                println!(
                    "job {job_id} ({}) {final_status} exit {exit_code} log {log_path}",
                    result
                        .get("job")
                        .and_then(|j| j.get("name"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("?")
                );
            }
            Ok(())
        }
    }
}

#[cfg(unix)]
fn configure_detached_monitor(command: &mut std::process::Command) {
    use std::os::unix::process::CommandExt;
    command.process_group(0);
}

#[cfg(windows)]
fn configure_detached_monitor(command: &mut std::process::Command) {
    use std::os::windows::process::CommandExt;
    const DETACHED_PROCESS: u32 = 0x0000_0008;
    const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    command.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP | CREATE_NO_WINDOW);
}

#[cfg(not(any(unix, windows)))]
fn configure_detached_monitor(_command: &mut std::process::Command) {}

#[cfg(unix)]
fn build_child(command: &[String], log_path: &str) -> anyhow::Result<std::process::Child> {
    use std::os::unix::process::CommandExt;
    use std::process::Stdio;
    let log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?;
    let log_clone = log.try_clone()?;
    let (program, rest) = command.split_first().expect("command non-empty");
    Ok(std::process::Command::new(program)
        .args(rest)
        .stdout(Stdio::from(log))
        .stderr(Stdio::from(log_clone))
        .stdin(Stdio::null())
        .process_group(0)
        .spawn()?)
}

#[cfg(not(unix))]
fn build_child(command: &[String], log_path: &str) -> anyhow::Result<std::process::Child> {
    use std::process::Stdio;
    let log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?;
    let log_clone = log.try_clone()?;
    let (program, rest) = command.split_first().expect("command non-empty");
    Ok(std::process::Command::new(program)
        .args(rest)
        .stdout(Stdio::from(log))
        .stderr(Stdio::from(log_clone))
        .stdin(Stdio::null())
        .spawn()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_args(internal_monitor: bool, job_id: Option<&str>, log_path: Option<&str>) -> RunArgs {
        RunArgs {
            name: "test-job".to_string(),
            cwd: Some("/tmp".to_string()),
            detach: false,
            internal_monitor,
            internal_job_id: job_id.map(str::to_string),
            internal_log_path: log_path.map(str::to_string),
            command: vec!["true".to_string()],
        }
    }

    #[test]
    fn detached_monitor_reuses_parent_job_binding() {
        let binding =
            internal_job_binding(&run_args(true, Some("bg-123"), Some("/tmp/bg-123.log")))
                .expect("valid binding");
        assert_eq!(
            binding,
            Some(("bg-123".to_string(), "/tmp/bg-123.log".to_string()))
        );
    }

    #[test]
    fn detached_monitor_requires_parent_job_binding() {
        let error = internal_job_binding(&run_args(true, None, None)).unwrap_err();
        assert!(error.to_string().contains("job_id missing"));
    }

    #[test]
    fn ordinary_run_creates_its_own_job() {
        assert_eq!(
            internal_job_binding(&run_args(false, None, None)).expect("ordinary run"),
            None
        );
    }
}
