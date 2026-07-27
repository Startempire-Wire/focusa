//! Thin CLI facade for the provider-neutral Master Release Cycle.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
};

use focusa_core::{
    release_adapters::{
        JsonProcessReleaseExecutor, ManifestReleaseAdapter, ReleaseAdapterManifest,
    },
    release_calibration::{
        ReleaseCalibrationLedger, ReleaseCalibrationObservation, ReleaseCalibrationPolicy,
        ReleaseCalibrator, ReleasePlanTuning,
    },
    release_cycle::{ReleaseCandidate, ReleaseTopology},
    release_ledger::JsonlReleaseRunLedger,
    release_orchestrator::{
        MasterReleaseOrchestrator, ReleaseAuthority, ReleaseRunInput, ReleaseRunMode,
    },
};
use serde_json::json;

use super::ReleaseSurfaceArg;

pub fn validate_adapter(manifest: PathBuf, topology: PathBuf) -> anyhow::Result<()> {
    let topology = load_topology(&topology)?;
    let manifest = load_manifest(&manifest)?;
    manifest.validate(&topology)?;
    let output = json!({
        "schema": "focusa.release_adapter_validation.v1",
        "status": "completed",
        "valid": true,
        "manifest_id": manifest.manifest_id,
        "adapter_id": manifest.descriptor.adapter_id,
        "project_id": topology.project_id,
        "profile": topology.profile,
        "operation_count": manifest.operations.len(),
    });
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

pub fn plan(
    manifest: PathBuf,
    topology: PathBuf,
    candidate: PathBuf,
    tuning: Option<PathBuf>,
    surface: ReleaseSurfaceArg,
) -> anyhow::Result<()> {
    let topology = load_topology(&topology)?;
    let manifest = load_manifest(&manifest)?;
    manifest.validate(&topology)?;
    let candidate: ReleaseCandidate = load_json(&candidate)?;
    let tuning = load_tuning(tuning)?;
    let plan = MasterReleaseOrchestrator::plan_for_surface(
        &candidate,
        &topology,
        &manifest.descriptor,
        &BTreeMap::new(),
        &tuning,
        surface.into(),
    )?;
    println!("{}", serde_json::to_string_pretty(&plan)?);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn execute(
    manifest: PathBuf,
    topology: PathBuf,
    candidate_path: PathBuf,
    tuning: Option<PathBuf>,
    plugin: PathBuf,
    ledger: PathBuf,
    surface: ReleaseSurfaceArg,
    yes: bool,
    allow_mutations: bool,
    approval_refs: Vec<String>,
) -> anyhow::Result<()> {
    anyhow::ensure!(yes, "release execution requires --yes");
    let topology = load_topology(&topology)?;
    let manifest = load_manifest(&manifest)?;
    manifest.validate(&topology)?;
    let mut candidate: ReleaseCandidate = load_json(&candidate_path)?;
    let tuning = load_tuning(tuning)?;
    let checkpoint_ledger = JsonlReleaseRunLedger::new(
        &ledger,
        candidate.project_root.clone(),
        candidate.candidate_id.clone(),
        candidate.exact_sha.clone(),
    )?;
    let mut resume_receipts = Vec::new();
    if let Some(latest) = checkpoint_ledger.latest()? {
        candidate = latest.candidate;
        resume_receipts = latest.receipts;
    }
    let executor_ids: BTreeSet<_> = manifest
        .operations
        .iter()
        .map(|operation| operation.executor_id.clone())
        .collect();
    let mut executors = Vec::new();
    for executor_id in executor_ids {
        executors.push(JsonProcessReleaseExecutor::new(
            executor_id,
            plugin.clone(),
            candidate.project_root.clone(),
        )?);
    }
    let adapter = ManifestReleaseAdapter::new(manifest, topology.clone(), executors)?;
    let authority = ReleaseAuthority {
        project_root: candidate.project_root.clone(),
        continuity_id: candidate.continuity_id.clone(),
        operator_confirmed: yes,
        mutation_allowed: allow_mutations,
        approval_refs,
    };
    let result = MasterReleaseOrchestrator::run_with_checkpoint_sink(
        &adapter,
        ReleaseRunInput {
            candidate,
            topology,
            authority,
            mode: ReleaseRunMode::Execute,
            observed_at: chrono::Utc::now().to_rfc3339(),
            reusable_evidence: BTreeMap::new(),
            tuning,
            invocation_surface: surface.into(),
            resume_receipts,
        },
        &checkpoint_ledger,
    )
    .await?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

pub fn calibrate(
    ledger: PathBuf,
    observation: PathBuf,
    active_tuning: Option<PathBuf>,
    output: Option<PathBuf>,
) -> anyhow::Result<()> {
    let observation: ReleaseCalibrationObservation = load_json(&observation)?;
    ReleaseCalibrationLedger::append(&ledger, &observation)?;
    let history =
        ReleaseCalibrationLedger::read(&ledger, &observation.project_id, &observation.profile)?;
    let active = load_tuning(active_tuning)?;
    let decision =
        ReleaseCalibrator::decide(&history, &active, &ReleaseCalibrationPolicy::default())?;
    let body = serde_json::to_string_pretty(&decision)?;
    if let Some(path) = output {
        fs::write(path, &body)?;
    }
    println!("{body}");
    Ok(())
}

fn load_topology(path: &PathBuf) -> anyhow::Result<ReleaseTopology> {
    load_json(path)
}
fn load_manifest(path: &PathBuf) -> anyhow::Result<ReleaseAdapterManifest> {
    load_json(path)
}

fn load_tuning(path: Option<PathBuf>) -> anyhow::Result<ReleasePlanTuning> {
    path.as_ref()
        .map_or_else(|| Ok(ReleasePlanTuning::default()), load_json)
}

fn load_json<T: serde::de::DeserializeOwned>(path: &PathBuf) -> anyhow::Result<T> {
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}
