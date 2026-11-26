//! Controller-owned remote Genesis journal and atomic publication fence.

use super::{RemoteOnboardingScope, RemoteWorkspaceInspection, bootstrap};
use crate::types::{
    FocusaState, HltLedgerEntry, TrajectoryLadderEvent, TrajectoryProjectionRecord,
    WorkpointRecord, WorkpointStatus,
};
use anyhow::{Result, ensure};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteGenesisIntent {
    pub intent_id: uuid::Uuid,
    pub bootstrap_receipt_id: uuid::Uuid,
    pub plan_digest: String,
}

impl RemoteGenesisIntent {
    /// Legacy projection tables key on a string. Reserve a non-filesystem
    /// key for this exact intent so identical host paths never share state.
    pub fn projection_key(&self) -> String {
        format!(
            "{}{}",
            crate::scope_safety::REMOTE_PROJECTION_PREFIX,
            self.intent_id
        )
    }

    /// Select only the uniquely identified record under this committed intent.
    /// The reserved projection key includes the intent id, not a host pathname.
    pub fn scoped_workpoint<'a>(
        &self,
        state: &'a FocusaState,
        continuity: &str,
        workpoint_id: uuid::Uuid,
    ) -> Option<&'a WorkpointRecord> {
        let key = self.projection_key();
        let mut records = state
            .workpoint
            .records
            .iter()
            .filter(|record| record.workpoint_id == workpoint_id);
        let record = records.next()?;
        (records.next().is_none()
            && record.canonical
            && record.project_root.as_deref() == Some(key.as_str())
            && record.continuity_id.as_deref() == Some(continuity)
            && record.work_item_id.is_some()
            && record
                .next_slice
                .as_deref()
                .is_some_and(|next| !next.trim().is_empty()))
        .then_some(record)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteGenesisPublication {
    pub trajectory_id: String,
    pub workpoint_id: uuid::Uuid,
    pub hlt: HltLedgerEntry,
    pub ladder: Vec<TrajectoryLadderEvent>,
}

impl RemoteGenesisPublication {
    /// Validate an attested publication's trajectory without treating its first
    /// Workpoint as the permanent frontier.
    pub fn verified_trajectory<'a>(
        &self,
        state: &'a FocusaState,
        intent: &RemoteGenesisIntent,
        continuity: &str,
    ) -> Option<&'a TrajectoryProjectionRecord> {
        let key = intent.projection_key();
        let valid = self.hlt.project_root == key
            && self.hlt.continuity_id.as_deref() == Some(continuity)
            && self.ladder.first().is_some_and(|first| {
                first.level == crate::types::TrajectoryLadderLevel::Hlt
                    && first.event_id == format!("legacy-hlt:{}", self.hlt.event_id)
                    && first.causal_parent_event_id.is_none()
            })
            && self.ladder.windows(2).all(|pair| {
                pair[1].causal_parent_event_id.as_deref() == Some(pair[0].event_id.as_str())
            })
            && self.ladder.iter().all(|event| {
                event.project_root == key
                    && event.continuity_id.as_deref() == Some(continuity)
                    && event.trajectory_id == self.trajectory_id
            });
        if !valid {
            return None;
        }
        let mut records = state
            .trajectory
            .records
            .iter()
            .filter(|record| record.trajectory_id == self.trajectory_id);
        let record = records.next()?;
        (records.next().is_none()
            && record.canonical
            && record.project_root.as_deref() == Some(key.as_str())
            && record.continuity_id.as_deref() == Some(continuity)
            && record.long_term_goal == self.hlt.new_hlt)
            .then_some(record)
    }

    /// The initial commit fence still requires the first task to be active.
    pub fn verifies(
        &self,
        state: &FocusaState,
        intent: &RemoteGenesisIntent,
        continuity: &str,
    ) -> bool {
        self.verified_trajectory(state, intent, continuity)
            .is_some_and(|record| record.active_workpoint_id == Some(self.workpoint_id))
            && intent
                .scoped_workpoint(state, continuity, self.workpoint_id)
                .is_some_and(|record| record.status == WorkpointStatus::Active)
    }
}

#[derive(Serialize, Deserialize)]
struct StagedGenesis {
    receipt: RemoteGenesisIntent,
    plan: serde_json::Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    publication: Option<RemoteGenesisPublication>,
}

fn digest_plan(plan: &serde_json::Value) -> Result<String> {
    let bytes = serde_json::to_vec(plan)?;
    ensure!(bytes.len() <= 256 * 1024, "remote_genesis_plan_too_large");
    Ok(format!("sha256:{:x}", Sha256::digest(bytes)))
}

/// Observe staged intent under the current attestation. Never infer canonical
/// readiness from the presence of this row.
pub fn read_remote_genesis_intent(
    ledger: &Connection,
    scope: &RemoteOnboardingScope<'_>,
    attested: &super::RemoteWorkspaceBinding,
    now: chrono::DateTime<chrono::Utc>,
    max_probe_age: chrono::TimeDelta,
    expected_replay_key: Option<&str>,
) -> Result<Option<RemoteGenesisIntent>> {
    Ok(read_remote_genesis_publication(
        ledger,
        scope,
        attested,
        now,
        max_probe_age,
        expected_replay_key,
    )?
    .map(|(intent, _)| intent))
}

