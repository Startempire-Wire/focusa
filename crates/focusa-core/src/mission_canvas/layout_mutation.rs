use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::workstream_context::{WorkstreamContext, WorkstreamContextError};
use crate::workstream_identity::{WorkSurfaceId, WorkstreamKey};

use super::{
    model::{
        MissionCanvasScope, OmissionDiagnostic, ResolvedContribution, ResolvedWorkspaceProjection,
    },
    persistence::{MissionCanvasStore, MissionCanvasStoreError},
    reducer::projection_digest,
};

pub const LAYOUT_MUTATE_OPERATION: &str = "focusa.mission_canvas.layout.mutate";
pub const LAYOUT_MUTATE_PERMISSION: &str = "mission_canvas:write";

const ACTIONS: &[&str] = &[
    "open",
    "focus",
    "pin",
    "unpin",
    "group",
    "ungroup",
    "reorder",
    "split_horizontal",
    "split_vertical",
    "resize_split",
    "compare",
    "suspend_projection",
    "rehydrate",
    "close_projection",
    "set_active_tab",
];

/// Generated `LayoutMutationCommand` as consumed by the Core operation.
///
/// The command is deliberately Workstream-bound.  Attachment, Work Surface,
/// and runtime values are subordinate identity, not alternate lookup keys.
/// `LayoutMutationService` validates the complete chain before it examines a
/// layout or contribution.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayoutMutationCommand {
    #[serde(flatten)]
    pub scope: MissionCanvasScope,
    pub command_id: String,
    pub action: String,
    #[serde(default)]
    pub secondary_work_surface_id: Option<WorkSurfaceId>,
    #[serde(default)]
    pub target_contribution_id: Option<String>,
    #[serde(default)]
    pub target_layout_node_id: Option<String>,
    #[serde(default)]
    pub split_ratio: Option<f64>,
    #[serde(default)]
    pub target_index: Option<u32>,
    pub expected_projection_revision: u64,
    pub expected_layout_revision: u64,
    pub idempotency_key: String,
    #[serde(default)]
    pub requested_at: Option<String>,
    #[serde(default)]
    pub target_work_surface_id: Option<WorkSurfaceId>,
}

/// The direct generated `LayoutMutationResult`; no layout/result wrapper is
/// allowed to cross the HTTP boundary. The canonical layout is persisted in
/// the Core projection and is read back through `projection.get` after this
/// result is accepted.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LayoutMutationResult {
    #[serde(flatten)]
    pub scope: MissionCanvasScope,
    pub command_id: String,
    pub accepted: bool,
    pub projection_revision: u64,
    pub layout_revision: u64,
    pub projection_digest: String,
    pub event_cursor: String,
    #[serde(default)]
    pub error_ref: Option<String>,
    #[serde(default)]
    pub evidence_ref: Option<String>,
    #[serde(default)]
    pub receipt_ref: Option<String>,
}

/// API-to-Core execution envelope. The generated command remains the only
/// operation DTO; authenticated context and permission projection are supplied
/// by the adapter and are never inferred from a contribution or layout node.
#[derive(Clone, Debug)]
pub struct LayoutMutationExecution {
    pub context: WorkstreamContext,
    pub command: LayoutMutationCommand,
    pub permissions: BTreeSet<String>,
}

