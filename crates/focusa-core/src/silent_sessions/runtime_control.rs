//! Portable runtime control requests with exact capability truth.

use serde::{Deserialize, Serialize};

use super::{RunGeneration, SilentSessionRunId};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PauseMode {
    Soft,
    Hard,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InputKind {
    Text,
    FollowUp,
    Steering,
    SpecialKey,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeControlCapabilities {
    pub soft_pause: bool,
    pub hard_pause: bool,
    pub text_input: bool,
    pub follow_up: bool,
    pub steering: bool,
    pub special_keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExactRuntimeTarget {
    pub run_id: SilentSessionRunId,
    pub generation: RunGeneration,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PauseRequest {
    pub target: ExactRuntimeTarget,
    pub mode: PauseMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeInputRequest {
    pub target: ExactRuntimeTarget,
    pub kind: InputKind,
    pub value: String,
}

impl RuntimeControlCapabilities {
    pub fn authorize_pause(&self, request: &PauseRequest) -> anyhow::Result<()> {
        let supported = match request.mode {
            PauseMode::Soft => self.soft_pause,
            PauseMode::Hard => self.hard_pause,
        };
        anyhow::ensure!(
            supported,
            "requested pause mode is unsupported; no control was applied"
        );
        Ok(())
    }

    pub fn authorize_input(&self, request: &RuntimeInputRequest) -> anyhow::Result<()> {
        anyhow::ensure!(!request.value.is_empty(), "runtime input cannot be empty");
        let supported = match request.kind {
            InputKind::Text => self.text_input,
            InputKind::FollowUp => self.follow_up,
            InputKind::Steering => self.steering,
            InputKind::SpecialKey => self.special_keys.iter().any(|key| key == &request.value),
        };
        anyhow::ensure!(
            supported,
            "requested input capability is unsupported; no input was delivered"
        );
        Ok(())
    }
}
