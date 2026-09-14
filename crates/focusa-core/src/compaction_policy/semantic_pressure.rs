use super::ContextManagementAction;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticPressureSignals {
    pub repeated_blocker_without_evidence: u32,
    pub repeated_failing_tool: u32,
    pub repeated_next_action_without_progress: u32,
    pub workpoint_revision_stagnation: u32,
    pub scope_correction: bool,
    pub tool_output_flood: bool,
    pub repeated_artifact_rehydration: u32,
    pub cross_project_contamination: bool,
    pub operator_correction: bool,
}

/// Choose the least expensive sufficient legal repair. This recommendation is
/// advisory and cannot expand the deterministic action mask.
pub fn recommend_semantic_repair(
    signals: &SemanticPressureSignals,
    legal_actions: &BTreeSet<ContextManagementAction>,
) -> ContextManagementAction {
    let preferred = if signals.cross_project_contamination || signals.scope_correction {
        vec![
            ContextManagementAction::CheckpointAndRollover,
            ContextManagementAction::CheckpointOnly,
            ContextManagementAction::NoAction,
        ]
    } else if signals.tool_output_flood || signals.repeated_artifact_rehydration >= 2 {
        vec![
            ContextManagementAction::ExternalizeToolArtifacts,
            ContextManagementAction::ProviderToolResultEdit,
            ContextManagementAction::PiStructuredCompaction,
            ContextManagementAction::NoAction,
        ]
    } else if signals.repeated_blocker_without_evidence >= 2
        || signals.repeated_failing_tool >= 2
        || signals.repeated_next_action_without_progress >= 2
        || signals.workpoint_revision_stagnation >= 2
        || signals.operator_correction
    {
        vec![
            ContextManagementAction::RebuildMinimalProjection,
            ContextManagementAction::CheckpointOnly,
            ContextManagementAction::PiStructuredCompaction,
            ContextManagementAction::NoAction,
        ]
    } else {
        vec![ContextManagementAction::NoAction]
    };
    preferred
        .iter()
        .find(|action| legal_actions.contains(action))
        .copied()
        .unwrap_or(ContextManagementAction::NoAction)
}