#[derive(Debug, Error)]
pub enum LayoutMutationError {
    #[error("layout mutation Workstream context is invalid: {0}")]
    Context(#[from] WorkstreamContextError),
    #[error("layout mutation scope is invalid: {0}")]
    Scope(&'static str),
    #[error("layout mutation requires permission: {0}")]
    PermissionDenied(String),
    #[error("layout mutation command is invalid: {0}")]
    CommandInvalid(&'static str),
    #[error("layout mutation projection was not found for the exact Workstream")]
    ProjectionNotFound,
    #[error("layout mutation projection or layout revision is stale")]
    RevisionConflict,
    #[error("layout mutation idempotency key conflicts with another command")]
    IdempotencyConflict,
    #[error("layout mutation references unknown contribution {0}")]
    UnknownContribution(String),
    #[error("layout mutation references unknown Work Surface {0}")]
    UnknownWorkSurface(String),
    #[error("layout mutation cannot be applied to the current canonical layout")]
    NotApplicable,
    #[error("layout mutation action is not represented by the canonical projection: {0}")]
    UnsupportedAction(String),
    #[error("canonical layout is invalid: {0}")]
    InvalidLayout(String),
    #[error("layout mutation revision overflow")]
    RevisionOverflow,
    #[error("layout mutation serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error(transparent)]
    Store(#[from] MissionCanvasStoreError),
}

#[derive(Clone, Copy, Debug, Default)]
pub struct LayoutMutationService;

impl LayoutMutationService {
    /// Validate, apply, and atomically persist one layout mutation. Core owns
    /// layout resolution, contribution identity, revisioning, evidence,
    /// receipt, and durable event state; the API route only adapts transport.
    pub fn mutate(
        &self,
        store: &MissionCanvasStore,
        execution: LayoutMutationExecution,
    ) -> Result<LayoutMutationResult, LayoutMutationError> {
        validate_layout_mutation_execution(&execution)?;
        let request_digest = layout_mutation_digest(&execution)?;

        // A retry must be replayable even after the current projection has
        // advanced. This is a read-only fast path; the persistence transaction
        // repeats the same check to close the concurrent-writer race.
        if let Some(result) = store.find_layout_mutation_replay(
            &execution.command.scope,
            &execution.command.idempotency_key,
            &request_digest,
        )? {
            return Ok(result);
        }

        let mut projection = store
            .get_projection(&execution.command.scope)?
            .ok_or(LayoutMutationError::ProjectionNotFound)?;
        projection
            .validate_scope(&execution.command.scope)
            .map_err(LayoutMutationError::Scope)?;
        if projection.projection_revision != execution.command.expected_projection_revision
            || projection.layout_revision != execution.command.expected_layout_revision
        {
            return Err(LayoutMutationError::RevisionConflict);
        }

        let eligible = eligible_contributions(&projection)?;
        validate_layout_tree(&projection.layout_tree, &eligible.keys().cloned().collect())?;
        let target = resolve_target_contribution(
            &projection,
            execution.command.target_contribution_id.as_deref(),
            execution.command.target_work_surface_id.as_ref(),
        )?;
        let secondary = execution
            .command
            .secondary_work_surface_id
            .as_ref()
            .map(|surface| resolve_work_surface_contribution(&projection, surface))
            .transpose()?
            .flatten();

        if target.as_deref() == secondary.as_deref() && target.is_some() {
            return Err(LayoutMutationError::CommandInvalid(
                "target_and_secondary_must_differ",
            ));
        }
        validate_action_targets(&execution.command, target.as_deref(), secondary.as_deref())?;

        let previous_layout = projection.layout_tree.clone();
        let mut next_layout = previous_layout.clone();
        let changed = match execution.command.action.as_str() {
            "focus" => {
                let contribution_id =
                    target
                        .as_deref()
                        .ok_or(LayoutMutationError::CommandInvalid(
                            "target_contribution_required",
                        ))?;
                let contribution = eligible.get(contribution_id).ok_or_else(|| {
                    LayoutMutationError::UnknownContribution(contribution_id.to_owned())
                })?;
                if projection.focused_semantic_target == contribution.semantic_binding_id {
                    false
                } else {
                    projection.focused_semantic_target = contribution.semantic_binding_id.clone();
                    true
                }
            }
            "set_active_tab" => set_active_tab(
                &mut next_layout,
                target
                    .as_deref()
                    .ok_or(LayoutMutationError::CommandInvalid(
                        "target_contribution_required",
                    ))?,
            ),
            "resize_split" => resize_split(
                &mut next_layout,
                execution.command.target_layout_node_id.as_deref(),
                target.as_deref(),
                execution.command.split_ratio.unwrap_or(0.67),
            ),
            "reorder" => reorder_layout(
                &mut next_layout,
                target
                    .as_deref()
                    .ok_or(LayoutMutationError::CommandInvalid(
                        "target_contribution_required",
                    ))?,
                secondary.as_deref(),
                execution.command.target_index,
            ),
            "group" => group_layout(
                &mut next_layout,
                target
                    .as_deref()
                    .ok_or(LayoutMutationError::CommandInvalid(
                        "target_contribution_required",
                    ))?,
                secondary
                    .as_deref()
                    .ok_or(LayoutMutationError::CommandInvalid(
                        "secondary_work_surface_required",
                    ))?,
                execution.command.target_layout_node_id.as_deref(),
            ),
            "ungroup" => ungroup_layout(
                &mut next_layout,
                target
                    .as_deref()
                    .ok_or(LayoutMutationError::CommandInvalid(
                        "target_contribution_required",
                    ))?,
            ),
            "split_horizontal" | "split_vertical" | "compare" => {
                let target = target
                    .as_deref()
                    .ok_or(LayoutMutationError::CommandInvalid(
                        "target_contribution_required",
                    ))?;
                let secondary = secondary
                    .as_deref()
                    .ok_or(LayoutMutationError::CommandInvalid(
                        "secondary_work_surface_required",
                    ))?;
                // A split cannot duplicate an already rendered contribution.
                // Remove an existing secondary from its canonical location,
                // then insert that same identity beside the target.
                if contains_contribution(&next_layout, secondary) {
                    if !remove_contribution(&mut next_layout, secondary) {
                        return Err(LayoutMutationError::NotApplicable);
                    }
                }
                split_layout(
                    &mut next_layout,
                    target,
                    secondary,
                    execution.command.target_layout_node_id.as_deref(),
                    execution.command.split_ratio.unwrap_or(0.67),
                    if execution.command.action == "split_vertical" {
                        "vertical"
                    } else {
                        "horizontal"
                    },
                )
            }
            "suspend_projection" | "close_projection" => {
                let target = target
                    .as_deref()
                    .ok_or(LayoutMutationError::CommandInvalid(
                        "target_contribution_required",
                    ))?;
                if projection.eligible_contributions.len() <= 1 {
                    return Err(LayoutMutationError::NotApplicable);
                }
                if !remove_contribution(&mut next_layout, target) {
                    return Err(LayoutMutationError::NotApplicable);
                }
                let next_eligible_ids = projection
                    .eligible_contributions
                    .iter()
                    .filter(|contribution| contribution.contribution_id != target)
                    .map(|contribution| contribution.contribution_id.clone())
                    .collect::<BTreeSet<_>>();
                projection
                    .eligible_contributions
                    .retain(|contribution| contribution.contribution_id != target);
                projection.operation_bindings.retain(|binding| {
                    binding
                        .get("target_contribution_id")
                        .and_then(Value::as_str)
                        != Some(target)
                });
                let next_revision = execution
                    .command
                    .expected_projection_revision
                    .checked_add(1)
                    .ok_or(LayoutMutationError::RevisionOverflow)?;
                projection.omission_diagnostics.push(OmissionDiagnostic {
                    contribution_id: target.to_owned(),
                    reason: "suspended".into(),
                    rule_revision: "layout-mutation:v1".into(),
                    projection_revision: next_revision,
                    canonical_input_refs: vec![],
                    details_ref: Some(format!("diagnostic:layout:{target}:suspended")),
                    observed_at: Utc::now().to_rfc3339(),
                });
                if projection.focused_semantic_target == eligible[target].semantic_binding_id {
                    projection.focused_semantic_target = projection
                        .eligible_contributions
                        .first()
                        .map(|contribution| contribution.semantic_binding_id.clone())
                        .unwrap_or_default();
                }
                validate_layout_tree(&next_layout, &next_eligible_ids)?;
                true
            }
            "open" | "pin" | "unpin" | "rehydrate" => {
                return Err(LayoutMutationError::UnsupportedAction(
                    execution.command.action.clone(),
                ));
            }
            other => return Err(LayoutMutationError::UnsupportedAction(other.to_owned())),
        };

        if !changed {
            return Err(LayoutMutationError::NotApplicable);
        }
        projection.layout_tree = next_layout;
        let eligible_ids = projection
            .eligible_contributions
            .iter()
            .map(|contribution| contribution.contribution_id.clone())
            .collect::<BTreeSet<_>>();
        validate_layout_tree(&projection.layout_tree, &eligible_ids)?;

        projection.projection_revision = projection
            .projection_revision
            .checked_add(1)
            .ok_or(LayoutMutationError::RevisionOverflow)?;
        projection.layout_revision = projection
            .layout_revision
            .checked_add(1)
            .ok_or(LayoutMutationError::RevisionOverflow)?;
        projection.durable_event_cursor = "event:pending".into();
        projection.resolved_at = Some(Utc::now().to_rfc3339());
        let scope_key = execution.command.scope.workstream.storage_key();
        let evidence_ref = format!("evidence:layout:{scope_key}:{}", projection.layout_revision);
        let receipt_ref = format!("receipt:layout:{scope_key}:{}", projection.layout_revision);
        projection.evidence_refs.push(evidence_ref.clone());
        projection.receipt_refs.push(receipt_ref.clone());
        projection.projection_digest = projection_digest(&projection)?;
        projection
            .validate_scope(&execution.command.scope)
            .map_err(LayoutMutationError::Scope)?;

        let event = super::model::CompositionEvent {
            event_id: layout_mutation_event_id(
                &execution.command.scope.workstream,
                &execution.command.idempotency_key,
            ),
            event_kind: "layout_changed".into(),
            scope: execution.command.scope.clone(),
            projection_revision: projection.projection_revision,
            layout_revision: projection.layout_revision,
            causation_id: Some(execution.command.idempotency_key.clone()),
            correlation_id: Some(execution.command.command_id.clone()),
            occurred_at: Utc::now().to_rfc3339(),
            payload: json!({
                "operation_id": LAYOUT_MUTATE_OPERATION,
                "action": execution.command.action,
                "target_contribution_id": target,
                "secondary_contribution_id": secondary,
                "target_layout_node_id": execution.command.target_layout_node_id,
                "request_digest": request_digest,
                "projection_digest": projection.projection_digest,
            }),
            evidence_refs: vec![evidence_ref.clone()],
            receipt_refs: vec![receipt_ref.clone()],
        };
        store
            .save_layout_mutation(
                &projection,
                execution.command.expected_projection_revision,
                &execution.command.command_id,
                &execution.command.idempotency_key,
                &request_digest,
                &event,
                &evidence_ref,
                &receipt_ref,
            )
            .map_err(LayoutMutationError::Store)
    }
}

fn validate_layout_mutation_execution(
    execution: &LayoutMutationExecution,
) -> Result<(), LayoutMutationError> {
    let command = &execution.command;
    command
        .scope
        .validate()
        .map_err(LayoutMutationError::Scope)?;
    if command.scope.attachment.is_none() {
        return Err(LayoutMutationError::CommandInvalid("attachment_required"));
    }
    execution
        .context
        .validate_for_workstream(&command.scope.workstream)?;
    if execution.context.attachment != command.scope.attachment {
        return Err(LayoutMutationError::Context(
            WorkstreamContextError::WorkstreamMismatch,
        ));
    }
    let expected_continuity = command.scope.continuity_id.clone().or_else(|| {
        command
            .scope
            .attachment
            .as_ref()
            .and_then(|attachment| attachment.continuity_id.clone())
    });
    if execution.context.continuity_id != expected_continuity {
        return Err(LayoutMutationError::Context(
            WorkstreamContextError::ContinuityMismatch,
        ));
    }
    let expected_binding = command.scope.workspace_binding_id.clone().or_else(|| {
        command
            .scope
            .attachment
            .as_ref()
            .map(|attachment| attachment.workspace_binding_id.clone())
    });
    if execution.context.workspace_binding_id != expected_binding {
        return Err(LayoutMutationError::Context(
            WorkstreamContextError::WorkspaceBindingMismatch,
        ));
    }
    if !has_layout_mutation_permission(&execution.permissions) {
        return Err(LayoutMutationError::PermissionDenied(
            LAYOUT_MUTATE_PERMISSION.into(),
        ));
    }
    if !is_layout_command_id(&command.command_id) {
        return Err(LayoutMutationError::CommandInvalid("command_id_invalid"));
    }
    if command.idempotency_key.trim().is_empty() {
        return Err(LayoutMutationError::CommandInvalid(
            "idempotency_key_missing",
        ));
    }
    if !ACTIONS.contains(&command.action.as_str()) {
        return Err(LayoutMutationError::CommandInvalid("action_invalid"));
    }
    if command
        .requested_at
        .as_deref()
        .is_some_and(|value| DateTime::parse_from_rfc3339(value).is_err())
    {
        return Err(LayoutMutationError::CommandInvalid("requested_at_invalid"));
    }
    if command
        .target_contribution_id
        .as_deref()
        .is_some_and(|value| !is_contribution_id(value))
    {
        return Err(LayoutMutationError::CommandInvalid(
            "target_contribution_id_invalid",
        ));
    }
    if command
        .target_layout_node_id
        .as_deref()
        .is_some_and(|value| value.trim().is_empty())
    {
        return Err(LayoutMutationError::CommandInvalid(
            "target_layout_node_id_invalid",
        ));
    }
    if command
        .split_ratio
        .is_some_and(|ratio| !ratio.is_finite() || !(0.1..=0.9).contains(&ratio))
    {
        return Err(LayoutMutationError::CommandInvalid("split_ratio_invalid"));
    }
    if command.action == "reorder"
        && command.target_index.is_none()
        && command.secondary_work_surface_id.is_none()
    {
        return Err(LayoutMutationError::CommandInvalid(
            "reorder_target_or_secondary_required",
        ));
    }
    Ok(())
}

fn validate_action_targets(
    command: &LayoutMutationCommand,
    target: Option<&str>,
    secondary: Option<&str>,
) -> Result<(), LayoutMutationError> {
    let target_required = matches!(
        command.action.as_str(),
        "focus"
            | "set_active_tab"
            | "reorder"
            | "group"
            | "ungroup"
            | "split_horizontal"
            | "split_vertical"
            | "compare"
            | "suspend_projection"
            | "close_projection"
    );
    if target_required && target.is_none() {
        return Err(LayoutMutationError::CommandInvalid(
            "target_contribution_required",
        ));
    }
    if matches!(
        command.action.as_str(),
        "group" | "split_horizontal" | "split_vertical" | "compare"
    ) && secondary.is_none()
    {
        return Err(LayoutMutationError::CommandInvalid(
            "secondary_work_surface_required",
        ));
    }
    if command.action == "resize_split"
        && command.target_layout_node_id.is_none()
        && target.is_none()
    {
        return Err(LayoutMutationError::CommandInvalid(
            "resize_target_required",
        ));
    }
    Ok(())
}

fn has_layout_mutation_permission(permissions: &BTreeSet<String>) -> bool {
    permissions.contains(LAYOUT_MUTATE_PERMISSION)
        || permissions.contains("mission_canvas:*")
        || permissions.contains("admin:*")
        || permissions.contains("*")
}

fn is_layout_command_id(value: &str) -> bool {
    let Some(suffix) = value.strip_prefix("layout-command:") else {
        return false;
    };
    !suffix.is_empty()
        && suffix.len() <= 160
        && suffix.chars().all(|value| {
            value.is_ascii_lowercase()
                || value.is_ascii_digit()
                || ". _:-".replace(' ', "").contains(value)
        })
}

fn is_contribution_id(value: &str) -> bool {
    let Some(suffix) = value.strip_prefix("contribution:") else {
        return false;
    };
    !suffix.is_empty()
        && suffix.len() <= 160
        && suffix
            .chars()
            .next()
            .is_some_and(|value| value.is_ascii_lowercase() || value.is_ascii_digit())
        && suffix.chars().all(|value| {
            value.is_ascii_lowercase() || value.is_ascii_digit() || "._:-".contains(value)
        })
}

fn layout_mutation_digest(
    execution: &LayoutMutationExecution,
) -> Result<String, serde_json::Error> {
    let value = json!({
        "context": execution.context,
        "command": execution.command,
        "permissions": execution.permissions,
    });
    let bytes = serde_json::to_vec(&value)?;
    Ok(format!("sha256:{}", hex::encode(Sha256::digest(bytes))))
}

fn layout_mutation_event_id(workstream: &WorkstreamKey, idempotency_key: &str) -> String {
    let owner = serde_json::to_vec(workstream).unwrap_or_default();
    let mut material = owner;
    material.extend_from_slice(idempotency_key.as_bytes());
    format!(
        "projection-event:layout:{}",
        hex::encode(Sha256::digest(material))
    )
}

fn eligible_contributions(
    projection: &ResolvedWorkspaceProjection,
) -> Result<BTreeMap<String, ResolvedContribution>, LayoutMutationError> {
    let mut eligible = BTreeMap::new();
    for contribution in &projection.eligible_contributions {
        if !is_contribution_id(&contribution.contribution_id) {
            return Err(LayoutMutationError::InvalidLayout(
                "eligible_contribution_id_invalid".into(),
            ));
        }
        if eligible
            .insert(contribution.contribution_id.clone(), contribution.clone())
            .is_some()
        {
            return Err(LayoutMutationError::InvalidLayout(
                "duplicate_eligible_contribution".into(),
            ));
        }
    }
    if eligible.is_empty() {
        return Err(LayoutMutationError::InvalidLayout(
            "no_eligible_contributions".into(),
        ));
    }
    Ok(eligible)
}

fn resolve_target_contribution(
    projection: &ResolvedWorkspaceProjection,
    target_contribution_id: Option<&str>,
    target_work_surface_id: Option<&WorkSurfaceId>,
) -> Result<Option<String>, LayoutMutationError> {
    if let Some(target) = target_contribution_id {
        if !projection
            .eligible_contributions
            .iter()
            .any(|contribution| contribution.contribution_id == target)
        {
            return Err(LayoutMutationError::UnknownContribution(target.into()));
        }
    }
    let from_surface = target_work_surface_id
        .map(|surface| resolve_work_surface_contribution(projection, surface))
        .transpose()?
        .flatten();
    if let (Some(target), Some(from_surface)) = (target_contribution_id, from_surface.as_deref()) {
        if target != from_surface {
            return Err(LayoutMutationError::CommandInvalid(
                "target_work_surface_mismatch",
            ));
        }
    }
    Ok(target_contribution_id.map(str::to_owned).or(from_surface))
}

fn resolve_work_surface_contribution(
    projection: &ResolvedWorkspaceProjection,
    surface: &WorkSurfaceId,
) -> Result<Option<String>, LayoutMutationError> {
    let requested = surface.as_str();
    let mut match_id = None;
    for contribution in &projection.eligible_contributions {
        let authority_surface = contribution
            .authority
            .get("work_surface_id")
            .and_then(Value::as_str);
        let data_surface = contribution
            .data_ref
            .as_object()
            .filter(|data| data.get("kind").and_then(Value::as_str) == Some("work_surface"))
            .and_then(|data| data.get("ref"))
            .and_then(Value::as_str);
        if authority_surface == Some(requested) || data_surface == Some(requested) {
            if match_id.is_some() {
                return Err(LayoutMutationError::InvalidLayout(
                    "duplicate_work_surface_contribution".into(),
                ));
            }
            match_id = Some(contribution.contribution_id.clone());
        }
    }
    if match_id.is_none() {
        return Err(LayoutMutationError::UnknownWorkSurface(requested.into()));
    }
    Ok(match_id)
}

fn validate_layout_tree(
    node: &Value,
    eligible: &BTreeSet<String>,
) -> Result<(), LayoutMutationError> {
    let mut nodes = BTreeSet::new();
    let mut contributions = BTreeSet::new();
    validate_layout_node(node, eligible, &mut nodes, &mut contributions)?;
    if contributions != *eligible {
        let missing = eligible
            .difference(&contributions)
            .next()
            .cloned()
            .unwrap_or_else(|| "unknown".into());
        return Err(LayoutMutationError::InvalidLayout(format!(
            "unplaced_contribution:{missing}"
        )));
    }
    Ok(())
}

fn validate_layout_node(
    node: &Value,
    eligible: &BTreeSet<String>,
    nodes: &mut BTreeSet<String>,
    contributions: &mut BTreeSet<String>,
) -> Result<(), LayoutMutationError> {
    let object = node
        .as_object()
        .ok_or_else(|| LayoutMutationError::InvalidLayout("layout_node_not_object".into()))?;
    let kind = object
        .get("kind")
        .and_then(Value::as_str)
        .ok_or_else(|| LayoutMutationError::InvalidLayout("layout_kind_missing".into()))?;
    let node_id = object
        .get("node_id")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty() && *value == value.trim())
        .ok_or_else(|| LayoutMutationError::InvalidLayout("layout_node_id_invalid".into()))?;
    if !nodes.insert(node_id.to_owned()) {
        return Err(LayoutMutationError::InvalidLayout(
            "duplicate_layout_node".into(),
        ));
    }
    match kind {
        "single" => {
            reject_fields(object, &["kind", "node_id", "contribution_id"])?;
            let contribution = required_string(object, "contribution_id")?;
            if !is_contribution_id(contribution) || !eligible.contains(contribution) {
                return Err(LayoutMutationError::UnknownContribution(
                    contribution.into(),
                ));
            }
            if !contributions.insert(contribution.into()) {
                return Err(LayoutMutationError::InvalidLayout(
                    "duplicate_layout_contribution".into(),
                ));
            }
        }
        "split" => {
            reject_fields(
                object,
                &["kind", "node_id", "orientation", "ratio", "children"],
            )?;
            let orientation = required_string(object, "orientation")?;
            if !matches!(orientation, "horizontal" | "vertical") {
                return Err(LayoutMutationError::InvalidLayout(
                    "split_orientation_invalid".into(),
                ));
            }
            let ratio = object
                .get("ratio")
                .and_then(Value::as_f64)
                .filter(|ratio| ratio.is_finite() && (0.1..=0.9).contains(ratio))
                .ok_or_else(|| LayoutMutationError::InvalidLayout("split_ratio_invalid".into()))?;
            let _ = ratio;
            let children = required_array(object, "children")?;
            if children.len() != 2 {
                return Err(LayoutMutationError::InvalidLayout(
                    "split_children_invalid".into(),
                ));
            }
            for child in children {
                validate_layout_node(child, eligible, nodes, contributions)?;
            }
        }
        "stack" | "grid" => {
            let allowed = if kind == "stack" {
                &["kind", "node_id", "children", "gap_token"][..]
            } else {
                &["kind", "node_id", "columns", "children", "gap_token"][..]
            };
            reject_fields(object, allowed)?;
            if let Some(gap) = object.get("gap_token") {
                if gap
                    .as_str()
                    .is_none_or(|value| value.trim().is_empty() || value != value.trim())
                {
                    return Err(LayoutMutationError::InvalidLayout(
                        "gap_token_invalid".into(),
                    ));
                }
            }
            if kind == "grid" {
                let columns = object
                    .get("columns")
                    .and_then(Value::as_u64)
                    .filter(|columns| (1..=12).contains(columns))
                    .ok_or_else(|| {
                        LayoutMutationError::InvalidLayout("grid_columns_invalid".into())
                    })?;
                let _ = columns;
            }
            let children = required_array(object, "children")?;
            if children.is_empty() {
                return Err(LayoutMutationError::InvalidLayout(
                    "layout_children_empty".into(),
                ));
            }
            for child in children {
                validate_layout_node(child, eligible, nodes, contributions)?;
            }
        }
        "tabs" => {
            reject_fields(
                object,
                &[
                    "kind",
                    "node_id",
                    "contribution_ids",
                    "active_contribution_id",
                ],
            )?;
            let ids = required_array(object, "contribution_ids")?;
            if ids.is_empty() {
                return Err(LayoutMutationError::InvalidLayout(
                    "tab_contributions_empty".into(),
                ));
            }
            let active = required_string(object, "active_contribution_id")?;
            let mut local = BTreeSet::new();
            for id in ids {
                let id = id.as_str().ok_or_else(|| {
                    LayoutMutationError::InvalidLayout("tab_contribution_invalid".into())
                })?;
                if !is_contribution_id(id) || !eligible.contains(id) || !local.insert(id) {
                    return Err(LayoutMutationError::UnknownContribution(id.into()));
                }
                if !contributions.insert(id.into()) {
                    return Err(LayoutMutationError::InvalidLayout(
                        "duplicate_layout_contribution".into(),
                    ));
                }
            }
            if !local.contains(active) {
                return Err(LayoutMutationError::InvalidLayout(
                    "active_tab_invalid".into(),
                ));
            }
        }
        "inspector" => {
            reject_fields(
                object,
                &[
                    "kind",
                    "node_id",
                    "side",
                    "primary",
                    "inspector_contribution_ids",
                    "span",
                ],
            )?;
            let side = required_string(object, "side")?;
            if !matches!(side, "start" | "end") {
                return Err(LayoutMutationError::InvalidLayout(
                    "inspector_side_invalid".into(),
                ));
            }
            if let Some(span) = object.get("span") {
                if span.as_u64().is_none_or(|span| !(1..=6).contains(&span)) {
                    return Err(LayoutMutationError::InvalidLayout(
                        "inspector_span_invalid".into(),
                    ));
                }
            }
            let primary = object.get("primary").ok_or_else(|| {
                LayoutMutationError::InvalidLayout("inspector_primary_missing".into())
            })?;
            validate_layout_node(primary, eligible, nodes, contributions)?;
            let ids = required_array(object, "inspector_contribution_ids")?;
            if ids.is_empty() {
                return Err(LayoutMutationError::InvalidLayout(
                    "inspector_contributions_empty".into(),
                ));
            }
            let mut local = BTreeSet::new();
            for id in ids {
                let id = id.as_str().ok_or_else(|| {
                    LayoutMutationError::InvalidLayout("inspector_contribution_invalid".into())
                })?;
                if !is_contribution_id(id) || !eligible.contains(id) || !local.insert(id) {
                    return Err(LayoutMutationError::UnknownContribution(id.into()));
                }
                if !contributions.insert(id.into()) {
                    return Err(LayoutMutationError::InvalidLayout(
                        "duplicate_layout_contribution".into(),
                    ));
                }
            }
        }
        _ => {
            return Err(LayoutMutationError::InvalidLayout(
                "layout_kind_unknown".into(),
            ));
        }
    }
    Ok(())
}

fn reject_fields(object: &Map<String, Value>, allowed: &[&str]) -> Result<(), LayoutMutationError> {
    if object.keys().any(|key| !allowed.contains(&key.as_str())) {
        return Err(LayoutMutationError::InvalidLayout(
            "layout_unknown_field".into(),
        ));
    }
    Ok(())
}

fn required_string<'a>(
    object: &'a Map<String, Value>,
    field: &str,
) -> Result<&'a str, LayoutMutationError> {
    object
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty() && *value == value.trim())
        .ok_or_else(|| LayoutMutationError::InvalidLayout(format!("{field}_invalid")))
}

