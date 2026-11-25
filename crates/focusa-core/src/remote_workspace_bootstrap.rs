//! Controller-owned Bootstrap receipt for an existing Git/Beads workspace.

use super::{RemoteOnboardingScope, RemoteWorkspaceInspection};
use anyhow::{Result, ensure};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

const SCHEMA: &str = "focusa.remote_bootstrap_receipt.v1";

/// Adoption does not imply Genesis or an active Workpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteBootstrapReceipt {
    pub schema: String,
    pub receipt_id: uuid::Uuid,
    pub binding_id: String,
    pub controller: super::ControllerIdentity,
    pub project_id: String,
    pub repo_remote: String,
    pub ssh_host: String,
    pub ssh_user: String,
    pub ssh_port: u16,
    pub principal: String,
    pub project_root: String,
    pub continuity_id: String,
    pub worktree_identity: Option<String>,
    pub git_directory: String,
    pub working_subpath: Option<String>,
    pub task_provider: String,
}

impl RemoteBootstrapReceipt {
    fn for_scope(
        scope: &RemoteOnboardingScope<'_>,
        receipt_id: uuid::Uuid,
        git_directory: String,
    ) -> Self {
        Self {
            schema: SCHEMA.into(),
            receipt_id,
            binding_id: scope.binding_id.into(),
            controller: scope.controller.clone(),
            project_id: scope.project.project_id.clone(),
            repo_remote: scope.project.repo_remote.clone(),
            ssh_host: scope.host.into(),
            ssh_user: scope.user.into(),
            ssh_port: scope.port,
            principal: scope.principal.into(),
            project_root: scope.remote_root.into(),
            continuity_id: scope.continuity_id.into(),
            worktree_identity: scope.worktree_identity.map(str::to_owned),
            git_directory,
            working_subpath: scope.working_subpath.map(str::to_owned),
            task_provider: "beads".into(),
        }
    }
}

/// Read the existing adoption receipt for a currently attested binding.
/// A missing table is a clean not-started state, not a reason to write it on
/// the status path. A historical receipt from another scope cannot satisfy
/// the Bootstrap precondition of Genesis.
pub fn read_remote_bootstrap(
    ledger: &Connection,
    scope: &RemoteOnboardingScope<'_>,
    attested: &super::RemoteWorkspaceBinding,
    now: chrono::DateTime<chrono::Utc>,
    max_probe_age: chrono::TimeDelta,
) -> Result<Option<RemoteBootstrapReceipt>> {
    let read = ledger.unchecked_transaction()?;
    super::resolve_remote_onboarding_binding(&read, scope, attested, now, max_probe_age)?;
    let receipt = load_receipt(&read, scope)?.map(|(_, receipt)| receipt);
    read.commit()?;
    Ok(receipt)
}

pub(super) fn load_receipt(
    ledger: &Connection,
    scope: &RemoteOnboardingScope<'_>,
) -> Result<Option<(String, RemoteBootstrapReceipt)>> {
    let exists: bool = ledger.query_row(
        "SELECT EXISTS (SELECT 1 FROM sqlite_schema WHERE type = 'table' AND name = 'remote_bootstrap_receipts')",
        [],
        |row| row.get(0),
    )?;
    if !exists {
        return Ok(None);
    }
    let record = ledger.query_row(
        "SELECT idempotency_key, receipt_json FROM remote_bootstrap_receipts WHERE binding_id = ?1",
        [scope.binding_id],
        |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
    ).optional()?;
    record
        .map(|(key, encoded)| {
            let receipt: RemoteBootstrapReceipt = serde_json::from_str(&encoded)?;
            ensure!(
                receipt.git_directory.starts_with('/')
                    && !receipt.git_directory.chars().any(char::is_control)
                    && scope
                        .worktree_identity
                        .is_none_or(|expected| expected == receipt.git_directory)
                    && receipt
                        == RemoteBootstrapReceipt::for_scope(
                            scope,
                            receipt.receipt_id,
                            receipt.git_directory.clone()
                        ),
                "remote_bootstrap_receipt_scope_mismatch"
            );
            Ok((key, receipt))
        })
        .transpose()
}

/// Caller must enforce operation authorization and confirmation. The receipt
/// and the final binding check share the same controller writer transaction.
/// Neither the checkout nor its Git or Beads state is changed.
pub fn adopt_remote_bootstrap(
    ledger: &mut Connection,
    scope: &RemoteOnboardingScope<'_>,
    inspection: &RemoteWorkspaceInspection,
    max_probe_age: chrono::TimeDelta,
    idempotency_key: &str,
) -> Result<RemoteBootstrapReceipt> {
    ensure!(
        !idempotency_key.trim().is_empty() && idempotency_key.len() <= 256,
        "remote_bootstrap_invalid_idempotency_key"
    );
    inspection.commit(ledger, scope, max_probe_age, |tx, inventory| {
        ensure!(inventory.beads_present, "remote_bootstrap_existing_beads_required");
        tx.execute_batch(
            "CREATE TABLE IF NOT EXISTS remote_bootstrap_receipts (
                binding_id TEXT PRIMARY KEY,
                idempotency_key TEXT NOT NULL,
                receipt_json TEXT NOT NULL
            )",
        )?;
        if let Some((key, receipt)) = load_receipt(tx, scope)? {
            ensure!(key == idempotency_key, "remote_bootstrap_receipt_conflict");
            ensure!(receipt.git_directory == inventory.git_directory(), "remote_bootstrap_worktree_changed");
            return Ok(receipt);
        }
        let receipt = RemoteBootstrapReceipt::for_scope(scope, uuid::Uuid::now_v7(), inventory.git_directory().into());
        tx.execute(
            "INSERT INTO remote_bootstrap_receipts (binding_id, idempotency_key, receipt_json) VALUES (?1, ?2, ?3)",
            params![scope.binding_id, idempotency_key, serde_json::to_string(&receipt)?],
        )?;
        Ok(receipt)
    })
}
