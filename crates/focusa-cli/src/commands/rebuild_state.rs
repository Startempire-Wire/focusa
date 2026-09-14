//! `focusa rebuild-state` — recover the canonical FocusaState from the
//! event chain (#263 recovery slice). Loads a snapshot from an older DB
//! (e.g. the pre-retention backup), reduces every newer event from the
//! live DB in order, and writes the rebuilt state back into the live
//! snapshots row. Never starts fresh over stored history.

use anyhow::Context;
use clap::Args;
use focusa_core::reducer::reduce_with_meta;
use focusa_core::types::{EventLogEntry, FocusaState};
use rusqlite::{Connection, OpenFlags};

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
    /// Confirm replacement of the live snapshots row.
    #[arg(long, default_value_t = false)]
    pub confirm: bool,
}

pub async fn run(args: RebuildStateArgs, json_mode: bool) -> anyhow::Result<()> {
    if !args.dry_run && !args.confirm {
        anyhow::bail!(
            "rebuild-state writes the live snapshots row; pass --confirm or use --dry-run"
        );
    }
    let snapshot_json: String = {
        let conn =
            Connection::open_with_flags(&args.snapshot_db, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        conn.query_row(
            "SELECT state_json FROM snapshots WHERE name='focusa'",
            [],
            |row| row.get(0),
        )
        .map_err(|error| anyhow::anyhow!("snapshot read failed: {error}"))?
    };
    let state: FocusaState = serde_json::from_str(&snapshot_json)
        .map_err(|error| anyhow::anyhow!("snapshot unparsable: {error}"))?;

    let events: Vec<EventLogEntry> = {
        let conn = Connection::open_with_flags(&args.events_db, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
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
                temporal: Default::default(),
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
    let state = replay_events(state, &events)?;

    if !args.dry_run {
        let mut conn =
            Connection::open_with_flags(&args.events_db, OpenFlags::SQLITE_OPEN_READ_WRITE)?;
        conn.busy_timeout(std::time::Duration::from_secs(30))?;
        write_snapshot(&mut conn, &state)?;
    }

    let summary = serde_json::json!({
        "status": if args.dry_run { "rebuilt_dry_run" } else { "rebuilt_and_written" },
        "snapshot_source": args.snapshot_db,
        "events_scanned": events.len(),
        "events_reduced": events.len(),
        "events_skipped": 0,
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

fn replay_events(mut state: FocusaState, events: &[EventLogEntry]) -> anyhow::Result<FocusaState> {
    for entry in events {
        state = reduce_with_meta(
            state,
            entry.event.clone(),
            entry.machine_id.as_deref(),
            entry.thread_id,
            entry.is_observation,
        )
        .with_context(|| {
            format!(
                "rebuild replay rejected event {}; snapshot not written",
                entry.id
            )
        })?
        .new_state;
    }
    Ok(state)
}

fn write_snapshot(conn: &mut Connection, state: &FocusaState) -> anyhow::Result<()> {
    let transaction = conn.transaction()?;
    let affected = transaction.execute(
        "UPDATE snapshots SET version = ?1, ts = ?2, state_json = ?3 WHERE name='focusa'",
        rusqlite::params![
            state.version as i64,
            chrono::Utc::now().to_rfc3339(),
            serde_json::to_string(state)?
        ],
    )?;
    anyhow::ensure!(
        affected == 1,
        "rebuild requires exactly one existing focusa snapshot; found {affected}; transaction rolled back"
    );
    transaction.commit()?;
    Ok(())
}

#[cfg(test)]
#[path = "rebuild_state_tests.rs"]
mod tests;