fn required_array<'a>(
    object: &'a Map<String, Value>,
    field: &str,
) -> Result<&'a Vec<Value>, LayoutMutationError> {
    object
        .get(field)
        .and_then(Value::as_array)
        .ok_or_else(|| LayoutMutationError::InvalidLayout(format!("{field}_invalid")))
}

fn contains_contribution(node: &Value, target: &str) -> bool {
    let Some(object) = node.as_object() else {
        return false;
    };
    match object.get("kind").and_then(Value::as_str) {
        Some("single") => object.get("contribution_id").and_then(Value::as_str) == Some(target),
        Some("tabs") => object
            .get("contribution_ids")
            .and_then(Value::as_array)
            .is_some_and(|ids| ids.iter().any(|id| id.as_str() == Some(target))),
        Some("inspector") => {
            object
                .get("inspector_contribution_ids")
                .and_then(Value::as_array)
                .is_some_and(|ids| ids.iter().any(|id| id.as_str() == Some(target)))
                || object
                    .get("primary")
                    .is_some_and(|primary| contains_contribution(primary, target))
        }
        Some("split") | Some("stack") | Some("grid") => object
            .get("children")
            .and_then(Value::as_array)
            .is_some_and(|children| {
                children
                    .iter()
                    .any(|child| contains_contribution(child, target))
            }),
        _ => false,
    }
}

