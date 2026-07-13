//! AppState-owned typed scoped CRDT ledger service for Spec 104.

use anyhow::{Context, Result};
use focusa_core::scoped_state::{ScopeKeyError, ScopedCrdtMap, ScopedCrdtRecord, WorkstreamKey};
use serde::{Serialize, de::DeserializeOwned};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use tokio::sync::RwLock;

pub struct ScopedCrdtLedger<T> {
    base_dir: PathBuf,
    domain: String,
    actor_id: String,
    partitions: RwLock<HashMap<String, ScopedCrdtMap<T>>>,
}

impl<T> ScopedCrdtLedger<T>
where
    T: Clone + Serialize + DeserializeOwned + Send + Sync + 'static,
{
    pub fn new(
        data_dir: impl AsRef<Path>,
        domain: impl Into<String>,
        actor_id: impl Into<String>,
    ) -> Self {
        Self {
            base_dir: data_dir.as_ref().join("runtime").join("scoped-state"),
            domain: domain.into(),
            actor_id: actor_id.into(),
            partitions: RwLock::new(HashMap::new()),
        }
    }

    fn ledger_path(&self, scope: &WorkstreamKey) -> PathBuf {
        self.base_dir
            .join(&self.domain)
            .join(scope.storage_key())
            .join("events.jsonl")
    }

    fn read_partition(&self, scope: &WorkstreamKey) -> Result<ScopedCrdtMap<T>> {
        let path = self.ledger_path(scope);
        let mut partition = ScopedCrdtMap::new(scope.clone());
        let Ok(text) = fs::read_to_string(&path) else {
            return Ok(partition);
        };
        for (line_number, line) in text.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let record: ScopedCrdtRecord<T> = serde_json::from_str(line)
                .with_context(|| format!("decode {} line {}", path.display(), line_number + 1))?;
            partition.apply(record).map_err(anyhow::Error::from)?;
        }
        Ok(partition)
    }

    fn append_event(&self, record: &ScopedCrdtRecord<T>) -> Result<()> {
        let path = self.ledger_path(&record.scope);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create scoped ledger parent {}", parent.display()))?;
        }
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .with_context(|| format!("open scoped ledger {}", path.display()))?;
        serde_json::to_writer(&mut file, record)?;
        file.write_all(b"\n")?;
        file.sync_data()?;
        Ok(())
    }

    async fn ensure_loaded(&self, scope: &WorkstreamKey) -> Result<String> {
        let key = scope.storage_key();
        if self.partitions.read().await.contains_key(&key) {
            return Ok(key);
        }
        let loaded = self.read_partition(scope)?;
        let mut partitions = self.partitions.write().await;
        partitions.entry(key.clone()).or_insert(loaded);
        Ok(key)
    }

    pub async fn upsert(
        &self,
        scope: WorkstreamKey,
        record_id: impl Into<String>,
        value: T,
    ) -> Result<ScopedCrdtRecord<T>> {
        let record_id = record_id.into();
        let key = self.ensure_loaded(&scope).await?;
        let mut partitions = self.partitions.write().await;
        let partition = partitions
            .get_mut(&key)
            .ok_or_else(|| anyhow::anyhow!("scoped partition disappeared"))?;
        let next = match partition.records.get(&record_id) {
            Some(current) => current.revise(&self.actor_id, value, false)?,
            None => ScopedCrdtRecord::new(scope, record_id, &self.actor_id, value)?,
        };
        self.append_event(&next)?;
        partition.apply(next.clone())?;
        Ok(next)
    }

    pub async fn tombstone(&self, scope: &WorkstreamKey, record_id: &str) -> Result<bool> {
        let key = self.ensure_loaded(scope).await?;
        let mut partitions = self.partitions.write().await;
        let partition = partitions
            .get_mut(&key)
            .ok_or_else(|| anyhow::anyhow!("scoped partition disappeared"))?;
        let Some(current) = partition.records.get(record_id).cloned() else {
            return Ok(false);
        };
        let next = current.revise(&self.actor_id, current.value.clone(), true)?;
        self.append_event(&next)?;
        partition.apply(next)?;
        Ok(true)
    }

    pub async fn get(
        &self,
        scope: &WorkstreamKey,
        record_id: &str,
    ) -> Result<Option<ScopedCrdtRecord<T>>> {
        let key = self.ensure_loaded(scope).await?;
        Ok(self
            .partitions
            .read()
            .await
            .get(&key)
            .and_then(|partition| partition.records.get(record_id))
            .filter(|record| !record.tombstone)
            .cloned())
    }

    pub async fn recent(
        &self,
        scope: &WorkstreamKey,
        limit: usize,
    ) -> Result<Vec<ScopedCrdtRecord<T>>> {
        let key = self.ensure_loaded(scope).await?;
        let partitions = self.partitions.read().await;
        let Some(partition) = partitions.get(&key) else {
            return Ok(Vec::new());
        };
        let mut records = partition.active_records().cloned().collect::<Vec<_>>();
        records.sort_by_key(|record| record.updated_at);
        if records.len() > limit {
            records.drain(0..records.len() - limit);
        }
        Ok(records)
    }

    pub async fn reconcile(
        &self,
        scope: &WorkstreamKey,
        incoming: Vec<ScopedCrdtRecord<T>>,
    ) -> Result<usize> {
        let key = self.ensure_loaded(scope).await?;
        let mut partitions = self.partitions.write().await;
        let partition = partitions
            .get_mut(&key)
            .ok_or_else(|| anyhow::anyhow!("scoped partition disappeared"))?;
        let mut applied = 0;
        for record in incoming {
            if &record.scope != scope {
                return Err(ScopeKeyError::ScopeMismatch.into());
            }
            let merged = match partition.records.get(&record.record_id) {
                Some(current) => current.reconcile(&record)?,
                None => record,
            };
            let changed = partition
                .records
                .get(&merged.record_id)
                .map(|current| serde_json::to_vec(current).ok() != serde_json::to_vec(&merged).ok())
                .unwrap_or(true);
            if changed {
                self.append_event(&merged)?;
                partition.apply(merged)?;
                applied += 1;
            }
        }
        Ok(applied)
    }

    pub async fn partition_count(&self) -> usize {
        self.partitions.read().await.len()
    }

    pub fn legacy_global_path(&self, file_name: &str) -> PathBuf {
        self.base_dir
            .parent()
            .unwrap_or(&self.base_dir)
            .join("legacy-quarantine")
            .join(file_name)
    }
}
