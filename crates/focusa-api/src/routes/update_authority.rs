//! API adapter for the canonical transactional updater implemented by the Focusa CLI.
//!
//! Keeping one mutation authority avoids a second, subtly different installer in the
//! daemon. The adapter fails closed when the CLI is absent, exits unsuccessfully, or
//! returns an envelope with the wrong schema.

use anyhow::{Context, bail};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use tokio::process::Command;

#[derive(Debug, Clone, Default)]
pub struct UpdateRequest {
    pub channel: Option<String>,
    pub latest_version: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ApplyRequest {
    pub update: UpdateRequest,
    pub dry_run: bool,
    pub yes: bool,
    pub allow_apply: bool,
}

#[derive(Debug, Clone)]
pub struct RollbackRequest {
    pub part: String,
    pub dry_run: bool,
    pub yes: bool,
}

#[derive(Debug, Clone)]
pub struct CliUpdateAuthority {
    cli: PathBuf,
}

impl CliUpdateAuthority {
    pub fn installed() -> Self {
        let cli = std::env::var_os("FOCUSA_CLI_PATH")
            .map(PathBuf::from)
            .or_else(|| {
                let sibling = std::env::current_exe()
                    .ok()?
                    .with_file_name(if cfg!(windows) {
                        "focusa.exe"
                    } else {
                        "focusa"
                    });
                sibling.is_file().then_some(sibling)
            })
            .unwrap_or_else(|| PathBuf::from("/usr/local/bin/focusa"));
        Self::new(cli)
    }

    pub fn new(cli: impl Into<PathBuf>) -> Self {
        Self { cli: cli.into() }
    }

    pub async fn plan(&self, request: UpdateRequest) -> Value {
        let mut args = vec!["update".into(), "plan".into()];
        push_update_args(&mut args, &request);
        args.push("--json".into());
        self.execute("focusa.update_plan.v1", &args).await
    }

    pub async fn apply(&self, request: ApplyRequest) -> Value {
        if request.dry_run || !request.yes || !request.allow_apply {
            return consent_blocked_apply(&request);
        }
        let mut args = vec!["update".into(), "apply".into()];
        push_update_args(&mut args, &request.update);
        args.extend([
            "--dry-run=false".into(),
            "--yes".into(),
            "--allow-apply".into(),
            "--json".into(),
        ]);
        self.execute("focusa.update_apply.v1", &args).await
    }

    pub async fn rollback(&self, request: RollbackRequest) -> Value {
        if request.dry_run || !request.yes {
            return consent_blocked_rollback(&request);
        }
        if !matches!(request.part.as_str(), "all" | "cli" | "tui" | "daemon") {
            return blocked(
                "focusa.update_rollback.v1",
                "invalid_rollback_part",
                "part must be one of all, cli, tui, or daemon",
            );
        }
        let args = vec![
            "update".into(),
            "rollback".into(),
            "--part".into(),
            request.part,
            "--dry-run=false".into(),
            "--yes".into(),
            "--json".into(),
        ];
        self.execute("focusa.update_rollback.v1", &args).await
    }