fn set_active_tab(node: &mut Value, target: &str) -> bool {
    let Some(object) = node.as_object_mut() else {
        return false;
    };
    if object.get("kind").and_then(Value::as_str) == Some("tabs") {
        let contains = object
            .get("contribution_ids")
            .and_then(Value::as_array)
            .is_some_and(|ids| ids.iter().any(|id| id.as_str() == Some(target)));
        if contains && object.get("active_contribution_id").and_then(Value::as_str) != Some(target)
        {
            object.insert("active_contribution_id".into(), json!(target));
            return true;
        }
        return false;
    }
    if let Some(children) = object.get_mut("children").and_then(Value::as_array_mut) {
        for child in children {
            if set_active_tab(child, target) {
                return true;
            }
        }
    }
    object
        .get_mut("primary")
        .is_some_and(|primary| set_active_tab(primary, target))
}

fn resize_split(
    node: &mut Value,
    target_node_id: Option<&str>,
    target_contribution_id: Option<&str>,
    ratio: f64,
) -> bool {
    let contribution_matches =
        target_contribution_id.is_none_or(|target| contains_contribution(node, target));
    let Some(object) = node.as_object_mut() else {
        return false;
    };
    if object.get("kind").and_then(Value::as_str) == Some("split") {
        let node_matches = target_node_id
            .is_none_or(|target| object.get("node_id").and_then(Value::as_str) == Some(target));
        if node_matches && contribution_matches {
            object.insert("ratio".into(), json!(ratio));
            return true;
        }
    }
    if let Some(children) = object.get_mut("children").and_then(Value::as_array_mut) {
        for child in children {
            if resize_split(child, target_node_id, target_contribution_id, ratio) {
                return true;
            }
        }
    }
    object
        .get_mut("primary")
        .is_some_and(|primary| resize_split(primary, target_node_id, target_contribution_id, ratio))
}

