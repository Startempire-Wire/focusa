//! Benchmark arms — comparison cells for the 4-arm design (Spec 113).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Arm {
    /// Baseline: no Focusa tools, no agent_prompt reminder.
    NoFocusa,
    /// Agent_prompt reminder only; no Focusa tools registered.
    PassiveFocusa,
    /// Focusa tools registered, but no Workpoint checkpointing/recovery.
    ToolOnlyFocusa,
    /// Full Focusa: tools + Workpoints + recovery + evidence + trajectory.
    FullFocusa,
}

impl Arm {
    pub fn as_str(self) -> &'static str {
        match self {
            Arm::NoFocusa => "no_focusa",
            Arm::PassiveFocusa => "passive_focusa",
            Arm::ToolOnlyFocusa => "tool_only_focusa",
            Arm::FullFocusa => "full_focusa",
        }
    }

    /// All four arms in canonical order.
    pub const ALL: [Arm; 4] = [
        Arm::NoFocusa,
        Arm::PassiveFocusa,
        Arm::ToolOnlyFocusa,
        Arm::FullFocusa,
    ];
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArmConfig {
    pub arm: Arm,
    /// Whether to register focusa_* tools in the agent context.
    pub focusa_tools_registered: bool,
    /// Whether the focusa_agent_prompt reminder is emitted on shell tools.
    pub emit_focusa_agent_prompt: bool,
    /// Whether the agent is required to use focusa_workpoint_checkpoint.
    pub focusa_workpoint_required: bool,
    /// Whether recovery / evidence / trajectory are enforced.
    pub evidence_chain_required: bool,
}

impl ArmConfig {
    pub fn for_arm(arm: Arm) -> Self {
        match arm {
            Arm::NoFocusa => ArmConfig {
                arm,
                focusa_tools_registered: false,
                emit_focusa_agent_prompt: false,
                focusa_workpoint_required: false,
                evidence_chain_required: false,
            },
            Arm::PassiveFocusa => ArmConfig {
                arm,
                focusa_tools_registered: false,
                emit_focusa_agent_prompt: true,
                focusa_workpoint_required: false,
                evidence_chain_required: false,
            },
            Arm::ToolOnlyFocusa => ArmConfig {
                arm,
                focusa_tools_registered: true,
                emit_focusa_agent_prompt: true,
                focusa_workpoint_required: false,
                evidence_chain_required: false,
            },
            Arm::FullFocusa => ArmConfig {
                arm,
                focusa_tools_registered: true,
                emit_focusa_agent_prompt: true,
                focusa_workpoint_required: true,
                evidence_chain_required: true,
            },
        }
    }
}
