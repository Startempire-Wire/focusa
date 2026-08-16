//! `focusa rebuild-state` — recover the canonical FocusaState from the
//! event chain (#263 recovery slice). Loads a snapshot from an older DB
//! (e.g. the pre-retention backup), reduces every newer event from the
//! live DB in order, and writes the rebuilt state back into the live
//! snapshots row. Never starts fresh over stored history.

use clap::Args;
use focusa_core::reducer::reduce_with_meta;
use focusa_core::types::{EventLogEntry, FocusaState};
use serde_json::Value;

#[derive(Args, Debug)]
pub struct RebuildStateArgs {
    /// DB holding the older canonical snapshot.
    #[arg(long)]
    pub snapshot_db: String,
    /// Live DB holding the newer event chain.
    #[arg(long)]
    pub events_db: String,
    /// Only reduce events strictly newer than this RFC3339 timestamp.
    #[arg(long)]
    pub since: String,
    /// Dry run: rebuild but do not write.
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,
}

pub async fn run(args: RebuildStateArgs, json_mode: bool) -> anyhow::Result<()> {
    let snapshot_json: String = {
        let conn = rusqlite::Connection::open(&args.snapshot_db)?;
        conn.query_row(
            "SELECT state_json FROM snapshots WHERE name='focusa'",
            [],
            |row| row.get(0),
        )
        .map_err(|error| anyhow::anyhow!("snapshot read failed: {error}"))?
    };
    let mut state: FocusaState = serde_json::from_str(&snapshot_json)
        .map_err(|error| anyhow::anyhow!("snapshot unparsable: {error}"))?;

    let events: Vec<EventLogEntry> = {
        let conn = rusqlite::Connection::open(&args.events_db)?;
        let mut stmt = conn.prepare(
            "SELECT event_id, ts, origin, correlation_id, payload_json, machine_id,
                    instance_id, session_id, thread_id, is_observation
             FROM events WHERE ts > ?1 ORDER BY ts, rowid",
        )?;
        let rows = stmt.query_map([&args.since], |row| {
            let origin_raw: String = row.get(2)?;
            let origin = match origin_raw.as_str() {
                "worker" => focusa_core::types::SignalOrigin::Worker,
                "daemon" => focusa_core::types::SignalOrigin::Daemon,
                "cli" => focusa_core::types::SignalOrigin::Cli,
                "gui" => focusa_core::types::SignalOrigin::Gui,
                "sync" => focusa_core::types::SignalOrigin::Sync,
                _ => focusa_core::types::SignalOrigin::Adapter,
            };
            let id_raw: String = row.get(0)?;
            Ok(EventLogEntry {
                id: uuid::Uuid::parse_str(&id_raw).unwrap_or(uuid::Uuid::nil()),
                timestamp: row.get(1)?,
                origin,
                correlation_id: row.get(3)?,
                event: serde_json::from_str::<focusa_core::types::FocusaEvent>(
                    row.get::<_, String>(4)?.as_str(),
                )
                .map_err(|error| {
                    rusqlite::Error::FromSqlConversionFailure(
                        4,
                        rusqlite::types::Type::Text,
                        Box::new(error),
                    )
                })?,
                machine_id: row.get(5)?,
                instance_id: row.get(6)?,
                session_id: row.get(7)?,
                thread_id: row.get(8)?,
                is_observation: row.get(9)?,
            })
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()?
    };

    let before_version = state.version;
    let mut reduced = 0usize;
    let mut skipped = 0usize;
    for entry in &events {
        let payload: Value = match serde_json::to_value(&entry.event) {
            Ok(value) => value,
            Err(_) => {
                skipped += 1;
                continue;
            }
        };
        let _ = payload;
        match reduce_with_meta(
            state.clone(),
            entry.event.clone(),
            entry.machine_id.as_deref(),
            entry.thread_id,
            entry.is_observation,
        ) {
            Ok(result) => {
                state = result.new_state;
                reduced += 1;
            }
            Err(_) => {
                skipped += 1;
            }
        }
    }

    if !args.dry_run {
        let conn = rusqlite::Connection::open(&args.events_db)?;
        conn.busy_timeout(std::time::Duration::from_secs(30))?;
        let state_json = serde_json::to_string(&state)?;
        conn.execute(
            "UPDATE snapshots SET version = ?1, ts = ?2, state_json = ?3 WHERE name='focusa'",
            rusqlite::params![
                state.version as i64,
                chrono::Utc::now().to_rfc3339(),
                state_json
            ],
        )?;
    }

    let summary = serde_json::json!({
        "status": if args.dry_run { "rebuilt_dry_run" } else { "rebuilt_and_written" },
        "snapshot_source": args.snapshot_db,
        "events_scanned": events.len(),
        "events_reduced": reduced,
        "events_skipped": skipped,
        "state_version_before": before_version,
        "state_version_after": state.version,
    });
    if json_mode {
        println!("{}", serde_json::to_string_pretty(&summary)?);
    } else {
        println!("{}", serde_json::to_string(&summary)?);
    }
    Ok(())
}