fn reorder_layout(
    node: &mut Value,
    target: &str,
    secondary: Option<&str>,
    target_index: Option<u32>,
) -> bool {
    let Some(object) = node.as_object_mut() else {
        return false;
    };
    if object.get("kind").and_then(Value::as_str) == Some("tabs") {
        if let Some(ids) = object
            .get_mut("contribution_ids")
            .and_then(Value::as_array_mut)
        {
            if let Some(index) = target_index {
                if let Some(current) = ids.iter().position(|id| id.as_str() == Some(target)) {
                    let index = index as usize;
                    if index < ids.len() && current != index {
                        let item = ids.remove(current);
                        ids.insert(index, item);
                        return true;
                    }
                }
            } else if let Some(secondary) = secondary {
                let left = ids.iter().position(|id| id.as_str() == Some(target));
                let right = ids.iter().position(|id| id.as_str() == Some(secondary));
                if let (Some(left), Some(right)) = (left, right) {
                    if left != right {
                        ids.swap(left, right);
                        return true;
                    }
                }
            }
        }
    }
    if let Some(children) = object.get_mut("children").and_then(Value::as_array_mut) {
        if let Some(index) = target_index {
            if let Some(current) = children
                .iter()
                .position(|child| contains_contribution(child, target))
            {
                let index = index as usize;
                if index < children.len() && current != index {
                    let item = children.remove(current);
                    children.insert(index, item);
                    return true;
                }
            }
        } else if let Some(secondary) = secondary {
            let left = children
                .iter()
                .position(|child| contains_contribution(child, target));
            let right = children
                .iter()
                .position(|child| contains_contribution(child, secondary));
            if let (Some(left), Some(right)) = (left, right) {
                if left != right {
                    children.swap(left, right);
                    return true;
                }
            }
        }
        for child in children {
            if reorder_layout(child, target, secondary, target_index) {
                return true;
            }
        }
    }
    if let Some(ids) = object
        .get_mut("inspector_contribution_ids")
        .and_then(Value::as_array_mut)
    {
        if let Some(secondary) = secondary {
            let left = ids.iter().position(|id| id.as_str() == Some(target));
            let right = ids.iter().position(|id| id.as_str() == Some(secondary));
            if let (Some(left), Some(right)) = (left, right) {
                if left != right {
                    ids.swap(left, right);
                    return true;
                }
            }
        }
    }
    object
        .get_mut("primary")
        .is_some_and(|primary| reorder_layout(primary, target, secondary, target_index))
}

