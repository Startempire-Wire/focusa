//! Workset SQLite persistence — #269 slice 2. Definitions, append-only
//! events, and replay. No execution state (authority separation: #267).

use anyhow::{Context, Result};
use rusqlite::{Connection, OptionalExtension, params};

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
    raw.map(|text| serde_json::from_str(&text).context("invalid stored Workset definition"))
        .transpose()
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
    let rows = stmt.query_map(params![workset_id], |row| row.get::<_, String>(0))?;
    rows.map(|raw| serde_json::from_str(&raw?).context("invalid stored Workset event"))
        .collect()
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
    fn missing_definition_is_distinct_from_corrupt_storage() {
        let conn = Connection::open_in_memory().unwrap();
        ensure_schema(&conn).unwrap();
        assert!(load_definition(&conn, "ws-1", 1).unwrap().is_none());
        for corrupt in ["{bad-json", "{}"] {
            conn.execute(
                "INSERT INTO worksets VALUES ('ws-1', 1, ?1)
                 ON CONFLICT(workset_id, revision) DO UPDATE SET definition_json = excluded.definition_json",
                params![corrupt],
            ).unwrap();
            let error = load_definition(&conn, "ws-1", 1).unwrap_err();
            assert_eq!(error.to_string(), "invalid stored Workset definition");
            let preserved: String = conn.query_row(
                "SELECT definition_json FROM worksets WHERE workset_id = 'ws-1' AND revision = 1",
                [], |row| row.get(0),
            ).unwrap();
            assert_eq!(preserved, corrupt);
        }
    }

    #[test]
    fn corrupt_events_fail_without_inventing_authority_or_repairing_rows() {
        let conn = Connection::open_in_memory().unwrap();
        ensure_schema(&conn).unwrap();
        upsert_definition(&conn, &definition()).unwrap();
        assert!(list_events(&conn, "ws-1").unwrap().is_empty());
        append_event(
            &conn,
            "ws-1",
            &WorksetEvent::RequirementAdmitted {
                requirement_id: "r1".to_string(),
                provider_ref: "test-provider:r1".to_string(),
                evidence_ref: None,
            },
        )
        .unwrap();
        conn.execute(
            "INSERT INTO workset_events (workset_id, event_json, recorded_at)
             VALUES ('ws-1', '{bad-json', 'test-fixture')",
            [],
        )
        .unwrap();
        let corrupt_sequence = conn.last_insert_rowid();
        append_event(
            &conn,
            "ws-1",
            &WorksetEvent::CompletionContracted {
                contract_digest: "valid-test-digest".to_string(),
            },
        )
        .unwrap();
        let error = list_events(&conn, "ws-1").unwrap_err();
        assert_eq!(error.to_string(), "invalid stored Workset event");
        let (count, raw): (i64, String) = conn
            .query_row(
                "SELECT (SELECT count(*) FROM workset_events), event_json
             FROM workset_events WHERE seq = ?1",
                params![corrupt_sequence],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(count, 3);
        assert_eq!(raw, "{bad-json");
        conn.execute("DROP TABLE workset_events", []).unwrap();
        assert!(list_events(&conn, "ws-1").is_err());
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
}
