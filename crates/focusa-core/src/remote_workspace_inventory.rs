//! Bounded, read-only inspection through the controller's trusted SSH setup.

use super::{RemoteOnboardingScope, RemoteWorkspaceBinding, resolve_remote_onboarding_binding};
use anyhow::{Context, Result, ensure};
use std::{process::Stdio, time::Duration};
use tokio::{io::AsyncReadExt, process::Command, time::timeout};

const BUDGET: Duration = Duration::from_secs(8);
const MAX_REPLY: u64 = 256 * 1024;

/// A point-in-time observation tied to a controller-attested binding. Commit
/// consumers must recheck the ledger under its writer lock before effects.
pub struct RemoteWorkspaceInspection {
    pub(super) attested: RemoteWorkspaceBinding,
    pub(super) inventory: RemoteWorkspaceInventory,
}

impl RemoteWorkspaceInspection {
    pub fn inventory(&self) -> &RemoteWorkspaceInventory {
        &self.inventory
    }

    pub fn into_attestation(self) -> RemoteWorkspaceBinding {
        self.attested
    }

    /// Run a controller-ledger mutation in the same SQLite transaction that
    /// checks the binding. The callback must not perform network or filesystem
    /// writes: those cannot roll back with this transaction.
    pub fn commit<T>(
        &self,
        ledger: &mut rusqlite::Connection,
        scope: &RemoteOnboardingScope<'_>,
        max_probe_age: chrono::TimeDelta,
        apply: impl FnOnce(&rusqlite::Transaction<'_>, &RemoteWorkspaceInventory) -> Result<T>,
    ) -> Result<T> {
        let transaction =
            ledger.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        resolve_remote_onboarding_binding(
            &transaction,
            scope,
            &self.attested,
            chrono::Utc::now(),
            max_probe_age,
        )?;
        let result = apply(&transaction, &self.inventory)?;
        transaction.commit()?;
        Ok(result)
    }
}

#[derive(PartialEq, Eq)]
pub struct RemoteWorkspaceInventory {
    /// A candidate for provider adoption, not proof that its queue is valid.
    pub beads_present: bool,
    beads_export: String,
    git_directory: String,
}

impl RemoteWorkspaceInventory {
    pub fn git_directory(&self) -> &str {
        &self.git_directory
    }