fn group_layout(
    node: &mut Value,
    target: &str,
    secondary: &str,
    target_node_id: Option<&str>,
) -> bool {
    let Some(object) = node.as_object_mut() else {
        return false;
    };
    if object.get("kind").and_then(Value::as_str) == Some("tabs") {
        return false;
    }
    if let Some(children) = object.get_mut("children").and_then(Value::as_array_mut) {
        let target_index = children.iter().position(|child| {
            child.get("kind").and_then(Value::as_str) == Some("single")
                && child.get("contribution_id").and_then(Value::as_str) == Some(target)
        });
        let secondary_index = children.iter().position(|child| {
            child.get("kind").and_then(Value::as_str) == Some("single")
                && child.get("contribution_id").and_then(Value::as_str) == Some(secondary)
        });
        if let (Some(target_index), Some(secondary_index)) = (target_index, secondary_index) {
            if target_index == secondary_index {
                return false;
            }
            let target_node = children[target_index]
                .get("node_id")
                .and_then(Value::as_str)
                .unwrap_or("layout:target");
            let tabs_id = target_node_id
                .filter(|value| !value.trim().is_empty())
                .unwrap_or(target_node);
            let tabs_id = format!("{tabs_id}:tabs");
            let (first, second) = if target_index < secondary_index {
                (target_index, secondary_index)
            } else {
                (secondary_index, target_index)
            };
            children.remove(second);
            children.remove(first);
            children.insert(
                first,
                json!({
                    "kind": "tabs",
                    "node_id": tabs_id,
                    "contribution_ids": [target, secondary],
                    "active_contribution_id": target
                }),
            );
            return true;
        }
        for child in children {
            if group_layout(child, target, secondary, target_node_id) {
                return true;
            }
        }
    }
    object
        .get_mut("primary")
        .is_some_and(|primary| group_layout(primary, target, secondary, target_node_id))
}

