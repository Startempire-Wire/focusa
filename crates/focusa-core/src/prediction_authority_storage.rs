use crate::prediction_authority_ledger::{
    PredictionAuthorityLedger, PredictionAuthorityProjection,
};
use crate::{
    prediction_authority::{EpistemicScope, PredictionAuthorityEvent, ScopedAuthorityEvent},
    scoped_state::ScopeKind,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DurablePredictionEvent {
    #[serde(default = "legacy_prediction_event_schema_version")]
    pub schema_version: u32,
    pub event: ScopedAuthorityEvent,
    pub predecessor_digest: Option<String>,
    pub digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PredictionStorageError {
    UnsafeScope,
    ScopeMismatch,
    EmptyBatch,
    MissingEvidence,
    MissingReceipt,
    InvalidSequence,
    DuplicateEvent,
    InvalidChain,
    CorruptLine(usize),
    LegacyScopeMigrationRequired(usize),
    Io(String),
    Projection(String),
    InvalidPrimitive(String),
    HostDataDirRequired,
    ScopeKindMismatch,
}

pub struct PersistentPredictionAuthorityLedger {
    path: PathBuf,
    scope: EpistemicScope,
}

impl PersistentPredictionAuthorityLedger {
    pub fn for_scope(
        scope: EpistemicScope,
        host_data_dir: Option<&str>,
    ) -> Result<Self, PredictionStorageError> {
        scope
            .validate()
            .map_err(|_| PredictionStorageError::UnsafeScope)?;
        let path = match scope.root_scope.scope_kind {
            ScopeKind::Project => scope
                .root_scope
                .root_path
                .join(".focusa")
                .join("prediction-authority")
                .join("events.jsonl"),
            ScopeKind::Host => Path::new(
                host_data_dir
                    .filter(|path| Path::new(path).is_absolute())
                    .ok_or(PredictionStorageError::HostDataDirRequired)?,
            )
            .join("scoped")
            .join("prediction-authority")
            .join(scope.storage_key())
            .join("events.jsonl"),
        };
        Ok(Self { path, scope })
    }

    pub fn for_project(scope: EpistemicScope) -> Result<Self, PredictionStorageError> {
        if scope.root_scope.scope_kind != ScopeKind::Project {
            return Err(PredictionStorageError::ScopeKindMismatch);
        }
        Self::for_scope(scope, None)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn read_all(&self) -> Result<Vec<DurablePredictionEvent>, PredictionStorageError> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let file = File::open(&self.path).map_err(io_error)?;
        let mut rows = Vec::new();
        for (index, line) in BufReader::new(file).lines().enumerate() {
            let line = line.map_err(io_error)?;
            if line.trim().is_empty() {
                continue;
            }
            let value = serde_json::from_str::<serde_json::Value>(&line)
                .map_err(|_| PredictionStorageError::CorruptLine(index + 1))?;
            if value.pointer("/event/scope/project_root").is_some()
                && value.pointer("/event/scope/root_scope").is_none()
            {
                return Err(PredictionStorageError::LegacyScopeMigrationRequired(
                    index + 1,
                ));
            }
            rows.push(
                serde_json::from_value::<DurablePredictionEvent>(value)
                    .map_err(|_| PredictionStorageError::CorruptLine(index + 1))?,
            );
        }
        self.verify(&rows)?;
        Ok(rows)
    }

    pub fn append_batch(
        &self,
        events: Vec<ScopedAuthorityEvent>,
    ) -> Result<Vec<DurablePredictionEvent>, PredictionStorageError> {
        if events.is_empty() {
            return Err(PredictionStorageError::EmptyBatch);
        }
        let mut existing = self.read_all()?;
        let existing_ids = existing
            .iter()
            .map(|row| row.event.event_id.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        if events
            .iter()
            .any(|event| existing_ids.contains(event.event_id.as_str()))
        {
            return Err(PredictionStorageError::DuplicateEvent);
        }
        let mut predecessor = existing.last().map(|row| row.digest.clone());
        let next_sequence = existing.last().map_or(1, |row| row.event.sequence + 1);
        let mut appended = Vec::new();
        for (offset, event) in events.into_iter().enumerate() {
            self.validate_event(&event, next_sequence + offset as u64)?;
            let row = seal(event, predecessor.clone())?;
            predecessor = Some(row.digest.clone());
            appended.push(row);
        }
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(io_error)?;
        }
        existing.extend(appended.clone());
        self.verify(&existing)?;
        self.atomic_write(&existing)?;
        Ok(appended)
    }

    pub fn backup_to(&self, backup_path: &Path) -> Result<(), PredictionStorageError> {
        let rows = self.read_all()?;
        if let Some(parent) = backup_path.parent() {
            fs::create_dir_all(parent).map_err(io_error)?;
        }
        atomic_write_rows(backup_path, &rows)
    }

    pub fn restore_from_backup(&self, backup_path: &Path) -> Result<(), PredictionStorageError> {
        let backup = Self {
            path: backup_path.to_path_buf(),
            scope: self.scope.clone(),
        };
        let rows = backup.read_all()?;
        self.verify(&rows)?;
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(io_error)?;
        }
        self.atomic_write(&rows)
    }

    pub fn projection(&self) -> Result<PredictionAuthorityProjection, PredictionStorageError> {
        let mut ledger = PredictionAuthorityLedger::default();
        for row in self.read_all()? {
            ledger
                .append(row.event)
                .map_err(PredictionStorageError::Projection)?;
        }
        Ok(ledger.project(&self.scope))
    }

    fn atomic_write(&self, rows: &[DurablePredictionEvent]) -> Result<(), PredictionStorageError> {
        atomic_write_rows(&self.path, rows)
    }

    fn validate_event(
        &self,
        event: &ScopedAuthorityEvent,
        expected_sequence: u64,
    ) -> Result<(), PredictionStorageError> {
        if event.scope != self.scope {
            return Err(PredictionStorageError::ScopeMismatch);
        }
        if event.sequence != expected_sequence {
            return Err(PredictionStorageError::InvalidSequence);
        }
        if event.evidence_refs.is_empty() {
            return Err(PredictionStorageError::MissingEvidence);
        }
        if event.receipt_ref.trim().is_empty() {
            return Err(PredictionStorageError::MissingReceipt);
        }
        crate::prediction_authority_validation::validate_scoped_authority_event(event)
            .map_err(PredictionStorageError::InvalidPrimitive)?;
        match &event.event {
            PredictionAuthorityEvent::EpistemicPrimitive(record) => {
                crate::epistemic_primitives::validate_epistemic_primitive(record).map_err(
                    |error| PredictionStorageError::InvalidPrimitive(format!("{error:?}")),
                )?;
            }
            PredictionAuthorityEvent::ReflectionClaim(claim) => {
                crate::metacognitive_learning::validate_reflection_claim(claim).map_err(
                    |error| PredictionStorageError::InvalidPrimitive(format!("{error:?}")),
                )?;
            }
            PredictionAuthorityEvent::PromotionAssessment(assessment) => {
                if assessment.evidence_refs.is_empty() || assessment.receipt_ref.trim().is_empty() {
                    return Err(PredictionStorageError::InvalidPrimitive(
                        "promotion assessment proof required".into(),
                    ));
                }
            }
            PredictionAuthorityEvent::LearningSettlement(settlement) => {
                if settlement.evidence_refs.is_empty() || settlement.receipt_ref.trim().is_empty() {
                    return Err(PredictionStorageError::InvalidPrimitive(
                        "learning settlement proof required".into(),
                    ));
                }
            }
            PredictionAuthorityEvent::OutcomeAuthority(outcome) => {
                crate::outcome_resolution::validate_outcome_authority_event(outcome).map_err(
                    |error| PredictionStorageError::InvalidPrimitive(format!("{error:?}")),
                )?;
            }
            PredictionAuthorityEvent::FusionResult(result) => {
                if result.evidence_refs.is_empty() || result.receipt_ref.trim().is_empty() {
                    return Err(PredictionStorageError::InvalidPrimitive(
                        "fusion proof required".into(),
                    ));
                }
            }
            PredictionAuthorityEvent::ScenarioProjection(result) => {
                if result.evidence_refs.is_empty() || result.receipt_ref.trim().is_empty() {
                    return Err(PredictionStorageError::InvalidPrimitive(
                        "scenario proof required".into(),
                    ));
                }
            }
            PredictionAuthorityEvent::TransferEvaluation(result) => {
                if result.evidence_refs.is_empty() || result.receipt_ref.trim().is_empty() {
                    return Err(PredictionStorageError::InvalidPrimitive(
                        "transfer proof required".into(),
                    ));
                }
            }
            PredictionAuthorityEvent::SelfModelEstimate(estimate) => {
                crate::prediction_advanced::validate_self_model(estimate, event.recorded_at)
                    .map_err(|error| {
                        PredictionStorageError::InvalidPrimitive(format!("{error:?}"))
                    })?;
            }
            PredictionAuthorityEvent::MemoryLifecycle(lifecycle) => {
                if lifecycle.evidence_refs.is_empty() || lifecycle.receipt_ref.trim().is_empty() {
                    return Err(PredictionStorageError::InvalidPrimitive(
                        "memory lifecycle proof required".into(),
                    ));
                }
            }
            PredictionAuthorityEvent::SourceSecurityDecision(decision) => {
                if decision.evidence_refs.is_empty() || decision.receipt_ref.trim().is_empty() {
                    return Err(PredictionStorageError::InvalidPrimitive(
                        "source security proof required".into(),
                    ));
                }
            }
            PredictionAuthorityEvent::LegacyMigration(migration)
                if migration.evidence_refs.is_empty()
                    || migration.receipt_ref.trim().is_empty()
                    || migration.lineage_refs.is_empty()
                    || migration.rollback_ref.trim().is_empty() =>
            {
                return Err(PredictionStorageError::InvalidPrimitive(
                    "legacy migration lineage/proof required".into(),
                ));
            }
            _ => {}
        }
        Ok(())
    }

    fn verify(&self, rows: &[DurablePredictionEvent]) -> Result<(), PredictionStorageError> {
        let mut predecessor: Option<&str> = None;
        for (index, row) in rows.iter().enumerate() {
            self.validate_event(&row.event, index as u64 + 1)?;
            if row.predecessor_digest.as_deref() != predecessor || digest(row)? != row.digest {
                return Err(PredictionStorageError::InvalidChain);
            }
            predecessor = Some(&row.digest);
        }
        Ok(())
    }
}

fn seal(
    event: ScopedAuthorityEvent,
    predecessor_digest: Option<String>,
) -> Result<DurablePredictionEvent, PredictionStorageError> {
    let mut row = DurablePredictionEvent {
        schema_version: current_prediction_event_schema_version(),
        event,
        predecessor_digest,
        digest: String::new(),
    };
    row.digest = digest(&row)?;
    Ok(row)
}

fn digest(row: &DurablePredictionEvent) -> Result<String, PredictionStorageError> {
    let bytes = if row.schema_version == 0 {
        #[derive(Serialize)]
        struct LegacyEnvelope<'a> {
            event: &'a ScopedAuthorityEvent,
            predecessor_digest: &'a Option<String>,
            digest: &'static str,
        }
        serde_json::to_vec(&LegacyEnvelope {
            event: &row.event,
            predecessor_digest: &row.predecessor_digest,
            digest: "",
        })
    } else {
        let mut unsigned = row.clone();
        unsigned.digest.clear();
        serde_json::to_vec(&unsigned)
    }
    .map_err(|error| PredictionStorageError::Io(error.to_string()))?;
    Ok(hex::encode(Sha256::digest(bytes)))
}

