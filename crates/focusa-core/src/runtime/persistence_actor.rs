//! Bounded single-writer persistence actor for daemon/API state snapshots.
//!
//! SQLite serialization and filesystem writes never run on Tokio core workers.
//! Ordinary writes coalesce to the latest state while preserving every event-log
//! entry; checkpoint writes receive an acknowledgement only after durable commit.

use crate::runtime::persistence_sqlite::SqlitePersistence;
use crate::types::{EventLogEntry, FocusaState};
use serde::Serialize;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use tokio::sync::{mpsc, oneshot};

const PERSISTENCE_QUEUE_CAPACITY: usize = 64;

#[derive(Default)]
pub struct PersistenceActorMetrics {
    queue_depth: AtomicU64,
    queue_depth_max: AtomicU64,
    batches_total: AtomicU64,
    requests_coalesced_total: AtomicU64,
    failures_total: AtomicU64,
    saturation_total: AtomicU64,
    last_write_duration_ms: AtomicU64,
    max_write_duration_ms: AtomicU64,
    snapshot_bytes: AtomicU64,
    database_bytes: AtomicU64,
    wal_bytes: AtomicU64,
}

#[derive(Clone, Debug, Serialize)]
pub struct PersistenceActorMetricsSnapshot {
    pub queue_depth: u64,
    pub queue_depth_max: u64,
    pub batches_total: u64,
    pub requests_coalesced_total: u64,
    pub failures_total: u64,
    pub saturation_total: u64,
    pub last_write_duration_ms: u64,
    pub max_write_duration_ms: u64,
    /// Serialized canonical FocusaState payload bytes.
    pub snapshot_bytes: u64,
    /// Main SQLite file bytes, including tables, indexes, and free pages.
    pub database_bytes: u64,
    pub wal_bytes: u64,
}

impl PersistenceActorMetrics {
    pub fn snapshot(&self) -> PersistenceActorMetricsSnapshot {
        PersistenceActorMetricsSnapshot {
            queue_depth: self.queue_depth.load(Ordering::Acquire),
            queue_depth_max: self.queue_depth_max.load(Ordering::Acquire),
            batches_total: self.batches_total.load(Ordering::Acquire),
            requests_coalesced_total: self.requests_coalesced_total.load(Ordering::Acquire),
            failures_total: self.failures_total.load(Ordering::Acquire),
            saturation_total: self.saturation_total.load(Ordering::Acquire),
            last_write_duration_ms: self.last_write_duration_ms.load(Ordering::Acquire),
            max_write_duration_ms: self.max_write_duration_ms.load(Ordering::Acquire),
            snapshot_bytes: self.snapshot_bytes.load(Ordering::Acquire),
            database_bytes: self.database_bytes.load(Ordering::Acquire),
            wal_bytes: self.wal_bytes.load(Ordering::Acquire),
        }
    }
}

struct PersistenceRequest {
    events: Vec<EventLogEntry>,
    state: Option<FocusaState>,
    acknowledge: Option<oneshot::Sender<Result<(), String>>>,
}

#[derive(Clone)]
pub struct PersistenceActor {
    tx: mpsc::Sender<PersistenceRequest>,
    metrics: Arc<PersistenceActorMetrics>,
}

impl PersistenceActor {
    pub fn start(persistence: SqlitePersistence) -> Self {
        let (tx, mut rx) = mpsc::channel::<PersistenceRequest>(PERSISTENCE_QUEUE_CAPACITY);
        let metrics = Arc::new(PersistenceActorMetrics::default());
        let actor_metrics = metrics.clone();

        tokio::spawn(async move {
            while let Some(first) = rx.recv().await {
                actor_metrics.queue_depth.fetch_sub(1, Ordering::AcqRel);
                let mut requests = vec![first];
                while let Ok(next) = rx.try_recv() {
                    actor_metrics.queue_depth.fetch_sub(1, Ordering::AcqRel);
                    requests.push(next);
                }
                actor_metrics
                    .requests_coalesced_total
                    .fetch_add(requests.len().saturating_sub(1) as u64, Ordering::AcqRel);

                let mut events = Vec::new();
                for request in &mut requests {
                    events.append(&mut request.events);
                }
                let latest_state = requests
                    .iter()
                    .rev()
                    .find_map(|request| request.state.clone());
                let persistence_for_write = persistence.clone();
                let started = Instant::now();
                let result =
                    tokio::task::spawn_blocking(move || -> anyhow::Result<(u64, u64, u64)> {
                        if let Some(latest_state) = &latest_state {
                            persistence_for_write
                                .persist_event_batch_and_state(&events, latest_state)?;
                        } else {
                            persistence_for_write.append_event_batch(&events)?;
                        }
                        let snapshot_bytes = persistence_for_write.snapshot_payload_bytes()?;
                        let database_bytes =
                            std::fs::metadata(persistence_for_write.data_dir.join("focusa.sqlite"))
                                .map(|metadata| metadata.len())
                                .unwrap_or(0);
                        let wal_bytes = std::fs::metadata(
                            persistence_for_write.data_dir.join("focusa.sqlite-wal"),
                        )
                        .map(|metadata| metadata.len())
                        .unwrap_or(0);
                        Ok((snapshot_bytes, database_bytes, wal_bytes))
                    })
                    .await
                    .map_err(|error| anyhow::anyhow!("persistence worker join failed: {error}"))
                    .and_then(|result| result);

                let duration_ms = started.elapsed().as_millis() as u64;
                actor_metrics
                    .last_write_duration_ms
                    .store(duration_ms, Ordering::Release);
                actor_metrics
                    .max_write_duration_ms
                    .fetch_max(duration_ms, Ordering::AcqRel);
                actor_metrics.batches_total.fetch_add(1, Ordering::AcqRel);
                if let Ok((snapshot_bytes, database_bytes, wal_bytes)) = &result {
                    actor_metrics
                        .snapshot_bytes
                        .store(*snapshot_bytes, Ordering::Release);
                    actor_metrics
                        .database_bytes
                        .store(*database_bytes, Ordering::Release);
                    actor_metrics.wal_bytes.store(*wal_bytes, Ordering::Release);
                } else {
                    actor_metrics.failures_total.fetch_add(1, Ordering::AcqRel);
                }
                let acknowledgement = result.map(|_| ()).map_err(|error| error.to_string());
                for request in requests {
                    if let Some(sender) = request.acknowledge {
                        let _ = sender.send(acknowledgement.clone());
                    }
                }
            }
        });

        Self { tx, metrics }
    }

