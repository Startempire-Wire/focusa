use focusa_core::compaction_policy::{
    CanaryEnrollmentReceipt, CapabilityEvidence, CompactionPolicyLease,
    CompactionPolicyObservation, CompactionRuntimeFingerprint, ContextManagementAction,
    ContextPolicyBundle, DriftVerdict, PolicyResolution, PromotionVerdict, RollbackReceipt,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    fs,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};
use uuid::Uuid;

static STORE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
const STORE_SCHEMA: &str = "focusa.compaction_policy_controller_store.v1";
const MAX_OBSERVATIONS: usize = 256;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ControllerRecord {
    pub scope_key: String,
    pub runtime_fingerprint: CompactionRuntimeFingerprint,
    pub legal_actions: BTreeSet<ContextManagementAction>,
    pub candidates: Vec<ContextPolicyBundle>,
    pub resolution: PolicyResolution,
    pub lease: CompactionPolicyLease,
    #[serde(default)]
    pub evidence: Vec<CapabilityEvidence>,
    #[serde(default)]
    pub observations: VecDeque<CompactionPolicyObservation>,
    pub canary_enrollment: Option<CanaryEnrollmentReceipt>,
    pub last_promotion: Option<PromotionVerdict>,
    pub last_drift: Option<DriftVerdict>,
    pub last_rollback: Option<RollbackReceipt>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct ControllerStore {
    schema: String,
    #[serde(default)]
    records: BTreeMap<String, ControllerRecord>,
}

fn path(data_dir: &str) -> PathBuf {
    Path::new(data_dir).join("compaction-policy-controller-v1.json")
}

fn load(path: &Path) -> ControllerStore {
    fs::read(path)
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_else(|| ControllerStore {
            schema: STORE_SCHEMA.into(),
            records: BTreeMap::new(),
        })
}

fn save(path: &Path, store: &ControllerStore) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let temporary = path.with_extension(format!("json.tmp-{}", Uuid::now_v7()));
    fs::write(
        &temporary,
        serde_json::to_vec_pretty(store).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    fs::rename(temporary, path).map_err(|error| error.to_string())
}

pub(crate) fn get(data_dir: &str, scope_key: &str) -> Option<ControllerRecord> {
    let _guard = STORE_LOCK.get_or_init(|| Mutex::new(())).lock().ok()?;
    load(&path(data_dir)).records.remove(scope_key)
}

pub(crate) fn replace(data_dir: &str, record: ControllerRecord) -> Result<(), String> {
    mutate(data_dir, &record.scope_key.clone(), move |slot| {
        *slot = Some(record);
        Ok(())
    })
}

pub(crate) fn mutate<T>(
    data_dir: &str,
    scope_key: &str,
    mutation: impl FnOnce(&mut Option<ControllerRecord>) -> Result<T, String>,
) -> Result<T, String> {
    let _guard = STORE_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .map_err(|_| "compaction_controller_store_lock_poisoned".to_string())?;
    let path = path(data_dir);
    let mut store = load(&path);
    let mut record = store.records.remove(scope_key);
    let result = mutation(&mut record)?;
    if let Some(mut record) = record {
        while record.observations.len() > MAX_OBSERVATIONS {
            record.observations.pop_front();
        }
        store.records.insert(scope_key.into(), record);
    }
    save(&path, &store)?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use focusa_core::compaction_policy::{
        CompactionRuntimeFacts, PolicyMode, PolicySelectionContext, compile_policy_lattice,
        resolve_policy, resolve_runtime_fingerprint,
    };

    fn record(scope_key: &str) -> ControllerRecord {
        let fingerprint = resolve_runtime_fingerprint(CompactionRuntimeFacts {
            provider_raw: None,
            api: None,
            model_id_raw: None,
            response_model: None,
            endpoint_class: None,
            api_version: None,
            beta_features: vec![],
            adapter_revision: "test".into(),
            capability_evidence_revision: "none".into(),
            context_window: Some(200_000),
            max_output_tokens: None,
            reasoning_enabled: None,
            transport: None,
            state_mode: None,
            cache_mode: None,
            harness_mode: Some("test".into()),
            objective_profile: Some("daily_driver".into()),
            session_id: "session".into(),
            attachment_id: "attachment".into(),
            project_root: Some("/tmp/project".into()),
            continuity_id: Some("continuity".into()),
        });
        let legal_actions = BTreeSet::new();
        let candidates = compile_policy_lattice(200_000, &legal_actions, "daily_driver", None);
        let resolution = resolve_policy(
            &PolicySelectionContext {
                mode: PolicyMode::Fixed,
                context_window: 200_000,
                sample_size: 0,
                measured_confidence: None,
                minimum_samples: 20,
                required_confidence: 0.95,
                dev_fleet_enrolled: false,
            },
            &candidates,
        );
        let lease = CompactionPolicyLease::freeze(
            &resolution,
            &fingerprint.segment_key,
            "none",
            "features",
        );
        ControllerRecord {
            scope_key: scope_key.into(),
            runtime_fingerprint: fingerprint,
            legal_actions,
            candidates,
            resolution,
            lease,
            evidence: vec![],
            observations: VecDeque::new(),
            canary_enrollment: None,
            last_promotion: None,
            last_drift: None,
            last_rollback: None,
        }
    }

    #[test]
    fn controller_store_is_durable_atomic_and_bounded() {
        let dir = std::env::temp_dir().join(format!("focusa-controller-store-{}", Uuid::now_v7()));
        fs::create_dir_all(&dir).unwrap();
        let data_dir = dir.to_string_lossy().to_string();
        replace(&data_dir, record("scope-a")).unwrap();
        assert_eq!(get(&data_dir, "scope-a").unwrap().scope_key, "scope-a");
        mutate(&data_dir, "scope-a", |slot| {
            let record = slot.as_mut().unwrap();
            for index in 0..300 {
                record.observations.push_back(CompactionPolicyObservation {
                    schema: "focusa.compaction_policy_observation.v1".into(),
                    runtime_segment: record.runtime_fingerprint.segment_key.clone(),
                    workstream_hash: "scope-a".into(),
                    epoch_id: format!("epoch-{index}"),
                    policy_id: "legacy_current_v1".into(),
                    trigger_class: "test".into(),
                    tokens_before: 100,
                    tokens_after: Some(50),
                    context_release_ratio: Some(0.5),
                    projection_tokens: 10,
                    prepare_latency_ms: None,
                    compaction_latency_ms: None,
                    verify_latency_ms: None,
                    first_productive_action_ms: None,
                    workpoint_revision_delta: 0,
                    repeat_error_delta: 0,
                    rehydrate_calls: 0,
                    rehydrated_bytes: 0,
                    hard_findings: vec![],
                    rollback_triggered: false,
                });
            }
            Ok(())
        })
        .unwrap();
        assert_eq!(get(&data_dir, "scope-a").unwrap().observations.len(), 256);
        assert!(path(&data_dir).exists());
        fs::remove_dir_all(dir).unwrap();
    }
}
