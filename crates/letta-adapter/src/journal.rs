use crate::{
    AdapterFuture, LettaAdapterError, LettaTurnIntent, LettaTurnJournal, LettaTurnReceipt,
    LettaTurnRequest,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    collections::BTreeMap,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    sync::Mutex,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LettaRecoveryGuidance {
    pub schema: String,
    pub event_id: String,
    pub request_id: uuid::Uuid,
    pub status: String,
    pub next_action: String,
    pub automatic_retry_budget: u32,
}

#[derive(Debug)]
pub struct FileTurnJournal {
    path: PathBuf,
    intents_path: PathBuf,
    intents: Mutex<BTreeMap<String, LettaTurnIntent>>,
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
        let intents_path = path.with_extension("intents.jsonl");
        let intents =
            read_records::<LettaTurnIntent>(&intents_path, |intent| intent.event_id.clone())?;
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
            intents_path,
            intents: Mutex::new(intents),
            state: Mutex::new(state),
        })
    }

    fn reserve_sync(&self, request: &LettaTurnRequest) -> Result<uuid::Uuid, LettaAdapterError> {
        let candidate = LettaTurnIntent::from(request);
        let mut intents = self
            .intents
            .lock()
            .map_err(|_| LettaAdapterError::Journal("journal_lock_poisoned".into()))?;
        match intents.get(&request.event_id) {
            Some(existing)
                if existing.provider_agent_id != candidate.provider_agent_id
                    || existing.epoch_id != candidate.epoch_id
                    || existing.input_digest != candidate.input_digest =>
            {
                Err(LettaAdapterError::Journal(
                    "event_id_content_conflict".into(),
                ))
            }
            Some(existing) => Ok(existing.request_id),
            None => {
                append_record(&self.intents_path, &candidate)?;
                intents.insert(candidate.event_id.clone(), candidate);
                Ok(request.request_id)
            }
        }
    }

    pub fn recovery_guidance(
        &self,
        event_id: &str,
    ) -> Result<Option<LettaRecoveryGuidance>, LettaAdapterError> {
        let intents = self
            .intents
            .lock()
            .map_err(|_| LettaAdapterError::Journal("journal_lock_poisoned".into()))?;
        let Some(intent) = intents.get(event_id) else {
            return Ok(None);
        };
        let settled = self
            .state
            .lock()
            .map_err(|_| LettaAdapterError::Journal("journal_lock_poisoned".into()))?
            .contains_key(event_id);
        Ok(Some(LettaRecoveryGuidance {
            schema: "focusa.letta_recovery_guidance.v1".into(),
            event_id: event_id.into(),
            request_id: intent.request_id,
            status: if settled { "settled" } else { "uncertain" }.into(),
            next_action: if settled {
                "replay_durable_receipt"
            } else {
                "retry_same_event_and_request_id"
            }
            .into(),
            automatic_retry_budget: 0,
        }))
    }

    fn append_sync(&self, receipt: &LettaTurnReceipt) -> Result<(), LettaAdapterError> {
        let intents = self
            .intents
            .lock()
            .map_err(|_| LettaAdapterError::Journal("journal_lock_poisoned".into()))?;
        if let Some(intent) = intents.get(&receipt.event_id)
            && (intent.request_id != receipt.request_id
                || intent.provider_agent_id != receipt.provider_agent_id
                || intent.epoch_id != receipt.epoch_id)
        {
            return Err(LettaAdapterError::Journal(
                "receipt_intent_identity_conflict".into(),
            ));
        }
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
        append_record(&self.path, receipt)?;
        state.insert(receipt.event_id.clone(), receipt.clone());
        Ok(())
    }
}

fn append_record<T: Serialize>(path: &Path, value: &T) -> Result<(), LettaAdapterError> {
    let bytes = serde_json::to_vec(value)
        .map_err(|_| LettaAdapterError::Journal("record_serialize_failed".into()))?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|_| LettaAdapterError::Journal("journal_append_open_failed".into()))?;
    file.write_all(&bytes)
        .and_then(|_| file.write_all(b"\n"))
        .and_then(|_| file.sync_data())
        .map_err(|_| LettaAdapterError::Journal("journal_durable_append_failed".into()))
}

fn read_records<T: DeserializeOwned>(
    path: &Path,
    key: impl Fn(&T) -> String,
) -> Result<BTreeMap<String, T>, LettaAdapterError> {
    let mut records = BTreeMap::new();
    if !path.exists() {
        return Ok(records);
    }
    let file =
        File::open(path).map_err(|_| LettaAdapterError::Journal("journal_open_failed".into()))?;
    for line in BufReader::new(file).lines() {
        let line = line.map_err(|_| LettaAdapterError::Journal("journal_read_failed".into()))?;
        if line.trim().is_empty() {
            continue;
        }
        let record: T = serde_json::from_str(&line)
            .map_err(|_| LettaAdapterError::Journal("journal_corrupt".into()))?;
        let record_key = key(&record);
        if records.insert(record_key, record).is_some() {
            return Err(LettaAdapterError::Journal(
                "duplicate_journal_record".into(),
            ));
        }
    }
    Ok(records)
}

impl LettaTurnJournal for FileTurnJournal {
    fn reserve<'a>(
        &'a self,
        request: &'a LettaTurnRequest,
    ) -> AdapterFuture<'a, Result<uuid::Uuid, LettaAdapterError>> {
        Box::pin(async move { self.reserve_sync(request) })
    }

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

    fn request(event_id: &str, request_id: Uuid) -> LettaTurnRequest {
        LettaTurnRequest {
            request_id,
            event_id: event_id.into(),
            provider_agent_id: "letta-agent".into(),
            epoch_id: Uuid::nil(),
            input: "bounded input".into(),
            input_digest: "sha256:input".into(),
            continuation: None,
        }
    }

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
    async fn uncertain_retry_reuses_durable_remote_idempotency_key() {
        let root = std::env::temp_dir().join(format!("focusa-letta-journal-{}", Uuid::now_v7()));
        let path = root.join("turns.jsonl");
        let first_id = Uuid::now_v7();
        let journal = FileTurnJournal::open(&path).unwrap();
        assert_eq!(
            journal
                .reserve(&request("event-1", first_id))
                .await
                .unwrap(),
            first_id
        );
        drop(journal);

        let replayed = FileTurnJournal::open(&path).unwrap();
        assert_eq!(
            replayed
                .reserve(&request("event-1", Uuid::now_v7()))
                .await
                .unwrap(),
            first_id
        );
        let uncertain = replayed.recovery_guidance("event-1").unwrap().unwrap();
        assert_eq!(uncertain.status, "uncertain");
        assert_eq!(uncertain.next_action, "retry_same_event_and_request_id");
        assert_eq!(uncertain.automatic_retry_budget, 0);
        let mut settled_receipt = receipt("event-1", "sha256:settled");
        settled_receipt.request_id = first_id;
        settled_receipt.epoch_id = Uuid::nil();
        replayed.append(&settled_receipt).await.unwrap();
        let settled = replayed.recovery_guidance("event-1").unwrap().unwrap();
        assert_eq!(settled.status, "settled");
        assert_eq!(settled.next_action, "replay_durable_receipt");
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
