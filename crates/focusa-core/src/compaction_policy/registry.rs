use super::{ContextPolicyBundle, ValidationState};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, VecDeque};

const MAX_SEGMENTS: usize = 128;
const MAX_OBSERVATIONS_PER_SEGMENT: usize = 256;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompactionPolicyObservation {
    pub schema: String,
    pub runtime_segment: String,
    pub workstream_hash: String,
    pub epoch_id: String,
    pub policy_id: String,
    pub trigger_class: String,
    pub tokens_before: u64,
    pub tokens_after: Option<u64>,
    pub context_release_ratio: Option<f64>,
    pub projection_tokens: u64,
    pub prepare_latency_ms: Option<u64>,
    pub compaction_latency_ms: Option<u64>,
    pub verify_latency_ms: Option<u64>,
    pub first_productive_action_ms: Option<u64>,
    pub workpoint_revision_delta: i64,
    pub repeat_error_delta: i64,
    pub rehydrate_calls: u32,
    pub rehydrated_bytes: u64,
    #[serde(default)]
    pub hard_findings: Vec<String>,
    pub rollback_triggered: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SegmentProjection {
    pub segment_key: String,
    pub workstream_hash: String,
    pub observation_count: u64,
    pub hard_failure_count: u64,
    pub rollback_count: u64,
    pub last_epoch_id: Option<String>,
    pub quarantined_policy_ids: Vec<String>,
    pub policies: Vec<ContextPolicyBundle>,
    #[serde(default)]
    pub observations: VecDeque<CompactionPolicyObservation>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CompactionPolicyRegistry {
    pub schema: String,
    pub segments: BTreeMap<String, SegmentProjection>,
}

impl CompactionPolicyRegistry {
    pub fn new() -> Self {
        Self {
            schema: "focusa.compaction_policy_registry.v1".into(),
            segments: BTreeMap::new(),
        }
    }

    fn composite_key(segment_key: &str, workstream_hash: &str) -> String {
        format!("{segment_key}\0{workstream_hash}")
    }

    pub fn project(&self, segment_key: &str, workstream_hash: &str) -> Option<&SegmentProjection> {
        self.segments
            .get(&Self::composite_key(segment_key, workstream_hash))
    }

    pub fn replace_policies(
        &mut self,
        segment_key: &str,
        workstream_hash: &str,
        mut policies: Vec<ContextPolicyBundle>,
    ) {
        policies.sort_by(|left, right| left.policy_id.cmp(&right.policy_id));
        let projection = self.segment_mut(segment_key, workstream_hash);
        projection.policies = policies;
    }

    /// Idempotent ordered observation fold. Raw prompts, reasoning, and tool
    /// payloads have no fields in this schema and therefore cannot be stored.
    pub fn observe(&mut self, mut observation: CompactionPolicyObservation) {
        observation.hard_findings.sort();
        observation.hard_findings.dedup();
        observation.hard_findings.truncate(32);
        let projection =
            self.segment_mut(&observation.runtime_segment, &observation.workstream_hash);
        if projection
            .observations
            .iter()
            .any(|existing| existing.epoch_id == observation.epoch_id)
        {
            return;
        }
        projection.observation_count += 1;
        projection.last_epoch_id = Some(observation.epoch_id.clone());
        if !observation.hard_findings.is_empty() {
            projection.hard_failure_count += 1;
        }
        if observation.rollback_triggered || !observation.hard_findings.is_empty() {
            projection.rollback_count += u64::from(observation.rollback_triggered);
            if !projection
                .quarantined_policy_ids
                .contains(&observation.policy_id)
            {
                projection
                    .quarantined_policy_ids
                    .push(observation.policy_id.clone());
                projection.quarantined_policy_ids.sort();
            }
            if let Some(policy) = projection
                .policies
                .iter_mut()
                .find(|policy| policy.policy_id == observation.policy_id)
            {
                policy.validation = ValidationState::Quarantined;
            }
        }
        projection.observations.push_back(observation);
        while projection.observations.len() > MAX_OBSERVATIONS_PER_SEGMENT {
            projection.observations.pop_front();
        }
    }

    fn segment_mut(&mut self, segment_key: &str, workstream_hash: &str) -> &mut SegmentProjection {
        let key = Self::composite_key(segment_key, workstream_hash);
        if !self.segments.contains_key(&key) && self.segments.len() >= MAX_SEGMENTS {
            if let Some(oldest) = self.segments.keys().next().cloned() {
                self.segments.remove(&oldest);
            }
        }
        self.segments
            .entry(key)
            .or_insert_with(|| SegmentProjection {
                segment_key: segment_key.into(),
                workstream_hash: workstream_hash.into(),
                ..SegmentProjection::default()
            })
    }
}
