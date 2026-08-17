//! Direction Workbench ledger — #291 slice 2: append-only direction
//! operations with receipts. The Workbench projects this ledger; every
//! operation is verified before it is recorded (verify_operation).

use anyhow::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::direction_operations::{verify_operation, DirectionOperation};

pub fn ensure_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS direction_operations (
            operation_id TEXT PRIMARY KEY,
            operation_json TEXT NOT NULL,
            receipt_json TEXT NOT NULL,
            recorded_at TEXT NOT NULL
        );
        "#,
    )?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirectionReceipt {
    pub schema: String,
    pub operation_id: String,
    pub operation: DirectionOperation,
    pub recorded_at: String,
}

/// Record a verified operation + receipt. Returns Err on verification
/// failure — untyped/free-text operations never enter the ledger.
pub fn record_operation(conn: &Connection, operation: &DirectionOperation) -> Result<DirectionReceipt> {
    verify_operation(operation)
        .map_err(|reason| anyhow::anyhow!("direction operation rejected: {reason}"))?;
    let operation_id = uuid::Uuid::now_v7().to_string();
    let recorded_at = chrono::Utc::now().to_rfc3339();
    let receipt = DirectionReceipt {
        schema: "focusa.direction_receipt.v1".to_string(),
        operation_id: operation_id.clone(),
        operation: operation.clone(),
        recorded_at: recorded_at.clone(),
    };
    conn.execute(
        "INSERT INTO direction_operations (operation_id, operation_json, receipt_json, recorded_at)
         VALUES (?1, ?2, ?3, ?4)",
        params![
            operation_id,
            serde_json::to_string(operation)?,
            serde_json::to_string(&receipt)?,
            recorded_at,
        ],
    )?;
    Ok(receipt)
}

pub fn list_operations(conn: &Connection) -> Result<Vec<DirectionReceipt>> {
    let mut stmt = conn.prepare(
        "SELECT receipt_json FROM direction_operations ORDER BY recorded_at",
    )?;
    let rows = stmt.query_map([], |row| {
        let receipt: DirectionReceipt = serde_json::from_str(&row.get::<_, String>(0)?)
            .unwrap_or_else(|_| DirectionReceipt {
                schema: "focusa.direction_receipt.v1".to_string(),
                operation_id: "unparsable".to_string(),
                operation: DirectionOperation::ReviewDecision {
                    decision_ref: "unparsable".to_string(),
                    outcome: "unparsable".to_string(),
                    feedback: "unparsable".to_string(),
                },
                recorded_at: String::new(),
            });
        Ok(receipt)
    })?;
    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn steer(evidence: Option<&str>) -> DirectionOperation {
        DirectionOperation::Steer {
            target_ref: "wp-1".to_string(),
            direction: "prioritize compaction".to_string(),
            rationale: "quota".to_string(),
            scope: "workpoint".to_string(),
            evidence_ref: evidence.map(|e| e.to_string()),
        }
    }

    #[test]
    fn verified_operation_records_with_receipt() {
        let conn = Connection::open_in_memory().unwrap();
        ensure_schema(&conn).unwrap();
        let receipt = record_operation(&conn, &steer(Some("docs/evidence/e.md"))).unwrap();
        assert_eq!(receipt.schema, "focusa.direction_receipt.v1");
        assert!(!receipt.operation_id.is_empty());
        assert_eq!(list_operations(&conn).unwrap().len(), 1);
    }

    #[test]
    fn unverified_operation_never_enters_the_ledger() {
        let conn = Connection::open_in_memory().unwrap();
        ensure_schema(&conn).unwrap();
        let result = record_operation(&conn, &steer(None));
        assert!(result.is_err());
        assert!(list_operations(&conn).unwrap().is_empty());
    }
}
