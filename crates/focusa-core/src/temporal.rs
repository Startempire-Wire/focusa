//! Spec137 temporal authority primitives.
//! Clock fact, commitment, estimate, forecast, urgency, and presentation are distinct planes.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemporalClockDomain {
    WallUtc,
    MonotonicActive,
    SuspendAwareElapsed,
    CivilTimeIntent,
    TaiProjection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemporalClaimKind {
    ClockFact,
    ExternalCommitment,
    InternalReadinessTarget,
    Estimate,
    Forecast,
    UrgencySignal,
    PresentationHint,
    NoDeadline,
    ObservedDuration,
    MissedTarget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemporalClaimStatus {
    Proposed,
    Canonical,
    Superseded,
    Satisfied,
    Breached,
    Retracted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemporalConfidence {
    Unavailable,
    Low,
    Medium,
    High,
    Verified,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemporalUncertainty {
    pub earliest_at: Option<DateTime<Utc>>,
    pub latest_at: Option<DateTime<Utc>>,
    pub coverage_probability: Option<f64>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemporalScope {
    pub project_root: String,
    pub continuity_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemporalClockSample {
    pub sample_id: String,
    pub domain: TemporalClockDomain,
    pub wall_utc: DateTime<Utc>,
    pub monotonic_ns: Option<u128>,
    pub suspend_aware_ns: Option<u128>,
    pub boot_id: Option<String>,
    pub timezone: String,
    pub tzdb_version: Option<String>,
    pub source: String,
    pub observed_offset_ns: Option<i128>,
    pub measurement_uncertainty_ns: u128,
    pub confidence: TemporalConfidence,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemporalClaim {
    pub claim_id: String,
    pub revision: u64,
    pub scope: TemporalScope,
    pub kind: TemporalClaimKind,
    pub status: TemporalClaimStatus,
    pub subject_ref: String,
    pub target_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<u64>,
    pub timezone: String,
    pub source: String,
    pub source_ref: Option<String>,
    pub operator_confirmed: bool,
    pub confidence: TemporalConfidence,
    pub uncertainty: Option<TemporalUncertainty>,
    pub observed_at: DateTime<Utc>,
    pub effective_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub supersedes_revision: Option<u64>,
    pub evidence_refs: Vec<String>,
    pub reason_code: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemporalEventKind {
    ClockSampleObserved,
    ClaimProposed,
    ClaimCommitted,
    ClaimRevised,
    ClaimSuperseded,
    DurationObserved,
    TargetSatisfied,
    TargetBreached,
    MissedTargetRecorded,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemporalEvent {
    pub event_id: String,
    pub sequence: u64,
    pub event_kind: TemporalEventKind,
    pub scope: TemporalScope,
    pub claim: Option<TemporalClaim>,
    pub clock_sample: Option<TemporalClockSample>,
    pub predecessor_digest: Option<String>,
    pub recorded_at: DateTime<Utc>,
    pub idempotency_key: String,
    pub digest: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeadlineStatus {
    None,
    Committed,
    ForecastOnly,
    Conflicted,
    Satisfied,
    Breached,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemporalProjection {
    pub scope: TemporalScope,
    pub as_of: DateTime<Utc>,
    pub deadline_status: DeadlineStatus,
    pub active_commitment: Option<TemporalClaim>,
    pub active_forecast: Option<TemporalClaim>,
    pub observed_duration_count: usize,
    pub critical_path_ms: Option<u64>,
    pub slack_ms: Option<i64>,
    pub urgency: Option<TemporalClaim>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemporalValidationError {
    UnsafeProjectRoot,
    MissingContinuity,
    MissingTimezone,
    MissingSource,
    MissingSubject,
    CommitmentRequiresConfirmation,
    CommitmentRequiresTarget,
    EstimateCannotBecomeCommitment,
    InvalidUncertaintyRange,
    InvalidCoverageProbability,
    RevisionMustAdvance,
    SupersessionRequired,
}

pub fn validate_claim(
    claim: &TemporalClaim,
    previous: Option<&TemporalClaim>,
) -> Result<(), TemporalValidationError> {
    let root = claim.scope.project_root.trim();
    if !root.starts_with('/') || matches!(root, "/" | "/root" | "/home" | "/tmp") {
        return Err(TemporalValidationError::UnsafeProjectRoot);
    }
    if claim.scope.continuity_id.trim().is_empty() {
        return Err(TemporalValidationError::MissingContinuity);
    }
    if claim.timezone.trim().is_empty() {
        return Err(TemporalValidationError::MissingTimezone);
    }
    if claim.source.trim().is_empty() {
        return Err(TemporalValidationError::MissingSource);
    }
    if claim.subject_ref.trim().is_empty() {
        return Err(TemporalValidationError::MissingSubject);
    }
    if claim.kind == TemporalClaimKind::ExternalCommitment {
        if !claim.operator_confirmed {
            return Err(TemporalValidationError::CommitmentRequiresConfirmation);
        }
        if claim.target_at.is_none() {
            return Err(TemporalValidationError::CommitmentRequiresTarget);
        }
    }
    if matches!(
        claim.kind,
        TemporalClaimKind::Estimate | TemporalClaimKind::Forecast
    ) && claim.status == TemporalClaimStatus::Canonical
        && claim.operator_confirmed
    {
        return Err(TemporalValidationError::EstimateCannotBecomeCommitment);
    }
    if let Some(uncertainty) = &claim.uncertainty {
        if uncertainty
            .earliest_at
            .zip(uncertainty.latest_at)
            .is_some_and(|(earliest, latest)| earliest > latest)
        {
            return Err(TemporalValidationError::InvalidUncertaintyRange);
        }
        if uncertainty
            .coverage_probability
            .is_some_and(|probability| !(0.0..=1.0).contains(&probability))
        {
            return Err(TemporalValidationError::InvalidCoverageProbability);
        }
    }
    if let Some(previous) = previous {
        if claim.claim_id != previous.claim_id || claim.revision <= previous.revision {
            return Err(TemporalValidationError::RevisionMustAdvance);
        }
        if claim.supersedes_revision != Some(previous.revision) {
            return Err(TemporalValidationError::SupersessionRequired);
        }
    }
    Ok(())
}

pub fn temporal_event_digest(event: &TemporalEvent) -> String {
    let mut canonical = event.clone();
    canonical.digest.clear();
    let bytes = serde_json::to_vec(&canonical).expect("TemporalEvent serializes");
    format!("sha256:{}", hex::encode(Sha256::digest(bytes)))
}

pub fn seal_event(mut event: TemporalEvent) -> TemporalEvent {
    event.digest = temporal_event_digest(&event);
    event
}

pub fn verify_event_chain(events: &[TemporalEvent]) -> bool {
    events.iter().enumerate().all(|(index, event)| {
        event.sequence == index as u64 + 1
            && event.digest == temporal_event_digest(event)
            && event.predecessor_digest.as_deref()
                == index
                    .checked_sub(1)
                    .map(|previous| events[previous].digest.as_str())
    })
}

#[derive(Debug)]
pub enum TemporalLedgerError {
    Io(String),
    CorruptLine(usize),
    InvalidChain,
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
        Ok(events)
    }

    pub fn append_batch(
        &self,
        idempotency_key: &str,
        drafts: Vec<TemporalEvent>,
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
            if event.scope != self.scope {
                return Err(TemporalLedgerError::ScopeMismatch);
            }
            event.sequence = sequence;
            event.predecessor_digest = predecessor.clone();
            event.idempotency_key = idempotency_key.to_string();
            event = seal_event(event);
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

pub fn project_temporal(
    scope: TemporalScope,
    events: &[TemporalEvent],
    as_of: DateTime<Utc>,
) -> TemporalProjection {
    let active = events
        .iter()
        .filter(|event| event.scope == scope && event.recorded_at <= as_of)
        .filter_map(|event| event.claim.as_ref())
        .filter(|claim| claim.status == TemporalClaimStatus::Canonical)
        .collect::<Vec<_>>();
    let commitment = active
        .iter()
        .rev()
        .find(|claim| claim.kind == TemporalClaimKind::ExternalCommitment)
        .cloned()
        .cloned();
    let forecast = active
        .iter()
        .rev()
        .find(|claim| claim.kind == TemporalClaimKind::Forecast)
        .cloned()
        .cloned();
    let deadline_status = match commitment.as_ref() {
        Some(claim) if claim.status == TemporalClaimStatus::Breached => DeadlineStatus::Breached,
        Some(claim) if claim.status == TemporalClaimStatus::Satisfied => DeadlineStatus::Satisfied,
        Some(_) => DeadlineStatus::Committed,
        None if forecast.is_some() => DeadlineStatus::ForecastOnly,
        None => DeadlineStatus::None,
    };
    TemporalProjection {
        scope,
        as_of,
        deadline_status,
        active_commitment: commitment,
        active_forecast: forecast,
        observed_duration_count: events
            .iter()
            .filter(|event| event.event_kind == TemporalEventKind::DurationObserved)
            .count(),
        critical_path_ms: None,
        slack_ms: None,
        urgency: active
            .iter()
            .rev()
            .find(|claim| claim.kind == TemporalClaimKind::UrgencySignal)
            .cloned()
            .cloned(),
        warnings: Vec::new(),
    }
}

#[cfg(test)]
#[path = "temporal_tests.rs"]
mod tests;
