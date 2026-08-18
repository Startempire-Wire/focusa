//! Workset SQLite persistence — #269 slice 2. Definitions, append-only
//! events, and replay. No execution state (authority separation: #267).

use anyhow::Result;
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::Value;

use crate::workset_ledger::{WorksetDefinition, WorksetEvent};

pub fn ensure_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS worksets (
            workset_id TEXT NOT NULL,
            revision INTEGER NOT NULL,
            definition_json TEXT NOT NULL,
            PRIMARY KEY (workset_id, revision)
        );
        CREATE TABLE IF NOT EXISTS workset_events (
            seq INTEGER PRIMARY KEY AUTOINCREMENT,
            workset_id TEXT NOT NULL,
            event_json TEXT NOT NULL,
            recorded_at TEXT NOT NULL
        );
        "#,
    )?;
    Ok(())
}

pub fn upsert_definition(conn: &Connection, definition: &WorksetDefinition) -> Result<()> {
    conn.execute(
        "INSERT INTO worksets (workset_id, revision, definition_json)
         VALUES (?1, ?2, ?3)
         ON CONFLICT(workset_id, revision) DO UPDATE SET definition_json = excluded.definition_json",
        params![
            definition.workset_id,
            definition.revision as i64,
            serde_json::to_string(definition)?
        ],
    )?;
    Ok(())
}

pub fn load_definition(
    conn: &Connection,
    workset_id: &str,
    revision: u64,
) -> Result<Option<WorksetDefinition>> {
    let raw: Option<String> = conn
        .query_row(
            "SELECT definition_json FROM worksets WHERE workset_id = ?1 AND revision = ?2",
            params![workset_id, revision as i64],
            |row| row.get(0),
        )
        .optional()?;
    Ok(raw.and_then(|text: String| serde_json::from_str(&text).ok()))
}

pub fn append_event(conn: &Connection, workset_id: &str, event: &WorksetEvent) -> Result<i64> {
    conn.execute(
        "INSERT INTO workset_events (workset_id, event_json, recorded_at)
         VALUES (?1, ?2, ?3)",
        params![
            workset_id,
            serde_json::to_string(event)?,
            chrono::Utc::now().to_rfc3339()
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn list_events(conn: &Connection, workset_id: &str) -> Result<Vec<WorksetEvent>> {
    let mut stmt =
        conn.prepare("SELECT event_json FROM workset_events WHERE workset_id = ?1 ORDER BY seq")?;
    let rows = stmt.query_map(params![workset_id], |row| {
        let raw: String = row.get(0)?;
        Ok(
            serde_json::from_str::<WorksetEvent>(&raw).unwrap_or_else(|_| {
                WorksetEvent::CompletionContracted {
                    contract_digest: "unparsable".to_string(),
                }
            }),
        )
    })?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workset_ledger::{CompletionContract, WorksetScope};

    fn definition() -> WorksetDefinition {
        WorksetDefinition {
            schema: crate::workset_ledger::WORKSET_LEDGER_SCHEMA.to_string(),
            workset_id: "ws-1".to_string(),
            revision: 1,
            scope: WorksetScope {
                project_root: "/r".to_string(),
                continuity_id: "c".to_string(),
            },
            completion_contract: CompletionContract {
                required_requirement_ids: vec!["r1".to_string()],
                release_gate_ref: None,
            },
        }
    }

    #[test]
    fn definition_and_events_roundtrip() {
        let conn = Connection::open_in_memory().unwrap();
        ensure_schema(&conn).unwrap();
        upsert_definition(&conn, &definition()).unwrap();
        let loaded = load_definition(&conn, "ws-1", 1).unwrap().expect("exists");
        assert_eq!(loaded.workset_id, "ws-1");
        append_event(
            &conn,
            "ws-1",
            &WorksetEvent::RequirementAdmitted {
                requirement_id: "r1".to_string(),
                provider_ref: "p1".to_string(),
                evidence_ref: None,
            },
        )
        .unwrap();
        let events = list_events(&conn, "ws-1").unwrap();
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn replay_projection_from_store_settles() {
        let conn = Connection::open_in_memory().unwrap();
        ensure_schema(&conn).unwrap();
        let definition = definition();
        upsert_definition(&conn, &definition).unwrap();
        append_event(
            &conn,
            "ws-1",
            &WorksetEvent::RequirementAdmitted {
                requirement_id: "r1".to_string(),
                provider_ref: "p1".to_string(),
                evidence_ref: None,
            },
        )
        .unwrap();
        append_event(
            &conn,
            "ws-1",
            &WorksetEvent::RequirementDisposed {
                requirement_id: "r1".to_string(),
                disposition: crate::workset_ledger::RequirementDisposition::Met,
                evidence_ref: None,
            },
        )
        .unwrap();
        let events = list_events(&conn, "ws-1").unwrap();
        let projection = crate::workset_ledger::replay_projection(&definition, &events).unwrap();
        assert!(projection.settled);
    }

    #[test]
    fn unparsable_event_does_not_poison_the_ledger() {
        let conn = Connection::open_in_memory().unwrap();
        ensure_schema(&conn).unwrap();
        conn.execute(
            "INSERT INTO workset_events (workset_id, event_json, recorded_at) VALUES ('ws-1', 'garbage', 't')",
            [],
        )
        .unwrap();
        let events = list_events(&conn, "ws-1").unwrap();
        assert_eq!(events.len(), 1); // survived as a placeholder, never panics
    }

    #[allow(dead_code)]
    fn _value_marker(_v: Value) {}
}