/// Read the exact intent and publication in a single binding-checked snapshot.
pub fn read_remote_genesis_publication(
    ledger: &Connection,
    scope: &RemoteOnboardingScope<'_>,
    attested: &super::RemoteWorkspaceBinding,
    now: chrono::DateTime<chrono::Utc>,
    max_probe_age: chrono::TimeDelta,
    expected_replay_key: Option<&str>,
) -> Result<Option<(RemoteGenesisIntent, Option<RemoteGenesisPublication>)>> {
    let read = ledger.unchecked_transaction()?;
    let intent = checked_intent(
        &read,
        scope,
        attested,
        now,
        max_probe_age,
        expected_replay_key,
    )?;
    read.commit()?;
    Ok(intent.map(|staged| (staged.receipt, staged.publication)))
}

/// Exact staged inputs that must still be current inside the reducer writer transaction.
pub struct RemoteGenesisFence {
    attested: super::RemoteWorkspaceBinding,
    max_probe_age: chrono::TimeDelta,
    replay_key: String,
    expected_intent: RemoteGenesisIntent,
    git_directory: String,
    publication: Option<RemoteGenesisPublication>,
}

impl RemoteGenesisFence {
    /// Load the immutable plan and its commit fence from one consistent read.
    /// Publication must compile from this plan, not a new caller-supplied body.
    pub fn load(
        ledger: &Connection,
        scope: &RemoteOnboardingScope<'_>,
        attested: &super::RemoteWorkspaceBinding,
        max_probe_age: chrono::TimeDelta,
        replay_key: &str,
        inventory: &super::RemoteWorkspaceInventory,
    ) -> Result<(Self, serde_json::Value)> {
        ensure!(
            !replay_key.trim().is_empty() && replay_key.len() <= 256,
            "remote_genesis_invalid_idempotency_key"
        );
        let read = ledger.unchecked_transaction()?;
        ensure!(
            inventory.beads_present,
            "remote_genesis_existing_beads_required"
        );
        let staged = checked_intent(
            &read,
            scope,
            attested,
            chrono::Utc::now(),
            max_probe_age,
            Some(replay_key),
        )?
        .ok_or_else(|| anyhow::anyhow!("remote_genesis_intent_not_found"))?;
        verify_inventory_receipt(
            &read,
            scope,
            inventory.git_directory(),
            staged.receipt.bootstrap_receipt_id,
        )?;
        read.commit()?;
        Ok((
            Self {
                attested: attested.clone(),
                max_probe_age,
                replay_key: replay_key.into(),
                expected_intent: staged.receipt,
                git_directory: inventory.git_directory().into(),
                publication: None,
            },
            staged.plan,
        ))
    }

    pub fn intent(&self) -> &RemoteGenesisIntent {
        &self.expected_intent
    }

    pub fn with_publication(mut self, publication: RemoteGenesisPublication) -> Self {
        self.publication = Some(publication);
        self
    }

    pub(crate) fn verify_and_publish(
        &self,
        transaction: &rusqlite::Transaction<'_>,
        state: &FocusaState,
    ) -> Result<()> {
        let binding = &self.attested;
        let scope = RemoteOnboardingScope {
            binding_id: &binding.binding_id,
            controller: &binding.controller,
            project: &binding.project,
            host: &binding.transport.host,
            user: &binding.transport.user,
            port: binding.transport.port,
            remote_root: &binding.roots.canonical_remote_root,
            working_subpath: binding.roots.working_subpath.as_deref(),
            worktree_identity: binding.roots.worktree_identity.as_deref(),
            continuity_id: &binding.session.continuity_id,
            principal: binding.session.principal.as_deref().unwrap_or_default(),
        };
        let current = checked_intent(
            transaction,
            &scope,
            binding,
            chrono::Utc::now(),
            self.max_probe_age,
            Some(&self.replay_key),
        )?;
        ensure!(
            current.as_ref().map(|staged| &staged.receipt) == Some(&self.expected_intent),
            "remote_genesis_intent_changed"
        );
        verify_inventory_receipt(
            transaction,
            &scope,
            &self.git_directory,
            self.expected_intent.bootstrap_receipt_id,
        )?;
        if let Some(publication) = &self.publication {
            let mut row =
                current.ok_or_else(|| anyhow::anyhow!("remote_genesis_intent_changed"))?;
            ensure!(
                row.publication.is_none(),
                "remote_genesis_already_published"
            );
            ensure!(
                publication.verifies(state, &row.receipt, scope.continuity_id),
                "remote_genesis_projection_mismatch"
            );
            row.publication = Some(publication.clone());
            ensure!(transaction.execute(
                "UPDATE remote_genesis_intents SET intent_json = ?1 WHERE binding_id = ?2 AND idempotency_key = ?3",
                params![serde_json::to_string(&row)?, scope.binding_id, self.replay_key],
            )? == 1, "remote_genesis_intent_changed");
        }
        Ok(())
    }
}

