//! Independent retry budgets for daemon-native Silent Sessions.
//!
//! Retry classes never share counters. In particular, provider failures cannot
//! consume runner reconnect, harness restart, or work-item retry authority.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

pub const RETRY_BUDGET_SCHEMA: &str = "focusa.silent_session_retry_budget.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetryClass {
    Provider,
    TransportReconnect,
    HarnessRestart,
    ToolEnvironmentRecovery,
    ModelFallback,
    RunnerReconnect,
    WorkItem,
}

pub const ALL_RETRY_CLASSES: [RetryClass; 7] = [
    RetryClass::Provider,
    RetryClass::TransportReconnect,
    RetryClass::HarnessRestart,
    RetryClass::ToolEnvironmentRecovery,
    RetryClass::ModelFallback,
    RetryClass::RunnerReconnect,
    RetryClass::WorkItem,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetryBudgetPolicy {
    pub max_retries: u32,
    pub base_backoff_ms: u64,
    pub max_backoff_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetryClassState {
    pub failures: u32,
    pub next_retry_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "decision", rename_all = "snake_case")]
pub enum RetryDecision {
    Scheduled {
        class: RetryClass,
        retry_number: u32,
        remaining_retries: u32,
        backoff_ms: u64,
        retry_at: DateTime<Utc>,
    },
    Exhausted {
        class: RetryClass,
        failures: u32,
        max_retries: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetryBudgetTracker {
    pub schema: String,
    pub policies: BTreeMap<RetryClass, RetryBudgetPolicy>,
    pub states: BTreeMap<RetryClass, RetryClassState>,
}

impl RetryBudgetTracker {
    pub fn new(
        policies: BTreeMap<RetryClass, RetryBudgetPolicy>,
    ) -> Result<Self, RetryBudgetError> {
        validate_retry_budgets(&policies)?;
        Ok(Self {
            schema: RETRY_BUDGET_SCHEMA.into(),
            policies,
            states: ALL_RETRY_CLASSES
                .into_iter()
                .map(|class| {
                    (
                        class,
                        RetryClassState {
                            failures: 0,
                            next_retry_at: None,
                        },
                    )
                })
                .collect(),
        })
    }

    pub fn record_failure(
        &mut self,
        class: RetryClass,
        observed_at: DateTime<Utc>,
    ) -> Result<RetryDecision, RetryBudgetError> {
        let policy = *self
            .policies
            .get(&class)
            .ok_or(RetryBudgetError::MissingClass(class))?;
        let state = self
            .states
            .get_mut(&class)
            .ok_or(RetryBudgetError::MissingClass(class))?;
        state.failures = state
            .failures
            .checked_add(1)
            .ok_or(RetryBudgetError::CounterExhausted(class))?;
        if state.failures > policy.max_retries {
            state.next_retry_at = None;
            return Ok(RetryDecision::Exhausted {
                class,
                failures: state.failures,
                max_retries: policy.max_retries,
            });
        }

        let exponent = state.failures.saturating_sub(1).min(63);
        let backoff_ms = policy
            .base_backoff_ms
            .saturating_mul(1_u64 << exponent)
            .min(policy.max_backoff_ms);
        let backoff = Duration::milliseconds(
            i64::try_from(backoff_ms).map_err(|_| RetryBudgetError::BackoffOutOfRange(class))?,
        );
        let retry_at = observed_at
            .checked_add_signed(backoff)
            .ok_or(RetryBudgetError::BackoffOutOfRange(class))?;
        state.next_retry_at = Some(retry_at);
        Ok(RetryDecision::Scheduled {
            class,
            retry_number: state.failures,
            remaining_retries: policy.max_retries - state.failures,
            backoff_ms,
            retry_at,
        })
    }

    pub fn record_success(&mut self, class: RetryClass) -> Result<(), RetryBudgetError> {
        let state = self
            .states
            .get_mut(&class)
            .ok_or(RetryBudgetError::MissingClass(class))?;
        state.failures = 0;
        state.next_retry_at = None;
        Ok(())
    }

    pub fn state(&self, class: RetryClass) -> Option<&RetryClassState> {
        self.states.get(&class)
    }
}

pub fn default_retry_budgets() -> BTreeMap<RetryClass, RetryBudgetPolicy> {
    [
        (RetryClass::Provider, 5, 1_000, 30_000),
        (RetryClass::TransportReconnect, 8, 250, 10_000),
        (RetryClass::HarnessRestart, 3, 2_000, 30_000),
        (RetryClass::ToolEnvironmentRecovery, 3, 1_000, 15_000),
        (RetryClass::ModelFallback, 2, 500, 5_000),
        (RetryClass::RunnerReconnect, 8, 250, 10_000),
        (RetryClass::WorkItem, 3, 2_000, 30_000),
    ]
    .into_iter()
    .map(|(class, max_retries, base_backoff_ms, max_backoff_ms)| {
        (
            class,
            RetryBudgetPolicy {
                max_retries,
                base_backoff_ms,
                max_backoff_ms,
            },
        )
    })
    .collect()
}

pub fn validate_retry_budgets(
    policies: &BTreeMap<RetryClass, RetryBudgetPolicy>,
) -> Result<(), RetryBudgetError> {
    for class in ALL_RETRY_CLASSES {
        let policy = policies
            .get(&class)
            .ok_or(RetryBudgetError::MissingClass(class))?;
        if policy.max_retries == 0
            || policy.base_backoff_ms == 0
            || policy.max_backoff_ms < policy.base_backoff_ms
            || policy.max_backoff_ms > i64::MAX as u64
        {
            return Err(RetryBudgetError::InvalidPolicy(class));
        }
    }
    if policies.len() != ALL_RETRY_CLASSES.len() {
        return Err(RetryBudgetError::UnknownClassEntry);
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RetryBudgetError {
    #[error("retry budget is missing class {0:?}")]
    MissingClass(RetryClass),
    #[error("retry budget policy is invalid for class {0:?}")]
    InvalidPolicy(RetryClass),
    #[error("retry budget contains an unknown class entry")]
    UnknownClassEntry,
    #[error("retry counter exhausted for class {0:?}")]
    CounterExhausted(RetryClass),
    #[error("retry backoff is outside the supported time range for class {0:?}")]
    BackoffOutOfRange(RetryClass),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seven_retry_classes_are_independent_and_backoff_is_bounded() {
        let now = Utc::now();
        let mut tracker = RetryBudgetTracker::new(default_retry_budgets()).unwrap();
        for expected in [250, 500, 1_000, 2_000, 4_000, 8_000, 10_000, 10_000] {
            let RetryDecision::Scheduled { backoff_ms, .. } = tracker
                .record_failure(RetryClass::RunnerReconnect, now)
                .unwrap()
            else {
                panic!("runner reconnect should remain scheduled");
            };
            assert_eq!(backoff_ms, expected);
        }
        assert!(matches!(
            tracker
                .record_failure(RetryClass::RunnerReconnect, now)
                .unwrap(),
            RetryDecision::Exhausted { .. }
        ));
        assert_eq!(
            tracker.state(RetryClass::Provider).unwrap().failures,
            0,
            "runner reconnect must not consume provider retry authority"
        );
        assert!(matches!(
            tracker.record_failure(RetryClass::Provider, now).unwrap(),
            RetryDecision::Scheduled {
                retry_number: 1,
                ..
            }
        ));
        tracker.record_success(RetryClass::RunnerReconnect).unwrap();
        assert_eq!(
            tracker.state(RetryClass::RunnerReconnect).unwrap().failures,
            0
        );
        assert_eq!(tracker.state(RetryClass::Provider).unwrap().failures, 1);
    }

    #[test]
    fn incomplete_or_invalid_budget_sets_fail_closed() {
        let mut policies = default_retry_budgets();
        policies.remove(&RetryClass::WorkItem);
        assert_eq!(
            RetryBudgetTracker::new(policies),
            Err(RetryBudgetError::MissingClass(RetryClass::WorkItem))
        );

        let mut policies = default_retry_budgets();
        policies.get_mut(&RetryClass::Provider).unwrap().max_retries = 0;
        assert_eq!(
            RetryBudgetTracker::new(policies),
            Err(RetryBudgetError::InvalidPolicy(RetryClass::Provider))
        );
    }
}
