//! Controller-only issuance for a registered, allowlisted SSH workspace.

use super::{
    BindingStatus, RemoteOnboardingScope, RemoteWorkspaceBinding, RemoteWorkspaceInspection,
    RemoteWorkspaceInventory, inventory::inspect_remote_workspace, upsert_binding,
};
use anyhow::{Result, ensure};
use rusqlite::{Connection, OpenFlags, OptionalExtension, TransactionBehavior, params};
use std::path::Path;

fn registered(
    ledger: &Connection,
    scope: &RemoteOnboardingScope<'_>,
    allowed: &RemoteWorkspaceBinding,
) -> Result<RemoteWorkspaceBinding> {
    scope.check_identity(allowed)?;
    let encoded: String = ledger.query_row(
        "SELECT binding_json FROM remote_workspace_bindings WHERE binding_id = ?1",
        [scope.binding_id],
        |row| row.get(0),
    )?;
    let current: RemoteWorkspaceBinding = serde_json::from_str(&encoded)?;
    scope.check_identity(&current)?;
    ensure!(
        current.roots == allowed.roots
            && current.transport.host_reference == allowed.transport.host_reference,
        "remote_onboarding_config_scope_mismatch"
    );
    ensure!(
        current.state.status != BindingStatus::Revoked && current.state.revocation.is_none(),
        "remote_onboarding_binding_revoked"
    );
    Ok(current)
}

/// A preview inspects the registered target but makes no authority or ledger change.
pub async fn preview_allowed_workspace(
    path: &Path,
    scope: &RemoteOnboardingScope<'_>,
    allowed: &RemoteWorkspaceBinding,
) -> Result<RemoteWorkspaceInventory> {
    let verify_registration = || {
        let ledger = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        registered(&ledger, scope, allowed)
    };
    verify_registration()?;
    let observed = inspect_remote_workspace(scope).await?;
    verify_registration()?;
    Ok(observed)
}

/// Only the operator-confirmed adoption path calls this. The allowlist
/// nominates a target; the controller itself must perform the exact probe.
pub async fn attest_allowed_workspace(
    path: &Path,
    scope: &RemoteOnboardingScope<'_>,
    allowed: &RemoteWorkspaceBinding,
) -> Result<RemoteWorkspaceInspection> {
    let inventory = preview_allowed_workspace(path, scope, allowed).await?;
    ensure!(
        inventory.beads_present,
        "remote_bootstrap_existing_beads_required"
    );
    let mut ledger = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_WRITE)?;
    let tx = ledger.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let mut attested = registered(&tx, scope, allowed)?;
    let stamp = chrono::Utc::now().to_rfc3339();
    attested.state.status = BindingStatus::Verified;
    attested.state.freshness = Some(stamp.clone());
    attested.transport.verified_at = Some(stamp);
    attested.transport.verification_evidence =
        vec![format!("controller-ssh-inventory:{}", uuid::Uuid::now_v7())];
    upsert_binding(&tx, &attested)?;
    tx.execute_batch(
        "CREATE TABLE IF NOT EXISTS remote_workspace_verifications (
        binding_id TEXT PRIMARY KEY,
        attested_json TEXT NOT NULL
    )",
    )?;
    tx.execute(
        "INSERT INTO remote_workspace_verifications VALUES (?1, ?2)
         ON CONFLICT(binding_id) DO UPDATE SET attested_json = excluded.attested_json",
        params![scope.binding_id, serde_json::to_string(&attested)?],
    )?;
    tx.commit()?;
    Ok(RemoteWorkspaceInspection {
        attested,
        inventory,
    })
}

/// Public binding writes may register and revoke a target, but cannot mint
/// or renew its controller verification. Both must match in the same read.
pub fn load_controller_attestation(
    ledger: &Connection,
    scope: &RemoteOnboardingScope<'_>,
    allowed: &RemoteWorkspaceBinding,
    now: chrono::DateTime<chrono::Utc>,
    max_age: chrono::TimeDelta,
) -> Result<Option<RemoteWorkspaceBinding>> {
    let read = ledger.unchecked_transaction()?;
    let current = registered(&read, scope, allowed)?;
    let exists = read.query_row("SELECT EXISTS (SELECT 1 FROM sqlite_schema WHERE type = 'table' AND name = 'remote_workspace_verifications')", [], |row| row.get::<_, bool>(0))?;
    let attested = if exists {
        read.query_row(
            "SELECT attested_json FROM remote_workspace_verifications WHERE binding_id = ?1",
            [scope.binding_id],
            |row| row.get::<_, String>(0),
        )
        .optional()?
        .map(|json| serde_json::from_str::<RemoteWorkspaceBinding>(&json))
        .transpose()?
    } else {
        None
    };
    if let Some(ref attested) = attested {
        scope.check(attested, now, max_age)?;
        ensure!(*attested == current, "remote_onboarding_binding_changed");
    }
    read.commit()?;
    Ok(attested)
}
