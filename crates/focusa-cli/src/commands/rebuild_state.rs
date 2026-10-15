//! Recover the canonical projection from a verified snapshot and durable event prefix.
//! Timestamp guesses never replace the snapshot's replay cursor.

use anyhow::Context;
use clap::Args;
use focusa_core::reducer::reduce_recorded;
use focusa_core::runtime::persistence_sqlite::SqlitePersistence;
use focusa_core::types::{EventLogEntry, FocusaState};
use rusqlite::{Connection, OpenFlags, TransactionBehavior};

#[derive(Args, Debug)]
pub struct RebuildStateArgs {
    /// DB holding the older canonical snapshot and its durable cursor.
    #[arg(long)]
    pub snapshot_db: String,
    /// Live DB holding the same event chain and newer durable events.
    #[arg(long)]
    pub events_db: String,
    /// Legacy hint, accepted for compatibility; the verified cursor determines replay.
    #[arg(long)]
    pub since: Option<String>,
    /// Dry run: rebuild but do not write.
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,
    /// Confirm replacement of the live snapshot and its matching cursor.
    #[arg(long, default_value_t = false)]
    pub confirm: bool,
}

pub async fn run(args: RebuildStateArgs, json_mode: bool) -> anyhow::Result<()> {
    if !args.dry_run && !args.confirm {
        anyhow::bail!(
            "rebuild-state writes the live snapshot and cursor; pass --confirm or use --dry-run"
        );
    }
    let (mut state, source_sequence, source_hash) = {
        let mut conn =
            Connection::open_with_flags(&args.snapshot_db, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        let transaction = conn.transaction()?;
        let (json, sequence, version): (String, i64, i64) = transaction
            .query_row(
                "SELECT s.state_json,c.event_sequence,c.snapshot_version FROM snapshots s
             JOIN snapshot_event_cursors c ON c.name=s.name WHERE s.name='focusa'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .context(
                "snapshot and durable cursor are required; timestamp-only recovery is not safe",
            )?;
        let state: FocusaState = serde_json::from_str(&json).context("snapshot unparsable")?;
        anyhow::ensure!(
            sequence >= 0 && u64::try_from(version)? == state.version,
            "snapshot/cursor version mismatch; snapshot not written"
        );
        anyhow::ensure!(
            sequence > 0 || state.version == 0,
            "nonempty snapshot has no verified durable prefix; snapshot not written"
        );
        let hash: String = if sequence == 0 {
            "GENESIS".into()
        } else {
            transaction
                .query_row(
                    "SELECT event_hash FROM event_hash_chain WHERE chain_index=?1",
                    [sequence - 1],
                    |row| row.get(0),
                )
                .context("snapshot prefix anchor missing")?
        };
        (state, sequence, hash)
    };
    let before_version = state.version;
    let (event_sequence, events_reduced) = {
        let mut conn =
            Connection::open_with_flags(&args.events_db, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        let transaction = conn.transaction()?;
        let tip: i64 = transaction.query_row(
            "SELECT COALESCE(MAX(chain_index)+1,0) FROM event_hash_chain",
            [],
            |row| row.get(0),
        )?;
        anyhow::ensure!(
            source_sequence <= tip,
            "snapshot is ahead of target history; snapshot not written"
        );
        let target_hash: String = if source_sequence == 0 {
            "GENESIS".into()
        } else {
            transaction
                .query_row(
                    "SELECT event_hash FROM event_hash_chain WHERE chain_index=?1",
                    [source_sequence - 1],
                    |row| row.get(0),
                )
                .context("target prefix anchor missing")?
        };
        anyhow::ensure!(
            target_hash == source_hash,
            "snapshot belongs to a different event prefix; snapshot not written"
        );
        let unlinked: i64 = transaction.query_row(
            "SELECT COUNT(*) FROM events e LEFT JOIN event_hash_chain h ON h.event_id=e.event_id WHERE h.event_id IS NULL",
            [], |row| row.get(0))?;
        anyhow::ensure!(
            unlinked == 0,
            "target contains unlinked events; snapshot not written"
        );
        let mut statement = transaction.prepare(
            "SELECT e.event_id,e.ts,e.origin,e.correlation_id,e.payload_json,e.machine_id,
                    e.instance_id,e.session_id,e.thread_id,e.is_observation,h.chain_index+1
             FROM events e JOIN event_hash_chain h ON h.event_id=e.event_id
             WHERE h.chain_index>=?1 ORDER BY h.chain_index",
        )?;
        let rows = statement.query_map([source_sequence], |row| {
            let origin_raw: String = row.get(2)?;
            let origin = match origin_raw.as_str() {
                "worker" => focusa_core::types::SignalOrigin::Worker,
                "daemon" => focusa_core::types::SignalOrigin::Daemon,
                "cli" => focusa_core::types::SignalOrigin::Cli,
                "gui" => focusa_core::types::SignalOrigin::Gui,
                "sync" => focusa_core::types::SignalOrigin::Sync,
                _ => focusa_core::types::SignalOrigin::Adapter,
            };
            let invalid = |column, error: Box<dyn std::error::Error + Send + Sync>| {
                rusqlite::Error::FromSqlConversionFailure(
                    column,
                    rusqlite::types::Type::Text,
                    error,
                )
            };
            let id_raw: String = row.get(0)?;
            let id = uuid::Uuid::parse_str(&id_raw).map_err(|error| invalid(0, Box::new(error)))?;
            let payload_raw: String = row.get(4)?;
            let mut entry = match serde_json::from_str::<EventLogEntry>(&payload_raw) {
                Ok(entry) => entry,
                Err(_) => {
                    let event =
                        serde_json::from_str::<focusa_core::types::FocusaEvent>(&payload_raw)
                            .map_err(|error| invalid(4, Box::new(error)))?;
                    let mut legacy = EventLogEntry::captured(event, origin, None);
                    legacy.temporal =
                        focusa_core::temporal_clock::TemporalActionEnvelope::unavailable(
                            "legacy_rebuild_event_missing_temporal_action_envelope",
                        );
                    legacy
                }
            };
            // Indexed ownership is authoritative, including legacy flat payloads.
            let thread: Option<String> = row.get(8)?;
            entry.id = id;
            entry.timestamp = row.get(1)?;
            entry.origin = origin;
            entry.correlation_id = row.get(3)?;
            entry.machine_id = row.get(5)?;
            entry.instance_id = row
                .get::<_, Option<String>>(6)?
                .map(|value| {
                    uuid::Uuid::parse_str(&value).map_err(|error| invalid(6, Box::new(error)))
                })
                .transpose()?;
            entry.session_id = row
                .get::<_, Option<String>>(7)?
                .map(|value| {
                    uuid::Uuid::parse_str(&value).map_err(|error| invalid(7, Box::new(error)))
                })
                .transpose()?;
            entry.thread_id = thread
                .map(|value| {
                    uuid::Uuid::parse_str(&value).map_err(|error| invalid(8, Box::new(error)))
                })
                .transpose()?;
            entry.is_observation = row.get(9)?;
            Ok((row.get::<_, i64>(10)?, entry))
        })?;
        let mut sequence = source_sequence;
        let mut count = 0_u64;
        for row in rows {
            let (next, entry) = row?;
            anyhow::ensure!(
                next == sequence + 1,
                "event prefix has a gap; snapshot not written"
            );
            state = replay_events(state, std::slice::from_ref(&entry))?;
            sequence = next;
            count += 1;
        }
        anyhow::ensure!(
            sequence == tip,
            "event prefix is incomplete; snapshot not written"
        );
        (sequence, count)
    };
    if !args.dry_run {
        let mut conn =
            Connection::open_with_flags(&args.events_db, OpenFlags::SQLITE_OPEN_READ_WRITE)?;
        conn.busy_timeout(std::time::Duration::from_secs(30))?;
        write_snapshot(&mut conn, &state, event_sequence)?;
    }
    let summary = serde_json::json!({
        "status": if args.dry_run { "rebuilt_dry_run" } else { "rebuilt_and_written" },
        "snapshot_source": args.snapshot_db,
        "replay_order": "durable_cursor",
        "legacy_since_hint": args.since,
        "source_event_sequence": source_sequence,
        "event_sequence": event_sequence,
        "events_scanned": events_reduced,
        "events_reduced": events_reduced,
        "before_version": before_version,
        "after_version": state.version,
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
        state = reduce_recorded(state, entry, entry.machine_id.as_deref(), entry.thread_id)
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

fn write_snapshot(
    conn: &mut Connection,
    state: &FocusaState,
    event_sequence: i64,
) -> anyhow::Result<()> {
    let transaction = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let tip: i64 = transaction.query_row(
        "SELECT COALESCE(MAX(chain_index)+1,0) FROM event_hash_chain",
        [],
        |row| row.get(0),
    )?;
    anyhow::ensure!(
        tip == event_sequence,
        "target history advanced during rebuild; snapshot not written"
    );
    let count: i64 = transaction.query_row(
        "SELECT COUNT(*) FROM snapshots WHERE name='focusa'",
        [],
        |row| row.get(0),
    )?;
    anyhow::ensure!(
        count == 1,
        "rebuild requires exactly one existing focusa snapshot; found {count}; transaction rolled back"
    );
    SqlitePersistence::save_state_on_connection(&transaction, state, event_sequence)?;
    transaction.commit()?;
    Ok(())
}

#[cfg(test)]
#[path = "rebuild_state_tests.rs"]
mod tests;
