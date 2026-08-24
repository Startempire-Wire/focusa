use chrono::{DateTime, Utc};
use rusqlite::{OptionalExtension, params};

use crate::runtime::persistence_sqlite::SqlitePersistence;

use super::{
    ApprovalId, AuthenticatedPrincipal, AuthenticatedRunnerCommand, ControlAuditId,
    DurableApprovalRecord, RedactedControlAuditRecord,
};

pub fn save_authorization_principal(
    persistence: &SqlitePersistence,
    principal: &AuthenticatedPrincipal,
    updated_at: DateTime<Utc>,
) -> anyhow::Result<()> {
    persistence.with_connection_mut(|connection| {
        connection.execute(
            r#"INSERT INTO silent_session_control_principals(
               principal_id,actor,os_user,role,principal_json,updated_at
               ) VALUES (?1,?2,?3,?4,?5,?6)
               ON CONFLICT(principal_id) DO UPDATE SET
                 actor=excluded.actor,os_user=excluded.os_user,role=excluded.role,
                 principal_json=excluded.principal_json,updated_at=excluded.updated_at"#,
            params![
                principal.principal_id,
                principal.actor,
                principal.os_user,
                enum_text(principal.role)?,
                serde_json::to_string(principal)?,
                updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    })
}

pub fn load_authorization_principal(
    persistence: &SqlitePersistence,
    principal_id: &str,
) -> anyhow::Result<Option<AuthenticatedPrincipal>> {
    load_json(
        persistence,
        "SELECT principal_json FROM silent_session_control_principals WHERE principal_id=?1",
        principal_id,
    )
}

pub fn save_durable_approval(
    persistence: &SqlitePersistence,
    approval: &DurableApprovalRecord,
) -> anyhow::Result<()> {
    persistence.with_connection_mut(|connection| {
        connection.execute(
            r#"INSERT INTO silent_session_control_approvals(
               approval_id,operator_actor,action,project_root,continuity_id,session_id,run_id,
               action_digest,expires_at,approval_json,issuance_idempotency_key,issuance_request_hash
               ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)"#,
            params![
                approval.approval_id.to_string(),
                approval.operator_actor,
                enum_text(approval.action)?,
                approval.project_root,
                approval.continuity_id,
                approval.session_id.map(|id| id.to_string()),
                approval.run_id.map(|id| id.to_string()),
                approval.action_digest,
                approval.expires_at.to_rfc3339(),
                serde_json::to_string(approval)?,
                (!approval.issuance_idempotency_key.is_empty())
                    .then_some(approval.issuance_idempotency_key.as_str()),
                (!approval.issuance_request_hash.is_empty())
                    .then_some(approval.issuance_request_hash.as_str()),
            ],
        )?;
        Ok(())
    })
}

pub fn load_durable_approval(
    persistence: &SqlitePersistence,
    approval_id: ApprovalId,
) -> anyhow::Result<Option<DurableApprovalRecord>> {
    load_json(
        persistence,
        "SELECT approval_json FROM silent_session_control_approvals WHERE approval_id=?1",
        &approval_id.to_string(),
    )
}

pub fn load_durable_approval_by_idempotency(
    persistence: &SqlitePersistence,
    operator_actor: &str,
    idempotency_key: &str,
) -> anyhow::Result<Option<DurableApprovalRecord>> {
    persistence.with_connection_mut(|connection| {
        let json = connection
            .query_row(
                "SELECT approval_json FROM silent_session_control_approvals                  WHERE operator_actor=?1 AND issuance_idempotency_key=?2",
                params![operator_actor, idempotency_key],
                |row| row.get::<_, String>(0),
            )
            .optional()?;
        json.map(|value| serde_json::from_str(&value).map_err(anyhow::Error::from))
            .transpose()
    })
}

pub fn append_redacted_control_audit(
    persistence: &SqlitePersistence,
    audit: &RedactedControlAuditRecord,
) -> anyhow::Result<()> {
    persistence.with_connection_mut(|connection| {
        connection.execute(
            r#"INSERT INTO silent_session_control_audits(
               audit_id,actor,action,project_root,continuity_id,session_id,run_id,audit_json,occurred_at
               ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)"#,
            params![
                audit.audit_id.to_string(),
                audit.actor,
                enum_text(audit.action)?,
                audit.project_root,
                audit.continuity_id,
                audit.session_id.map(|id| id.to_string()),
                audit.run_id.map(|id| id.to_string()),
                serde_json::to_string(audit)?,
                audit.occurred_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    })
}

pub fn load_redacted_control_audit(
    persistence: &SqlitePersistence,
    audit_id: ControlAuditId,
) -> anyhow::Result<Option<RedactedControlAuditRecord>> {
    load_json(
        persistence,
        "SELECT audit_json FROM silent_session_control_audits WHERE audit_id=?1",
        &audit_id.to_string(),
    )
}

pub fn consume_runner_nonce(
    persistence: &SqlitePersistence,
    command: &AuthenticatedRunnerCommand,
    consumed_at: DateTime<Utc>,
) -> anyhow::Result<bool> {
    persistence.with_connection_mut(|connection| {
        let inserted = connection.execute(
            r#"INSERT OR IGNORE INTO silent_session_control_runner_nonces(
               runner_principal_id,nonce,command_id,expires_at,consumed_at
               ) VALUES (?1,?2,?3,?4,?5)"#,
            params![
                command.runner_principal_id,
                command.nonce,
                command.command_id.to_string(),
                command.expires_at.to_rfc3339(),
                consumed_at.to_rfc3339(),
            ],
        )?;
        Ok(inserted == 1)
    })
}

fn load_json<T: serde::de::DeserializeOwned>(
    persistence: &SqlitePersistence,
    sql: &str,
    id: &str,
) -> anyhow::Result<Option<T>> {
    persistence.with_connection_mut(|connection| {
        let json = connection
            .query_row(sql, [id], |row| row.get::<_, String>(0))
            .optional()?;
        json.map(|json| serde_json::from_str(&json).map_err(anyhow::Error::from))
            .transpose()
    })
}

fn enum_text<T: serde::Serialize>(value: T) -> anyhow::Result<String> {
    Ok(serde_json::to_string(&value)?.trim_matches('"').to_string())
}
