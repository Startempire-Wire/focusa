use focusa_core::silent_sessions::{
    ApprovalId, RunGeneration, SilentSessionAction, SilentSessionLifecycle, SilentSessionRunId,
};
use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use super::silent_sessions::{ApiResponse, failure};

pub(super) const MAX_TEXT_BYTES: usize = 65_536;
const MAX_KEYS: usize = 32;
const MAX_KEY_BYTES: usize = 64;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum DeliveryKind {
    Input,
    Steer,
    FollowUp,
    Keys,
}

impl DeliveryKind {
    pub(super) fn event_kind(self) -> &'static str {
        match self {
            Self::Input => "input.requested",
            Self::Steer => "steering.requested",
            Self::FollowUp => "follow_up.queued",
            Self::Keys => "key.requested",
        }
    }

    pub(super) fn side_effects(self, request_hash: &str) -> Vec<String> {
        let runner = format!("runner_{}_request:{request_hash}", self.as_str());
        if matches!(self, Self::Steer) {
            vec![format!("workpoint_steering_request:{request_hash}"), runner]
        } else {
            vec![runner]
        }
    }

    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Input => "input",
            Self::Steer => "steer",
            Self::FollowUp => "follow_up",
            Self::Keys => "keys",
        }
    }

    pub(super) fn accepts(self, lifecycle: SilentSessionLifecycle) -> bool {
        matches!(
            lifecycle,
            SilentSessionLifecycle::Running
                | SilentSessionLifecycle::WaitingInput
                | SilentSessionLifecycle::Blocked
        ) || matches!(self, Self::FollowUp) && lifecycle == SilentSessionLifecycle::Paused
    }
}

pub(super) fn delivery_request_hash_for_approval(
    run_id: SilentSessionRunId,
    generation: RunGeneration,
    approval_id: ApprovalId,
    kind: DeliveryKind,
    payload: &Value,
) -> String {
    let bytes = serde_json::to_vec(&json!({
        "run_id": run_id,
        "generation": generation,
        "approval_id": approval_id,
        "delivery_kind": kind,
        "content": payload,
    }))
    .expect("delivery request serializes");
    hex::encode(Sha256::digest(bytes))
}

pub(super) fn validate_approval_payload(
    kind: DeliveryKind,
    payload: &Value,
) -> Result<(), Box<ApiResponse>> {
    let object = payload
        .as_object()
        .ok_or_else(|| Box::new(validation_failure("approval payload must be an object")))?;
    let field = match kind {
        DeliveryKind::Input => "text",
        DeliveryKind::Steer => "instruction",
        DeliveryKind::FollowUp => "prompt",
        DeliveryKind::Keys => "keys",
    };
    if object.len() != 1 || !object.contains_key(field) {
        return Err(Box::new(validation_failure(&format!(
            "approval payload must contain only {field}"
        ))));
    }
    if kind == DeliveryKind::Keys {
        let keys = object[field]
            .as_array()
            .ok_or_else(|| Box::new(validation_failure("keys must be an array")))?;
        if keys.is_empty()
            || keys.len() > MAX_KEYS
            || keys.iter().any(|key| {
                key.as_str()
                    .is_none_or(|value| value.trim().is_empty() || value.len() > MAX_KEY_BYTES)
            })
        {
            return Err(Box::new(validation_failure(
                "keys must contain 1..32 non-empty names of at most 64 bytes each",
            )));
        }
        return Ok(());
    }
    let text = object[field]
        .as_str()
        .ok_or_else(|| Box::new(validation_failure(&format!("{field} must be a string"))))?;
    validate_text(text, field)
}

pub(super) fn validate_text(value: &str, field: &str) -> Result<(), Box<ApiResponse>> {
    if value.trim().is_empty() || value.len() > MAX_TEXT_BYTES {
        Err(Box::new(validation_failure(&format!(
            "{field} must contain 1..={MAX_TEXT_BYTES} bytes"
        ))))
    } else {
        Ok(())
    }
}

fn validation_failure(hint: &str) -> ApiResponse {
    failure(
        axum::http::StatusCode::BAD_REQUEST,
        "invalid_request",
        "validation_rejected",
        hint,
    )
}