    /// The export is bounded and remains in memory for task projection only.
    pub fn beads_export(&self) -> &str {
        &self.beads_export
    }
}

/// Combine exact ledger admission with owner-side inventory. Open the
/// controller ledger only for the two synchronous checks, never across SSH.
/// A mutation consumer still must serialize its commit with the final scope
/// check; this operation performs no writes and returns no mutation grant.
pub async fn inspect_attested_remote_workspace(
    ledger_path: &std::path::Path,
    scope: &RemoteOnboardingScope<'_>,
    attested: &RemoteWorkspaceBinding,
    max_probe_age: chrono::TimeDelta,
) -> Result<RemoteWorkspaceInspection> {
    let recheck = || {
        let ledger = rusqlite::Connection::open_with_flags(
            ledger_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )?;
        resolve_remote_onboarding_binding(
            &ledger,
            scope,
            attested,
            chrono::Utc::now(),
            max_probe_age,
        )
    };
    recheck()?;
    let inventory = inspect_remote_workspace(scope).await?;
    recheck()?;
    Ok(RemoteWorkspaceInspection {
        attested: attested.clone(),
        inventory,
    })
}

/// Inspect a controller-selected owner and target. The caller must resolve
/// this scope from trusted configuration and be authorized to use that SSH
/// route. The probe is neither a grant nor proof of Bootstrap readiness.
pub(super) async fn inspect_remote_workspace(
    scope: &RemoteOnboardingScope<'_>,
) -> Result<RemoteWorkspaceInventory> {
    let script = inventory_command(scope.remote_root, scope.user)?;
    ensure!(
        !scope.host.is_empty()
            && !scope.host.starts_with('-')
            && scope
                .host
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b".-:_".contains(&b))
            && !scope.user.is_empty()
            && !scope.user.starts_with('-')
            && scope
                .user
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
            && scope.port != 0,
        "remote_onboarding_invalid_ssh_target"
    );
    let reply = timeout(BUDGET, async {
        let mut child = Command::new("ssh")
            .args([
                "-T",
                "-o",
                "BatchMode=yes",
                "-o",
                "StrictHostKeyChecking=yes",
                "-o",
                "ConnectTimeout=4",
                "-o",
                "ClearAllForwardings=yes",
                "-o",
                "ControlMaster=no",
                "-o",
                "ControlPath=none",
                "-l",
                scope.user,
                "-p",
                &scope.port.to_string(),
                scope.host,
                &script,
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .context("remote_onboarding_ssh_spawn_failed")?;
        let mut bytes = Vec::new();
        child
            .stdout
            .take()
            .context("remote_onboarding_ssh_stdout_missing")?
            .take(MAX_REPLY + 1)
            .read_to_end(&mut bytes)
            .await?;
        ensure!(
            bytes.len() as u64 <= MAX_REPLY,
            "remote_onboarding_inventory_too_large"
        );
        let status = child.wait().await?;
        ensure!(status.success(), "remote_onboarding_ssh_failed: {status}");
        Ok::<_, anyhow::Error>(bytes)
    })
    .await
    .context("remote_onboarding_inventory_timeout")??;
    parse_reply(
        scope.remote_root,
        &scope.project.repo_remote,
        scope.worktree_identity,
        &reply,
    )
}

fn inventory_command(root: &str, owner: &str) -> Result<String> {
    ensure!(
        root.starts_with('/') && root != "/" && !root.chars().any(char::is_control),
        "remote_onboarding_invalid_root"
    );
    let root = root.replace('\'', "'\"'\"'");
    let owner = owner.replace('\'', "'\"'\"'");
    Ok(format!(
        r#"set -e
cd '{root}'
test "$(id -un)" = '{owner}'
test "$(find . -prune -user '{owner}' -print)" = .
unset GIT_DIR GIT_WORK_TREE GIT_COMMON_DIR GIT_CONFIG GIT_CONFIG_COUNT GIT_CONFIG_PARAMETERS
pwd -P
git rev-parse --show-toplevel
git remote get-url origin
git rev-parse --path-format=absolute --git-dir
if test -d .beads && test ! -L .beads && test -f .beads/issues.jsonl && test ! -L .beads/issues.jsonl && test -r .beads/issues.jsonl; then
    printf '%s\n' beads
    head -c 262145 .beads/issues.jsonl
else
    printf '%s\n' absent
fi"#
    ))
}

fn parse_reply(
    root: &str,
    origin: &str,
    worktree: Option<&str>,
    reply: &[u8],
) -> Result<RemoteWorkspaceInventory> {
    let mut lines = std::str::from_utf8(reply)?.splitn(6, '\n');
    ensure!(
        lines.next() == Some(root) && lines.next() == Some(root),
        "remote_onboarding_git_root_mismatch"
    );
    ensure!(
        lines.next() == Some(origin),
        "remote_onboarding_repo_mismatch"
    );
    let git_dir = lines.next().context("remote_onboarding_git_dir_missing")?;
    ensure!(
        git_dir.starts_with('/') && !git_dir.chars().any(char::is_control),
        "remote_onboarding_git_dir_invalid"
    );
    if let Some(expected) = worktree {
        ensure!(git_dir == expected, "remote_onboarding_worktree_mismatch");
    }
    let provider = lines.next();
    let export = lines
        .next()
        .context("remote_onboarding_inventory_malformed")?;
    ensure!(
        provider == Some("beads") || (provider == Some("absent") && export.is_empty()),
        "remote_onboarding_inventory_malformed"
    );
    Ok(RemoteWorkspaceInventory {
        beads_present: provider == Some("beads"),
        beads_export: export.into(),
        git_directory: git_dir.into(),
    })
}

#[cfg(test)]
#[path = "remote_workspace_inventory_tests.rs"]
mod tests;
