//! Deduplicated, channel-neutral Silent Session notification policy.

use crate::silent_session::{
    SilentSessionHealth, SilentSessionId, SilentSessionLifecycleState, SilentSessionRunId,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;
use uuid::Uuid;

pub const SILENT_SESSION_NOTIFICATION_SCHEMA: &str = "focusa.silent_session_notification.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationTrigger {
    WaitingForOperatorInput,
    BlockerRequiresJudgment,
    ModelMismatch,
    AuthOrEntitlementFailure,
    RepeatedProviderFailure,
    ResourcePressure,
    CheckpointFailure,
    ProcessFailure,
    OrphanedRun,
    CompletionMissingEvidence,
    VerifiedCompletion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationChannel {
    Menubar,
    Desktop,
    Webhook,
    Email,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotificationPolicy {
    pub channels: BTreeSet<NotificationChannel>,
    pub dedupe_cooldown_seconds: u64,
    pub repeated_provider_failure_threshold: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotificationObservation {
    pub session_id: SilentSessionId,
    pub run_id: SilentSessionRunId,
    pub generation: u64,
    pub lifecycle_state: SilentSessionLifecycleState,
    pub health: SilentSessionHealth,
    pub waiting_input_ref: Option<String>,
    pub waiting_input_prompt: Option<String>,
    pub blocker_ref: Option<String>,
    pub model_mismatch_ref: Option<String>,
    pub auth_failure_ref: Option<String>,
    pub provider_failure_count: u32,
    pub provider_failure_ref: Option<String>,
    pub resource_pressure_ref: Option<String>,
    pub checkpoint_failure_ref: Option<String>,
    pub process_failure_ref: Option<String>,
    pub completion_missing_evidence_ref: Option<String>,
    pub verified_completion_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotificationHistoryEntry {
    pub dedupe_key: String,
    pub last_delivered_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub existing_event_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotificationDeliveryRequest {
    pub schema: String,
    pub notification_id: Uuid,
    pub dedupe_key: String,
    pub trigger: NotificationTrigger,
    pub channel: NotificationChannel,
    pub session_id: SilentSessionId,
    pub run_id: SilentSessionRunId,
    pub generation: u64,
    pub title: String,
    pub why: String,
    pub exact_action: String,
    pub evidence_ref: String,
    pub created_at: DateTime<Utc>,
    pub persistent_dashboard_visible: bool,
    pub persist_delivery_via_existing_event_chain: bool,
}

pub fn evaluate_notifications(
    policy: &NotificationPolicy,
    observation: &NotificationObservation,
    history: &[NotificationHistoryEntry],
    now: DateTime<Utc>,
) -> Result<Vec<NotificationDeliveryRequest>, NotificationPolicyError> {
    if policy.channels.is_empty()
        || policy.dedupe_cooldown_seconds == 0
        || policy.repeated_provider_failure_threshold == 0
        || !observation.session_id.is_uuid_v7()
        || !observation.run_id.is_uuid_v7()
        || observation.generation == 0
    {
        return Err(NotificationPolicyError::InvalidPolicyOrScope);
    }
    let active = active_conditions(policy, observation)?;
    let history = history
        .iter()
        .map(|entry| (entry.dedupe_key.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    let cooldown = Duration::seconds(policy.dedupe_cooldown_seconds as i64);
    let mut deliveries = Vec::new();
    for condition in active {
        let dedupe_key = condition.dedupe_key(observation);
        if history.get(dedupe_key.as_str()).is_some_and(|entry| {
            entry.resolved_at.is_none() && now - entry.last_delivered_at < cooldown
        }) {
            continue;
        }
        for channel in &policy.channels {
            deliveries.push(NotificationDeliveryRequest {
                schema: SILENT_SESSION_NOTIFICATION_SCHEMA.into(),
                notification_id: Uuid::now_v7(),
                dedupe_key: dedupe_key.clone(),
                trigger: condition.trigger,
                channel: *channel,
                session_id: observation.session_id,
                run_id: observation.run_id,
                generation: observation.generation,
                title: condition.title.to_owned(),
                why: condition.why.clone(),
                exact_action: condition.exact_action.clone(),
                evidence_ref: condition.evidence_ref.clone(),
                created_at: now,
                persistent_dashboard_visible: true,
                persist_delivery_via_existing_event_chain: true,
            });
        }
    }
    Ok(deliveries)
}

struct ActiveCondition {
    trigger: NotificationTrigger,
    title: &'static str,
    why: String,
    exact_action: String,
    evidence_ref: String,
}

impl ActiveCondition {
    fn dedupe_key(&self, observation: &NotificationObservation) -> String {
        let material = format!(
            "{}:{}:{}:{:?}:{}",
            observation.session_id,
            observation.run_id,
            observation.generation,
            self.trigger,
            self.evidence_ref
        );
        format!(
            "sha256:{}",
            hex::encode(Sha256::digest(material.as_bytes()))
        )
    }
}

fn active_conditions(
    policy: &NotificationPolicy,
    observation: &NotificationObservation,
) -> Result<Vec<ActiveCondition>, NotificationPolicyError> {
    let mut conditions = Vec::new();
    if observation.lifecycle_state == SilentSessionLifecycleState::WaitingInput {
        let evidence = required(&observation.waiting_input_ref, "waiting_input_ref")?;
        let prompt = required(&observation.waiting_input_prompt, "waiting_input_prompt")?;
        conditions.push(ActiveCondition {
            trigger: NotificationTrigger::WaitingForOperatorInput,
            title: "Silent Session is waiting for input",
            why: prompt,
            exact_action: format!(
                "focusa silent send {} --run {} --text <response>",
                observation.session_id, observation.run_id
            ),
            evidence_ref: evidence,
        });
    }
    if observation.lifecycle_state == SilentSessionLifecycleState::Blocked {
        let evidence = required(&observation.blocker_ref, "blocker_ref")?;
        conditions.push(ActiveCondition {
            trigger: NotificationTrigger::BlockerRequiresJudgment,
            title: "Silent Session needs operator judgment",
            why: "A durable blocker prevents governed continuation.".into(),
            exact_action: format!(
                "focusa silent show {} --run {}",
                observation.session_id, observation.run_id
            ),
            evidence_ref: evidence,
        });
    }
    push_optional(
        &mut conditions,
        NotificationTrigger::ModelMismatch,
        "Silent Session model mismatch",
        "Requested, effective, and observed model bindings disagree.",
        "Inspect model binding and approve a restart or model switch.",
        &observation.model_mismatch_ref,
    );
    push_optional(
        &mut conditions,
        NotificationTrigger::AuthOrEntitlementFailure,
        "Silent Session authorization failure",
        "Provider authentication or entitlement blocked execution.",
        "Repair provider authorization, then restart the exact run.",
        &observation.auth_failure_ref,
    );
    if observation.provider_failure_count >= policy.repeated_provider_failure_threshold {
        let evidence = required(&observation.provider_failure_ref, "provider_failure_ref")?;
        conditions.push(ActiveCondition {
            trigger: NotificationTrigger::RepeatedProviderFailure,
            title: "Silent Session provider failures repeated",
            why: format!(
                "Provider failure count {} reached the configured threshold {}.",
                observation.provider_failure_count, policy.repeated_provider_failure_threshold
            ),
            exact_action: "Inspect retry budgets and approve recovery or model fallback.".into(),
            evidence_ref: evidence,
        });
    }
    push_optional(
        &mut conditions,
        NotificationTrigger::ResourcePressure,
        "Silent Session resource pressure",
        "Resource pressure crossed the governed admission threshold.",
        "Inspect usage and pause, stop, or raise the approved quota.",
        &observation.resource_pressure_ref,
    );
    push_optional(
        &mut conditions,
        NotificationTrigger::CheckpointFailure,
        "Silent Session checkpoint failed",
        "A required runtime or Workpoint checkpoint did not persist.",
        "Inspect checkpoint evidence before resuming mutation.",
        &observation.checkpoint_failure_ref,
    );
    if observation.lifecycle_state == SilentSessionLifecycleState::Failed
        || matches!(
            observation.health,
            SilentSessionHealth::ProcessExited | SilentSessionHealth::Unresponsive
        )
    {
        let evidence = required(&observation.process_failure_ref, "process_failure_ref")?;
        conditions.push(ActiveCondition {
            trigger: NotificationTrigger::ProcessFailure,
            title: "Silent Session process failed",
            why: "The supervised process failed or became unresponsive.".into(),
            exact_action: "Inspect failure evidence, then choose restart, recovery, or stop."
                .into(),
            evidence_ref: evidence,
        });
    }
    if observation.lifecycle_state == SilentSessionLifecycleState::Orphaned {
        let evidence = required(&observation.process_failure_ref, "orphan_evidence_ref")?;
        conditions.push(ActiveCondition {
            trigger: NotificationTrigger::OrphanedRun,
            title: "Silent Session run is orphaned",
            why: "The daemon cannot currently prove ownership of the surviving run.".into(),
            exact_action: format!(
                "focusa silent adopt {} --run {} <authority options>",
                observation.session_id, observation.run_id
            ),
            evidence_ref: evidence,
        });
    }
    push_optional(
        &mut conditions,
        NotificationTrigger::CompletionMissingEvidence,
        "Silent Session completion is blocked",
        "Required completion evidence is missing.",
        "Open completion evidence and satisfy the listed missing classes.",
        &observation.completion_missing_evidence_ref,
    );
    if observation.lifecycle_state == SilentSessionLifecycleState::Completed {
        let evidence = required(
            &observation.verified_completion_ref,
            "verified_completion_ref",
        )?;
        conditions.push(ActiveCondition {
            trigger: NotificationTrigger::VerifiedCompletion,
            title: "Silent Session completion verified",
            why: "Completion evidence, acceptance, and receipt commit are verified.".into(),
            exact_action: "Open the final report or governed integration receipt.".into(),
            evidence_ref: evidence,
        });
    }
    Ok(conditions)
}

fn push_optional(
    conditions: &mut Vec<ActiveCondition>,
    trigger: NotificationTrigger,
    title: &'static str,
    why: &'static str,
    action: &'static str,
    evidence: &Option<String>,
) {
    if let Some(evidence_ref) = evidence.as_deref().filter(|value| !value.trim().is_empty()) {
        conditions.push(ActiveCondition {
            trigger,
            title,
            why: why.into(),
            exact_action: action.into(),
            evidence_ref: evidence_ref.into(),
        });
    }
}

fn required(
    value: &Option<String>,
    field: &'static str,
) -> Result<String, NotificationPolicyError> {
    value
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(ToOwned::to_owned)
        .ok_or(NotificationPolicyError::MissingTriggerEvidence(field))
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum NotificationPolicyError {
    #[error("notification policy, channels, or session/run scope is invalid")]
    InvalidPolicyOrScope,
    #[error("active notification trigger is missing exact evidence: {0}")]
    MissingTriggerEvidence(&'static str),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn observation() -> NotificationObservation {
        NotificationObservation {
            session_id: SilentSessionId::new(),
            run_id: SilentSessionRunId::new(),
            generation: 2,
            lifecycle_state: SilentSessionLifecycleState::WaitingInput,
            health: SilentSessionHealth::ProcessExited,
            waiting_input_ref: Some("input:1".into()),
            waiting_input_prompt: Some("Choose integration strategy".into()),
            blocker_ref: Some("blocker:1".into()),
            model_mismatch_ref: Some("model-mismatch:1".into()),
            auth_failure_ref: Some("auth-failure:1".into()),
            provider_failure_count: 3,
            provider_failure_ref: Some("provider-failure:1".into()),
            resource_pressure_ref: Some("resource-pressure:1".into()),
            checkpoint_failure_ref: Some("checkpoint-failure:1".into()),
            process_failure_ref: Some("process-failure:1".into()),
            completion_missing_evidence_ref: Some("completion-missing:1".into()),
            verified_completion_ref: None,
        }
    }

    fn policy() -> NotificationPolicy {
        NotificationPolicy {
            channels: BTreeSet::from([NotificationChannel::Menubar, NotificationChannel::Desktop]),
            dedupe_cooldown_seconds: 300,
            repeated_provider_failure_threshold: 3,
        }
    }

    #[test]
    fn all_nonexclusive_triggers_emit_across_channels_with_exact_why_and_action() {
        let deliveries =
            evaluate_notifications(&policy(), &observation(), &[], Utc::now()).unwrap();
        let triggers = deliveries
            .iter()
            .map(|delivery| delivery.trigger)
            .collect::<BTreeSet<_>>();
        assert!(triggers.contains(&NotificationTrigger::WaitingForOperatorInput));
        assert!(triggers.contains(&NotificationTrigger::ModelMismatch));
        assert!(triggers.contains(&NotificationTrigger::AuthOrEntitlementFailure));
        assert!(triggers.contains(&NotificationTrigger::RepeatedProviderFailure));
        assert!(triggers.contains(&NotificationTrigger::ResourcePressure));
        assert!(triggers.contains(&NotificationTrigger::CheckpointFailure));
        assert!(triggers.contains(&NotificationTrigger::ProcessFailure));
        assert_eq!(deliveries.len(), triggers.len() * 2);
        assert!(deliveries.iter().all(|delivery| {
            !delivery.why.is_empty()
                && !delivery.exact_action.is_empty()
                && delivery.persistent_dashboard_visible
                && delivery.persist_delivery_via_existing_event_chain
        }));
    }

    #[test]
    fn dedupe_suppresses_same_condition_until_cooldown_or_resolution() {
        let now = Utc::now();
        let observation = observation();
        let first = evaluate_notifications(&policy(), &observation, &[], now).unwrap();
        let history = first
            .iter()
            .map(|delivery| NotificationHistoryEntry {
                dedupe_key: delivery.dedupe_key.clone(),
                last_delivered_at: now,
                resolved_at: None,
                existing_event_ref: "event:notification".into(),
            })
            .collect::<Vec<_>>();
        assert!(
            evaluate_notifications(
                &policy(),
                &observation,
                &history,
                now + Duration::seconds(1)
            )
            .unwrap()
            .is_empty()
        );
        assert!(
            !evaluate_notifications(
                &policy(),
                &observation,
                &history,
                now + Duration::seconds(301),
            )
            .unwrap()
            .is_empty()
        );
    }

    #[test]
    fn blocker_orphan_completion_and_waiting_input_actions_are_explicit() {
        for (state, expected) in [
            (
                SilentSessionLifecycleState::Blocked,
                NotificationTrigger::BlockerRequiresJudgment,
            ),
            (
                SilentSessionLifecycleState::Orphaned,
                NotificationTrigger::OrphanedRun,
            ),
            (
                SilentSessionLifecycleState::Completed,
                NotificationTrigger::VerifiedCompletion,
            ),
        ] {
            let mut observation = observation();
            observation.lifecycle_state = state;
            observation.health = SilentSessionHealth::Healthy;
            observation.verified_completion_ref = Some("completion:verified".into());
            let deliveries =
                evaluate_notifications(&policy(), &observation, &[], Utc::now()).unwrap();
            assert!(
                deliveries
                    .iter()
                    .any(|delivery| delivery.trigger == expected)
            );
        }
    }
}