fn legacy_prediction_event_schema_version() -> u32 {
    0
}

fn current_prediction_event_schema_version() -> u32 {
    1
}

fn atomic_write_rows(
    path: &Path,
    rows: &[DurablePredictionEvent],
) -> Result<(), PredictionStorageError> {
    let temporary = path.with_extension(format!("tmp-{}", std::process::id()));
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&temporary)
        .map_err(io_error)?;
    for row in rows {
        let encoded = serde_json::to_string(row)
            .map_err(|error| PredictionStorageError::Io(error.to_string()))?;
        file.write_all(encoded.as_bytes()).map_err(io_error)?;
        file.write_all(b"\n").map_err(io_error)?;
    }
    file.sync_all().map_err(io_error)?;
    crate::durable_fs::atomic_replace(&temporary, path).map_err(io_error)?;
    if let Some(parent) = path.parent() {
        crate::durable_fs::sync_directory(parent).map_err(io_error)?;
    }
    Ok(())
}

fn io_error(error: std::io::Error) -> PredictionStorageError {
    PredictionStorageError::Io(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prediction_authority::PredictionQuestion;
    use chrono::Utc;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_SCOPE: AtomicU64 = AtomicU64::new(1);

    fn scope() -> EpistemicScope {
        let root_path = std::env::temp_dir().join(format!(
            "focusa-prediction-ledger-{}-{}",
            std::process::id(),
            NEXT_SCOPE.fetch_add(1, Ordering::Relaxed)
        ));
        crate::scoped_state::WorkstreamKey::new(
            crate::scoped_state::ScopeRef::project(
                "project:prediction-ledger-test",
                &root_path,
                "prediction-ledger-test",
                format!("test:{}", root_path.display()),
            )
            .unwrap(),
            "spec138-test",
        )
        .unwrap()
    }

    fn event(scope: &EpistemicScope, sequence: u64) -> ScopedAuthorityEvent {
        ScopedAuthorityEvent {
            event_id: format!("event-{sequence}"),
            sequence,
            scope: scope.clone(),
            recorded_at: Utc::now(),
            event: PredictionAuthorityEvent::Question(PredictionQuestion {
                question_id: format!("question-{sequence}"),
                subject_ref: "release-success".into(),
                outcome_space: vec!["yes".into(), "no".into()],
                created_at: Utc::now(),
                horizon_claim_ref: "temporal:horizon".into(),
                evidence_refs: vec!["evidence:question".into()],
            }),
            evidence_refs: vec!["evidence:event".into()],
            receipt_ref: "receipt:event".into(),
        }
    }

    #[test]
    fn durable_ledger_restarts_projects_and_rejects_tamper() {
        let scope = scope();
        let _ = std::fs::remove_dir_all(&scope.root_scope.root_path);
        let ledger = PersistentPredictionAuthorityLedger::for_project(scope.clone()).unwrap();
        ledger
            .append_batch(vec![event(&scope, 1), event(&scope, 2)])
            .unwrap();
        let restarted = PersistentPredictionAuthorityLedger::for_project(scope.clone()).unwrap();
        assert_eq!(restarted.read_all().unwrap().len(), 2);
        let projection = restarted.projection().unwrap();
        assert_eq!(projection.sequence, 2);
        let backup = scope.root_scope.root_path.join("backup/events.jsonl");
        restarted.backup_to(&backup).unwrap();
        let mut body = std::fs::read_to_string(restarted.path()).unwrap();
        body = body.replacen("release-success", "tampered-subject", 1);
        std::fs::write(restarted.path(), body).unwrap();
        assert_eq!(
            restarted.read_all(),
            Err(PredictionStorageError::InvalidChain)
        );
        restarted.restore_from_backup(&backup).unwrap();
        assert_eq!(restarted.read_all().unwrap().len(), 2);
        let _ = std::fs::remove_dir_all(&scope.root_scope.root_path);
    }

    #[test]
    fn legacy_envelope_migrates_forward_without_rewriting_history() {
        let scope = scope();
        let _ = std::fs::remove_dir_all(&scope.root_scope.root_path);
        let ledger = PersistentPredictionAuthorityLedger::for_project(scope.clone()).unwrap();
        let mut legacy = DurablePredictionEvent {
            schema_version: 0,
            event: event(&scope, 1),
            predecessor_digest: None,
            digest: String::new(),
        };
        legacy.digest = digest(&legacy).unwrap();
        let mut value = serde_json::to_value(&legacy).unwrap();
        value.as_object_mut().unwrap().remove("schema_version");
        std::fs::create_dir_all(ledger.path().parent().unwrap()).unwrap();
        std::fs::write(
            ledger.path(),
            format!("{}\n", serde_json::to_string(&value).unwrap()),
        )
        .unwrap();
        assert_eq!(ledger.read_all().unwrap()[0].schema_version, 0);
        ledger.append_batch(vec![event(&scope, 2)]).unwrap();
        let rows = ledger.read_all().unwrap();
        assert_eq!(rows[0].schema_version, 0);
        assert_eq!(rows[1].schema_version, 1);
        let _ = std::fs::remove_dir_all(&scope.root_scope.root_path);
    }

    #[test]
    fn durable_ledger_requires_exact_scope_sequence_evidence_and_receipt() {
        let scope = scope();
        let _ = std::fs::remove_dir_all(&scope.root_scope.root_path);
        let ledger = PersistentPredictionAuthorityLedger::for_project(scope.clone()).unwrap();
        let mut invalid = event(&scope, 2);
        assert_eq!(
            ledger.append_batch(vec![invalid.clone()]),
            Err(PredictionStorageError::InvalidSequence)
        );
        invalid.sequence = 1;
        invalid.evidence_refs.clear();
        assert_eq!(
            ledger.append_batch(vec![invalid]),
            Err(PredictionStorageError::MissingEvidence)
        );
        let _ = std::fs::remove_dir_all(&scope.root_scope.root_path);
    }
}