fn checked_intent(
    transaction: &rusqlite::Transaction<'_>,
    scope: &RemoteOnboardingScope<'_>,
    attested: &super::RemoteWorkspaceBinding,
    now: chrono::DateTime<chrono::Utc>,
    max_probe_age: chrono::TimeDelta,
    expected_replay_key: Option<&str>,
) -> Result<Option<StagedGenesis>> {
    super::resolve_remote_onboarding_binding(transaction, scope, attested, now, max_probe_age)?;
    let current_receipt =
        bootstrap::load_receipt(transaction, scope)?.map(|(_, row)| row.receipt_id);
    let intent = load_intent(transaction, scope.binding_id)?;
    if let (Some(expected), Some((stored, _))) = (expected_replay_key, &intent) {
        ensure!(expected == stored, "remote_genesis_intent_conflict");
    }
    let intent = intent.map(|(_, row)| row);
    if let Some(row) = &intent {
        ensure!(
            Some(row.receipt.bootstrap_receipt_id) == current_receipt,
            "remote_genesis_bootstrap_receipt_changed"
        );
    }
    Ok(intent)
}

fn load_intent(ledger: &Connection, binding_id: &str) -> Result<Option<(String, StagedGenesis)>> {
    let exists: bool = ledger.query_row(
        "SELECT EXISTS (SELECT 1 FROM sqlite_schema WHERE type = 'table' AND name = 'remote_genesis_intents')", [], |row| row.get(0),
    )?;
    if !exists {
        return Ok(None);
    }
    ledger
        .query_row(
            "SELECT idempotency_key, intent_json FROM remote_genesis_intents WHERE binding_id = ?1",
            [binding_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )
        .optional()?
        .map(|(key, json)| {
            let staged: StagedGenesis = serde_json::from_str(&json)?;
            ensure!(
                staged.receipt.plan_digest == digest_plan(&staged.plan)?,
                "remote_genesis_plan_integrity_failed"
            );
            Ok((key, staged))
        })
        .transpose()
}

fn verify_inventory_receipt(
    tx: &rusqlite::Transaction<'_>,
    scope: &RemoteOnboardingScope<'_>,
    git_directory: &str,
    receipt_id: uuid::Uuid,
) -> Result<()> {
    let receipt = bootstrap::load_receipt(tx, scope)?
        .map(|(_, receipt)| receipt)
        .filter(|receipt| receipt.receipt_id == receipt_id)
        .ok_or_else(|| anyhow::anyhow!("remote_genesis_bootstrap_receipt_changed"))?;
    ensure!(
        receipt.git_directory == git_directory,
        "remote_genesis_worktree_changed"
    );
    Ok(())
}

/// Reserve a specific plan and adoption receipt without publishing outcomes.
/// Replays must match both the operation key and the exact plan inputs.
pub fn stage_remote_genesis(
    ledger: &mut Connection,
    scope: &RemoteOnboardingScope<'_>,
    inspection: &RemoteWorkspaceInspection,
    max_probe_age: chrono::TimeDelta,
    bootstrap_receipt_id: uuid::Uuid,
    idempotency_key: &str,
    plan: &serde_json::Value,
) -> Result<RemoteGenesisIntent> {
    ensure!(
        !idempotency_key.trim().is_empty() && idempotency_key.len() <= 256,
        "remote_genesis_invalid_idempotency_key"
    );
    let plan_digest = digest_plan(plan)?;
    inspection.commit(ledger, scope, max_probe_age, |tx, inventory| {
        ensure!(
            inventory.beads_present,
            "remote_genesis_existing_beads_required"
        );
        verify_inventory_receipt(tx, scope, inventory.git_directory(), bootstrap_receipt_id)?;
        tx.execute_batch(
            "CREATE TABLE IF NOT EXISTS remote_genesis_intents (
            binding_id TEXT PRIMARY KEY,
            idempotency_key TEXT NOT NULL,
            intent_json TEXT NOT NULL
        )",
        )?;
        if let Some((key, prior)) = load_intent(tx, scope.binding_id)? {
            let prior = prior.receipt;
            ensure!(
                key == idempotency_key
                    && prior.bootstrap_receipt_id == bootstrap_receipt_id
                    && prior.plan_digest == plan_digest,
                "remote_genesis_intent_conflict"
            );
            return Ok(prior);
        }
        let intent = RemoteGenesisIntent {
            intent_id: uuid::Uuid::now_v7(),
            bootstrap_receipt_id,
            plan_digest,
        };
        tx.execute(
            "INSERT INTO remote_genesis_intents VALUES (?1, ?2, ?3)",
            params![
                scope.binding_id,
                idempotency_key,
                serde_json::to_string(&StagedGenesis {
                    receipt: intent.clone(),
                    plan: plan.clone(),
                    publication: None,
                })?
            ],
        )?;
        Ok(intent)
    })
}