fn split_layout(
    node: &mut Value,
    target: &str,
    secondary: &str,
    target_node_id: Option<&str>,
    ratio: f64,
    orientation: &str,
) -> bool {
    let original = node.clone();
    let Some(object) = node.as_object_mut() else {
        return false;
    };
    if object.get("kind").and_then(Value::as_str) == Some("single")
        && object.get("contribution_id").and_then(Value::as_str) == Some(target)
    {
        let original_node_id = object
            .get("node_id")
            .and_then(Value::as_str)
            .unwrap_or("layout:target");
        let split_id = target_node_id
            .filter(|value| !value.trim().is_empty())
            .map(str::to_owned)
            .unwrap_or_else(|| format!("{original_node_id}:split"));
        *node = json!({
            "kind": "split",
            "node_id": split_id,
            "orientation": orientation,
            "ratio": ratio,
            "children": [
                original,
                {"kind":"single", "node_id":format!("{original_node_id}:secondary"), "contribution_id":secondary}
            ]
        });
        return true;
    }
    if let Some(children) = object.get_mut("children").and_then(Value::as_array_mut) {
        for child in children {
            if split_layout(child, target, secondary, target_node_id, ratio, orientation) {
                return true;
            }
        }
    }
    object.get_mut("primary").is_some_and(|primary| {
        split_layout(
            primary,
            target,
            secondary,
            target_node_id,
            ratio,
            orientation,
        )
    })
}

fn ungroup_layout(node: &mut Value, target: &str) -> bool {
    let Some(object) = node.as_object_mut() else {
        return false;
    };
    if object.get("kind").and_then(Value::as_str) == Some("tabs") {
        let ids = object
            .get("contribution_ids")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        if ids.iter().any(|id| id.as_str() == Some(target)) {
            let node_id = object
                .get("node_id")
                .and_then(Value::as_str)
                .unwrap_or("layout:tabs");
            let children = ids
                .iter()
                .enumerate()
                .map(|(index, id)| {
                    json!({
                        "kind": "single",
                        "node_id": format!("{node_id}:item:{index}"),
                        "contribution_id": id
                    })
                })
                .collect::<Vec<_>>();
            *node = json!({"kind":"stack", "node_id":format!("{node_id}:ungrouped"), "children":children});
            return true;
        }
    }
    if let Some(children) = object.get_mut("children").and_then(Value::as_array_mut) {
        for child in children {
            if ungroup_layout(child, target) {
                return true;
            }
        }
    }
    object
        .get_mut("primary")
        .is_some_and(|primary| ungroup_layout(primary, target))
}