    async fn execute(&self, expected_schema: &str, args: &[String]) -> Value {
        match execute_cli(&self.cli, expected_schema, args).await {
            Ok(value) => value,
            Err(error) => blocked(
                expected_schema,
                "canonical_update_authority_failed",
                &format!("{error:#}"),
            ),
        }
    }
}

fn push_update_args(args: &mut Vec<String>, request: &UpdateRequest) {
    if let Some(channel) = request.channel.as_deref().filter(|v| !v.trim().is_empty()) {
        args.extend(["--channel".into(), channel.into()]);
    }
    if let Some(version) = request
        .latest_version
        .as_deref()
        .filter(|v| !v.trim().is_empty())
    {
        args.extend(["--latest-version".into(), version.into()]);
    }
}

async fn execute_cli(cli: &Path, expected_schema: &str, args: &[String]) -> anyhow::Result<Value> {
    let output = Command::new(cli)
        .args(args)
        .output()
        .await
        .with_context(|| format!("start canonical updater {}", cli.display()))?;
    let value: Value = serde_json::from_slice(&output.stdout).with_context(|| {
        format!(
            "canonical updater returned non-JSON output (exit={:?}, stderr={})",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
                .chars()
                .take(512)
                .collect::<String>()
        )
    })?;
    if value.get("schema").and_then(Value::as_str) != Some(expected_schema) {
        bail!("canonical updater returned an unexpected envelope schema");
    }
    // The CLI intentionally exits non-zero after emitting a truthful failed/rolled-back
    // apply envelope. Preserve that envelope, but never accept a successful-looking one.
    if !output.status.success()
        && !matches!(
            value.get("status").and_then(Value::as_str),
            Some("failed_rolled_back" | "blocked_read_only" | "failed")
        )
    {
        bail!("canonical updater exited unsuccessfully without a failure envelope");
    }
    Ok(value)
}

fn consent_blocked_apply(request: &ApplyRequest) -> Value {
    json!({
        "schema": "focusa.update_apply.v1",
        "status": "blocked_read_only",
        "read_only": true,
        "mutations_performed": false,
        "apply_requested": request.yes || request.allow_apply || !request.dry_run,
        "apply_executed": false,
        "dry_run": request.dry_run,
        "consent": {"yes": request.yes, "allow_apply": request.allow_apply, "effective": false},
        "blocked_reason": ["explicit_yes_allow_apply_and_dry_run_false_required"],
        "recovery_hint": "Inspect /v1/update/plan, then provide yes=true, allow_apply=true, and dry_run=false."
    })
}

fn consent_blocked_rollback(request: &RollbackRequest) -> Value {
    json!({
        "schema": "focusa.update_rollback.v1",
        "status": "blocked_read_only",
        "read_only": true,
        "mutations_performed": false,
        "rollback_executed": false,
        "part": request.part,
        "dry_run": request.dry_run,
        "consent_yes": request.yes,
        "blocked_reason": ["explicit_yes_and_dry_run_false_required"],
        "recovery_hint": "Inspect update history, then provide yes=true and dry_run=false."
    })
}

fn blocked(schema: &str, failure_class: &str, error: &str) -> Value {
    json!({
        "schema": schema,
        "status": "blocked",
        "read_only": true,
        "mutations_performed": false,
        "failure_class": failure_class,
        "error": error,
        "fail_closed": true
    })
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use sha2::Digest;
    use std::os::unix::fs::PermissionsExt;

    fn fixture(script_body: &str) -> (PathBuf, PathBuf) {
        let root = std::env::temp_dir().join(format!(
            "focusa-api-update-authority-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let script = root.join("focusa");
        std::fs::write(&script, format!("#!/bin/sh\nset -eu\n{script_body}\n")).unwrap();
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
        (root, script)
    }

    #[tokio::test]
    async fn plan_delegates_channel_and_version_to_live_release_resolver() {
        let body = r#"
printf '%s\n' "$*" > "$(dirname "$0")/args"
printf '%s\n' '{"schema":"focusa.update_plan.v1","status":"planned_read_only","apply_allowed":true}'
"#;
        let (root, script) = fixture(body);
        let out = CliUpdateAuthority::new(script)
            .plan(UpdateRequest {
                channel: Some("stable".into()),
                latest_version: Some("0.9.150".into()),
            })
            .await;
        assert_eq!(out["schema"], "focusa.update_plan.v1");
        let args = std::fs::read_to_string(root.join("args")).unwrap();
        assert!(args.contains("update plan --channel stable --latest-version 0.9.150 --json"));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn apply_without_complete_consent_never_invokes_updater() {
        let (root, script) = fixture("touch \"$(dirname \"$0\")/invoked\"");
        let out = CliUpdateAuthority::new(script)
            .apply(ApplyRequest {
                update: UpdateRequest::default(),
                dry_run: false,
                yes: true,
                allow_apply: false,
            })
            .await;
        assert_eq!(out["status"], "blocked_read_only");
        assert!(!root.join("invoked").exists());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn isolated_apply_and_rollback_delegate_to_one_transaction_authority() {
        let body = r#"
root=$(dirname "$0")
case "$2" in
  apply)
    cp "$root/live" "$root/backup"
    cp "$root/candidate" "$root/live"
    printf '%s\n' '{"schema":"focusa.update_apply.v1","status":"completed","mutations_performed":true,"apply_executed":true}'
    ;;
  rollback)
    test "$(sha256sum "$root/backup" | cut -d' ' -f1)" = "$(cat "$root/backup.sha256")"
    cp "$root/backup" "$root/live"
    printf '%s\n' '{"schema":"focusa.update_rollback.v1","status":"completed","mutations_performed":true,"rollback_executed":true}'
    ;;
  *) exit 64 ;;
esac
"#;
        let (root, script) = fixture(body);
        std::fs::write(root.join("live"), b"old").unwrap();
        std::fs::write(root.join("candidate"), b"new").unwrap();
        let digest = format!("{:x}", sha2::Sha256::digest(b"old"));
        std::fs::write(root.join("backup.sha256"), digest).unwrap();
        let authority = CliUpdateAuthority::new(script);
        let applied = authority
            .apply(ApplyRequest {
                update: UpdateRequest::default(),
                dry_run: false,
                yes: true,
                allow_apply: true,
            })
            .await;
        assert_eq!(applied["status"], "completed");
        assert_eq!(std::fs::read(root.join("live")).unwrap(), b"new");
        let rolled_back = authority
            .rollback(RollbackRequest {
                part: "all".into(),
                dry_run: false,
                yes: true,
            })
            .await;
        assert_eq!(rolled_back["status"], "completed");
        assert_eq!(std::fs::read(root.join("live")).unwrap(), b"old");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn wrong_schema_fails_closed() {
        let (root, script) =
            fixture("printf '%s\\n' '{\"schema\":\"wrong\",\"status\":\"completed\"}'");
        let out = CliUpdateAuthority::new(script)
            .plan(UpdateRequest::default())
            .await;
        assert_eq!(out["failure_class"], "canonical_update_authority_failed");
        assert_eq!(out["fail_closed"], true);
        std::fs::remove_dir_all(root).unwrap();
    }
}
