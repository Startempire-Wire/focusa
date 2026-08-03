use crate::{AdapterFuture, LettaAdapterError, LettaTurnJournal, LettaTurnReceipt};
use std::{
    collections::BTreeMap,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    sync::Mutex,
};

#[derive(Debug)]
pub struct FileTurnJournal {
    path: PathBuf,
    state: Mutex<BTreeMap<String, LettaTurnReceipt>>,
}

impl FileTurnJournal {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, LettaAdapterError> {
        let path = path.as_ref().to_path_buf();
        if path.as_os_str().is_empty() {
            return Err(LettaAdapterError::Journal("path_missing".into()));
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|_| LettaAdapterError::Journal("parent_create_failed".into()))?;
        }
        let mut state = BTreeMap::new();
        if path.exists() {
            let file = File::open(&path)
                .map_err(|_| LettaAdapterError::Journal("journal_open_failed".into()))?;
            for line in BufReader::new(file).lines() {
                let line =
                    line.map_err(|_| LettaAdapterError::Journal("journal_read_failed".into()))?;
                if line.trim().is_empty() {
                    continue;
                }
                let receipt: LettaTurnReceipt = serde_json::from_str(&line)
                    .map_err(|_| LettaAdapterError::Journal("journal_corrupt".into()))?;
                match state.get(&receipt.event_id) {
                    Some(existing) if existing != &receipt => {
                        return Err(LettaAdapterError::Journal(
                            "event_id_content_conflict".into(),
                        ));
                    }
                    Some(_) => {}
                    None => {
                        state.insert(receipt.event_id.clone(), receipt);
                    }
                }
            }
        }
        Ok(Self {
            path,
            state: Mutex::new(state),
        })
    }

    fn append_sync(&self, receipt: &LettaTurnReceipt) -> Result<(), LettaAdapterError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| LettaAdapterError::Journal("journal_lock_poisoned".into()))?;
        if let Some(existing) = state.get(&receipt.event_id) {
            return if existing == receipt {
                Ok(())
            } else {
                Err(LettaAdapterError::Journal(
                    "event_id_content_conflict".into(),
                ))
            };
        }
        let bytes = serde_json::to_vec(receipt)
            .map_err(|_| LettaAdapterError::Journal("receipt_serialize_failed".into()))?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|_| LettaAdapterError::Journal("journal_append_open_failed".into()))?;
        file.write_all(&bytes)
            .and_then(|_| file.write_all(b"\n"))
            .and_then(|_| file.sync_data())
            .map_err(|_| LettaAdapterError::Journal("journal_durable_append_failed".into()))?;
        state.insert(receipt.event_id.clone(), receipt.clone());
        Ok(())
    }
}

impl LettaTurnJournal for FileTurnJournal {
    fn find<'a>(
        &'a self,
        event_id: &'a str,
    ) -> AdapterFuture<'a, Result<Option<LettaTurnReceipt>, LettaAdapterError>> {
        Box::pin(async move {
            self.state
                .lock()
                .map_err(|_| LettaAdapterError::Journal("journal_lock_poisoned".into()))
                .map(|state| state.get(event_id).cloned())
        })
    }

    fn append<'a>(
        &'a self,
        receipt: &'a LettaTurnReceipt,
    ) -> AdapterFuture<'a, Result<(), LettaAdapterError>> {
        Box::pin(async move { self.append_sync(receipt) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn receipt(event_id: &str, digest: &str) -> LettaTurnReceipt {
        LettaTurnReceipt {
            schema: "focusa.letta_turn_receipt.v1".into(),
            request_id: Uuid::now_v7(),
            event_id: event_id.into(),
            provider_agent_id: "letta-agent".into(),
            epoch_id: Uuid::now_v7(),
            response_digest: digest.into(),
            evidence_refs: vec!["evidence:1".into()],
            tool_continuations: 0,
        }
    }

    #[tokio::test]
    async fn durable_receipt_replays_after_reopen_and_duplicate_is_idempotent() {
        let root = std::env::temp_dir().join(format!("focusa-letta-journal-{}", Uuid::now_v7()));
        let path = root.join("turns.jsonl");
        let journal = FileTurnJournal::open(&path).unwrap();
        let receipt = receipt("event-1", "sha256:response");
        journal.append(&receipt).await.unwrap();
        journal.append(&receipt).await.unwrap();
        drop(journal);

        let replayed = FileTurnJournal::open(&path).unwrap();
        assert_eq!(replayed.find("event-1").await.unwrap(), Some(receipt));
        let content = std::fs::read_to_string(&path).unwrap();
        assert_eq!(content.lines().count(), 1);
        let _ = std::fs::remove_dir_all(root);
    }

    #[tokio::test]
    async fn conflicting_event_content_is_rejected() {
        let root = std::env::temp_dir().join(format!("focusa-letta-journal-{}", Uuid::now_v7()));
        let path = root.join("turns.jsonl");
        let journal = FileTurnJournal::open(&path).unwrap();
        journal
            .append(&receipt("event-1", "sha256:a"))
            .await
            .unwrap();
        assert!(matches!(
            journal.append(&receipt("event-1", "sha256:b")).await,
            Err(LettaAdapterError::Journal(reason)) if reason == "event_id_content_conflict"
        ));
        let _ = std::fs::remove_dir_all(root);
    }
}