    pub fn metrics(&self) -> PersistenceActorMetricsSnapshot {
        self.metrics.snapshot()
    }

    async fn enqueue(
        &self,
        events: Vec<EventLogEntry>,
        state: Option<FocusaState>,
        acknowledge: bool,
    ) -> anyhow::Result<()> {
        let (acknowledge_tx, acknowledge_rx) = if acknowledge {
            let (tx, rx) = oneshot::channel();
            (Some(tx), Some(rx))
        } else {
            (None, None)
        };
        let depth = self.metrics.queue_depth.fetch_add(1, Ordering::AcqRel) + 1;
        self.metrics
            .queue_depth_max
            .fetch_max(depth, Ordering::AcqRel);
        if depth >= PERSISTENCE_QUEUE_CAPACITY as u64 {
            self.metrics.saturation_total.fetch_add(1, Ordering::AcqRel);
        }
        if let Err(error) = self
            .tx
            .send(PersistenceRequest {
                events,
                state,
                acknowledge: acknowledge_tx,
            })
            .await
        {
            self.metrics.queue_depth.fetch_sub(1, Ordering::AcqRel);
            return Err(anyhow::anyhow!("persistence actor unavailable: {error}"));
        }
        if let Some(receiver) = acknowledge_rx {
            receiver
                .await
                .map_err(|error| anyhow::anyhow!("persistence acknowledgement lost: {error}"))?
                .map_err(anyhow::Error::msg)?;
        }
        Ok(())
    }

    pub async fn persist_ordinary(
        &self,
        events: Vec<EventLogEntry>,
        state: FocusaState,
    ) -> anyhow::Result<()> {
        self.enqueue(events, Some(state), false).await
    }

    pub async fn persist_checkpoint(
        &self,
        events: Vec<EventLogEntry>,
        state: FocusaState,
    ) -> anyhow::Result<()> {
        self.enqueue(events, Some(state), true).await
    }

    pub async fn append_events_checkpoint(&self, events: Vec<EventLogEntry>) -> anyhow::Result<()> {
        self.enqueue(events, None, true).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::FocusaConfig;
    use std::time::Duration;
    use uuid::Uuid;

    fn test_persistence() -> (SqlitePersistence, std::path::PathBuf) {
        let root =
            std::env::temp_dir().join(format!("focusa-persistence-actor-{}", Uuid::now_v7()));
        std::fs::create_dir_all(&root).expect("create test data dir");
        let config = FocusaConfig {
            data_dir: root.to_string_lossy().into_owned(),
            ..FocusaConfig::default()
        };
        let persistence = SqlitePersistence::new(&config).expect("create sqlite persistence");
        (persistence, root)
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn coalesces_ordinary_snapshots_and_checkpoint_ack_is_restart_durable() {
        let (persistence, root) = test_persistence();
        let actor = PersistenceActor::start(persistence.clone());
        let mut tasks = Vec::new();
        for version in 1..=24_u64 {
            let actor = actor.clone();
            let state = FocusaState {
                version,
                ..FocusaState::default()
            };
            tasks.push(tokio::spawn(async move {
                actor.persist_ordinary(Vec::new(), state).await
            }));
        }
        for task in tasks {
            task.await
                .expect("ordinary task joined")
                .expect("ordinary enqueue");
        }
        let final_state = FocusaState {
            version: 25,
            ..FocusaState::default()
        };
        actor
            .persist_checkpoint(Vec::new(), final_state)
            .await
            .expect("checkpoint acknowledgement");

        let restored = persistence
            .load_state()
            .expect("load state")
            .expect("saved state exists");
        assert_eq!(restored.version, 25);
        let metrics = actor.metrics();
        assert!(metrics.batches_total > 0);
        assert!(metrics.requests_coalesced_total > 0);
        assert_eq!(metrics.failures_total, 0);
        assert!(metrics.snapshot_bytes > 0);
        assert!(metrics.database_bytes >= metrics.snapshot_bytes);
        let _ = std::fs::remove_dir_all(root);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn large_snapshot_write_does_not_stall_tokio_timer() {
        let (persistence, root) = test_persistence();
        let actor = PersistenceActor::start(persistence);
        let mut state = FocusaState {
            version: 1,
            ..FocusaState::default()
        };
        state
            .anticipated_context
            .extend((0..20_000).map(|index| format!("context-{index}-{}", "x".repeat(128))));

        let write = tokio::spawn({
            let actor = actor.clone();
            async move { actor.persist_checkpoint(Vec::new(), state).await }
        });
        let timer_started = Instant::now();
        tokio::time::sleep(Duration::from_millis(10)).await;
        assert!(
            timer_started.elapsed() < Duration::from_millis(250),
            "Tokio timer stalled behind SQLite serialization"
        );
        write
            .await
            .expect("write task joined")
            .expect("large checkpoint");
        let _ = std::fs::remove_dir_all(root);
    }
}
