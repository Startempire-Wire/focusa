//! `focusa bg` — typed background execution with durable completion.
//! The CLI owns monitoring; the daemon persists and broadcasts transitions.

use clap::{Args, Subcommand};
use focusa_core::background_jobs::BackgroundJobFailureClass;
use serde_json::{Value, json};

use super::bg_lifecycle::{
    await_monitor_binding, bind_detached_monitor, build_child, configure_detached_monitor,
    published_attachment, reconcile_lapsed_job, response_has_job_pid, response_has_job_status,
    settle_failure, terminate_unregistered_child,
};

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
    /// Return after dispatch while a detached self-monitor records completion.
    #[arg(long)]
    pub detach: bool,
    /// Internal: re-executed detached lifecycle monitor.
    #[arg(long, hide = true)]
    pub internal_monitor: bool,
    /// Internal: durable job id created by the `--detach` parent.
    #[arg(long, hide = true, requires = "internal_monitor")]
    pub internal_job_id: Option<String>,
    /// Internal: wait until the parent binds this monitor's exact PID.
    #[arg(long, hide = true, requires = "internal_monitor")]
    pub internal_registration_required: bool,
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

pub async fn run(cmd: BgCmd, json_mode: bool) -> anyhow::Result<()> {
    let api = crate::api_client::ApiClient::new();
    match cmd {
        BgCmd::List => {
            let mut result: Value = api.get("/v1/background-jobs").await?;
            if let Some(jobs) = result.get_mut("jobs").and_then(Value::as_array_mut) {
                for job in jobs.iter_mut() {
                    if let Some(reconciled) = reconcile_lapsed_job(&api, job).await? {
                        if let Some(updated_job) = reconciled.get("job") {
                            *job = updated_job.clone();
                        }
                    }
                }
            }
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
            if let Some(job) = result.get("job") {
                if let Some(reconciled) = reconcile_lapsed_job(&api, job).await? {
                    print_bg(reconciled, json_mode);
                    return Ok(());
                }
            }
            print_bg(result, json_mode);
            Ok(())
        }
        BgCmd::Wait(args) => {
            let current: Value = api
                .get(&format!("/v1/background-jobs/{}", args.job))
                .await?;
            if let Some(job) = current.get("job") {
                let _ = reconcile_lapsed_job(&api, job).await?;
            }
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
                let attachment = published_attachment()?;
                let created: Value = api
                    .post(
                        "/v1/background-jobs",
                        &json!({
                            "name": args.name,
                            "command": command,
                            "cwd": cwd,
                            "attachment": attachment,
                            "pid": std::process::id(),
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

            // Re-exec a detached process-group-0 monitor; no shell wrapper.
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
                        "--internal-registration-required",
                    ])
                    .arg("--")
                    .args(&args.command)
                    .stdin(std::process::Stdio::piped())
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null());
                configure_detached_monitor(&mut monitor);
                if let Some(dir) = args.cwd.as_deref() {
                    monitor.current_dir(dir);
                }
                let mut monitor_child = match monitor.spawn() {
                    Ok(child) => child,
                    Err(spawn_error) => {
                        if let Err(settlement_error) = settle_failure(
                            &api,
                            &job_id,
                            BackgroundJobFailureClass::LaunchFailed,
                            "detached_monitor_spawn",
                            &spawn_error,
                        )
                        .await
                        {
                            anyhow::bail!(
                                "detached monitor spawn failed: {spawn_error}; settling job {job_id} failed: {settlement_error}"
                            );
                        }
                        return Err(spawn_error.into());
                    }
                };
                let registration_error = bind_detached_monitor(&api, &job_id, &mut monitor_child)
                    .await
                    .err();
                if let Some(registration_error) = registration_error {
                    let termination_error = terminate_unregistered_child(&mut monitor_child).err();
                    if let Err(settlement_error) = settle_failure(
                        &api,
                        &job_id,
                        BackgroundJobFailureClass::LaunchFailed,
                        "detached_monitor_registration",
                        &registration_error,
                    )
                    .await
                    {
                        anyhow::bail!(
                            "detached monitor registration failed: {registration_error}; termination={termination_error:?}; settling job {job_id} failed: {settlement_error}"
                        );
                    }
                    if let Some(termination_error) = termination_error {
                        anyhow::bail!(
                            "detached monitor registration failed: {registration_error}; exact monitor termination also failed: {termination_error}"
                        );
                    }
                    return Err(registration_error);
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

            if args.internal_registration_required {
                if let Err(binding_error) = await_monitor_binding() {
                    if let Err(settlement_error) = settle_failure(
                        &api,
                        &job_id,
                        BackgroundJobFailureClass::LaunchFailed,
                        "detached_monitor_registration",
                        &binding_error,
                    )
                    .await
                    {
                        anyhow::bail!(
                            "detached monitor binding failed: {binding_error}; settling job {job_id} failed: {settlement_error}"
                        );
                    }
                    return Err(binding_error);
                }
            }

            // Register before log-open/child-spawn so every row has a live
            // monitor PID or terminal launch_failed settlement.
            let pid = std::process::id();
            let running = api
                .post(
                    &format!("/v1/background-jobs/{job_id}"),
                    &json!({ "status": "running", "pid": pid }),
                )
                .await;
            let registration_error = match running {
                Ok(response)
                    if response_has_job_status(&response, &job_id, "running")
                        && response_has_job_pid(&response, &job_id, pid) =>
                {
                    None
                }
                Ok(_) => Some(anyhow::anyhow!(
                    "daemon did not durably transition background job {job_id} to running"
                )),
                Err(error) => Some(error),
            };
            if let Some(registration_error) = registration_error {
                if let Err(settlement_error) = settle_failure(
                    &api,
                    &job_id,
                    BackgroundJobFailureClass::LaunchFailed,
                    "running_transition",
                    &registration_error,
                )
                .await
                {
                    anyhow::bail!(
                        "background running transition failed: {registration_error}; settling job {job_id} failed: {settlement_error}"
                    );
                }
                return Err(registration_error);
            }

            // Detach the child from the terminal signal group, then wait on
            // it — this CLI is the exact monitor recorded above.
            let mut child = match build_child(&args.command, &log_path, args.cwd.as_deref()) {
                Ok(child) => child,
                Err(spawn_error) => {
                    if let Err(settlement_error) = settle_failure(
                        &api,
                        &job_id,
                        BackgroundJobFailureClass::LaunchFailed,
                        "command_spawn",
                        &spawn_error,
                    )
                    .await
                    {
                        anyhow::bail!(
                            "background command spawn failed: {spawn_error}; settling job {job_id} failed: {settlement_error}"
                        );
                    }
                    return Err(spawn_error);
                }
            };

            let status = match child.wait() {
                Ok(status) => status,
                Err(wait_error) => {
                    if let Err(settlement_error) = settle_failure(
                        &api,
                        &job_id,
                        BackgroundJobFailureClass::MonitorFailed,
                        "command_wait",
                        &wait_error,
                    )
                    .await
                    {
                        anyhow::bail!(
                            "background command wait failed: {wait_error}; settling job {job_id} failed: {settlement_error}"
                        );
                    }
                    return Err(wait_error.into());
                }
            };
            let exit_code = status.code().unwrap_or(-1);
            let output_tail = focusa_core::background_jobs::bounded_log_tail(&log_path, 4096);
            let result: Value = api
                .post(
                    &format!("/v1/background-jobs/{job_id}/complete"),
                    &json!({ "exit_code": exit_code, "output_tail": output_tail }),
                )
                .await?;
            let expected_status = if exit_code == 0 {
                "completed"
            } else {
                "failed"
            };
            anyhow::ensure!(
                response_has_job_status(&result, &job_id, expected_status),
                "daemon did not durably complete background job {job_id} as {expected_status}"
            );

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
            internal_registration_required: false,
            internal_log_path: log_path.map(str::to_string),
            command: vec!["true".to_string()],
        }
    }

    #[cfg(unix)]
    #[test]
    fn child_execution_honors_requested_working_directory() {
        let root = std::env::temp_dir().join(format!("focusa-bg-cwd-{}", uuid::Uuid::now_v7()));
        std::fs::create_dir_all(&root).unwrap();
        let log = root.join("pwd.log");
        let command = vec!["pwd".to_string()];
        let mut child = build_child(&command, log.to_str().unwrap(), root.to_str()).unwrap();
        assert!(child.wait().unwrap().success());
        assert_eq!(
            std::fs::read_to_string(&log).unwrap().trim(),
            root.to_str().unwrap()
        );
        std::fs::remove_dir_all(root).unwrap();
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
