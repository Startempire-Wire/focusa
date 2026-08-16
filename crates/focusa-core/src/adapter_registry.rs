//! Adapter registry — #254 slice 10 foundation. Harnesses/models register
//! capability sets; CallGraph dispatch routes against this registry. The
//! registry is ledger-backed: routing survives restarts and liveness is
//! observable (healthy flag + last_seen).

use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

pub const ADAPTER_REGISTRY_SCHEMA: &str = "focusa.adapter_registry.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdapterRecord {
    pub adapter_id: String,
    pub model: String,
    pub harness: String,
    pub capabilities: Vec<String>,
    pub healthy: bool,
    pub last_seen: String,
}

pub fn ensure_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS adapter_registry (
            adapter_id TEXT NOT NULL,
            model TEXT NOT NULL,
            harness TEXT NOT NULL,
            capabilities_json TEXT NOT NULL,
            healthy INTEGER NOT NULL DEFAULT 1,
            last_seen TEXT NOT NULL,
            PRIMARY KEY (adapter_id, model)
        );
        "#,
    )?;
    Ok(())
}

pub fn upsert_adapter(conn: &Connection, record: &AdapterRecord) -> Result<()> {
    conn.execute(
        "INSERT INTO adapter_registry
         (adapter_id, model, harness, capabilities_json, healthy, last_seen)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(adapter_id, model) DO UPDATE SET
            harness = excluded.harness,
            capabilities_json = excluded.capabilities_json,
            healthy = excluded.healthy,
            last_seen = excluded.last_seen",
        params![
            record.adapter_id,
            record.model,
            record.harness,
            serde_json::to_string(&record.capabilities)?,
            record.healthy as i64,
            record.last_seen,
        ],
    )?;
    Ok(())
}

pub fn list_adapters(conn: &Connection) -> Result<Vec<AdapterRecord>> {
    let mut stmt = conn.prepare(
        "SELECT adapter_id, model, harness, capabilities_json, healthy, last_seen
         FROM adapter_registry ORDER BY last_seen DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        let capabilities: Vec<String> =
            serde_json::from_str(&row.get::<_, String>(3)?).unwrap_or_default();
        Ok(AdapterRecord {
            adapter_id: row.get(0)?,
            model: row.get(1)?,
            harness: row.get(2)?,
            capabilities,
            healthy: row.get::<_, i64>(4)? != 0,
            last_seen: row.get(5)?,
        })
    })?;
    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(Into::into)
}

pub fn set_healthy(conn: &Connection, adapter_id: &str, model: &str, healthy: bool) -> Result<()> {
    conn.execute(
        "UPDATE adapter_registry SET healthy = ?3 WHERE adapter_id = ?1 AND model = ?2",
        params![adapter_id, model, healthy as i64],
    )?;
    Ok(())
}

pub fn load_adapter(conn: &Connection, adapter_id: &str, model: &str) -> Result<Option<AdapterRecord>> {
    conn.query_row(
        "SELECT adapter_id, model, harness, capabilities_json, healthy, last_seen
         FROM adapter_registry WHERE adapter_id = ?1 AND model = ?2",
        params![adapter_id, model],
        |row| {
            let capabilities: Vec<String> =
                serde_json::from_str(&row.get::<_, String>(3)?).unwrap_or_default();
            Ok(AdapterRecord {
                adapter_id: row.get(0)?,
                model: row.get(1)?,
                harness: row.get(2)?,
                capabilities,
                healthy: row.get::<_, i64>(4)? != 0,
                last_seen: row.get(5)?,
            })
        },
    )
    .optional()
    .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(id: &str, model: &str, caps: &[&str]) -> AdapterRecord {
        AdapterRecord {
            adapter_id: id.to_string(),
            model: model.to_string(),
            harness: "pi".to_string(),
            capabilities: caps.iter().map(|c| c.to_string()).collect(),
            healthy: true,
            last_seen: "2026-08-16T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn upsert_and_list_roundtrip() {
        let conn = Connection::open_in_memory().unwrap();
        ensure_schema(&conn).unwrap();
        upsert_adapter(&conn, &record("pi", "m1", &["shell", "browser"])).unwrap();
        upsert_adapter(&conn, &record("uiai", "m2", &["browser"])).unwrap();
        let adapters = list_adapters(&conn).unwrap();
        assert_eq!(adapters.len(), 2);
        let loaded = load_adapter(&conn, "pi", "m1").unwrap().unwrap();
        assert_eq!(loaded.capabilities, vec!["shell", "browser"]);
    }

    #[test]
    fn upsert_is_idempotent_and_health_toggles() {
        let conn = Connection::open_in_memory().unwrap();
        ensure_schema(&conn).unwrap();
        upsert_adapter(&conn, &record("pi", "m1", &["shell"])).unwrap();
        upsert_adapter(&conn, &record("pi", "m1", &["shell"])).unwrap();
        assert_eq!(list_adapters(&conn).unwrap().len(), 1);
        set_healthy(&conn, "pi", "m1", false).unwrap();
        assert!(!load_adapter(&conn, "pi", "m1").unwrap().unwrap().healthy);
    }
}