fn remove_contribution(node: &mut Value, target: &str) -> bool {
    let Some(object) = node.as_object_mut() else {
        return false;
    };
    match object.get("kind").and_then(Value::as_str) {
        Some("tabs") => {
            let (remaining_len, first_remaining) = {
                let Some(ids) = object
                    .get_mut("contribution_ids")
                    .and_then(Value::as_array_mut)
                else {
                    return false;
                };
                let before = ids.len();
                ids.retain(|id| id.as_str() != Some(target));
                if ids.len() == before {
                    return false;
                }
                (ids.len(), ids.first().cloned())
            };
            if remaining_len == 1 {
                let Some(id) = first_remaining else {
                    return false;
                };
                let node_id = object
                    .get("node_id")
                    .cloned()
                    .unwrap_or_else(|| json!("layout:tab-item"));
                *node = json!({"kind":"single", "node_id":node_id, "contribution_id":id});
            } else {
                let active = object.get("active_contribution_id").and_then(Value::as_str);
                if active == Some(target) {
                    if let Some(first_remaining) = first_remaining {
                        object.insert("active_contribution_id".into(), first_remaining);
                    }
                }
            }
            true
        }
        Some("inspector") => {
            if let Some(ids) = object
                .get_mut("inspector_contribution_ids")
                .and_then(Value::as_array_mut)
            {
                let before = ids.len();
                ids.retain(|id| id.as_str() != Some(target));
                if ids.len() != before {
                    if ids.is_empty() {
                        if let Some(primary) = object.remove("primary") {
                            *node = primary;
                        }
                    }
                    return true;
                }
            }
            object
                .get_mut("primary")
                .is_some_and(|primary| remove_contribution(primary, target))
        }
        Some("split") | Some("stack") | Some("grid") => {
            let Some(children) = object.get_mut("children").and_then(Value::as_array_mut) else {
                return false;
            };
            for index in 0..children.len() {
                if !contains_contribution(&children[index], target) {
                    continue;
                }
                if children[index].get("kind").and_then(Value::as_str) == Some("single") {
                    children.remove(index);
                    normalize_node(node);
                    return true;
                }
                if remove_contribution(&mut children[index], target) {
                    normalize_node(node);
                    return true;
                }
            }
            false
        }
        Some("single") | _ => false,
    }
}

fn normalize_node(node: &mut Value) {
    let Some(object) = node.as_object_mut() else {
        return;
    };
    if let Some(children) = object.get_mut("children").and_then(Value::as_array_mut) {
        for child in children.iter_mut() {
            normalize_node(child);
        }
        if children.len() == 1 {
            let child = children[0].clone();
            *node = child;
            return;
        }
    }
    if object.get("kind").and_then(Value::as_str) == Some("tabs") {
        if let Some(ids) = object.get("contribution_ids").and_then(Value::as_array) {
            if ids.len() == 1 {
                let node_id = object
                    .get("node_id")
                    .cloned()
                    .unwrap_or_else(|| json!("layout:tab-item"));
                *node = json!({"kind":"single", "node_id":node_id, "contribution_id":ids[0]});
            }
        }
    } else if object.get("kind").and_then(Value::as_str) == Some("inspector")
        && object
            .get("inspector_contribution_ids")
            .and_then(Value::as_array)
            .is_some_and(Vec::is_empty)
    {
        if let Some(primary) = object.get("primary").cloned() {
            *node = primary;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layout() -> Value {
        json!({
            "kind":"tabs",
            "node_id":"layout:tabs",
            "contribution_ids":["contribution:pi","contribution:inspector"],
            "active_contribution_id":"contribution:pi"
        })
    }

    #[test]
    fn canonical_layout_validation_rejects_unknown_contribution_before_mutation() {
        let result = validate_layout_tree(
            &json!({"kind":"single","node_id":"layout:root","contribution_id":"contribution:unknown"}),
            &BTreeSet::from(["contribution:pi".to_owned()]),
        );
        assert!(
            matches!(result, Err(LayoutMutationError::UnknownContribution(value)) if value == "contribution:unknown")
        );
    }

    #[test]
    fn tab_mutation_is_recursive_and_preserves_canonical_ids() {
        let mut value = json!({
            "kind":"stack","node_id":"layout:root","children":[
                {"kind":"single","node_id":"layout:one","contribution_id":"contribution:one"},
                layout()
            ]
        });
        assert!(set_active_tab(&mut value, "contribution:inspector"));
        assert_eq!(
            value["children"][1]["active_contribution_id"],
            "contribution:inspector"
        );
    }

    #[test]
    fn malformed_or_foreign_layout_never_gets_reflowed() {
        let value = json!({
            "kind":"split","node_id":"layout:root","orientation":"horizontal","ratio":0.5,
            "children":[
                {"kind":"single","node_id":"layout:one","contribution_id":"contribution:one"},
                {"kind":"single","node_id":"layout:foreign","contribution_id":"contribution:foreign"}
            ]
        });
        let before = value.clone();
        let result = validate_layout_tree(&value, &BTreeSet::from(["contribution:one".to_owned()]));
        assert!(result.is_err());
        assert_eq!(value, before);
    }

    #[test]
    fn ungroup_expands_all_tabs_without_dropping_contributions() {
        let mut value = layout();
        assert!(ungroup_layout(&mut value, "contribution:inspector"));
        assert_eq!(value["kind"], "stack");
        assert_eq!(value["children"].as_array().unwrap().len(), 2);
    }
}
