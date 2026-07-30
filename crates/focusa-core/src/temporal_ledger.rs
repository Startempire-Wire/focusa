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
