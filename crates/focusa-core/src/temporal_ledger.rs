//! Durable append-only storage for Spec137 temporal events.

use chrono::{DateTime, Utc};
use std::{
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
};

use crate::temporal::{TemporalEvent, TemporalScope, seal_event, verify_event_chain};

#[derive(Debug)]
pub enum TemporalLedgerError {
    Io(String),
    CorruptLine(usize),
    InvalidChain,
    InvalidSignature(usize),
    ScopeMismatch,
    EmptyBatch,
}

pub struct TemporalLedger {
    path: PathBuf,
    scope: TemporalScope,
}

impl TemporalLedger {
    pub fn for_project(scope: TemporalScope) -> Result<Self, TemporalLedgerError> {
        if !Path::new(&scope.project_root).is_absolute()
            || matches!(
                scope.project_root.as_str(),
                "/" | "/root" | "/home" | "/tmp"
            )
        {
            return Err(TemporalLedgerError::ScopeMismatch);
        }
        Ok(Self {
            path: Path::new(&scope.project_root)
                .join(".focusa")
                .join("temporal")
                .join("events.jsonl"),
            scope,
        })
    }

    pub fn read_all(&self) -> Result<Vec<TemporalEvent>, TemporalLedgerError> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let file =
            File::open(&self.path).map_err(|error| TemporalLedgerError::Io(error.to_string()))?;
        let mut events = Vec::new();
        for (index, line) in BufReader::new(file).lines().enumerate() {
            let line = line.map_err(|error| TemporalLedgerError::Io(error.to_string()))?;
            let event = serde_json::from_str(&line)
                .map_err(|_| TemporalLedgerError::CorruptLine(index + 1))?;
            events.push(event);
        }
        if !verify_event_chain(&events) {
            return Err(TemporalLedgerError::InvalidChain);
        }
        for (index, event) in events.iter().enumerate() {
            if event.signature.is_some()
                && crate::temporal_integrity::verify_temporal_event_signature(event, None).is_err()
            {
                return Err(TemporalLedgerError::InvalidSignature(index + 1));
            }
        }
        Ok(events)
    }

    pub fn append_batch(
        &self,
        idempotency_key: &str,
        drafts: Vec<TemporalEvent>,
    ) -> Result<Vec<TemporalEvent>, TemporalLedgerError> {
        self.append_batch_with_signer(idempotency_key, drafts, None)
    }

    pub fn append_signed_batch(
        &self,
        idempotency_key: &str,
        drafts: Vec<TemporalEvent>,
        key_id: &str,
        signing_key: &ed25519_dalek::SigningKey,
    ) -> Result<Vec<TemporalEvent>, TemporalLedgerError> {
        self.append_batch_with_signer(idempotency_key, drafts, Some((key_id, signing_key)))
    }

    fn append_batch_with_signer(
        &self,
        idempotency_key: &str,
        drafts: Vec<TemporalEvent>,
        signer: Option<(&str, &ed25519_dalek::SigningKey)>,
    ) -> Result<Vec<TemporalEvent>, TemporalLedgerError> {
        if drafts.is_empty() {
            return Err(TemporalLedgerError::EmptyBatch);
        }
        let existing = self.read_all()?;
        let replay = existing
            .iter()
            .filter(|event| event.idempotency_key == idempotency_key)
            .cloned()
            .collect::<Vec<_>>();
        if !replay.is_empty() {
            return Ok(replay);
        }
        let mut predecessor = existing.last().map(|event| event.digest.clone());
        let first_sequence = existing.len() as u64 + 1;
        let mut sealed = Vec::with_capacity(drafts.len());
        for (sequence, mut event) in (first_sequence..).zip(drafts) {
            if !event.scope.same_workstream(&self.scope) {
                return Err(TemporalLedgerError::ScopeMismatch);
            }
            event.sequence = sequence;
            event.predecessor_digest = predecessor.clone();
            event.idempotency_key = idempotency_key.to_string();
            if let Some((key_id, signing_key)) = signer {
                crate::temporal_integrity::sign_temporal_event(&mut event, key_id, signing_key);
            } else {
                event = seal_event(event);
            }
            predecessor = Some(event.digest.clone());
            sealed.push(event);
        }
        let parent = self.path.parent().expect("temporal ledger parent");
        fs::create_dir_all(parent).map_err(|error| TemporalLedgerError::Io(error.to_string()))?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|error| TemporalLedgerError::Io(error.to_string()))?;
        for event in &sealed {
            serde_json::to_writer(&mut file, event)
                .map_err(|error| TemporalLedgerError::Io(error.to_string()))?;
            file.write_all(b"\n")
                .map_err(|error| TemporalLedgerError::Io(error.to_string()))?;
        }
        file.sync_data()
            .map_err(|error| TemporalLedgerError::Io(error.to_string()))?;
        File::open(parent)
            .and_then(|directory| directory.sync_all())
            .map_err(|error| TemporalLedgerError::Io(error.to_string()))?;
        Ok(sealed)
    }

    pub fn as_of(&self, at: DateTime<Utc>) -> Result<Vec<TemporalEvent>, TemporalLedgerError> {
        Ok(self
            .read_all()?
            .into_iter()
            .filter(|event| event.recorded_at <= at)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::temporal::{TemporalEvent, TemporalEventKind, TemporalScope, seal_event};
    use std::fs;

    fn test_scope() -> TemporalScope {
        let root = format!("/tmp/focusa-test-ledger-{}", uuid::Uuid::now_v7());
        fs::create_dir_all(&root).unwrap();
        TemporalScope {
            project_root: root,
            continuity_id: "test".into(),
            host_id: None, operator_id: None, workpoint_id: None, item_id: None, task_id: None,
        }
    }

    fn test_event(scope: &TemporalScope, kind: TemporalEventKind) -> TemporalEvent {
        TemporalEvent {
            event_id: uuid::Uuid::now_v7().to_string(),
            sequence: 0,
            event_kind: kind,
            scope: scope.clone(),
            claim: None, clock_sample: None,
            metadata: std::collections::BTreeMap::new(),
            signature: None, predecessor_digest: None,
            recorded_at: chrono::Utc::now(),
            idempotency_key: String::new(),
            digest: String::new(),
        }
    }

    #[test]
    fn ledger_rejects_root_path() {
        let scope = TemporalScope { project_root: "/tmp".into(), ..test_scope() };
        assert!(TemporalLedger::for_project(scope).is_err());
    }

    #[test]
    fn ledger_read_all_returns_empty_for_new_scope() {
        let scope = test_scope();
        let ledger = TemporalLedger::for_project(scope.clone()).unwrap();
        let events = ledger.read_all().unwrap();
        assert!(events.is_empty());
        fs::remove_dir_all(&scope.project_root).unwrap();
    }

    #[test]
    fn ledger_append_and_read_roundtrip() {
        let scope = test_scope();
        let project_root = scope.project_root.clone();
        let ledger = TemporalLedger::for_project(scope.clone()).unwrap();
        let events: Vec<TemporalEvent> = (0..3).map(|i| {
            let mut e = test_event(&scope, TemporalEventKind::ClaimCommitted);
            e.event_id = format!("ev-{}", i);
            e
        }).collect();
        let sealed = ledger.append_batch("key-1", events).unwrap();
        assert_eq!(sealed.len(), 3);
        assert!(sealed[0].sequence > 0);
        assert!(sealed[2].predecessor_digest.is_some());
        let read = ledger.read_all().unwrap();
        assert_eq!(read.len(), 3);
        fs::remove_dir_all(&project_root).unwrap();
    }

    #[test]
    fn ledger_rejects_empty_batch() {
        let scope = test_scope();
        let project_root = scope.project_root.clone();
        let ledger = TemporalLedger::for_project(scope).unwrap();
        assert!(ledger.append_batch("key-1", vec![]).is_err());
        fs::remove_dir_all(&project_root).unwrap();
    }

    #[test]
    fn ledger_as_of_filters_by_time() {
        let scope = test_scope();
        let project_root = scope.project_root.clone();
        let ledger = TemporalLedger::for_project(scope.clone()).unwrap();
        let past = chrono::Utc::now() - chrono::Duration::hours(1);
        let now = chrono::Utc::now();
        let mut e1 = test_event(&scope, TemporalEventKind::ClaimCommitted);
        e1.recorded_at = past;
        let mut e2 = test_event(&scope, TemporalEventKind::TargetSatisfied);
        e2.recorded_at = now;
        ledger.append_batch("key-2", vec![e1, e2]).unwrap();
        let as_of_past = ledger.as_of(past + chrono::Duration::seconds(1)).unwrap();
        assert_eq!(as_of_past.len(), 1);
        let as_of_now = ledger.as_of(now).unwrap();
        assert_eq!(as_of_now.len(), 2);
        fs::remove_dir_all(&project_root).unwrap();
    }

    #[test]
    fn ledger_idempotency_replays_existing_events() {
        let scope = test_scope();
        let project_root = scope.project_root.clone();
        let ledger = TemporalLedger::for_project(scope.clone()).unwrap();
        let events = vec![test_event(&scope, TemporalEventKind::ClaimCommitted)];
        let first = ledger.append_batch("key-3", events.clone()).unwrap();
        let replay = ledger.append_batch("key-3", events).unwrap();
        assert_eq!(first[0].digest, replay[0].digest);
        assert_eq!(first.len(), replay.len());
        fs::remove_dir_all(&project_root).unwrap();
    }

    #[test]
    fn ledger_rejects_cross_scope_events() {
        let scope = test_scope();
        let project_root = scope.project_root.clone();
        let other_root = format!("/tmp/focusa-test-ledger-other-{}", uuid::Uuid::now_v7());
        fs::create_dir_all(&other_root).unwrap();
        let other_scope = TemporalScope { project_root: other_root.clone(), ..scope.clone() };
        let ledger = TemporalLedger::for_project(scope.clone()).unwrap();
        let events = vec![test_event(&other_scope, TemporalEventKind::ClaimCommitted)];
        assert!(ledger.append_batch("key-4", events).is_err());
        fs::remove_dir_all(&project_root).unwrap();
        fs::remove_dir_all(&other_root).unwrap();
    }
}
